use general_traits::Sender as SenderTrait;

use async_trait::async_trait;
use std::{
    fmt::{Debug, Formatter},
    io::Write,
};

/// All Data send over this Sender will automatically be encrypted
/// using TLS
pub struct Sender<S> {
    og_send: S,
    session: std::sync::Arc<std::sync::Mutex<rustls::ServerConnection>>,
}

impl<S> Debug for Sender<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TLS-Sender ()")
    }
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
    pub fn new(og: S, session: std::sync::Arc<std::sync::Mutex<rustls::ServerConnection>>) -> Self {
        Self {
            og_send: og,
            session,
        }
    }

    fn write_tls(&self, buf: &[u8]) -> usize {
        let mut tls_writer = self.session.lock().unwrap();

        let mut writer = tls_writer.writer();
        writer.write(buf).unwrap()
    }

    /// Get TLS-Data that should be send to the Client
    fn get_write_data<W>(&self, buf: &mut W) -> Option<usize>
    where
        W: std::io::Write,
    {
        let mut tls_writer = self.session.lock().unwrap();
        if !tls_writer.wants_write() {
            return None;
        }

        match tls_writer.write_tls(buf) {
            Ok(n) => Some(n),
            Err(_) => None,
        }
    }
}

#[async_trait]
impl<S> SenderTrait for Sender<S>
where
    S: SenderTrait + Send + Sync,
{
    async fn send(&mut self, buf: &[u8]) {
        // Writes the Plaintext data into the TLS-Session
        self.write_tls(buf);

        // Get all the encrypted TLS-Data out of the TLS-Session
        // and send it to the User
        let mut write_buffer = Vec::with_capacity(4096);
        loop {
            match self.get_write_data(&mut write_buffer) {
                Some(written) => {
                    self.og_send.send(&write_buffer[..written]).await;
                    write_buffer.clear();
                }
                None => {
                    return;
                }
            };
        }
    }
}
