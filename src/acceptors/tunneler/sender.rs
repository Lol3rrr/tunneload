use crate::acceptors::traits::Sender as SenderTrait;

use async_trait::async_trait;

/// The Sending half of a single Connection made through the
/// Tunneler-Acceptor
pub struct Sender<S>
where
    S: tunneler_core::client::Sender + Send + Sync,
{
    tx: S,
}

impl<S> Sender<S>
where
    S: tunneler_core::client::Sender + Send + Sync,
{
    /// Creates a new Sender that can be used by the
    /// rest of the Load-Balancer
    pub fn new(tx: S) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl<S> SenderTrait for Sender<S>
where
    S: tunneler_core::client::Sender + Send + Sync,
{
    async fn send(&mut self, data: Vec<u8>, length: usize) {
        self.tx.send_msg(data, length as u64).await;
    }
}
