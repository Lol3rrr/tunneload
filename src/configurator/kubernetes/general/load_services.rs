use k8s_openapi::api::core::v1::Endpoints;
use kube::api::{Api, ListParams, Meta};

use crate::rules::Service;

pub fn parse_endpoint(ep: &Endpoints) -> Option<Service> {
    let endpoint_name = Meta::name(ep);

    let subsets = match &ep.subsets {
        Some(s) => s,
        None => {
            return None;
        }
    };

    let mut endpoint_result = Vec::new();

    for subset in subsets {
        let addresses = match &subset.addresses {
            Some(a) => a,
            None => {
                continue;
            }
        };

        let ports = match &subset.ports {
            Some(p) => p,
            None => {
                continue;
            }
        };

        for address in addresses {
            let ip = address.ip.clone();

            for port in ports {
                let port = port.port;

                let final_end = format!("{}:{}", ip, port);
                endpoint_result.push(final_end);
            }
        }
    }

    Some(Service::new(endpoint_name, endpoint_result))
}

pub async fn load_services(client: kube::Client, namespace: &str) -> Vec<Service> {
    let mut result = Vec::new();

    let endpoints: Api<Endpoints> = Api::namespaced(client, namespace);
    let lp = ListParams::default();
    for endpoint in endpoints.list(&lp).await.unwrap() {
        if let Some(ep) = parse_endpoint(&endpoint) {
            result.push(ep);
        }
    }

    result
}
