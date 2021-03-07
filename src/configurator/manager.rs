use crate::configurator::Configurator;
use crate::rules::{Middleware, Rule, WriteManager};
use crate::tls;

use super::{manager_builder::ManagerBuilder, ServiceList};

use lazy_static::lazy_static;
use prometheus::Registry;

lazy_static! {
    static ref CONFIG_MIDDLEWARE_COUNT: prometheus::IntGauge = prometheus::IntGauge::new(
        "config_middlewares",
        "The Number of middlewares currently registered",
    )
    .unwrap();
    static ref CONFIG_RULES_COUNT: prometheus::IntGauge =
        prometheus::IntGauge::new("config_rules", "The Number of rules currently registered",)
            .unwrap();
}

pub struct Manager {
    pub(crate) configurators: Vec<Box<dyn Configurator + Send>>,
    pub(crate) writer: WriteManager,
    pub(crate) tls: tls::ConfigManager,
    pub(crate) services: ServiceList,
    pub(crate) wait_time: std::time::Duration,
}

impl Manager {
    pub fn builder() -> ManagerBuilder {
        ManagerBuilder::new()
    }

    /// Used to register all the needed Metrics
    pub fn register_metrics(reg: Registry) {
        reg.register(Box::new(CONFIG_MIDDLEWARE_COUNT.clone()))
            .unwrap();
        reg.register(Box::new(CONFIG_RULES_COUNT.clone())).unwrap();

        ServiceList::register_metrics(reg);
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

    async fn load_middlewares(&mut self) -> Vec<Middleware> {
        let mut result = Vec::new();
        for config in self.configurators.iter_mut() {
            let mut tmp = config.load_middleware().await;
            result.append(&mut tmp);
        }

        result
    }

    async fn load_rules(&mut self, middlewares: &[Middleware]) -> Vec<Rule> {
        let mut result = Vec::new();

        for config in self.configurators.iter_mut() {
            let mut tmp = config.load_rules(middlewares, &self.services).await;
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
        let middlewares = self.load_middlewares().await;
        let n_rules = self.load_rules(&middlewares).await;
        let tls = self.load_tls(&n_rules).await;

        // Update the Metrics
        CONFIG_MIDDLEWARE_COUNT.set(middlewares.len() as i64);
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
    }

    fn start_event_listeners(&mut self) {
        for tmp_conf in self.configurators.iter_mut() {
            let service_event_listener = tmp_conf.get_serivce_event_listener(self.services.clone());
            tokio::task::spawn(service_event_listener);
        }
    }

    /// Starts the Manager itself and all the Tasks
    /// that belong to it
    pub async fn start(mut self) {
        // Load the
        self.initial_load().await;

        self.start_event_listeners();

        let wait_time = self.wait_time;
        self.update_loop(wait_time).await;
    }
}
