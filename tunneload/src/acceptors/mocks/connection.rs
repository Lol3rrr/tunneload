use super::{Receiver, Sender};
use general_traits::{Receiver as ReceiverTrait, Sender as SenderTrait};

use async_trait::async_trait;

#[derive(Debug)]
pub struct Connection<'recv, 'send> {
    receiver: &'recv mut Receiver,
    sender: &'send mut Sender,
}

impl<'recv, 'send> Connection<'recv, 'send> {
    pub fn new(receiver: &'recv mut Receiver, sender: &'send mut Sender) -> Self {
        Self { receiver, sender }
    }
}

#[async_trait]
impl ReceiverTrait for Connection<'_, '_> {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.receiver.read(buf).await
    }
}

#[async_trait]
impl SenderTrait for Connection<'_, '_> {
    async fn send(&mut self, data: &[u8]) {
        self.sender.send(data).await
    }
}
