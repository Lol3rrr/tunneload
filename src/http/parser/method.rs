use crate::http::Method;

pub fn parse_method(raw_part: &[u8]) -> Option<(Method, usize)> {
    for (index, c) in raw_part.iter().enumerate() {
        if let b' ' = c {
            match Method::parse(std::str::from_utf8(&raw_part[0..index]).unwrap()) {
                Some(s) => {
                    return Some((s, index));
                }
                None => {
                    return None;
                }
            };
        }
    }

    None
}

#[test]
fn parse_line() {
    let line = "GET /path/ HTTP/1.1\r\n";
    assert_eq!(Some((Method::GET, 3)), parse_method(line.as_bytes()));
}
