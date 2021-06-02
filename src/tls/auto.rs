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
    /// This simply stores the single given Certificate for the Domain
    async fn store(&self, domain: String, priv_key: &PKey<Private>, certificate: &X509);
}
