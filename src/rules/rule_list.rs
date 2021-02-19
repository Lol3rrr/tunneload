use crate::rules::Rule;

#[cfg(test)]
use crate::rules::{Matcher, Service};

use left_right::{Absorb, ReadHandle, WriteHandle};

// The OP-Log type
enum ListOp {
    Add(Rule),
    Sort,
    Clear,
}

impl Absorb<ListOp> for Vec<Rule> {
    fn absorb_first(&mut self, operation: &mut ListOp, _: &Self) {
        match operation {
            ListOp::Add(n_rule) => {
                self.push(n_rule.clone());
            }
            ListOp::Sort => {
                self.sort_by(|a, b| b.priority().cmp(&a.priority()));
            }
            ListOp::Clear => {
                self.clear();
            }
        };
    }
    fn absorb_second(&mut self, operation: ListOp, _: &Self) {
        match operation {
            ListOp::Add(n_rule) => {
                self.push(n_rule);
            }
            ListOp::Sort => {
                self.sort_by(|a, b| b.priority().cmp(&a.priority()));
            }
            ListOp::Clear => {
                self.clear();
            }
        };
    }

    fn drop_first(self: Box<Self>) {}

    fn sync_with(&mut self, first: &Self) {
        *self = first.clone();
    }
}

pub struct RuleListWriteHandle(WriteHandle<Vec<Rule>, ListOp>);
impl RuleListWriteHandle {
    pub fn add_single(&mut self, n_rule: Rule) {
        self.0.append(ListOp::Add(n_rule));
        self.0.append(ListOp::Sort);
        self.0.publish();
    }
    pub fn add_slice(&mut self, rules: Vec<Rule>) {
        for tmp in rules {
            self.0.append(ListOp::Add(tmp));
        }
        self.0.append(ListOp::Sort);
        self.0.publish();
    }
    pub fn clear(&mut self) {
        self.0.append(ListOp::Clear);
    }

    pub fn publish(&mut self) {
        self.0.publish();
    }
}

pub struct RuleListReader(ReadHandle<Vec<Rule>>);
impl RuleListReader {
    pub fn get(&self) -> left_right::ReadGuard<'_, Vec<Rule>> {
        self.0.enter().unwrap()
    }
}
unsafe impl Send for RuleListReader {}
unsafe impl Sync for RuleListReader {}
impl Clone for RuleListReader {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// Creates a new Write/Read pair
pub fn new() -> (RuleListWriteHandle, RuleListReader) {
    let (write, read) = left_right::new::<Vec<Rule>, ListOp>();

    (RuleListWriteHandle(write), RuleListReader(read))
}

#[test]
fn hold_read_while_updating() {
    let (mut write, read) = new();

    let pre_update = read.get();
    assert_eq!(Vec::<Rule>::new(), *pre_update);

    let n_rule = Rule::new(
        "test".to_owned(),
        3,
        Matcher::Domain("example.net".to_owned()),
        Vec::new(),
        Service::new(vec!["localhost".to_owned()]),
    );
    write.add_single(n_rule.clone());

    // Still the older stuff
    assert_eq!(Vec::<Rule>::new(), *pre_update);
    // Newer stuff
    assert_eq!(vec![n_rule], *read.get());
}
