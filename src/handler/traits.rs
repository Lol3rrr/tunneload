//! All the Traits for handling Connections and Requests

use std::fmt::Debug;

use crate::acceptors::traits::{Receiver, Sender};

use async_trait::async_trait;

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
