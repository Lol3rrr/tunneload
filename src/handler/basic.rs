use crate::acceptors::traits::Sender;
use crate::handler::traits::Handler;
use crate::http::{Request, Response};
use crate::rules::ReadManager;

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use log::{debug, error};

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

        debug!("[{}] Requesting '{}' from '{}'", id, out_req.path(), addr);

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

        let mut response_data: Vec<u8> = Vec::with_capacity(2048);
        loop {
            let mut read_data: Vec<u8> = vec![0; 2048];
            match connection.read(&mut read_data).await {
                Ok(n) => {
                    if n == 0 {
                        debug!("[{}] EOF", id);
                        break;
                    }
                    debug!("[{}] Read {} Bytes", id, n);
                    read_data.truncate(n);
                    response_data.append(&mut read_data);
                }
                Err(e) => {
                    error!("[{}] Reading from Connection: {}", id, e);
                    return;
                }
            };
        }

        let mut response = match Response::parse(&response_data) {
            Some(r) => r,
            None => {
                error!("Parsing Response");
                return;
            }
        };

        matched.apply_middlewares_resp(&mut response);

        let serialized_response = response.serialize();
        let serialized_length = serialized_response.len();
        sender.send(serialized_response, serialized_length).await;
    }
}
