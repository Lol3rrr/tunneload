use crate::handler::traits::ServiceConnection;

use async_trait::async_trait;

use tokio::io::AsyncReadExt;

#[async_trait]
impl ServiceConnection for tokio::net::TcpStream {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        AsyncReadExt::read(self, buf).await
    }
}
