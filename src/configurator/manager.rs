use crate::configurator::Configurator;
use crate::rules::{Rule, WriteManager};
use crate::tls;

use super::{manager_builder::ManagerBuilder, MiddlewareList, ServiceList};

use lazy_static::lazy_static;
use prometheus::Registry;

lazy_static! {
    static ref CONFIG_RULES_COUNT: prometheus::IntGauge =
        prometheus::IntGauge::new("config_rules", "The Number of rules currently registered",)
            .unwrap();
}

pub struct Manager {
    pub(crate) configurators: Vec<Box<dyn Configurator + Send>>,
    pub(crate) writer: WriteManager,
    pub(crate) tls: tls::ConfigManager,
    pub(crate) services: ServiceList,
    pub(crate) middlewares: MiddlewareList,
    pub(crate) wait_time: std::time::Duration,
}

impl Manager {
    pub fn builder() -> ManagerBuilder {
        ManagerBuilder::new()
    }

    /// Used to register all the needed Metrics
    pub fn register_metrics(mut reg: Registry) {
        reg.register(Box::new(CONFIG_RULES_COUNT.clone())).unwrap();

        ServiceList::register_metrics(&mut reg);
        MiddlewareList::register_metrics(&mut reg);
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
            let mut tmp = config.load_middleware().await;
            result.append(&mut tmp);
        }

        for tmp_mid in result.drain(..) {
            self.middlewares.set_middleware(tmp_mid);
        }
    }

    async fn load_rules(&mut self) -> Vec<Rule> {
        let mut result = Vec::new();

        for config in self.configurators.iter_mut() {
            let mut tmp = config.load_rules(&self.middlewares, &self.services).await;
            result.append(&mut tmp);
        }

        result
    }

    async fn load_tls(&mut self, rules: &[Rule]) -> Vec<(String, rustls::sign::CertifiedKey)> {
        let mut result = Vec::new();
        for config in self.configurators.iter_mut() {
            let mut tmp = config.load_tls(rules).await;
            result.append(&mut tmp);
        }

        result
    }

    async fn update(&mut self) {
        let n_rules = self.load_rules().await;
        let tls = self.load_tls(&n_rules).await;

        // Update the Metrics
        CONFIG_RULES_COUNT.set(n_rules.len() as i64);

        self.writer.add_rules(n_rules);
        self.writer.publish();

        self.tls.set_certs(tls);
    }
    async fn update_loop(mut self, wait_time: std::time::Duration) {
        loop {
            self.update().await;

            tokio::time::sleep(wait_time).await;
        }
    }

    async fn initial_load(&mut self) {
        self.update_services().await;
        self.update_middlewares().await;
    }

    fn start_event_listeners(&mut self) {
        for tmp_conf in self.configurators.iter_mut() {
            tokio::task::spawn(tmp_conf.get_serivce_event_listener(self.services.clone()));
            tokio::task::spawn(tmp_conf.get_middleware_event_listener(self.middlewares.clone()));
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

        let wait_time = self.wait_time;
        self.update_loop(wait_time).await;
    }
}
