use std::fmt::{Debug, Formatter};

use crate::acceptors::traits::Receiver as ReceiverTrait;

use tokio::io::AsyncReadExt;

use async_trait::async_trait;

/// The Receiver half of a Connection established through the
/// Webserver-Acceptor
pub struct Receiver {
    rx: tokio::net::tcp::OwnedReadHalf,
}

impl Debug for Receiver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Webserver-Receiver ()")
    }
}

impl Receiver {
    /// Creates a new Receiver with the given TCP-Reader as the underlying
    /// connection medium
    pub fn new(rx: tokio::net::tcp::OwnedReadHalf) -> Self {
        Self { rx }
    }
}

#[async_trait]
impl ReceiverTrait for Receiver {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        AsyncReadExt::read(&mut self.rx, buf).await
    }
}
