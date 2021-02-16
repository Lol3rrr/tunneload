#[derive(Debug, PartialEq, Clone)]
pub struct Headers<'a> {
    headers: std::collections::BTreeMap<&'a str, &'a str>,
}

impl<'a> Headers<'a> {
    pub fn new() -> Self {
        Self {
            headers: std::collections::BTreeMap::new(),
        }
    }

    pub fn add<'b>(&mut self, key: &'b str, value: &'b str)
    where
        'b: 'a,
    {
        self.headers.insert(key, value);
    }

    pub fn remove(&'a mut self, key: &str) -> Option<&'a str> {
        self.headers.remove(key)
    }

    pub fn get(&'a self, key: &str) -> Option<&&str> {
        self.headers.get(key)
    }

    pub fn serialize(&self, buf: &mut Vec<u8>) {
        for (key, value) in self.headers.iter() {
            buf.extend_from_slice(key.as_bytes());
            buf.extend_from_slice(": ".as_bytes());
            buf.extend_from_slice(value.as_bytes());
            buf.extend_from_slice("\r\n".as_bytes());
        }
    }
}
