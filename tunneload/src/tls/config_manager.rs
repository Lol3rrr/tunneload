use arc_swap::ArcSwap;
use rustls::ServerConfig;
use std::{
    fmt::{Debug, Formatter},
    sync::Arc,
};

/// Manages all the Configuration options around TLS
#[derive(Clone)]
pub struct ConfigManager {
    config: Arc<ArcSwap<ServerConfig>>,
    certs: Arc<std::sync::Mutex<std::collections::BTreeMap<String, rustls::sign::CertifiedKey>>>,
}

impl Debug for ConfigManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ConfigManager ()")
    }
}

impl ConfigManager {
    /// Creates a new Configuration Manager
    pub fn new() -> Self {
        let server_conf = ServerConfig::builder()
            .with_safe_default_cipher_suites()
            .with_safe_default_kx_groups()
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_cert_resolver(Arc::new(rustls::server::ResolvesServerCertUsingSni::new()));

        Self {
            config: Arc::new(ArcSwap::from(Arc::new(server_conf))),
            certs: Arc::new(std::sync::Mutex::new(std::collections::BTreeMap::new())),
        }
    }

    /// Returns the current TLS-Config to be used for an
    /// actual Connection
    pub fn get_config(&self) -> Arc<ServerConfig> {
        self.config.load_full()
    }

    /// This is not cheap, because it copies the entire
    /// BTreeMap
    pub fn get_certs(&self) -> std::collections::BTreeMap<String, rustls::sign::CertifiedKey> {
        let inner = self.certs.lock().unwrap();
        inner.clone()
    }

    /// Creates a new Resolver with all the Keys from the given BTreeMap
    fn create_resolver(
        certs: &std::collections::BTreeMap<String, rustls::sign::CertifiedKey>,
    ) -> rustls::server::ResolvesServerCertUsingSni {
        let mut resolver = rustls::server::ResolvesServerCertUsingSni::new();

        for (key, value) in certs.iter() {
            resolver.add(key, value.clone()).unwrap();
        }

        resolver
    }

    /// Adds the given Certificates to the current Map of Certs or replaces
    /// any previous Certificates under the same name.
    ///
    /// This will then also update the currently held Config and so it takes
    /// effect immediately
    pub fn set_certs(&self, mut certs: Vec<(String, rustls::sign::CertifiedKey)>) {
        let mut inner_btree = self.certs.lock().unwrap();

        for (name, cert) in certs.drain(..) {
            inner_btree.insert(name, cert);
        }
        let config = ServerConfig::builder()
            .with_safe_default_cipher_suites()
            .with_safe_default_kx_groups()
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_cert_resolver(Arc::new(Self::create_resolver(&inner_btree)));

        self.config.store(Arc::new(config));
    }

    /// Sets or Updates the single Certificate for the given Domain
    pub fn set_cert(&self, cert: (String, rustls::sign::CertifiedKey)) {
        let mut inner_btree = self.certs.lock().unwrap();
        inner_btree.insert(cert.0, cert.1);

        let config = ServerConfig::builder()
            .with_safe_default_cipher_suites()
            .with_safe_default_kx_groups()
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_cert_resolver(Arc::new(Self::create_resolver(&inner_btree)));

        self.config.store(Arc::new(config));
    }

    /// Remove the Certificate for the given Domain
    pub fn remove_cert(&self, domain: &str) {
        let mut inner_btree = self.certs.lock().unwrap();
        inner_btree.remove(domain);

        let config = ServerConfig::builder()
            .with_safe_default_cipher_suites()
            .with_safe_default_kx_groups()
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_cert_resolver(Arc::new(Self::create_resolver(&inner_btree)));

        self.config.store(Arc::new(config));
    }

    /// Checks if the Manager has a Certificate registered for the given Domain
    pub fn contains_cert(&self, domain: &str) -> bool {
        let inner_btree = self.certs.lock().unwrap();
        inner_btree.get(domain).is_some()
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
