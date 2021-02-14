use crate::configurator::kubernetes::traefik_bindings;
use crate::configurator::Configurator;
use crate::rules::WriteManager;
use crate::rules::{Middleware, Rule};

use async_trait::async_trait;
use kube::Client;

use log::debug;

pub struct Loader {
    client: Client,
    namespace: String,
}

impl Loader {
    pub async fn new(namespace: String) -> Self {
        let client = Client::try_default().await.unwrap();

        Self { client, namespace }
    }
}

#[async_trait]
impl Configurator for Loader {
    async fn load_middleware(&mut self) -> Vec<Middleware> {
        traefik_bindings::load_middlewares(self.client.clone(), &self.namespace).await
    }

    async fn load_rules(&mut self, middlewares: &[Middleware]) -> Vec<Rule> {
        traefik_bindings::load_routes(self.client.clone(), &self.namespace, middlewares).await
    }
}
