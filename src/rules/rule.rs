use crate::general::Shared;
use crate::rules::{Matcher, Middleware, Service};

use stream_httparse::Request;

use super::MiddlewareList;

use crate::configurator::ConfigItem;

/// A Rule represents a single Routing-Rule, this consists
/// of a Matcher, Priority, Middlewares and a Service
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    name: String,
    priority: u32,
    matcher: Matcher,
    middlewares: Vec<Shared<Middleware>>,
    service: Shared<Service>,
    tls: Option<String>,
}

impl Rule {
    /// Creates a new Rule from the given Parameters
    pub fn new(
        name: String,
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
            tls: None,
        }
    }

    /// Enables TLS for the Rule
    pub fn set_tls(&mut self, name: String) {
        self.tls = Some(name);
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

    /// Returns the current TLS-Setting for the
    /// Rule
    pub fn tls(&self) -> Option<&String> {
        self.tls.as_ref()
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
        // TODO:
        // Fix this by moving it into the Matcher type, which can
        // then also check for the Host-Domain in a better way, for
        // example when there is an OR or AND
        if let Matcher::Domain(ref domain) = self.matcher {
            return Some(domain.to_owned());
        }
        None
    }
}

impl ConfigItem for Rule {
    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use stream_httparse::{Headers, Method};

    #[test]
    fn test_1_matches_valid() {
        let mut headers = Headers::new();
        headers.set("Host", "lol3r.net");
        let req = Request::new("HTTP/1.1", Method::GET, "/", headers, "".as_bytes());

        let rule = Rule::new(
            "test-rule".to_owned(),
            1,
            Matcher::Domain("lol3r.net".to_owned()),
            vec![],
            Shared::new(Service::new("test", vec!["test".to_owned()])),
        );

        assert_eq!(true, rule.matches(&req));
    }
    #[test]
    fn test_1_matches_invalid() {
        let mut headers = Headers::new();
        headers.set("Host", "lol3r.net");
        let req = Request::new("HTTP/1.1", Method::GET, "/", headers, "".as_bytes());

        let rule = Rule::new(
            "test-rule".to_owned(),
            1,
            Matcher::Domain("google.com".to_owned()),
            vec![],
            Shared::new(Service::new("test", vec!["test".to_owned()])),
        );

        assert_eq!(false, rule.matches(&req));
    }

    #[test]
    fn test_2_matches_valid() {
        let mut headers = Headers::new();
        headers.set("Host", "lol3r.net");
        let req = Request::new("HTTP/1.1", Method::GET, "/api/test", headers, "".as_bytes());

        let rule = Rule::new(
            "test-rule".to_owned(),
            1,
            Matcher::And(vec![
                Matcher::Domain("lol3r.net".to_owned()),
                Matcher::PathPrefix("/api/".to_owned()),
            ]),
            vec![],
            Shared::new(Service::new("test", vec!["test".to_owned()])),
        );

        assert_eq!(true, rule.matches(&req));
    }
    #[test]
    fn test_2_matches_invalid_1() {
        let mut headers = Headers::new();
        headers.set("Host", "lol3r.net");
        let req = Request::new("HTTP/1.1", Method::GET, "/api/test", headers, "".as_bytes());

        let rule = Rule::new(
            "test-rule".to_owned(),
            1,
            Matcher::And(vec![
                Matcher::Domain("google.com".to_owned()),
                Matcher::PathPrefix("/api/".to_owned()),
            ]),
            vec![],
            Shared::new(Service::new("test", vec!["test".to_owned()])),
        );

        assert_eq!(false, rule.matches(&req));
    }
    #[test]
    fn test_2_matches_invalid_2() {
        let mut headers = Headers::new();
        headers.set("Host", "lol3r.net");
        let req = Request::new("HTTP/1.1", Method::GET, "/api/test", headers, "".as_bytes());

        let rule = Rule::new(
            "test-rule".to_owned(),
            1,
            Matcher::And(vec![
                Matcher::Domain("lol3r.net".to_owned()),
                Matcher::PathPrefix("/other/".to_owned()),
            ]),
            vec![],
            Shared::new(Service::new("test", vec!["test".to_owned()])),
        );

        assert_eq!(false, rule.matches(&req));
    }
}
