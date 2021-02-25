use rustls::ServerConfig;
use std::sync::Arc;

#[derive(Clone)]
pub struct ConfigManager {
    config: Arc<std::sync::Mutex<Arc<ServerConfig>>>,
}

impl ConfigManager {
    pub fn new() -> Self {
        let mut server_conf = ServerConfig::new(Arc::new(rustls::NoClientAuth));
        server_conf.cert_resolver = Arc::new(rustls::ResolvesServerCertUsingSNI::new());

        Self {
            config: Arc::new(std::sync::Mutex::new(Arc::new(server_conf))),
        }
    }

    pub fn get_config(&self) -> Arc<ServerConfig> {
        let inner = self.config.lock().unwrap();
        inner.clone()
    }

    pub fn set_certs(&self, mut certs: Vec<(String, rustls::sign::CertifiedKey)>) {
        let mut n_resolver = rustls::ResolvesServerCertUsingSNI::new();

        for (name, cert) in certs.drain(..) {
            n_resolver.add(&name, cert).unwrap();
        }

        let mut config = ServerConfig::new(Arc::new(rustls::NoClientAuth));
        config.cert_resolver = Arc::new(n_resolver);

        let mut inner_config = self.config.lock().unwrap();
        *inner_config = Arc::new(config);
        drop(inner_config);
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
