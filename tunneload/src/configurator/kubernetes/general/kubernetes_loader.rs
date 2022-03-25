use async_trait::async_trait;
use k8s_openapi::api::core::v1::{Endpoints, Secret};
use kube::{api::ListParams, Api};

use crate::configurator::parser::{Loader, RawServiceConfig, RawTLSConfig};

/// The Loader for the general Kubernetes-Configuration
pub struct KubernetesLoader {
    client: kube::Client,
    namespace: String,
}

impl KubernetesLoader {
    /// Creates a new Instance of the Loader from the given initial
    /// Values
    pub fn new(client: kube::Client, namespace: String) -> Self {
        Self { client, namespace }
    }

    async fn load_services(&self) -> Vec<RawServiceConfig> {
        let mut result = Vec::new();

        let endpoints: Api<Endpoints> = Api::namespaced(self.client.clone(), &self.namespace);
        let lp = ListParams::default();
        let endpoint_list = match endpoints.list(&lp).await {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };
        for endpoint in endpoint_list
            .into_iter()
            .filter_map(|e| serde_json::to_value(e).ok())
        {
            result.push(RawServiceConfig { config: endpoint });
        }

        result
    }
}

#[async_trait]
impl Loader for KubernetesLoader {
    async fn services(&self) -> Vec<RawServiceConfig> {
        self.load_services().await
    }

    async fn tls(&self) -> Vec<RawTLSConfig> {
        let mut result = Vec::new();

        let secrets: Api<Secret> = Api::namespaced(self.client.clone(), &self.namespace);
        let lp = ListParams::default();
        let secrets_list = match secrets.list(&lp).await {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        for secret in secrets_list {
            if secret.type_ != Some("kubernetes.io/tls".to_owned()) {
                continue;
            }

            if let Ok(conf) = serde_json::to_value(secret) {
                result.push(RawTLSConfig { config: conf });
            }
        }

        result
    }
}
