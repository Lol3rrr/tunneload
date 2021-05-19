use serde::Serialize;
use stream_httparse::Request;

/// Used to determine if a Request matches certain
/// criteria
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", content = "c")]
pub enum Matcher {
    /// Evaluates all the internal Matchers and only returns
    /// true if all of them evalutate to true
    And(Vec<Matcher>),
    /// Evaluates all of the internal Matchers until one
    /// of them returns true
    Or(Vec<Matcher>),
    /// Matches the Domain("Host"-Header) of the Request
    /// against the given Domain
    Domain(String),
    /// Matches the Path of the Request against the given
    /// Prefix
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
            Self::Domain(ref domain) => match req.headers().get("Host") {
                Some(value) => value == domain,
                None => false,
            },
            Self::PathPrefix(ref path) => {
                let path_length = path.len();
                let req_path_length = req.path().len();

                req_path_length >= path_length && &req.path()[0..path_length] == path.as_str()
            }
        }
    }

    /// Returns the Domain that belongs to this Matcher
    pub fn get_host(&self) -> Option<String> {
        match *self {
            Self::And(ref matchers) | Self::Or(ref matchers) => {
                for tmp in matchers.iter() {
                    if let Some(d) = tmp.get_host() {
                        return Some(d);
                    }
                }
                None
            }
            Self::Domain(ref domain) => Some(domain.to_owned()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use stream_httparse::{Headers, Method};

    #[test]
    fn matcher_domain_matching() {
        let mut headers = Headers::new();
        headers.set("Host", "lol3r.net");

        let req = Request::new("HTTP/1.1", Method::GET, "/path", headers, "".as_bytes());

        let rule = Matcher::Domain("lol3r.net".to_owned());
        assert_eq!(true, rule.matches(&req));
    }
    #[test]
    fn matcher_domain_not_matching() {
        let mut headers = Headers::new();
        headers.set("Host", "lol3r.net");

        let req = Request::new("HTTP/1.1", Method::GET, "/path", headers, "".as_bytes());

        let rule = Matcher::Domain("google.com".to_owned());
        assert_eq!(false, rule.matches(&req));
    }

    #[test]
    fn matcher_pathprefix_matching() {
        let headers = Headers::new();

        let req = Request::new("HTTP/1.1", Method::GET, "/api/test", headers, "".as_bytes());

        let rule = Matcher::PathPrefix("/api/".to_owned());
        assert_eq!(true, rule.matches(&req));
    }
    #[test]
    fn matcher_pathprefix_not_matching() {
        let headers = Headers::new();

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
        let headers = Headers::new();

        let req = Request::new("HTTP/1.1", Method::GET, "/", headers, "".as_bytes());

        let rule = Matcher::PathPrefix("/api/".to_owned());
        assert_eq!(false, rule.matches(&req));
    }

    #[test]
    fn and_all_matching() {
        let mut headers = Headers::new();
        headers.set("Host", "example.net");
        let req = Request::new("HTTP/1.1", Method::GET, "/api/test", headers, "".as_bytes());

        let rule = Matcher::And(vec![
            Matcher::PathPrefix("/api/".to_owned()),
            Matcher::Domain("example.net".to_owned()),
        ]);

        assert_eq!(true, rule.matches(&req));
    }
    #[test]
    fn and_one_not_matching() {
        let mut headers = Headers::new();
        headers.set("Host", "example.net");
        let req = Request::new("HTTP/1.1", Method::GET, "/test", headers, "".as_bytes());

        let rule = Matcher::And(vec![
            Matcher::PathPrefix("/api/".to_owned()),
            Matcher::Domain("example.net".to_owned()),
        ]);

        assert_eq!(false, rule.matches(&req));
    }

    #[test]
    fn or_all_matching() {
        let mut headers = Headers::new();
        headers.set("Host", "example.net");
        let req = Request::new("HTTP/1.1", Method::GET, "/api/test", headers, "".as_bytes());

        let rule = Matcher::Or(vec![
            Matcher::PathPrefix("/api/".to_owned()),
            Matcher::Domain("example.net".to_owned()),
        ]);

        assert_eq!(true, rule.matches(&req));
    }
    #[test]
    fn or_one_matching() {
        let mut headers = Headers::new();
        headers.set("Host", "example.net");
        let req = Request::new("HTTP/1.1", Method::GET, "/test", headers, "".as_bytes());

        let rule = Matcher::Or(vec![
            Matcher::PathPrefix("/api/".to_owned()),
            Matcher::Domain("example.net".to_owned()),
        ]);

        assert_eq!(true, rule.matches(&req));
    }
    #[test]
    fn or_none_matching() {
        let mut headers = Headers::new();
        headers.set("Host", "other.net");
        let req = Request::new("HTTP/1.1", Method::GET, "/test", headers, "".as_bytes());

        let rule = Matcher::Or(vec![
            Matcher::PathPrefix("/api/".to_owned()),
            Matcher::Domain("example.net".to_owned()),
        ]);

        assert_eq!(false, rule.matches(&req));
    }

    #[test]
    fn get_host_domain() {
        let matcher = Matcher::Domain("test".to_owned());
        assert_eq!(Some("test".to_owned()), matcher.get_host());
    }
    #[test]
    fn get_host_path_prefix() {
        let matcher = Matcher::PathPrefix("test".to_owned());
        assert_eq!(None, matcher.get_host());
    }
    #[test]
    fn get_host_and_path_domain() {
        let matcher = Matcher::And(vec![
            Matcher::PathPrefix("test".to_owned()),
            Matcher::Domain("test".to_owned()),
        ]);
        assert_eq!(Some("test".to_owned()), matcher.get_host());
    }
    #[test]
    fn get_host_or_path_domain() {
        let matcher = Matcher::Or(vec![
            Matcher::PathPrefix("test".to_owned()),
            Matcher::Domain("test".to_owned()),
        ]);
        assert_eq!(Some("test".to_owned()), matcher.get_host());
    }
    #[test]
    fn get_host_and_path() {
        let matcher = Matcher::And(vec![Matcher::PathPrefix("test".to_owned())]);
        assert_eq!(None, matcher.get_host());
    }
    #[test]
    fn get_host_or_path() {
        let matcher = Matcher::Or(vec![Matcher::PathPrefix("test".to_owned())]);
        assert_eq!(None, matcher.get_host());
    }
}
