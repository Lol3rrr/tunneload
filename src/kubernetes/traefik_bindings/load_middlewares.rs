use crate::kubernetes::traefik_bindings::{self, parse::parse_middleware};
use crate::rules::Middleware;

use kube::api::{Api, ListParams, Meta};

/// Loads all the Middlewares specified by Traefik-Bindings
pub async fn load_middlewares(client: kube::Client, namespace: &str) -> Vec<Middleware> {
    let mut result = Vec::new();

    let ingressroutes: Api<traefik_bindings::middleware::Middleware> =
        Api::namespaced(client, namespace);
    let lp = ListParams::default();
    for p in ingressroutes.list(&lp).await.unwrap() {
        let route_name = Meta::name(&p);

        let route = ingressroutes.get(&route_name).await.unwrap();
        let metadata = route.metadata;
        if let Some(raw_annotations) = metadata.annotations {
            let last_applied = raw_annotations
                .get("kubectl.kubernetes.io/last-applied-configuration")
                .unwrap();

            let current_config: traefik_bindings::middleware::Config =
                serde_json::from_str(last_applied).unwrap();

            result.extend(parse_middleware(current_config));
        }
    }

    result
}
