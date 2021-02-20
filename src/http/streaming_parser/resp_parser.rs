use crate::http::{Headers, Response, StatusCode};

type ProtocolState = (usize, usize);
type StatusCodeState = (usize, usize);
type HeaderKeyState = (usize, usize);

enum State {
    Nothing,
    ProtocolParsed(ProtocolState),
    HeaderKey(
        ProtocolState,
        StatusCodeState,
        usize,
        Vec<((usize, usize), (usize, usize))>,
    ),
    HeaderValue(
        ProtocolState,
        StatusCodeState,
        HeaderKeyState,
        Vec<((usize, usize), (usize, usize))>,
    ),
    HeadersParsed(
        ProtocolState,
        StatusCodeState,
        Vec<((usize, usize), (usize, usize))>,
        usize,
    ),
}

pub struct RespParser {
    buffer: Vec<u8>,
    state: State,
}

impl RespParser {
    pub fn new_capacity(cap: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(cap),
            state: State::Nothing,
        }
    }

    fn parse(&mut self, byte: u8, current: usize) {
        match &mut self.state {
            State::Nothing if byte == b' ' => {
                let end = current;
                let n_state = State::ProtocolParsed((0, end));
                self.state = n_state;
            }
            State::ProtocolParsed(protocol) if byte == b'\r' => {
                let start = protocol.1;
                let end = current;

                let headers = Vec::with_capacity(10);
                let n_state = State::HeaderKey(*protocol, (start + 1, end), end, headers);
                self.state = n_state;
            }
            State::HeaderKey(protocol, status_code, raw_start, headers)
                if current == *raw_start + 2 && byte == b'\r' =>
            {
                let n_state = State::HeadersParsed(
                    *protocol,
                    *status_code,
                    std::mem::take(headers),
                    current + 2,
                );
                self.state = n_state;
            }
            State::HeaderKey(protocol, status_code, raw_start, headers) if byte == b':' => {
                let start = *raw_start + 2;
                let end = current;

                let n_state = State::HeaderValue(
                    *protocol,
                    *status_code,
                    (start, end),
                    std::mem::take(headers),
                );
                self.state = n_state;
            }
            State::HeaderValue(protocol, status_code, header_key, headers) if byte == b'\r' => {
                let start = header_key.1 + 2;
                let end = current;

                headers.push((*header_key, (start, end)));
                let n_state =
                    State::HeaderKey(*protocol, *status_code, end, std::mem::take(headers));
                self.state = n_state;
            }
            _ => {}
        };
    }

    pub fn block_parse(&mut self, bytes: &[u8]) {
        let start_point = self.buffer.len();
        self.buffer.reserve(bytes.len());

        for (index, tmp_byte) in bytes.iter().enumerate() {
            self.parse(*tmp_byte, start_point + index);
            self.buffer.push(*tmp_byte);
        }
    }

    pub fn finish<'a, 'b>(&'a self) -> Option<Response<'b>>
    where
        'a: 'b,
    {
        let (protocol, status_code, header, header_end) = match &self.state {
            State::HeadersParsed(p, stc, h, he) => (p, stc, h, he),
            _ => {
                return None;
            }
        };

        let raw_protocol = &self.buffer[protocol.0..protocol.1];
        let raw_status_code = &self.buffer[status_code.0..status_code.1];

        let protocol = unsafe { std::str::from_utf8_unchecked(raw_protocol) };
        let status_code = unsafe { std::str::from_utf8_unchecked(raw_status_code) };

        let parsed_status_code = StatusCode::parse(status_code).unwrap();

        let mut headers = Headers::new();
        for tmp_header in header {
            let key_range = tmp_header.0;
            let raw_key = &self.buffer[key_range.0..key_range.1];

            let value_range = tmp_header.1;
            let raw_value = &self.buffer[value_range.0..value_range.1];

            let key = unsafe { std::str::from_utf8_unchecked(raw_key) };
            let value = unsafe { std::str::from_utf8_unchecked(raw_value) };

            headers.add(key, value);
        }

        let body = &self.buffer[*header_end..];

        Some(Response::new(protocol, parsed_status_code, headers, body))
    }
}

#[test]
fn parser_parse_no_body() {
    let block = "HTTP/1.1 200 OK\r\nTest-1: Value-1\r\n\r\n";

    let mut parser = RespParser::new_capacity(4096);
    parser.block_parse(block.as_bytes());

    let mut headers = Headers::new();
    headers.add("Test-1", "Value-1");
    assert_eq!(
        Some(Response::new(
            "HTTP/1.1",
            StatusCode::OK,
            headers,
            "".as_bytes()
        )),
        parser.finish()
    );
}
#[test]
fn parser_parse_with_body() {
    let block = "HTTP/1.1 200 OK\r\nTest-1: Value-1\r\n\r\nThis is just some body";

    let mut parser = RespParser::new_capacity(4096);
    parser.block_parse(block.as_bytes());

    let mut headers = Headers::new();
    headers.add("Test-1", "Value-1");
    assert_eq!(
        Some(Response::new(
            "HTTP/1.1",
            StatusCode::OK,
            headers,
            "This is just some body".as_bytes()
        )),
        parser.finish()
    );
}
#[test]
fn parser_parse_multiple_headers_with_body() {
    let block =
        "HTTP/1.1 200 OK\r\nTest-1: Value-1\r\nTest-2: Value-2\r\n\r\nThis is just some body";
    let mut parser = RespParser::new_capacity(4096);
    parser.block_parse(block.as_bytes());

    let mut headers = Headers::new();
    headers.add("Test-1", "Value-1");
    headers.add("Test-2", "Value-2");
    assert_eq!(
        Some(Response::new(
            "HTTP/1.1",
            StatusCode::OK,
            headers,
            "This is just some body".as_bytes()
        )),
        parser.finish()
    );
}
