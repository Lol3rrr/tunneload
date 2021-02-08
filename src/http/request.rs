use crate::http::Method;

/// A single Header-Pair
#[derive(Debug, PartialEq)]
pub struct Header<'a> {
    key: &'a str,
    value: &'a str,
}

/// Represents a single HTTP-Request
#[derive(Debug, PartialEq)]
pub struct Request<'a> {
    buffer: &'a [u8],
    method: Method,
    path: &'a str,
    headers: Vec<Header<'a>>,
    body: &'a [u8],
}

impl Request<'_> {
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

    fn parse_raw_path<'a>(raw_part: &'a [u8]) -> Option<(&'a str, usize)> {
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

    fn parse_raw_protocol<'a>(raw_part: &'a [u8]) -> Option<(&'a str, usize)> {
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

    fn parse_raw_headers<'a>(raw_part: &'a [u8]) -> Option<(Vec<Header<'a>>, usize)> {
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

                    let tmp_header = Header { key, value };
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
    pub fn parse<'a>(raw_in_request: &'a [u8]) -> Option<Request<'a>> {
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
            headers,
            body,
        })
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
            headers: vec![
                Header {
                    key: "Test-1",
                    value: "Value-1",
                },
                Header {
                    key: "Test-2",
                    value: "Value-2",
                },
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
            headers: vec![
                Header {
                    key: "Test-1",
                    value: "Value-1",
                },
                Header {
                    key: "Test-2",
                    value: "Value-2",
                },
            ],
            body: "".as_bytes(),
        }),
        Request::parse(req)
    );
}
