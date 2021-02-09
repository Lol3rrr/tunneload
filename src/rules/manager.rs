use crate::http::Request;
use crate::rules::Rule;

#[cfg(test)]
use crate::rules::Matcher;

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
        rules.sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    fn find_match<'a>(rules: &'a Vec<Rule>, req: &Request) -> Option<&'a Rule> {
        for rule in rules.iter() {
            if rule.matches(req) {
                return Some(rule);
            }
        }

        None
    }

    pub fn match_req(&self, req: &Request) -> Option<Rule> {
        let rules = self.rules.read().unwrap();
        let matched = match Manager::find_match(&rules, req) {
            Some(s) => s,
            None => {
                return None;
            }
        };

        Some(matched.clone())
    }
}

#[test]
fn add_rule() {
    let rule_1 = Rule::new(
        1,
        vec![Matcher::Domain("test".to_owned())],
        vec![],
        Service::new("testDest".to_owned()),
    );
    let rule_2 = Rule::new(
        4,
        vec![Matcher::Domain("test2".to_owned())],
        vec![],
        Service::new("testDest".to_owned()),
    );
    let rule_3 = Rule::new(
        1,
        vec![Matcher::Domain("test3".to_owned())],
        vec![],
        Service::new("testDest".to_owned()),
    );

    let manager = Manager::new();
    manager.add_rule(rule_1.clone());

    let internal_rules = manager.rules.read().unwrap();
    assert_eq!(vec![rule_1.clone()], *internal_rules);
    drop(internal_rules);

    manager.add_rule(rule_2.clone());

    let internal_rules = manager.rules.read().unwrap();
    assert_eq!(vec![rule_2.clone(), rule_1.clone()], *internal_rules);
    drop(internal_rules);

    manager.add_rule(rule_3.clone());

    let internal_rules = manager.rules.read().unwrap();
    assert_eq!(
        vec![rule_2.clone(), rule_1.clone(), rule_3.clone()],
        *internal_rules
    );
    drop(internal_rules);
}
