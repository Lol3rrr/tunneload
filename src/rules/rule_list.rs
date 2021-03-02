use crate::{http::Request, rules::Rule};

use left_right::{Absorb, ReadHandle, WriteHandle};
use std::sync::Arc;

// The OP-Log type
enum ListOp {
    Add(Rule),
    Sort,
    Clear,
}

impl Absorb<ListOp> for Vec<Arc<Rule>> {
    fn absorb_first(&mut self, operation: &mut ListOp, _: &Self) {
        match operation {
            ListOp::Add(n_rule) => {
                self.push(Arc::new(n_rule.clone()));
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
                self.push(Arc::new(n_rule.clone()));
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

pub struct RuleListWriteHandle(WriteHandle<Vec<Arc<Rule>>, ListOp>);
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

pub struct RuleListReader(ReadHandle<Vec<Arc<Rule>>>);
impl RuleListReader {
    pub fn find(&self, req: &Request) -> Option<Arc<Rule>> {
        self.0
            .enter()
            .map(|rules| {
                for rule in rules.iter() {
                    if rule.matches(req) {
                        return Some(rule.clone());
                    }
                }
                None
            })
            .unwrap_or(None)
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
    let (write, read) = left_right::new::<Vec<Arc<Rule>>, ListOp>();

    (RuleListWriteHandle(write), RuleListReader(read))
}
