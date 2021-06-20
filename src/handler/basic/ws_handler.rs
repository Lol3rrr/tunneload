use std::sync::Arc;

use stream_httparse::{streaming_parser::RespParser, Request};

use crate::{
    acceptors::traits::{Receiver, Sender},
    rules::Rule,
    websockets,
};

mod websocket_con;

#[tracing::instrument(skip(resp_parser, request))]
pub async fn handle<R, S>(
    id: u32,
    request: Request<'_>,
    receiver: R,
    mut sender: S,
    matched: Arc<Rule>,
    resp_parser: &mut RespParser,
) where
    R: Receiver + Send + 'static,
    S: Sender + Send + 'static,
{
    let (read, write) =
        match websockets::handshake::handle(&request, &mut sender, &matched, resp_parser).await {
            Some(c) => c,
            None => {
                tracing::error!("[{}] Performing Websocket Handshake", id);
                return;
            }
        };

    tokio::task::spawn(websocket_con::run_receiver(receiver, write));
    tokio::task::spawn(websocket_con::run_sender(sender, read));

    return;
}
