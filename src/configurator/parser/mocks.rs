use crate::rules::{Action, Rule, Service};

use async_trait::async_trait;
use rustls::sign::CertifiedKey;

use super::{ParseRuleContext, Parser};

pub struct MockParser {
    service_result: Option<Service>,
    action_result: Option<Action>,
    rule_result: Option<Rule>,
    tls_result: Option<(String, CertifiedKey)>,
}

impl MockParser {
    /// Creates a new MockParser that will always return a cloned version
    /// of the given Option<Action>
    pub fn new(
        service: Option<Service>,
        action: Option<Action>,
        rule: Option<Rule>,
        tls: Option<(String, CertifiedKey)>,
    ) -> Self {
        Self {
            service_result: service,
            action_result: action,
            rule_result: rule,
            tls_result: tls,
        }
    }
}

#[async_trait]
impl Parser for MockParser {
    async fn service(&self, _config: &serde_json::Value) -> Option<Service> {
        self.service_result.clone()
    }

    async fn parse_action(&self, _name: &str, _config: &serde_json::Value) -> Option<Action> {
        self.action_result.clone()
    }

    async fn rule<'a>(
        &self,
        _config: &serde_json::Value,
        _context: ParseRuleContext<'a>,
    ) -> Option<Rule> {
        self.rule_result.clone()
    }

    async fn tls(
        &self,
        _config: &serde_json::Value,
    ) -> Option<(String, rustls::sign::CertifiedKey)> {
        self.tls_result.clone()
    }
}
