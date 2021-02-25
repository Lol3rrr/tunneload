use k8s_openapi::api::core::v1::Secret;
use kube::api::{Api, ListParams, Meta};

use crate::rules::Rule;

pub async fn load_tls(
    client: kube::Client,
    namespace: &str,
    rules: &[Rule],
) -> Vec<(String, rustls::sign::CertifiedKey)> {
    let mut result = Vec::new();

    let secrets: Api<Secret> = Api::namespaced(client, namespace);
    let lp = ListParams::default();
    for secret in secrets.list(&lp).await.unwrap() {
        let secret_name = Meta::name(&secret);

        let secret_type = match secret.type_ {
            Some(t) => t,
            None => {
                continue;
            }
        };

        if secret_type != "kubernetes.io/tls" {
            continue;
        }

        let mut secret_data = match secret.data {
            Some(d) => d,
            None => {
                continue;
            }
        };

        let raw_crt = match secret_data.remove("tls.crt") {
            Some(r) => r,
            None => {
                continue;
            }
        };
        let mut certs_reader = std::io::BufReader::new(std::io::Cursor::new(raw_crt.0));
        let certs: Vec<rustls::Certificate> = match rustls_pemfile::certs(&mut certs_reader) {
            Ok(c) => c.iter().map(|v| rustls::Certificate(v.clone())).collect(),
            Err(e) => {
                println!("Getting Certs: {}", e);
                continue;
            }
        };

        let raw_key = match secret_data.remove("tls.key") {
            Some(r) => r,
            None => {
                continue;
            }
        };
        let mut key_reader = std::io::BufReader::new(std::io::Cursor::new(raw_key.0));
        let key = match rustls_pemfile::read_one(&mut key_reader).expect("Cannot parse key data") {
            Some(rustls_pemfile::Item::RSAKey(key)) => rustls::PrivateKey(key),
            Some(rustls_pemfile::Item::PKCS8Key(key)) => rustls::PrivateKey(key),
            _ => continue,
        };

        let certified_key = rustls::sign::CertifiedKey::new(
            certs,
            std::sync::Arc::new(Box::new(rustls::sign::RSASigningKey::new(&key).unwrap())),
        );

        for tmp_rule in rules {
            if let Some(name) = tmp_rule.tls() {
                if name == &secret_name {
                    if let Some(domain) = tmp_rule.get_host() {
                        result.push((domain, certified_key));
                    }
                    break;
                }
            }
        }
    }

    result
}
