use tunneler_core::client::Client as TClient;
use tunneler_core::Destination;

use crate::acceptors::tunneler::Receiver;
use crate::acceptors::tunneler::Sender;
use crate::handler::traits::Handler;
use crate::tls;

use lazy_static::lazy_static;
use prometheus::Registry;

use log::error;

lazy_static! {
    static ref OPEN_COONECTIONS: prometheus::IntGauge = prometheus::IntGauge::new(
        "tunneler_open_connections",
        "The Number of currently open Connections from the Tunneler-Acceptor"
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
        match reg.register(Box::new(OPEN_COONECTIONS.clone())) {
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

    /// Actually runs the Handle function and is used to therefore
    /// abstract away the actual execution as well as encapsulates
    /// any errors produced by it
    #[inline(always)]
    async fn run_handle<T, R, S>(
        id: u32,
        mut rx: Receiver<R>,
        mut tx: Sender<S>,
        handler: T,
        tls_conf: Option<tls::ConfigManager>,
    ) where
        T: Handler + Send + Sync + 'static,
        R: tunneler_core::client::Receiver + Send + Sync,
        S: tunneler_core::client::Sender + Send + Sync,
    {
        match tls_conf {
            Some(tls_config) => {
                let config = tls_config.get_config();
                let session = rustls::ServerSession::new(&config);

                let (mut tls_receiver, mut tls_sender) =
                    match tls::create_sender_receiver(&mut rx, &mut tx, session).await {
                        Some(s) => s,
                        None => {
                            error!("[{}] Creating TLS-Session", id);
                            return;
                        }
                    };

                handler.handle(id, &mut tls_receiver, &mut tls_sender).await;
            }
            None => {
                handler.handle(id, &mut rx, &mut tx).await;
            }
        }
    }

    /// Handles all new connections from the Tunneler
    async fn tunneler_handler<T, R, S>(
        id: u32,
        rx: R,
        tx: S,
        data: Option<(T, Option<tls::ConfigManager>)>,
    ) where
        R: tunneler_core::client::Receiver + Send + Sync,
        S: tunneler_core::client::Sender + Send + Sync,
        T: Handler + Send + 'static + Sync,
    {
        let (handler, tls_conf) = data.unwrap();

        OPEN_COONECTIONS.inc();

        let receiver = Receiver::new(rx);
        let sender = Sender::new(tx);

        // Actually runs and executes the Handler with the given
        // Data
        Self::run_handle(id, receiver, sender, handler, tls_conf).await;

        OPEN_COONECTIONS.dec();
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
