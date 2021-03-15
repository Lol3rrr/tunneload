use k8s_openapi::api::core::v1::Secret;
use kube::api::{Api, ListParams};

use super::parse_tls;

pub async fn load_tls(
    client: kube::Client,
    namespace: &str,
) -> Vec<(String, rustls::sign::CertifiedKey)> {
    let mut result = Vec::new();

    let secrets: Api<Secret> = Api::namespaced(client, namespace);
    let lp = ListParams::default();
    for secret in secrets.list(&lp).await.unwrap() {
        if let Some(tls) = parse_tls(secret) {
            log::info!("Loaded TLS for '{:?}'", tls.0);
            result.push(tls);
        }
    }

    result
}
