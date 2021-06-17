use tokio::task::JoinHandle;

use super::Server;
use crate::{cli::WebserverOpts, handler::traits::Handler, tls};

/// This handles all the Setup related stuff for the Webserver, according to the
/// provided Configuration
pub fn setup<H>(
    rt: &tokio::runtime::Runtime,
    config: &WebserverOpts,
    tls_config: tls::ConfigManager,
    handler: H,
    metrics_registry: &prometheus::Registry,
) -> Vec<JoinHandle<()>>
where
    H: Handler + Clone + Send + Sync + 'static,
{
    let mut result = Vec::new();

    if let Some(port) = config.port {
        tracing::info!("Starting Non-TLS Webserver...");

        let web_server = Server::new(port, metrics_registry.clone(), None);
        result.push(rt.spawn(web_server.start(handler.clone())));
    }
    if let Some(port) = config.tls_port {
        tracing::info!("Starting TLS Webserver...");

        let web_server = Server::new(port, metrics_registry.clone(), Some(tls_config));
        result.push(rt.spawn(web_server.start(handler)));
    }

    result
}
