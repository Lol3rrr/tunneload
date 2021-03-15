use k8s_openapi::api::extensions::v1beta1::Ingress;
use kube::api::Meta;

use log::error;

use crate::general::Shared;
use crate::rules::{Matcher, Rule, Service};

pub fn parse(p: Ingress, default_priority: u32) -> Vec<Rule> {
    let mut result = Vec::new();

    let route_name = Meta::name(&p);

    let spec = match p.spec {
        Some(s) => s,
        None => {
            error!("Could not load Spec for Ingress: '{}'", route_name);
            return result;
        }
    };

    let rules = match spec.rules {
        Some(r) => r,
        None => {
            error!("Could not load Rules for Ingress: '{}'", route_name);
            return result;
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

            let matcher = Matcher::And(vec![
                Matcher::Domain(host.clone()),
                Matcher::PathPrefix(path),
            ]);

            let addresses = vec![format!("{}:{}", service_name, service_port)];
            let n_rule = Rule::new(
                route_name.clone(),
                default_priority,
                matcher,
                Vec::new(),
                Shared::new(Service::new(service_name, addresses)),
            );
            result.push(n_rule);
        }
    }

    result
}
