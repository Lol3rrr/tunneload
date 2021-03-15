use k8s_openapi::api::core::v1::Secret;
use kube::{api::ListParams, Api, Client};

use crate::configurator::kubernetes::general::Watcher;
use crate::configurator::kubernetes::general::{get_tls_domain, parse_tls, Event};
use crate::tls;

pub async fn listen_tls(client: Client, namespace: String, tls_manager: tls::ConfigManager) {
    let secrets: Api<Secret> = Api::namespaced(client, &namespace);
    let lp = ListParams::default();

    let mut stream = match Watcher::from_api(secrets, Some(lp)).await {
        Ok(s) => s,
        Err(e) => {
            log::error!("Creating Stream: {}", e);
            return;
        }
    };

    loop {
        let event = match stream.next_event().await {
            Some(e) => e,
            None => {
                return;
            }
        };

        match event {
            Event::Updated(secret) => {
                let tls = match parse_tls(secret) {
                    Some(t) => t,
                    None => continue,
                };

                log::info!("Updated TLS '{}'", tls.0);
                tls_manager.set_cert(tls);
            }
            Event::Removed(secret) => {
                let domain = match get_tls_domain(&secret) {
                    Some(d) => d,
                    None => continue,
                };

                log::info!("Removed TLS: '{}'", domain);
                tls_manager.remove_cert(&domain);
            }
            _ => {}
        };
    }
}
