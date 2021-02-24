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
    async fn read(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let mut read_buf = [0; 2048];
        match self.connection.read(&mut read_buf).await {
            Ok(n) if n == 0 => Ok(0),
            Ok(n) => {
                buf.extend_from_slice(&read_buf[..n]);
                Ok(n)
            }
            Err(e) => Err(e),
        }
    }
}
