use tokio::task::JoinHandle;

use tunneload::{
    acceptors::{tunneler, webserver},
    cli,
    configurator::{self, Manager},
    forwarder::BasicForwarder,
    handler::{traits::Handler, BasicHandler},
    internal_services::{Dashboard, DashboardEntityList, Internals},
    metrics, plugins, rules,
    tls::{self, auto::StoreTLS},
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

        let (_, service_list, middleware_list, action_plugin_list) =
            config_manager.get_config_lists();
        let mut internal_dashboard = Dashboard::new(
            read_manager.clone(),
            service_list,
            middleware_list,
            DashboardEntityList::new(),
            dashboard_configurators,
            action_plugin_list,
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

    rt.block_on(setup_auto_tls(
        &config,
        &mut internals,
        &mut config_manager,
        tls_config.clone(),
    ));

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
    config_builder = configurator::kubernetes::setup(
        rt,
        &config.kubernetes,
        config_builder,
        dashboard_configurators,
    );

    config_builder = configurator::files::setup(&config, config_builder, dashboard_configurators);

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

    acceptor_futures.extend(webserver::setup(
        rt,
        &config.webserver,
        tls_config.clone(),
        handler.clone(),
        &metrics_registry,
    ));

    acceptor_futures.extend(tunneler::setup(
        rt,
        &config.tunneler,
        handler,
        tls_config,
        &metrics_registry,
    ));

    acceptor_futures
}

async fn setup_auto_tls(
    config: &cli::Options,
    internals: &mut Internals,
    config_manager: &mut Manager,
    tls_config: tls::ConfigManager,
) {
    if config.auto_tls.auto_tls_enabled {
        log::info!("Enabled Auto-TLS");

        let (rule_list, service_list, _, _) = config_manager.get_config_lists();

        let env = if config.auto_tls.auto_tls_production {
            tls::auto::Environment::Production
        } else {
            tls::auto::Environment::Staging
        };
        let contacts = Vec::new();

        let (internal_acme, auto_session) =
            tls::auto::new(env, contacts, rule_list, service_list, tls_config).await;

        // TODO
        let mut tls_stores: Vec<Box<dyn StoreTLS + Send + Sync + 'static>> = Vec::new();
        tls_stores.push(Box::new(tls::stores::kubernetes::KubeStore {}));

        config_manager.register_internal_service(&internal_acme);
        internals.add_service(Box::new(internal_acme));
        let tx = auto_session.start(tls_stores);

        config_manager.update_tls_queue(Some(tx));
    }
}
