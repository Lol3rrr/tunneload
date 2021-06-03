//! This handles all the parts related to Auto-TLS, which mostly revolves
//! around obtaining and managing TLS-Certificates automatically so the
//! end-user does not have to worry about it

mod acme;
pub use acme::*;

use acme2::openssl::{
    pkey::{PKey, Private},
    x509::X509,
};
use async_trait::async_trait;

use crate::{
    configurator::{RuleList, ServiceList},
    internal_services,
};

use super::ConfigManager;
mod consensus;

mod challenges;
pub use challenges::{ChallengeList, ChallengeState};

mod session;
pub use session::AutoSession;

mod queue;
pub use queue::{CertificateQueue, CertificateRequest};

/// Creates all the Parts needed for the Automatic-TLS stuff
pub async fn new(
    env: Environment,
    contacts: Vec<String>,
    rules: RuleList,
    services: ServiceList,
    tls_config: ConfigManager,
) -> (internal_services::ACMEHandler, AutoSession) {
    let challenges = ChallengeList::new();

    let internal_handler = internal_services::ACMEHandler::new(challenges.clone());
    let auto_session =
        AutoSession::new(env, contacts, rules, services, tls_config, challenges).await;

    (internal_handler, auto_session)
}

/// This defines a uniformi interface to allow for multiple Storage-Engines
/// to be used to actually save the generated Certificates
#[async_trait]
pub trait StoreTLS {
    /// This is used to store the Private Key for the ACME-Account
    async fn store_acc_key(&self, priv_key: &PKey<Private>);
    /// This is used to load the Private Key for the ACME-Account
    async fn load_acc_key(&self) -> Option<PKey<Private>>;

    /// This simply stores the single given Certificate for the Domain
    async fn store(&self, domain: String, priv_key: &PKey<Private>, certificate: &X509);

    /// Turns the given Private-Key into the byte sequence that should be stored
    fn private_key_to_bytes(key: &PKey<Private>) -> Option<Vec<u8>> {
        match key.private_key_to_pem_pkcs8() {
            Ok(d) => Some(d),
            Err(e) => {
                log::error!("Converting Private-Key to Data: {:?}", e);
                None
            }
        }
    }
    /// Turns the given Certificate into the byte sequence that should be stored
    fn cert_to_bytes(certificate: &X509) -> Option<Vec<u8>> {
        match certificate.to_pem() {
            Ok(d) => Some(d),
            Err(e) => {
                log::error!("Converting Certificate to Data: {:?}", e);
                None
            }
        }
    }
}

#[cfg(test)]
mod mocks {
    use super::*;

    pub struct MockStorage {}

    #[async_trait]
    impl StoreTLS for MockStorage {
        async fn store(&self, _domain: String, _priv_key: &PKey<Private>, _certificate: &X509) {}
        async fn load_acc_key(&self) -> Option<PKey<Private>> {
            None
        }
        async fn store_acc_key(&self, _priv_key: &PKey<Private>) {}
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
