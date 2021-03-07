use crate::configurator::kubernetes::traefik_bindings::middleware;
use crate::rules::{Action, Middleware};

use log::error;

mod basic_auth;
mod headers;
mod strip_prefix;

pub async fn parse_middleware(
    client: Option<kube::Client>,
    namespace: Option<&str>,
    raw_mid: middleware::Config,
) -> Vec<Middleware> {
    let mut result = Vec::new();

    let name = raw_mid.metadata.name;

    for (key, value) in raw_mid.spec.iter() {
        match key.as_str() {
            "stripPrefix" => {
                if let Some(n_middleware) = strip_prefix::parse(&name, value) {
                    result.push(n_middleware);
                }
            }
            "headers" => {
                if let Some(n_middleware) = headers::parse(&name, value) {
                    result.push(n_middleware);
                }
            }
            "compress" => {
                result.push(Middleware::new(&name, Action::Compress));
            }
            "basicAuth" => {
                let kube_client = client.as_ref().unwrap();
                let kube_namespace = namespace.unwrap();

                if let Some(n_middleware) =
                    basic_auth::parse(&name, value, kube_client.clone(), &kube_namespace).await
                {
                    result.push(n_middleware);
                }
            }
            _ => {
                error!("Unknown: '{:?}': '{:?}'", key, value);
            }
        };
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::configurator::kubernetes::general_crd;
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
            parse_middleware(None, None, config).await
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
            parse_middleware(None, None, config).await
        );
    }
}
