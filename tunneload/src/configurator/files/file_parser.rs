use std::{error::Error, fmt::Display};

use crate::configurator::parser::{ParseRuleContext, Parser};
use general::{Group, Name};
use rules::{
    parser::{parse_matchers, ParseMatcherError},
    Action, CorsOpts, Rule,
};

use async_trait::async_trait;

use super::route::ConfigRoute;

/// This is the Parser for all the File-Configurator related stuff
#[derive(Debug, Clone)]
pub struct FileParser {}

impl FileParser {
    /// Creates a new Instance of the FileParser
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for FileParser {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum ActionParseError {
    InvalidConfig,
    UnknownAction,
}

impl Display for ActionParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Action-Parse-Error")
    }
}
impl Error for ActionParseError {}

#[derive(Debug)]
pub enum RuleParseError {
    InvalidConfig(serde_json::Error),
    InvalidMatchers(ParseMatcherError),
}

impl Display for RuleParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rule-Parse-Error")
    }
}
impl Error for RuleParseError {}

#[async_trait]
impl Parser for FileParser {
    async fn parse_action(
        &self,
        name: &str,
        config: &serde_json::Value,
    ) -> Result<Action, Box<dyn Error>> {
        match name {
            "RemovePrefix" => {
                let prefix = config
                    .as_str()
                    .ok_or_else(|| Box::new(ActionParseError::InvalidConfig))?;
                Ok(Action::RemovePrefix(prefix.to_owned()))
            }
            "AddHeader" => {
                let headers = config
                    .as_array()
                    .ok_or_else(|| Box::new(ActionParseError::InvalidConfig))?;
                let mut result = Vec::with_capacity(headers.len());
                for tmp in headers.iter() {
                    let raw_key = match tmp.get("key") {
                        Some(k) => k,
                        None => continue,
                    };
                    let raw_value = match tmp.get("value") {
                        Some(v) => v,
                        None => continue,
                    };

                    let key = match raw_key.as_str() {
                        Some(k) => k,
                        None => continue,
                    };
                    let value = match raw_value.as_str() {
                        Some(v) => v,
                        None => continue,
                    };

                    result.push((key.to_owned(), value.to_owned()));
                }

                Ok(Action::AddHeaders(result))
            }
            "CORS" => {
                let origins =
                    config
                        .get("origins")
                        .map(|tmp| tmp.as_array())
                        .map_or(Vec::new(), |raw| {
                            let tmp = match raw {
                                Some(t) => t,
                                None => return Vec::new(),
                            };

                            let mut result: Vec<String> = Vec::with_capacity(tmp.len());

                            for raw in tmp {
                                if let Some(tmp_str) = raw.as_str() {
                                    result.push(tmp_str.to_string());
                                }
                            }

                            result
                        });

                let max_age = match config.get("max_age") {
                    Some(tmp) => tmp.as_u64().map(|tmp| tmp as usize),
                    None => None,
                };

                let credentials = match config.get("credentials") {
                    Some(value) => value.as_bool().unwrap_or(false),
                    None => false,
                };

                let methods =
                    config
                        .get("methods")
                        .map(|tmp| tmp.as_array())
                        .map_or(Vec::new(), |raw| {
                            let tmp = match raw {
                                Some(t) => t,
                                None => return Vec::new(),
                            };

                            let mut result = Vec::with_capacity(tmp.len());

                            for raw in tmp {
                                if let Some(tmp_str) = raw.as_str() {
                                    result.push(tmp_str.to_string());
                                }
                            }

                            result
                        });

                let headers =
                    config
                        .get("headers")
                        .map(|tmp| tmp.as_array())
                        .map_or(Vec::new(), |raw| {
                            let tmp = match raw {
                                Some(t) => t,
                                None => return Vec::new(),
                            };

                            let mut result = Vec::with_capacity(tmp.len());

                            for raw in tmp {
                                if let Some(tmp_str) = raw.as_str() {
                                    result.push(tmp_str.to_string());
                                }
                            }

                            result
                        });

                Ok(Action::Cors(CorsOpts {
                    origins,
                    max_age,
                    credentials,
                    methods,
                    headers,
                }))
            }
            "BasicAuth" => {
                let auth = config
                    .as_str()
                    .ok_or_else(|| Box::new(ActionParseError::InvalidConfig))?;
                Ok(Action::new_basic_auth_hashed(auth))
            }
            _ => Err(Box::new(ActionParseError::UnknownAction)),
        }
    }

    async fn rule<'a>(
        &self,
        config: &serde_json::Value,
        context: ParseRuleContext<'a>,
    ) -> Result<Rule, Box<dyn Error>> {
        let route: ConfigRoute = match serde_json::from_value(config.to_owned()) {
            Ok(d) => d,
            Err(e) => {
                return Err(Box::new(RuleParseError::InvalidConfig(e)));
            }
        };

        let name = route.name;
        let priority = route.priority;
        let matcher = parse_matchers(&route.rule)
            .map_err(|e| Box::new(RuleParseError::InvalidMatchers(e)))?;

        let service_name = Name::parse(&route.service, || Group::File {});
        let service = context.services.get_with_default(service_name);

        let middlewares = match route.middleware {
            Some(m) => {
                let mut result = Vec::new();
                for raw_mid_name in m.iter() {
                    let mid_name = Name::parse(raw_mid_name, || Group::File {});
                    result.push(context.middlewares.get_with_default(mid_name));
                }

                result
            }
            None => Vec::new(),
        };

        let rule_name = Name::new(name, Group::File {});
        Ok(Rule::new(
            rule_name,
            priority,
            matcher,
            middlewares,
            service,
        ))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use general::Shared;
    use rules::{Matcher, Middleware, Service};

    use crate::configurator::{MiddlewareList, ServiceList};

    use super::*;

    #[tokio::test]
    async fn full_rule() {
        let parser = FileParser::default();

        let config = json!({
            "name": "test-name",
            "priority": 5,
            "rule": "PathPrefix(`/test/`)",
            "service": "test-service",
            "middleware": [
                "test-middleware"
            ],
        });
        let context = ParseRuleContext {
            middlewares: &MiddlewareList::new(),
            services: &ServiceList::new(),
            cert_queue: None,
        };

        let result = parser.rule(&config, context).await;
        let expected = Rule::new(
            Name::new("test-name", Group::File {}),
            5,
            Matcher::PathPrefix("/test/".to_owned()),
            vec![Shared::new(Middleware::new(
                Name::new("test-middleware", Group::File {}),
                Action::Noop,
            ))],
            Shared::new(Service::new(
                Name::new("test-service", Group::File {}),
                vec![],
            )),
        );

        assert_eq!(true, result.is_ok());
        assert_eq!(expected, result.unwrap());
    }

    #[tokio::test]
    async fn minimal_rule() {
        let parser = FileParser::default();

        let config = json!({
            "name": "test-name",
            "rule": "PathPrefix(`/test/`)",
            "service": "test-service",
        });
        let context = ParseRuleContext {
            middlewares: &MiddlewareList::new(),
            services: &ServiceList::new(),
            cert_queue: None,
        };

        let result = parser.rule(&config, context).await;
        let expected = Rule::new(
            Name::new("test-name", Group::File {}),
            1,
            Matcher::PathPrefix("/test/".to_owned()),
            vec![],
            Shared::new(Service::new(
                Name::new("test-service", Group::File {}),
                vec![],
            )),
        );

        assert_eq!(true, result.is_ok());
        assert_eq!(expected, result.unwrap());
    }
}
