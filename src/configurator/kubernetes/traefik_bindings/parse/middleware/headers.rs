use serde_json::Value;

use crate::rules::{Action, CorsOpts, Middleware};

pub fn parse(name: &str, value: &Value) -> Option<Middleware> {
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
        Some(Middleware::new(&name, Action::Cors(cors_options)))
    } else {
        Some(Middleware::new(&name, Action::AddHeaders(tmp_headers)))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn valid_add_headers() {
        let name = "test";
        let value = json!({
            "test-header-1": [
                "test-value-1-1",
                "test-value-1-2",
            ]
        });

        let result = parse(name, &value);
        assert_eq!(
            Some(Middleware::new(
                "test",
                Action::AddHeaders(vec![(
                    "test-header-1".to_owned(),
                    "test-value-1-1, test-value-1-2".to_owned()
                )])
            )),
            result
        );
    }

    #[test]
    fn only_cors_headers() {
        let name = "test";
        let value = json!({
            "accessControlAllowOriginList": [
                "http://example.net",
                "http://localhost",
            ]
        });

        let result = parse(name, &value);
        assert_eq!(
            Some(Middleware::new(
                "test",
                Action::Cors(CorsOpts {
                    origins: vec![
                        "http://example.net".to_owned(),
                        "http://localhost".to_owned()
                    ],
                    max_age: None,
                    credentials: false,
                    methods: vec![],
                    headers: vec![],
                })
            )),
            result
        );
    }

    #[test]
    fn cors_and_normal_mixed() {
        let name = "test";
        let value = json!({
            "test-header-1": [
                "test-value-1",
            ],
            "accessControlAllowOriginList": [
                "http://example.net",
                "http://localhost",
            ]
        });

        let result = parse(name, &value);
        assert_eq!(
            Some(Middleware::new(
                "test",
                Action::Cors(CorsOpts {
                    origins: vec![
                        "http://example.net".to_owned(),
                        "http://localhost".to_owned()
                    ],
                    max_age: None,
                    credentials: false,
                    methods: vec![],
                    headers: vec![],
                })
            )),
            result
        );
    }
}
