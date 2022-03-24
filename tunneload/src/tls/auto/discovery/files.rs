//! This module contains all the Logic for using files to discover the other Nodes in the
//! cluster

use async_raft::NodeId;
use async_trait::async_trait;
use pnet::ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};

use std::{collections::HashSet, net::SocketAddrV4, sync::Arc};

use crate::tls::auto::{session::addr_to_id, AutoDiscover, NodeUpdateEvent};

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

        let metadata = std::fs::metadata(&path).expect("Could not load metadata for File");
        if metadata.is_file() {
            let data = std::fs::read(path).expect("Reading File");
            let content: DiscoverConfig = match serde_yaml::from_slice(&data) {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("Parsing Content: {:?}", e);
                    return result;
                }
            };

            for tmp in content.nodes.iter() {
                let addr: SocketAddrV4 = match tmp.parse() {
                    Ok(i) => i,
                    Err(_) => continue,
                };
                let id = addr_to_id(addr);
                result.insert(id);
            }
        } else {
            for entry in std::fs::read_dir(&path)
                .expect("We should be able to list all the Files in the Directory")
            {
                let sub_path = entry.expect("The Entry should be Ok").path();
                for item in Self::read_file(
                    sub_path
                        .to_str()
                        .expect("The Patch should be a valid String"),
                )
                .iter()
                {
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
    async fn watch_nodes(
        self: Arc<Self>,
        _raft: tokio::sync::mpsc::UnboundedSender<NodeUpdateEvent>,
    ) {
        // This is empty because the File-Discovery mechanism currently does not provide a way
        // to update the configuration while it is running
    }
}
