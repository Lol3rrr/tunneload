use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use super::Request;

mod sender;
use async_raft::NodeId;
pub use sender::Network;

mod receiver;
pub use receiver::NetworkReceiver;

/// Converts the ID + Port into a useable SocketAddr, which will point to
/// the Target Node
pub fn id_to_addr(id: NodeId, port: u16) -> SocketAddr {
    let parts = id.to_be_bytes();
    let addr = SocketAddrV4::new(Ipv4Addr::new(parts[4], parts[5], parts[6], parts[7]), port);

    SocketAddr::V4(addr)
}

/// Converts the given Address to the right NodeID
pub fn addr_to_id(addr: Ipv4Addr) -> Option<NodeId> {
    let raw_parts = addr.octets();
    let mut parts = [0; 8];
    parts[4..8].copy_from_slice(&raw_parts);

    Some(NodeId::from_be_bytes(parts))
}
