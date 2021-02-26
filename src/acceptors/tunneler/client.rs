use tunneler_core::client::queues::Sender;
use tunneler_core::client::Client as TClient;
use tunneler_core::message::Message;
use tunneler_core::streams::mpsc::StreamReader;
use tunneler_core::Destination;

use crate::acceptors::tunneler::Receiver;
use crate::handler::traits::Handler;
use crate::tls;

use lazy_static::lazy_static;
use prometheus::Registry;

use log::error;

lazy_static! {
    static ref TOTAL_REQS: prometheus::IntCounter = prometheus::IntCounter::new(
        "tunneler_req_total",
        "The total Number of requests received by the Tunneler-Acceptor"
    )
    .unwrap();
}

pub struct Client {
    client: TClient,
    tls_conf: Option<tls::ConfigManager>,
}

impl Client {
    /// Creates a new valid instance of a Client that is
    /// ready to be started
    pub fn new(
        dest: Destination,
        key: Vec<u8>,
        reg: Registry,
        tls_opt: Option<tls::ConfigManager>,
    ) -> Self {
        match reg.register(Box::new(TOTAL_REQS.clone())) {
            Ok(_) => {}
            Err(e) => {
                error!("Registering Total-Reqs Metric: {}", e);
            }
        };

        let tunneler_client = TClient::new(dest, key);

        Self {
            client: tunneler_client,
            tls_conf: tls_opt,
        }
    }

    /// Handles all new connections from the Tunneler
    async fn tunneler_handler<T>(
        id: u32,
        rx: StreamReader<Message>,
        mut tx: Sender,
        data: Option<(T, Option<tls::ConfigManager>)>,
    ) where
        T: Handler + Send + 'static + Sync,
    {
        let (handler, tls_conf) = data.unwrap();

        TOTAL_REQS.inc();

        let mut receiver = Receiver::new(rx);

        match tls_conf {
            Some(tls_config) => {
                let config = tls_config.get_config();
                let session = rustls::ServerSession::new(&config);

                let (mut tls_receiver, mut tls_sender) =
                    match tls::create_sender_receiver(&mut receiver, &mut tx, session).await {
                        Some(s) => s,
                        None => {
                            error!("[{}] Creating TLS-Session", id);
                            return;
                        }
                    };

                handler.handle(id, &mut tls_receiver, &mut tls_sender).await;
            }
            None => {
                handler.handle(id, &mut receiver, &mut tx).await;
            }
        }
    }

    /// Starts the tunneler client itself
    pub async fn start<T>(self, handler: T)
    where
        T: Handler + Send + Clone + 'static + Sync,
    {
        self.client
            .start(
                Client::tunneler_handler,
                Some((handler, self.tls_conf.clone())),
            )
            .await
    }
}
