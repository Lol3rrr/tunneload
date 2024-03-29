use async_trait::async_trait;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};

use rules::Rule;

use stream_httparse::Request;

/// Defines an Interface to send and receive Data between
/// the Load-Balancer and a Service
///
/// This can take form in different ways and is not required
/// to be done using an underlying networking connection, however
/// this is most commonly the case.
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
        if let Err(e) = self.write_all(body).await {
            return Err(e);
        }

        Ok(())
    }

    /// The Writer Half of the Service-Connection
    type WriteHalf: ServiceWriter;
    /// The Reader Half of the Service-Connection
    type ReadHalf: ServiceReader;

    /// Splits the given Connection into an owned Read and Write
    /// halves
    fn halves_owned(self) -> (Self::ReadHalf, Self::WriteHalf);
}

/// The Writer Half of a Service Connection
#[async_trait]
pub trait ServiceWriter: Send + Sync + 'static {
    /// Attempts to write the given Buffer
    async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>;
}

/// The Reader Half of a Service Connection
#[async_trait]
pub trait ServiceReader: Send + Sync + 'static {
    /// Attempts to write the given Buffer
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
}

/// A Forwarder is responsible for establishing a new Connection
/// based on the provided Rule.
///
/// This Connection does not need to be an actual network connection,
/// but rather can take any form that implements the ServiceConnection
/// Trait.
#[async_trait]
pub trait Forwarder {
    /// The underlying Type of the Connection that will be established
    /// by the Forwarder
    type Connection: ServiceConnection;
    /// The Error type returned by the Create_Con function
    type ConnectError: std::fmt::Debug + Send;

    /// Attempts to create a new Connection based on the Data provided
    /// by the Rule
    async fn create_con(&self, rule: &Rule) -> Result<Self::Connection, Self::ConnectError>;
}

#[async_trait]
impl ServiceConnection for tokio::net::TcpStream {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        AsyncReadExt::read(self, buf).await
    }

    async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        AsyncWriteExt::write(self, buf).await
    }

    type ReadHalf = OwnedReadHalf;
    type WriteHalf = OwnedWriteHalf;

    fn halves_owned(self) -> (Self::ReadHalf, Self::WriteHalf) {
        self.into_split()
    }
}

#[async_trait]
impl ServiceReader for tokio::net::tcp::OwnedReadHalf {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        AsyncReadExt::read(self, buf).await
    }
}
#[async_trait]
impl ServiceWriter for tokio::net::tcp::OwnedWriteHalf {
    async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        AsyncWriteExt::write(self, buf).await
    }
}
