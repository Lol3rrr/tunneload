use crate::acceptors::traits::Sender;
use crate::http::Request;

use async_trait::async_trait;

/// Defines a generic Handler that is responsible to
/// handle all incoming requests and route them to the
/// right destinations as well as handling the response
#[async_trait]
pub trait Handler {
    /// Handles a single request
    async fn handle<T>(&self, id: u32, request: Request<'_>, sender: T)
    where
        T: Sender + Send + Sync;
}
