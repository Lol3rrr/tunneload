use crate::http::{HeaderKey, HeaderValue, Headers, StatusCode};

/// Represents a single HTTP-Request
#[derive(Debug, PartialEq)]
pub struct Response<'a> {
    buffer: &'a [u8],
    status_code: StatusCode,
    protocol: &'a str,
    pub headers: Headers<'a>,
    pub body: Vec<u8>,
}

impl<'a> Response<'a> {
    pub fn new(
        protocol: &'a str,
        status_code: StatusCode,
        headers: Headers<'a>,
        body: Vec<u8>,
    ) -> Self {
        Self {
            buffer: &[],
            status_code,
            protocol,
            headers,
            body,
        }
    }

    pub fn serialize(&self) -> (Vec<u8>, &[u8]) {
        let protocol = self.protocol;
        let status_code = self.status_code.serialize();

        let capacity = protocol.len() + 1 + status_code.len() + 4;
        let mut result = Vec::with_capacity(capacity);

        // The first line with method, path, protocol
        result.extend_from_slice(protocol.as_bytes());
        result.push(b' ');
        result.extend_from_slice(status_code.as_bytes());
        result.extend_from_slice("\r\n".as_bytes());

        // The headers
        self.headers.serialize(&mut result);

        // The ending of the head
        result.extend_from_slice("\r\n".as_bytes());

        (result, &self.body)
    }

    pub fn protocol(&'a self) -> &'a str {
        &self.protocol
    }
    pub fn status_code(&'a self) -> &StatusCode {
        &self.status_code
    }
    pub fn headers(&'a self) -> &'a Headers<'a> {
        &self.headers
    }
    pub fn body(&'a self) -> &'a [u8] {
        &self.body
    }

    pub fn add_header<'b, K, V>(&mut self, key: K, value: V)
    where
        'b: 'a,
        K: Into<HeaderKey<'a>>,
        V: Into<HeaderValue<'a>>,
    {
        self.headers.add(key, value);
    }
}

#[test]
fn serialize_valid() {
    let mut headers = Headers::new();
    headers.add("test-1", "value-1");

    let req = Response::new(
        "HTTP/1.1",
        StatusCode::OK,
        headers,
        "body".as_bytes().to_vec(),
    );
    let raw_resp_header = "HTTP/1.1 200 OK\r\ntest-1: value-1\r\n\r\n";
    let resp_header = raw_resp_header.as_bytes().to_vec();
    let resp_body = "body".as_bytes();

    assert_eq!(req.serialize(), (resp_header, resp_body));
}

#[test]
fn serialize_valid_no_body() {
    let mut headers = Headers::new();
    headers.add("test-1", "value-1");

    let req = Response::new("HTTP/1.1", StatusCode::OK, headers, "".as_bytes().to_vec());
    let raw_resp_header = "HTTP/1.1 200 OK\r\ntest-1: value-1\r\n\r\n";
    let resp_header = raw_resp_header.as_bytes().to_vec();
    let resp_body = "".as_bytes();

    assert_eq!(req.serialize(), (resp_header, resp_body));
}
