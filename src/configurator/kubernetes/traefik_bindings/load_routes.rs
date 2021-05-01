use crate::rules::Rule;
use crate::{
    configurator::kubernetes::traefik_bindings::{self, parse::parse_rule},
    configurator::{MiddlewareList, ServiceList},
};

use kube::api::{Api, ListParams};
use log::error;

/// Loads all the raw routes in the given Namespace
pub async fn load_routes(
    client: kube::Client,
    namespace: &str,
    middlewares: &MiddlewareList,
    services: &ServiceList,
) -> Vec<Rule> {
    let mut result = Vec::new();

    let ingressroutes: Api<traefik_bindings::ingressroute::IngressRoute> =
        Api::namespaced(client, namespace);
    let lp = ListParams::default();
    for route in ingressroutes.list(&lp).await.unwrap() {
        let metadata = route.metadata;
        if let Some(raw_annotations) = metadata.annotations {
            let last_applied = raw_annotations
                .get("kubectl.kubernetes.io/last-applied-configuration")
                .unwrap();

            let current_config: traefik_bindings::ingressroute::Config =
                serde_json::from_str(last_applied).unwrap();

            match parse_rule(current_config, middlewares, services) {
                Ok(r) => {
                    result.push(r);
                }
                Err(e) => {
                    error!("Unknown Rule('{:?}'): {:?}", last_applied, e);
                }
            };
        }
    }

    result
}
