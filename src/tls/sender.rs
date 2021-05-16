use crate::acceptors::traits::Sender as SenderTrait;

use async_trait::async_trait;
use rustls::Session;
use std::io::Write;

/// All Data send over this Sender will automatically be encrypted
/// using TLS
pub struct Sender<S>
where
    S: SenderTrait + Send,
{
    og_send: S,
    session: std::sync::Arc<std::sync::Mutex<rustls::ServerSession>>,
}

impl<S> Sender<S>
where
    S: SenderTrait + Send,
{
    /// Creates a new TLS-Sender that establishes a new TLS-Session/Connection
    /// and uses the given Connection as the underlying Connection to talk
    /// to the other Side.
    ///
    /// This allows the TLS-Session to be established over any other type
    /// of connection
    pub fn new(og: S, session: std::sync::Arc<std::sync::Mutex<rustls::ServerSession>>) -> Self {
        Self {
            og_send: og,
            session,
        }
    }

    fn write_tls(&self, buf: Vec<u8>) -> usize {
        let mut tls_writer = self.session.lock().unwrap();

        tls_writer.write(&buf).unwrap()
    }

    /// Get TLS-Data that should be send to the Client
    fn get_write_data(&self) -> Option<Vec<u8>> {
        let mut tls_writer = self.session.lock().unwrap();
        if !tls_writer.wants_write() {
            return None;
        }

        let mut buf = Vec::with_capacity(2048);
        tls_writer.write_tls(&mut buf).unwrap();

        Some(buf)
    }
}

#[async_trait]
impl<S> SenderTrait for Sender<S>
where
    S: SenderTrait + Send + Sync,
{
    async fn send(&mut self, buf: Vec<u8>, _length: usize) {
        // Writes the Plaintext data into the TLS-Session
        self.write_tls(buf);

        // Get all the encrypted TLS-Data out of the TLS-Session
        // and send it to the User
        loop {
            match self.get_write_data() {
                Some(out_buf) => {
                    let out_length = out_buf.len();
                    self.og_send.send(out_buf, out_length).await;
                }
                None => {
                    return;
                }
            };
        }
    }
}
