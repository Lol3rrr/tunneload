use crate::http::Request;
use crate::rules::{rule_list, Rule};

#[derive(Clone)]
pub struct ReadManager {
    rules: rule_list::RuleListReader,
}

pub fn new() -> (ReadManager, WriteManager) {
    let (writer, reader) = rule_list::new();

    (
        ReadManager { rules: reader },
        WriteManager {
            rules: std::sync::Mutex::new(writer),
        },
    )
}

impl ReadManager {
    fn find_match<'a>(rules: &'a [Rule], req: &Request) -> Option<&'a Rule> {
        for rule in rules.iter() {
            if rule.matches(req) {
                return Some(rule);
            }
        }

        None
    }

    pub fn match_req(&self, req: &Request) -> Option<Rule> {
        let rules = match self.rules.get() {
            Some(r) => r,
            None => {
                return None;
            }
        };

        let matched = match ReadManager::find_match(&rules, req) {
            Some(s) => s,
            None => {
                return None;
            }
        };

        Some(matched.clone())
    }
}

pub struct WriteManager {
    rules: std::sync::Mutex<rule_list::RuleListWriteHandle>,
}

impl WriteManager {
    pub fn add_rule(&self, n_rule: Rule) {
        let mut rules = self.rules.lock().unwrap();
        rules.add_single(n_rule);
    }
    pub fn add_rules(&self, n_rules: Vec<Rule>) {
        let mut rules = self.rules.lock().unwrap();
        rules.add_slice(n_rules);
    }

    pub fn publish(&self) {
        let mut rules = self.rules.lock().unwrap();
        rules.publish();
    }

    pub fn clear_rules(&self) {
        let mut rules = self.rules.lock().unwrap();
        rules.clear();
    }
}
