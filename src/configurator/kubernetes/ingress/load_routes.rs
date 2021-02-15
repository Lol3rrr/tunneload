use crate::rules::{Matcher, Rule, Service};

use k8s_openapi::api::extensions::v1beta1::Ingress;
use kube::api::{Api, ListParams, Meta};
use log::error;

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
        let route_name = Meta::name(&p);

        let spec = match p.spec {
            Some(s) => s,
            None => {
                error!("Could not load Spec for Ingress: '{}'", route_name);
                continue;
            }
        };

        let rules = match spec.rules {
            Some(r) => r,
            None => {
                error!("Could not load Rules for Ingress: '{}'", route_name);
                continue;
            }
        };

        for rule in rules {
            let host = rule.host.unwrap();

            let http = rule.http.unwrap();
            for http_path in http.paths {
                let backend = http_path.backend;
                let service_name = backend.service_name.unwrap();
                let service_port = match backend.service_port.unwrap() {
                    k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(v) => v,
                    _ => {
                        error!("Could not get Service-Port");
                        continue;
                    }
                };
                let path = http_path.path.unwrap();

                let mut matcher = Vec::new();
                matcher.push(Matcher::Domain(host.clone()));
                matcher.push(Matcher::PathPrefix(path));
                let n_rule = Rule::new(
                    route_name.clone(),
                    default_priority,
                    matcher,
                    Vec::new(),
                    Service::new(format!("{}:{}", service_name, service_port)),
                );
                result.push(n_rule);
            }
        }
    }

    result
}
