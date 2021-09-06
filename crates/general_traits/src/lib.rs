#![warn(missing_docs)]
//! This contains all the General-Traits for Tunneload that are used in a wide variety of
//! places and that dont really have one single place to exist in other than this Crate

use std::fmt::Debug;

use async_trait::async_trait;
use stream_httparse::Response;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Defines an interface for a simple Configuration that can
/// easily be managed
pub trait ConfigItem {
    /// Returns the Name of the ConfigItem
    fn name(&self) -> &str;
}
/// A Trait that allows Configs to generate a Default-Configuration
/// using a given Name
///
/// # Usage:
/// This is needed for certain parts where the perceived Update-Order
/// by the Load-Balancer can be out of order and therefore needs a way
/// to create temporary Configs that effectively do nothing.
pub trait DefaultConfig {
    /// Returns a default Config with the given Name
    fn default_name(name: String) -> Self;
}

/// Defines a generic Handler that is responsible to
/// handle incoming Connections, parse the Requests and
/// route them to the right destinations as well as
/// handling the responses
#[async_trait]
pub trait Handler: Debug {
    /// Handles a single Connection
    async fn handle<R, S>(&self, id: u32, receiver: R, sender: S)
    where
        R: Receiver + Send + 'static,
        S: Sender + Send + 'static;
}

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

/// The Bounds needed to register a new Entity on the
/// Dashboard.
/// This will provide all the Information that can then later be accessed
/// and displayed on the Dashboard
pub trait DashboardEntity {
    /// This should uniquely identify the Entity as it otherwise
    /// may lead to confusion when displaying the Data
    ///
    /// This will be included in the form of a "type" Entry in
    /// the generated JSON object
    fn get_type(&self) -> &str;

    /// This should return all the relevant Data for the given
    /// Entity
    ///
    /// This will be included in the form of a "content" Entry in
    /// the generated JSON object
    fn get_content(&self) -> serde_json::Value;
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
