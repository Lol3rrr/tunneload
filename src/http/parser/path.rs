pub fn parse_path(raw_part: &[u8]) -> Option<(&str, usize)> {
    for (index, c) in raw_part.iter().enumerate() {
        if let b' ' = c {
            let result = std::str::from_utf8(&raw_part[0..index]).unwrap();
            return Some((result, index));
        }
    }

    None
}

#[test]
fn parse() {
    let rest_line = "/path/ HTTP/1.1\r\n";
    assert_eq!(Some(("/path/", 6)), parse_path(rest_line.as_bytes()));
}
