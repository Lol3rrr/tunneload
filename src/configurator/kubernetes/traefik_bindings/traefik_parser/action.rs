use crate::{
    configurator::kubernetes::general::load_secret,
    rules::{Action, CorsOpts},
};

pub fn strip_prefix(value: &serde_json::Value) -> Option<Action> {
    let prefixes = match value.get("prefixes") {
        Some(p) => p.as_array().unwrap(),
        None => {
            log::error!("StripPrefixes is missing 'prefixes'-Entry: {:?}", value);
            return None;
        }
    };
    let raw_prefix = match prefixes.get(0) {
        Some(r) => r,
        None => {
            log::error!("StripPrefixes missing prefix-Entries");
            return None;
        }
    };
    let mut prefix = raw_prefix.as_str().unwrap();

    if prefix.ends_with('/') {
        prefix = &prefix[..prefix.len() - 1];
    }

    Some(Action::RemovePrefix(prefix.to_owned()))
}

pub fn headers(value: &serde_json::Value) -> Option<Action> {
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
        Some(Action::Cors(cors_options))
    } else {
        Some(Action::AddHeaders(tmp_headers))
    }
}

pub async fn basic_auth(
    value: &serde_json::Value,
    client: kube::Client,
    namespace: &str,
) -> Option<Action> {
    let auth_value = value.as_object().unwrap();

    let raw_secret_name = match auth_value.get("secret") {
        Some(s) => s,
        None => {
            log::error!("Could not load Secret-Name for basic-Auth");
            return None;
        }
    };
    let secret_name = match raw_secret_name.as_str() {
        Some(s) => s,
        None => {
            log::error!("Secret-Name is not a String");
            return None;
        }
    };

    let raw_secret_value = match load_secret(client, namespace, secret_name).await {
        Some(s) => s,
        None => {
            log::error!("Loading Secret-Data");
            return None;
        }
    };

    let raw_users_data = match raw_secret_value.get("users") {
        Some(d) => d,
        None => {
            log::error!("Loading Users from Secret-Data");
            return None;
        }
    };

    let users_data = match std::str::from_utf8(&raw_users_data.0) {
        Ok(d) => d,
        Err(e) => {
            log::error!("Getting Base64-Data from Secret: {}", e);
            return None;
        }
    };

    Some(Action::new_basic_auth_hashed(users_data))
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn valid_add_headers() {
        let value = json!({
            "test-header-1": [
                "test-value-1-1",
                "test-value-1-2",
            ]
        });

        let result = headers(&value);
        assert_eq!(
            Some(Action::AddHeaders(vec![(
                "test-header-1".to_owned(),
                "test-value-1-1, test-value-1-2".to_owned()
            )])),
            result
        );
    }

    #[test]
    fn only_cors_headers() {
        let value = json!({
            "accessControlAllowOriginList": [
                "http://example.net",
                "http://localhost",
            ]
        });

        let result = headers(&value);
        assert_eq!(
            Some(Action::Cors(CorsOpts {
                origins: vec![
                    "http://example.net".to_owned(),
                    "http://localhost".to_owned()
                ],
                max_age: None,
                credentials: false,
                methods: vec![],
                headers: vec![],
            })),
            result
        );
    }

    #[test]
    fn cors_and_normal_mixed() {
        let value = json!({
            "test-header-1": [
                "test-value-1",
            ],
            "accessControlAllowOriginList": [
                "http://example.net",
                "http://localhost",
            ]
        });

        let result = headers(&value);
        assert_eq!(
            Some(Action::Cors(CorsOpts {
                origins: vec![
                    "http://example.net".to_owned(),
                    "http://localhost".to_owned()
                ],
                max_age: None,
                credentials: false,
                methods: vec![],
                headers: vec![],
            })),
            result
        );
    }

    #[test]
    fn non_trailing_path() {
        let value = json!({
            "prefixes": [
                "/test",
            ],
        });

        let result = strip_prefix(&value);
        assert_eq!(Some(Action::RemovePrefix("/test".to_owned())), result);
    }

    #[test]
    fn trailing_path() {
        let value = json!({
            "prefixes": [
                "/test/",
            ],
        });

        let result = strip_prefix(&value);
        assert_eq!(Some(Action::RemovePrefix("/test".to_owned())), result);
    }

    #[test]
    fn prefixes_key_missing() {
        let value = json!({
            "some_other_key": [
                "other_data",
            ],
        });

        let result = strip_prefix(&value);
        assert_eq!(None, result);
    }

    #[test]
    fn prefixes_empty() {
        let value = json!({
            "prefixes": [],
        });

        let result = strip_prefix(&value);
        assert_eq!(None, result);
    }
}
