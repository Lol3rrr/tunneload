use crate::acceptors::traits::{Receiver, Sender};
use crate::handler::traits::Handler;
use crate::http::streaming_parser::{ReqParser, RespParser};
use crate::rules::ReadManager;

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;

use lazy_static::lazy_static;
use prometheus::Registry;

use log::error;

mod chunks;
mod error_messages;
mod request;
mod response;

lazy_static! {
    static ref HANDLE_TIME_VEC: prometheus::HistogramVec = prometheus::HistogramVec::new(
        prometheus::HistogramOpts::new(
            "basic_handling",
            "The Time, in seconds, it takes for a request to be fully handled"
        ),
        &["service"]
    )
    .unwrap();
    static ref SERVICE_REQ_VEC: prometheus::IntCounterVec = prometheus::IntCounterVec::new(
        prometheus::Opts::new("service_reqs", "The Requests going to each service"),
        &["service"]
    )
    .unwrap();
}

#[derive(Clone)]
pub struct BasicHandler {
    rules: ReadManager,
}

impl BasicHandler {
    pub fn new(rules_manager: ReadManager, reg: Registry) -> Self {
        reg.register(Box::new(HANDLE_TIME_VEC.clone())).unwrap();
        reg.register(Box::new(SERVICE_REQ_VEC.clone())).unwrap();

        Self {
            rules: rules_manager,
        }
    }
}

#[async_trait]
impl Handler for BasicHandler {
    async fn handle<R, S>(&self, id: u32, receiver: &mut R, sender: &mut S)
    where
        R: Receiver + Send,
        S: Sender + Send,
    {
        let mut keep_alive = true;

        let mut req_buf = [0; 2048];
        let mut req_offset = 0;
        let mut req_parser = ReqParser::new_capacity(2048);

        let mut resp_buf = [0; 2048];
        let mut resp_parser = RespParser::new_capacity(1024);

        while keep_alive {
            let request =
                match request::receive(id, &mut req_parser, receiver, &mut req_buf, req_offset)
                    .await
                {
                    Some((r, n_offset)) => {
                        req_offset = n_offset;
                        r
                    }
                    None => {
                        error_messages::bad_request(sender).await;
                        return;
                    }
                };
            keep_alive = request.is_keep_alive();

            let matched = match self.rules.match_req(&request) {
                Some(m) => m,
                None => {
                    error!("[{}] No Rule matched the Request", id);
                    error_messages::not_found(sender).await;
                    return;
                }
            };

            // Some metrics related stuff
            let rule_name = matched.name();
            let handle_timer = HANDLE_TIME_VEC
                .get_metric_with_label_values(&[rule_name])
                .unwrap()
                .start_timer();
            SERVICE_REQ_VEC
                .get_metric_with_label_values(&[rule_name])
                .unwrap()
                .inc();

            let mut out_req = request;
            matched.apply_middlewares_req(&mut out_req);
            let mut connection = match matched.service().connect().await {
                Some(a) => a,
                None => {
                    error!("[{}] Connecting to Service", id);
                    error_messages::service_unavailable(sender).await;
                    return;
                }
            };

            let (serialized_headers, serialized_body) = out_req.serialize();
            match connection.write_all(&serialized_headers).await {
                Ok(_) => {}
                Err(e) => {
                    error!("[{}] Writing Data to connection: {}", id, e);
                    error_messages::internal_server_error(sender).await;
                    return;
                }
            };
            match connection.write_all(serialized_body).await {
                Ok(_) => {}
                Err(e) => {
                    error!("[{}] Writing Data to connection: {}", id, e);
                    error_messages::internal_server_error(sender).await;
                    return;
                }
            };

            let (mut response, left_over_buffer) =
                match response::receive(id, &mut resp_parser, &mut connection, &mut resp_buf).await
                {
                    Some(resp) => resp,
                    None => {
                        error_messages::internal_server_error(sender).await;
                        return;
                    }
                };

            matched.apply_middlewares_resp(&out_req, &mut response);

            let (resp_header, resp_body) = response.serialize();
            let resp_header_length = resp_header.len();
            sender.send(resp_header, resp_header_length).await;

            // First assumes that it is NOT chunked and should
            // just send the data normally
            if !response.is_chunked() {
                let resp_body_length = resp_body.len();
                sender.send(resp_body.to_vec(), resp_body_length).await;
            } else {
                chunks::forward(id, &mut connection, sender, &mut resp_buf, left_over_buffer).await;
            }

            handle_timer.observe_duration();

            // Clearing the Parser and therefore making it ready
            // parse a new Request without needing to allocate
            // another block
            req_parser.clear();
            resp_parser.clear();
        }
    }
}
