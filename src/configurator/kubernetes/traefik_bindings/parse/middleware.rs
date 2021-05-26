use crate::configurator::kubernetes::traefik_bindings::{middleware, TraefikParser};
use crate::configurator::{parser, ActionPluginList};
use crate::rules::Middleware;

/// Parses the raw-middleware into a List of Middlewares that can
/// then be used as configurations
pub async fn parse_middleware(
    parser: &TraefikParser,
    raw_mid: middleware::Config,
    action_plugins: &ActionPluginList,
) -> Vec<Middleware> {
    let mut result = Vec::new();

    let name = raw_mid.metadata.name;
    for (key, value) in raw_mid.spec.iter() {
        if let Some(tmp) = parser::parse_middleware(&name, key, value, parser, action_plugins).await
        {
            result.push(tmp);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{configurator::kubernetes::general_crd, rules::Action};
    use serde_json::json;

    #[tokio::test]
    async fn parse_middleware_stripprefix() {
        let mut spec = std::collections::BTreeMap::new();
        spec.insert(
            "stripPrefix".to_owned(),
            json!({
                "prefixes": [
                    "/api"
                ],
            }),
        );

        let config = middleware::Config {
            api_version: "v1".to_owned(),
            kind: "middleware".to_owned(),
            metadata: general_crd::Metadata {
                name: "test".to_owned(),
                namespace: "default".to_owned(),
            },
            spec,
        };

        assert_eq!(
            vec![Middleware::new(
                "test",
                Action::RemovePrefix("/api".to_owned())
            )],
            parse_middleware(
                &TraefikParser::default(),
                config,
                &ActionPluginList::default()
            )
            .await
        );
    }

    #[tokio::test]
    async fn parse_middleware_add_headers() {
        let mut spec = std::collections::BTreeMap::new();
        spec.insert(
            "headers".to_owned(),
            json!({
                "test-header": [
                    "test-value",
                ],
            }),
        );

        let config = middleware::Config {
            api_version: "v1".to_owned(),
            kind: "middleware".to_owned(),
            metadata: general_crd::Metadata {
                name: "test".to_owned(),
                namespace: "default".to_owned(),
            },
            spec,
        };

        assert_eq!(
            vec![Middleware::new(
                "test",
                Action::AddHeaders(vec![("test-header".to_owned(), "test-value".to_owned())]),
            )],
            parse_middleware(
                &TraefikParser::default(),
                config,
                &ActionPluginList::default()
            )
            .await
        );
    }
}
