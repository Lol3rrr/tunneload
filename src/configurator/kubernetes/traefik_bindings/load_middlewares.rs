use crate::configurator::kubernetes::traefik_bindings::{self, parse::parse_middleware};
use crate::rules::Middleware;

use kube::api::{Api, ListParams};

/// Loads all the Middlewares specified by Traefik-Bindings
pub async fn load_middlewares(client: kube::Client, namespace: &str) -> Vec<Middleware> {
    let mut result = Vec::new();

    let middlewares: Api<traefik_bindings::middleware::Middleware> =
        Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default();
    for p in middlewares.list(&lp).await.unwrap() {
        let metadata = p.metadata;
        if let Some(raw_annotations) = metadata.annotations {
            let last_applied = raw_annotations
                .get("kubectl.kubernetes.io/last-applied-configuration")
                .unwrap();

            let current_config: traefik_bindings::middleware::Config =
                serde_json::from_str(last_applied).unwrap();

            result.extend(
                parse_middleware(Some(client.clone()), Some(namespace), current_config).await,
            );
        }
    }

    result
}
