use crate::kubernetes::middleware;
use crate::rules::{Action, Middleware};

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

                result.push(Middleware::new(
                    &name,
                    Action::RemovePrefix(prefix.to_owned()),
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
