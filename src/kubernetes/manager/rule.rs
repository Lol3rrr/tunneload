use crate::kubernetes::ingressroute;
use crate::kubernetes::ingressroute::Config;
use crate::rules::{Matcher, Middleware, Rule, Service};

#[cfg(test)]
use crate::kubernetes::general_crd::Metadata;
#[cfg(test)]
use crate::rules::Action;

fn parse_matcher_rule(raw: &str) -> Option<Vec<Matcher>> {
    let mut result = Vec::new();

    let parts = raw.split("&&");
    for part in parts {
        let key_end = part.find('(').unwrap_or_else(|| part.len());
        let raw_key = part[0..key_end].to_owned();
        let key = raw_key.replace(' ', "");

        let content_parts: Vec<&str> = part.split('`').collect();
        let content = content_parts.get(1).unwrap();

        match key.as_str() {
            "Host" => {
                result.push(Matcher::Domain(content.to_string()));
            }
            "PathPrefix" => {
                result.push(Matcher::PathPrefix(content.to_string()));
            }
            _ => {
                println!("Unknown: '{}'", part);
            }
        };
    }

    Some(result)
}

fn parse_middleware(
    raw: &[ingressroute::Middleware],
    registered: &[Middleware],
) -> Vec<Middleware> {
    let mut result = Vec::new();

    for tmp in raw.iter() {
        for tmp_reg in registered.iter() {
            if tmp.name == tmp_reg.get_name() {
                result.push(tmp_reg.clone());
                break;
            }
        }
    }

    result
}

pub fn parse_rule(ingress: Config, middlewares: &[Middleware]) -> Option<Rule> {
    let name = ingress.metadata.name;

    let route = ingress.spec.routes.get(0).unwrap();
    let raw_rule = &route.rule;
    let priority = route.priority.unwrap_or(1);

    let matcher = match parse_matcher_rule(&raw_rule) {
        Some(m) => m,
        None => {
            return None;
        }
    };

    let rule_middleware = parse_middleware(&route.middlewares, middlewares);

    let route_service = route.services.get(0).unwrap();
    let address = format!(
        "{}:{}",
        route_service.name,
        route_service.port.unwrap_or(80)
    );
    let service = Service::new(address);

    Some(Rule::new(name, priority, matcher, rule_middleware, service))
}

#[test]
fn parse_rule_basic() {
    let ingress = Config {
        api_version: "".to_owned(),
        kind: "IngressRoute".to_owned(),
        metadata: Metadata {
            name: "test-route".to_owned(),
            namespace: "default".to_owned(),
        },
        spec: ingressroute::Spec {
            entry_points: vec![],
            routes: vec![ingressroute::Route {
                kind: "IngressRoute".to_owned(),
                middlewares: vec![],
                priority: Some(3),
                rule: "Host(`lol3r.net`)".to_owned(),
                services: vec![ingressroute::Service {
                    name: "personal".to_owned(),
                    port: Some(8080),
                }],
            }],
            tls: Some(ingressroute::TLS {
                secret_name: "test-tls".to_owned(),
            }),
        },
    };
    let middlewares = vec![];

    assert_eq!(
        Some(Rule::new(
            "test-route".to_owned(),
            3,
            vec![Matcher::Domain("lol3r.net".to_owned())],
            vec![],
            Service::new("personal:8080".to_owned())
        )),
        parse_rule(ingress, &middlewares)
    );
}
#[test]
fn parse_rule_multiple_matcher() {
    let ingress = Config {
        api_version: "".to_owned(),
        kind: "IngressRoute".to_owned(),
        metadata: Metadata {
            name: "test-route".to_owned(),
            namespace: "default".to_owned(),
        },
        spec: ingressroute::Spec {
            entry_points: vec![],
            routes: vec![ingressroute::Route {
                kind: "IngressRoute".to_owned(),
                middlewares: vec![],
                priority: Some(3),
                rule: "Host(`lol3r.net`) && PathPrefix(`/api/`)".to_owned(),
                services: vec![ingressroute::Service {
                    name: "personal".to_owned(),
                    port: Some(8080),
                }],
            }],
            tls: Some(ingressroute::TLS {
                secret_name: "test-tls".to_owned(),
            }),
        },
    };
    let middlewares = vec![];

    assert_eq!(
        Some(Rule::new(
            "test-route".to_owned(),
            3,
            vec![
                Matcher::Domain("lol3r.net".to_owned()),
                Matcher::PathPrefix("/api/".to_owned())
            ],
            vec![],
            Service::new("personal:8080".to_owned())
        )),
        parse_rule(ingress, &middlewares)
    );
}

#[test]
fn parse_rule_matcher_one_middleware() {
    let ingress = Config {
        api_version: "".to_owned(),
        kind: "IngressRoute".to_owned(),
        metadata: Metadata {
            name: "test-route".to_owned(),
            namespace: "default".to_owned(),
        },
        spec: ingressroute::Spec {
            entry_points: vec![],
            routes: vec![ingressroute::Route {
                kind: "IngressRoute".to_owned(),
                middlewares: vec![ingressroute::Middleware {
                    name: "header".to_owned(),
                }],
                priority: Some(3),
                rule: "Host(`lol3r.net`)".to_owned(),
                services: vec![ingressroute::Service {
                    name: "personal".to_owned(),
                    port: Some(8080),
                }],
            }],
            tls: Some(ingressroute::TLS {
                secret_name: "test-tls".to_owned(),
            }),
        },
    };
    let middlewares = vec![Middleware::new(
        "header",
        Action::AddHeader("test".to_owned(), "value".to_owned()),
    )];

    assert_eq!(
        Some(Rule::new(
            "test-route".to_owned(),
            3,
            vec![Matcher::Domain("lol3r.net".to_owned()),],
            vec![Middleware::new(
                "header",
                Action::AddHeader("test".to_owned(), "value".to_owned())
            )],
            Service::new("personal:8080".to_owned())
        )),
        parse_rule(ingress, &middlewares)
    );
}
