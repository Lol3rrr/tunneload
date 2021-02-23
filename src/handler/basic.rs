use crate::acceptors::traits::Sender;
use crate::handler::traits::Handler;
use crate::http::{streaming_parser::RespParser, Headers, Request, Response, StatusCode};
use crate::rules::ReadManager;

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use lazy_static::lazy_static;
use prometheus::Registry;

use log::error;

lazy_static! {
    static ref HANDLE_TIME: prometheus::Histogram =
        prometheus::Histogram::with_opts(prometheus::HistogramOpts::new(
            "basic_handling",
            "The Time, in seconds, it takes for a request to be fully handled"
        ))
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
        reg.register(Box::new(HANDLE_TIME.clone())).unwrap();
        reg.register(Box::new(SERVICE_REQ_VEC.clone())).unwrap();

        Self {
            rules: rules_manager,
        }
    }

    async fn not_found<T>(sender: T)
    where
        T: Sender + Send + Sync,
    {
        let response = Response::new(
            "HTTP/1.1",
            StatusCode::NotFound,
            Headers::new(),
            "Not Found".as_bytes(),
        );
        let (resp_header, resp_body) = response.serialize();
        let resp_header_length = resp_header.len();
        sender.send(resp_header, resp_header_length).await;
        let resp_body_length = resp_body.len();
        sender.send(resp_body.to_vec(), resp_body_length).await;
    }

    async fn service_unavailable<T>(sender: T)
    where
        T: Sender + Send + Sync,
    {
        let response = Response::new(
            "HTTP/1.1",
            StatusCode::ServiceUnavailable,
            Headers::new(),
            "Service Unavailable".as_bytes(),
        );
        let (resp_header, resp_body) = response.serialize();
        let resp_header_length = resp_header.len();
        sender.send(resp_header, resp_header_length).await;
        let resp_body_length = resp_body.len();
        sender.send(resp_body.to_vec(), resp_body_length).await;
    }

    async fn internal_server_error<T>(sender: T)
    where
        T: Sender + Send + Sync,
    {
        let response = Response::new(
            "HTTP/1.1",
            StatusCode::InternalServerError,
            Headers::new(),
            "Internal Server Error".as_bytes(),
        );
        let (resp_header, resp_body) = response.serialize();
        let resp_header_length = resp_header.len();
        sender.send(resp_header, resp_header_length).await;
        let resp_body_length = resp_body.len();
        sender.send(resp_body.to_vec(), resp_body_length).await;
    }
}

#[async_trait]
impl Handler for BasicHandler {
    async fn handle<T>(&self, id: u32, request: Request<'_>, sender: T)
    where
        T: Sender + Send + Sync,
    {
        let handle_timer = HANDLE_TIME.start_timer();

        let matched = match self.rules.match_req(&request) {
            Some(m) => m,
            None => {
                error!("[{}] No Rule matched the Request", id);
                Self::not_found(sender).await;
                return;
            }
        };

        SERVICE_REQ_VEC
            .get_metric_with_label_values(&[matched.name()])
            .unwrap()
            .inc();

        let mut out_req = request;
        matched.apply_middlewares_req(&mut out_req);
        let mut connection = match matched.service().connect().await {
            Some(a) => a,
            None => {
                error!("[{}] Connecting to Service", id);
                Self::service_unavailable(sender).await;
                return;
            }
        };

        let (serialized_headers, serialized_body) = out_req.serialize();
        match connection.write_all(&serialized_headers).await {
            Ok(_) => {}
            Err(e) => {
                error!("[{}] Writing Data to connection: {}", id, e);
                Self::internal_server_error(sender).await;
                return;
            }
        };
        match connection.write_all(serialized_body).await {
            Ok(_) => {}
            Err(e) => {
                error!("[{}] Writing Data to connection: {}", id, e);
                Self::internal_server_error(sender).await;
                return;
            }
        };

        let mut response_parser = RespParser::new_capacity(4096);
        let mut read_buffer: [u8; 2048] = [0; 2048];
        loop {
            match connection.read(&mut read_buffer).await {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                    response_parser.block_parse(&read_buffer[0..n]);
                }
                Err(e) => {
                    error!("[{}] Reading from Connection: {}", id, e);
                    Self::internal_server_error(sender).await;
                    return;
                }
            };
        }

        let mut response = match response_parser.finish() {
            Some(r) => r,
            None => {
                error!("Parsing Response");
                Self::internal_server_error(sender).await;
                return;
            }
        };

        matched.apply_middlewares_resp(&mut response);

        let (resp_header, resp_body) = response.serialize();
        let resp_header_length = resp_header.len();
        sender.send(resp_header, resp_header_length).await;
        let resp_body_length = resp_body.len();
        sender.send(resp_body.to_vec(), resp_body_length).await;

        handle_timer.observe_duration();
    }
}
