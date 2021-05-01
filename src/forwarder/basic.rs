use async_trait::async_trait;

use crate::rules::Rule;

use super::Forwarder;

/// This is a simple Forwarder
///
/// # Behaviour
/// This Forwader establishes a new Connection to a Rules-Service
/// and then forwards the requests without any further processing
#[derive(Debug, Clone)]
pub struct BasicForwarder {}

impl BasicForwarder {
    /// Creates a new empty BasicForwarder Instance
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for BasicForwarder {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Forwarder for BasicForwarder {
    type Connection = tokio::net::TcpStream;

    async fn create_con(&self, rule: &Rule) -> Option<Self::Connection> {
        let service = rule.service();

        match service.connect().await {
            Ok(c) => Some(c),
            Err(e) => {
                log::error!("Connecting to Service: {}", e);
                None
            }
        }
    }
}
