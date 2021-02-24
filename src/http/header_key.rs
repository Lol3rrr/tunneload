/// Allows the HeaderKey to take the form of a variety of different
/// valid Types, mostly related to their lifetimes.
/// This however also gives more control over how they are compared
/// to each other, ignoring case in this case
///
/// ```rust
/// use tunneload::http::HeaderKey;
///
/// assert_eq!(HeaderKey::StrRef("TeSt"), HeaderKey::StrRef("test"));
/// ```
#[derive(Debug, Clone)]
pub enum HeaderKey<'a> {
    StrRef(&'a str),
    Str(String),
}

impl<'a> Into<HeaderKey<'a>> for &'a str {
    fn into(self) -> HeaderKey<'a> {
        HeaderKey::StrRef(self)
    }
}
impl<'a> Into<HeaderKey<'a>> for String {
    fn into(self) -> HeaderKey<'a> {
        HeaderKey::Str(self)
    }
}

impl<'a> HeaderKey<'a> {
    pub fn serialize(&self, buf: &mut Vec<u8>) {
        match *self {
            Self::StrRef(ref value) => {
                buf.extend_from_slice(value.as_bytes());
            }
            Self::Str(ref value) => {
                buf.extend_from_slice(value.as_bytes());
            }
        }
    }
}

impl AsRef<str> for HeaderKey<'_> {
    fn as_ref(&self) -> &str {
        match *self {
            Self::Str(ref value) => &value,
            Self::StrRef(ref value) => value,
        }
    }
}

impl PartialEq for HeaderKey<'_> {
    fn eq(&self, other: &Self) -> bool {
        caseless::default_caseless_match_str(self.as_ref(), other.as_ref())
    }
}

#[test]
fn equals_ignore_case() {
    assert_eq!(HeaderKey::StrRef("test"), HeaderKey::StrRef("test"));
    assert_eq!(HeaderKey::StrRef("TEST"), HeaderKey::StrRef("test"));
    assert_eq!(HeaderKey::StrRef("TeSt"), HeaderKey::StrRef("test"));
}

#[test]
fn serialize_str() {
    let mut result: Vec<u8> = Vec::new();
    HeaderKey::Str("test-key".to_owned()).serialize(&mut result);

    assert_eq!("test-key".as_bytes(), &result);
}
#[test]
fn serialize_str_ref() {
    let mut result: Vec<u8> = Vec::new();
    HeaderKey::StrRef("test-key").serialize(&mut result);

    assert_eq!("test-key".as_bytes(), &result);
}
