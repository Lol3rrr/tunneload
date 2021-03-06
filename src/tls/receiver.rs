use crate::acceptors::traits::Receiver as ReceiverTrait;

use async_trait::async_trait;
use log::info;
use rustls::Session;
use std::io::Read;

pub struct Receiver<'a, R>
where
    R: ReceiverTrait + Send,
{
    og_read: &'a mut R,
    session: std::sync::Arc<std::sync::Mutex<rustls::ServerSession>>,
}

impl<'a, R> Receiver<'a, R>
where
    R: ReceiverTrait + Send,
{
    pub fn new(
        og: &'a mut R,
        session: std::sync::Arc<std::sync::Mutex<rustls::ServerSession>>,
    ) -> Self {
        Self {
            og_read: og,
            session,
        }
    }

    fn read_from_buf(&self, buf: &mut [u8]) -> Option<std::io::Result<usize>> {
        let mut tls_session = self.session.lock().unwrap();
        if tls_session.wants_read() {
            None
        } else {
            Some(tls_session.read(buf))
        }
    }
}

#[async_trait]
impl<'a, R> ReceiverTrait for Receiver<'a, R>
where
    R: ReceiverTrait + Send,
{
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if let Some(res) = self.read_from_buf(buf) {
            return res;
        }

        let mut read_more = true;

        // Read more into the TLS-Session until it has
        // plaintext that can be read
        while read_more {
            let mut tmp: Vec<u8> = vec![0; buf.len()];
            let read = self.og_read.read(&mut tmp).await?;
            tmp.truncate(read);

            {
                let mut tls_session = self.session.lock().unwrap();

                let tls_read = tls_session.read_tls(&mut &tmp[..]).unwrap();
                if tls_read < read {
                    info!("TLS-Read less than it could: {} < {}", tls_read, read);
                }
                tls_session.process_new_packets().unwrap();
                read_more = tls_session.wants_read();
            }
        }

        let mut tls_session = self.session.lock().unwrap();
        tls_session.read(buf)
    }
}
