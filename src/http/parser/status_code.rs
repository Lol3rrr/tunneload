use crate::http::StatusCode;

pub fn parse_status_code(raw: &[u8]) -> Option<(StatusCode, usize)> {
    for (index, c) in raw.iter().enumerate() {
        if let b'\r' = c {
            let tmp = std::str::from_utf8(&raw[0..index]).unwrap();
            println!("TMP: '{}'", tmp);
            let result = StatusCode::parse(tmp).unwrap();
            return Some((result, index));
        }
    }

    None
}

#[test]
fn parse_valid() {
    let line = "200 OK\r\n";
    assert_eq!(
        Some((StatusCode::OK, 6)),
        parse_status_code(line.as_bytes())
    );
}
