use crate::acceptors::traits::Receiver as ReceiverTrait;

use async_trait::async_trait;

pub struct Receiver<'a, R>
where
    R: ReceiverTrait + Send,
{
    og_read: &'a mut R,
}

impl<'a, R> Receiver<'a, R>
where
    R: ReceiverTrait + Send,
{
    pub fn new(og: &'a mut R) -> Self {
        Self { og_read: og }
    }
}

#[async_trait]
impl<'a, R> ReceiverTrait for Receiver<'a, R>
where
    R: ReceiverTrait + Send,
{
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.og_read.read(buf).await
    }
}
