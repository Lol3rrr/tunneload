use std::sync::Arc;

use acme2::{
    openssl::pkey::{PKey, Private},
    Order,
};

/// The Let's Encrypt Environment, this allows you to switch to the Staging
/// environment when testing Configurations
pub enum Environment {
    /// The Staging/Development Environment
    ///
    /// This should be used during Testing, but does not actually
    /// generate any valid Certificates
    Staging,
    /// The Production Environment
    ///
    /// This actually performs the ACME-Challenge and generates
    /// valid Certificates
    Production,
}

impl Environment {
    /// Returns the URL for the given Enviroment
    pub fn url(&self) -> String {
        match self {
            Environment::Staging => {
                "https://acme-staging-v02.api.letsencrypt.org/directory".to_owned()
            }
            Environment::Production => "https://acme-v02.api.letsencrypt.org/directory".to_owned(),
        }
    }
}

/// The Error type returned when generating Verifying Data
#[derive(Debug)]
pub enum VerifyError {
    /// The Order failed to be build
    OrderBuild(acme2::Error),
    /// All other Verify errors
    Other,
}

impl From<acme2::Error> for VerifyError {
    fn from(other: acme2::Error) -> Self {
        Self::OrderBuild(other)
    }
}

/// A single Account used for generating and performing ACME-Challenges
pub struct Account {
    account: Arc<acme2::Account>,
}

impl Account {
    /// Creates a new Account from the given Parameters
    ///
    /// ## Parameters:
    /// * `env`: The Let's Encrypt Environment
    /// * `contact`: The List of Contacts to list
    pub async fn new(
        env: &Environment,
        contact: Vec<String>,
        priv_key: Option<PKey<Private>>,
    ) -> Option<Self> {
        let url = env.url();
        let directory = acme2::DirectoryBuilder::new(url)
            .build()
            .await
            .expect("Creating Directory Builder");

        let mut builder = acme2::AccountBuilder::new(directory);
        builder.contact(contact);
        builder.terms_of_service_agreed(true);
        if let Some(key) = priv_key {
            builder.private_key(key);
        }
        let account = match builder.build().await {
            Ok(acc) => acc,
            Err(e) => {
                tracing::error!("Creating ACME-Account: {:?}", e);
                return None;
            }
        };

        Some(Self { account })
    }

    /// Returns the Private-Key of the ACME-Account
    pub fn private_key(&self) -> PKey<Private> {
        self.account.private_key()
    }

    /// Generates all the Challenges for the Domain
    ///
    /// ## Parameters:
    /// * `domain`: The Domain to generate the TLS-Certificate for
    pub async fn generate_verify(
        &self,
        domain: String,
    ) -> Result<(Order, Vec<(PendingTLS, acme2::Challenge)>), VerifyError> {
        let mut order_builder = acme2::OrderBuilder::new(self.account.clone());
        order_builder.add_dns_identifier(domain);
        let order = order_builder.build().await?;
        let mut results = Vec::new();

        let authorizations = order
            .authorizations()
            .await
            .map_err(|_| VerifyError::Other)?;
        for auth in authorizations.iter() {
            let challenge = match auth.get_challenge("http-01") {
                Some(c) => c,
                None => continue,
            };

            let token = match challenge.token.clone() {
                Some(t) => t,
                None => continue,
            };
            let key = match challenge.key_authorization() {
                Ok(Some(k)) => k,
                _ => continue,
            };

            let pending_tls = PendingTLS::new(key, token);
            results.push((pending_tls, challenge));
        }

        Ok((order, results))
    }
}

/// This represents a single Pending ACME-Challenge
pub struct PendingTLS {
    token: String,
    key: String,
}

impl PendingTLS {
    /// Creates a new Pending ACME-Challenge
    pub fn new(key: String, token: String) -> Self {
        Self { token, key }
    }

    /// The Token/ID of the Pending-TLS/ACME-Challenge
    pub fn token(&self) -> &str {
        &self.token
    }

    /// The Key to verify this Request
    pub fn key(&self) -> &str {
        &self.key
    }
}
