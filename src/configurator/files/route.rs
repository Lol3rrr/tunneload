use crate::configurator::{files::Config, ServiceList};
use crate::{
    configurator::MiddlewareList,
    rules::{parser::parse_matchers, Rule},
};

use log::error;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ConfigService {
    name: String,
    addresses: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigRoute {
    name: String,
    #[serde(default = "default_priority")]
    priority: u32,
    rule: String,
    service: ConfigService,
    middleware: Option<Vec<String>>,
}

fn default_priority() -> u32 {
    1
}

fn parse_route(content: &str, middlewares: &MiddlewareList, services: &ServiceList) -> Vec<Rule> {
    let deserialized: Config = match serde_yaml::from_str(content) {
        Ok(d) => d,
        Err(e) => {
            error!("[Config] Parsing YAML: {}", e);
            return Vec::new();
        }
    };

    if deserialized.routes.is_none() {
        return Vec::new();
    }

    let mut result = Vec::new();
    for tmp_route in deserialized.routes.unwrap() {
        let name = tmp_route.name;
        let priority = tmp_route.priority;
        let matcher = match parse_matchers(&tmp_route.rule) {
            Some(m) => m,
            None => {
                continue;
            }
        };
        let service = services.get_with_default(tmp_route.service.name);

        let middlewares = match tmp_route.middleware {
            None => Vec::new(),
            Some(m) => {
                let mut result = Vec::new();
                for tmp_middle_name in m {
                    result.push(middlewares.get_with_default(&tmp_middle_name));
                }

                result
            }
        };

        let n_rule = Rule::new(name, priority, matcher, middlewares, service);
        result.push(n_rule);
    }

    result
}

/// Loads all the Rules/Routes from the given File
pub fn load_routes<P: AsRef<std::path::Path>>(
    path: P,
    middlewares: &MiddlewareList,
    services: &ServiceList,
) -> Vec<Rule> {
    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            error!("[Config] Reading File: {}", e);
            return Vec::new();
        }
    };

    parse_route(&contents, middlewares, services)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::general::Shared;
    use crate::rules::{Action, Matcher, Middleware, Service};

    #[test]
    fn parse_empty() {
        let content = "";
        let middlewares = MiddlewareList::new();
        assert_eq!(
            vec![] as Vec<Rule>,
            parse_route(content, &middlewares, &ServiceList::new())
        );
    }

    #[test]
    fn parse_basic() {
        let content = "
    routes:
        - name: Test
          priority: 1
          rule: Host(`example.com`)
          service:
            name: test
            addresses:
              - out:30000
        ";
        let middlewares = MiddlewareList::new();

        assert_eq!(
            vec![Rule::new(
                "Test".to_owned(),
                1,
                Matcher::Domain("example.com".to_owned()),
                vec![],
                Shared::new(Service::new("test", vec!["out:30000".to_owned()]))
            )],
            parse_route(content, &middlewares, &ServiceList::new())
        );
    }
    #[test]
    fn parse_basic_two_rules() {
        let content = "
    routes:
        - name: Test
          priority: 1
          rule: Host(`example.com`) && PathPrefix(`/api/`)
          service: 
            name: test
            addresses:
              - out:30000
        ";
        let middlewares = MiddlewareList::new();

        assert_eq!(
            vec![Rule::new(
                "Test".to_owned(),
                1,
                Matcher::And(vec![
                    Matcher::Domain("example.com".to_owned()),
                    Matcher::PathPrefix("/api/".to_owned())
                ]),
                vec![],
                Shared::new(Service::new("test", vec!["out:30000".to_owned()]))
            )],
            parse_route(content, &middlewares, &ServiceList::new())
        );
    }

    #[test]
    fn parse_basic_with_middleware() {
        let content = "
    routes:
        - name: Test
          priority: 1
          rule: Host(`example.com`)
          service:
            name: test
            addresses:
              - out:30000
          middleware:
            - test-1
            - test-2
        ";

        let middlewares = MiddlewareList::new();
        middlewares.set_middleware(Middleware::new(
            "test-1",
            Action::RemovePrefix("/api/".to_owned()),
        ));
        middlewares.set_middleware(Middleware::new(
            "test-2",
            Action::AddHeaders(vec![("test-key".to_owned(), "test-value".to_owned())]),
        ));
        assert_eq!(
            vec![Rule::new(
                "Test".to_owned(),
                1,
                Matcher::Domain("example.com".to_owned()),
                vec![
                    Shared::new(Middleware::new(
                        "test-1",
                        Action::RemovePrefix("/api/".to_owned()),
                    )),
                    Shared::new(Middleware::new(
                        "test-2",
                        Action::AddHeaders(vec![("test-key".to_owned(), "test-value".to_owned())]),
                    ))
                ],
                Shared::new(Service::new("test", vec!["out:30000".to_owned()]))
            )],
            parse_route(content, &middlewares, &ServiceList::new())
        );
    }
}
