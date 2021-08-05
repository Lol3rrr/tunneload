use std::collections::HashMap;

use tokio::task::JoinHandle;

use super::Server;
use crate::{cli::WebserverOpts, handler::traits::Handler, tls};

/// This handles all the Setup related stuff for the Webserver, according to the
/// provided Configuration
pub fn setup<H>(
    rt: &tokio::runtime::Runtime,
    config: &HashMap<String, WebserverOpts>,
    tls_config: tls::ConfigManager,
    handler: H,
    metrics_registry: &prometheus::Registry,
) -> Vec<JoinHandle<()>>
where
    H: Handler + Clone + Send + Sync + 'static,
{
    let mut result = Vec::new();

    for (name, conf) in config {
        tracing::info!("Starting Webserver-{} ...", name);

        let tls_conf = if conf.tls {
            Some(tls_config.clone())
        } else {
            None
        };

        let web_server = Server::new(conf.port, metrics_registry.clone(), tls_conf);
        result.push(rt.spawn(web_server.start(handler.clone())));
    }

    result
}
