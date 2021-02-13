use crate::kubernetes::traefik_bindings::middleware;
use crate::rules::{Action, Middleware};

#[cfg(test)]
use crate::kubernetes::general_crd;

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
                for (header_key, header_values) in value.as_object().unwrap() {
                    let mut header_value = "".to_owned();
                    for tmp_value in header_values.as_array().unwrap() {
                        header_value.push_str(tmp_value.as_str().unwrap());
                        header_value.push_str(", ");
                    }
                    header_value.remove(header_value.len() - 1);
                    header_value.remove(header_value.len() - 1);

                    result.push(Middleware::new(
                        &name,
                        Action::AddHeader(header_key.to_owned(), header_value),
                    ));
                }
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
