use k8s_openapi::api::core::v1::{EndpointSubset, Endpoints};
use kube::api::{Api, ListParams, Meta};

fn parse_subset(subset: EndpointSubset) -> Option<Vec<String>> {
    let addresses = subset.addresses?;
    let ports = subset.ports?;

    let mut result = Vec::new();
    for address in addresses {
        let ip = address.ip;

        for port in &ports {
            result.push(format!("{}:{}", ip, port.port));
        }
    }

    Some(result)
}

fn parse_endpoint(endpoint: Endpoints) -> Option<(String, Vec<String>)> {
    let endpoint_name = Meta::name(&endpoint);

    let subsets = endpoint.subsets?;

    let mut endpoint_result = Vec::new();
    for subset in subsets {
        if let Some(tmp) = parse_subset(subset) {
            endpoint_result.extend(tmp);
        }
    }
    Some((endpoint_name, endpoint_result))
}

/// Loads the Endpoints in the given Namespace
pub async fn load_endpoints(
    client: kube::Client,
    namespace: &str,
) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut result = std::collections::BTreeMap::new();

    let endpoints: Api<Endpoints> = Api::namespaced(client, namespace);
    let lp = ListParams::default();
    for endpoint in endpoints.list(&lp).await.unwrap() {
        if let Some((name, value)) = parse_endpoint(endpoint) {
            result.insert(name, value);
        }
    }

    result
}
