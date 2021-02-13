use tunneler_core::client::queues::Sender as TSender;

use async_trait::async_trait;

/// This Trait specifies an interface that the
/// Rest of the Codebase can use to send the Data
/// back to the User, without needing to differentiate
/// between having a normal Webserver serve the user or
/// a connection from Tunneler
#[async_trait]
pub trait Sender {
    /// Sends the given Piece of data
    async fn send(&self, data: Vec<u8>, length: usize);
}

#[async_trait]
impl Sender for TSender {
    async fn send(&self, data: Vec<u8>, length: usize) {
        self.send(data, length as u64).await;
    }
}
