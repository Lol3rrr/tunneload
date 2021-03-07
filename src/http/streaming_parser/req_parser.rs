use crate::http::streaming_parser::{ParseError, ParseResult};
use crate::http::{HeaderKey, Headers, Method, Request};

type MethodState = (usize, usize);
type PathState = (usize, usize);
type ProtocolState = (usize, usize);
type HeaderKeyState = (usize, usize);

enum State {
    Nothing,
    MethodParsed(MethodState),
    PathParsed(MethodState, PathState),
    HeaderKey(MethodState, PathState, ProtocolState, usize),
    HeaderValue(MethodState, PathState, ProtocolState, HeaderKeyState),
    HeadersParsed(MethodState, PathState, ProtocolState, usize),
}

enum ProgressState {
    Head,
    Body(usize),
    Done,
}

pub struct ReqParser {
    buffer: Vec<u8>,
    body_buffer: Vec<u8>,
    headers_buf: Vec<((usize, usize), (usize, usize))>,
    state: State,
    progress: ProgressState,
}

impl ReqParser {
    /// Creates a new Request-Parser with the given
    /// capacity as its pre-reserved capacity to store
    /// the Head of the Request
    pub fn new_capacity(cap: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(cap),
            body_buffer: Vec::new(),
            headers_buf: Vec::with_capacity(20),
            state: State::Nothing,
            progress: ProgressState::Head,
        }
    }

    /// Clears the internal Buffers and resets everything
    /// to be ready to receive and parse a new request
    ///
    /// This should be the prefered way to parse mulitple
    /// sequential requests, as this avoids extra allocations
    pub fn clear(&mut self) {
        // Clearing out the buffers of the previous request
        // without needing to perform any new allocations
        self.buffer.clear();
        self.body_buffer.clear();
        self.headers_buf.clear();

        // Sets the Progress and internal State back to the
        // beginning
        self.state = State::Nothing;
        self.progress = ProgressState::Head;
    }

    fn parse(&mut self, byte: u8, current: usize) -> ProgressState {
        match &mut self.state {
            State::Nothing if byte == b' ' => {
                let end = current;
                let n_state = State::MethodParsed((0, end));
                self.state = n_state;
                ProgressState::Head
            }
            State::MethodParsed(method) if byte == b' ' => {
                let start = method.1;
                let end = current;

                let n_state = State::PathParsed(*method, (start + 1, end));
                self.state = n_state;
                ProgressState::Head
            }
            State::PathParsed(method, path) if byte == b'\r' => {
                let start = path.1;
                let end = current;

                let n_state = State::HeaderKey(*method, *path, (start + 1, end), end);
                self.state = n_state;
                ProgressState::Head
            }
            State::HeaderKey(method, path, protocol, raw_start)
                if current == *raw_start + 2 && byte == b'\r' =>
            {
                let n_state = State::HeadersParsed(*method, *path, *protocol, current + 2);
                self.state = n_state;
                ProgressState::Head
            }
            State::HeaderKey(method, path, protocol, raw_start)
                if byte == b':' && *raw_start + 2 <= current =>
            {
                let start = *raw_start + 2;
                let end = current;

                let n_state = State::HeaderValue(*method, *path, *protocol, (start, end));
                self.state = n_state;
                ProgressState::Head
            }
            State::HeaderValue(method, path, protocol, header_key)
                if byte == b'\r' && header_key.1 + 2 <= current =>
            {
                let start = header_key.1 + 2;
                let end = current;

                self.headers_buf.push((*header_key, (start, end)));
                let n_state = State::HeaderKey(*method, *path, *protocol, end);
                self.state = n_state;
                ProgressState::Head
            }
            State::HeadersParsed(_, _, _, end) if current == *end - 1 => {
                // The Length the body is supposed to have
                let mut length: usize = 0;
                for raw_header_pair in self.headers_buf.iter() {
                    let key_pair = raw_header_pair.0;
                    let value_pair = raw_header_pair.1;

                    let key_str = match std::str::from_utf8(&self.buffer[key_pair.0..key_pair.1]) {
                        Ok(k) => k,
                        Err(_) => {
                            continue;
                        }
                    };
                    if HeaderKey::StrRef(key_str) != HeaderKey::StrRef("Content-Length") {
                        continue;
                    }

                    let value_str =
                        match std::str::from_utf8(&self.buffer[value_pair.0..value_pair.1]) {
                            Ok(v) => v,
                            Err(_) => {
                                continue;
                            }
                        };

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

    /// Returns a touple that stands for (done, data-left-in-buffer)
    ///
    /// Explanation:
    /// * `done`: True if the request has been fully received and parsed
    /// * `data-left-in-buffer`: The Amount of bytes at the end of the given
    /// slice that were unused
    pub fn block_parse(&mut self, bytes: &[u8]) -> (bool, Option<usize>) {
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

                (false, None)
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
                    (self.body_buffer.len() == length, None)
                } else {
                    self.body_buffer.extend_from_slice(&bytes[..left_to_read]);
                    self.progress = ProgressState::Done;
                    self.block_parse(&bytes[left_to_read..])
                }
            }
            ProgressState::Done => {
                let length = bytes.len();
                let rest = (length > 0).then(|| length);

                (true, rest)
            }
        }
    }

    pub fn finish<'a, 'b>(&'a self) -> ParseResult<Request<'b>>
    where
        'a: 'b,
    {
        let (method, path, protocol) = match &self.state {
            State::HeadersParsed(m, p, pt, _) => (m, p, pt),
            State::Nothing => {
                return Err(ParseError::MissingMethod);
            }
            State::MethodParsed(_) => {
                return Err(ParseError::MissingPath);
            }
            State::PathParsed(_, _) => {
                return Err(ParseError::MissingProtocol);
            }
            State::HeaderKey(_, _, _, _) | State::HeaderValue(_, _, _, _) => {
                return Err(ParseError::MissingHeaders);
            }
        };

        let raw_method = &self.buffer[method.0..method.1];
        let raw_path = &self.buffer[path.0..path.1];
        let raw_protocol = &self.buffer[protocol.0..protocol.1];

        let method = unsafe { std::str::from_utf8_unchecked(raw_method) };
        let path = unsafe { std::str::from_utf8_unchecked(raw_path) };
        let protocol = unsafe { std::str::from_utf8_unchecked(raw_protocol) };

        let parsed_method = match Method::parse(method) {
            Some(m) => m,
            None => return Err(ParseError::MissingMethod),
        };

        let header_count = self.headers_buf.len();
        let mut headers = Headers::with_capacity(header_count);
        for tmp_header in self.headers_buf.iter() {
            let key_range = tmp_header.0;
            let raw_key = &self.buffer[key_range.0..key_range.1];

            let value_range = tmp_header.1;
            let raw_value = &self.buffer[value_range.0..value_range.1];

            let key = unsafe { std::str::from_utf8_unchecked(raw_key) };
            let value = unsafe { std::str::from_utf8_unchecked(raw_value) };

            headers.add(key, value);
        }

        let body = &self.body_buffer;

        Ok(Request::new(protocol, parsed_method, path, headers, body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_parse_no_body() {
        let block = "GET /path/ HTTP/1.1\r\nTest-1: Value-1\r\n\r\n";

        let mut parser = ReqParser::new_capacity(4096);
        assert_eq!((true, None), parser.block_parse(block.as_bytes()));

        let mut headers = Headers::new();
        headers.add("Test-1", "Value-1");
        assert_eq!(
            Ok(Request::new(
                "HTTP/1.1",
                Method::GET,
                "/path/",
                headers,
                "".as_bytes()
            )),
            parser.finish()
        );
    }
    #[test]
    fn parser_parse_with_body() {
        let block = "GET /path/ HTTP/1.1\r\nContent-Length: 22\r\n\r\nThis is just some body";

        let mut parser = ReqParser::new_capacity(4096);
        assert_eq!((true, None), parser.block_parse(block.as_bytes()));

        let mut headers = Headers::new();
        headers.add("Content-Length", "22");
        assert_eq!(
            Ok(Request::new(
                "HTTP/1.1",
                Method::GET,
                "/path/",
                headers,
                "This is just some body".as_bytes()
            )),
            parser.finish()
        );
    }
    #[test]
    fn parser_parse_multiple_headers_with_body() {
        let block =
        "GET /path/ HTTP/1.1\r\nContent-Length: 22\r\nTest-2: Value-2\r\n\r\nThis is just some body";
        let mut parser = ReqParser::new_capacity(4096);
        assert_eq!((true, None), parser.block_parse(block.as_bytes()));

        let mut headers = Headers::new();
        headers.add("Content-Length", "22");
        headers.add("Test-2", "Value-2");
        assert_eq!(
            Ok(Request::new(
                "HTTP/1.1",
                Method::GET,
                "/path/",
                headers,
                "This is just some body".as_bytes()
            )),
            parser.finish()
        );
    }
    #[test]
    fn parser_parse_multiple_headers_with_body_set_shorter() {
        let block =
        "GET /path/ HTTP/1.1\r\nContent-Length: 10\r\nTest-2: Value-2\r\n\r\nThis is just some body";
        let mut parser = ReqParser::new_capacity(4096);
        assert_eq!((true, Some(12)), parser.block_parse(block.as_bytes()));

        let mut headers = Headers::new();
        headers.add("Content-Length", "10");
        headers.add("Test-2", "Value-2");
        assert_eq!(
            Ok(Request::new(
                "HTTP/1.1",
                Method::GET,
                "/path/",
                headers,
                "This is ju".as_bytes()
            )),
            parser.finish()
        );
    }

    #[test]
    fn parser_missing_method() {
        let block = "";
        let mut parser = ReqParser::new_capacity(4096);
        assert_eq!((false, None), parser.block_parse(block.as_bytes()));

        assert_eq!(Err(ParseError::MissingMethod), parser.finish());
    }
    #[test]
    fn parser_missing_path() {
        let block = "GET ";
        let mut parser = ReqParser::new_capacity(4096);
        assert_eq!((false, None), parser.block_parse(block.as_bytes()));

        assert_eq!(Err(ParseError::MissingPath), parser.finish());
    }
    #[test]
    fn parser_missing_protocol() {
        let block = "GET /path/ ";
        let mut parser = ReqParser::new_capacity(4096);
        assert_eq!((false, None), parser.block_parse(block.as_bytes()));

        assert_eq!(Err(ParseError::MissingProtocol), parser.finish());
    }
    #[test]
    fn parser_missing_headers() {
        let block = "GET /path/ HTTP/1.1\r\n";
        let mut parser = ReqParser::new_capacity(4096);
        assert_eq!((false, None), parser.block_parse(block.as_bytes()));

        assert_eq!(Err(ParseError::MissingHeaders), parser.finish());
    }

    #[test]
    fn parser_fuzzing_bug_0() {
        let block = vec![
            13, 36, 32, 32, 36, 13, 58, 32, 32, 13, 36, 13, 36, 32, 32, 36, 13, 58, 36, 32, 32, 36,
            13, 58, 1,
        ];
        let mut parser = ReqParser::new_capacity(2048);

        assert_eq!((false, None), parser.block_parse(&block));
    }
    #[test]
    fn parser_fuzzing_bug_1() {
        let block = vec![
            84, 82, 65, 67, 69, 32, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 85, 58, 13, 36, 36, 58, 93, 0, 36,
            32, 32, 13, 213, 58, 13, 36, 36, 58, 13, 36, 32, 32, 13, 85, 58, 13, 36, 36, 58, 93, 0,
            36, 32, 32, 13, 213, 58, 13, 36, 36, 58, 13, 64, 13, 36, 64,
        ];
        let mut parser = ReqParser::new_capacity(2048);

        assert_eq!((true, Some(1)), parser.block_parse(&block));
        // This is somehow a valid request according to my parser
        assert_eq!(true, parser.finish().is_ok());
    }
}
