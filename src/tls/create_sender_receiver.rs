use crate::acceptors::traits::{Receiver, Sender};
use crate::tls;

use rustls::Session;

pub async fn create_sender_receiver<'a, R, S>(
    rx: &'a mut R,
    tx: &'a mut S,
    mut tls_session: rustls::ServerSession,
) -> Option<(tls::Receiver<'a, R>, tls::Sender<'a, S>)>
where
    R: Receiver + Send,
    S: Sender + Send,
{
    // Handles the TLS-Handshake
    loop {
        if tls_session.wants_read() {
            let mut tmp = [0; 2048];
            let read = match rx.read(&mut tmp).await {
                Ok(n) if n == 0 => {
                    return None;
                }
                Ok(n) => n,
                Err(_) => {
                    return None;
                }
            };

            let mut read_data = &tmp[..read];
            if let Err(_) = tls_session.read_tls(&mut read_data) {
                return None;
            }

            if let Err(_) = tls_session.process_new_packets() {
                return None;
            }
            continue;
        }

        if tls_session.wants_write() {
            let mut tmp_buf = Vec::with_capacity(2048);
            let written = match tls_session.write_tls(&mut tmp_buf) {
                Ok(n) => n,
                Err(_) => {
                    return None;
                }
            };

            tx.send(tmp_buf, written).await;
            continue;
        }

        break;
    }

    let final_tls = std::sync::Arc::new(std::sync::Mutex::new(tls_session));
    Some((
        tls::Receiver::new(rx, final_tls.clone()),
        tls::Sender::new(tx, final_tls),
    ))
}
