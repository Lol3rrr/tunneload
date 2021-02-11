use crate::kubernetes;
use crate::rules::WriteManager;
use crate::rules::{Middleware, Rule};

mod middleware;
use middleware::parse_middleware;

mod rule;
use rule::parse_rule;

use kube::api::{Api, ListParams, Meta};
use kube::Client;

use log::{debug, error};

pub struct Manager {
    client: Client,
}

impl Manager {
    pub async fn new() -> Self {
        // Read the environment to find config for kube client.
        // Note that this tries an in-cluster configuration first,
        // then falls back on a kubeconfig file.
        let client = Client::try_default().await.unwrap();

        Self { client }
    }

    /// Loads all the declared Middlewares in the cluster
    async fn load_middlewares(&self, namespace: &str) -> Vec<Middleware> {
        let mut result = Vec::new();

        let ingressroutes: Api<kubernetes::middleware::Middleware> =
            Api::namespaced(self.client.clone(), namespace);
        let lp = ListParams::default();
        for p in ingressroutes.list(&lp).await.unwrap() {
            let route_name = Meta::name(&p);

            let route = ingressroutes.get(&route_name).await.unwrap();
            let metadata = route.metadata;
            if let Some(raw_annotations) = metadata.annotations {
                let last_applied = raw_annotations
                    .get("kubectl.kubernetes.io/last-applied-configuration")
                    .unwrap();

                let current_config: kubernetes::middleware::Config =
                    serde_json::from_str(last_applied).unwrap();

                result.extend(parse_middleware(current_config));
            }
        }

        result
    }

    /// Loads all the raw routes in the cluster
    async fn load_routes(&self, namespace: &str, middlewares: &[Middleware]) -> Vec<Rule> {
        let mut result = Vec::new();

        let ingressroutes: Api<kubernetes::ingressroute::IngressRoute> =
            Api::namespaced(self.client.clone(), namespace);
        let lp = ListParams::default();
        for p in ingressroutes.list(&lp).await.unwrap() {
            let route_name = Meta::name(&p);

            let route = ingressroutes.get(&route_name).await.unwrap();
            let metadata = route.metadata;
            if let Some(raw_annotations) = metadata.annotations {
                let last_applied = raw_annotations
                    .get("kubectl.kubernetes.io/last-applied-configuration")
                    .unwrap();

                let current_config: kubernetes::ingressroute::Config =
                    serde_json::from_str(last_applied).unwrap();

                match parse_rule(current_config, middlewares) {
                    Some(r) => {
                        result.push(r);
                    }
                    None => {
                        error!("Unknown Rule: '{:?}'", last_applied);
                    }
                };
            }
        }

        result
    }

    async fn get_rules(&self, namespace: &str) -> Vec<Rule> {
        let middlewares = self.load_middlewares(namespace).await;
        self.load_routes(namespace, &middlewares).await
    }

    async fn update_rules(&self, writer: &WriteManager) {
        debug!("Updating Rules");
        let rules = self.get_rules("default").await;
        writer.add_rules(rules);

        debug!("Updated Rules");
    }

    pub async fn update_loop(self, writer: WriteManager, wait_time: std::time::Duration) {
        loop {
            self.update_rules(&writer).await;

            tokio::time::sleep(wait_time).await;
        }
    }
}
