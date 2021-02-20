use crate::configurator::kubernetes::traefik_bindings::{self, parse::parse_rule};
use crate::rules::{Middleware, Rule};

use kube::api::{Api, ListParams, Meta};
use log::error;

/// Loads all the raw routes in the cluster
pub async fn load_routes(
    client: kube::Client,
    namespace: &str,
    middlewares: &[Middleware],
    services: std::collections::BTreeMap<String, Vec<String>>,
) -> Vec<Rule> {
    let mut result = Vec::new();

    let ingressroutes: Api<traefik_bindings::ingressroute::IngressRoute> =
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

            let current_config: traefik_bindings::ingressroute::Config =
                serde_json::from_str(last_applied).unwrap();

            match parse_rule(current_config, middlewares, &services) {
                Some(r) => {
                    result.push(r);
                }
                None => {
                    error!("Unknown Rule: '{:?}'", last_applied);
                }
            };
        }
    }

    result
}
