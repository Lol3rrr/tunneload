/// A single Header-Pair
#[derive(Debug, PartialEq)]
pub struct Header<'a> {
    key: &'a str,
    value: &'a str,
}

impl<'a> Header<'a> {
    /// Creates a new Header using the given Key and value
    pub fn new(key: &'a str, value: &'a str) -> Self {
        Self { key, value }
    }
}
