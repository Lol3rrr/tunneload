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
            let metadata = p.metadata;
            if let Some(raw_annotations) = metadata.annotations {
                let last_applied = raw_annotations
                    .get("kubectl.kubernetes.io/last-applied-configuration")
                    .unwrap();

                let current_config: traefik_bindings::middleware::Config =
                    serde_json::from_str(last_applied).unwrap();

                let name = current_config.metadata.name;
                for (key, value) in current_config.spec.iter() {
                    result.push(RawMiddlewareConfig {
                        name: name.clone(),
                        action_name: key.clone(),
                        config: value.clone(),
                    });
                }
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
                log::error!("Listing Ingress-Routes: {:?}", e);
                return Vec::new();
            }
        };

        for route in route_list {
            let metadata = route.metadata;
            if let Some(raw_annotations) = metadata.annotations {
                let last_applied = raw_annotations
                    .get("kubectl.kubernetes.io/last-applied-configuration")
                    .unwrap();

                let current_config: serde_json::Value = serde_json::from_str(last_applied).unwrap();

                result.push(RawRuleConfig {
                    config: current_config,
                });
            }
        }

        result
    }
}
