#[derive(Debug, PartialEq, Clone)]
pub enum HeaderValue<'a> {
    StrRef(&'a str),
    Str(String),
    NumberUsize(usize),
}

impl<'a> Into<HeaderValue<'a>> for &'a str {
    fn into(self) -> HeaderValue<'a> {
        HeaderValue::StrRef(self)
    }
}
impl<'a> Into<HeaderValue<'a>> for String {
    fn into(self) -> HeaderValue<'a> {
        HeaderValue::Str(self)
    }
}
impl<'a> Into<HeaderValue<'a>> for usize {
    fn into(self) -> HeaderValue<'a> {
        HeaderValue::NumberUsize(self)
    }
}

impl<'a> HeaderValue<'a> {
    pub fn serialize(&self, buf: &mut Vec<u8>) {
        match *self {
            Self::StrRef(ref value) => {
                buf.extend_from_slice(value.as_bytes());
            }
            Self::Str(ref value) => {
                buf.extend_from_slice(value.as_bytes());
            }
            Self::NumberUsize(ref value) => {
                buf.extend_from_slice(value.to_string().as_bytes());
            }
        }
    }
}

impl PartialEq<std::string::String> for HeaderValue<'_> {
    fn eq(&self, other: &std::string::String) -> bool {
        match *self {
            Self::StrRef(ref value) => value == other,
            Self::Str(ref value) => value == other,
            _ => false,
        }
    }
}

#[test]
fn serialize_str() {
    let mut result: Vec<u8> = Vec::new();
    HeaderValue::Str("test-value".to_owned()).serialize(&mut result);

    assert_eq!("test-value".as_bytes(), &result);
}
#[test]
fn serialize_str_ref() {
    let mut result: Vec<u8> = Vec::new();
    HeaderValue::StrRef("test-value").serialize(&mut result);

    assert_eq!("test-value".as_bytes(), &result);
}
#[test]
fn serialize_number_usize() {
    let mut result: Vec<u8> = Vec::new();
    HeaderValue::NumberUsize(80).serialize(&mut result);

    assert_eq!("80".as_bytes(), &result);
}
