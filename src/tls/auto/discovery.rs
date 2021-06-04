pub mod kubernetes {
    use std::{collections::HashSet, net::Ipv4Addr, sync::Arc};

    use async_raft::{NodeId, Raft};
    use async_trait::async_trait;
    use k8s_openapi::api::core::v1::Endpoints;
    use kube::{api::ListParams, Api};
    use pnet::ipnetwork::IpNetwork;
    use tokio::sync::RwLock;

    use crate::{
        configurator::kubernetes::general::{Event, Watcher},
        tls::auto::{
            consensus::{self, addr_to_id},
            AutoDiscover,
        },
    };

    pub struct Discover {
        client: kube::Client,
        service_name: String,
        nodes: RwLock<HashSet<NodeId>>,
    }

    impl Discover {
        pub async fn new_default(service_name: String) -> Self {
            let client = kube::Client::try_default().await.unwrap();

            Self {
                client,
                service_name,
                nodes: RwLock::new(HashSet::new()),
            }
        }

        fn parse_endpoints(p: Endpoints) -> Vec<NodeId> {
            let subsets = match p.subsets {
                Some(s) => s,
                None => return Vec::new(),
            };

            let mut result = Vec::new();

            for subset in subsets.iter() {
                let addresses = match &subset.addresses {
                    Some(a) => a,
                    None => continue,
                };

                for address in addresses.iter() {
                    let raw_ip = &address.ip;
                    let ip: Ipv4Addr = raw_ip.parse().unwrap();
                    let id = addr_to_id(ip.clone()).unwrap();

                    result.push(id);
                }
            }

            result
        }

        async fn update_single(
            &self,
            p: Endpoints,
            raft: &Raft<
                consensus::Request,
                consensus::Response,
                consensus::Network,
                consensus::Storage,
            >,
        ) {
            let endpoint_ids = Self::parse_endpoints(p);

            let mut updated = false;
            let mut nodes = self.nodes.write().await;
            for id in endpoint_ids {
                if !nodes.contains(&id) {
                    nodes.insert(id);

                    if raft.client_read().await.is_err() {
                        log::info!("Current Node is not the Leader");
                        continue;
                    }

                    log::info!("Adding Node to cluster: {}", id);
                    if let Err(e) = raft.add_non_voter(id).await {
                        log::error!("Could not add Node({}) to the Cluster: {:?}", id, e);
                        continue;
                    }
                    updated = true;
                }
            }

            if updated {
                if let Err(e) = raft.change_membership(nodes.clone()).await {
                    log::error!("Changing Membership after Updating-Nodes: {:?}", e);
                }
            }
        }
    }

    #[async_trait]
    impl AutoDiscover for Discover {
        async fn get_own_id() -> NodeId {
            let all_interfaces = pnet::datalink::interfaces();
            let default_interface = all_interfaces
                .iter()
                .find(|e| e.is_up() && !e.is_loopback() && !e.ips.is_empty());

            let interface = match default_interface {
                Some(i) => i,
                None => return 0,
            };

            let default_ip = interface.ips.iter().find(|i| i.is_ipv4());
            match default_ip {
                Some(ip) => match ip {
                    IpNetwork::V4(v4) => addr_to_id(v4.ip()).unwrap(),
                    _ => 0,
                },
                None => 0,
            }
        }

        async fn get_all_nodes(&self) -> HashSet<NodeId> {
            let nodes = self.nodes.read().await;
            nodes.clone()
        }

        async fn watch_nodes(
            self: Arc<Self>,
            raft: Raft<
                consensus::Request,
                consensus::Response,
                consensus::Network,
                consensus::Storage,
            >,
        ) {
            let api: Api<Endpoints> = Api::namespaced(self.client.clone(), "default");
            let mut lp = ListParams::default();
            lp = lp.fields(&format!("metadata.name={}", self.service_name));

            let mut watcher = Watcher::from_api(api, Some(lp)).await.unwrap();
            loop {
                let tmp = watcher.next_event().await;

                let tmp = match tmp {
                    Some(t) => t,
                    None => {
                        log::error!("Watcher returned None");
                        return;
                    }
                };

                match tmp {
                    Event::Started(mut all_p) => {
                        for p in all_p.drain(..) {
                            self.update_single(p, &raft).await;
                        }
                    }
                    Event::Updated(p) => {
                        let subsets = match &p.subsets {
                            Some(s) => s,
                            None => {
                                // Generally speaking this case should never actually occur
                                // because the current instance should be part of the Endpoints
                                // as well and therefore there is always at least one instance
                                // running or the situation with no entries will not be observed
                                let mut nodes = self.nodes.write().await;
                                nodes.clear();
                                continue;
                            }
                        };

                        let nodes = self.nodes.read().await;
                        if subsets.len() < nodes.len() {
                            let mut registered_ids = nodes.clone();
                            let subset_ids = Self::parse_endpoints(p);
                            drop(nodes);

                            for subset_id in subset_ids.iter() {
                                registered_ids.remove(subset_id);
                            }

                            let mut write_nodes = self.nodes.write().await;
                            for removed_id in registered_ids.iter() {
                                write_nodes.remove(removed_id);
                            }
                        } else {
                            self.update_single(p, &raft).await;
                        }
                    }
                    Event::Removed(_) => {
                        log::info!("Removed");
                    }
                    Event::Other => {
                        log::info!("Other")
                    }
                };
            }
        }
    }
}
