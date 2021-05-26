use crate::configurator::files::Config;
use crate::configurator::parser;
use crate::configurator::ActionPluginList;
use crate::rules::Middleware;

use log::error;

use super::FileParser;

async fn parse_single_middleware(
    tmp_middle: &serde_json::Value,
    parser: &FileParser,
    action_plugins: &ActionPluginList,
) -> Option<Vec<Middleware>> {
    let name = tmp_middle.get("name")?;
    let name = name.as_str()?;

    let tmp_obj = tmp_middle.as_object()?;

    let mut result = Vec::with_capacity(tmp_obj.keys().len());
    for key in tmp_obj.keys() {
        if key == "name" {
            continue;
        }
        let value = tmp_obj.get(key).unwrap();

        if let Some(m) =
            parser::parse_middleware(name, key.as_str(), value, parser, action_plugins).await
        {
            result.push(m);
        }
    }

    Some(result)
}

async fn parse_middlewares(
    content: &str,
    parser: &FileParser,
    action_plugins: &ActionPluginList,
) -> Vec<Middleware> {
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
        if let Some(parsed) = parse_single_middleware(&tmp_middle, parser, action_plugins).await {
            result.extend(parsed);
        }
    }

    result
}

/// Loads all the Middlewares from the given File
pub async fn load_middlewares<P: AsRef<std::path::Path>>(
    path: P,
    parser: &FileParser,
    action_plugins: &ActionPluginList,
) -> Vec<Middleware> {
    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            error!("[Config] Reading File: {}", e);
            return Vec::new();
        }
    };

    parse_middlewares(&contents, parser, action_plugins).await
}

#[cfg(test)]
mod tests {
    use crate::rules::{Action, CorsOpts};

    use super::*;

    #[tokio::test]
    async fn parse_empty() {
        let content = "";
        assert_eq!(
            vec![] as Vec<Middleware>,
            parse_middlewares(content, &FileParser::new(), &ActionPluginList::default()).await
        );
    }

    #[tokio::test]
    async fn parse_remove_prefix() {
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
            parse_middlewares(content, &FileParser::new(), &ActionPluginList::default()).await
        );
    }
    #[tokio::test]
    async fn parse_add_header() {
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
            parse_middlewares(content, &FileParser::new(), &ActionPluginList::default()).await
        );
    }
    #[tokio::test]
    async fn parse_cors() {
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
            parse_middlewares(content, &FileParser::new(), &ActionPluginList::default()).await
        );
    }
}
