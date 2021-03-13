use crate::rules::ReadManager;
use crate::{
    acceptors::traits::{Receiver, Sender},
    forwarder::Forwarder,
};
use crate::{forwarder::ServiceConnection, handler::traits::Handler};

use stream_httparse::streaming_parser::{ReqParser, RespParser};

use async_trait::async_trait;

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
    static ref STATUS_CODES_VEC: prometheus::IntCounterVec = prometheus::IntCounterVec::new(
        prometheus::Opts::new("status_codes", "The StatusCodes returned by each service"),
        &["service", "status_code"]
    )
    .unwrap();
}

#[derive(Clone)]
pub struct BasicHandler<F>
where
    F: Forwarder,
{
    rules: ReadManager,
    forwarder: F,
}

impl<F> BasicHandler<F>
where
    F: Forwarder,
{
    pub fn new(rules_manager: ReadManager, forwarder: F, registry: Option<Registry>) -> Self {
        if let Some(reg) = registry {
            reg.register(Box::new(HANDLE_TIME_VEC.clone())).unwrap();
            reg.register(Box::new(SERVICE_REQ_VEC.clone())).unwrap();
            reg.register(Box::new(STATUS_CODES_VEC.clone())).unwrap();
        }

        Self {
            rules: rules_manager,
            forwarder,
        }
    }
}

#[async_trait]
impl<F> Handler for BasicHandler<F>
where
    F: Forwarder + Send + Sync,
{
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
        let mut resp_parser = RespParser::new_capacity(2048);

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
                        error!("[{}] Received Invalid request", id);
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
            let middlewares = matched.get_middleware_list();

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
            // If a middleware decided that this request should not be processed
            // anymore and instead a certain Response needs to be send to the
            // Client first, sends the given Response to the client and moves
            // on from this request
            if let Some(mid_resp) = middlewares.apply_middlewares_req(&mut out_req) {
                let (resp_header, resp_body) = mid_resp.serialize();
                let resp_header_length = resp_header.len();
                sender.send(resp_header, resp_header_length).await;
                let resp_body_length = resp_body.len();
                sender.send(resp_body.to_vec(), resp_body_length).await;

                handle_timer.observe_duration();

                req_parser.clear();
                resp_parser.clear();

                continue;
            }

            let mut connection = match self.forwarder.create_con(&matched).await {
                Some(c) => c,
                None => {
                    error!("[{}] Connecting to Service", id);
                    error_messages::service_unavailable(sender).await;
                    return;
                }
            };

            if let Err(e) = connection.write_req(&out_req).await {
                error!("[{}] Sending Request to Backend-Service: {}", id, e);
                error_messages::internal_server_error(sender).await;
                return;
            }

            let (mut response, left_over_buffer) =
                match response::receive(id, &mut resp_parser, &mut connection, &mut resp_buf).await
                {
                    Some(resp) => resp,
                    None => {
                        error_messages::internal_server_error(sender).await;
                        return;
                    }
                };

            middlewares.apply_middlewares_resp(&out_req, &mut response);

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

            STATUS_CODES_VEC
                .get_metric_with_label_values(&[rule_name, response.status_code().serialize()])
                .unwrap()
                .inc();

            // Clearing the Parser and therefore making it ready
            // parse a new Request without needing to allocate
            // another block
            req_parser.clear();
            resp_parser.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use rules::Service;

    use super::*;

    use crate::acceptors::mocks::Sender as MockSender;
    use crate::forwarder::mocks::Forwarder as MockForwarder;
    use crate::forwarder::mocks::ServiceConnection as MockServiceConnection;
    use crate::rules;
    use crate::{
        acceptors::mocks::Receiver as MockReceiver,
        general::Shared,
        rules::{Matcher, Rule},
    };

    #[tokio::test]
    async fn basic_handle_valid() {
        let mut tmp_service_con = MockServiceConnection::new();
        tmp_service_con.add_chunk("HTTP/1.1 200 OK\r\n\r\n".as_bytes().to_vec());
        let tmp_forwarder = MockForwarder::new(tmp_service_con);

        let mut receiver = MockReceiver::new();
        receiver.add_chunk("GET /api/test/ HTTP/1.1\r\n\r\n".as_bytes().to_vec());
        let mut sender = MockSender::new();

        let (read, write) = rules::new();
        write.add_rule(Rule::new(
            "test-rule".to_owned(),
            12,
            Matcher::PathPrefix("/api".to_owned()),
            vec![],
            Shared::new(Service::new("test-service".to_owned(), vec![])),
        ));

        let handler = BasicHandler::new(read.clone(), tmp_forwarder, None);

        handler.handle(12, &mut receiver, &mut sender).await;

        assert_eq!(
            Ok("HTTP/1.1 200 OK\r\n\r\n".to_owned()),
            String::from_utf8(sender.get_combined_data())
        );
    }

    #[tokio::test]
    async fn basic_handle_invalid_no_rules_match() {
        let mut tmp_service_con = MockServiceConnection::new();
        tmp_service_con.add_chunk("HTTP/1.1 200 OK\r\n\r\n".as_bytes().to_vec());
        let tmp_forwarder = MockForwarder::new(tmp_service_con);

        let mut receiver = MockReceiver::new();
        receiver.add_chunk("GET /test/ HTTP/1.1\r\n\r\n".as_bytes().to_vec());
        let mut sender = MockSender::new();

        let (read, write) = rules::new();
        write.add_rule(Rule::new(
            "test-rule".to_owned(),
            12,
            Matcher::PathPrefix("/api".to_owned()),
            vec![],
            Shared::new(Service::new("test-service".to_owned(), vec![])),
        ));

        let handler = BasicHandler::new(read.clone(), tmp_forwarder, None);

        handler.handle(12, &mut receiver, &mut sender).await;

        assert_eq!(
            Ok("HTTP/1.1 404 Not Found\r\n\r\nNot Found".to_owned()),
            String::from_utf8(sender.get_combined_data())
        );
    }
}
