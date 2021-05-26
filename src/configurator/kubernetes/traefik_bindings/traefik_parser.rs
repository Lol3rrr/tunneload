use async_trait::async_trait;

use crate::{configurator::parser::Parser, rules::Action};

mod action;

/// This is the Parser for all the Traefik related Parts
#[derive(Clone)]
pub struct TraefikParser {
    client: Option<kube::Client>,
    namespace: Option<String>,
}

impl TraefikParser {
    /// Creates a new Instance of the Parser
    pub fn new(client: Option<kube::Client>, namespace: Option<String>) -> Self {
        Self { client, namespace }
    }
}

impl Default for TraefikParser {
    fn default() -> Self {
        Self {
            client: None,
            namespace: None,
        }
    }
}

#[async_trait]
impl Parser for TraefikParser {
    async fn parse_action(&self, name: &str, config: &serde_json::Value) -> Option<Action> {
        match name {
            "stripPrefix" => action::strip_prefix(config),
            "headers" => action::headers(config),
            "compress" => Some(Action::Compress),
            "basicAuth" => {
                action::basic_auth(
                    config,
                    self.client.clone().unwrap(),
                    self.namespace.as_ref().unwrap(),
                )
                .await
            }
            _ => None,
        }
    }
}
