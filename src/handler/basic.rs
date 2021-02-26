use crate::acceptors::traits::{Receiver, Sender};
use crate::handler::traits::Handler;
use crate::http::{
    streaming_parser::ReqParser, streaming_parser::RespParser, Headers, Response, StatusCode,
};
use crate::rules::ReadManager;

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use lazy_static::lazy_static;
use prometheus::Registry;

use log::error;

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

    async fn bad_request<T>(sender: &mut T)
    where
        T: Sender,
    {
        let response = Response::new(
            "HTTP/1.1",
            StatusCode::BadRequest,
            Headers::new(),
            "Bad Request".as_bytes().to_vec(),
        );
        let (resp_header, resp_body) = response.serialize();
        let resp_header_length = resp_header.len();
        sender.send(resp_header, resp_header_length).await;
        let resp_body_length = resp_body.len();
        sender.send(resp_body.to_vec(), resp_body_length).await;
    }

    async fn not_found<T>(sender: &mut T)
    where
        T: Sender,
    {
        let response = Response::new(
            "HTTP/1.1",
            StatusCode::NotFound,
            Headers::new(),
            "Not Found".as_bytes().to_vec(),
        );
        let (resp_header, resp_body) = response.serialize();
        let resp_header_length = resp_header.len();
        sender.send(resp_header, resp_header_length).await;
        let resp_body_length = resp_body.len();
        sender.send(resp_body.to_vec(), resp_body_length).await;
    }

    async fn service_unavailable<T>(sender: &mut T)
    where
        T: Sender,
    {
        let response = Response::new(
            "HTTP/1.1",
            StatusCode::ServiceUnavailable,
            Headers::new(),
            "Service Unavailable".as_bytes().to_vec(),
        );
        let (resp_header, resp_body) = response.serialize();
        let resp_header_length = resp_header.len();
        sender.send(resp_header, resp_header_length).await;
        let resp_body_length = resp_body.len();
        sender.send(resp_body.to_vec(), resp_body_length).await;
    }

    async fn internal_server_error<T>(sender: &mut T)
    where
        T: Sender,
    {
        let response = Response::new(
            "HTTP/1.1",
            StatusCode::InternalServerError,
            Headers::new(),
            "Internal Server Error".as_bytes().to_vec(),
        );
        let (resp_header, resp_body) = response.serialize();
        let resp_header_length = resp_header.len();
        sender.send(resp_header, resp_header_length).await;
        let resp_body_length = resp_body.len();
        sender.send(resp_body.to_vec(), resp_body_length).await;
    }
}

async fn resp_parse<'a, 'b>(
    id: u32,
    parser: &'a mut RespParser,
    con: &mut tokio::net::TcpStream,
) -> Option<Response<'b>>
where
    'a: 'b,
{
    let mut read_buffer: [u8; 2048] = [0; 2048];
    loop {
        match con.read(&mut read_buffer).await {
            Ok(n) if n == 0 => {
                return None;
            }
            Ok(n) => {
                if parser.block_parse(&read_buffer[0..n]) {
                    break;
                }
            }
            Err(e) => {
                error!("[{}] Reading from Connection: {}", id, e);
                return None;
            }
        };
    }

    parser.finish()
}

#[async_trait]
impl Handler for BasicHandler {
    async fn handle<R, S>(&self, id: u32, receiver: &mut R, sender: &mut S)
    where
        R: Receiver + Send,
        S: Sender + Send,
    {
        // Very crude Keep-Alive work around
        loop {
            let mut req_parser = ReqParser::new_capacity(2048);
            let mut buf = [0; 2048];
            loop {
                match receiver.read(&mut buf).await {
                    Ok(n) if n == 0 => {
                        break;
                    }
                    Ok(n) => {
                        if req_parser.block_parse(&buf[..n]) {
                            break;
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        error!("[{}] Reading Request: {}", id, e);
                        return;
                    }
                };
            }

            let request = match req_parser.finish() {
                Ok(req) => req,
                Err(e) => {
                    error!("[{}] Parsing HTTP-Request: {}", id, e);
                    Self::bad_request(sender).await;
                    return;
                }
            };

            let matched = match self.rules.match_req(&request) {
                Some(m) => m,
                None => {
                    error!("[{}] No Rule matched the Request", id);
                    Self::not_found(sender).await;
                    return;
                }
            };

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

            let mut response_parser = RespParser::new_capacity(1024);
            let mut response = match resp_parse(id, &mut response_parser, &mut connection).await {
                Some(resp) => resp,
                None => {
                    Self::internal_server_error(sender).await;
                    return;
                }
            };

            matched.apply_middlewares_resp(&out_req, &mut response);

            let (resp_header, resp_body) = response.serialize();
            let resp_header_length = resp_header.len();
            sender.send(resp_header, resp_header_length).await;
            let resp_body_length = resp_body.len();
            sender.send(resp_body.to_vec(), resp_body_length).await;

            handle_timer.observe_duration();
        }
    }
}
