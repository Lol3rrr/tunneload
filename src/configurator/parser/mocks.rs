use crate::rules::Action;

use async_trait::async_trait;

use super::Parser;

pub struct MockParser {
    action_result: Option<Action>,
}

impl MockParser {
    /// Creates a new MockParser that will always return a cloned version
    /// of the given Option<Action>
    pub fn new(result: Option<Action>) -> Self {
        Self {
            action_result: result,
        }
    }
}

#[async_trait]
impl Parser for MockParser {
    async fn parse_action(&self, _name: &str, _config: &serde_json::Value) -> Option<Action> {
        self.action_result.clone()
    }
}
