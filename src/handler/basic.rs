use crate::acceptors::traits::Sender;
use crate::handler::traits::Handler;
use crate::http::{streaming_parser::RespParser, Request};
use crate::rules::ReadManager;

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use log::error;

#[derive(Clone)]
pub struct BasicHandler {
    rules: ReadManager,
}

impl BasicHandler {
    pub fn new(rules_manager: ReadManager) -> Self {
        Self {
            rules: rules_manager,
        }
    }
}

#[async_trait]
impl Handler for BasicHandler {
    async fn handle<T>(&self, id: u32, request: Request<'_>, sender: T)
    where
        T: Sender + Send + Sync,
    {
        let matched = match self.rules.match_req(&request) {
            Some(m) => m,
            None => {
                error!("[{}] No Rule matched the Request", id);
                return;
            }
        };

        let mut out_req = request;
        matched.apply_middlewares_req(&mut out_req);
        let addr = matched.service().address();
        let mut connection = match tokio::net::TcpStream::connect(addr).await {
            Ok(c) => c,
            Err(e) => {
                error!("[{}] Connecting to Address: {}", id, e);
                return;
            }
        };

        let (serialized_headers, serialized_body) = out_req.serialize();
        match connection.write_all(&serialized_headers).await {
            Ok(_) => {}
            Err(e) => {
                error!("[{}] Writing Data to connection: {}", id, e);
                return;
            }
        };
        match connection.write_all(serialized_body).await {
            Ok(_) => {}
            Err(e) => {
                error!("[{}] Writing Data to connection: {}", id, e);
                return;
            }
        };

        let mut response_parser = RespParser::new_capacity(4096);
        loop {
            let mut read_data: Vec<u8> = vec![0; 2048];
            match connection.read(&mut read_data).await {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                    response_parser.block_parse(&read_data[0..n]);
                }
                Err(e) => {
                    error!("[{}] Reading from Connection: {}", id, e);
                    return;
                }
            };
        }

        let mut response = match response_parser.finish() {
            Some(r) => r,
            None => {
                error!("Parsing Response");
                return;
            }
        };

        matched.apply_middlewares_resp(&mut response);

        let (resp_header, resp_body) = response.serialize();
        let resp_header_length = resp_header.len();
        sender.send(resp_header, resp_header_length).await;
        let resp_body_length = resp_body.len();
        sender.send(resp_body.to_vec(), resp_body_length).await;
    }
}
