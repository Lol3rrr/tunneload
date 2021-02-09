use crate::http::Request;

#[cfg(test)]
use crate::http::{Header, Method};

#[derive(Clone, Debug, PartialEq)]
pub enum Matcher {
    Domain(String),
}

impl Matcher {
    /// Checks if the current Rule matches the given
    /// HTTP-Request
    pub fn matches(&self, req: &Request) -> bool {
        match *self {
            Self::Domain(ref domain) => {
                for header in req.headers() {
                    if header.key() == "Host" {
                        return header.value() == domain;
                    }
                }
                false
            }
        }
    }
}

#[test]
fn matcher_domain_matching() {
    let headers = vec![Header::new("Host", "lol3r.net")];
    let req = Request::new("HTTP/1.1", Method::GET, "/path", headers, "".as_bytes());

    let rule = Matcher::Domain("lol3r.net".to_owned());
    assert_eq!(true, rule.matches(&req));
}
#[test]
fn matcher_domain_not_matching() {
    let headers = vec![Header::new("Host", "lol3r.net")];
    let req = Request::new("HTTP/1.1", Method::GET, "/path", headers, "".as_bytes());

    let rule = Matcher::Domain("google.com".to_owned());
    assert_eq!(false, rule.matches(&req));
}
