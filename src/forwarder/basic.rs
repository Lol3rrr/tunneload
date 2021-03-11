use async_trait::async_trait;

use crate::rules::Rule;

use super::Forwarder;

#[derive(Debug, Clone)]
pub struct BasicForwarder {}

impl BasicForwarder {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Forwarder for BasicForwarder {
    type Connection = tokio::net::TcpStream;

    async fn create_con(&self, rule: &Rule) -> Option<Self::Connection> {
        let service = rule.service();

        service.connect().await
    }
}
