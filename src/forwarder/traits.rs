use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{http::Request, rules::Rule};

/// Defines a generic Interface for reading Data from a
/// Service-Connection
#[async_trait]
pub trait ServiceConnection: Send + Sync + 'static {
    /// Attempts to Read data from the Connection into the given
    /// Buffer-Array and returns how how many bytes were actually
    /// read and placed into the buffer
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;

    /// Attempts to Write the given Buffer to the underlying
    /// Connection and returns how many bytes were actually written
    async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>;

    /// Writes the entire Buffer to the underlying connection
    async fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        let mut written = 0;
        let mut left_to_write = buf.len();

        while left_to_write > 0 {
            match self.write(&buf[written..(written + left_to_write)]).await {
                Ok(wr) => {
                    written += wr;
                    left_to_write -= wr;
                }
                Err(e) => {
                    return Err(e);
                }
            };
        }
        Ok(())
    }

    /// Serializes the Request and then sends it over the underlying
    /// connection to the User
    async fn write_req(&mut self, req: &Request<'_>) -> std::io::Result<()> {
        let (headers, body) = req.serialize();

        if let Err(e) = self.write_all(&headers).await {
            return Err(e);
        }
        if let Err(e) = self.write_all(&body).await {
            return Err(e);
        }

        Ok(())
    }
}

#[async_trait]
pub trait Forwarder {
    type Connection: ServiceConnection;

    async fn create_con(&self, rule: &Rule) -> Option<Self::Connection>;
}

#[async_trait]
impl ServiceConnection for tokio::net::TcpStream {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        AsyncReadExt::read(self, buf).await
    }

    async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        AsyncWriteExt::write(self, buf).await
    }
}
