use log::error;
use serde_json::Value;

use crate::configurator::kubernetes::general::load_secret;
use crate::rules::{Action, Middleware};

pub async fn parse(
    name: &str,
    value: &Value,
    client: kube::Client,
    namespace: &str,
) -> Option<Middleware> {
    let auth_value = value.as_object().unwrap();

    let raw_secret_name = match auth_value.get("secret") {
        Some(s) => s,
        None => {
            error!("Could not load Secret-Name for basic-Auth");
            return None;
        }
    };
    let secret_name = match raw_secret_name.as_str() {
        Some(s) => s,
        None => {
            error!("Secret-Name is not a String");
            return None;
        }
    };

    let raw_secret_value = match load_secret(client, namespace, secret_name).await {
        Some(s) => s,
        None => {
            error!("Loading Secret-Data");
            return None;
        }
    };

    let raw_users_data = match raw_secret_value.get("users") {
        Some(d) => d,
        None => {
            error!("Loading Users from Secret-Data");
            return None;
        }
    };

    let users_data = match std::str::from_utf8(&raw_users_data.0) {
        Ok(d) => d,
        Err(e) => {
            error!("Getting Base64-Data from Secret: {}", e);
            return None;
        }
    };

    Some(Middleware::new(
        name,
        Action::new_basic_auth_hashed(users_data),
    ))
}
