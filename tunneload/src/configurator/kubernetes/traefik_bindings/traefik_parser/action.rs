use crate::{
    configurator::kubernetes::traefik_bindings::middleware,
    util::kubernetes::secret::{load_secret, LoadSecretError},
};
use rules::{Action, CorsOpts};

#[derive(Debug, PartialEq)]
pub enum StripPrefixError {
    InvalidConfig(String),
    MissingPrefix,
}

/// Attempts to parse the given Value as the configuration for the Strip-Prefix
/// Action
pub fn strip_prefix(value: &serde_json::Value) -> Result<Action, StripPrefixError> {
    let parsed: middleware::StripPrefix = serde_json::from_value(value.clone())
        .map_err(|_| StripPrefixError::InvalidConfig(serde_json::to_string(&value).unwrap()))?;

    let mut prefix: &str = match parsed.prefixes.get(0) {
        Some(r) => r.as_ref(),
        None => return Err(StripPrefixError::MissingPrefix),
    };

    if prefix.ends_with('/') {
        prefix = &prefix[..prefix.len() - 1];
    }

    Ok(Action::RemovePrefix(prefix.to_owned()))
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

    for (header_key, header_values) in value.as_object()? {
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

#[derive(Debug)]
pub enum BasicAuthError {
    InvalidConfig(String),
    MissingSecret,
    InvalidSecretName,
    LoadingSecret(LoadSecretError),
    MissingUsers,
    InvalidUsersData(std::str::Utf8Error),
}

pub async fn basic_auth(
    value: &serde_json::Value,
    client: kube::Client,
    namespace: &str,
) -> Result<Action, BasicAuthError> {
    let auth_value = match value.as_object() {
        Some(v) => v,
        None => {
            return Err(BasicAuthError::InvalidConfig(
                serde_json::to_string(&value).unwrap(),
            ))
        }
    };

    let raw_secret_name = auth_value
        .get("secret")
        .ok_or(BasicAuthError::MissingSecret)?;
    let secret_name = raw_secret_name
        .as_str()
        .ok_or(BasicAuthError::InvalidSecretName)?;

    let raw_secret_value = load_secret(client, namespace, secret_name)
        .await
        .map_err(BasicAuthError::LoadingSecret)?;

    let raw_users_data = raw_secret_value
        .get("users")
        .ok_or(BasicAuthError::MissingUsers)?;

    let users_data =
        std::str::from_utf8(&raw_users_data.0).map_err(BasicAuthError::InvalidUsersData)?;

    Ok(Action::new_basic_auth_hashed(users_data))
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
        assert_eq!(Ok(Action::RemovePrefix("/test".to_owned())), result);
    }

    #[test]
    fn trailing_path() {
        let value = json!({
            "prefixes": [
                "/test/",
            ],
        });

        let result = strip_prefix(&value);
        assert_eq!(Ok(Action::RemovePrefix("/test".to_owned())), result);
    }

    #[test]
    fn prefixes_key_missing() {
        let value = json!({
            "some_other_key": [
                "other_data",
            ],
        });

        let result = strip_prefix(&value);
        assert_eq!(
            Err(StripPrefixError::InvalidConfig(
                serde_json::to_string(&value).unwrap()
            )),
            result
        );
    }

    #[test]
    fn prefixes_empty() {
        let value = json!({
            "prefixes": [],
        });

        let result = strip_prefix(&value);
        assert_eq!(Err(StripPrefixError::MissingPrefix), result);
    }
}
