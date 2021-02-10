use crate::http::Method;

/// Represents a single HTTP-Request
#[derive(Debug, PartialEq)]
pub struct Request<'a> {
    buffer: &'a [u8],
    method: Method,
    pub path: &'a str,
    protocol: &'a str,
    pub headers: std::collections::BTreeMap<String, String>,
    body: &'a [u8],
}

impl<'a> Request<'a> {
    pub fn new(
        protocol: &'a str,
        method: Method,
        path: &'a str,
        headers: std::collections::BTreeMap<String, String>,
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

    fn parse_raw_method(raw_part: &[u8]) -> Option<(Method, usize)> {
        for (index, c) in raw_part.iter().enumerate() {
            if let b' ' = c {
                match Method::parse(std::str::from_utf8(&raw_part[0..index]).unwrap()) {
                    Some(s) => {
                        return Some((s, index));
                    }
                    None => {
                        return None;
                    }
                };
            }
        }

        None
    }

    fn parse_raw_path(raw_part: &[u8]) -> Option<(&str, usize)> {
        for (index, c) in raw_part.iter().enumerate() {
            if let b' ' = c {
                let result = std::str::from_utf8(&raw_part[0..index]).unwrap();
                return Some((result, index));
            }
        }

        None
    }

    fn parse_raw_protocol(raw_part: &[u8]) -> Option<(&str, usize)> {
        for (index, c) in raw_part.iter().enumerate() {
            if let b'\r' = c {
                let result = std::str::from_utf8(&raw_part[0..index]).unwrap();
                return Some((result, index));
            }
        }

        None
    }

    fn parse_raw_headers(
        raw_part: &[u8],
    ) -> Option<(std::collections::BTreeMap<String, String>, usize)> {
        let mut result = std::collections::BTreeMap::new();

        let mut start = 0;

        let mut key = "";

        let mut key_part = true;
        for (index, c) in raw_part.iter().enumerate() {
            match c {
                b':' if key_part => {
                    key = std::str::from_utf8(&raw_part[start..index]).unwrap();
                    key_part = !key_part;
                    start = index + 2;
                }
                b'\r' if !key_part => {
                    let value = std::str::from_utf8(&raw_part[start..index]).unwrap();

                    result.insert(key.to_owned(), value.to_owned());

                    key_part = !key_part;
                    start = index + 2;
                }
                b'\r' if key_part => {
                    return Some((result, index));
                }
                _ => {}
            };
        }

        None
    }

    /// Parses a raw byte-slice into a HTTP-Request
    pub fn parse(raw_in_request: &'a [u8]) -> Option<Request<'a>> {
        let mut global_offset = 0;

        let method_part = &raw_in_request[global_offset..];
        let (method, end_index) = match Request::parse_raw_method(method_part) {
            Some(s) => s,
            None => {
                println!("Could not parse Method");
                return None;
            }
        };
        global_offset += end_index + 1;

        let path_part = &raw_in_request[global_offset..];
        let (path, end_index) = match Request::parse_raw_path(path_part) {
            Some(s) => s,
            None => {
                println!("Could not get path");
                return None;
            }
        };
        global_offset += end_index + 1;
        let protocol_part = &raw_in_request[global_offset..];
        let (protocol, end_index) = match Request::parse_raw_protocol(protocol_part) {
            Some(s) => s,
            None => {
                println!("Could not get protocol");
                return None;
            }
        };
        global_offset += end_index + 2;

        let headers_part = &raw_in_request[global_offset..];
        let (headers, end_index) = match Request::parse_raw_headers(headers_part) {
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

    pub fn serialize(&self) -> Vec<u8> {
        let method = self.method.serialize();
        let capacity =
            method.len() + 1 + self.path.len() + 1 + self.protocol.len() + 4 + self.body.len();
        let mut result = Vec::with_capacity(capacity);

        // The first line with method, path, protocol
        result.extend_from_slice(method.as_bytes());
        result.push(b' ');
        result.extend_from_slice(self.path.as_bytes());
        result.push(b' ');
        result.extend_from_slice(self.protocol.as_bytes());
        result.extend_from_slice("\r\n".as_bytes());

        // The headers
        for (key, value) in self.headers.iter() {
            result.extend_from_slice(key.as_bytes());
            result.extend_from_slice(": ".as_bytes());
            result.extend_from_slice(value.as_bytes());
            result.extend_from_slice("\r\n".as_bytes());
        }

        // The ending of the head
        result.extend_from_slice("\r\n".as_bytes());

        // The body
        result.extend_from_slice(self.body);

        result
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
    pub fn headers(&'a self) -> &'a std::collections::BTreeMap<String, String> {
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
    let mut headers = std::collections::BTreeMap::new();
    headers.insert("Test-1".to_owned(), "Value-1".to_owned());
    headers.insert("Test-2".to_owned(), "Value-2".to_owned());

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
    let mut headers = std::collections::BTreeMap::new();
    headers.insert("Test-1".to_owned(), "Value-1".to_owned());
    headers.insert("Test-2".to_owned(), "Value-2".to_owned());

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
    let mut headers = std::collections::BTreeMap::new();
    headers.insert("test-1".to_owned(), "value-1".to_owned());

    let req = Request::new("HTTP/1.1", Method::GET, "/test", headers, "body".as_bytes());
    let raw_resp = "GET /test HTTP/1.1\r\ntest-1: value-1\r\n\r\nbody";
    let resp = raw_resp.as_bytes();

    assert_eq!(req.serialize(), resp);
}

#[test]
fn serialize_valid_no_body() {
    let mut headers = std::collections::BTreeMap::new();
    headers.insert("test-1".to_owned(), "value-1".to_owned());

    let req = Request::new("HTTP/1.1", Method::GET, "/test", headers, "".as_bytes());
    let raw_resp = "GET /test HTTP/1.1\r\ntest-1: value-1\r\n\r\n";
    let resp = raw_resp.as_bytes();

    assert_eq!(req.serialize(), resp);
}
