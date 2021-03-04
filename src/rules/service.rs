/// A single Service that can receive Requests
#[derive(Debug)]
pub struct Service {
    name: String,
    addresses: Vec<String>,
    current: std::sync::atomic::AtomicUsize,
}

impl PartialEq for Service {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Service {
    /// Creates a New Service instance with the given Name and Destinations
    pub fn new<S>(name: S, destinations: Vec<String>) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            addresses: destinations,
            current: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Returns the Name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns all addresses associated with the Service
    pub fn addresses(&self) -> &[String] {
        &self.addresses
    }

    /// Gets the next Address to be used for a request
    pub fn round_robin(&self) -> Option<&str> {
        let length = self.addresses.len();
        if length == 0 {
            return None;
        }
        let index = self.current.load(std::sync::atomic::Ordering::Relaxed) % length;
        self.current
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Some(&self.addresses.get(index).unwrap())
    }

    /// Automatically gets the next Address from the Service
    /// using `round_robin` and then connects to it
    pub async fn connect(&self) -> Option<tokio::net::TcpStream> {
        let address = match self.round_robin() {
            Some(a) => a,
            None => {
                return None;
            }
        };

        match tokio::net::TcpStream::connect(address).await {
            Ok(c) => Some(c),
            Err(_) => None,
        }
    }
}

#[test]
fn round_robin_0_entries() {
    let tmp = Service::new("test", vec![]);

    assert_eq!(None, tmp.round_robin());
}
#[test]
fn round_robin_1_entry() {
    let tmp = Service::new("test", vec!["test1".to_owned()]);

    assert_eq!(Some("test1"), tmp.round_robin());
    assert_eq!(Some("test1"), tmp.round_robin());
}
#[test]
fn round_robin_2_entries() {
    let tmp = Service::new("test", vec!["test1".to_owned(), "test2".to_owned()]);

    assert_eq!(Some("test1"), tmp.round_robin());
    assert_eq!(Some("test2"), tmp.round_robin());
}

#[test]
fn partial_eq_same() {
    assert_eq!(
        Service::new("test-1", vec![]),
        Service::new("test-1", vec![])
    );
}
#[test]
fn partial_eq_different_capitalization() {
    assert_ne!(
        Service::new("TeSt-1", vec![]),
        Service::new("test-1", vec![])
    );
}
#[test]
fn partial_eq_different() {
    assert_ne!(
        Service::new("test-1", vec![]),
        Service::new("test-2", vec![])
    );
}
