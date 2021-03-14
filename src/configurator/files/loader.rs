use crate::configurator::{Configurator, MiddlewareList, RuleList};
use crate::rules::{Middleware, Rule};
use crate::{configurator::files, configurator::ServiceList, rules::Service};

use async_trait::async_trait;
use futures::Future;

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
        middlewares: &MiddlewareList,
        _services: &ServiceList,
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

    async fn load_tls(
        &mut self,
        _rules: &[std::sync::Arc<Rule>],
    ) -> Vec<(String, rustls::sign::CertifiedKey)> {
        Vec::new()
    }

    fn get_serivce_event_listener(
        &mut self,
        _services: ServiceList,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        // TODO
        // Actually listen to file-events
        async fn run() {}

        Box::pin(run())
    }

    fn get_middleware_event_listener(
        &mut self,
        _middlewares: MiddlewareList,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        // TODO
        // Actually listen to file-events
        async fn run() {}

        Box::pin(run())
    }

    fn get_rules_event_listener(
        &mut self,
        _middlewares: MiddlewareList,
        _services: ServiceList,
        _rules: RuleList,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        // TODO
        // Actually listen to file-events
        async fn run() {}

        Box::pin(run())
    }
}
