use k8s_openapi::api::extensions::v1beta1::{HTTPIngressPath, Ingress, IngressRule};
use kube::api::Meta;

use log::error;

use crate::general::Shared;
use crate::rules::{Matcher, Rule, Service};

fn parse_path(
    http_path: &HTTPIngressPath,
    host: String,
    name: String,
    priority: u32,
) -> Option<Rule> {
    let backend = &http_path.backend;
    let service_name = backend.service_name.as_ref()?;
    let service_port = match backend.service_port.as_ref()? {
        k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(v) => v,
        _ => {
            error!("Could not get Service-Port");
            return None;
        }
    };
    let path = match http_path.path.as_ref() {
        Some(p) => p,
        None => return None,
    };

    let matcher = Matcher::And(vec![
        Matcher::Domain(host),
        Matcher::PathPrefix(path.to_string()),
    ]);

    let addresses = vec![format!("{}:{}", service_name, service_port)];
    Some(Rule::new(
        name,
        priority,
        matcher,
        Vec::new(),
        Shared::new(Service::new(service_name, addresses)),
    ))
}

fn parse_rule(rule: &IngressRule, name: &str, priority: u32) -> Option<Vec<Rule>> {
    let host = rule.host.as_ref()?;

    let http = rule.http.as_ref()?;

    let mut result = Vec::new();
    for http_path in http.paths.iter() {
        if let Some(tmp) = parse_path(http_path, host.clone(), name.to_string(), priority) {
            result.push(tmp);
        }
    }

    Some(result)
}

/// The Errors that could be returned when attempting to
/// parse an Ingress Ressource
#[derive(Debug)]
pub enum Error {
    /// The Ressource was missing a Spec
    MissingSpec,
    /// The Ressource was missing any Rules
    MissingRules,
}

/// Attempts to parse the the given Ingress Ressource as a Rule or
/// List of Rules
pub fn parse(p: Ingress, default_priority: u32) -> Result<Vec<Rule>, Error> {
    let route_name = Meta::name(&p);

    let spec = match p.spec {
        Some(s) => s,
        None => return Err(Error::MissingSpec),
    };

    let rules = match spec.rules {
        Some(r) => r,
        None => return Err(Error::MissingRules),
    };

    let mut result = Vec::new();

    for rule in rules.iter() {
        if let Some(tmp) = parse_rule(rule, &route_name, default_priority) {
            result.extend(tmp);
        }
    }

    Ok(result)
}
