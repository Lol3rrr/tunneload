use crate::configurator::Configurator;
use crate::rules::{Middleware, Rule, WriteManager};

pub struct Manager {
    configurators: Vec<Box<dyn Configurator + Send>>,
    writer: WriteManager,
}

impl Manager {
    pub fn new(configs: Vec<Box<dyn Configurator + Send>>, writer: WriteManager) -> Self {
        Self {
            configurators: configs,
            writer,
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
            let mut tmp = config.load_rules(middlewares).await;
            result.append(&mut tmp);
        }

        result
    }

    pub async fn update(&mut self) {
        let middlewares = self.load_middlewares().await;
        let result = self.load_rules(&middlewares).await;

        self.writer.add_rules(result);
        self.writer.publish();
    }
    pub async fn update_loop(mut self, wait_time: std::time::Duration) {
        loop {
            self.update().await;

            tokio::time::sleep(wait_time).await;
        }
    }
}
