use crate::{
    configurator::parser::Parser,
    rules::{Action, CorsOpts},
};

use async_trait::async_trait;

/// This is the Parser for all the File-Configurator related stuff
#[derive(Debug, Clone)]
pub struct FileParser {}

impl FileParser {
    /// Creates a new Instance of the FileParser
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Parser for FileParser {
    async fn parse_action(&self, name: &str, config: &serde_json::Value) -> Option<Action> {
        match name {
            "RemovePrefix" => {
                let prefix = config.as_str()?;
                Some(Action::RemovePrefix(prefix.to_owned()))
            }
            "AddHeader" => {
                let headers = config.as_array()?;
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

                Some(Action::AddHeaders(result))
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
                    Some(tmp) => match tmp.as_u64() {
                        Some(tmp) => Some(tmp as usize),
                        None => None,
                    },
                    None => None,
                };

                let credentials = match config.get("credentials") {
                    Some(value) => match value.as_bool() {
                        Some(value) => value,
                        None => false,
                    },
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

                Some(Action::Cors(CorsOpts {
                    origins,
                    max_age,
                    credentials,
                    methods,
                    headers,
                }))
            }
            "BasicAuth" => {
                let auth = config.as_str()?;
                Some(Action::new_basic_auth_hashed(auth))
            }
            _ => None,
        }
    }
}
