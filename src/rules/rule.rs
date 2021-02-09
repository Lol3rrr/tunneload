use crate::http::Request;

#[derive(Clone, Debug)]
pub struct Rule {
    matcher: Matcher,
    service: Service,
}

#[derive(Clone, Debug)]
pub enum Matcher {
    Domain(String),
}

#[derive(Clone, Debug)]
pub struct Service {
    address: String,
}

impl Rule {
    pub fn new(matcher: Matcher, service: Service) -> Self {
        Self { matcher, service }
    }

    pub fn matches(&self, req: &Request) -> Option<Service> {
        if self.matcher.matches(req) {
            Some(self.service.clone())
        } else {
            None
        }
    }
}

impl Service {
    pub fn new(dest: String) -> Self {
        Self { address: dest }
    }

    pub fn address(&self) -> &str {
        &self.address
    }
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
            _ => false,
        }
    }
}

#[test]
fn matcher_match_domain() {
    // A Request where the "Host" header was set to "lol3r.net"
    let req = Request::parse("GET /path HTTP/1.1.\r\nHost: lol3r.net\r\n\r\n".as_bytes()).unwrap();
    let rule = Matcher::Domain("lol3r.net".to_owned());
    assert_eq!(true, rule.matches(&req));
}
#[test]
fn matcher_doesnt_match_domain() {
    // A Request where the "Host" header was set to "lol3r.net"
    let req = Request::parse("GET /path HTTP/1.1.\r\nHost: lol3r.net\r\n\r\n".as_bytes()).unwrap();
    let rule = Matcher::Domain("google.com".to_owned());
    assert_eq!(false, rule.matches(&req));
}
