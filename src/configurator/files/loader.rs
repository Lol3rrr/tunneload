use crate::rules::{Middleware, Rule};
use crate::{configurator::files, rules::Service};
use crate::{configurator::Configurator, general::Shared};

use async_trait::async_trait;

use std::fs;

pub struct Loader {
    path: String,
}

impl Loader {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

#[async_trait]
impl Configurator for Loader {
    // TODO
    async fn load_services(&mut self) -> Vec<Service> {
        Vec::new()
    }

    async fn load_middleware(&mut self) -> Vec<Middleware> {
        let metadata = fs::metadata(&self.path).unwrap();
        if metadata.is_file() {
            files::load_middlewares(&self.path)
        } else {
            let mut tmp = Vec::new();
            for entry in fs::read_dir(&self.path).unwrap() {
                let mut result = files::load_middlewares(entry.unwrap().path());
                tmp.append(&mut result);
            }
            tmp
        }
    }

    async fn load_rules(
        &mut self,
        middlewares: &[Middleware],
        _services: &[Shared<Service>],
    ) -> Vec<Rule> {
        let metadata = fs::metadata(&self.path).unwrap();
        if metadata.is_file() {
            files::load_routes(&self.path, middlewares)
        } else {
            let mut tmp = Vec::new();
            for entry in fs::read_dir(&self.path).unwrap() {
                let mut result = files::load_routes(entry.unwrap().path(), middlewares);
                tmp.append(&mut result);
            }
            tmp
        }
    }

    async fn load_tls(&mut self, _rules: &[Rule]) -> Vec<(String, rustls::sign::CertifiedKey)> {
        Vec::new()
    }
}
