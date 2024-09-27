use std::net::{IpAddr, Ipv4Addr, SocketAddr};

// Discriminator table (singular byte)
// 0 => Nop
// 1 => Add

pub const MAX_OP_SIZE: usize = 1 + 8;

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Nop,
    Add(u64),
}

impl Op {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            Op::Nop => {
                bytes.push(0);
            }
            Op::Add(value) => {
                bytes.push(1);
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        };
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> (Self, usize) {
        let discriminator = bytes[0];
        match discriminator {
            0 => (Op::Nop, 1),
            1 => {
                let value = u64::from_le_bytes(bytes[1..9].try_into().unwrap());
                (Op::Add(value), 9)
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
