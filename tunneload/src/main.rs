use std::{env, sync::Arc, time::Duration};

use argser::FromArgs;
use tokio::task::JoinHandle;

use general_traits::Handler;
use tunneload::{
    acceptors::{tunneler, webserver},
    cli,
    configurator::{self, Manager},
    forwarder::BasicForwarder,
    handler::BasicHandler,
    internal_services::{DashboardEntityList, Internals},
    metrics, tls,
};

use lazy_static::lazy_static;
use prometheus::Registry;

lazy_static! {
    static ref RUNTIME_THREADS: prometheus::IntGauge = prometheus::IntGauge::new(
        "runtime_threads",
        "The Number of threads running in the Runtime"
    )
    .unwrap();
}

fn help_message() {
    let args = cli::Options::arguments();
    for arg in args {
        println!(
            "* {:?} ({}) : {}",
            arg.name,
            if arg.required { "required" } else { "optional" },
            arg.description
        );
    }
}

fn main() {
    // Setup the Async-Runtime
    let rt = setup_runtime();

    // Parse the given Configuration from the CLI
    let config: cli::Options = match argser::parse_cli() {
        Ok(c) => c,
        Err(_) => {
            println!("Invalid CLI-Arguments");
            help_message();
            return;
        }
    };

    if env::args().any(|arg| arg == "help") {
        help_message();
        return;
    }

    // Setup all the Telemetry stuff (Logging, Tracing, Metrics)
    let metrics_registry = setup_telemetry(&rt, &config);
    configurator::Manager::register_metrics(metrics_registry.clone());
    metrics_registry
        .register(Box::new(RUNTIME_THREADS.clone()))
        .unwrap();

    // Create the List of Rules
    let (read_manager, write_manager) = rules::new();

    // The Builder for the Configuration-Manager
    let mut config_builder = configurator::Manager::builder();
    config_builder = config_builder.writer(write_manager);

    // Setup the TLS-Configuration
    let tls_config = tls::ConfigManager::new();
    config_builder = config_builder.tls(tls_config.clone());

    // Setting up the Dashboard-Configurators
    let mut dashboard_configurators = DashboardEntityList::new();

    // Setup the Configurators
    config_builder =
        setup_configurators(&rt, &config, config_builder, &mut dashboard_configurators);

    // Setup the Plugin-Acceptors, in case there are any
    let mut plugin_acceptors = Vec::new();
    if let Some(path) = config.plugin_file.clone() {
        log::info!("Enabling Plugins");

        let plugin_manager = plugins::Loader::new(path);
        plugin_acceptors = plugin_manager.load_acceptors();

        config_builder = config_builder.plugin_loader(plugin_manager);
    }

    // Actually construct the Config-Manager
    let mut config_manager = config_builder.build();

    // Create the "Manager" for all the Internal-Services
    let mut internals = Internals::new();
    internals.configure(
        &config,
        &mut config_manager,
        read_manager.clone(),
        dashboard_configurators,
        &plugin_acceptors,
    );

    // Setup auto-tls, if enabled
    rt.block_on(setup_auto_tls(
        &config,
        &mut internals,
        &mut config_manager,
        tls_config.clone(),
        metrics_registry.clone(),
    ));

    // Start the Config-Manager
    rt.spawn(config_manager.start());

    // Initialize the standard Forwarder and Handler for Requests
    let forwarder = BasicForwarder::new();
    let handler = BasicHandler::new(
        read_manager,
        forwarder,
        internals,
        Some(metrics_registry.clone()),
    );

    // Setup all the Acceptors
    let acceptor_futures =
        setup_acceptors(&rt, &config, handler.clone(), tls_config, metrics_registry);
    for plugin_acceptor in plugin_acceptors.into_iter() {
        rt.block_on(plugin_acceptor.start(handler.clone()));
    }

    // Actually run all the Acceptors
    rt.block_on(futures::future::join_all(acceptor_futures));
}

