use crate::acceptors::traits::Sender as SenderTrait;

use tokio::io::AsyncWriteExt;

use async_trait::async_trait;
use log::error;

/// The Sender half of a Connection established through the
/// Webserver-Acceptor
pub struct Sender<'a> {
    connection: tokio::net::tcp::WriteHalf<'a>,
}

impl<'a> Sender<'a> {
    /// Creates a new Sender to be used by the Rest of
    /// the Load-Balancer
    pub fn new(con: tokio::net::tcp::WriteHalf<'a>) -> Self {
        Self { connection: con }
    }
}

#[async_trait]
impl SenderTrait for Sender<'_> {
    async fn send(&mut self, data: Vec<u8>, _length: usize) {
        if let Err(e) = self.connection.write_all(&data).await {
            error!("Writing to Connection: {}", e);
            return;
        };
    }
}