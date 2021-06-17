use async_trait::async_trait;

use crate::{
    configurator::{
        parser::{ParseRuleContext, Parser},
        MiddlewareList,
    },
    general::Shared,
    rules::{parser::parse_matchers, Action, Middleware, Rule, RuleTLS},
};

use super::ingressroute::{self, Config};

mod action;

/// This is the Parser for all the Traefik related Parts
#[derive(Clone)]
pub struct TraefikParser {
    client: Option<kube::Client>,
    namespace: Option<String>,
}

impl TraefikParser {
    /// Creates a new Instance of the Parser
    pub fn new(client: Option<kube::Client>, namespace: Option<String>) -> Self {
        Self { client, namespace }
    }

    fn find_middlewares(
        raw: &[ingressroute::Middleware],
        registered: &MiddlewareList,
    ) -> Vec<Shared<Middleware>> {
        let mut result = Vec::new();

        for tmp in raw.iter() {
            result.push(registered.get_with_default(&tmp.name));
        }

        result
    }
}

impl Default for TraefikParser {
    fn default() -> Self {
        Self {
            client: None,
            namespace: None,
        }
    }
}

#[async_trait]
impl Parser for TraefikParser {
    async fn parse_action(&self, name: &str, config: &serde_json::Value) -> Option<Action> {
        match name {
            "stripPrefix" => action::strip_prefix(config),
            "headers" => action::headers(config),
            "compress" => Some(Action::Compress),
            "basicAuth" => {
                action::basic_auth(
                    config,
                    self.client.clone().unwrap(),
                    self.namespace.as_ref().unwrap(),
                )
                .await
            }
            _ => None,
        }
    }

    async fn rule<'a>(
        &self,
        raw_config: &serde_json::Value,
        context: ParseRuleContext<'a>,
    ) -> Option<Rule> {
        let ingress: Config = match serde_json::from_value(raw_config.to_owned()) {
            Ok(i) => i,
            Err(e) => {
                tracing::error!("Parsing Config: {:?}", e);
                return None;
            }
        };

        let name = ingress.metadata.name;

        let route = match ingress.spec.routes.get(0) {
            Some(r) => r,
            None => {
                tracing::error!("Rule config is missing Routes");
                return None;
            }
        };
        let raw_rule = &route.rule;
        let priority = route.priority.unwrap_or(1);

        let matcher = match parse_matchers(&raw_rule) {
            Some(m) => m,
            None => {
                tracing::error!("Missing or Malformed Matcher");
                return None;
            }
        };

        let rule_middleware = Self::find_middlewares(&route.middlewares, context.middlewares);

        let route_service = match route.services.get(0) {
            Some(s) => s,
            None => {
                tracing::error!("Missing Service");
                return None;
            }
        };
        let service = context.services.get_with_default(&route_service.name);

        let mut rule = Rule::new(name, priority, matcher.clone(), rule_middleware, service);

        if let Some(tls) = ingress.spec.tls {
            if let Some(name) = tls.secret_name {
                rule.set_tls(RuleTLS::Secret(name));
                return Some(rule);
            }
        }

        // Attempt to generate the Domain
        if let Some(tx) = context.cert_queue {
            let domain = match matcher.get_host() {
                Some(d) => d,
                None => {
                    tracing::error!("Could not get Domain to request Certificate");
                    return Some(rule);
                }
            };
            tx.request(domain.clone());

            rule.set_tls(RuleTLS::Generate(domain));
            return Some(rule);
        }

        Some(rule)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{
        configurator::ServiceList,
        rules::{Matcher, Service},
    };

    use super::*;

    #[tokio::test]
    async fn action_strip_prefix() {
        let parser = TraefikParser::new(None, None);

        let result = parser
            .parse_action(
                "stripPrefix",
                &json!({
                    "prefixes": [
                        "/api",
                    ],
                }),
            )
            .await;
        let expected = Some(Action::RemovePrefix("/api".to_owned()));

        assert_eq!(expected, result);
    }

    #[tokio::test]
    async fn action_add_headers() {
        let parser = TraefikParser::new(None, None);

        let result = parser
            .parse_action(
                "headers",
                &json!({
                    "test-header": [
                        "test-value",
                    ],
                }),
            )
            .await;
        let expected = Some(Action::AddHeaders(vec![(
            "test-header".to_owned(),
            "test-value".to_owned(),
        )]));

        assert_eq!(expected, result);
    }

    #[tokio::test]
    async fn parse_rule_matcher_one_middleware() {
        let ingress = json!({
            "apiVersion": "",
            "kind": "IngressRoute",
            "metadata": {
                "name": "test-route",
                "namespace": "default",
            },
            "spec":  {
                "entryPoints": [],
                "routes": [ {
                    "kind": "IngressRoute",
                    "middlewares": [ {
                        "name": "header",
                    }],
                    "priority": 3,
                    "match": "Host(`lol3r.net`)",
                    "services": [ {
                        "name": "personal",
                        "port": 8080,
                    }],
                }],
                "tls": {
                    "secretName": "test-tls",
                },
            },
        });

        let middlewares = MiddlewareList::new();
        middlewares.set(Middleware::new(
            "header",
            Action::AddHeaders(vec![("test".to_owned(), "value".to_owned())]),
        ));

        let services = ServiceList::new();
        services.set_service(Service::new(
            "personal",
            vec!["192.168.0.0:8080".to_owned()],
        ));

        let context = ParseRuleContext {
            services: &services,
            middlewares: &middlewares,
            cert_queue: None,
        };

        let mut expected_rule = Rule::new(
            "test-route".to_owned(),
            3,
            Matcher::Domain("lol3r.net".to_owned()),
            vec![Shared::new(Middleware::new(
                "header",
                Action::AddHeaders(vec![("test".to_owned(), "value".to_owned())]),
            ))],
            Shared::new(Service::new(
                "personal",
                vec!["192.168.0.0:8080".to_owned()],
            )),
        );
        expected_rule.set_tls(RuleTLS::Secret("test-tls".to_owned()));

        let parser = TraefikParser::new(None, None);
        assert_eq!(Some(expected_rule), parser.rule(&ingress, context).await);
    }
}
