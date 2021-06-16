//! This module contains all the given Discovery-Mechanisms that Tunneload
//! provides by default

pub mod kubernetes {
    //! This module contains all the needed Parts to use the Kubernetes
    //! discovery mechanism

    use std::{
        collections::HashSet,
        fmt::{Debug, Formatter},
        net::{Ipv4Addr, SocketAddrV4},
        sync::Arc,
    };

    use async_raft::NodeId;
    use async_trait::async_trait;
    use k8s_openapi::api::core::v1::Endpoints;
    use kube::{api::ListParams, Api};
    use pnet::ipnetwork::IpNetwork;
    use tokio::sync::RwLock;

    use crate::{
        tls::auto::{
            cluster::{addr_to_id, Cluster},
            AutoDiscover,
        },
        util::kubernetes::watcher::{Event, Watcher},
    };

    /// This holds all the information needed by the Kubernetes-Discoverer
    /// to find and add new Nodes/Instances as needed
    pub struct Discover {
        client: kube::Client,
        service_name: String,
        nodes: RwLock<HashSet<NodeId>>,
        port: u16,
    }

    impl Debug for Discover {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "Kuberntes-Discover (service = {:?})", self.service_name)
        }
    }

    impl Discover {
        /// Creates a new Instance of the Discover-Struct and loads
        /// the Kubernetes Config from the default values
        pub async fn new_default(service_name: String, port: u16) -> Self {
            let client = kube::Client::try_default().await.unwrap();

            Self {
                client,
                service_name,
                nodes: RwLock::new(HashSet::new()),
                port,
            }
        }

        fn parse_endpoints(&self, p: Endpoints) -> Vec<NodeId> {
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
                    let addr = SocketAddrV4::new(ip, self.port);
                    let id = addr_to_id(addr);

                    result.push(id);
                }
            }

            result
        }

        #[tracing::instrument]
        async fn update_single<D>(&self, p: Endpoints, cluster: &Cluster<D>)
        where
            D: AutoDiscover + Send + Sync + 'static,
        {
            let endpoint_ids = self.parse_endpoints(p);

            let mut nodes = self.nodes.write().await;
            for id in endpoint_ids {
                if !nodes.contains(&id) {
                    nodes.insert(id);

                    cluster.add_node(id).await;
                }
            }
        }
    }

    #[async_trait]
    impl AutoDiscover for Discover {
        async fn get_own_id(&self) -> NodeId {
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
                Some(IpNetwork::V4(v4)) => addr_to_id(SocketAddrV4::new(v4.ip(), self.port)),
                _ => 0,
            }
        }

        async fn get_all_nodes(&self) -> HashSet<NodeId> {
            let nodes = self.nodes.read().await;
            nodes.clone()
        }

        #[tracing::instrument]
        async fn watch_nodes<D>(self: Arc<Self>, cluster: Arc<Cluster<D>>)
        where
            D: AutoDiscover + Send + Sync + 'static,
        {
            let api: Api<Endpoints> = Api::namespaced(self.client.clone(), "default");
            let mut lp = ListParams::default();
            lp = lp.fields(&format!("metadata.name={}", self.service_name));

            let mut watcher = Watcher::from_api(api, Some(lp)).await.unwrap();
            loop {
                let tmp = watcher.next_event().await;

                let tmp = match tmp {
                    Some(t) => t,
                    None => {
                        tracing::error!("Watcher returned None");
                        return;
                    }
                };

                match tmp {
                    Event::Started(mut all_p) => {
                        for p in all_p.drain(..) {
                            self.update_single(p, &cluster).await;
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
                            // Some node has been removed
                            let mut removed_nodes = HashSet::clone(&nodes);
                            let subset_ids = self.parse_endpoints(p);
                            drop(nodes);

                            for subset_id in subset_ids.iter() {
                                removed_nodes.remove(subset_id);
                            }

                            tracing::info!("Removing Nodes: {:?}", removed_nodes);

                            let mut write_nodes = self.nodes.write().await;
                            for removed_id in removed_nodes.iter() {
                                write_nodes.remove(removed_id);
                            }
                        } else {
                            drop(nodes);
                            self.update_single(p, &cluster).await;
                        }
                    }
                    Event::Restarted | Event::Removed(_) | Event::Other => {}
                };
            }
        }
    }
}

pub mod files {
    //! This module contains all the Logic for using files to discover the other Nodes in the
    //! cluster

    use async_raft::NodeId;
    use async_trait::async_trait;
    use pnet::ipnetwork::IpNetwork;
    use serde::{Deserialize, Serialize};

    use std::{collections::HashSet, net::SocketAddrV4, sync::Arc};

    use crate::tls::auto::{
        cluster::{addr_to_id, Cluster},
        AutoDiscover,
    };

    #[derive(Debug, Serialize, Deserialize)]
    struct DiscoverConfig {
        nodes: Vec<String>,
    }

    /// The File-Discoverer
    pub struct Discover {
        port: u16,
        nodes: HashSet<NodeId>,
    }

    impl Discover {
        /// Creates a new File-Discoverer that loads the Configuration from the
        /// given Path, which can either be a directory or specific file
        pub fn new(path: String, own_port: u16) -> Self {
            let nodes = Self::read_file(&path);

            Self {
                port: own_port,
                nodes,
            }
        }

        /// Attempts to load all the Nodes registed in the given Config file
        fn read_file(path: &str) -> HashSet<NodeId> {
            let mut result = HashSet::new();

            let metadata = std::fs::metadata(&path).unwrap();
            if metadata.is_file() {
                let data = std::fs::read(path).unwrap();
                let content: DiscoverConfig = match serde_yaml::from_slice(&data) {
                    Ok(c) => c,
                    Err(e) => {
                        log::error!("Parsing Content: {:?}", e);
                        return result;
                    }
                };

                for tmp in content.nodes.iter() {
                    let addr: SocketAddrV4 = tmp.parse().unwrap();
                    let id = addr_to_id(addr);
                    result.insert(id);
                }
            } else {
                for entry in std::fs::read_dir(&path).unwrap() {
                    let sub_path = entry.unwrap().path();
                    for item in Self::read_file(sub_path.to_str().unwrap()).iter() {
                        result.insert(*item);
                    }
                }
            }

            result
        }
    }

    #[async_trait]
    impl AutoDiscover for Discover {
        async fn get_own_id(&self) -> NodeId {
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
                Some(IpNetwork::V4(v4)) => addr_to_id(SocketAddrV4::new(v4.ip(), self.port)),
                _ => 0,
            }
        }

        async fn get_all_nodes(&self) -> HashSet<NodeId> {
            self.nodes.clone()
        }

        // TODO
        async fn watch_nodes<D>(self: Arc<Self>, _raft: Arc<Cluster<D>>)
        where
            D: AutoDiscover + Send + Sync + 'static,
        {
            // This is empty because the File-Discovery mechanism currently does not provide a way
            // to update the configuration while it is running
        }
    }
}
