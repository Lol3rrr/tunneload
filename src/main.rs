use tunneler_core::Destination;
use tunneload::acceptors::{tunneler, webserver};
use tunneload::cli;
use tunneload::configurator;
use tunneload::general;
use tunneload::handler::BasicHandler;
use tunneload::metrics;
use tunneload::rules;

use structopt::StructOpt;

use prometheus::Registry;

use log::info;

fn main() {
    env_logger::init();

    let metrics_registry = Registry::new();

    let config = cli::Options::from_args();

    let (read_manager, write_manager) = rules::new();

    let handler = BasicHandler::new(read_manager, metrics_registry.clone());

    let threads = 6;
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threads)
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    let mut config_builder = configurator::Manager::builder();
    config_builder = config_builder.writer(write_manager);

    if config.is_kubernetes_enabled() {
        info!("Enabling Kubernetes-Configurator");
        let mut k8s_manager =
            rt.block_on(configurator::kubernetes::Loader::new("default".to_owned()));

        if config.kube_traefik {
            info!("Enabling Traefik-Kubernetes-Configurator");
            k8s_manager.enable_traefik();
        }
        if config.kube_ingress {
            info!("Enabling Ingress-Kubernetes-Configurator");
            k8s_manager.enable_ingress();
        }
        config_builder = config_builder.configurator(k8s_manager);
    }
    if let Some(path) = config.file {
        info!("Enabling File-Configurator");

        let file_manager = configurator::files::Loader::new(path);
        config_builder = config_builder.configurator(file_manager);
    }

    let config_manager = config_builder.build();
    let config_wait_time =
        general::parse_time(&std::env::var("UTIME").unwrap_or_else(|_| "30s".to_owned())).unwrap();
    rt.spawn(config_manager.update_loop(config_wait_time));

    if let Some(port) = config.metrics {
        info!("Starting Metrics-Endpoint...");

        let endpoint = metrics::Endpoint::new(metrics_registry.clone());
        rt.spawn(endpoint.start(port));
    }

    let mut acceptor_futures = Vec::new();
    if let Some(port) = config.webserver {
        info!("Starting Webserver...");

        let web_server = webserver::Server::new(port, metrics_registry.clone());
        acceptor_futures.push(rt.spawn(web_server.start(handler.clone())));
    }

    if config.tunneler {
        info!("Starting Tunneler...");

        let key_file = match std::env::var("KEY_FILE") {
            Ok(val) => val,
            Err(_) => {
                let mut key_path = dirs::home_dir().unwrap();
                key_path.push(".tunneler");
                key_path.push("key");
                key_path.as_path().to_str().unwrap().to_string()
            }
        };

        let server_addr = std::env::var("SERVER_ADDR").unwrap_or_else(|_| "localhost".to_owned());
        let raw_server_port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "8081".to_owned());
        let server_port: u32 = raw_server_port.parse().unwrap();

        let raw_key = std::fs::read(key_file).expect("Reading Key File");
        let key = base64::decode(raw_key).unwrap();
        let t_client = tunneler::Client::new(
            Destination::new(server_addr, server_port),
            key,
            metrics_registry,
        );

        acceptor_futures.push(rt.spawn(t_client.start(handler)));
    }

    rt.block_on(futures::future::join_all(acceptor_futures));
}
