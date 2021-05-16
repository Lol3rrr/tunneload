use crate::acceptors::traits::Receiver as ReceiverTrait;

use tokio::io::AsyncReadExt;

use async_trait::async_trait;

/// The Receiver half of a Connection established through the
/// Webserver-Acceptor
pub type Receiver = tokio::net::tcp::OwnedReadHalf;

#[async_trait]
impl ReceiverTrait for tokio::net::tcp::OwnedReadHalf {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        AsyncReadExt::read(self, buf).await
    }
}
