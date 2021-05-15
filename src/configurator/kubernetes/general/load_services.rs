use k8s_openapi::api::core::v1::{EndpointSubset, Endpoints};
use kube::api::{Api, ListParams, Meta};

use crate::rules::Service;

fn parse_subset(subset: &EndpointSubset) -> Option<Vec<String>> {
    let addresses = subset.addresses.as_ref()?;
    let ports = subset.ports.as_ref()?;

    let mut res = Vec::new();

    for address in addresses {
        let ip = address.ip.clone();

        for port in ports {
            let port = port.port;

            let final_end = format!("{}:{}", ip, port);
            res.push(final_end);
        }
    }

    Some(res)
}

/// The Errors that can be returned when parsing an Endpoint
#[derive(Debug)]
pub enum EndpointError {
    /// The given Endpoint did not contain any Subsets
    MissingSubsets,
}

/// Attempts to parse an Endpoint as a Service
pub fn parse_endpoint(ep: &Endpoints) -> Result<Service, EndpointError> {
    let endpoint_name = Meta::name(ep);

    let subsets = match &ep.subsets {
        Some(s) => s,
        None => {
            return Err(EndpointError::MissingSubsets);
        }
    };

    let mut endpoint_result = Vec::new();

    for subset in subsets {
        if let Some(parsed_endpoints) = parse_subset(subset) {
            endpoint_result.extend(parsed_endpoints);
        }
    }

    Ok(Service::new(endpoint_name, endpoint_result))
}

/// Loads all the Services from the Kubernetes Cluster
pub async fn load_services(client: kube::Client, namespace: &str) -> Vec<Service> {
    let mut result = Vec::new();

    let endpoints: Api<Endpoints> = Api::namespaced(client, namespace);
    let lp = ListParams::default();
    for endpoint in endpoints.list(&lp).await.unwrap() {
        if let Ok(ep) = parse_endpoint(&endpoint) {
            result.push(ep);
        }
    }

    result
}
