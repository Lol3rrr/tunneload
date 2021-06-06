mod load_secret;
pub use load_secret::load_secret;

mod custom_watcher;
pub use custom_watcher::*;

mod kubernetes_loader;
pub use kubernetes_loader::KubernetesLoader;
mod kubernetes_parser;
pub use kubernetes_parser::KubernetesParser;
mod kubernetes_events;
pub use kubernetes_events::KubernetesEvents;

use k8s_openapi::api::core::v1::Secret;

const TLS_TYPE: &str = "kubernetes.io/tls";
const TLS_DOMAIN_KEY_CERT_MANAGER: &str = "cert-manager.io/common-name";
const TLS_DOMAIN_KEY_TUNNELOAD: &str = "tunneload/common-name";

/// Loads the Domain from the given TLS-Secret
pub fn get_tls_domain(secret: &Secret) -> Option<String> {
    if secret.type_.as_ref()? != TLS_TYPE {
        return None;
    }

    let annotations = secret.metadata.annotations.as_ref()?;

    if let Some(domain) = annotations.get(TLS_DOMAIN_KEY_CERT_MANAGER) {
        return Some(domain.clone());
    }
    if let Some(domain) = annotations.get(TLS_DOMAIN_KEY_TUNNELOAD) {
        return Some(domain.clone());
    }
    None
}

/// Parses a givene Secret as a TLS-Secret and attempts to retrive
/// the Domain and Key Pair to be used by the Acceptors for TLS
pub fn parse_tls(secret: Secret) -> Option<(String, rustls::sign::CertifiedKey)> {
    let domain = get_tls_domain(&secret)?;
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
    let certified_key = rustls::sign::CertifiedKey::new(certs, std::sync::Arc::new(Box::new(key)));

    Some((domain, certified_key))
}
