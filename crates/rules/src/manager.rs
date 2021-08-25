use crate::{rule_list, Rule};

use std::sync::Arc;

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
    pub fn match_req(&self, req: &Request) -> Option<Arc<Rule>> {
        self.rules.find(req)
    }

    /// Attempts to load all the currently visible Rules
    /// in the Load-Balancer
    pub fn get_all_rules(&self) -> Option<Vec<Arc<Rule>>> {
        self.rules.clone_all_rules()
    }
}
