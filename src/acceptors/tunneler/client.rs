use std::sync::Arc;

use serde_json::json;
use tunneler_core::{client::Client as TClient, metrics::Empty, Destination};

use crate::{handler::traits::Handler, internal_services::DashboardEntity, tls};

use prometheus::Registry;

mod normal_handler;
mod tls_handler;

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
        normal_handler::register_metrics(reg.clone());
        tls_handler::register_metrics(reg);

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
        match self.tls_conf {
            Some(tls_config) => {
                let tmp = tls_handler::TLSHandler::new(handler, tls_config);
                self.client.start(Arc::new(tmp)).await;
            }
            None => {
                let tmp = normal_handler::TunnelerHandler::new(handler);
                self.client.start(Arc::new(tmp)).await;
            }
        }
    }
}

/// The Dashboard-Entity for the Tunneler-Acceptor
pub struct TunnelerAcceptor;

impl TunnelerAcceptor {
    /// Creates a new Empty Entity
    pub fn new() -> Self {
        Self {}
    }
}

impl DashboardEntity for TunnelerAcceptor {
    fn get_type(&self) -> &str {
        "Tunneler"
    }

    fn get_content(&self) -> serde_json::Value {
        json!({})
    }
}
