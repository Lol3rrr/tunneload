use std::collections::HashMap;

use tokio::task::JoinHandle;
use tunneler_core::Destination;

use crate::{cli, tls};
use general_traits::Handler;

use super::Client;

/// This handles all the initial Setup for the Tunneler-Acceptor based on the
/// provided Configuration
pub fn setup<H>(
    rt: &tokio::runtime::Runtime,
    config: &HashMap<String, cli::TunnelerOpts>,
    handler: H,
    tls_config: tls::ConfigManager,
    metrics_registry: &prometheus::Registry,
) -> Vec<JoinHandle<()>>
where
    H: Handler + Clone + Send + Sync + 'static,
{
    let mut result = Vec::new();

    for (name, conf) in config.iter() {
        tracing::info!("Starting Tunneler-{} ...", name);

        let raw_key = std::fs::read(&conf.key).expect("Reading Key File");
        let key = base64::decode(raw_key).expect("The Key should be valid base64");
        let tls_conf = if conf.tls {
            Some(tls_config.clone())
        } else {
            None
        };
        let t_client = Client::new(
            Destination::new(conf.addr.clone(), conf.port),
            conf.public_port,
            key,
            metrics_registry.clone(),
            tls_conf,
        );

        result.push(rt.spawn(t_client.start(handler.clone())));
    }

    result
}
