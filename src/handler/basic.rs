use crate::acceptors::traits::{Receiver, Sender};
use crate::handler::traits::Handler;
use crate::http::{
    streaming_parser::ChunkParser, streaming_parser::ReqParser, streaming_parser::RespParser,
    Headers, Request, Response, StatusCode,
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
) -> Option<(Response<'b>, Option<Vec<u8>>)>
where
    'a: 'b,
{
    let mut left_over_buffer: Option<Vec<u8>> = None;

    let mut read_buffer: [u8; 2048] = [0; 2048];
    loop {
        match con.read(&mut read_buffer).await {
            Ok(n) if n == 0 => {
                return None;
            }
            Ok(n) => {
                let (parser_done, parser_rest) = parser.block_parse(&read_buffer[0..n]);
                if parser_done {
                    if let Some(rest) = parser_rest {
                        left_over_buffer = Some(rest.to_vec());
                    }

                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                error!("[{}] Reading from Connection: {}", id, e);
                return None;
            }
        };
    }

    let result = match parser.finish() {
        Some(r) => r,
        None => return None,
    };
    Some((result, left_over_buffer))
}

async fn req_parse<'a, 'b, R>(id: u32, parser: &'a mut ReqParser, rx: &mut R) -> Option<Request<'b>>
where
    R: Receiver + Send,
    'a: 'b,
{
    let mut buf = [0; 2048];
    loop {
        match rx.read(&mut buf).await {
            Ok(n) if n == 0 => {
                return None;
            }
            Ok(n) => {
                if parser.block_parse(&buf[..n]) {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                error!("[{}] Reading Request: {}", id, e);
                return None;
            }
        };
    }

    match parser.finish() {
        Ok(req) => Some(req),
        Err(e) => {
            error!("[{}] Parsing Request: {}", id, e);
            None
        }
    }
}

async fn send_chunked<S>(
    id: u32,
    con: &mut tokio::net::TcpStream,
    sender: &mut S,
    inital_data: Option<Vec<u8>>,
) where
    S: Sender + Send,
{
    let mut chunk_parser = ChunkParser::new();
    if let Some(tmp) = inital_data {
        let done = chunk_parser.block_parse(&tmp);
        if done {
            let result = match chunk_parser.finish() {
                Some(r) => r,
                None => return,
            };
            let chunk_size = result.size();

            let mut out = Vec::with_capacity(result.size() + 32);
            result.serialize(&mut out);
            let out_length = out.len();
            sender.send(out, out_length).await;

            if chunk_size == 0 {
                return;
            }

            chunk_parser = ChunkParser::new();
        }
    }

    let mut read_buf = [0; 2048];
    loop {
        match con.read(&mut read_buf).await {
            Ok(n) if n == 0 => {
                return;
            }
            Ok(n) => {
                let done = chunk_parser.block_parse(&read_buf[0..n]);
                if done {
                    let result = match chunk_parser.finish() {
                        Some(r) => r,
                        None => return,
                    };
                    let chunk_size = result.size();

                    let mut out = Vec::with_capacity(result.size() + 32);
                    result.serialize(&mut out);
                    let out_length = out.len();
                    sender.send(out, out_length).await;

                    if chunk_size == 0 {
                        return;
                    }

                    chunk_parser = ChunkParser::new();
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                error!("[{}] Reading from Connection: {}", id, e);
                return;
            }
        };
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

        while keep_alive {
            let mut req_parser = ReqParser::new_capacity(2048);
            let request = match req_parse(id, &mut req_parser, receiver).await {
                Some(r) => r,
                None => {
                    Self::bad_request(sender).await;
                    return;
                }
            };

            keep_alive = request.is_keep_alive();

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
            let (mut response, left_over_buffer) =
                match resp_parse(id, &mut response_parser, &mut connection).await {
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

            // First assumes that it is NOT chunked and should
            // just send the data normally
            if !response.is_chunked() {
                let resp_body_length = resp_body.len();
                sender.send(resp_body.to_vec(), resp_body_length).await;
            } else {
                send_chunked(id, &mut connection, sender, left_over_buffer).await;
            }

            handle_timer.observe_duration();
        }
    }
}
