//! This contains a variety of "Storage-Backends" which can be used to store
//! TLS-Certificates

pub mod kubernetes {
    //! This module contains all the TLS-relevant Storage stuff for Kubernetes

    use acme2::openssl::{
        pkey::{PKey, Private},
        x509::X509,
    };
    use async_trait::async_trait;

    use crate::tls::auto::StoreTLS;

    /// The TLS-Storage using Kubernetes
    pub struct KubeStore {}

    #[async_trait]
    impl StoreTLS for KubeStore {
        async fn store(&self, domain: String, priv_key: &PKey<Private>, certificate: &X509) {
            // Create Secret containing
            // * Data
            //   * `tls.crt`: DER-encoded X509
            //   * `tls.key`: The private Key
            // * Annotations
            //   * cert-manager.io/common-name

            let raw_cert_pem_data = certificate.to_pem().unwrap();
            log::info!(
                "Certificate-Pem-Data: {:?}",
                String::from_utf8(raw_cert_pem_data)
            );

            let raw_private_key_data = priv_key.private_key_to_pem_pkcs8().unwrap();
            log::info!(
                "Private-Key-Data: {:?}",
                String::from_utf8(raw_private_key_data)
            );

            log::info!("Saving Certificate for Domain: {:?}", domain);
        }
    }
}
