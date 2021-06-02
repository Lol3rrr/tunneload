use crate::rules::rule_list::RuleListWriteHandle;
use crate::tls::auto::CertificateQueue;
use crate::{configurator::Configurator, internal_services::traits::InternalService};
use crate::{plugins, tls};

use super::ActionPluginList;
use super::{manager_builder::ManagerBuilder, MiddlewareList, RuleList, ServiceList};

use prometheus::Registry;

/// Manages all the Configuration for the Load-Balancer
pub struct Manager {
    /// All the Configurators used by the Load-Balancer
    configurators: Vec<Box<dyn Configurator + Send>>,
    /// The TLS-Configuration for the Load-Balancer
    tls: tls::ConfigManager,
    /// The Loader responsible for the Plugins
    plugin_loader: Option<plugins::Loader>,
    /// All registered Action-Plugins
    action_plugins: ActionPluginList,
    /// All currently active Rules
    rules: RuleList,
    /// All currently active Services
    services: ServiceList,
    /// All the registered Middlewares
    middlewares: MiddlewareList,
    /// The Queue of Domains to generate
    auto_tls_queue: Option<CertificateQueue>,
}

impl Manager {
    pub(crate) fn new(
        configurators: Vec<Box<dyn Configurator + Send>>,
        tls: tls::ConfigManager,
        writer: RuleListWriteHandle,
        plugin_loader: Option<plugins::Loader>,
    ) -> Self {
        Self {
            configurators,
            tls,
            plugin_loader,
            action_plugins: ActionPluginList::new(),
            services: ServiceList::new(),
            middlewares: MiddlewareList::new(),
            rules: RuleList::new(writer),
            auto_tls_queue: None,
        }
    }

    /// # Returns
    /// A simple Builder to create a new Manager
    pub fn builder() -> ManagerBuilder {
        ManagerBuilder::new()
    }

    /// Replaces the previous queue with the new queue
    pub fn update_tls_queue(&mut self, n_queue: Option<CertificateQueue>) {
        self.auto_tls_queue = n_queue;
    }

    /// Used to register all the needed Metrics
    pub fn register_metrics(mut reg: Registry) {
        ServiceList::register_metrics(&mut reg);
        MiddlewareList::register_metrics(&mut reg);
        RuleList::register_metrics(&mut reg);
    }

    /// Returns cloned versions of all the internal
    /// Configuration-Lists
    pub fn get_config_lists(&self) -> (RuleList, ServiceList, MiddlewareList, ActionPluginList) {
        (
            self.rules.clone(),
            self.services.clone(),
            self.middlewares.clone(),
            self.action_plugins.clone(),
        )
    }

    /// This function is used to register all internal Services as they
    /// can not be "found"/"discovered" using the Configurators
    pub fn register_internal_service<I>(&mut self, service: &I)
    where
        I: InternalService,
    {
        let tmp_service = service.service();
        self.services.set_service(tmp_service);
    }

    async fn update_services(&mut self) {
        let mut all_services = Vec::new();
        for config in self.configurators.iter_mut() {
            let mut tmp = config.load_services().await;
            all_services.append(&mut tmp);
        }

        for tmp_srv in all_services.drain(..) {
            self.services.set_service(tmp_srv);
        }
    }

    async fn update_middlewares(&mut self) {
        let mut result = Vec::new();
        for config in self.configurators.iter_mut() {
            let mut tmp = config.load_middleware(&self.action_plugins).await;
            result.append(&mut tmp);
        }

        for tmp_mid in result.drain(..) {
            self.middlewares.set_middleware(tmp_mid);
        }
    }

    async fn update_rules(&mut self) {
        let mut result = Vec::new();

        for config in self.configurators.iter_mut() {
            let mut tmp = config
                .load_rules(
                    &self.middlewares,
                    &self.services,
                    self.auto_tls_queue.clone(),
                )
                .await;
            result.append(&mut tmp);
        }

        for tmp_rule in result.drain(..) {
            self.rules.set_rule(tmp_rule);
        }
    }

    async fn update_tls(&mut self) {
        let mut result = Vec::new();
        for config in self.configurators.iter_mut() {
            let mut tmp = config.load_tls().await;
            result.append(&mut tmp);
        }

        self.tls.set_certs(result);
    }

    fn update_plugins(&mut self) {
        if let Some(loader) = self.plugin_loader.as_ref() {
            for tmp in loader.load_action_plugins().drain(..) {
                self.action_plugins.set_plugin_action(tmp);
            }
        }
    }

    async fn initial_load(&mut self) {
        self.update_services().await;
        self.update_middlewares().await;
        self.update_rules().await;
        self.update_tls().await;
        self.update_plugins();
    }

    fn start_event_listeners(&mut self) {
        for tmp_conf in self.configurators.iter_mut() {
            tokio::task::spawn(tmp_conf.get_serivce_event_listener(self.services.clone()));
            tokio::task::spawn(tmp_conf.get_middleware_event_listener(
                self.middlewares.clone(),
                self.action_plugins.clone(),
            ));
            tokio::task::spawn(tmp_conf.get_rules_event_listener(
                self.middlewares.clone(),
                self.services.clone(),
                self.rules.clone(),
                self.auto_tls_queue.clone(),
            ));
            tokio::task::spawn(tmp_conf.get_tls_event_listener(self.tls.clone()));
        }
    }

    /// Starts the Manager itself and all the Tasks
    /// that belong to it
    pub async fn start(mut self) {
        // Load the initial Configuration
        self.initial_load().await;

        // Start all the needed listeners to update the
        // configuration
        self.start_event_listeners();
    }
}
