use crate::configurator::Configurator;
use crate::rules::{Middleware, Rule};
use crate::{configurator::kubernetes::general::load_tls, rules::Service};
use crate::{
    configurator::kubernetes::{ingress, traefik_bindings},
    general::Shared,
};

use crate::configurator::kubernetes::general::parse_endpoint;
use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Event;
use kube::{
    api::{ListParams, Meta},
    Api, Client,
};
use kube_runtime::{
    utils::{try_flatten_applied, try_flatten_touched},
    watcher,
};

use super::general::load_services;

#[derive(Clone)]
pub struct Loader {
    client: Client,
    namespace: String,
    use_traefik: bool,
    use_ingress: bool,
    ingress_priority: u32,
}

impl Loader {
    pub async fn new(namespace: String) -> Self {
        let client = Client::try_default().await.unwrap();

        Self {
            client,
            namespace,
            use_traefik: false,
            use_ingress: false,
            ingress_priority: 100,
        }
    }
    pub fn enable_traefik(&mut self) {
        self.use_traefik = true;
    }
    pub fn enable_ingress(&mut self) {
        self.use_ingress = true;
    }
    pub fn set_ingress_priority(&mut self, n_priority: u32) {
        self.ingress_priority = n_priority;
    }

    pub async fn service_events(self) {
        //let events: Api<Event> = Api::namespaced(self.client.clone(), &self.namespace);
        let endpoints: Api<k8s_openapi::api::core::v1::Endpoints> =
            Api::namespaced(self.client.clone(), &self.namespace);

        let lp = ListParams::default();

        let mut touched_stream = try_flatten_applied(watcher(endpoints, lp)).boxed();
        while let Some(srv) = touched_stream.try_next().await.unwrap() {
            let service = match parse_endpoint(&srv) {
                Some(s) => s,
                None => continue,
            };

            println!("Service: {:?}", service);
        }
    }
}

#[async_trait]
impl Configurator for Loader {
    async fn load_services(&mut self) -> Vec<Service> {
        load_services(self.client.clone(), &self.namespace).await
    }

    async fn load_middleware(&mut self) -> Vec<Middleware> {
        let mut result = Vec::new();

        if self.use_traefik {
            let mut traefik =
                traefik_bindings::load_middlewares(self.client.clone(), &self.namespace).await;
            result.append(&mut traefik);
        }

        result
    }

    async fn load_rules(
        &mut self,
        middlewares: &[Middleware],
        services: &[Shared<Service>],
    ) -> Vec<Rule> {
        let mut result = Vec::new();

        if self.use_traefik {
            let mut traefik = traefik_bindings::load_routes(
                self.client.clone(),
                &self.namespace,
                middlewares,
                services,
            )
            .await;
            result.append(&mut traefik);
        }

        if self.use_ingress {
            let mut ingress_routes =
                ingress::load_routes(self.client.clone(), &self.namespace, self.ingress_priority)
                    .await;
            result.append(&mut ingress_routes);
        }

        result
    }

    async fn load_tls(&mut self, rules: &[Rule]) -> Vec<(String, rustls::sign::CertifiedKey)> {
        load_tls(self.client.clone(), &self.namespace, rules).await
    }
}