fn setup_runtime() -> tokio::runtime::Runtime {
    // Creating the Tokio-Runtime
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

    rt
}

fn setup_telemetry(rt: &tokio::runtime::Runtime, config: &cli::Options) -> Registry {
    // Setting up the Env-Logger, still used in some cases
    env_logger::init();

    // Setting up the logging/tracing stuff
    let colored_tracing = env::var("RUST_LOG_COLOR").is_ok();
    let tracing_directive_str =
        env::var("RUST_LOG").unwrap_or_else(|_| "tunneload=info".to_owned());
    let tracing_sub = tracing_subscriber::FmtSubscriber::builder()
        .json()
        .with_level(true)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing_directive_str.parse().unwrap()),
        )
        .with_ansi(colored_tracing)
        .finish();
    tracing::subscriber::set_global_default(tracing_sub)
        .expect("Setting initial Tracing-Subscriber");

    let metrics_registry = Registry::new_custom(Some("tunneload".to_owned()), None).unwrap();
    // Check if the Metrics-Endpoint is enabled and act accordingly
    if let Some(port) = config.metrics {
        log::info!("Starting Metrics-Endpoint...");

        let endpoint = metrics::Endpoint::new(metrics_registry.clone());
        rt.spawn(endpoint.start(port));
    }

    metrics_registry
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

    config_builder = configurator::files::setup(config, config_builder, dashboard_configurators);

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
    metrics_registry: prometheus::Registry,
) {
    if config.auto_tls.auto_tls_enabled {
        log::info!("Enabled Auto-TLS");

        tls::auto::register_metrics(&metrics_registry);

        let (rule_list, service_list, _, _) = config_manager.get_config_lists();

        let env = if config.auto_tls.auto_tls_production {
            tls::auto::Environment::Production
        } else {
            tls::auto::Environment::Staging
        };
        let contacts = Vec::new();

        let cluster_port = config.auto_tls.cluster.port;

        if let Some(service) = config.auto_tls.kubernetes_service.clone() {
            let discoverer =
                tls::auto::discovery::kubernetes::Discover::new_default(service, cluster_port)
                    .await;

            let (internal_acme, auto_session) = tls::auto::new(
                env,
                contacts,
                rule_list,
                service_list,
                tls_config,
                discoverer,
                cluster_port,
            )
            .await;

            config_manager.register_internal_service(&internal_acme);
            internals.add_service(Box::new(internal_acme));

            let kube_namespace = config.auto_tls.kubernetes_namespace.clone();

            let kube_store = tls::stores::kubernetes::KubeStore::new(kube_namespace).await;
            let storage = Arc::new(kube_store);
            let tx = auto_session.start(storage.clone());

            let three_week_seconds = 60 * 60 * 24 * 7 * 3;
            let expire_threshold = Duration::from_secs(three_week_seconds);
            tokio::task::spawn(tls::auto::renew(storage, tx.clone(), expire_threshold));

            config_manager.update_tls_queue(Some(tx));
            return;
        }

        if let Some(conf_path) = config.auto_tls.file.path.clone() {
            let discoverer = tls::auto::discovery::files::Discover::new(conf_path, cluster_port);

            let (internal_acme, auto_session) = tls::auto::new(
                env,
                contacts,
                rule_list,
                service_list,
                tls_config,
                discoverer,
                cluster_port,
            )
            .await;

            config_manager.register_internal_service(&internal_acme);
            internals.add_service(Box::new(internal_acme));

            let store_folder = config.auto_tls.file.directory.clone();
            let file_store = tls::stores::files::FileStore::new(store_folder);
            let storage = Arc::new(file_store);
            let tx = auto_session.start(storage.clone());

            let three_week_seconds = 60 * 60 * 24 * 7 * 3;
            let expire_threshold = Duration::from_secs(three_week_seconds);
            tokio::task::spawn(tls::auto::renew(storage, tx.clone(), expire_threshold));

            config_manager.update_tls_queue(Some(tx));
            return;
        }
    }
}
