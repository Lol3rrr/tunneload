//! This modules provides the File-Storage for Certificates

// TODO

use std::path::{Path, PathBuf};

use crate::tls::auto::TLSStorage;

use acme2::openssl::{
    pkey::{PKey, Private},
    x509::X509,
};
use async_trait::async_trait;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

const ACC_KEY_PATH: &str = "acc.key";

/// This is the main Datastructure for handling generated Certificates in
/// File-Format
#[derive(Debug)]
pub struct FileStore {
    folder: PathBuf,
}

impl FileStore {
    /// Creates a new FileStore that uses the given Path as its Root-Path to
    /// look up and store Certificates there
    pub fn new(folder: String) -> Self {
        let path = PathBuf::from(folder);
        Self { folder: path }
    }

    fn get_path(&self, file_name: &str) -> PathBuf {
        self.folder.join(file_name)
    }

    fn cert_filename(domain: &str) -> String {
        format!("{}.tunneload.cert", domain)
    }

    fn cert_path(&self, domain: &str) -> PathBuf {
        let path = Self::cert_filename(domain);
        self.get_path(&path)
    }

    fn is_cert_file(path: &Path) -> bool {
        let file_name = match path.file_name() {
            Some(f) => f.to_string_lossy(),
            None => return false,
        };
        file_name.ends_with(".tunneload.cert")
    }
    fn get_domain(path: &Path) -> Option<String> {
        let file_name = match path.file_name() {
            Some(f) => f.to_string_lossy(),
            None => return None,
        };
        let domain = file_name.strip_suffix(".tunneload.cert")?;
        Some(domain.to_owned())
    }
}

#[derive(Serialize, Deserialize)]
struct StoredCertEntry {
    cert: Vec<u8>,
    key: Vec<u8>,
}

impl StoredCertEntry {
    pub fn new(cert: &X509, priv_key: &PKey<Private>) -> Self {
        let cert_bytes = FileStore::cert_to_bytes(&cert).unwrap();
        let key_bytes = FileStore::private_key_to_bytes(&priv_key).unwrap();

        Self {
            cert: cert_bytes,
            key: key_bytes,
        }
    }
}

struct CertEntry {
    cert: X509,
    key: PKey<Private>,
}

impl CertEntry {
    pub fn store(&self, path: &PathBuf) {
        let store_entry = StoredCertEntry::new(&self.cert, &self.key);
        let data = serde_json::to_vec(&store_entry).unwrap();

        std::fs::write(path, &data).unwrap();
    }

    pub fn load_cert_expiration(path: &PathBuf) -> NaiveDateTime {
        let data = std::fs::read(path).unwrap();

        let stored_cert: StoredCertEntry = serde_json::from_slice(&data).unwrap();

        let cert_buffer = stored_cert.cert;

        let mut certs_reader = std::io::BufReader::new(std::io::Cursor::new(cert_buffer));
        let certs = rustls_pemfile::certs(&mut certs_reader).unwrap();

        let tmp_c = certs.get(0).unwrap();
        let cert = X509::from_der(&tmp_c).unwrap();

        let not_after_string = format!("{:?}", cert.not_after());

        chrono::NaiveDateTime::parse_from_str(&not_after_string, "%b %e %H:%M:%S %Y GMT").unwrap()
    }
}

#[async_trait]
impl TLSStorage for FileStore {
    async fn store_acc_key(
        &self,
        priv_key: &acme2::openssl::pkey::PKey<acme2::openssl::pkey::Private>,
    ) {
        let path = self.get_path(ACC_KEY_PATH);

        let content = Self::private_key_to_bytes(priv_key).unwrap();
        match std::fs::write(&path, &content) {
            Ok(_) => {
                tracing::info!("Stored Account-Key into {:?}", path);
            }
            Err(e) => {
                tracing::error!("Storing Account-Key into File ({:?}): {:?}", path, e);
            }
        };
    }
    async fn load_acc_key(
        &self,
    ) -> Option<acme2::openssl::pkey::PKey<acme2::openssl::pkey::Private>> {
        let path = self.get_path(ACC_KEY_PATH);

        match std::fs::read(&path) {
            Ok(content) => match PKey::private_key_from_der(&content) {
                Ok(k) => Some(k),
                Err(e) => {
                    tracing::error!("Parsing Private-Key: {:?}", e);
                    None
                }
            },
            Err(e) => {
                tracing::error!("Loading Account-Key from File ({:?}): {:?}", path, e);
                None
            }
        }
    }

    async fn store(
        &self,
        domain: String,
        priv_key: PKey<acme2::openssl::pkey::Private>,
        certificate: X509,
    ) {
        let path = self.cert_path(&domain);

        let cert_entry = CertEntry {
            cert: certificate,
            key: priv_key,
        };

        cert_entry.store(&path);
    }
    async fn update(
        &self,
        domain: String,
        priv_key: PKey<acme2::openssl::pkey::Private>,
        certificate: X509,
    ) {
        let path = self.cert_path(&domain);

        let cert_entry = CertEntry {
            cert: certificate,
            key: priv_key,
        };

        cert_entry.store(&path);
    }

    async fn load_expiration_dates(&self) -> Vec<(String, chrono::NaiveDateTime)> {
        let mut result = Vec::new();

        let entries = match std::fs::read_dir(&self.folder) {
            Ok(e) => e,
            Err(e) => {
                tracing::error!("Listing Entries in Directory({:?}): {:?}", self.folder, e);
                return Vec::new();
            }
        };
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    tracing::error!("Getting Entry in Directory: {:?}", e);
                    continue;
                }
            };

            if !entry.file_type().unwrap().is_file() {
                continue;
            }

            let path = entry.path();
            if !Self::is_cert_file(&path) {
                continue;
            }

            let domain = match Self::get_domain(&path) {
                Some(d) => d,
                None => continue,
            };

            let expire_time = CertEntry::load_cert_expiration(&path);

            result.push((domain, expire_time));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_file_name() {
        let file_store = FileStore::new("test/".to_owned());

        assert_eq!(
            PathBuf::from("test/example.com.tunneload.cert"),
            file_store.cert_path("example.com")
        );
    }

    #[test]
    fn is_cert_file() {
        assert_eq!(
            true,
            FileStore::is_cert_file(&PathBuf::from("test/example.com.tunneload.cert"))
        );
    }
    #[test]
    fn cert_domain() {
        assert_eq!(
            Some("example.com".to_owned()),
            FileStore::get_domain(&PathBuf::from("test/example.com.tunneload.cert"))
        );
    }
}
