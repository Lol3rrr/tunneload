use crate::http::Headers;

pub fn parse_headers(raw_part: &[u8]) -> Option<(Headers, usize)> {
    let mut result = Headers::new();

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

                result.add(key, value);

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

#[test]
fn parse_valid() {
    let body = "Key-1: Test-1\r\nKey-2: Test-2\r\n\r\n";
    let mut expected = Headers::new();
    expected.add("Key-1", "Test-1");
    expected.add("Key-2", "Test-2");
    assert_eq!(Some((expected, 30)), parse_headers(body.as_bytes()));
}
