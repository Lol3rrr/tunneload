use crate::rules::{Action, Middleware};

use async_trait::async_trait;

use super::ActionPluginList;

#[cfg(test)]
pub mod mocks;

#[async_trait]
pub trait Parser {
    /// Parses the given Action
    ///
    /// # Params:
    /// * `name`: The Name of the Action
    /// * `config`: The Config that belongs to the Action
    async fn parse_action(&self, name: &str, config: &serde_json::Value) -> Option<Action>;
}

/// # Params:
/// * `name`: The Name of the Configured Middleware
/// * `action_name`: The Name of the Middleware/Action to use
/// * `config`: The Configuration to use for the Middleware/Action
pub async fn parse_middleware<P>(
    name: &str,
    action_name: &str,
    config: &serde_json::Value,
    parser: &P,
    action_plugins: &ActionPluginList,
) -> Option<Middleware>
where
    P: Parser,
{
    let action = if action_name.contains('@') {
        let (name, group) = action_name.split_once('@')?;

        match group {
            "plugin" => {
                let plugin = action_plugins.get(name)?;

                let config_str = serde_json::to_string(config).unwrap();
                let instance = plugin.get().create_instance(config_str)?;

                Action::Plugin(instance)
            }
            _ => return None,
        }
    } else {
        parser.parse_action(action_name, config).await?
    };

    Some(Middleware::new(name, action))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::mocks::MockParser;
    use super::*;

    #[tokio::test]
    async fn normal_action() {
        assert_eq!(
            Some(Middleware::new("test", Action::Compress),),
            parse_middleware(
                "test",
                "compress",
                &json!({}),
                &MockParser::new(Some(Action::Compress)),
                &ActionPluginList::new()
            )
            .await
        );
    }

    #[tokio::test]
    async fn attempt_to_load_plugin() {
        assert_eq!(
            None,
            parse_middleware(
                "test",
                "testplug@plugin",
                &json!({}),
                &MockParser::new(Some(Action::Compress)),
                &ActionPluginList::new()
            )
            .await
        );
    }
}
