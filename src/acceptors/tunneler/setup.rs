use tokio::task::JoinHandle;
use tunneler_core::Destination;

use crate::{cli, handler::traits::Handler, tls};

use super::Client;

/// This handles all the initial Setup for the Tunneler-Acceptor based on the
/// provided Configuration
pub fn setup<H>(
    rt: &tokio::runtime::Runtime,
    config: &cli::TunnelerOpts,
    handler: H,
    tls_config: tls::ConfigManager,
    metrics_registry: &prometheus::Registry,
) -> Vec<JoinHandle<()>>
where
    H: Handler + Clone + Send + Sync + 'static,
{
    let mut result = Vec::new();

    if config.is_normal_enabled() {
        tracing::info!("Starting Non-TLS Tunneler...");

        let (key_file, server_addr, server_port) = config.get_normal_with_defaults();

        let raw_key = std::fs::read(key_file).expect("Reading Key File");
        let key = base64::decode(raw_key).unwrap();
        let t_client = Client::new(
            Destination::new(server_addr, server_port),
            80,
            key,
            metrics_registry.clone(),
            None,
        );

        result.push(rt.spawn(t_client.start(handler.clone())));
    }
    if config.is_tls_enabled() {
        tracing::info!("Starting TLS Tunneler...");

        let (key_file, server_addr, server_port) = config.get_tls_with_defaults();

        let raw_key = std::fs::read(key_file).expect("Reading Key File");
        let key = base64::decode(raw_key).unwrap();
        let t_client = Client::new(
            Destination::new(server_addr, server_port),
            443,
            key,
            metrics_registry.clone(),
            Some(tls_config),
        );

        result.push(rt.spawn(t_client.start(handler)));
    }

    result
}
