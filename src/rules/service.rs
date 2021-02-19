#[derive(Clone, Debug)]
pub struct Service {
    addresses: Vec<String>,
    current: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl PartialEq for Service {
    fn eq(&self, other: &Self) -> bool {
        self.addresses == other.addresses
    }
}

impl Service {
    pub fn new(destinations: Vec<String>) -> Self {
        Self {
            addresses: destinations,
            current: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    pub fn addresses(&self) -> &[String] {
        &self.addresses
    }

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
}

#[test]
fn round_robin_0_entries() {
    let tmp = Service::new(vec![]);

    assert_eq!(None, tmp.round_robin());
}
#[test]
fn round_robin_1_entry() {
    let tmp = Service::new(vec!["test1".to_owned()]);

    assert_eq!(Some("test1"), tmp.round_robin());
    assert_eq!(Some("test1"), tmp.round_robin());
}
#[test]
fn round_robin_2_entries() {
    let tmp = Service::new(vec!["test1".to_owned(), "test2".to_owned()]);

    assert_eq!(Some("test1"), tmp.round_robin());
    assert_eq!(Some("test2"), tmp.round_robin());
}
