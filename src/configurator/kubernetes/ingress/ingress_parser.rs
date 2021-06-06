use async_trait::async_trait;
use k8s_openapi::api::extensions::v1beta1::{HTTPIngressPath, Ingress};
use kube::api::Meta;

use crate::{
    configurator::parser::{ParseRuleContext, Parser},
    general::Shared,
    rules::{Matcher, Rule, Service},
};

pub struct IngressParser {
    priority: u32,
}

impl IngressParser {
    pub fn new(priority: u32) -> Self {
        Self { priority }
    }

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
                log::error!("Could not get Service-Port");
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
}

#[async_trait]
impl Parser for IngressParser {
    async fn rule<'a>(
        &self,
        config: &serde_json::Value,
        _context: ParseRuleContext<'a>,
    ) -> Option<Rule> {
        let p: Ingress = match serde_json::from_value(config.to_owned()) {
            Ok(i) => i,
            Err(e) => {
                log::error!("Parsing Ingress: {:?}", e);
                return None;
            }
        };

        let name = Meta::name(&p);
        let spec = p.spec?;

        let rules = spec.rules?;
        let rule = rules.get(0)?;

        let host = rule.host.as_ref()?;
        let http = rule.http.as_ref()?;

        let raw_path = http.paths.get(0)?;
        Self::parse_path(raw_path, host.clone(), name, self.priority)
    }
}
