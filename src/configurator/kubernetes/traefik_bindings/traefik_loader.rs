use async_trait::async_trait;
use kube::{api::ListParams, Api};

use crate::configurator::parser::{Loader, RawMiddlewareConfig, RawRuleConfig};

use crate::configurator::kubernetes::traefik_bindings;

/// This is the Loader for the Kubernetes-Traefik-Configuration
pub struct TraefikLoader {
    client: kube::Client,
    namespace: String,
}

impl TraefikLoader {
    /// Creates a new Instance of the Loader from the given initial
    /// Values
    pub fn new(client: kube::Client, namespace: String) -> Self {
        Self { client, namespace }
    }
}

#[async_trait]
impl Loader for TraefikLoader {
    async fn middlewares(&self) -> Vec<RawMiddlewareConfig> {
        let mut result = Vec::new();

        let middlewares: Api<traefik_bindings::middleware::Middleware> =
            Api::namespaced(self.client.clone(), &self.namespace);
        let lp = ListParams::default();
        for p in middlewares.list(&lp).await.unwrap() {
            let metadata = &p.metadata;
            let name = metadata.name.as_ref().unwrap().to_owned();

            let current_config = serde_json::to_value(p).unwrap();
            let spec = current_config.as_object().expect("");

            for (key, value) in spec.iter() {
                result.push(RawMiddlewareConfig {
                    name: name.clone(),
                    action_name: key.clone(),
                    config: value.clone(),
                });
            }
        }

        result
    }

    async fn rules(&self) -> Vec<RawRuleConfig> {
        let mut result = Vec::new();

        let ingressroutes: Api<traefik_bindings::ingressroute::IngressRoute> =
            Api::namespaced(self.client.clone(), &self.namespace);
        let lp = ListParams::default();

        let route_list = match ingressroutes.list(&lp).await {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Listing Ingress-Routes: {:?}", e);
                return Vec::new();
            }
        };

        for route in route_list {
            let spec_value = serde_json::to_value(route).unwrap();
            result.push(RawRuleConfig { config: spec_value });
        }

        result
    }
}
