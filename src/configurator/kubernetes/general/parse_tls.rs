use k8s_openapi::api::core::v1::Secret;

pub fn get_tls_domain(secret: &Secret) -> Option<String> {
    match &secret.type_ {
        Some(t) => {
            if t != "kubernetes.io/tls" {
                return None;
            }
        }
        None => return None,
    };

    let annotations = match &secret.metadata.annotations {
        Some(a) => a,
        None => return None,
    };

    annotations.get("cert-manager.io/common-name").cloned()
}

pub fn parse_tls(secret: Secret) -> Option<(String, rustls::sign::CertifiedKey)> {
    let domain = match get_tls_domain(&secret) {
        Some(d) => d,
        None => return None,
    };

    let mut secret_data = match secret.data {
        Some(d) => d,
        None => return None,
    };

    let raw_crt = match secret_data.remove("tls.crt") {
        Some(r) => r,
        None => return None,
    };
    let mut certs_reader = std::io::BufReader::new(std::io::Cursor::new(raw_crt.0));
    let certs: Vec<rustls::Certificate> = match rustls_pemfile::certs(&mut certs_reader) {
        Ok(c) => c.iter().map(|v| rustls::Certificate(v.clone())).collect(),
        Err(e) => {
            log::error!("Getting Certs: {}", e);
            return None;
        }
    };

    let raw_key = match secret_data.remove("tls.key") {
        Some(r) => r,
        None => return None,
    };
    let mut key_reader = std::io::BufReader::new(std::io::Cursor::new(raw_key.0));
    let key = match rustls_pemfile::read_one(&mut key_reader).expect("Cannot parse key data") {
        Some(rustls_pemfile::Item::RSAKey(key)) => rustls::PrivateKey(key),
        Some(rustls_pemfile::Item::PKCS8Key(key)) => rustls::PrivateKey(key),
        _ => return None,
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
