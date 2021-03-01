use crate::http::Chunk;

enum ParseState {
    Size,
    Content(usize),
}

pub struct ChunkParser {
    state: ParseState,
    head: Vec<u8>,
    body: Vec<u8>,
}

impl ChunkParser {
    pub fn new() -> ChunkParser {
        Self {
            state: ParseState::Size,
            head: Vec::with_capacity(16),
            body: Vec::new(),
        }
    }

    /// Clears and resets the internal State to allow
    /// the parser to accept, receive and parse a new
    /// chunk without using up extra allocations,
    pub fn clear(&mut self) {
        // Clear the internal buffer
        self.head.clear();
        self.body.clear();

        // Reset the internal state
        self.state = ParseState::Size;
    }

    /// Parses and handles each individual byte
    fn parse_size(&mut self) -> Option<usize> {
        match self.head.last() {
            Some(byte) if *byte != b'\n' => return None,
            None => return None,
            _ => {}
        };

        self.head.pop();
        self.head.pop();

        let head_str = match std::str::from_utf8(&self.head) {
            Ok(t) => t,
            Err(_) => {
                return None;
            }
        };

        let result = match usize::from_str_radix(head_str, 16) {
            Ok(n) => n,
            Err(_) => {
                return None;
            }
        };

        Some(result)
    }

    /// Parses the given Block of data,
    /// returns the size it parsed as well
    /// as if it is done with parsing
    ///
    /// Returns:
    /// * If it is done and the `finish` function should be called
    /// * The amount of data that is still left in the Buffer (at the end)
    pub fn block_parse(&mut self, data: &[u8]) -> (bool, usize) {
        match self.state {
            ParseState::Size => {
                for (index, tmp) in data.iter().enumerate() {
                    self.head.push(*tmp);
                    if let Some(n_size) = self.parse_size() {
                        let n_state = ParseState::Content(n_size);
                        self.state = n_state;
                        self.body.reserve(n_size);
                        return self.block_parse(&data[index + 1..]);
                    }
                }
                (false, 0)
            }
            ParseState::Content(size) => {
                let body_length = self.body.len();
                let left_to_read = size - body_length;

                let data_length = data.len();
                let read_size = std::cmp::min(left_to_read, data_length);

                self.body.extend_from_slice(&data[..read_size]);
                (
                    self.body.len() >= size,
                    data_length.saturating_sub(read_size + 2),
                )
            }
        }
    }

    /// Finishes the Parsing and returns the
    /// finsihed Chunk
    pub fn finish(&mut self) -> Option<Chunk> {
        let size = match self.state {
            ParseState::Size => return None,
            ParseState::Content(s) => s,
        };

        let body = std::mem::take(&mut self.body);
        Some(Chunk::new(size, body))
    }
}

impl Default for ChunkParser {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn parse_valid_chunk() {
    let content = "9\r\nDeveloper\r\n".as_bytes();

    let mut parser = ChunkParser::new();
    assert_eq!((true, 0), parser.block_parse(&content));

    assert_eq!(
        Some(Chunk::new(9, "Developer".as_bytes().to_vec())),
        parser.finish()
    );
}
#[test]
fn parse_zero_sized_chunk() {
    let content = "0\r\n\r\n".as_bytes();

    let mut parser = ChunkParser::new();
    assert_eq!((true, 0), parser.block_parse(&content));

    assert_eq!(Some(Chunk::new(0, "".as_bytes().to_vec())), parser.finish());
}

#[test]
fn parse_valid_chunk_that_contains_other() {
    let content = "9\r\nDeveloper\r\n0\r\n\r\n".as_bytes();

    let mut parser = ChunkParser::new();
    assert_eq!((true, 5), parser.block_parse(&content));

    assert_eq!(
        Some(Chunk::new(9, "Developer".as_bytes().to_vec())),
        parser.finish()
    );
}

#[test]
fn parse_valid_multiple_chunks() {
    let mut parser = ChunkParser::new();
    assert_eq!((false, 0), parser.block_parse(&"9\r\nDevel".as_bytes()));
    assert_eq!((true, 0), parser.block_parse(&"oper\r\n".as_bytes()));

    assert_eq!(
        Some(Chunk::new(9, "Developer".as_bytes().to_vec())),
        parser.finish()
    );
}
