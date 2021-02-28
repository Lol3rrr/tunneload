use crate::configurator::Configurator;
use crate::rules::{Middleware, Rule, WriteManager};
use crate::tls;

pub struct ManagerBuilder {
    configurators: Vec<Box<dyn Configurator + Send>>,
    tls_config: Option<tls::ConfigManager>,
    writer: Option<WriteManager>,
}

impl ManagerBuilder {
    fn new() -> Self {
        Self {
            configurators: Vec::new(),
            tls_config: None,
            writer: None,
        }
    }

    pub fn writer(self, writer: WriteManager) -> Self {
        Self {
            configurators: self.configurators,
            tls_config: self.tls_config,
            writer: Some(writer),
        }
    }
    pub fn configurator<C: Configurator + Send + 'static>(self, conf: C) -> Self {
        let mut tmp_confs = self.configurators;
        tmp_confs.push(Box::new(conf));

        Self {
            configurators: tmp_confs,
            tls_config: self.tls_config,
            writer: self.writer,
        }
    }
    pub fn tls(self, config: tls::ConfigManager) -> Self {
        Self {
            configurators: self.configurators,
            tls_config: Some(config),
            writer: self.writer,
        }
    }

    pub fn build(self) -> Manager {
        Manager {
            configurators: self.configurators,
            writer: self.writer.unwrap(),
            tls: self.tls_config.unwrap(),
        }
    }
}

pub struct Manager {
    configurators: Vec<Box<dyn Configurator + Send>>,
    writer: WriteManager,
    tls: tls::ConfigManager,
}

impl Manager {
    pub fn builder() -> ManagerBuilder {
        ManagerBuilder::new()
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
            let mut tmp = config.load_rules(middlewares).await;
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
