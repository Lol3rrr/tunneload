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
}

#[async_trait]
impl<S> SenderTrait for Sender<S>
where
    S: SenderTrait + Send + Sync,
{
    async fn send(&mut self, buf: &[u8]) {
        let mut send_buf = buf;

        loop {
            loop {
                let mut write_buffer = Vec::with_capacity(4096);
                let written = {
                    let mut tls_session = self.session.lock().expect("Obtaining lock for Session");
                    if !tls_session.wants_write() {
                        break;
                    }

                    match tls_session.write_tls(&mut write_buffer) {
                        Ok(n) => n,
                        Err(_) => return,
                    }
                };

                self.og_send.send(&write_buffer[..written]).await;
            }

            if send_buf.is_empty() {
                break;
            }

            let mut tls_session = self.session.lock().expect("Obtaining lock for Session");
            let mut writer = tls_session.writer();
            match writer.write(send_buf) {
                Ok(n) => {
                    send_buf = &send_buf[n..];
                }
                Err(_) => return,
            };
        }
    }
}
