use std::fmt::{Display, Formatter};

use general::{Group, Name};
use general_traits::{ConfigItem, DefaultConfig};

use serde::Serialize;

/// The Error returned by the Service when it fails to establish
/// an outgoing connection
#[derive(Debug)]
pub enum ConnectError {
    /// The Service did not contain any Target-Endpoints
    NoEndpoint,
    /// There was an IO-related Error when establishing the connection
    IO(tokio::io::Error),
}
impl Display for ConnectError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            ConnectError::NoEndpoint => write!(f, "Service contains no Endpoint-Entry"),
            ConnectError::IO(ref e) => write!(f, "IO-Error: {}", e),
        }
    }
}

/// A Service represents a Collection of final IP-Addresses
/// that can receive Requests
#[derive(Debug, Serialize)]
pub struct Service {
    name: Name,
    addresses: Vec<String>,
    current: std::sync::atomic::AtomicUsize,
}

impl Clone for Service {
    fn clone(&self) -> Self {
        Service::new(self.name.clone(), self.addresses.clone())
    }
}

impl PartialEq for Service {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Service {
    /// Creates a New Service instance with the given Name and Destinations
    pub fn new(name: Name, destinations: Vec<String>) -> Self {
        Self {
            name,
            addresses: destinations,
            current: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Returns whether or not the Service is an internal
    /// service
    pub fn is_internal(&self) -> bool {
        matches!(self.name.group(), Group::Internal)
    }

    /// Returns all addresses associated with the Service
    pub fn addresses(&self) -> &[String] {
        &self.addresses
    }

    /// Returns the number of Addresses for this
    /// service
    pub fn address_count(&self) -> usize {
        self.addresses.len()
    }

    /// Gets the next Address to be used for a request
    pub fn round_robin(&self) -> Option<&str> {
        let length = self.addresses.len();
        if length == 0 {
            return None;
        }
        let index = self
            .current
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % length;
        Some(
            self.addresses
                .get(index)
                .expect("The loaded Index is always in bounds because of the Module"),
        )
    }

    /// Automatically gets the next Address from the Service
    /// using `round_robin` and then connects to it
    pub async fn connect(&self) -> Result<tokio::net::TcpStream, ConnectError> {
        let address = match self.round_robin() {
            Some(a) => a,
            None => {
                return Err(ConnectError::NoEndpoint);
            }
        };

        match tokio::net::TcpStream::connect(address).await {
            Ok(c) => Ok(c),
            Err(e) => Err(ConnectError::IO(e)),
        }
    }
}

impl ConfigItem for Service {
    fn name(&self) -> &Name {
        &self.name
    }
}
impl DefaultConfig for Service {
    fn default_name(name: Name) -> Self {
        Self {
            name,
            addresses: Vec::new(),
            current: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use general::Group;

    use super::*;

    #[test]
    fn is_internal_true() {
        let tmp = Service::new(Name::new("test", Group::Internal), vec![]);
        assert_eq!(true, tmp.is_internal());
    }
    #[test]
    fn is_internal_false() {
        let tmp = Service::new(Name::new("test", Group::File {}), vec![]);
        assert_eq!(false, tmp.is_internal());

        let tmp = Service::new(
            Name::new(
                "test",
                Group::Kubernetes {
                    namespace: "ns".to_owned(),
                },
            ),
            vec![],
        );
        assert_eq!(false, tmp.is_internal());
    }

    #[test]
    fn round_robin_0_entries() {
        let tmp = Service::new(Name::new("test", Group::Internal), vec![]);

        assert_eq!(None, tmp.round_robin());
    }
    #[test]
    fn round_robin_1_entry() {
        let tmp = Service::new(Name::new("test", Group::Internal), vec!["test1".to_owned()]);

        assert_eq!(Some("test1"), tmp.round_robin());
        assert_eq!(Some("test1"), tmp.round_robin());
    }
    #[test]
    fn round_robin_2_entries() {
        let tmp = Service::new(
            Name::new("test", Group::Internal),
            vec!["test1".to_owned(), "test2".to_owned()],
        );

        assert_eq!(Some("test1"), tmp.round_robin());
        assert_eq!(Some("test2"), tmp.round_robin());
    }

    #[test]
    fn partial_eq_same() {
        assert_eq!(
            Service::new(Name::new("test-1", Group::Internal), vec![]),
            Service::new(Name::new("test-1", Group::Internal), vec![])
        );
    }
    #[test]
    fn partial_eq_different_capitalization() {
        assert_ne!(
            Service::new(Name::new("TeSt-1", Group::Internal), vec![]),
            Service::new(Name::new("test-1", Group::Internal), vec![])
        );
    }
    #[test]
    fn partial_eq_different() {
        assert_ne!(
            Service::new(Name::new("test-1", Group::Internal), vec![]),
            Service::new(Name::new("test-2", Group::Internal), vec![])
        );
    }
}
