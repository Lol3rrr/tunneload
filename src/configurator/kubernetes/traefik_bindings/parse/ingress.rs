use crate::configurator::kubernetes::traefik_bindings::ingressroute::{self, Config};
use crate::configurator::ServiceList;
use crate::rules::{parser::parse_matchers, Middleware, Rule};

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

pub fn parse_rule(
    ingress: Config,
    middlewares: &[Middleware],
    services: &ServiceList,
) -> Option<Rule> {
    let name = ingress.metadata.name;

    let route = ingress.spec.routes.get(0).unwrap();
    let raw_rule = &route.rule;
    let priority = route.priority.unwrap_or(1);

    let matcher = match parse_matchers(&raw_rule) {
        Some(m) => m,
        None => {
            return None;
        }
    };

    let rule_middleware = parse_middleware(&route.middlewares, middlewares);

    let route_service = route.services.get(0).unwrap();
    let service = match services.get_service(&route_service.name) {
        Some(s) => s,
        None => return None,
    };

    let mut rule = Rule::new(name, priority, matcher, rule_middleware, service);

    if let Some(tls) = ingress.spec.tls {
        if let Some(name) = tls.secret_name {
            rule.set_tls(name);
        }
    }

    Some(rule)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        configurator::kubernetes::general_crd::Metadata,
        general::Shared,
        rules::{Action, Matcher, Service},
    };

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
                entry_points: Some(vec![]),
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
                    secret_name: Some("test-tls".to_owned()),
                }),
            },
        };
        let middlewares = vec![Middleware::new(
            "header",
            Action::AddHeaders(vec![("test".to_owned(), "value".to_owned())]),
        )];

        let services = ServiceList::new();
        services.set_service(Service::new(
            "personal",
            vec!["192.168.0.0:8080".to_owned()],
        ));

        let mut expected_rule = Rule::new(
            "test-route".to_owned(),
            3,
            Matcher::Domain("lol3r.net".to_owned()),
            vec![Middleware::new(
                "header",
                Action::AddHeaders(vec![("test".to_owned(), "value".to_owned())]),
            )],
            Shared::new(Service::new(
                "personal",
                vec!["192.168.0.0:8080".to_owned()],
            )),
        );
        expected_rule.set_tls("test-tls".to_owned());

        assert_eq!(
            Some(expected_rule),
            parse_rule(ingress, &middlewares, &services)
        );
    }
}
