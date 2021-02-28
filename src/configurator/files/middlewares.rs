use crate::configurator::files::Config;
use crate::rules::{Action, Middleware};

use log::error;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ConfigMiddleware {
    name: String,
    #[serde(rename = "RemovePrefix")]
    remove_prefix: Option<String>,
    #[serde(rename = "AddHeader")]
    add_header: Option<Vec<AddHeaderConfig>>,
}

#[derive(Debug, Deserialize)]
pub struct AddHeaderConfig {
    key: String,
    value: String,
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
        if tmp_middle.remove_prefix.is_some() {
            result.push(Middleware::new(
                &name,
                Action::RemovePrefix(tmp_middle.remove_prefix.unwrap()),
            ));
            continue;
        }

        if tmp_middle.add_header.is_some() {
            let add_headers = tmp_middle.add_header.unwrap();
            let mut tmp_headers = Vec::<(String, String)>::new();
            for header in add_headers {
                tmp_headers.push((header.key, header.value));
            }

            result.push(Middleware::new(&name, Action::AddHeaders(tmp_headers)));
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
