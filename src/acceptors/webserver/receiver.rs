use crate::acceptors::traits::Receiver as ReceiverTrait;

use tokio::io::AsyncReadExt;

use async_trait::async_trait;

/// The Receiver half of a Connection established through the
/// Webserver-Acceptor
pub struct Receiver {
    connection: tokio::net::tcp::OwnedReadHalf,
}

impl Receiver {
    /// Creates a new Receiver to be used by the Rest
    /// of the Load-Balancer
    pub fn new(con: tokio::net::tcp::OwnedReadHalf) -> Self {
        Self { connection: con }
    }
}

#[async_trait]
impl ReceiverTrait for Receiver {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.connection.read(buf).await
    }
}
