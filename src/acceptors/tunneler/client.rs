use std::sync::Arc;

use tunneler_core::{
    client::{Client as TClient, Handler as THandler, QueueSender},
    message::Message,
    metrics::Empty,
    streams::mpsc,
    Destination, Details,
};

use crate::{
    acceptors::tunneler::{Receiver, Sender},
    handler::traits::Handler,
    tls,
};

use async_trait::async_trait;

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

/// A single Instance of the Tunneler-Acceptor that
/// receives Requests from a single Tunneler-Server
pub struct Client {
    client: TClient<Empty>,
    tls_conf: Option<tls::ConfigManager>,
}

impl Client {
    /// Creates a new Instance of the Tunneler-Acceptor
    /// that is ready to be started
    pub fn new(
        dest: Destination,
        external: u16,
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

        let tunneler_client = TClient::new(dest, external, key);

        Self {
            client: tunneler_client,
            tls_conf: tls_opt,
        }
    }

    /// Connects the Tunneler-Client to the external Tunneler-Server
    /// and starts waiting for incoming Connections
    pub async fn start<T>(self, handler: T)
    where
        T: Handler + Send + Clone + 'static + Sync,
    {
        let t_handler = TunnelerHandler::new(handler, self.tls_conf);

        self.client.start(Arc::new(t_handler)).await
    }
}

struct TunnelerHandler<H> {
    handler: H,
    tls_conf: Option<tls::ConfigManager>,
}

impl<H> TunnelerHandler<H>
where
    H: Handler + Send + Sync,
{
    pub fn new(handler: H, tls_conf: Option<tls::ConfigManager>) -> Self {
        Self { handler, tls_conf }
    }

    /// Actually runs the Handle function and is used to therefore
    /// abstract away the actual execution as well as encapsulates
    /// any errors produced by it
    #[inline(always)]
    async fn run_handle<T, R, S>(
        id: u32,
        mut rx: Receiver<R>,
        mut tx: Sender<S>,
        handler: &T,
        tls_conf: Option<&tls::ConfigManager>,
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
}

#[async_trait]
impl<H> THandler for TunnelerHandler<H>
where
    H: Handler + Send + Sync + 'static,
{
    async fn new_con(
        self: Arc<Self>,
        id: u32,
        _details: Details,
        rx: mpsc::StreamReader<Message>,
        tx: QueueSender,
    ) {
        OPEN_COONECTIONS.inc();

        let receiver = Receiver::new(rx);
        let sender = Sender::new(tx);

        // Actually runs and executes the Handler with the given
        // Data
        Self::run_handle(id, receiver, sender, &self.handler, self.tls_conf.as_ref()).await;

        OPEN_COONECTIONS.dec();
    }
}
