use std::net::{IpAddr, Ipv4Addr, SocketAddr};

// Discriminator table (singular byte)
// 0 => Nop
// 1 => Add

#[derive(Debug, Clone)]
pub enum Op {
    Nop,
    Add(u64),
}

impl Op {
    pub fn to_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            Op::Nop => {
                bytes.extend_from_slice(&0u8.to_le_bytes());
            }
            Op::Add(value) => {
                bytes.extend_from_slice(&1u8.to_le_bytes());
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        };
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let discriminator = bytes[0];
        match discriminator {
            0 => Op::Nop,
            1 => {
                let value = u64::from_le_bytes(bytes[1..9].try_into().unwrap());
                Op::Add(value)
            }
            _ => unreachable!(),
        }
    }
}

pub const ADDRESSES: [SocketAddr; 3] = [
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1337),
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 2137),
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 6969),
];
