use crate::http::Request;
use crate::rules::{Rule, Service};

#[derive(Clone, Debug)]
pub struct Manager {
    rules: std::sync::Arc<std::sync::RwLock<Vec<Rule>>>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            rules: std::sync::Arc::new(std::sync::RwLock::new(Vec::new())),
        }
    }

    pub fn add_rule(&self, n_rule: Rule) {
        let mut rules = self.rules.write().unwrap();
        rules.push(n_rule);
    }

    pub fn match_req(&self, req: &Request) -> Option<Service> {
        let rules = self.rules.read().unwrap();

        for rule in rules.iter() {
            if let Some(service) = rule.matches(req) {
                return Some(service);
            }
        }

        None
    }
}
