use crate::{Matcher, Middleware, Service};

use general::{Name, Shared};
use general_traits::ConfigItem;

use serde::Serialize;
use stream_httparse::Request;

use super::MiddlewareList;

/// The TLS-Value for a single Rule
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum RuleTLS {
    /// There is no TLS configured
    None,
    /// TLS is configured using a K8s-Secret
    Secret(String),
    /// TLS is configured using a generated Certificate
    Generate(String),
}

/// A Rule represents a single Routing-Rule, this consists
/// of a Matcher, Priority, Middlewares and a Service
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Rule {
    name: Name,
    priority: u32,
    matcher: Matcher,
    middlewares: Vec<Shared<Middleware>>,
    service: Shared<Service>,
    tls: RuleTLS,
}

impl Rule {
    /// Creates a new Rule from the given Parameters
    pub fn new(
        name: Name,
        priority: u32,
        matcher: Matcher,
        middlewares: Vec<Shared<Middleware>>,
        service: Shared<Service>,
    ) -> Self {
        Self {
            name,
            priority,
            matcher,
            middlewares,
            service,
            tls: RuleTLS::None,
        }
    }

    /// Enables TLS for the Rule
    pub fn set_tls(&mut self, val: RuleTLS) {
        self.tls = val;
    }

    /// Returns the Priority of the Rule
    pub fn priority(&self) -> u32 {
        self.priority
    }
    /// Returns the Service that all the Requests
    /// should be forwarded to
    pub fn service(&self) -> std::sync::Arc<Service> {
        self.service.get()
    }

    /// Returns the Matcher for the Rule
    pub fn matcher(&self) -> &Matcher {
        &self.matcher
    }

    /// Returns the current TLS-Setting for the
    /// Rule
    pub fn tls(&self) -> &RuleTLS {
        &self.tls
    }

    /// Checks if the Rule matches for the given Request
    pub fn matches(&self, req: &Request) -> bool {
        self.matcher.matches(req)
    }

    /// Returns the Rule's Middleware List
    pub fn get_middleware_list(&self) -> MiddlewareList {
        MiddlewareList::from(&self.middlewares[..])
    }

    /// Returns the Domain specified for this Rule, if
    /// there is one
    pub fn get_host(&self) -> Option<String> {
        self.matcher.get_host()
    }
}

impl ConfigItem for Rule {
    fn name(&self) -> &Name {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use general::Group;
    use stream_httparse::{Headers, Method};

    #[test]
    fn test_1_matches_valid() {
        let mut headers = Headers::new();
        headers.set("Host", "lol3r.net");
        let req = Request::new("HTTP/1.1", Method::GET, "/", headers, "".as_bytes());

        let rule = Rule::new(
            Name::new("test-rule", Group::Internal),
            1,
            Matcher::Domain("lol3r.net".to_owned()),
            vec![],
            Shared::new(Service::new(
                Name::new("test", Group::Internal),
                vec!["test".to_owned()],
            )),
        );

        assert_eq!(true, rule.matches(&req));
    }
    #[test]
    fn test_1_matches_invalid() {
        let mut headers = Headers::new();
        headers.set("Host", "lol3r.net");
        let req = Request::new("HTTP/1.1", Method::GET, "/", headers, "".as_bytes());

        let rule = Rule::new(
            Name::new("test-rule", Group::Internal),
            1,
            Matcher::Domain("google.com".to_owned()),
            vec![],
            Shared::new(Service::new(
                Name::new("test", Group::Internal),
                vec!["test".to_owned()],
            )),
        );

        assert_eq!(false, rule.matches(&req));
    }

    #[test]
    fn test_2_matches_valid() {
        let mut headers = Headers::new();
        headers.set("Host", "lol3r.net");
        let req = Request::new("HTTP/1.1", Method::GET, "/api/test", headers, "".as_bytes());

        let rule = Rule::new(
            Name::new("test-rule", Group::Internal),
            1,
            Matcher::And(vec![
                Matcher::Domain("lol3r.net".to_owned()),
                Matcher::PathPrefix("/api/".to_owned()),
            ]),
            vec![],
            Shared::new(Service::new(
                Name::new("test", Group::Internal),
                vec!["test".to_owned()],
            )),
        );

        assert_eq!(true, rule.matches(&req));
    }
    #[test]
    fn test_2_matches_invalid_1() {
        let mut headers = Headers::new();
        headers.set("Host", "lol3r.net");
        let req = Request::new("HTTP/1.1", Method::GET, "/api/test", headers, "".as_bytes());

        let rule = Rule::new(
            Name::new("test-rule", Group::Internal),
            1,
            Matcher::And(vec![
                Matcher::Domain("google.com".to_owned()),
                Matcher::PathPrefix("/api/".to_owned()),
            ]),
            vec![],
            Shared::new(Service::new(
                Name::new("test", Group::Internal),
                vec!["test".to_owned()],
            )),
        );

        assert_eq!(false, rule.matches(&req));
    }
    #[test]
    fn test_2_matches_invalid_2() {
        let mut headers = Headers::new();
        headers.set("Host", "lol3r.net");
        let req = Request::new("HTTP/1.1", Method::GET, "/api/test", headers, "".as_bytes());

        let rule = Rule::new(
            Name::new("test-rule", Group::Internal),
            1,
            Matcher::And(vec![
                Matcher::Domain("lol3r.net".to_owned()),
                Matcher::PathPrefix("/other/".to_owned()),
            ]),
            vec![],
            Shared::new(Service::new(
                Name::new("test", Group::Internal),
                vec!["test".to_owned()],
            )),
        );

        assert_eq!(false, rule.matches(&req));
    }
}
