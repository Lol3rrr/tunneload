#[derive(Clone, Debug, PartialEq)]
pub struct Service {
    address: String,
}

impl Service {
    pub fn new(dest: String) -> Self {
        Self { address: dest }
    }

    pub fn address(&self) -> &str {
        &self.address
    }
}
