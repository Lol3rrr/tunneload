use crate::acceptors::traits::Sender;
use crate::handler::traits::Handler;
use crate::http::Request;

use async_trait::async_trait;

#[derive(Clone)]
pub struct BasicHandler {}

impl BasicHandler {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Handler for BasicHandler {
    async fn handle<T>(&self, request: Request<'_>, sender: T)
    where
        T: Sender + Send + Sync,
    {
        println!("Request: {}", request);

        let content =
            "HTTP/1.1 200 OK\r\ncontent-length: 16\r\nserver: tunneload\r\n\r\nThis is the body";
        let mut data = Vec::new();
        data.extend_from_slice(content.as_bytes());
        let length = data.len();
        sender.send(data, length);
    }
}
