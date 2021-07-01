//! This contains a variety of "Storage-Backends" which can be used to store
//! TLS-Certificates

pub mod kubernetes {
    //! This module contains all the TLS-relevant Storage stuff for Kubernetes

    use std::{
        collections::BTreeMap,
        fmt::{Debug, Formatter},
    };

    use acme2::openssl::{
        pkey::{PKey, Private},
        x509::X509,
    };
    use async_trait::async_trait;
    use chrono::NaiveDateTime;
    use k8s_openapi::{api::core::v1::Secret, ByteString};
    use kube::{
        api::{ListParams, PostParams},
        Api, Client,
    };

    use crate::tls::auto::TLSStorage;

    /// The TLS-Storage using Kubernetes
    pub struct KubeStore {
        client: kube::Client,
        namespace: String,
    }

    impl Debug for KubeStore {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "KubeStore (namespace = {})", self.namespace)
        }
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

        fn parse_tls_entry(entry: Secret) -> Option<(String, NaiveDateTime)> {
            let metadata = entry.metadata;
            let ty = entry.type_?;
            if ty != "kubernetes.io/tls" {
                return None;
            }
            let annotations = metadata.annotations?;

            let domain = annotations.get("tunneload/common-name")?;

            let mut secret_data = entry.data?;

            let raw_crt = secret_data.remove("tls.crt")?;
            let mut certs_reader = std::io::BufReader::new(std::io::Cursor::new(raw_crt.0));
            let certs: Vec<rustls::Certificate> = match rustls_pemfile::certs(&mut certs_reader) {
                Ok(c) => c.iter().map(|v| rustls::Certificate(v.clone())).collect(),
                Err(e) => {
                    tracing::error!("Getting Certs: {}", e);
                    return None;
                }
            };

            let tmp_c = certs.get(0)?;
            let cert = X509::from_der(tmp_c.as_ref()).unwrap();
            let not_after_string = format!("{:?}", cert.not_after());

            let date = match chrono::NaiveDateTime::parse_from_str(
                &not_after_string,
                "%b %e %H:%M:%S %Y GMT",
            ) {
                Ok(d) => d,
                Err(_) => return None,
            };

            Some((domain.to_owned(), date))
        }
    }

    #[async_trait]
    impl TLSStorage for KubeStore {
        async fn store(&self, domain: String, priv_key: &PKey<Private>, certificate: &X509) {
            let cert = Self::cert_to_bytes(certificate).unwrap();
            let priv_key = Self::private_key_to_bytes(priv_key).unwrap();

            // Create Secret containing
            // * type: kubernetes.io/tls
            // * name: cert-domain
            // * Data
            //   * `tls.crt`: DER-encoded X509
            //   * `tls.key`: The private Key
            // * Annotations
            //   * tunneload/common-name: The Domain

            let secrets: Api<Secret> = Api::namespaced(self.client.clone(), &self.namespace);

            let mut n_secret = Secret {
                type_: Some("kubernetes.io/tls".to_owned()),
                ..Default::default()
            };

            let mut annotations: BTreeMap<String, String> = BTreeMap::new();
            annotations.insert("tunneload/common-name".to_owned(), domain.clone());
            n_secret.metadata.annotations = Some(annotations);

            let mut data: BTreeMap<String, ByteString> = BTreeMap::new();
            data.insert("tls.key".to_owned(), ByteString(priv_key));
            data.insert("tls.crt".to_owned(), ByteString(cert));
            n_secret.data = Some(data);

            n_secret.metadata.name = Some(format!("cert-{}", domain));

            match secrets.create(&PostParams::default(), &n_secret).await {
                Ok(_) => {
                    tracing::info!("Saved Certificate for Domain: {:?}", domain);
                }
                Err(e) => {
                    tracing::error!("Saving Certificate for Domain({:?}): {:?}", domain, e);
                }
            };
        }

        async fn store_acc_key(&self, priv_key: &PKey<Private>) {
            let raw_private_key_data = priv_key.private_key_to_der().unwrap();

            let secrets: Api<Secret> = Api::namespaced(self.client.clone(), &self.namespace);

            let mut n_secret = Secret {
                type_: Some("tunneload/acme.acc".to_owned()),
                ..Default::default()
            };

            let mut data: BTreeMap<String, ByteString> = BTreeMap::new();
            data.insert("key".to_owned(), ByteString(raw_private_key_data));
            n_secret.data = Some(data);

            n_secret.metadata.name = Some("tunneload.acme.acc".to_owned());

            match secrets.create(&PostParams::default(), &n_secret).await {
                Ok(_) => {
                    tracing::info!("Stored ACME-Account-Key");
                }
                Err(e) => {
                    tracing::error!("Storing ACME-Account-Key: {:?}", e);
                }
            };
        }
        async fn load_acc_key(&self) -> Option<PKey<Private>> {
            let secrets: Api<Secret> = Api::namespaced(self.client.clone(), &self.namespace);

            let acc_secret = match secrets.get("tunneload.acme.acc").await {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Loading Account-Key: {:?}", e);
                    return None;
                }
            };

            if acc_secret.type_ == Some("".to_owned()) {
                tracing::error!(
                    "Wrong Tunneload-Account Secret Type: {:?}",
                    acc_secret.type_
                );
                return None;
            }

            let data = acc_secret.data?;
            let raw_key = data.get("key")?;

            let key = match PKey::private_key_from_der(&raw_key.0) {
                Ok(k) => k,
                Err(e) => {
                    tracing::error!("Parsing Private-Key: {:?}", e);
                    return None;
                }
            };

            Some(key)
        }

        async fn load_expiration_dates(&self) -> Vec<(String, NaiveDateTime)> {
            let secrets: Api<Secret> = Api::namespaced(self.client.clone(), &self.namespace);

            let mut result = Vec::new();

            let lp = ListParams::default();
            let list = secrets.list(&lp).await.unwrap();
            for entry in list {
                if let Some(cert) = Self::parse_tls_entry(entry) {
                    result.push(cert);
                }
            }

            result
        }
    }
}
