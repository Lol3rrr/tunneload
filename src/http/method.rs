/// The different HTTP-Methods as defined by
/// [RFC 2616 5.1.1](https://tools.ietf.org/html/rfc2616#section-5.1.1)
#[derive(Debug, PartialEq)]
pub enum Method {
    OPTIONS,
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    TRACE,
    CONNECT,
}

impl Method {
    pub fn parse(raw_method: &str) -> Option<Method> {
        match raw_method {
            "OPTIONS" => Some(Method::OPTIONS),
            "GET" => Some(Method::GET),
            "HEAD" => Some(Method::HEAD),
            "POST" => Some(Method::POST),
            "PUT" => Some(Method::PUT),
            "DELETE" => Some(Method::DELETE),
            "TRACE" => Some(Method::TRACE),
            "CONNECT" => Some(Method::CONNECT),
            _ => None,
        }
    }
}

#[test]
fn parse_method_options() {
    assert_eq!(Some(Method::OPTIONS), Method::parse("OPTIONS"));
}
#[test]
fn parse_method_get() {
    assert_eq!(Some(Method::GET), Method::parse("GET"));
}
#[test]
fn parse_method_head() {
    assert_eq!(Some(Method::HEAD), Method::parse("HEAD"));
}
#[test]
fn parse_method_post() {
    assert_eq!(Some(Method::POST), Method::parse("POST"));
}
#[test]
fn parse_method_put() {
    assert_eq!(Some(Method::PUT), Method::parse("PUT"));
}
#[test]
fn parse_method_delete() {
    assert_eq!(Some(Method::DELETE), Method::parse("DELETE"));
}
#[test]
fn parse_method_trace() {
    assert_eq!(Some(Method::TRACE), Method::parse("TRACE"));
}
#[test]
fn parse_method_connect() {
    assert_eq!(Some(Method::CONNECT), Method::parse("CONNECT"));
}
