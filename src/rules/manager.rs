use crate::rules::{rule_list, Rule};

use stream_httparse::Request;

use super::rule_list::RuleListWriteHandle;

/// A Wrapper around the RuleList-Reader that makes it
/// easier to find a matching Rule for a given Request
#[derive(Clone)]
pub struct ReadManager {
    rules: rule_list::RuleListReader,
}

/// Creates a new Read/Write Pair for the Rule-List
pub fn new() -> (ReadManager, RuleListWriteHandle) {
    let (writer, reader) = rule_list::new();

    (ReadManager { rules: reader }, writer)
}

impl ReadManager {
    /// Searches for a Rule that matches the given Request
    ///
    /// # Returns:
    /// * None if no Rule matches the Request
    /// * Some(rule) the first Rule that matched
    pub fn match_req(&self, req: &Request) -> Option<std::sync::Arc<Rule>> {
        self.rules.find(req)
    }
}
