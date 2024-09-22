use std::net::{IpAddr, Ipv4Addr, SocketAddr};
#[derive(Clone)]
pub enum Op {
    Nop,
    Add(u64),
}

pub const ADDRESSES: [SocketAddr; 3] = [
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 69),
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 2137),
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 42069),
];
