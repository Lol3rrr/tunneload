use async_trait::async_trait;
use k8s_openapi::api::core::v1::{EndpointSubset, Endpoints, Secret};
use kube::api::Meta;

use crate::{configurator::parser::Parser, rules::Service, util::kubernetes::secret::tls_domain};

/// This is the Parser for the general Kubernetes-Configuration
pub struct KubernetesParser {}

impl KubernetesParser {
    /// Creates a new Instance of the Parser
    pub fn new() -> Self {
        Self {}
    }

    fn parse_subset(subset: EndpointSubset) -> Option<Vec<String>> {
        let addresses = subset.addresses?;
        let ports = subset.ports?;

        let mut result = Vec::new();
        for address in addresses {
            let ip = address.ip;

            for port in &ports {
                result.push(format!("{}:{}", ip, port.port));
            }
        }

        Some(result)
    }

    fn parse_endpoint(endpoint: Endpoints) -> Option<(String, Vec<String>)> {
        let endpoint_name = Meta::name(&endpoint);

        let subsets = endpoint.subsets?;

        let mut endpoint_result = Vec::new();
        for subset in subsets {
            if let Some(tmp) = Self::parse_subset(subset) {
                endpoint_result.extend(tmp);
            }
        }
        Some((endpoint_name, endpoint_result))
    }
}

impl Default for KubernetesParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for KubernetesParser {
    async fn service(&self, config: &serde_json::Value) -> Option<Service> {
        let endpoint = match serde_json::from_value(config.to_owned()) {
            Ok(e) => e,
            Err(e) => {
                log::error!("Parsing Endpoint: {:?}", e);
                return None;
            }
        };

        let (name, destinations) = Self::parse_endpoint(endpoint)?;

        Some(Service::new(name, destinations))
    }

    async fn tls(
        &self,
        config: &serde_json::Value,
    ) -> Option<(String, rustls::sign::CertifiedKey)> {
        let secret: Secret = match serde_json::from_value(config.to_owned()) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Parsing TLS-Secret: {:?}", e);
                return None;
            }
        };

        let domain = tls_domain(&secret)?;
        let mut secret_data = secret.data?;

        let raw_crt = secret_data.remove("tls.crt")?;
        let mut certs_reader = std::io::BufReader::new(std::io::Cursor::new(raw_crt.0));
        let certs: Vec<rustls::Certificate> = match rustls_pemfile::certs(&mut certs_reader) {
            Ok(c) => c.iter().map(|v| rustls::Certificate(v.clone())).collect(),
            Err(e) => {
                log::error!("Getting Certs: {}", e);
                return None;
            }
        };

        let raw_key = secret_data.remove("tls.key")?;
        let mut key_reader = std::io::BufReader::new(std::io::Cursor::new(raw_key.0));
        let key = match rustls_pemfile::read_one(&mut key_reader).expect("Cannot parse key data") {
            Some(rustls_pemfile::Item::RSAKey(key)) => rustls::PrivateKey(key),
            Some(rustls_pemfile::Item::PKCS8Key(key)) => rustls::PrivateKey(key),
            _ => {
                log::error!("[{}] Unknown Key", domain);
                return None;
            }
        };

        let key = match rustls::sign::RSASigningKey::new(&key) {
            Ok(k) => k,
            Err(_) => {
                log::error!("Parsing RSA-Key for '{}'", &domain);
                return None;
            }
        };
        let certified_key =
            rustls::sign::CertifiedKey::new(certs, std::sync::Arc::new(Box::new(key)));

        Some((domain, certified_key))
    }
}
