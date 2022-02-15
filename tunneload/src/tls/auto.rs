//! This handles all the parts related to Auto-TLS, which mostly revolves
//! around obtaining and managing TLS-Certificates automatically so the
//! end-user does not have to worry about it

mod acme;
use std::{
    collections::HashSet,
    fmt::Debug,
    sync::Arc,
    time::{Duration, SystemTime},
};

pub use acme::*;

use async_raft::NodeId;
use async_trait::async_trait;

use crate::{
    configurator::{RuleList, ServiceList},
    internal_services,
};

use super::ConfigManager;

mod challenges;
pub use challenges::{ChallengeList, ChallengeState};

mod session;
pub use session::{register_metrics, AutoSession};

mod queue;
pub use queue::{CertificateQueue, CertificateRequest};

pub mod discovery;

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
    S: ::tls::TLSStorage + Debug,
{
    tracing::info!("Starting Auto-TLS with {:?}", storage);

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
                // Create the right Certificate Request to initiate the Renew-Operation
                let mut cert_req = CertificateRequest::new(domain);
                cert_req.renew_cert();

                // Actually request it
                cert_queue.custom_request(cert_req);
            }
        }

        // Sleep for one hour
        tokio::time::sleep(Duration::from_secs(60 * 60)).await;
    }
}

#[derive(Debug)]
pub enum NodeUpdateEvent {
    Add(NodeId),
    Remove(NodeId),
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
    /// new Nodes for the Cluster and also notices the removal of Nodes
    async fn watch_nodes(
        self: Arc<Self>,
        updates: tokio::sync::mpsc::UnboundedSender<NodeUpdateEvent>,
    );
}

#[cfg(test)]
mod mocks {
    use acme2::openssl::{
        pkey::{PKey, Private},
        x509::X509,
    };
    use chrono::NaiveDateTime;

    use super::*;

    use tls::TLSStorage;

    pub struct MockStorage {}

    #[async_trait]
    impl TLSStorage for MockStorage {
        async fn store(&self, _domain: String, _priv_key: PKey<Private>, _certificate: X509) {}
        async fn update(&self, _domain: String, _priv_key: PKey<Private>, _certificate: X509) {}
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

    use tls::TLSStorage;

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
