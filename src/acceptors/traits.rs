use async_trait::async_trait;
use stream_httparse::Response;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// This Trait specifies an interface that the
/// Rest of the Codebase can use to send the Data
/// back to the User, without needing to differentiate
/// between having a normal Webserver serve the user or
/// a connection from Tunneler
#[async_trait]
pub trait Sender: Send + std::fmt::Debug {
    /// Sends the given Piece of data
    async fn send(&mut self, data: &[u8]);

    /// Serializes and Sends the given Response using this Sender.
    /// This is just meant as a convience to not repeat
    /// the serialization and sending of responses in all
    /// sorts of situations
    async fn send_response(&mut self, response: &Response<'_>) {
        let (head, body) = response.serialize();
        self.send(&head).await;
        self.send(body).await;
    }
}

/// This Trait specifies an interface that the Rest
/// of the Codebase can use to read from an existing
/// connection without needing to know about how this
/// is actually done or through what acceptor this goes
#[async_trait]
pub trait Receiver: Send + std::fmt::Debug {
    /// Reads from the Connection until there is either no more
    /// data left to read or until the provided Buffer is full
    ///
    /// Returns:
    /// The number of bytes that were read from the connection
    /// and written into the provided Buffer
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;

    /// Reads until the given Buffer is full or an error was encountered
    async fn read_full(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        let mut index = 0;
        let mut left_to_read = buf.len();

        while left_to_read > 0 {
            let read = self.read(&mut buf[index..]).await?;
            index += read;
            left_to_read -= read;
        }

        Ok(())
    }
}

#[async_trait]
impl Receiver for tokio::net::TcpStream {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        AsyncReadExt::read(self, buf).await
    }
}
#[async_trait]
impl Sender for tokio::net::TcpStream {
    async fn send(&mut self, data: &[u8]) {
        if let Err(e) = AsyncWriteExt::write_all(self, data).await {
            tracing::error!("Writing to TCP-Stream: {:?}", e);
            return;
        }
    }
}

#[async_trait]
impl Receiver for tokio::net::tcp::OwnedReadHalf {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        AsyncReadExt::read(self, buf).await
    }
}
#[async_trait]
impl Sender for tokio::net::tcp::OwnedWriteHalf {
    async fn send(&mut self, data: &[u8]) {
        if let Err(e) = AsyncWriteExt::write_all(self, data).await {
            tracing::error!("Writing to TCP-Stream: {:?}", e);
            return;
        }
    }
}
