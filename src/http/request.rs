use crate::http::{Header, Method};

/// Represents a single HTTP-Request
#[derive(Debug, PartialEq)]
pub struct Request<'a> {
    buffer: &'a [u8],
    method: Method,
    path: &'a str,
    protocol: &'a str,
    headers: Vec<Header<'a>>,
    body: &'a [u8],
}

impl<'a> Request<'a> {
    pub fn new(
        protocol: &'a str,
        method: Method,
        path: &'a str,
        headers: Vec<Header<'a>>,
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
            match c {
                b' ' => {
                    match Method::parse(std::str::from_utf8(&raw_part[0..index]).unwrap()) {
                        Some(s) => {
                            return Some((s, index));
                        }
                        None => {
                            return None;
                        }
                    };
                }
                _ => {}
            };
        }

        None
    }

    fn parse_raw_path<'b>(raw_part: &'b [u8]) -> Option<(&'b str, usize)> {
        for (index, c) in raw_part.iter().enumerate() {
            match c {
                b' ' => {
                    let result = std::str::from_utf8(&raw_part[0..index]).unwrap();
                    return Some((result, index));
                }
                _ => {}
            };
        }

        None
    }

    fn parse_raw_protocol<'b>(raw_part: &'b [u8]) -> Option<(&'b str, usize)> {
        for (index, c) in raw_part.iter().enumerate() {
            match c {
                b'\r' => {
                    let result = std::str::from_utf8(&raw_part[0..index]).unwrap();
                    return Some((result, index));
                }
                _ => {}
            };
        }

        None
    }

    fn parse_raw_headers<'b>(raw_part: &'b [u8]) -> Option<(Vec<Header<'b>>, usize)> {
        let mut result = Vec::new();

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

                    let tmp_header = Header::new(key, value);
                    result.push(tmp_header);

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

    pub fn serialize(self) -> Vec<u8> {
        let method = self.method.serialize();
        let capacity = method.len() + 1 + self.path.len() + 1 + self.protocol.len() + 2;
        let mut result = Vec::with_capacity(capacity);

        // The first line with method, path, protocol
        result.extend_from_slice(method.as_bytes());
        result.push(b' ');
        result.extend_from_slice(self.path.as_bytes());
        result.push(b' ');
        result.extend_from_slice(self.protocol.as_bytes());
        result.extend_from_slice("\r\n".as_bytes());

        // The headers
        for header in self.headers {
            result.extend_from_slice(header.key().as_bytes());
            result.extend_from_slice(": ".as_bytes());
            result.extend_from_slice(header.value().as_bytes());
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
    pub fn headers(&'a self) -> &'a Vec<Header<'a>> {
        &self.headers
    }
    pub fn body(&'a self) -> &'a [u8] {
        self.body
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
    assert_eq!(
        Some(Request {
            buffer: req,
            method: Method::GET,
            path: "/test",
            protocol: "HTTP/1.1",
            headers: vec![
                Header::new("Test-1", "Value-1"),
                Header::new("Test-2", "Value-2"),
            ],
            body: "This is just some test-body".as_bytes(),
        }),
        Request::parse(req)
    );
}

#[test]
fn parse_valid_no_body() {
    let req = "GET /test HTTP/1.1\r\nTest-1: Value-1\r\nTest-2: Value-2\r\n\r\n".as_bytes();
    assert_eq!(
        Some(Request {
            buffer: req,
            method: Method::GET,
            path: "/test",
            protocol: "HTTP/1.1",
            headers: vec![
                Header::new("Test-1", "Value-1"),
                Header::new("Test-2", "Value-2"),
            ],
            body: "".as_bytes(),
        }),
        Request::parse(req)
    );
}

#[test]
fn serialize_valid() {
    let mut headers = Vec::new();
    headers.push(Header::new("test-1", "value-1"));

    let req = Request::new("HTTP/1.1", Method::GET, "/test", headers, "body".as_bytes());
    let raw_resp = "GET /test HTTP/1.1\r\ntest-1: value-1\r\n\r\nbody";
    let resp = raw_resp.as_bytes();

    assert_eq!(req.serialize(), resp);
}

#[test]
fn serialize_valid_no_body() {
    let mut headers = Vec::new();
    headers.push(Header::new("test-1", "value-1"));

    let req = Request::new("HTTP/1.1", Method::GET, "/test", headers, "".as_bytes());
    let raw_resp = "GET /test HTTP/1.1\r\ntest-1: value-1\r\n\r\n";
    let resp = raw_resp.as_bytes();

    assert_eq!(req.serialize(), resp);
}
