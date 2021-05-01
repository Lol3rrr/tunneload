use rustls::ServerConfig;
use std::sync::Arc;

/// Manages all the Configuration options around TLS
#[derive(Clone)]
pub struct ConfigManager {
    config: Arc<std::sync::Mutex<Arc<ServerConfig>>>,
    certs: Arc<std::sync::Mutex<std::collections::BTreeMap<String, rustls::sign::CertifiedKey>>>,
}

impl ConfigManager {
    /// Creates a new Configuration Manager
    pub fn new() -> Self {
        let mut server_conf = ServerConfig::new(Arc::new(rustls::NoClientAuth));
        server_conf.cert_resolver = Arc::new(rustls::ResolvesServerCertUsingSNI::new());

        Self {
            config: Arc::new(std::sync::Mutex::new(Arc::new(server_conf))),
            certs: Arc::new(std::sync::Mutex::new(std::collections::BTreeMap::new())),
        }
    }

    pub fn get_config(&self) -> Arc<ServerConfig> {
        let inner = self.config.lock().unwrap();
        inner.clone()
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
    ) -> rustls::ResolvesServerCertUsingSNI {
        let mut resolver = rustls::ResolvesServerCertUsingSNI::new();

        for (key, value) in certs.iter() {
            resolver.add(key, value.clone()).unwrap();
        }

        resolver
    }

    pub fn set_certs(&self, mut certs: Vec<(String, rustls::sign::CertifiedKey)>) {
        let mut inner_btree = self.certs.lock().unwrap();

        for (name, cert) in certs.drain(..) {
            inner_btree.insert(name, cert);
        }
        let mut config = ServerConfig::new(Arc::new(rustls::NoClientAuth));
        config.cert_resolver = Arc::new(Self::create_resolver(&inner_btree));

        let mut inner_config = self.config.lock().unwrap();
        *inner_config = Arc::new(config);
    }

    /// Sets or Updates the single Certificate for the given Domain
    pub fn set_cert(&self, cert: (String, rustls::sign::CertifiedKey)) {
        let mut inner_btree = self.certs.lock().unwrap();
        inner_btree.insert(cert.0, cert.1);

        let mut config = ServerConfig::new(Arc::new(rustls::NoClientAuth));
        config.cert_resolver = Arc::new(Self::create_resolver(&inner_btree));

        let mut inner_config = self.config.lock().unwrap();
        *inner_config = Arc::new(config);
    }

    /// Remove the Certificate for the given Domain
    pub fn remove_cert(&self, domain: &str) {
        let mut inner_btree = self.certs.lock().unwrap();
        inner_btree.remove(domain);

        let mut config = ServerConfig::new(Arc::new(rustls::NoClientAuth));
        config.cert_resolver = Arc::new(Self::create_resolver(&inner_btree));

        let mut inner_config = self.config.lock().unwrap();
        *inner_config = Arc::new(config);
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
