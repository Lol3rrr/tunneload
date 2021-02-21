use tunneler_core::client::queues::Sender;
use tunneler_core::message::Message;
use tunneler_core::streams::mpsc::StreamReader;

use crate::handler::traits::Handler;
use crate::http::streaming_parser::ReqParser;

use log::error;

/// Actually handles a new connection from a client
pub async fn handle<T>(id: u32, mut rx: StreamReader<Message>, tx: Sender, handler: T)
where
    T: Handler + Send + 'static,
{
    let mut parser = ReqParser::new_capacity(4096);

    loop {
        match rx.recv().await {
            Ok(msg) => {
                if msg.is_eof() {
                    break;
                }

                let n_data = msg.get_data();
                parser.block_parse(n_data);
            }
            Err(e) => {
                error!("Receiving Message: {}", e);
                break;
            }
        };
    }

    let req = match parser.finish() {
        Some(r) => r,
        None => {
            error!("[{}] Could not parse request", id);
            return;
        }
    };

    handler.handle(id, req, tx).await;
}
