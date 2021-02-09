use crate::http::Request;
use crate::rules::{Matcher, Middleware, Service};

#[cfg(test)]
use crate::http::{Header, Method};

#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    priority: u32,
    matcher: Vec<Matcher>,
    middlewares: Vec<Middleware>,
    service: Service,
}

impl Rule {
    pub fn new(
        priority: u32,
        matcher: Vec<Matcher>,
        middlewares: Vec<Middleware>,
        service: Service,
    ) -> Self {
        Self {
            priority,
            matcher,
            middlewares,
            service,
        }
    }

    pub fn priority(&self) -> u32 {
        self.priority
    }
    pub fn service(&self) -> &Service {
        &self.service
    }

    pub fn matches(&self, req: &Request) -> bool {
        for tmp_matcher in self.matcher.iter() {
            if !tmp_matcher.matches(req) {
                return false;
            }
        }

        true
    }

    pub fn apply_middlewares(&self, req: &mut Request) {
        for middleware in self.middlewares.iter() {
            middleware.apply(req);
        }
    }
}

#[test]
fn test_1_matches_valid() {
    let req = Request::new(
        "HTTP/1.1",
        Method::GET,
        "/",
        vec![Header::new("Host", "lol3r.net")],
        "".as_bytes(),
    );

    let rule = Rule::new(
        1,
        vec![Matcher::Domain("lol3r.net".to_owned())],
        vec![],
        Service::new("test".to_owned()),
    );

    assert_eq!(true, rule.matches(&req));
}
#[test]
fn test_1_matches_invalid() {
    let req = Request::new(
        "HTTP/1.1",
        Method::GET,
        "/",
        vec![Header::new("Host", "lol3r.net")],
        "".as_bytes(),
    );

    let rule = Rule::new(
        1,
        vec![Matcher::Domain("google.com".to_owned())],
        vec![],
        Service::new("test".to_owned()),
    );

    assert_eq!(false, rule.matches(&req));
}

#[test]
fn test_2_matches_valid() {
    let req = Request::new(
        "HTTP/1.1",
        Method::GET,
        "/api/test",
        vec![Header::new("Host", "lol3r.net")],
        "".as_bytes(),
    );

    let rule = Rule::new(
        1,
        vec![
            Matcher::Domain("lol3r.net".to_owned()),
            Matcher::PathPrefix("/api/".to_owned()),
        ],
        vec![],
        Service::new("test".to_owned()),
    );

    assert_eq!(true, rule.matches(&req));
}
#[test]
fn test_2_matches_invalid_1() {
    let req = Request::new(
        "HTTP/1.1",
        Method::GET,
        "/api/test",
        vec![Header::new("Host", "lol3r.net")],
        "".as_bytes(),
    );

    let rule = Rule::new(
        1,
        vec![
            Matcher::Domain("google.com".to_owned()),
            Matcher::PathPrefix("/api/".to_owned()),
        ],
        vec![],
        Service::new("test".to_owned()),
    );

    assert_eq!(false, rule.matches(&req));
}
#[test]
fn test_2_matches_invalid_2() {
    let req = Request::new(
        "HTTP/1.1",
        Method::GET,
        "/api/test",
        vec![Header::new("Host", "lol3r.net")],
        "".as_bytes(),
    );

    let rule = Rule::new(
        1,
        vec![
            Matcher::Domain("lol3r.net".to_owned()),
            Matcher::PathPrefix("/other/".to_owned()),
        ],
        vec![],
        Service::new("test".to_owned()),
    );

    assert_eq!(false, rule.matches(&req));
}
