use std::{error::Error, fmt::Display};

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

#[derive(Debug)]
pub enum PathError {
    MissingServiceName,
    MissingServicePort,
    InvalidServicePort,
    MissingPath,
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
    ) -> Result<Rule, PathError> {
        let backend = &http_path.backend;
        let service_name = backend
            .service_name
            .as_ref()
            .ok_or(PathError::MissingServiceName)?;
        let service_port = match backend
            .service_port
            .as_ref()
            .ok_or(PathError::MissingServicePort)?
        {
            k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(v) => v,
            _ => {
                return Err(PathError::InvalidServicePort);
            }
        };
        let path = http_path.path.as_ref().ok_or(PathError::MissingPath)?;
        let matcher = Matcher::And(vec![
            Matcher::Domain(host),
            Matcher::PathPrefix(path.to_string()),
        ]);

        let addresses = vec![format!("{}:{}", service_name, service_port)];
        Ok(Rule::new(
            name,
            priority,
            matcher,
            Vec::new(),
            Shared::new(Service::new(service_name, addresses)),
        ))
    }
}

#[derive(Debug)]
pub enum RuleParseError {
    InvalidConfig(serde_json::Error),
    MissingSpec,
    MissingRules,
    MissingHost,
    MissingHttp,
    MissingPath,
    InvalidPath(PathError),
}

impl Display for RuleParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rule-Parse-Error")
    }
}
impl Error for RuleParseError {}

#[async_trait]
impl Parser for IngressParser {
    async fn rule<'a>(
        &self,
        config: &serde_json::Value,
        _context: ParseRuleContext<'a>,
    ) -> Result<Rule, Box<dyn Error>> {
        let p: Ingress = serde_json::from_value(config.to_owned())
            .map_err(|e| Box::new(RuleParseError::InvalidConfig(e)))?;

        let name = Meta::name(&p);
        let spec = p
            .spec
            .ok_or_else(|| Box::new(RuleParseError::MissingSpec))?;

        let rules = spec
            .rules
            .ok_or_else(|| Box::new(RuleParseError::MissingRules))?;
        let rule = rules
            .get(0)
            .ok_or_else(|| Box::new(RuleParseError::MissingRules))?;

        let host = rule
            .host
            .as_ref()
            .ok_or_else(|| Box::new(RuleParseError::MissingHost))?;
        let http = rule
            .http
            .as_ref()
            .ok_or_else(|| Box::new(RuleParseError::MissingHttp))?;

        let raw_path = http
            .paths
            .get(0)
            .ok_or_else(|| Box::new(RuleParseError::MissingPath))?;
        Self::parse_path(raw_path, host.clone(), name, self.priority)
            .map_err(|e| Box::new(RuleParseError::InvalidPath(e)) as Box<dyn Error>)
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
        let expected = Rule::new(
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
        );

        assert_eq!(true, result.is_ok());
        assert_eq!(expected, result.unwrap());
    }
}
