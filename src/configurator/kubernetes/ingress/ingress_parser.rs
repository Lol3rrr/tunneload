use async_trait::async_trait;
use k8s_openapi::api::extensions::v1beta1::{HTTPIngressPath, Ingress};
use kube::api::Meta;

use crate::{
    configurator::parser::{ParseRuleContext, Parser},
    general::Shared,
    rules::{Matcher, Rule, Service},
};

/// The Parser for the Kubernetes-Ingress-Configuration
pub struct IngressParser {
    priority: u32,
}

impl IngressParser {
    /// Creates a new Instance of the Parser using the given
    /// initial Values
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

#[cfg(test)]
mod tests {
    use k8s_openapi::{
        api::extensions::v1beta1::{
            HTTPIngressRuleValue, IngressBackend, IngressRule, IngressSpec,
        },
        apimachinery::pkg::util::intstr::IntOrString,
    };
    use kube::api::ObjectMeta;

    use crate::configurator::{MiddlewareList, ServiceList};

    use super::*;

    #[tokio::test]
    async fn valid_rule() {
        let parser = IngressParser::new(10);

        let ingress_rule = Ingress {
            metadata: ObjectMeta {
                name: Some("test-rule".to_owned()),
                ..Default::default()
            },
            spec: Some(IngressSpec {
                rules: Some(vec![IngressRule {
                    host: Some("example.com".to_owned()),
                    http: Some(HTTPIngressRuleValue {
                        paths: vec![HTTPIngressPath {
                            path: Some("/test/".to_owned()),
                            backend: IngressBackend {
                                service_name: Some("test-service".to_owned()),
                                service_port: Some(IntOrString::Int(8080)),
                                ..Default::default()
                            },
                            ..Default::default()
                        }],
                    }),
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let config = serde_json::to_value(ingress_rule).unwrap();
        let context = ParseRuleContext {
            middlewares: &MiddlewareList::default(),
            services: &ServiceList::default(),
            cert_queue: None,
        };

        let result = parser.rule(&config, context).await;
        let expected = Some(Rule::new(
            "test-rule".to_owned(),
            10,
            Matcher::And(vec![
                Matcher::Domain("example.com".to_owned()),
                Matcher::PathPrefix("/test/".to_owned()),
            ]),
            vec![],
            Shared::new(Service::new(
                "test-service".to_owned(),
                vec!["test-service:8080".to_owned()],
            )),
        ));

        assert_eq!(expected, result);
    }
}
