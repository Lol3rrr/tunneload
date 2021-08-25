use general_traits::Receiver as ReceiverTrait;

use async_trait::async_trait;
use rustls::Session;
use std::{
    fmt::{Debug, Formatter},
    io::Read,
};

/// All Data received over this Receiver is encrypted using TLS
/// under the hood and is automatically decoded when you read from it
pub struct Receiver<R> {
    og_read: R,
    session: std::sync::Arc<std::sync::Mutex<rustls::ServerSession>>,
}

impl<R> Debug for Receiver<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TLS-Receiver ()")
    }
}

impl<R> Receiver<R>
where
    R: ReceiverTrait + Send,
{
    /// Creates a new TLS-Receiver that establishes a new TLS-Session/Connection
    /// and uses the given Connection as the underlying Connection to talk
    /// to the other Side.
    ///
    /// This allows the TLS-Session to be established over any other type
    /// of connection
    pub fn new(og: R, session: std::sync::Arc<std::sync::Mutex<rustls::ServerSession>>) -> Self {
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
impl<R> ReceiverTrait for Receiver<R>
where
    R: ReceiverTrait + Send,
{
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if let Some(res) = self.read_from_buf(buf) {
            return res;
        }

        // Read more into the TLS-Session until it has
        // plaintext that can be read
        let mut tmp_buf: Vec<u8> = vec![0; buf.len()];
        loop {
            tmp_buf.resize(buf.len(), 0);
            let read = self.og_read.read(&mut tmp_buf).await?;
            tmp_buf.truncate(read);

            {
                let mut tls_session = self.session.lock().unwrap();

                let tls_read = tls_session.read_tls(&mut &tmp_buf[..]).unwrap();
                if tls_read < read {
                    tracing::info!("TLS-Read less than it could: {} < {}", tls_read, read);
                }
                tls_session.process_new_packets().unwrap();

                if !tls_session.wants_read() {
                    return tls_session.read(buf);
                }
            }
        }
    }
}
