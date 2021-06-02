pub mod kubernetes {
    //! This module contains all the TLS-relevant Storage stuff for Kubernetes

    use acme2::openssl::x509::X509;
    use async_trait::async_trait;

    use crate::tls::auto::StoreTLS;

    /// The TLS-Storage using Kubernetes
    pub struct KubeStore {}

    #[async_trait]
    impl StoreTLS for KubeStore {
        async fn store(&self, domain: String, certificate: &X509) {
            log::info!("Saving Certificate for Domain: {:?}", domain);
        }
    }
}
