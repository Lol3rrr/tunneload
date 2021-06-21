use std::fmt::{Debug, Formatter};

use crate::acceptors::traits::Sender as SenderTrait;

use async_trait::async_trait;

/// The Sending half of a single Connection made through the
/// Tunneler-Acceptor
pub struct Sender<S> {
    tx: S,
}

impl<S> Debug for Sender<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tunneler-Sender ()")
    }
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
    async fn send(&mut self, data: &[u8]) {
        if let Err(e) = self.tx.send_msg(data.to_vec(), data.len() as u64).await {
            tracing::error!("Sending: {:?}", e);
        }
    }
}
