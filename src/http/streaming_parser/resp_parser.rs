use crate::http::{HeaderKey, Headers, Response, StatusCode};

type ProtocolState = (usize, usize);
type StatusCodeState = (usize, usize);
type HeaderKeyState = (usize, usize);

enum ParseState {
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

#[derive(Debug)]
enum ProgressState {
    Head,
    /// The Length the Body is expected to be
    Body(usize),
    Done,
}

pub struct RespParser {
    buffer: Vec<u8>,
    body_buffer: Vec<u8>,
    state: ParseState,
    progress: ProgressState,
}

impl RespParser {
    pub fn new_capacity(head_cap: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(head_cap),
            body_buffer: Vec::new(),
            state: ParseState::Nothing,
            progress: ProgressState::Head,
        }
    }

    /// Clears the internal buffers and resets everything
    /// to the start and makes it ready to receive and parse
    /// another Response
    ///
    /// This enables the reuse of a single parser, which helps
    /// to avoid extra allocations that are not needed.
    pub fn clear(&mut self) {
        // Clears the internal buffers
        self.buffer.clear();
        self.body_buffer.clear();

        // Reset internal State to the beginning
        self.state = ParseState::Nothing;
        self.progress = ProgressState::Head;
    }

    fn parse(&mut self, byte: u8, current: usize) -> ProgressState {
        match &mut self.state {
            ParseState::Nothing if byte == b' ' => {
                let end = current;
                let n_state = ParseState::ProtocolParsed((0, end));
                self.state = n_state;
                ProgressState::Head
            }
            ParseState::ProtocolParsed(protocol) if byte == b'\r' => {
                let start = protocol.1;
                let end = current;

                let headers = Vec::with_capacity(10);
                let n_state = ParseState::HeaderKey(*protocol, (start + 1, end), end, headers);
                self.state = n_state;
                ProgressState::Head
            }
            ParseState::HeaderKey(protocol, status_code, raw_start, headers)
                if current == *raw_start + 2 && byte == b'\r' =>
            {
                let n_state = ParseState::HeadersParsed(
                    *protocol,
                    *status_code,
                    std::mem::take(headers),
                    current + 2,
                );
                self.state = n_state;
                ProgressState::Head
            }
            ParseState::HeaderKey(protocol, status_code, raw_start, headers) if byte == b':' => {
                let start = *raw_start + 2;
                let end = current;

                let n_state = ParseState::HeaderValue(
                    *protocol,
                    *status_code,
                    (start, end),
                    std::mem::take(headers),
                );
                self.state = n_state;
                ProgressState::Head
            }
            ParseState::HeaderValue(protocol, status_code, header_key, headers)
                if byte == b'\r' =>
            {
                let start = header_key.1 + 2;
                let end = current;

                headers.push((*header_key, (start, end)));
                let n_state =
                    ParseState::HeaderKey(*protocol, *status_code, end, std::mem::take(headers));
                self.state = n_state;
                ProgressState::Head
            }
            ParseState::HeadersParsed(_, _, headers, end) if current == *end - 1 => {
                // The Length the body is supposed to have
                let mut length: usize = 0;
                for raw_header_pair in headers {
                    let key_pair = raw_header_pair.0;
                    let value_pair = raw_header_pair.1;

                    let key_str =
                        std::str::from_utf8(&self.buffer[key_pair.0..key_pair.1]).unwrap();
                    if HeaderKey::StrRef(key_str) != HeaderKey::StrRef("Content-Length") {
                        continue;
                    }

                    let value_str =
                        std::str::from_utf8(&self.buffer[value_pair.0..value_pair.1]).unwrap();

                    length = value_str.parse().unwrap();
                }

                if length > 0 {
                    ProgressState::Body(length)
                } else {
                    ProgressState::Done
                }
            }
            _ => ProgressState::Head,
        }
    }

