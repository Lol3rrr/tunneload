use tunneler_core::client::queues::Sender;
use tunneler_core::client::Client as TClient;
use tunneler_core::message::Message;
use tunneler_core::streams::mpsc::StreamReader;
use tunneler_core::Destination;

use crate::handler::traits::Handler;
use crate::http::streaming_parser::ReqParser;

use lazy_static::lazy_static;
use prometheus::Registry;

use log::error;

lazy_static! {
    static ref TOTAL_REQS: prometheus::IntCounter = prometheus::IntCounter::new("tunneler_req_total", "The total Number of requests received by the Tunneler-Acceptor").unwrap();
    static ref PARSE_TIME: prometheus::Histogram = prometheus::Histogram::with_opts(prometheus::HistogramOpts::new("tunneler_req_parsing", "The Time, in seconds, it takes for a request to be fully received and parsed by the Tunneler-Acceptor")).unwrap();
}

pub struct Client {
    client: TClient,
}

impl Client {
    /// Creates a new valid instance of a Client that is
    /// ready to be started
    pub fn new(dest: Destination, key: Vec<u8>, reg: Registry) -> Self {
        reg.register(Box::new(TOTAL_REQS.clone())).unwrap();
        reg.register(Box::new(PARSE_TIME.clone())).unwrap();

        let tunneler_client = TClient::new(dest, key);

        Self {
            client: tunneler_client,
        }
    }

    /// Handles all new connections from the Tunneler
    async fn tunneler_handler<T>(
        id: u32,
        mut rx: StreamReader<Message>,
        tx: Sender,
        data: Option<T>,
    ) where
        T: Handler + Send + 'static + Sync,
    {
        let handler = data.unwrap();

        TOTAL_REQS.inc();
        let parse_timer = PARSE_TIME.start_timer();

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

        parse_timer.observe_duration();

        handler.handle(id, req, tx).await;
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
