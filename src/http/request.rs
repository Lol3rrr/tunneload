use crate::http::{HeaderValue, Headers, Method};

/// Represents a single HTTP-Request
#[derive(Debug, PartialEq)]
pub struct Request<'a> {
    method: Method,
    pub path: &'a str,
    protocol: &'a str,
    pub headers: Headers<'a>,
    body: &'a [u8],
}

impl<'a> Request<'a> {
    pub fn new(
        protocol: &'a str,
        method: Method,
        path: &'a str,
        headers: Headers<'a>,
        body: &'a [u8],
    ) -> Self {
        Self {
            method,
            path,
            protocol,
            headers,
            body,
        }
    }

    pub fn serialize(&self) -> (Vec<u8>, &[u8]) {
        let method = self.method.serialize();
        let capacity = method.len() + 1 + self.path.len() + 1 + self.protocol.len() + 4;
        let mut result = Vec::with_capacity(capacity);

        // The first line with method, path, protocol
        result.extend_from_slice(method.as_bytes());
        result.push(b' ');
        result.extend_from_slice(self.path.as_bytes());
        result.push(b' ');
        result.extend_from_slice(self.protocol.as_bytes());
        result.extend_from_slice("\r\n".as_bytes());

        // The headers
        self.headers.serialize(&mut result);

        // The ending of the head
        result.extend_from_slice("\r\n".as_bytes());

        (result, self.body)
    }

    pub fn protocol(&'a self) -> &'a str {
        &self.protocol
    }
    pub fn method(&'a self) -> &'a Method {
        &self.method
    }
    pub fn path(&'a self) -> &'a str {
        &self.path
    }
    pub fn headers(&'a self) -> &'a Headers<'a> {
        &self.headers
    }
    pub fn body(&'a self) -> &'a [u8] {
        self.body
    }

    pub fn is_keep_alive(&self) -> bool {
        match self.headers.get("Connection") {
            None => false,
            Some(value) => value == &HeaderValue::StrRef("Keep-Alive"),
        }
    }

    pub fn set_path(&'a mut self, n_path: &'a str) {
        self.path = n_path;
    }
}

impl std::fmt::Display for Request<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] Path: '{}'", self.method, self.path)
    }
}

#[test]
fn serialize_valid() {
    let mut headers = Headers::new();
    headers.add("test-1", "value-1");

    let req = Request::new("HTTP/1.1", Method::GET, "/test", headers, "body".as_bytes());
    let raw_header = "GET /test HTTP/1.1\r\ntest-1: value-1\r\n\r\n";
    let header_resp = raw_header.as_bytes().to_vec();
    let body_resp = "body".as_bytes();

    assert_eq!(req.serialize(), (header_resp, body_resp));
}
#[test]
fn serialize_valid_no_body() {
    let mut headers = Headers::new();
    headers.add("test-1", "value-1");

    let req = Request::new("HTTP/1.1", Method::GET, "/test", headers, "".as_bytes());
    let raw_header = "GET /test HTTP/1.1\r\ntest-1: value-1\r\n\r\n";
    let resp_header = raw_header.as_bytes().to_vec();
    let resp_body = "".as_bytes();

    assert_eq!(req.serialize(), (resp_header, resp_body));
}

#[test]
fn is_keep_alive_not_set() {
    let mut headers = Headers::new();
    headers.add("test-1", "value-1");

    let req = Request::new("HTTP/1.1", Method::GET, "/test", headers, "".as_bytes());

    assert_eq!(false, req.is_keep_alive());
}
#[test]
fn is_keep_alive_is_set() {
    let mut headers = Headers::new();
    headers.add("Connection", "Keep-Alive");

    let req = Request::new("HTTP/1.1", Method::GET, "/test", headers, "".as_bytes());

    assert_eq!(true, req.is_keep_alive());
}
#[test]
fn is_keep_alive_is_set_to_off() {
    let mut headers = Headers::new();
    headers.add("Connection", "Close");

    let req = Request::new("HTTP/1.1", Method::GET, "/test", headers, "".as_bytes());

    assert_eq!(false, req.is_keep_alive());
}
