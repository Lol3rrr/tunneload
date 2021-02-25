use crate::acceptors::traits::Sender as SenderTrait;

use async_trait::async_trait;
use rustls::Session;
use std::io::Write;

pub struct Sender<'a, 'b, S>
where
    S: SenderTrait + Send,
{
    og_send: &'a mut S,
    session: &'b std::sync::Mutex<rustls::ServerSession>,
}

impl<'a, 'b, S> Sender<'a, 'b, S>
where
    S: SenderTrait + Send,
{
    pub fn new(og: &'a mut S, session: &'b mut std::sync::Mutex<rustls::ServerSession>) -> Self {
        Self {
            og_send: og,
            session,
        }
    }

    fn write_tls(&self, buf: Vec<u8>) -> Vec<u8> {
        let mut s_writer = self.session.lock().unwrap();

        let written = s_writer.write(&buf).unwrap();
        let mut result: Vec<u8> = Vec::with_capacity(written);

        s_writer.write_tls(&mut result).unwrap();

        result
    }
}

#[async_trait]
impl<'a, 'b, S> SenderTrait for Sender<'a, 'b, S>
where
    S: SenderTrait + Send,
{
    async fn send(&mut self, buf: Vec<u8>, _length: usize) {
        let send_data = self.write_tls(buf);

        let send_length = send_data.len();
        self.og_send.send(send_data, send_length).await
    }
}
