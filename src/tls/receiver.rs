use crate::acceptors::traits::Receiver as ReceiverTrait;

use async_trait::async_trait;
use rustls::Session;
use std::io::Read;

pub struct Receiver<'a, R>
where
    R: ReceiverTrait + Send,
{
    og_read: &'a mut R,
    session: std::sync::Mutex<rustls::ServerSession>,
}

impl<'a, R> Receiver<'a, R>
where
    R: ReceiverTrait + Send,
{
    pub fn new(og: &'a mut R, session: std::sync::Mutex<rustls::ServerSession>) -> Self {
        Self {
            og_read: og,
            session,
        }
    }
}

#[async_trait]
impl<'a, R> ReceiverTrait for Receiver<'a, R>
where
    R: ReceiverTrait + Send,
{
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut tmp: Vec<u8> = Vec::with_capacity(buf.len());
        self.og_read.read(&mut tmp).await.unwrap();

        let mut tls_session = self.session.lock().unwrap();

        let mut tmp_buffer = std::io::Cursor::new(tmp);
        tls_session.read_tls(&mut tmp_buffer).unwrap();
        tls_session.process_new_packets().unwrap();
        tls_session.read(buf)
    }
}
