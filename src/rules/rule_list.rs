//! Contains the Datatypes for the Left-Right inspired List of Rules

use crate::{configurator::ConfigItem, rules::Rule};

use stream_httparse::Request;

use left_right::{Absorb, ReadHandle, WriteHandle};
use std::sync::Arc;

// The OP-Log type
enum ListOp {
    Add(Rule),
    Set(Rule),
    Remove(String),
    Sort,
    Clear,
}

impl Absorb<ListOp> for Vec<Arc<Rule>> {
    fn absorb_first(&mut self, operation: &mut ListOp, _: &Self) {
        match operation {
            ListOp::Add(n_rule) => {
                self.push(Arc::new(n_rule.clone()));
            }
            ListOp::Set(n_rule) => {
                if let Some(index) = self.iter().position(|x| x.name() == n_rule.name()) {
                    self.remove(index);
                }

                self.push(Arc::new(n_rule.clone()));
            }
            ListOp::Remove(name) => {
                if let Some(index) = self.iter().position(|x| x.name() == name) {
                    self.remove(index);
                }
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
                self.push(Arc::new(n_rule));
            }
            ListOp::Set(n_rule) => {
                if let Some(index) = self.iter().position(|x| x.name() == n_rule.name()) {
                    self.remove(index);
                }

                self.push(Arc::new(n_rule));
            }
            ListOp::Remove(name) => {
                if let Some(index) = self.iter().position(|x| x.name() == name) {
                    self.remove(index);
                }
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

/// The Write-Half of the Rule-List, this is used to perform
/// any sort of Updates needed on the List
pub struct RuleListWriteHandle(WriteHandle<Vec<Arc<Rule>>, ListOp>);
impl RuleListWriteHandle {
    /// This function also sorts the list and then
    /// publishes the result
    pub fn add_single(&mut self, n_rule: Rule) {
        self.0.append(ListOp::Add(n_rule));
        self.0.append(ListOp::Sort);
        self.0.publish();
    }
    /// This function also sorts the list and then
    /// publishes the result
    pub fn add_slice(&mut self, rules: Vec<Rule>) {
        for tmp in rules {
            self.0.append(ListOp::Add(tmp));
        }
        self.0.append(ListOp::Sort);
        self.0.publish();
    }
    /// Overwrites the Rule if it has already been
    /// set before, otherwise simply adds it to
    /// the List as well
    ///
    /// This function also sorts the list and then
    /// publishes the result
    pub fn set_single(&mut self, n_rule: Rule) -> usize {
        self.0.append(ListOp::Set(n_rule));
        self.0.append(ListOp::Sort);
        self.0.publish();

        match self.0.enter() {
            Some(guard) => guard.len(),
            None => 0,
        }
    }
    /// Removes the first Rule that matches the given
    /// Name
    ///
    /// This function also publishes the result
    pub fn remove(&mut self, name: String) -> usize {
        self.0.append(ListOp::Remove(name));
        self.0.publish();

        match self.0.enter() {
            Some(guard) => guard.len(),
            None => 0,
        }
    }

    /// Clears the List of Rules
    pub fn clear(&mut self) {
        self.0.append(ListOp::Clear);
    }

    /// Publishes all the latest Changes, which makes them
    /// visible for all the Readers upon the next Read attempt
    pub fn publish(&mut self) {
        self.0.publish();
    }
}

/// The Reader Part for the List of Rules
pub struct RuleListReader(ReadHandle<Vec<Arc<Rule>>>);
impl RuleListReader {
    /// Attempts to find a Rule that matches against the given
    /// Request. If multiple Rules match against the Request
    /// the one with the highest Priority will be returned
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

    pub fn clone_all_rules(&self) -> Option<Vec<Arc<Rule>>> {
        self.0
            .enter()
            .map(|rules| Some(rules.clone()))
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
