use std::sync::Arc;

use prometheus::Registry;
use tunneler_core::{
    client::Handler as THandler, client::QueueSender, message::Message, streams::mpsc, Details,
};

use crate::{
    acceptors::tunneler::{Receiver, Sender},
    handler::traits::Handler,
    tls,
};

use async_trait::async_trait;

use lazy_static::lazy_static;

lazy_static! {
    static ref OPEN_CONNECTIONS: prometheus::IntGauge = prometheus::IntGauge::new(
        "tunneler_tls_open_connections",
        "The Number of currently open Connections from the TLS Tunneler-Acceptor"
    )
    .unwrap();
    static ref OPEN_TIME: prometheus::Histogram =
        prometheus::Histogram::with_opts(prometheus::HistogramOpts::new(
            "tunneler_tls_open_time",
            "The Duration for which the Connections are kept open on the TLS Tunneler-Acceptor"
        ))
        .unwrap();
}

/// Registers the TLS related metrics for Tunneler
pub fn register_metrics(reg: Registry) {
    if let Err(e) = reg.register(Box::new(OPEN_CONNECTIONS.clone())) {
        tracing::error!("Registering Open-Connections metrics: {}", e);
    }
    if let Err(e) = reg.register(Box::new(OPEN_TIME.clone())) {
        tracing::error!("Registering Open-Time metrics: {}", e);
    }
}

pub struct TLSHandler<H> {
    handler: H,
    tls_config: tls::ConfigManager,
}

impl<H> TLSHandler<H>
where
    H: Handler + Send + Sync,
{
    pub fn new(handler: H, tls_conf: tls::ConfigManager) -> Self {
        Self {
            handler,
            tls_config: tls_conf,
        }
    }
}

#[async_trait]
impl<H> THandler for TLSHandler<H>
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
        OPEN_CONNECTIONS.inc();

        let raw_receiver = Receiver::new(rx);
        let raw_sender = Sender::new(tx);

        let config = self.tls_config.get_config();
        let session = rustls::ServerSession::new(&config);

        let (receiver, sender) =
            match tls::create_sender_receiver(raw_receiver, raw_sender, session).await {
                Some(s) => s,
                None => {
                    tracing::error!("[{}] Creating TLS-Session", id);
                    return;
                }
            };

        let open_timer = OPEN_TIME.start_timer();

        self.handler.handle(id, receiver, sender).await;

        open_timer.observe_duration();
        OPEN_CONNECTIONS.dec();
    }
}
