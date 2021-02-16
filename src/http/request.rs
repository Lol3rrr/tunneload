use crate::http::{parser, Headers, Method};

/// Represents a single HTTP-Request
#[derive(Debug, PartialEq)]
pub struct Request<'a> {
    buffer: &'a [u8],
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
            buffer: &[],
            method,
            path,
            protocol,
            headers,
            body,
        }
    }

    /// Parses a raw byte-slice into a HTTP-Request
    pub fn parse(raw_in_request: &'a [u8]) -> Option<Request<'a>> {
        let mut global_offset = 0;

        let method_part = &raw_in_request[global_offset..];
        let (method, end_index) = match parser::parse_method(method_part) {
            Some(s) => s,
            None => {
                println!("Could not parse Method");
                return None;
            }
        };
        global_offset += end_index + 1;

        let path_part = &raw_in_request[global_offset..];
        let (path, end_index) = match parser::parse_path(path_part) {
            Some(s) => s,
            None => {
                println!("Could not get path");
                return None;
            }
        };
        global_offset += end_index + 1;
        let protocol_part = &raw_in_request[global_offset..];
        let (protocol, end_index) = match parser::parse_protocol(protocol_part) {
            Some(s) => s,
            None => {
                println!("Could not get protocol");
                return None;
            }
        };
        global_offset += end_index + 2;

        let headers_part = &raw_in_request[global_offset..];
        let (headers, end_index) = match parser::parse_headers(headers_part) {
            Some(s) => s,
            None => {
                println!("Could not parse headers");
                return None;
            }
        };
        global_offset += end_index + 2;

        let body = &raw_in_request[global_offset..];

        Some(Request {
            buffer: raw_in_request,
            method,
            path,
            protocol,
            headers,
            body,
        })
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
fn parse_valid() {
    let req = "GET /test HTTP/1.1\r\nTest-1: Value-1\r\nTest-2: Value-2\r\n\r\nThis is just some test-body".as_bytes();
    let mut headers = Headers::new();
    headers.add("Test-1", "Value-1");
    headers.add("Test-2", "Value-2");

    assert_eq!(
        Some(Request {
            buffer: req,
            method: Method::GET,
            path: "/test",
            protocol: "HTTP/1.1",
            headers,
            body: "This is just some test-body".as_bytes(),
        }),
        Request::parse(req)
    );
}

#[test]
fn parse_valid_no_body() {
    let req = "GET /test HTTP/1.1\r\nTest-1: Value-1\r\nTest-2: Value-2\r\n\r\n".as_bytes();
    let mut headers = Headers::new();
    headers.add("Test-1", "Value-1");
    headers.add("Test-2", "Value-2");

    assert_eq!(
        Some(Request {
            buffer: req,
            method: Method::GET,
            path: "/test",
            protocol: "HTTP/1.1",
            headers,
            body: "".as_bytes(),
        }),
        Request::parse(req)
    );
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
