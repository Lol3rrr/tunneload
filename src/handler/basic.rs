use crate::acceptors::traits::Sender;
use crate::handler::traits::Handler;
use crate::http::Request;
use crate::rules::ReadManager;

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use log::{debug, error, info};

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
    async fn handle<T>(&self, request: Request<'_>, sender: T)
    where
        T: Sender + Send + Sync,
    {
        let matched = match self.rules.match_req(&request) {
            Some(m) => m,
            None => {
                error!("No Rule matched the Request");
                return;
            }
        };

        let mut out_req = request;
        matched.apply_middlewares(&mut out_req);
        let mut connection = match tokio::net::TcpStream::connect(matched.service().address()).await
        {
            Ok(c) => c,
            Err(e) => {
                error!("Connecting to Address: {}", e);
                return;
            }
        };

        let serialized = out_req.serialize();
        match connection.write_all(&serialized).await {
            Ok(_) => {}
            Err(e) => {
                error!("Writing Data to connection: {}", e);
                return;
            }
        };

        loop {
            let mut read_data: Vec<u8> = vec![0; 2048];
            match connection.read(&mut read_data).await {
                Ok(n) => {
                    if n == 0 {
                        return;
                    }

                    sender.send(read_data, n);
                }
                Err(e) => {
                    error!("Reading from Connection: {}", e);
                    return;
                }
            };
        }
    }
}
