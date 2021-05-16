use crate::acceptors::traits::{Receiver, Sender};

/// Sends the messages from the Client to the Backend Server
pub async fn run_receiver<R>(mut rx: R, mut target: tokio::net::tcp::OwnedWriteHalf)
where
    R: Receiver + Send,
{
    log::info!("Starting Websocket Receiver");
}

/// Sends the messages from the Backend server to the Client
pub async fn run_sender<S>(mut tx: S, mut receiver: tokio::net::tcp::OwnedReadHalf)
where
    S: Sender + Send,
{
    log::info!("Starting Websocket Sender");
}
