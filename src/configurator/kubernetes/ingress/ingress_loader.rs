use async_trait::async_trait;
use k8s_openapi::api::extensions::v1beta1::Ingress;
use kube::{api::ListParams, Api};

use crate::configurator::parser::{Loader, RawRuleConfig};

/// The Loader for the Kubernetes-Ingress-Configuration
pub struct IngressLoader {
    client: kube::Client,
    namespace: String,
}

impl IngressLoader {
    /// Creates a new Instance of the Loader from the given initial
    /// Values
    pub fn new(client: kube::Client, namespace: String) -> Self {
        Self { client, namespace }
    }
}

#[async_trait]
impl Loader for IngressLoader {
    async fn rules(&self) -> Vec<RawRuleConfig> {
        let mut result = Vec::new();

        let ingress: Api<Ingress> = Api::namespaced(self.client.clone(), &self.namespace);
        let lp = ListParams::default();
        for p in ingress.list(&lp).await.unwrap() {
            result.push(RawRuleConfig {
                config: serde_json::to_value(p).unwrap(),
            });
        }

        result
    }
}
