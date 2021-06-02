use crate::rules::{parser::parse_matchers, Middleware, Rule};
use crate::tls::auto::CertificateQueue;
use crate::{configurator::ServiceList, general::Shared};
use crate::{
    configurator::{
        kubernetes::traefik_bindings::ingressroute::{self, Config},
        MiddlewareList,
    },
    rules::RuleTLS,
};

#[derive(Debug, PartialEq)]
pub enum ParseRuleError {
    MissingRoute,
    MissingMatcher,
    MissingService,
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

/// Parses the given Config as a Rule
pub fn parse_rule(
    ingress: Config,
    middlewares: &MiddlewareList,
    services: &ServiceList,
    cert_queue: &Option<&CertificateQueue>,
) -> Result<Rule, ParseRuleError> {
    let name = ingress.metadata.name;

    let route = match ingress.spec.routes.get(0) {
        Some(r) => r,
        None => return Err(ParseRuleError::MissingRoute),
    };
    let raw_rule = &route.rule;
    let priority = route.priority.unwrap_or(1);

    let matcher = match parse_matchers(&raw_rule) {
        Some(m) => m,
        None => return Err(ParseRuleError::MissingMatcher),
    };

    let rule_middleware = find_middlewares(&route.middlewares, middlewares);

    let route_service = match route.services.get(0) {
        Some(s) => s,
        None => return Err(ParseRuleError::MissingService),
    };
    let service = services.get_with_default(&route_service.name);

    let mut rule = Rule::new(name, priority, matcher.clone(), rule_middleware, service);

    if let Some(tls) = ingress.spec.tls {
        if let Some(name) = tls.secret_name {
            rule.set_tls(RuleTLS::Secret(name));
            return Ok(rule);
        }
    }

    // Attempt to generate the Domain
    if let Some(tx) = cert_queue {
        let domain = match matcher.get_host() {
            Some(d) => d,
            None => {
                log::error!("Could not get Domain to request Certificate");
                return Ok(rule);
            }
        };
        tx.request(domain.clone());

        rule.set_tls(RuleTLS::Generate(domain));
        return Ok(rule);
    }

    Ok(rule)
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
                tls: Some(ingressroute::Tls {
                    secret_name: Some("test-tls".to_owned()),
                }),
            },
        };

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

        assert_eq!(
            Ok(expected_rule),
            parse_rule(ingress, &middlewares, &services, &None)
        );
    }
}
