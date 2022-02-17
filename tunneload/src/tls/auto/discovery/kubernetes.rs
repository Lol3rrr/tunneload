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
    tls::auto::{session::addr_to_id, AutoDiscover, NodeUpdateEvent},
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
        let subsets = p.subsets.unwrap_or_else(Vec::new);

        let mut result = Vec::new();

        for subset in subsets.iter() {
            let addresses = match subset.addresses.as_ref() {
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

    #[tracing::instrument(skip(p, cluster))]
    async fn update_single(
        &self,
        p: Endpoints,
        cluster: &tokio::sync::mpsc::UnboundedSender<NodeUpdateEvent>,
    ) {
        let endpoint_ids = self.parse_endpoints(p);

        let mut nodes = self.nodes.write().await;
        for id in endpoint_ids {
            if !nodes.contains(&id) {
                nodes.insert(id);

                if let Err(e) = cluster.send(NodeUpdateEvent::Add(id)) {
                    tracing::error!("Sending NodeUpdateEvent to Cluster: {:?}", e);
                }
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
    async fn watch_nodes(
        self: Arc<Self>,
        update_queue: tokio::sync::mpsc::UnboundedSender<NodeUpdateEvent>,
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
                    tracing::error!("Watcher returned None");
                    return;
                }
            };

            match tmp {
                Event::Started(mut all_p) => {
                    for p in all_p.drain(..) {
                        self.update_single(p, &update_queue).await;
                    }
                }
                Event::Updated(p) => {
                    let subsets = match p.subsets.as_ref() {
                        Some(s) => s.clone(),
                        None => Vec::new(),
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
                        self.update_single(p, &update_queue).await;
                    }
                }
                Event::Restarted | Event::Removed(_) | Event::Other => {}
            };
        }
    }
}
