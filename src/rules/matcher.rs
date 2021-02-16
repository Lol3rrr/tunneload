use crate::http::Request;

#[cfg(test)]
use crate::http::Method;

#[derive(Clone, Debug, PartialEq)]
pub enum Matcher {
    And(Vec<Matcher>),
    Or(Vec<Matcher>),
    Domain(String),
    PathPrefix(String),
}

impl Matcher {
    /// Checks if the current Rule matches the given
    /// HTTP-Request
    pub fn matches(&self, req: &Request) -> bool {
        match *self {
            Self::And(ref matchers) => {
                for tmp in matchers {
                    if !tmp.matches(req) {
                        return false;
                    }
                }

                true
            }
            Self::Or(ref matchers) => {
                for tmp in matchers {
                    if tmp.matches(req) {
                        return true;
                    }
                }

                false
            }
            Self::Domain(ref domain) => {
                for (key, value) in req.headers() {
                    if key == "Host" {
                        return value == domain;
                    }
                }
                false
            }
            Self::PathPrefix(ref path) => {
                let path_length = path.len();
                let req_path_length = req.path().len();

                req_path_length >= path_length && &req.path()[0..path_length] == path.as_str()
            }
        }
    }
}

#[test]
fn matcher_domain_matching() {
    let mut headers = std::collections::BTreeMap::new();
    headers.insert("Host".to_owned(), "lol3r.net".to_owned());

    let req = Request::new("HTTP/1.1", Method::GET, "/path", headers, "".as_bytes());

    let rule = Matcher::Domain("lol3r.net".to_owned());
    assert_eq!(true, rule.matches(&req));
}
#[test]
fn matcher_domain_not_matching() {
    let mut headers = std::collections::BTreeMap::new();
    headers.insert("Host".to_owned(), "lol3r.net".to_owned());

    let req = Request::new("HTTP/1.1", Method::GET, "/path", headers, "".as_bytes());

    let rule = Matcher::Domain("google.com".to_owned());
    assert_eq!(false, rule.matches(&req));
}

#[test]
fn matcher_pathprefix_matching() {
    let headers = std::collections::BTreeMap::new();

    let req = Request::new("HTTP/1.1", Method::GET, "/api/test", headers, "".as_bytes());

    let rule = Matcher::PathPrefix("/api/".to_owned());
    assert_eq!(true, rule.matches(&req));
}
#[test]
fn matcher_pathprefix_not_matching() {
    let headers = std::collections::BTreeMap::new();

    let req = Request::new(
        "HTTP/1.1",
        Method::GET,
        "/otherapi/test",
        headers,
        "".as_bytes(),
    );

    let rule = Matcher::PathPrefix("/api/".to_owned());
    assert_eq!(false, rule.matches(&req));
}
#[test]
fn matcher_pathprefix_not_matching_shorter_path() {
    let headers = std::collections::BTreeMap::new();

    let req = Request::new("HTTP/1.1", Method::GET, "/", headers, "".as_bytes());

    let rule = Matcher::PathPrefix("/api/".to_owned());
    assert_eq!(false, rule.matches(&req));
}

#[test]
fn and_all_matching() {
    let mut headers = std::collections::BTreeMap::new();
    headers.insert("Host".to_owned(), "example.net".to_owned());
    let req = Request::new("HTTP/1.1", Method::GET, "/api/test", headers, "".as_bytes());

    let rule = Matcher::And(vec![
        Matcher::PathPrefix("/api/".to_owned()),
        Matcher::Domain("example.net".to_owned()),
    ]);

    assert_eq!(true, rule.matches(&req));
}
#[test]
fn and_one_not_matching() {
    let mut headers = std::collections::BTreeMap::new();
    headers.insert("Host".to_owned(), "example.net".to_owned());
    let req = Request::new("HTTP/1.1", Method::GET, "/test", headers, "".as_bytes());

    let rule = Matcher::And(vec![
        Matcher::PathPrefix("/api/".to_owned()),
        Matcher::Domain("example.net".to_owned()),
    ]);

    assert_eq!(false, rule.matches(&req));
}

#[test]
fn or_all_matching() {
    let mut headers = std::collections::BTreeMap::new();
    headers.insert("Host".to_owned(), "example.net".to_owned());
    let req = Request::new("HTTP/1.1", Method::GET, "/api/test", headers, "".as_bytes());

    let rule = Matcher::Or(vec![
        Matcher::PathPrefix("/api/".to_owned()),
        Matcher::Domain("example.net".to_owned()),
    ]);

    assert_eq!(true, rule.matches(&req));
}
#[test]
fn or_one_matching() {
    let mut headers = std::collections::BTreeMap::new();
    headers.insert("Host".to_owned(), "example.net".to_owned());
    let req = Request::new("HTTP/1.1", Method::GET, "/test", headers, "".as_bytes());

    let rule = Matcher::Or(vec![
        Matcher::PathPrefix("/api/".to_owned()),
        Matcher::Domain("example.net".to_owned()),
    ]);

    assert_eq!(true, rule.matches(&req));
}
#[test]
fn or_none_matching() {
    let mut headers = std::collections::BTreeMap::new();
    headers.insert("Host".to_owned(), "other.net".to_owned());
    let req = Request::new("HTTP/1.1", Method::GET, "/test", headers, "".as_bytes());

    let rule = Matcher::Or(vec![
        Matcher::PathPrefix("/api/".to_owned()),
        Matcher::Domain("example.net".to_owned()),
    ]);

    assert_eq!(false, rule.matches(&req));
}
