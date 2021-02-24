use crate::http::{HeaderKey, HeaderValue};

#[derive(Clone, Debug)]
pub struct Header<'a> {
    pub key: HeaderKey<'a>,
    pub value: HeaderValue<'a>,
}

impl PartialEq for Header<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key)
    }
}

impl<'a> Header<'a> {
    pub fn serialize(&self, buf: &mut Vec<u8>) {
        self.key.serialize(buf);
        buf.extend_from_slice(": ".as_bytes());
        self.value.serialize(buf);
        buf.extend_from_slice("\r\n".as_bytes());
    }
}

#[test]
fn compare() {
    assert_eq!(
        Header {
            key: HeaderKey::StrRef("test"),
            value: HeaderValue::StrRef("value"),
        },
        Header {
            key: HeaderKey::StrRef("test"),
            value: HeaderValue::StrRef("some other value"),
        }
    );
}

#[test]
fn serialize() {
    let header = Header {
        key: HeaderKey::StrRef("test-key"),
        value: HeaderValue::StrRef("test-value"),
    };

    let mut buf: Vec<u8> = Vec::new();
    header.serialize(&mut buf);
    assert_eq!("test-key: test-value\r\n".as_bytes(), &buf);
}
