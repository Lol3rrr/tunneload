use crate::acceptors::traits::{Receiver, Sender};
use crate::tls;

use rustls::Session;

#[derive(Debug)]
pub enum Error {
    InvalidConAttempt,
    ReadTLS(std::io::Error),
    WriteTLS(std::io::Error),
    TLS(rustls::TLSError),
}

// This leans heavily on this example
// https://github.com/ctz/rustls/issues/77
#[tracing::instrument]
async fn complete_handshake<R, S>(
    rx: &mut R,
    tx: &mut S,
    tls_session: &mut rustls::ServerSession,
) -> Result<(), Error>
where
    R: Receiver + Send,
    S: Sender + Send,
{
    while tls_session.is_handshaking() {
        if tls_session.is_handshaking() && tls_session.wants_write() {
            // Write the Data
            loop {
                let mut tmp_buf = Vec::with_capacity(2048);
                let written = match tls_session.write_tls(&mut tmp_buf) {
                    Ok(n) => n,
                    Err(e) => {
                        return Err(Error::WriteTLS(e));
                    }
                };

                if written == 0 {
                    break;
                }

                tx.send(&tmp_buf[..written]).await;
            }
        }

        if tls_session.is_handshaking() && tls_session.wants_read() {
            // Read and process the Data
            let mut tmp = [0; 2048];
            let read = match rx.read(&mut tmp).await {
                Ok(n) if n == 0 => {
                    return Err(Error::InvalidConAttempt);
                }
                Ok(n) => n,
                Err(e) => {
                    return Err(Error::ReadTLS(e));
                }
            };

            let mut read_data = &tmp[..read];
            if let Err(e) = tls_session.read_tls(&mut read_data) {
                return Err(Error::ReadTLS(e));
            }

            if let Err(e) = tls_session.process_new_packets() {
                return Err(Error::TLS(e));
            }
        }
    }

    Ok(())
}

/// Creates a new Receiver and Sender using TLS that utilize the
/// given Receiver and Sender as the underlying connection to transmit
/// the Data over
pub async fn create_sender_receiver<R, S>(
    mut rx: R,
    mut tx: S,
    mut tls_session: rustls::ServerSession,
) -> Result<(tls::Receiver<R>, tls::Sender<S>), Error>
where
    R: Receiver + Send,
    S: Sender + Send,
{
    complete_handshake(&mut rx, &mut tx, &mut tls_session).await?;

    let final_tls = std::sync::Arc::new(std::sync::Mutex::new(tls_session));
    Ok((
        tls::Receiver::new(rx, final_tls.clone()),
        tls::Sender::new(tx, final_tls),
    ))
}
