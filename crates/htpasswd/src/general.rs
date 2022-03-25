use crate::md5::{self, APR1_ID};

use crypto::{digest::Digest, sha1::Sha1};
use serde::Serialize;
use std::collections::HashMap;

const BCRYPT_ID: &str = "$2y$";
const SHA1_ID: &str = "{SHA}";

/// Stores a collection of Usernames and their
/// matching Hashed passwords.
///
/// This is not intended for very secure storage
/// or anything the like but rather for quick and easy
/// user logins to sites which maybe not everyone
/// should see, but it also would not be the worst
/// if they were exposed
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Htpasswd(pub HashMap<String, Hash>);

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Hash {
    MD5(MD5Hash),
    BCrypt(String),
    SHA1(String),
    Crypt(String),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MD5Hash {
    pub salt: String,
    pub hash: String,
}

impl Htpasswd {
    /// Checks if the username and password are known and
    /// match up
    pub fn check(&self, username: &str, password: &str) -> bool {
        let hash = match self.0.get(username) {
            Some(h) => h,
            None => return false,
        };
        match hash {
            Hash::MD5(hash) => {
                if let Some(comp_hash) = md5::md5_apr1_encode(password, &hash.salt) {
                    comp_hash.as_str() == hash.hash
                } else {
                    false
                }
            }
            Hash::BCrypt(hash) => bcrypt::verify(password, hash).unwrap_or(false),
            Hash::SHA1(hash) => {
                let mut hasher = Sha1::new();
                hasher.input_str(password);
                let size = hasher.output_bytes();
                let mut buf = vec![0u8; size];
                hasher.result(&mut buf);
                base64::encode(&buf).as_str() == *hash
            }
            Hash::Crypt(hash) => pwhash::unix_crypt::verify(password, hash),
        }
    }
}

/// Attempts to load and "parse" valid Username-Password
/// combinations from the given String
pub fn load(bytes: &str) -> Htpasswd {
    let lines = bytes.split('\n');
    let hashes = lines
        .filter_map(parse_hash_entry)
        .collect::<HashMap<String, Hash>>();
    Htpasswd(hashes)
}

fn parse_hash_entry(entry: &str) -> Option<(String, Hash)> {
    let semicolon = match entry.find(':') {
        Some(idx) => idx,
        None => return None,
    };
    let username = entry[..semicolon].to_owned();

    let hash_id = &entry[(semicolon + 1)..];
    if hash_id.starts_with(md5::APR1_ID) {
        Some((
            username,
            Hash::MD5(MD5Hash {
                salt: entry[(semicolon + 1 + APR1_ID.len())..(semicolon + 1 + APR1_ID.len() + 8)]
                    .to_owned(),
                hash: entry[(semicolon + 1 + APR1_ID.len() + 8 + 1)..].to_owned(),
            }),
        ))
    } else if hash_id.starts_with(BCRYPT_ID) {
        Some((username, Hash::BCrypt(entry[(semicolon + 1)..].to_owned())))
    } else if hash_id.starts_with("{SHA}") {
        Some((
            username,
            Hash::SHA1(entry[(semicolon + 1 + SHA1_ID.len())..].to_owned()),
        ))
    } else {
        //Ignore plaintext, assume crypt

        Some((username, Hash::Crypt(entry[(semicolon + 1)..].to_owned())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static DATA: &str = "user2:$apr1$7/CTEZag$omWmIgXPJYoxB3joyuq4S/
user:$apr1$lZL6V/ci$eIMz/iKDkbtys/uU7LEK00
bcrypt_test:$2y$05$nC6nErr9XZJuMJ57WyCob.EuZEjylDt2KaHfbfOtyb.EgL1I2jCVa
sha1_test:{SHA}W6ph5Mm5Pz8GgiULbPgzG37mj9g=
crypt_test:bGVh02xkuGli2";

    #[test]
    fn unix_crypt_verify_htpasswd() {
        let htpasswd = load(DATA);
        assert_eq!(htpasswd.check("crypt_test", "password"), true);
    }

    #[test]
    fn sha1_verify_htpasswd() {
        let htpasswd = load(DATA);
        assert_eq!(htpasswd.check("sha1_test", "password"), true);
    }

    #[test]
    fn bcrypt_verify_htpasswd() {
        let htpasswd = load(DATA);
        assert_eq!(htpasswd.check("bcrypt_test", "password"), true);
    }

    #[test]
    fn md5_verify_htpasswd() {
        let htpasswd = load(DATA);
        assert_eq!(htpasswd.check("user", "password"), true);
        assert_eq!(htpasswd.check("user", "passwort"), false);
        assert_eq!(htpasswd.check("user2", "zaq1@WSX"), true);
        assert_eq!(htpasswd.check("user2", "ZAQ1@WSX"), false);
    }

    #[test]
    fn md5_apr1() {
        assert_eq!(
            md5::format_hash(
                md5::md5_apr1_encode("password", "xxxxxxxx")
                    .unwrap()
                    .as_str(),
                "xxxxxxxx",
            ),
            "$apr1$xxxxxxxx$dxHfLAsjHkDRmG83UXe8K0".to_string()
        );
    }

    #[test]
    fn apr1() {
        assert!(
            md5::verify_apr1_hash("$apr1$xxxxxxxx$dxHfLAsjHkDRmG83UXe8K0", "password").unwrap()
        );
    }
}
