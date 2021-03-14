use crate::rules::{rule_list, Rule};

use stream_httparse::Request;

use super::rule_list::RuleListWriteHandle;

#[derive(Clone)]
pub struct ReadManager {
    rules: rule_list::RuleListReader,
}

pub fn new() -> (ReadManager, RuleListWriteHandle) {
    let (writer, reader) = rule_list::new();

    (ReadManager { rules: reader }, writer)
}

impl ReadManager {
    pub fn match_req(&self, req: &Request) -> Option<std::sync::Arc<Rule>> {
        self.rules.find(req)
    }
}
