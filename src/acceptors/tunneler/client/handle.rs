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

                // TODO:
                // Replace this with an actual check if its done
                if n_data.len() < 4096 {
                    break;
                }
            }
            Err(e) => {
                error!("Receiving Message: {}", e);
                return;
            }
        };
    }

    let req = match parser.finish() {
        Ok(r) => r,
        Err(e) => {
            error!("[{}] Could not parse request: {}", id, e);
            return;
        }
    };

    handler.handle(id, req, tx).await;
}
