use crate::rules::{rule_list::RuleListWriteHandle, Rule};

use lazy_static::lazy_static;
use prometheus::Registry;

lazy_static! {
    static ref CONFIG_RULES_COUNT: prometheus::IntGauge =
        prometheus::IntGauge::new("config_rules", "The Number of rules currently registered",)
            .unwrap();
}

#[derive(Clone)]
pub struct RuleList {
    writer: std::sync::Arc<std::sync::Mutex<RuleListWriteHandle>>,
}

impl RuleList {
    pub fn new(write_handle: RuleListWriteHandle) -> Self {
        Self {
            writer: std::sync::Arc::new(std::sync::Mutex::new(write_handle)),
        }
    }

    /// This registers all the Prometheus Metrics related to
    /// service configuration
    pub fn register_metrics(reg: &mut Registry) {
        reg.register(Box::new(CONFIG_RULES_COUNT.clone())).unwrap();
    }

    pub fn set_rule(&self, n_srv: Rule) {
        let mut writer = self.writer.lock().unwrap();

        CONFIG_RULES_COUNT.set(writer.set_single(n_srv) as i64);
    }

    pub fn clone_vec(&self) -> Vec<std::sync::Arc<Rule>> {
        let mut writer = self.writer.lock().unwrap();

        writer.clone_vec()
    }
}
