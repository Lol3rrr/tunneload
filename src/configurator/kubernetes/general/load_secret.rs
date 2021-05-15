use k8s_openapi::{api::core::v1::Secret, ByteString};
use kube::Api;

/// Attempts to load the Secret from the connected Kubernetes
/// Cluster
pub async fn load_secret(
    client: kube::Client,
    namespace: &str,
    secret_name: &str,
) -> Option<std::collections::BTreeMap<String, ByteString>> {
    let secrets: Api<Secret> = Api::namespaced(client, namespace);

    let secret = match secrets.get(secret_name).await {
        Ok(r) => r,
        Err(e) => {
            log::error!("Loading Secret: {}", e);
            return None;
        }
    };

    secret.data
}
