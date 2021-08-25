use std::{error::Error, fmt::Display};

use rules::{Action, Rule, Service};

use async_trait::async_trait;
use rustls::sign::CertifiedKey;

use super::{ParseRuleContext, Parser};

#[derive(Debug, Clone)]
pub struct MockError {}

impl Display for MockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mock-Error")
    }
}
impl Error for MockError {}

pub struct MockParser<S, A, R, T> {
    service_result: Result<Service, S>,
    action_result: Result<Action, A>,
    rule_result: Result<Rule, R>,
    tls_result: Result<(String, CertifiedKey), T>,
}

impl<S, A, R, T> MockParser<S, A, R, T> {
    /// Creates a new MockParser that will always return a cloned version
    /// of the given Option<Action>
    pub fn new(
        service: Result<Service, S>,
        action: Result<Action, A>,
        rule: Result<Rule, R>,
        tls: Result<(String, CertifiedKey), T>,
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
impl<S, A, R, T> Parser for MockParser<S, A, R, T>
where
    S: Error + Clone + Send + Sync + 'static,
    A: Error + Clone + Send + Sync + 'static,
    R: Error + Clone + Send + Sync + 'static,
    T: Error + Clone + Send + Sync + 'static,
{
    async fn service(&self, _config: &serde_json::Value) -> Result<Service, Box<dyn Error>> {
        match self.service_result.clone() {
            Ok(k) => Ok(k),
            Err(e) => Err(Box::new(e)),
        }
    }

    async fn parse_action(
        &self,
        _name: &str,
        _config: &serde_json::Value,
    ) -> Result<Action, Box<dyn Error>> {
        match self.action_result.clone() {
            Ok(k) => Ok(k),
            Err(e) => Err(Box::new(e)),
        }
    }

    async fn rule<'a>(
        &self,
        _config: &serde_json::Value,
        _context: ParseRuleContext<'a>,
    ) -> Result<Rule, Box<dyn Error>> {
        match self.rule_result.clone() {
            Ok(k) => Ok(k),
            Err(e) => Err(Box::new(e)),
        }
    }

    async fn tls(
        &self,
        _config: &serde_json::Value,
    ) -> Result<(String, rustls::sign::CertifiedKey), Box<dyn Error>> {
        match self.tls_result.clone() {
            Ok(k) => Ok(k),
            Err(e) => Err(Box::new(e)),
        }
    }
}
