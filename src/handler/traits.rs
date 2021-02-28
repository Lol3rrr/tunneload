use crate::acceptors::traits::{Receiver, Sender};

use async_trait::async_trait;

/// Defines a generic Handler that is responsible to
/// handle all incoming requests and route them to the
/// right destinations as well as handling the response
#[async_trait]
pub trait Handler {
    /// Handles a single request
    async fn handle<R, S>(&self, id: u32, receiver: &mut R, sender: &mut S)
    where
        R: Receiver + Send,
        S: Sender + Send;
}

/// Defines a generic Interface for reading Data from a
/// Service-Connection
#[async_trait]
pub trait ServiceConnection {
    /// Attempts to Read data from the Connection into the given
    /// Buffer-Array and returns how how many bytes were actually
    /// read and placed into the buffer
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
}
