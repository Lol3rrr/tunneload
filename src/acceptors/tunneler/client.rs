use tunneler_core::client::queues::Sender;
use tunneler_core::client::Client as TClient;
use tunneler_core::message::Message;
use tunneler_core::streams::mpsc::StreamReader;
use tunneler_core::Destination;

use crate::handler::traits::Handler;

mod handle;

pub struct Client {
    client: TClient,
}

impl Client {
    /// Creates a new valid instance of a Client that is
    /// ready to be started
    pub fn new(dest: Destination, key: Vec<u8>) -> Self {
        let tunneler_client = TClient::new(dest, key);

        Self {
            client: tunneler_client,
        }
    }

    /// Handles all new connections from the Tunneler
    async fn tunneler_handler<T>(_id: u32, rx: StreamReader<Message>, tx: Sender, data: Option<T>)
    where
        T: Handler + Send + 'static + Sync,
    {
        tokio::task::spawn(handle::handle(rx, tx, data.unwrap()));
    }

    /// Starts the tunneler client itself
    pub async fn start<T>(self, handler: T)
    where
        T: Handler + Send + Clone + 'static + Sync,
    {
        self.client
            .start(Client::tunneler_handler, Some(handler))
            .await
    }
}
