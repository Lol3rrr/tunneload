//! This handles all the parts related to Auto-TLS, which mostly revolves
//! around obtaining and managing TLS-Certificates automatically so the
//! end-user does not have to worry about it

mod acme;
use std::{
    collections::HashSet,
    sync::Arc,
    time::{Duration, SystemTime},
};

pub use acme::*;

use acme2::openssl::{
    asn1::Asn1Time,
    pkey::{PKey, Private},
    x509::X509,
};
use async_raft::NodeId;
use async_trait::async_trait;
use chrono::NaiveDateTime;

use crate::{
    configurator::{RuleList, ServiceList},
    internal_services,
};

use self::cluster::Cluster;

use super::ConfigManager;

mod challenges;
pub use challenges::{ChallengeList, ChallengeState};

mod session;
pub use session::{register_metrics, AutoSession};

mod queue;
pub use queue::{CertificateQueue, CertificateRequest};

pub mod discovery;

mod cluster;

/// Creates all the Parts needed for the Automatic-TLS stuff
pub async fn new<D>(
    env: Environment,
    contacts: Vec<String>,
    rules: RuleList,
    services: ServiceList,
    tls_config: ConfigManager,
    discover: D,
    listen_port: u16,
) -> (internal_services::ACMEHandler, AutoSession<D>)
where
    D: AutoDiscover + Send + Sync + 'static,
{
    let challenges = ChallengeList::new();

    let internal_handler = internal_services::ACMEHandler::new(challenges.clone());
    let auto_session = AutoSession::new(
        env,
        contacts,
        rules,
        services,
        tls_config,
        challenges,
        discover,
        listen_port,
    )
    .await;

    (internal_handler, auto_session)
}

/// This is responsible for finding old certificates and then trying to renew
/// them before they expire
pub async fn renew<S>(storage: Arc<S>, cert_queue: CertificateQueue, threshold: Duration)
where
    S: TLSStorage,
{
    loop {
        // Load and try to refresh certificates that are about to expire
        let certificates = storage.load_expiration_dates().await;

        let today = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let unix_timestamp = today.as_secs();

        for (domain, expire_date) in certificates {
            let chrono_threshold = chrono::Duration::from_std(threshold.clone()).unwrap();
            let adjusted_date = expire_date.checked_sub_signed(chrono_threshold).unwrap();
            let adjusted_timestamp = adjusted_date.timestamp() as u64;

            if adjusted_timestamp < unix_timestamp {
                cert_queue.request(domain);
            }
        }

        // Sleep for one hour
        tokio::time::sleep(Duration::from_secs(60 * 60)).await;
    }
}

/// This defines a uniformi interface to allow for multiple Storage-Engines
/// to be used to actually save the generated Certificates
#[async_trait]
pub trait TLSStorage {
    /// This is used to store the Private Key for the ACME-Account
    async fn store_acc_key(&self, priv_key: &PKey<Private>);
    /// This is used to load the Private Key for the ACME-Account
    async fn load_acc_key(&self) -> Option<PKey<Private>>;

    /// This simply stores the single given Certificate for the Domain
    async fn store(&self, domain: String, priv_key: &PKey<Private>, certificate: &X509);

    /// Loads all the Certificates from this Storage-Backend
    async fn load_expiration_dates(&self) -> Vec<(String, NaiveDateTime)>;

    /// Turns the given Private-Key into the byte sequence that should be stored
    fn private_key_to_bytes(key: &PKey<Private>) -> Option<Vec<u8>> {
        match key.private_key_to_pem_pkcs8() {
            Ok(d) => Some(d),
            Err(e) => {
                tracing::error!("Converting Private-Key to Data: {:?}", e);
                None
            }
        }
    }
    /// Turns the given Certificate into the byte sequence that should be stored
    fn cert_to_bytes(certificate: &X509) -> Option<Vec<u8>> {
        match certificate.to_pem() {
            Ok(d) => Some(d),
            Err(e) => {
                tracing::error!("Converting Certificate to Data: {:?}", e);
                None
            }
        }
    }
}

/// This Trait describes a basic interface that the Cluster relies upon to
/// discover other Nodes that should also be moved into the Clustser
#[async_trait]
pub trait AutoDiscover {
    /// Gets the ID of the own/current Node
    async fn get_own_id(&self) -> NodeId;

    /// Gets a Set of all currently known Nodes that are part
    /// of the Cluster
    async fn get_all_nodes(&self) -> HashSet<NodeId>;

    /// This should be the main task in which the Discovery-Mechanism
    /// runs and then updates the Cluster configuration as it discovers
    /// new Nodes for the Cluster
    async fn watch_nodes<D>(self: Arc<Self>, raft: Arc<Cluster<D>>)
    where
        D: AutoDiscover + Send + Sync + 'static;
}

#[cfg(test)]
mod mocks {
    use super::*;

    pub struct MockStorage {}

    #[async_trait]
    impl TLSStorage for MockStorage {
        async fn store(&self, _domain: String, _priv_key: &PKey<Private>, _certificate: &X509) {}
        async fn load_acc_key(&self) -> Option<PKey<Private>> {
            None
        }
        async fn store_acc_key(&self, _priv_key: &PKey<Private>) {}
        async fn load_expiration_dates(&self) -> Vec<(String, NaiveDateTime)> {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tls::auto::mocks::MockStorage;

    use super::*;

    #[test]
    fn priv_key() {
        let key = acme2::gen_rsa_private_key(4096).unwrap();

        let result = MockStorage::private_key_to_bytes(&key);
        assert!(result.is_some());

        let result_data = result.unwrap();

        let mut key_reader = std::io::BufReader::new(std::io::Cursor::new(result_data));
        match rustls_pemfile::read_one(&mut key_reader).expect("Cannot parse key data") {
            Some(rustls_pemfile::Item::RSAKey(_)) => assert!(true),
            Some(rustls_pemfile::Item::PKCS8Key(_)) => assert!(true),
            Some(rustls_pemfile::Item::X509Certificate(_)) => panic!("Received X509Certificate"),
            None => panic!("Unrecognized"),
        };
    }
}
