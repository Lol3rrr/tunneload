use crate::http::{HeaderKey, HeaderValue};

#[derive(Clone, Debug)]
struct Pair<'a> {
    key: HeaderKey<'a>,
    value: HeaderValue<'a>,
}

impl PartialEq for Pair<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Headers<'a> {
    headers: Vec<Pair<'a>>,
}

impl<'a> Headers<'a> {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
        }
    }

    pub fn add<'b, K, V>(&mut self, key: K, value: V)
    where
        'b: 'a,
        K: Into<HeaderKey<'a>>,
        V: Into<HeaderValue<'a>>,
    {
        let final_key = key.into();
        if let Some(index) = self.find(&final_key) {
            self.headers.remove(index);
        }

        self.headers.push(Pair {
            key: final_key,
            value: value.into(),
        });
    }

    fn find(&self, key: &HeaderKey<'a>) -> Option<usize> {
        for (index, pair) in self.headers.iter().enumerate() {
            if &pair.key == key {
                return Some(index);
            }
        }
        None
    }

    pub fn remove<K>(&mut self, key: K)
    where
        K: Into<HeaderKey<'a>>,
    {
        if let Some(index) = self.find(&key.into()) {
            self.headers.remove(index);
        }
    }

    pub fn get<K>(&self, key: K) -> Option<&HeaderValue<'a>>
    where
        K: Into<HeaderKey<'a>>,
    {
        match self.find(&key.into()) {
            Some(index) => Some(&self.headers.get(index).unwrap().value),
            None => None,
        }
    }

    pub fn serialize(&self, buf: &mut Vec<u8>) {
        for pair in self.headers.iter() {
            pair.key.serialize(buf);
            buf.extend_from_slice(": ".as_bytes());
            pair.value.serialize(buf);
            buf.extend_from_slice("\r\n".as_bytes());
        }
    }
}

impl<'a> Default for Headers<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn headers_add_new() {
    let mut headers = Headers::new();
    headers.add("test-key", "test-value");

    assert_eq!(
        vec![Pair {
            key: HeaderKey::StrRef("test-key"),
            value: HeaderValue::StrRef("test-value")
        }],
        headers.headers
    );
}
#[test]
fn headers_add_already_exists() {
    let mut headers = Headers::new();
    headers.add("test-key", "test-value");

    assert_eq!(
        vec![Pair {
            key: HeaderKey::StrRef("test-key"),
            value: HeaderValue::StrRef("test-value")
        }],
        headers.headers
    );

    headers.add("test-key", "other value");
    assert_eq!(
        vec![Pair {
            key: HeaderKey::StrRef("test-key"),
            value: HeaderValue::StrRef("other value")
        }],
        headers.headers
    );
}

#[test]
fn headers_remove_existing() {
    let mut headers = Headers::new();
    headers.add("test-key", "test-value");

    assert_eq!(
        vec![Pair {
            key: HeaderKey::StrRef("test-key"),
            value: HeaderValue::StrRef("test-value")
        }],
        headers.headers
    );

    headers.remove("test-key");
    assert_eq!(Vec::<Pair>::new(), headers.headers);
}
#[test]
fn headers_remove_non_existing() {
    let mut headers = Headers::new();
    headers.add("test-key", "test-value");

    assert_eq!(
        vec![Pair {
            key: HeaderKey::StrRef("test-key"),
            value: HeaderValue::StrRef("test-value")
        }],
        headers.headers
    );

    headers.remove("other-key");
    assert_eq!(
        vec![Pair {
            key: HeaderKey::StrRef("test-key"),
            value: HeaderValue::StrRef("test-value")
        }],
        headers.headers
    );
}

#[test]
fn headers_get_existing() {
    let mut headers = Headers::new();
    headers.add("test-key", "test-value");

    assert_eq!(
        vec![Pair {
            key: HeaderKey::StrRef("test-key"),
            value: HeaderValue::StrRef("test-value")
        }],
        headers.headers
    );

    assert_eq!(
        Some(&HeaderValue::StrRef("test-value")),
        headers.get("test-key")
    );
}
#[test]
fn headers_get_not_existing() {
    let mut headers = Headers::new();
    headers.add("test-key", "test-value");

    assert_eq!(
        vec![Pair {
            key: HeaderKey::StrRef("test-key"),
            value: HeaderValue::StrRef("test-value")
        }],
        headers.headers
    );

    assert_eq!(None, headers.get("other-key"));
}

#[test]
fn headers_serialize() {
    let mut headers = Headers::new();
    headers.add("test-key", "test-value");

    assert_eq!(
        vec![Pair {
            key: HeaderKey::StrRef("test-key"),
            value: HeaderValue::StrRef("test-value")
        }],
        headers.headers
    );

    let result = "test-key: test-value\r\n".as_bytes();
    let mut tmp: Vec<u8> = Vec::new();
    headers.serialize(&mut tmp);
    assert_eq!(result, &tmp);
}
