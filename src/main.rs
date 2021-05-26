use tokio::task::JoinHandle;
use tunneler_core::Destination;

use tunneload::{
    acceptors::{tunneler, webserver},
    cli, configurator,
    forwarder::BasicForwarder,
    handler::{traits::Handler, BasicHandler},
    internal_services::{Dashboard, DashboardEntityList, Internals},
    metrics, plugins, rules, tls,
};

use structopt::StructOpt;

use lazy_static::lazy_static;
use prometheus::Registry;

use log::info;

lazy_static! {
    static ref RUNTIME_THREADS: prometheus::IntGauge = prometheus::IntGauge::new(
        "runtime_threads",
        "The Number of threads running in the Runtime"
    )
    .unwrap();
}

fn main() {
    env_logger::init();

    let metrics_registry = Registry::new_custom(Some("tunneload".to_owned()), None).unwrap();
    configurator::Manager::register_metrics(metrics_registry.clone());

    metrics_registry
        .register(Box::new(RUNTIME_THREADS.clone()))
        .unwrap();

    let config = cli::Options::from_args();

    let (read_manager, write_manager) = rules::new();

    let threads = match std::env::var("THREADS") {
        Ok(raw) => raw.parse().unwrap_or(6),
        Err(_) => 6,
    };
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threads)
        .enable_io()
        .enable_time()
        .on_thread_start(|| RUNTIME_THREADS.inc())
        .on_thread_stop(|| RUNTIME_THREADS.dec())
        .build()
        .unwrap();

    let mut config_builder = configurator::Manager::builder();
    config_builder = config_builder.writer(write_manager);

    let tls_config = tls::ConfigManager::new();
    config_builder = config_builder.tls(tls_config.clone());

    let mut dashboard_configurators = DashboardEntityList::new();

    config_builder =
        setup_configurators(&rt, &config, config_builder, &mut dashboard_configurators);

    if let Some(path) = config.plugin_file.clone() {
        log::info!("Enabling Plugins");

        let plugin_manager = plugins::Loader::new(path);

        config_builder = config_builder.plugin_loader(plugin_manager);
    }

    if let Some(port) = config.metrics {
        info!("Starting Metrics-Endpoint...");

        let endpoint = metrics::Endpoint::new(metrics_registry.clone());
        rt.spawn(endpoint.start(port));
    }

    let mut config_manager = config_builder.build();

    let mut internals = Internals::new();

    if config.dashboard {
        log::info!("Enabled the internal Dashboard");

        let (_, service_list, middleware_list) = config_manager.get_config_lists();
        let mut internal_dashboard = Dashboard::new(
            read_manager.clone(),
            service_list,
            middleware_list,
            DashboardEntityList::new(),
            dashboard_configurators,
        );

        if let Some(port) = config.webserver.port {
            internal_dashboard.add_acceptor(webserver::WebAcceptor::new(port));
        }
        if let Some(port) = config.webserver.tls_port {
            internal_dashboard.add_acceptor(webserver::WebAcceptor::new(port));
        }
        if config.tunneler.is_normal_enabled() {
            internal_dashboard.add_acceptor(tunneler::TunnelerAcceptor::new(80));
        }
        if config.tunneler.is_tls_enabled() {
            internal_dashboard.add_acceptor(tunneler::TunnelerAcceptor::new(443));
        }

        config_manager.register_internal_service(&internal_dashboard);
        internals.add_service(Box::new(internal_dashboard));
    }

    rt.spawn(config_manager.start());

    let forwarder = BasicForwarder::new();
    let handler = BasicHandler::new(
        read_manager,
        forwarder,
        internals,
        Some(metrics_registry.clone()),
    );

    let acceptor_futures = setup_acceptors(&rt, &config, handler, tls_config, metrics_registry);

    rt.block_on(futures::future::join_all(acceptor_futures));
}

fn setup_configurators(
    rt: &tokio::runtime::Runtime,
    config: &cli::Options,
    mut config_builder: configurator::ManagerBuilder,
    dashboard_configurators: &mut DashboardEntityList,
) -> configurator::ManagerBuilder {
    if config.kubernetes.is_enabled() {
        let mut kube_dashboard = configurator::kubernetes::KubernetesConfigurator::new();

        info!("Enabling Kubernetes-Configurator");
        let kube_conf = &config.kubernetes;
        let mut k8s_manager =
            rt.block_on(configurator::kubernetes::Loader::new("default".to_owned()));

        if kube_conf.traefik {
            info!("Enabling Traefik-Kubernetes-Configurator");
            k8s_manager.enable_traefik();
            kube_dashboard.enable_traefik();
        }
        if kube_conf.ingress {
            info!("Enabling Ingress-Kubernetes-Configurator");
            k8s_manager.enable_ingress();
            kube_dashboard.enable_ingress();

            // Checks if a new Priority has been set and if that is
            // the case, overwrites the old default one
            if let Some(n_priority) = kube_conf.ingress_priority {
                k8s_manager.set_ingress_priority(n_priority);
            }
        }
        config_builder = config_builder.configurator(k8s_manager);

        dashboard_configurators.push(Box::new(kube_dashboard));
    }
    if let Some(path) = config.file.clone() {
        info!("Enabling File-Configurator");

        let (file_manager, file_configurator) = configurator::files::new(path);
        config_builder = config_builder.configurator(file_manager);

        dashboard_configurators.push(Box::new(file_configurator));
    }

    config_builder
}

fn setup_acceptors<H>(
    rt: &tokio::runtime::Runtime,
    config: &cli::Options,
    handler: H,
    tls_config: tls::ConfigManager,
    metrics_registry: prometheus::Registry,
) -> Vec<JoinHandle<()>>
where
    H: Clone + Handler + Send + Sync + 'static,
{
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
            80,
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
            443,
            key,
            metrics_registry,
            Some(tls_config),
        );

        acceptor_futures.push(rt.spawn(t_client.start(handler)));
    }

    acceptor_futures
}
