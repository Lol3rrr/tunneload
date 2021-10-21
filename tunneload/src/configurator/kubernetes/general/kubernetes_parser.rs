use std::{error::Error, fmt::Display};

use async_trait::async_trait;
use general::{Group, Name};
use k8s_openapi::api::core::v1::{EndpointSubset, Endpoints, Secret};
use kube::api::Meta;

use crate::{configurator::parser::Parser, util::kubernetes::secret::tls_domain};
use rules::Service;

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

    fn parse_endpoint(endpoint: Endpoints) -> (Name, Vec<String>) {
        let name = Meta::name(&endpoint);
        let namespace = Meta::namespace(&endpoint).unwrap_or_else(|| "default".to_string());

        let targets = match endpoint.subsets {
            Some(subsets) => {
                let mut endpoint_result = Vec::new();
                for subset in subsets {
                    if let Some(tmp) = Self::parse_subset(subset) {
                        endpoint_result.extend(tmp);
                    }
                }
                endpoint_result
            }
            None => Vec::new(),
        };

        let endpoint_name = Name::new(name, Group::Kubernetes { namespace });
        (endpoint_name, targets)
    }
}

impl Default for KubernetesParser {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum ServiceParseError {
    InvalidRawConfig(serde_json::Error),
}

impl Display for ServiceParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Service-Parse-Error")
    }
}

impl Error for ServiceParseError {}

#[derive(Debug)]
pub enum TlsParseError {
    InvalidRawConfig(serde_json::Error),
    MissingDomain,
    MissingData,
    MissingCertificate,
    InvalidCertficiate(std::io::Error),
    MissingKey,
    InvalidKey,
}

impl Display for TlsParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TLS-Parse-Error")
    }
}
impl Error for TlsParseError {}

#[async_trait]
impl Parser for KubernetesParser {
    async fn service(&self, config: &serde_json::Value) -> Result<Service, Box<dyn Error>> {
        let endpoint = match serde_json::from_value(config.to_owned()) {
            Ok(e) => e,
            Err(e) => {
                return Err(Box::new(ServiceParseError::InvalidRawConfig(e)));
            }
        };

        let (name, destinations) = Self::parse_endpoint(endpoint);
        Ok(Service::new(name, destinations))
    }

    async fn tls(
        &self,
        config: &serde_json::Value,
    ) -> Result<(String, rustls::sign::CertifiedKey), Box<dyn Error>> {
        let secret: Secret = match serde_json::from_value(config.to_owned()) {
            Ok(s) => s,
            Err(e) => {
                return Err(Box::new(TlsParseError::InvalidRawConfig(e)));
            }
        };

        let domain = tls_domain(&secret).ok_or_else(|| Box::new(TlsParseError::MissingDomain))?;
        let mut secret_data = secret
            .data
            .ok_or_else(|| Box::new(TlsParseError::MissingData))?;

        let raw_crt = secret_data
            .remove("tls.crt")
            .ok_or_else(|| Box::new(TlsParseError::MissingCertificate))?;
        let mut certs_reader = std::io::BufReader::new(std::io::Cursor::new(raw_crt.0));
        let certs: Vec<rustls::Certificate> = match rustls_pemfile::certs(&mut certs_reader) {
            Ok(c) => c.iter().map(|v| rustls::Certificate(v.clone())).collect(),
            Err(e) => {
                return Err(Box::new(TlsParseError::InvalidCertficiate(e)));
            }
        };

        let raw_key = secret_data
            .remove("tls.key")
            .ok_or_else(|| Box::new(TlsParseError::MissingKey))?;
        let mut key_reader = std::io::BufReader::new(std::io::Cursor::new(raw_key.0));
        let key = match rustls_pemfile::read_one(&mut key_reader).expect("Cannot parse key data") {
            Some(rustls_pemfile::Item::RSAKey(key)) => rustls::PrivateKey(key),
            Some(rustls_pemfile::Item::PKCS8Key(key)) => rustls::PrivateKey(key),
            _ => {
                return Err(Box::new(TlsParseError::InvalidKey));
            }
        };

        let key = match rustls::sign::RSASigningKey::new(&key) {
            Ok(k) => k,
            Err(_) => {
                return Err(Box::new(TlsParseError::InvalidKey));
            }
        };
        let certified_key =
            rustls::sign::CertifiedKey::new(certs, std::sync::Arc::new(Box::new(key)));

        Ok((domain, certified_key))
    }
}

#[cfg(test)]
mod tests {
    use k8s_openapi::api::core::v1::{EndpointAddress, EndpointPort};
    use kube::api::ObjectMeta;

    use super::*;

    #[tokio::test]
    async fn service_no_targets() {
        let parser = KubernetesParser::default();

        let endpoints = Endpoints {
            metadata: ObjectMeta {
                name: Some("test".to_owned()),
                ..Default::default()
            },
            subsets: None,
        };
        let config = serde_json::to_value(endpoints).unwrap();

        let result = parser.service(&config).await;
        let expected = Service::new(
            Name::new(
                "test",
                Group::Kubernetes {
                    namespace: "default".to_owned(),
                },
            ),
            vec![],
        );

        assert_eq!(true, result.is_ok());
        assert_eq!(expected, result.unwrap());
    }

    #[tokio::test]
    async fn service_targets() {
        let parser = KubernetesParser::default();

        let endpoints = Endpoints {
            metadata: ObjectMeta {
                name: Some("test".to_owned()),
                ..Default::default()
            },
            subsets: Some(vec![EndpointSubset {
                addresses: Some(vec![EndpointAddress {
                    ip: "192.168.1.1".to_owned(),
                    ..Default::default()
                }]),
                ports: Some(vec![EndpointPort {
                    port: 8080,
                    ..Default::default()
                }]),
                ..Default::default()
            }]),
        };
        let config = serde_json::to_value(endpoints).unwrap();

        let result = parser.service(&config).await;
        let expected = Service::new(
            Name::new(
                "test",
                Group::Kubernetes {
                    namespace: "default".to_owned(),
                },
            ),
            vec!["192.168.1.1:8080".to_owned()],
        );

        assert_eq!(true, result.is_ok());
        assert_eq!(expected, result.unwrap());
    }

    #[tokio::test]
    async fn other_namespace() {
        let parser = KubernetesParser::default();

        let endpoints = Endpoints {
            metadata: ObjectMeta {
                name: Some("test".to_owned()),
                namespace: Some("other".to_string()),
                ..Default::default()
            },
            subsets: Some(vec![EndpointSubset {
                addresses: Some(vec![EndpointAddress {
                    ip: "192.168.1.1".to_owned(),
                    ..Default::default()
                }]),
                ports: Some(vec![EndpointPort {
                    port: 8080,
                    ..Default::default()
                }]),
                ..Default::default()
            }]),
        };
        let config = serde_json::to_value(endpoints).unwrap();

        let result = parser.service(&config).await;
        let expected = Service::new(
            Name::new(
                "test",
                Group::Kubernetes {
                    namespace: "other".to_owned(),
                },
            ),
            vec!["192.168.1.1:8080".to_owned()],
        );

        assert_eq!(true, result.is_ok());
        assert_eq!(expected, result.unwrap());
    }
}
