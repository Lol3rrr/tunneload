use acme2::openssl::{
    pkey::{PKey, Private},
    x509::X509,
};
use async_trait::async_trait;
use chrono::NaiveDateTime;

/// This defines a uniformi interface to allow for multiple Storage-Engines
/// to be used to actually save the generated Certificates
#[async_trait]
pub trait TLSStorage {
    /// This is used to store the Private Key for the ACME-Account
    async fn store_acc_key(&self, priv_key: &PKey<Private>);
    /// This is used to load the Private Key for the ACME-Account
    async fn load_acc_key(&self) -> Option<PKey<Private>>;

    /// This simply stores the single given Certificate for the Domain
    async fn store(&self, domain: String, priv_key: PKey<Private>, certificate: X509);

    /// This updates the Certificate for given Domain
    async fn update(&self, domain: String, priv_key: PKey<Private>, certificate: X509);

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
