use crate::configurator::kubernetes::traefik_bindings::middleware;
use crate::rules::{action::CorsOpts, Action, Middleware};

#[cfg(test)]
use crate::configurator::kubernetes::general_crd;

use log::error;

pub fn parse_middleware(raw_mid: middleware::Config) -> Vec<Middleware> {
    let mut result = Vec::new();

    let name = raw_mid.metadata.name;

    for (key, value) in raw_mid.spec.iter() {
        match key.as_str() {
            "stripPrefix" => {
                let prefixes = value.get("prefixes").unwrap().as_array().unwrap();
                let raw_prefix = prefixes.get(0).unwrap();
                let prefix = raw_prefix.as_str().unwrap();

                let prefix_end = if prefix.as_bytes()[prefix.len() - 1] == b'/' {
                    prefix.len() - 1
                } else {
                    prefix.len()
                };

                result.push(Middleware::new(
                    &name,
                    Action::RemovePrefix(prefix[0..prefix_end].to_owned()),
                ));
            }
            "headers" => {
                let mut tmp_headers = Vec::<(String, String)>::new();
                let mut cors_options = CorsOpts {
                    origins: vec![],
                    max_age: None,
                    credentials: false,
                    methods: vec![],
                    headers: vec![],
                };
                let mut use_cors = false;

                for (header_key, header_values) in value.as_object().unwrap() {
                    let values = header_values.as_array().unwrap();

                    match header_key.as_str() {
                        "accessControlAllowOriginList" => {
                            use_cors = true;
                            for tmp_value in values {
                                cors_options
                                    .origins
                                    .push(tmp_value.as_str().unwrap().to_owned());
                            }
                        }
                        "accessControlAllowHeaders" => {
                            use_cors = true;
                            for tmp_value in values {
                                cors_options
                                    .headers
                                    .push(tmp_value.as_str().unwrap().to_owned());
                            }
                        }
                        "accessControlAllowMethods" => {
                            use_cors = true;
                            for tmp_value in values {
                                cors_options
                                    .methods
                                    .push(tmp_value.as_str().unwrap().to_owned());
                            }
                        }
                        _ => {
                            let mut header_value = "".to_owned();
                            for tmp_value in values {
                                header_value.push_str(tmp_value.as_str().unwrap());
                                header_value.push_str(", ");
                            }
                            header_value.pop();
                            header_value.pop();

                            tmp_headers.push((header_key.to_owned(), header_value));
                        }
                    };
                }

                if use_cors {
                    result.push(Middleware::new(&name, Action::CORS(cors_options)))
                } else {
                    result.push(Middleware::new(&name, Action::AddHeaders(tmp_headers)));
                }
            }
            "compress" => {
                result.push(Middleware::new(&name, Action::Compress));
            }
            _ => {
                error!("Unknown: '{:?}': '{:?}'", key, value);
            }
        };
    }

    result
}

#[test]
fn parse_middleware_stripprefix_trailing_slash() {
    let mut spec = std::collections::BTreeMap::new();
    let mut map = serde_json::value::Map::new();
    map.insert(
        "prefixes".to_owned(),
        serde_json::Value::Array(vec![serde_json::Value::String("/api/".to_owned())]),
    );
    spec.insert("stripPrefix".to_owned(), serde_json::Value::Object(map));

    let config = middleware::Config {
        api_version: "v1".to_owned(),
        kind: "middleware".to_owned(),
        metadata: general_crd::Metadata {
            name: "test".to_owned(),
            namespace: "default".to_owned(),
        },
        spec: spec,
    };

    assert_eq!(
        vec![Middleware::new(
            "test",
            Action::RemovePrefix("/api".to_owned())
        )],
        parse_middleware(config)
    );
}
#[test]
fn parse_middleware_stripprefix() {
    let mut spec = std::collections::BTreeMap::new();
    let mut map = serde_json::value::Map::new();
    map.insert(
        "prefixes".to_owned(),
        serde_json::Value::Array(vec![serde_json::Value::String("/api".to_owned())]),
    );
    spec.insert("stripPrefix".to_owned(), serde_json::Value::Object(map));

    let config = middleware::Config {
        api_version: "v1".to_owned(),
        kind: "middleware".to_owned(),
        metadata: general_crd::Metadata {
            name: "test".to_owned(),
            namespace: "default".to_owned(),
        },
        spec: spec,
    };

    assert_eq!(
        vec![Middleware::new(
            "test",
            Action::RemovePrefix("/api".to_owned())
        )],
        parse_middleware(config)
    );
}

#[test]
fn parse_middleware_cors_headers() {
    let mut spec = std::collections::BTreeMap::new();
    let mut map = serde_json::value::Map::new();
    map.insert(
        "accessControlAllowOriginList".to_owned(),
        serde_json::Value::Array(vec![serde_json::Value::String(
            "http://localhost".to_owned(),
        )]),
    );
    map.insert(
        "accessControlAllowHeaders".to_owned(),
        serde_json::Value::Array(vec![serde_json::Value::String("Authorization".to_owned())]),
    );
    map.insert(
        "accessControlAllowMethods".to_owned(),
        serde_json::Value::Array(vec![serde_json::Value::String("GET".to_owned())]),
    );
    spec.insert("headers".to_owned(), serde_json::Value::Object(map));

    let config = middleware::Config {
        api_version: "v1".to_owned(),
        kind: "middleware".to_owned(),
        metadata: general_crd::Metadata {
            name: "test".to_owned(),
            namespace: "default".to_owned(),
        },
        spec: spec,
    };

    assert_eq!(
        vec![Middleware::new(
            "test",
            Action::CORS(CorsOpts {
                origins: vec!["http://localhost".to_owned()],
                max_age: None,
                credentials: false,
                methods: vec!["GET".to_owned()],
                headers: vec!["Authorization".to_owned()],
            }),
        )],
        parse_middleware(config)
    );
}
