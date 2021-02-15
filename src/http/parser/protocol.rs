pub fn parse_protocol(raw_part: &[u8]) -> Option<(&str, usize)> {
    for (index, c) in raw_part.iter().enumerate() {
        if b'\r' == *c || b' ' == *c {
            let result = std::str::from_utf8(&raw_part[0..index]).unwrap();
            return Some((result, index));
        }
    }

    None
}

#[test]
fn parse_valid_eof() {
    let line = "HTTP/1.1\r\n";
    assert_eq!(Some(("HTTP/1.1", 8)), parse_protocol(line.as_bytes()));
}
#[test]
fn parse_valid_space() {
    let line = "HTTP/1.1 200 OK\r\n";
    assert_eq!(Some(("HTTP/1.1", 8)), parse_protocol(line.as_bytes()));
}