    /// Parses the given byte-chunk
    ///
    /// Returns:
    /// * `True` if the parser is done and finish can be called
    /// * `False` if it is not yet done with parsing
    /// * Some when there was still data in the given buffer, which
    /// was not consumed/used
    pub fn block_parse(&mut self, bytes: &[u8]) -> (bool, usize) {
        match self.progress {
            ProgressState::Head => {
                let start_point = self.buffer.len();
                self.buffer.reserve(bytes.len());

                for (index, tmp_byte) in bytes.iter().enumerate() {
                    self.buffer.push(*tmp_byte);
                    self.progress = self.parse(*tmp_byte, start_point + index);
                    match self.progress {
                        ProgressState::Body(length) => {
                            self.body_buffer.reserve(length);
                            return self.block_parse(&bytes[index + 1..]);
                        }
                        ProgressState::Done => {
                            return self.block_parse(&bytes[index + 1..]);
                        }
                        _ => {}
                    }
                }
                (false, 0)
            }
            ProgressState::Body(length) => {
                let left_to_read = length - self.body_buffer.len();
                if left_to_read == 0 {
                    self.progress = ProgressState::Done;
                    return self.block_parse(&[]);
                }

                let chunk_size = bytes.len();
                if left_to_read >= chunk_size {
                    self.body_buffer.extend_from_slice(bytes);
                    (self.body_buffer.len() == length, 0)
                } else {
                    self.body_buffer.extend_from_slice(&bytes[..left_to_read]);
                    self.progress = ProgressState::Done;
                    self.block_parse(&bytes[left_to_read..])
                }
            }
            ProgressState::Done => (true, bytes.len()),
        }
    }

    pub fn finish<'a, 'b>(&'a mut self) -> Option<Response<'b>>
    where
        'a: 'b,
    {
        let (protocol, status_code, header) = match &self.state {
            ParseState::HeadersParsed(p, stc, h, _) => (p, stc, h),
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

        Some(Response::new(
            protocol,
            parsed_status_code,
            headers,
            std::mem::take(&mut self.body_buffer),
        ))
    }
}

#[test]
fn parser_parse_no_body() {
    let block = "HTTP/1.1 200 OK\r\nTest-1: Value-1\r\n\r\n";

    let mut parser = RespParser::new_capacity(1024);
    assert_eq!((true, 0), parser.block_parse(block.as_bytes()));

    let mut headers = Headers::new();
    headers.add("Test-1", "Value-1");
    assert_eq!(
        Some(Response::new(
            "HTTP/1.1",
            StatusCode::OK,
            headers,
            "".as_bytes().to_vec()
        )),
        parser.finish()
    );
}
#[test]
fn parser_parse_with_body() {
    let block = "HTTP/1.1 200 OK\r\nContent-Length: 22\r\n\r\nThis is just some body";

    let mut parser = RespParser::new_capacity(1024);
    assert_eq!((true, 0), parser.block_parse(block.as_bytes()));

    let mut headers = Headers::new();
    headers.add("Content-Length", "22");
    assert_eq!(
        Some(Response::new(
            "HTTP/1.1",
            StatusCode::OK,
            headers,
            "This is just some body".as_bytes().to_vec()
        )),
        parser.finish()
    );
}
#[test]
fn parser_parse_multiple_headers_with_body() {
    let block =
        "HTTP/1.1 200 OK\r\nTest-1: Value-1\r\nContent-Length: 22\r\n\r\nThis is just some body";
    let mut parser = RespParser::new_capacity(1024);
    assert_eq!((true, 0), parser.block_parse(block.as_bytes()));

    let mut headers = Headers::new();
    headers.add("Test-1", "Value-1");
    headers.add("Content-Length", "22");
    assert_eq!(
        Some(Response::new(
            "HTTP/1.1",
            StatusCode::OK,
            headers,
            "This is just some body".as_bytes().to_vec()
        )),
        parser.finish()
    );
}
#[test]
fn parser_parse_multiple_headers_with_body_longer_than_told() {
    let block =
        "HTTP/1.1 200 OK\r\nTest-1: Value-1\r\nContent-Length: 10\r\n\r\nThis is just some body";
    let mut parser = RespParser::new_capacity(1024);
    assert_eq!((true, 12), parser.block_parse(block.as_bytes()));

    let mut headers = Headers::new();
    headers.add("Test-1", "Value-1");
    headers.add("Content-Length", "10");
    assert_eq!(
        Some(Response::new(
            "HTTP/1.1",
            StatusCode::OK,
            headers,
            "This is ju".as_bytes().to_vec()
        )),
        parser.finish()
    );
}
