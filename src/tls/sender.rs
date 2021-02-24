use crate::acceptors::traits::Sender as SenderTrait;

use async_trait::async_trait;

pub struct Sender<'a, S>
where
    S: SenderTrait + Send,
{
    og_send: &'a mut S,
}

impl<'a, S> Sender<'a, S>
where
    S: SenderTrait + Send,
{
    pub fn new(og: &'a mut S) -> Self {
        Self { og_send: og }
    }
}

#[async_trait]
impl<'a, S> SenderTrait for Sender<'a, S>
where
    S: SenderTrait + Send,
{
    async fn send(&mut self, buf: Vec<u8>, length: usize) {
        self.og_send.send(buf, length).await
    }
}
