use crate::rules::{Matcher, Middleware, Service};
use crate::{general::Shared, http::Request};

use super::MiddlewareList;

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

    pub fn set_tls(&mut self, name: String) {
        self.tls = Some(name);
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn priority(&self) -> u32 {
        self.priority
    }
    pub fn service(&self) -> std::sync::Arc<Service> {
        self.service.get()
    }

    pub fn tls(&self) -> Option<&String> {
        self.tls.as_ref()
    }

    pub fn matches(&self, req: &Request) -> bool {
        self.matcher.matches(req)
    }

    pub fn get_middleware_list(&self) -> MiddlewareList {
        MiddlewareList::from(&self.middlewares[..])
    }

    pub fn get_host(&self) -> Option<String> {
        if let Matcher::Domain(ref domain) = self.matcher {
            return Some(domain.to_owned());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::http::{Headers, Method};

    #[test]
    fn test_1_matches_valid() {
        let mut headers = Headers::new();
        headers.add("Host", "lol3r.net");
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
        headers.add("Host", "lol3r.net");
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
        headers.add("Host", "lol3r.net");
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
        headers.add("Host", "lol3r.net");
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
        headers.add("Host", "lol3r.net");
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
