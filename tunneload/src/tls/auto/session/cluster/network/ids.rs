use std::{
    convert::TryInto,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
};

use async_raft::NodeId;

/// Converts the ID into a useable SocketAddr, which will point to
/// the Target Node
pub fn id_to_addr(id: NodeId) -> SocketAddr {
    let parts = id.to_be_bytes();
    let port = u16::from_be_bytes(parts[2..4].try_into().unwrap());
    let addr = SocketAddrV4::new(Ipv4Addr::new(parts[4], parts[5], parts[6], parts[7]), port);

    SocketAddr::V4(addr)
}

/// Converts the given Address to the right NodeID
pub fn addr_to_id(addr: SocketAddrV4) -> NodeId {
    let raw_port = addr.port().to_be_bytes();
    let raw_ip = addr.ip().octets();
    let mut parts = [0; 8];
    parts[2..4].copy_from_slice(&raw_port);
    parts[4..8].copy_from_slice(&raw_ip);

    NodeId::from_be_bytes(parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion() {
        let input = SocketAddrV4::new(Ipv4Addr::new(10, 11, 12, 13), 8080);

        let id = addr_to_id(input);
        let parsed = id_to_addr(id);

        assert_eq!(SocketAddr::V4(input), parsed);
    }
}
