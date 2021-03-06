use crate::rules::{Middleware, Rule, WriteManager};
use crate::tls;
use crate::{configurator::Configurator, general::Shared, rules::Service};

use super::manager_builder::ManagerBuilder;

pub struct Manager {
    pub(crate) configurators: Vec<Box<dyn Configurator + Send>>,
    pub(crate) writer: WriteManager,
    pub(crate) tls: tls::ConfigManager,
    pub(crate) services: Vec<Shared<Service>>,
}

impl Manager {
    pub fn builder() -> ManagerBuilder {
        ManagerBuilder::new()
    }

    fn add_service(&mut self, n_srv: Service) {
        for tmp in self.services.iter() {
            let inner = tmp.get();
            if inner.name() == n_srv.name() {
                tmp.update(n_srv);
                return;
            }
        }

        self.services.push(Shared::new(n_srv));
    }
    async fn update_services(&mut self) {
        let mut all_services = Vec::new();
        for config in self.configurators.iter_mut() {
            let mut tmp = config.load_services().await;
            all_services.append(&mut tmp);
        }

        for tmp_srv in all_services.drain(..) {
            self.add_service(tmp_srv);
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

    pub async fn update(&mut self) {
        self.update_services().await;

        let middlewares = self.load_middlewares().await;
        let n_rules = self.load_rules(&middlewares).await;
        let tls = self.load_tls(&n_rules).await;

        self.writer.add_rules(n_rules);
        self.writer.publish();

        self.tls.set_certs(tls);
    }
    pub async fn update_loop(mut self, wait_time: std::time::Duration) {
        loop {
            self.update().await;

            tokio::time::sleep(wait_time).await;
        }
    }
}
