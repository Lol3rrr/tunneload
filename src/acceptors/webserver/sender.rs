use std::fmt::{Debug, Formatter};

use crate::acceptors::traits::Sender as SenderTrait;

use tokio::io::AsyncWriteExt;

use async_trait::async_trait;
use tracing::Level;

/// The Sender half of a Connection established through the
/// Webserver-Acceptor
pub struct Sender {
    connection: tokio::net::tcp::OwnedWriteHalf,
}

impl Debug for Sender {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Webserver-Sender")
    }
}

impl Sender {
    /// Creates a new Sender to be used by the Rest of
    /// the Load-Balancer
    pub fn new(con: tokio::net::tcp::OwnedWriteHalf) -> Self {
        Self { connection: con }
    }
}

#[async_trait]
impl SenderTrait for Sender {
    async fn send(&mut self, data: &[u8]) {
        if let Err(e) = self.connection.write_all(data).await {
            tracing::event!(Level::ERROR, "Writing to Connection: {}", e);
            return;
        }
    }
}
