use tunneler_core::Destination;
use tunneload::acceptors::{tunneler, webserver};
use tunneload::cli;
use tunneload::configurator;
use tunneload::general;
use tunneload::handler::BasicHandler;
use tunneload::metrics;
use tunneload::rules;
use tunneload::tls;

use structopt::StructOpt;

use prometheus::Registry;

use log::info;

fn main() {
    env_logger::init();

    let metrics_registry = Registry::new_custom(Some("tunneload".to_owned()), None).unwrap();

    let config = cli::Options::from_args();

    let (read_manager, write_manager) = rules::new();

    let handler = BasicHandler::new(read_manager, metrics_registry.clone());

    let threads = match std::env::var("THREADS") {
        Ok(raw) => raw.parse().unwrap_or(6),
        Err(_) => 6,
    };
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threads)
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    let mut config_builder = configurator::Manager::builder();
    config_builder = config_builder.writer(write_manager);

    if let Ok(raw_time) = std::env::var("UTIME") {
        if let Some(time) = general::parse_time(&raw_time) {
            config_builder = config_builder.wait_time(time);
        }
    }

    let tls_config = tls::ConfigManager::new();
    config_builder = config_builder.tls(tls_config.clone());

    if config.kubernetes.is_enabled() {
        info!("Enabling Kubernetes-Configurator");
        let kube_conf = config.kubernetes;
        let mut k8s_manager =
            rt.block_on(configurator::kubernetes::Loader::new("default".to_owned()));

        if kube_conf.traefik {
            info!("Enabling Traefik-Kubernetes-Configurator");
            k8s_manager.enable_traefik();
        }
        if kube_conf.ingress {
            info!("Enabling Ingress-Kubernetes-Configurator");
            k8s_manager.enable_ingress();

            // Checks if a new Priority has been set and if that is
            // the case, overwrites the old default one
            if let Some(n_priority) = kube_conf.ingress_priority {
                k8s_manager.set_ingress_priority(n_priority);
            }
        }
        config_builder = config_builder.configurator(k8s_manager);
    }
    if let Some(path) = config.file {
        info!("Enabling File-Configurator");

        let file_manager = configurator::files::Loader::new(path);
        config_builder = config_builder.configurator(file_manager);
    }

    let config_manager = config_builder.build();
    rt.spawn(config_manager.start());

    if let Some(port) = config.metrics {
        info!("Starting Metrics-Endpoint...");

        let endpoint = metrics::Endpoint::new(metrics_registry.clone());
        rt.spawn(endpoint.start(port));
    }

    let mut acceptor_futures = Vec::new();
    if let Some(port) = config.webserver.port {
        info!("Starting Non-TLS Webserver...");

        let web_server = webserver::Server::new(port, metrics_registry.clone(), None);
        acceptor_futures.push(rt.spawn(web_server.start(handler.clone())));
    }
    if let Some(port) = config.webserver.tls_port {
        info!("Starting TLS Webserver...");

        let web_server =
            webserver::Server::new(port, metrics_registry.clone(), Some(tls_config.clone()));
        acceptor_futures.push(rt.spawn(web_server.start(handler.clone())));
    }

    if config.tunneler.is_normal_enabled() {
        info!("Starting Non-TLS Tunneler...");

        let (key_file, server_addr, server_port) = config.tunneler.get_normal_with_defaults();

        let raw_key = std::fs::read(key_file).expect("Reading Key File");
        let key = base64::decode(raw_key).unwrap();
        let t_client = tunneler::Client::new(
            Destination::new(server_addr, server_port),
            key,
            metrics_registry.clone(),
            None,
        );

        acceptor_futures.push(rt.spawn(t_client.start(handler.clone())));
    }
    if config.tunneler.is_tls_enabled() {
        info!("Starting TLS Tunneler...");

        let (key_file, server_addr, server_port) = config.tunneler.get_tls_with_defaults();

        let raw_key = std::fs::read(key_file).expect("Reading Key File");
        let key = base64::decode(raw_key).unwrap();
        let t_client = tunneler::Client::new(
            Destination::new(server_addr, server_port),
            key,
            metrics_registry,
            Some(tls_config),
        );

        acceptor_futures.push(rt.spawn(t_client.start(handler)));
    }

    rt.block_on(futures::future::join_all(acceptor_futures));
}
