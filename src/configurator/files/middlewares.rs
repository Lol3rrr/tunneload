use crate::configurator::files::Config;
use crate::rules::{action::CorsOpts, Action, Middleware};

use log::error;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ConfigMiddleware {
    name: String,
    #[serde(rename = "RemovePrefix")]
    remove_prefix: Option<String>,
    #[serde(rename = "AddHeader")]
    add_header: Option<Vec<AddHeaderConfig>>,
    #[serde(rename = "CORS")]
    cors: Option<CORSConfig>,
    #[serde(rename = "BasicAuth")]
    basic_auth: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddHeaderConfig {
    key: String,
    value: String,
}

#[derive(Debug, Deserialize)]
pub struct CORSConfig {
    origins: Option<Vec<String>>,
    max_age: Option<usize>,
    credentials: Option<bool>,
    methods: Option<Vec<String>>,
    headers: Option<Vec<String>>,
}

fn parse_middlewares(content: &str) -> Vec<Middleware> {
    let deserialized: Config = match serde_yaml::from_str(content) {
        Ok(d) => d,
        Err(e) => {
            error!("[Config] Parsing YAML: {}", e);
            return Vec::new();
        }
    };

    if deserialized.middleware.is_none() {
        return Vec::new();
    }

    let mut result = Vec::new();

    for tmp_middle in deserialized.middleware.unwrap() {
        let name = tmp_middle.name;
        if let Some(remove_prefix) = tmp_middle.remove_prefix {
            result.push(Middleware::new(&name, Action::RemovePrefix(remove_prefix)));
            continue;
        }

        if let Some(add_headers) = tmp_middle.add_header {
            let mut tmp_headers = Vec::<(String, String)>::new();
            for header in add_headers {
                tmp_headers.push((header.key, header.value));
            }

            result.push(Middleware::new(&name, Action::AddHeaders(tmp_headers)));
            continue;
        }

        if let Some(cors) = tmp_middle.cors {
            let opts = CorsOpts {
                origins: cors.origins.unwrap_or_default(),
                max_age: cors.max_age,
                credentials: cors.credentials.unwrap_or(false),
                methods: cors.methods.unwrap_or_default(),
                headers: cors.headers.unwrap_or_default(),
            };

            result.push(Middleware::new(&name, Action::Cors(opts)));
            continue;
        }

        if let Some(auth_str) = tmp_middle.basic_auth {
            result.push(Middleware::new(
                &name,
                Action::new_basic_auth_hashed(auth_str),
            ));
            continue;
        }
    }

    result
}

pub fn load_middlewares<P: AsRef<std::path::Path>>(path: P) -> Vec<Middleware> {
    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            error!("[Config] Reading File: {}", e);
            return Vec::new();
        }
    };

    parse_middlewares(&contents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty() {
        let content = "";
        assert_eq!(vec![] as Vec<Middleware>, parse_middlewares(content));
    }

    #[test]
    fn parse_remove_prefix() {
        let content = "
    middleware:
        - name: Test
          RemovePrefix: /api/
        ";
        assert_eq!(
            vec![Middleware::new(
                "Test",
                Action::RemovePrefix("/api/".to_owned())
            )],
            parse_middlewares(content)
        );
    }
    #[test]
    fn parse_add_header() {
        let content = "
    middleware:
        - name: Test
          AddHeader:
              - key: test-key
                value: test-value
        ";
        assert_eq!(
            vec![Middleware::new(
                "Test",
                Action::AddHeaders(vec![("test-key".to_owned(), "test-value".to_owned())])
            )],
            parse_middlewares(content)
        );
    }
    #[test]
    fn parse_cors() {
        let content = "
    middleware:
        - name: Test
          CORS:
              origins:
                  - http://localhost
              max_age: 123
              credentials: true
              methods:
                  - GET
                  - POST
              headers:
                  - Authorization
              ";
        assert_eq!(
            vec![Middleware::new(
                "Test",
                Action::Cors(CorsOpts {
                    origins: vec!["http://localhost".to_owned()],
                    max_age: Some(123),
                    credentials: true,
                    methods: vec!["GET".to_owned(), "POST".to_owned()],
                    headers: vec!["Authorization".to_owned()],
                })
            )],
            parse_middlewares(content)
        );
    }
}
