use crate::acceptors::traits::Sender;

use tunneler_core::client::queues::Sender as TSender;

use async_trait::async_trait;

#[async_trait]
impl Sender for TSender {
    async fn send(&mut self, data: Vec<u8>, length: usize) {
        TSender::send(self, data, length as u64).await;
    }
}
