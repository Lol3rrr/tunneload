pub fn parse_headers(
    raw_part: &[u8],
) -> Option<(std::collections::BTreeMap<String, String>, usize)> {
    let mut result = std::collections::BTreeMap::new();

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

                result.insert(key.to_owned(), value.to_owned());

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
    let mut expected = std::collections::BTreeMap::new();
    expected.insert("Key-1".to_owned(), "Test-1".to_owned());
    expected.insert("Key-2".to_owned(), "Test-2".to_owned());
    assert_eq!(Some((expected, 30)), parse_headers(body.as_bytes()));
}
