use crate::configurator::kubernetes::ingress::parse::parse_rule;
use crate::rules::Rule;

use k8s_openapi::api::extensions::v1beta1::Ingress;
use kube::api::{Api, ListParams};

/// Loads all the ingress-routes in the cluster
pub async fn load_routes(
    client: kube::Client,
    namespace: &str,
    default_priority: u32,
) -> Vec<Rule> {
    let mut result = Vec::new();

    let ingress: Api<Ingress> = Api::namespaced(client, namespace);
    let lp = ListParams::default();
    for p in ingress.list(&lp).await.unwrap() {
        if let Ok(parsed) = parse_rule(p, default_priority) {
            result.extend(parsed);
        }
    }

    result
}
