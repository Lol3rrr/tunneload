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
