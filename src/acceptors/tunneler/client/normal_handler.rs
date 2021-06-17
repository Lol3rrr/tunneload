use std::sync::Arc;

use prometheus::Registry;
use tunneler_core::{
    client::Handler as THandler, client::QueueSender, message::Message, streams::mpsc, Details,
};

use crate::{
    acceptors::tunneler::{Receiver, Sender},
    handler::traits::Handler,
};

use async_trait::async_trait;

use lazy_static::lazy_static;

lazy_static! {
    static ref OPEN_CONNECTIONS: prometheus::IntGauge = prometheus::IntGauge::new(
        "tunneler_open_connections",
        "The Number of currently open Connections from the Tunneler-Acceptor"
    )
    .unwrap();
    static ref OPEN_TIME: prometheus::Histogram =
        prometheus::Histogram::with_opts(prometheus::HistogramOpts::new(
            "tunneler_open_time",
            "The Duration for which the Connections are kept open on the Tunneler-Acceptor"
        ))
        .unwrap();
}

/// Registers the non TLS related metrics for Tunneler
pub fn register_metrics(reg: Registry) {
    if let Err(e) = reg.register(Box::new(OPEN_CONNECTIONS.clone())) {
        tracing::error!("Registering Open-Connections metrics: {}", e);
    }
    if let Err(e) = reg.register(Box::new(OPEN_TIME.clone())) {
        tracing::error!("Registering Open-Time metrics: {}", e);
    }
}

pub struct TunnelerHandler<H> {
    handler: H,
}

impl<H> TunnelerHandler<H>
where
    H: Handler + Send + Sync,
{
    pub fn new(handler: H) -> Self {
        Self { handler }
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
        OPEN_CONNECTIONS.inc();

        let receiver = Receiver::new(rx);
        let sender = Sender::new(tx);

        let open_timer = OPEN_TIME.start_timer();

        self.handler.handle(id, receiver, sender).await;

        open_timer.observe_duration();
        OPEN_CONNECTIONS.dec();
    }
}
