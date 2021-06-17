use std::sync::Arc;

use futures::FutureExt;
use serde_json::json;
use tracing::Level;
use wasmer::{Module, Store};

use crate::{
    handler::traits::Handler,
    internal_services::DashboardEntity,
    plugins::{
        api::{self, PluginEnv},
        start_instance,
    },
};

use super::InstantiatePlugin;

mod sender;
pub use sender::AcceptorPluginSender;
mod receiver;
pub use receiver::AcceptorPluginReceiver;

/// A Single instance of a loaded Acceptor-Plugin
pub struct AcceptorPluginInstance {
    name: String,
    store: Store,
    module: Module,
    config: Arc<Vec<u8>>,
}

pub struct AcceptorMessage {
    pub id: i32,
    pub data: Vec<u8>,
}

pub struct AcceptorQueueReceiver {
    rx: std::sync::mpsc::Receiver<AcceptorMessage>,
    peeked: Option<AcceptorMessage>,
}

impl AcceptorQueueReceiver {
    pub fn new(rx: std::sync::mpsc::Receiver<AcceptorMessage>) -> Self {
        Self { rx, peeked: None }
    }

    pub fn try_recv(&mut self) -> Option<AcceptorMessage> {
        if self.peeked.is_some() {
            return std::mem::replace(&mut self.peeked, None);
        }

        match self.rx.try_recv() {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }

    pub fn peek(&mut self) -> Option<&AcceptorMessage> {
        if self.peeked.is_some() {
            return self.peeked.as_ref();
        }

        match self.rx.try_recv() {
            Ok(v) => {
                self.peeked = Some(v);
                self.peeked.as_ref()
            }
            Err(_) => None,
        }
    }
}

impl AcceptorPluginInstance {
    #[tracing::instrument]
    async fn start_handler<H>(
        handler: Arc<H>,
        id: i32,
        receiver: AcceptorPluginReceiver,
        sender: AcceptorPluginSender,
    ) where
        H: Handler + Send + Sync + 'static,
    {
        tracing::event!(Level::INFO, "Plugin received Connection");
        handler.handle(id as u32, receiver, sender).await;
    }

    /// Actually runs the Plugin in a blocking way
    pub fn run<H>(self, handler: H)
    where
        H: Handler + Send + Sync + 'static,
    {
        let (tx, rx) = std::sync::mpsc::channel();

        let handler = Arc::new(handler);

        let env = PluginEnv::new(
            self.config.clone(),
            api::PluginContext::new_acceptor_context(
                AcceptorQueueReceiver::new(rx),
                tx,
                move |id, receiver, sender| {
                    Self::start_handler(handler.clone(), id, receiver, sender).boxed()
                },
            ),
        );

        let instance = match start_instance(&self.store, &env, &self.module) {
            Some(i) => i,
            None => {
                tracing::error!("Starting Acceptor-Instance");
                return;
            }
        };

        let run_instance = match instance.exports.get_native_function::<i32, i32>("accept") {
            Ok(f) => f,
            Err(e) => {
                tracing::error!("Handler does not contain Handle-Function: {}", e);
                return;
            }
        };

        if let Err(e) = run_instance.call(0) {
            tracing::error!("Running Plugin: {}", e);
        }
    }
}

pub struct PluginAcceptor {
    plugin: AcceptorPluginInstance,
}

impl PluginAcceptor {
    pub fn new(plugin: AcceptorPluginInstance) -> Self {
        Self { plugin }
    }

    fn run<H>(self, handler: H)
    where
        H: Handler + Send + Sync + 'static,
    {
        self.plugin.run(handler);
    }

    pub async fn start<H>(self, handler: H)
    where
        H: Handler + Send + Sync + 'static,
    {
        tracing::info!("Starting Plugin-Acceptor: {}", self.plugin.name);
        tokio::task::spawn_blocking(move || {
            self.run(handler);
        });
    }

    pub fn dashboard_entity(&self) -> PluginDashboard {
        PluginDashboard {}
    }
}

impl InstantiatePlugin for AcceptorPluginInstance {
    fn instantiate(name: String, store: Store, module: Module, config: Option<Vec<u8>>) -> Self {
        let (_config_size, config) = match config {
            Some(config) => (config.len() as i32, Arc::new(config)),
            None => (-1, Arc::new(Vec::new())),
        };

        Self {
            name,
            store,
            module,
            config,
        }
    }
}

pub struct PluginDashboard {}

impl DashboardEntity for PluginDashboard {
    fn get_type(&self) -> &str {
        "Plugin"
    }

    fn get_content(&self) -> serde_json::Value {
        json!({})
    }
}
