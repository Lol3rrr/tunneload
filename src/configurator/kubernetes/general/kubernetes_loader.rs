use async_trait::async_trait;
use k8s_openapi::api::core::v1::{Endpoints, Secret};
use kube::{api::ListParams, Api};

use crate::configurator::parser::{Loader, RawServiceConfig, RawTLSConfig};

pub struct KubernetesLoader {
    client: kube::Client,
    namespace: String,
}

impl KubernetesLoader {
    pub fn new(client: kube::Client, namespace: String) -> Self {
        Self { client, namespace }
    }
}

#[async_trait]
impl Loader for KubernetesLoader {
    async fn services(&self) -> Vec<RawServiceConfig> {
        let mut result = Vec::new();

        let endpoints: Api<Endpoints> = Api::namespaced(self.client.clone(), &self.namespace);
        let lp = ListParams::default();
        for endpoint in endpoints.list(&lp).await.unwrap() {
            result.push(RawServiceConfig {
                config: serde_json::to_value(endpoint).unwrap(),
            });
        }

        result
    }

    async fn tls(&self) -> Vec<RawTLSConfig> {
        let mut result = Vec::new();

        let secrets: Api<Secret> = Api::namespaced(self.client.clone(), &self.namespace);
        let lp = ListParams::default();
        for secret in secrets.list(&lp).await.unwrap() {
            if secret.type_ != Some("kubernetes.io/tls".to_owned()) {
                continue;
            }

            result.push(RawTLSConfig {
                config: serde_json::to_value(secret).unwrap(),
            });
        }

        result
    }
}
