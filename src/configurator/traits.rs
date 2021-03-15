use crate::rules::{Middleware, Rule, Service};
use crate::tls;

use async_trait::async_trait;
use futures::Future;

use super::{MiddlewareList, RuleList, ServiceList};

#[async_trait]
pub trait Configurator {
    async fn load_services(&mut self) -> Vec<Service>;
    async fn load_middleware(&mut self) -> Vec<Middleware>;
    async fn load_rules(
        &mut self,
        middlewares: &MiddlewareList,
        services: &ServiceList,
    ) -> Vec<Rule>;
    async fn load_tls(&mut self) -> Vec<(String, rustls::sign::CertifiedKey)>;

    fn get_serivce_event_listener(
        &mut self,
        services: ServiceList,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

    fn get_middleware_event_listener(
        &mut self,
        middlewares: MiddlewareList,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

    fn get_rules_event_listener(
        &mut self,
        middlewares: MiddlewareList,
        services: ServiceList,
        rules: RuleList,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

    fn get_tls_event_listener(
        &mut self,
        tls_manager: tls::ConfigManager,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
}
