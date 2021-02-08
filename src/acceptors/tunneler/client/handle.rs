use tunneler_core::client::queues::Sender;
use tunneler_core::message::Message;
use tunneler_core::streams::mpsc::StreamReader;

use crate::handler::traits::Handler;
use crate::http::Request;

use log::error;

/// Actually handles a new connection from a client
pub async fn handle<T>(mut rx: StreamReader<Message>, tx: Sender, handler: T)
where
    T: Handler + Send + 'static,
{
    let mut buffer = Vec::with_capacity(4096);

    loop {
        match rx.recv().await {
            Ok(msg) => {
                let n_data = msg.get_data();
                buffer.extend_from_slice(n_data);

                if n_data.len() < 4092 {
                    break;
                }
            }
            Err(e) => {
                error!("Receiving Message: {}", e);
                return;
            }
        };
    }

    let req = match Request::parse(&buffer) {
        Some(r) => r,
        None => {
            error!("Could not parse request");
            return;
        }
    };

    handler.handle(req, tx).await;
}
