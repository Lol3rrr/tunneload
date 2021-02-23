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
