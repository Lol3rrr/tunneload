use crate::acceptors::traits::Receiver as ReceiverTrait;

use tokio::io::AsyncReadExt;

use async_trait::async_trait;

pub struct Receiver<'a> {
    connection: tokio::net::tcp::ReadHalf<'a>,
}

impl<'a> Receiver<'a> {
    pub fn new(con: tokio::net::tcp::ReadHalf<'a>) -> Self {
        Self { connection: con }
    }
}

#[async_trait]
impl ReceiverTrait for Receiver<'_> {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.connection.read(buf).await
    }
}
