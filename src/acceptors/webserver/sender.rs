use crate::acceptors::traits::Sender as SenderTrait;

use async_trait::async_trait;
use log::error;

pub struct Sender {
    connection: tokio::net::TcpStream,
}

impl Sender {
    pub fn new(con: tokio::net::TcpStream) -> Self {
        Self { connection: con }
    }
}

#[async_trait]
impl SenderTrait for Sender {
    async fn send(&self, data: Vec<u8>, length: usize) {
        let mut data_left = &data[..];
        let mut left_to_write = length;
        while left_to_write > 0 {
            match self.connection.writable().await {
                Ok(_) => {}
                Err(e) => {
                    error!("Checking if the Connection is writable: {}", e);
                    return;
                }
            };

            match self.connection.try_write(data_left) {
                Ok(n) if n == 0 => {}
                Ok(n) => {
                    data_left = &data_left[n..];
                    left_to_write -= n;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    error!("Writing to Connection: {}", e);
                    return;
                }
            };
        }
    }
}
