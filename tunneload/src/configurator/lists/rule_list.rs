use std::fmt::Debug;

use general::Name;
use rules::{rule_list::RuleListWriteHandle, Rule};

use lazy_static::lazy_static;
use prometheus::Registry;

lazy_static! {
    static ref CONFIG_RULES_COUNT: prometheus::IntGauge =
        prometheus::IntGauge::new("config_rules", "The Number of rules currently registered",)
            .expect("Creating a Metric should never fail");
}

/// The List that contains all the Rules for Routing incoming Requests.
/// This list is only used to update the Rules and not read from it.
#[derive(Clone)]
pub struct RuleList {
    writer: std::sync::Arc<std::sync::Mutex<RuleListWriteHandle>>,
}

impl RuleList {
    /// Creates a new RuleList from the given WriteHandle to
    /// the actual Left-Right based RuleList
    pub fn new(write_handle: RuleListWriteHandle) -> Self {
        Self {
            writer: std::sync::Arc::new(std::sync::Mutex::new(write_handle)),
        }
    }

    /// This registers all the Prometheus Metrics related to
    /// service configuration
    pub fn register_metrics(reg: &mut Registry) {
        if let Err(e) = reg.register(Box::new(CONFIG_RULES_COUNT.clone())) {
            tracing::error!("Registering Metric: {:?}", e);
        }
    }

    /// Sets/Updates the List with the given Rule
    pub fn set_rule(&self, n_srv: Rule) {
        let mut writer = self.writer.lock().expect("Locking Writer");

        CONFIG_RULES_COUNT.set(writer.set_single(n_srv) as i64);
    }

    /// Removes the Rules matching the given Name
    pub fn remove_rule(&self, name: Name) {
        let mut writer = self.writer.lock().expect("Locking Writer");

        CONFIG_RULES_COUNT.set(writer.remove(name) as i64);
    }
}

impl Debug for RuleList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use stream_httparse::{Headers, Request};

    use general::{Group, Shared};
    use rules::{self, Action, Matcher, Middleware, Service};

    use super::*;

    #[test]
    fn set_rule() {
        let (read, write) = rules::new();

        let tmp_rule_list = RuleList::new(write);

        tmp_rule_list.set_rule(Rule::new(
            Name::new("test-name", Group::Internal),
            1,
            Matcher::PathPrefix("/".to_owned()),
            vec![Shared::new(Middleware::new(
                Name::new("test-middleware", Group::Internal),
                Action::Noop,
            ))],
            Shared::new(Service::new(
                Name::new("test-service", Group::Internal),
                vec![],
            )),
        ));

        let tmp_req = Request::new(
            "HTTP/1.1",
            stream_httparse::Method::GET,
            "/path",
            Headers::new(),
            &[],
        );
        let matched_res = read.match_req(&tmp_req);
        assert_eq!(true, matched_res.is_some());
    }
}
