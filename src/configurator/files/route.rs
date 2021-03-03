use crate::rules::{parser::parse_matchers, Middleware, Rule, Service};
use crate::{configurator::files::Config, general::Shared};

use log::error;
use serde::Deserialize;

#[cfg(test)]
use crate::rules::{Action, Matcher};

#[derive(Debug, Deserialize)]
pub struct ConfigRoute {
    name: String,
    #[serde(default = "default_priority")]
    priority: u32,
    rule: String,
    service: Vec<String>,
    middleware: Option<Vec<String>>,
}

fn default_priority() -> u32 {
    1
}

fn parse_route(content: &str, middlewares: &[Middleware]) -> Vec<Rule> {
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
        let service = Shared::new(Service::new(tmp_route.service));

        let middlewares = match tmp_route.middleware {
            None => Vec::new(),
            Some(m) => {
                let mut result = Vec::new();
                for tmp_middle_name in m {
                    for tmp_middle in middlewares {
                        if tmp_middle.get_name() == tmp_middle_name {
                            result.push(tmp_middle.clone());
                            break;
                        }
                    }
                }

                result
            }
        };

        result.push(Rule::new(name, priority, matcher, middlewares, service));
    }

    result
}

pub fn load_routes<P: AsRef<std::path::Path>>(path: P, middlewares: &[Middleware]) -> Vec<Rule> {
    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            error!("[Config] Reading File: {}", e);
            return Vec::new();
        }
    };

    parse_route(&contents, middlewares)
}

#[test]
fn parse_empty() {
    let content = "";
    let middlewares = vec![];
    assert_eq!(vec![] as Vec<Rule>, parse_route(content, &middlewares));
}

#[test]
fn parse_basic() {
    let content = "
    routes:
        - name: Test
          priority: 1
          rule: Host(`example.com`)
          service:
            - out:30000
        ";
    let middlewares = vec![];

    assert_eq!(
        vec![Rule::new(
            "Test".to_owned(),
            1,
            Matcher::Domain("example.com".to_owned()),
            vec![],
            Shared::new(Service::new(vec!["out:30000".to_owned()]))
        )],
        parse_route(content, &middlewares)
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
            - out:30000
        ";
    let middlewares = vec![];

    assert_eq!(
        vec![Rule::new(
            "Test".to_owned(),
            1,
            Matcher::And(vec![
                Matcher::Domain("example.com".to_owned()),
                Matcher::PathPrefix("/api/".to_owned())
            ]),
            vec![],
            Shared::new(Service::new(vec!["out:30000".to_owned()]))
        )],
        parse_route(content, &middlewares)
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
            - out:30000
          middleware:
            - test-1
            - test-2
        ";
    let middlewares = vec![
        Middleware::new("test-1", Action::RemovePrefix("/api/".to_owned())),
        Middleware::new(
            "test-2",
            Action::AddHeaders(vec![("test-key".to_owned(), "test-value".to_owned())]),
        ),
    ];
    assert_eq!(
        vec![Rule::new(
            "Test".to_owned(),
            1,
            Matcher::Domain("example.com".to_owned()),
            middlewares.clone(),
            Shared::new(Service::new(vec!["out:30000".to_owned()]))
        )],
        parse_route(content, &middlewares)
    );
}
