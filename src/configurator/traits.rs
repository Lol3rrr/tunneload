use crate::rules::{Middleware, Rule};

use async_trait::async_trait;

#[async_trait]
pub trait Configurator {
    async fn load_middleware(&mut self) -> Vec<Middleware>;
    async fn load_rules(&mut self, middlewares: &[Middleware]) -> Vec<Rule>;
    async fn load_tls(&mut self, rules: &[Rule]) -> Vec<(String, rustls::sign::CertifiedKey)>;
}
