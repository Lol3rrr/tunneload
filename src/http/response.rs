use crate::http::{parser, StatusCode};

/// Represents a single HTTP-Request
#[derive(Debug, PartialEq)]
pub struct Response<'a> {
    buffer: &'a [u8],
    status_code: StatusCode,
    protocol: &'a str,
    pub headers: std::collections::BTreeMap<String, String>,
    body: &'a [u8],
}

impl<'a> Response<'a> {
    pub fn new(
        protocol: &'a str,
        status_code: StatusCode,
        headers: std::collections::BTreeMap<String, String>,
        body: &'a [u8],
    ) -> Self {
        Self {
            buffer: &[],
            status_code,
            protocol,
            headers,
            body,
        }
    }

    /// Parses a raw byte-slice into a HTTP-Request
    pub fn parse(raw_in_response: &'a [u8]) -> Option<Response<'a>> {
        let mut global_offset = 0;

        let protocol_part = &raw_in_response[global_offset..];
        let (protocol, end_index) = match parser::parse_protocol(protocol_part) {
            Some(s) => s,
            None => {
                println!("Could not get protocol");
                return None;
            }
        };
        global_offset += end_index + 1;

        let status_code_part = &raw_in_response[global_offset..];
        let (status_code, end_index) = match parser::parse_status_code(status_code_part) {
            Some(s) => s,
            None => {
                println!("Could not get status-code");
                return None;
            }
        };
        global_offset += end_index + 2;

        let headers_part = &raw_in_response[global_offset..];
        let (headers, end_index) = match parser::parse_headers(headers_part) {
            Some(s) => s,
            None => {
                println!("Could not parse headers");
                return None;
            }
        };
        global_offset += end_index + 2;

        let body = &raw_in_response[global_offset..];

        Some(Response {
            buffer: raw_in_response,
            status_code,
            protocol,
            headers,
            body,
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let protocol = self.protocol;
        let status_code = self.status_code.serialize();

        let capacity = protocol.len() + 1 + status_code.len() + 4 + self.body.len();
        let mut result = Vec::with_capacity(capacity);

        // The first line with method, path, protocol
        result.extend_from_slice(protocol.as_bytes());
        result.push(b' ');
        result.extend_from_slice(&status_code);
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
    pub fn status_code(&'a self) -> &StatusCode {
        &self.status_code
    }
    pub fn headers(&'a self) -> &'a std::collections::BTreeMap<String, String> {
        &self.headers
    }
    pub fn body(&'a self) -> &'a [u8] {
        self.body
    }
}

#[test]
fn parse_valid() {
    let req =
        "HTTP/1.1 200 OK\r\nTest-1: Value-1\r\nTest-2: Value-2\r\n\r\nThis is just some test-body"
            .as_bytes();
    let mut headers = std::collections::BTreeMap::new();
    headers.insert("Test-1".to_owned(), "Value-1".to_owned());
    headers.insert("Test-2".to_owned(), "Value-2".to_owned());

    assert_eq!(
        Some(Response {
            buffer: req,
            status_code: StatusCode::OK,
            protocol: "HTTP/1.1",
            headers,
            body: "This is just some test-body".as_bytes(),
        }),
        Response::parse(req)
    );
}

#[test]
fn parse_valid_no_body() {
    let req = "HTTP/1.1 404 Not Found\r\nTest-1: Value-1\r\nTest-2: Value-2\r\n\r\n".as_bytes();
    let mut headers = std::collections::BTreeMap::new();
    headers.insert("Test-1".to_owned(), "Value-1".to_owned());
    headers.insert("Test-2".to_owned(), "Value-2".to_owned());

    assert_eq!(
        Some(Response {
            buffer: req,
            status_code: StatusCode::NotFound,
            protocol: "HTTP/1.1",
            headers,
            body: "".as_bytes(),
        }),
        Response::parse(req)
    );
}

#[test]
fn serialize_valid() {
    let mut headers = std::collections::BTreeMap::new();
    headers.insert("test-1".to_owned(), "value-1".to_owned());

    let req = Response::new("HTTP/1.1", StatusCode::OK, headers, "body".as_bytes());
    let raw_resp = "HTTP/1.1 200\r\ntest-1: value-1\r\n\r\nbody";
    let resp = raw_resp.as_bytes();

    assert_eq!(req.serialize(), resp);
}

#[test]
fn serialize_valid_no_body() {
    let mut headers = std::collections::BTreeMap::new();
    headers.insert("test-1".to_owned(), "value-1".to_owned());

    let req = Response::new("HTTP/1.1", StatusCode::OK, headers, "".as_bytes());
    let raw_resp = "HTTP/1.1 200\r\ntest-1: value-1\r\n\r\n";
    let resp = raw_resp.as_bytes();

    assert_eq!(req.serialize(), resp);
}
