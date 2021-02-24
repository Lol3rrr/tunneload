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
