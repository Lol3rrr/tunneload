use k8s_openapi::api::core::v1::Endpoints;
use kube::api::{Api, ListParams, Meta};

pub async fn load_endpoints(
    client: kube::Client,
    namespace: &str,
) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut result = std::collections::BTreeMap::new();

    let endpoints: Api<Endpoints> = Api::namespaced(client, namespace);
    let lp = ListParams::default();
    for endpoint in endpoints.list(&lp).await.unwrap() {
        let endpoint_name = Meta::name(&endpoint);

        let subsets = match endpoint.subsets {
            Some(s) => s,
            None => {
                continue;
            }
        };

        let mut endpoint_result = Vec::new();

        for subset in subsets {
            let addresses = match subset.addresses {
                Some(a) => a,
                None => {
                    continue;
                }
            };

            let ports = match subset.ports {
                Some(p) => p,
                None => {
                    continue;
                }
            };

            for address in addresses {
                let ip = address.ip;

                for port in &ports {
                    let port = port.port;

                    let final_end = format!("{}:{}", ip, port);
                    endpoint_result.push(final_end);
                }
            }
        }

        result.insert(endpoint_name, endpoint_result);
    }

    result
}
