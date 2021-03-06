use crate::{
    general::Shared,
    rules::{Middleware, Rule, Service},
};

use async_trait::async_trait;
use futures::Future;

use super::ServiceList;

#[async_trait]
pub trait Configurator {
    async fn load_services(&mut self) -> Vec<Service>;
    async fn load_middleware(&mut self) -> Vec<Middleware>;
    async fn load_rules(&mut self, middlewares: &[Middleware], services: &ServiceList)
        -> Vec<Rule>;
    async fn load_tls(&mut self, rules: &[Rule]) -> Vec<(String, rustls::sign::CertifiedKey)>;

    fn get_serivce_event_listener(
        &mut self,
        services: ServiceList,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
}
