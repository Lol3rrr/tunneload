//! This contains a variety of "Storage-Backends" which can be used to store
//! TLS-Certificates

fn strip_headers(content: &str, h_type: &str) -> String {
    let after_begin = match content.strip_prefix(&format!("-----BEGIN {}-----\n", h_type)) {
        Some(r) => r,
        None => content,
    };

    let after_end = match after_begin.strip_suffix(&format!("\n-----END {}-----\n", h_type)) {
        Some(r) => r,
        None => after_begin,
    };

    after_end.to_owned()
}

fn strip_cert_headers<S>(content: S) -> String
where
    S: AsRef<str>,
{
    strip_headers(content.as_ref(), "CERTIFICATE")
}
fn strip_key_headers<S>(content: S) -> String
where
    S: AsRef<str>,
{
    strip_headers(content.as_ref(), "PRIVATE KEY")
}

pub mod kubernetes {
    //! This module contains all the TLS-relevant Storage stuff for Kubernetes

    use std::collections::BTreeMap;

    use acme2::openssl::{
        pkey::{PKey, Private},
        x509::X509,
    };
    use async_trait::async_trait;
    use k8s_openapi::api::core::v1::Secret;
    use kube::{api::PostParams, Api, Client};

    use crate::tls::{
        auto::StoreTLS,
        stores::{strip_cert_headers, strip_key_headers},
    };

    /// The TLS-Storage using Kubernetes
    pub struct KubeStore {
        client: kube::Client,
        namespace: String,
    }

    impl KubeStore {
        /// Creates a new Kube-Store instance
        pub async fn new() -> Self {
            let client = Client::try_default().await.unwrap();

            Self {
                client,
                namespace: "default".to_owned(),
            }
        }

        async fn store_cert(&self, domain: String, priv_key: String, cert: String) {
            // Create Secret containing
            // * type: kubernetes.io/tls
            // * name: cert-domain
            // * Data
            //   * `tls.crt`: DER-encoded X509
            //   * `tls.key`: The private Key
            // * Annotations
            //   * tunneload/common-name: The Domain

            let secrets: Api<Secret> = Api::namespaced(self.client.clone(), &self.namespace);

            let mut n_secret = Secret::default();
            n_secret.type_ = Some("kubernetes.io/tls".to_owned());

            let mut annotations: BTreeMap<String, String> = BTreeMap::new();
            annotations.insert("tunneload/common-name".to_owned(), domain.clone());
            n_secret.metadata.annotations = Some(annotations);

            let mut data: BTreeMap<String, String> = BTreeMap::new();
            data.insert("tls.key".to_owned(), priv_key);
            data.insert("tls.crt".to_owned(), cert);
            n_secret.string_data = Some(data);

            n_secret.metadata.name = Some(format!("cert-{}", domain));

            match secrets.create(&PostParams::default(), &n_secret).await {
                Ok(_) => {
                    log::info!("Saved Certificate for Domain: {:?}", domain);
                }
                Err(e) => {
                    log::error!("Saving Certificate for Domain({:?}): {:?}", domain, e);
                }
            };
        }
    }

    #[async_trait]
    impl StoreTLS for KubeStore {
        async fn store(&self, domain: String, priv_key: &PKey<Private>, certificate: &X509) {
            let raw_cert_pem_data = certificate.to_pem().unwrap();
            let cert_pem_string = String::from_utf8(raw_cert_pem_data).unwrap();

            let raw_private_key_data = priv_key.private_key_to_pem_pkcs8().unwrap();
            let private_key_string = String::from_utf8(raw_private_key_data).unwrap();

            self.store_cert(
                domain,
                strip_key_headers(private_key_string),
                strip_cert_headers(cert_pem_string),
            )
            .await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_no_headers() {
        let original = "test".to_owned();

        let result = strip_cert_headers(original);
        let expected = "test".to_owned();

        assert_eq!(expected, result);
    }

    #[test]
    fn strip_simple_headers() {
        let original = "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----\n".to_owned();

        let result = strip_cert_headers(original);
        let expected = "test".to_owned();

        assert_eq!(expected, result);
    }

    #[test]
    fn strip_hard_headers() {
        let original =
            "-----BEGIN CERTIFICATE-----\ntest\ntest2\n-----END CERTIFICATE-----\n".to_owned();

        let result = strip_cert_headers(original);
        let expected = "test\ntest2".to_owned();

        assert_eq!(expected, result);
    }
}
