use crate::http::Request;

#[cfg(test)]
use crate::http::{Header, Method};

#[derive(Clone, Debug, PartialEq)]
pub enum Matcher {
    Domain(String),
    PathPrefix(String),
}

impl Matcher {
    /// Checks if the current Rule matches the given
    /// HTTP-Request
    pub fn matches(&self, req: &Request) -> bool {
        match *self {
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
    let mut headers = std::collections::BTreeMap::new();

    let req = Request::new("HTTP/1.1", Method::GET, "/api/test", headers, "".as_bytes());

    let rule = Matcher::PathPrefix("/api/".to_owned());
    assert_eq!(true, rule.matches(&req));
}
#[test]
fn matcher_pathprefix_not_matching() {
    let mut headers = std::collections::BTreeMap::new();

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
    let mut headers = std::collections::BTreeMap::new();

    let req = Request::new("HTTP/1.1", Method::GET, "/", headers, "".as_bytes());

    let rule = Matcher::PathPrefix("/api/".to_owned());
    assert_eq!(false, rule.matches(&req));
}
