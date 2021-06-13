use tokio::io::AsyncWriteExt;

use crate::{
    acceptors::traits::{Receiver, Sender},
    websockets::DataFrame,
};

/// Sends the messages from the Client to the Backend Server
pub async fn run_receiver<R>(mut rx: R, mut target: tokio::net::tcp::OwnedWriteHalf)
where
    R: Receiver + Send,
{
    log::info!("Starting Websocket Receiver");

    loop {
        let frame = match DataFrame::receive(&mut rx).await {
            Some(f) => f,
            None => {
                log::error!("Receiving DataFrame from the Client");
                break;
            }
        };

        let serialized_frame = frame.serialize();
        if let Err(e) = target.write_all(&serialized_frame).await {
            log::error!("[WS] Failed to send DataFrame: {:?}", e);
        }
    }
}

/// Sends the messages from the Backend server to the Client
pub async fn run_sender<S>(mut tx: S, mut receiver: tokio::net::tcp::OwnedReadHalf)
where
    S: Sender + Send,
{
    log::info!("Starting Websocket Sender");

    loop {
        let frame = match DataFrame::receive(&mut receiver).await {
            Some(f) => f,
            None => {
                log::error!("Receiving DataFrame from the Backend-Service");
                break;
            }
        };

        let serialized_frame = frame.serialize();
        tx.send(&serialized_frame).await;
    }
}
