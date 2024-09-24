use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use client::Op;

// Discriminator table (singular byte)
// 1 => Request
// 2 => Prepare
// 3 => PrepareOk
// 4 => Commit
// 5 => NewNode
// TODO: Add variants to handle state transfers.

#[derive(Debug)]
pub enum Message<Op: Clone> {
    Request {
        client_id: usize,
        request_number: usize,
        op: Op,
    },
    Prepare {
        view_number: usize,
        op: Op,
        op_number: usize,
        commit_number: usize,
    },
    PrepareOk {
        view_number: usize,
        op_number: usize,
    },
    Commit {
        view_number: usize,
        commit_number: usize,
    },
    NewNode {
        replica_id: usize,
        addr: SocketAddr,
    }

    // TODO: Add variants to handle state tranfers.
}

impl Message<Op> {
    pub fn parse_message(buf: &[u8]) -> Self {
        let discriminator = buf[0];
        match discriminator {
            1 => {
                let client_id = usize::from_le_bytes(buf[1..9].try_into().unwrap());
                let request_number = usize::from_le_bytes(buf[9..17].try_into().unwrap());
                let remainder = &buf[17..];
                let op = Op::from_bytes(remainder);

                Message::Request {
                    client_id,
                    request_number,
                    op,
                }
            },
            5 => {
                let replica_id = usize::from_le_bytes(buf[1..9].try_into().unwrap());
                let octets = u32::from_be_bytes(buf[9..13].try_into().unwrap());
                let port = u16::from_le_bytes(buf[13..15].try_into().unwrap());
                let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::from(octets)), port);
                Message::NewNode {
                    replica_id,
                    addr
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Message::Request {
                client_id,
                request_number,
                op,
            } => {
                let op_bytes = op.to_bytes();
                let op_len = op_bytes.len();
                let length = 1 + 8 + 8 + op_len;
                let mut bytes = Vec::with_capacity(length + 4);
                let discriminator = 1u8;
                bytes.extend_from_slice(&(length as u32).to_le_bytes());
                bytes.extend_from_slice(&discriminator.to_le_bytes());
                bytes.extend_from_slice(&client_id.to_le_bytes());
                bytes.extend_from_slice(&request_number.to_le_bytes());
                bytes.extend_from_slice(&op_bytes);
                bytes
            }
            Message::Prepare {
                view_number,
                op,
                op_number,
                commit_number,
            } => todo!(),
            Message::PrepareOk {
                view_number,
                op_number,
            } => todo!(),
            Message::Commit {
                view_number,
                commit_number,
            } => todo!(),
            Message::NewNode { replica_id, addr } => {
                let length = 1 + 8 + 4 + 2;
                let mut bytes = Vec::with_capacity(length + 4);
                let discriminator = 5u8;
                let octets = match addr.ip() {
                    IpAddr::V4(ipv4_addr) => {
                        ipv4_addr.octets()
                    },
                    IpAddr::V6(_) => panic!("IpV6 not supported")
                };
                let port = addr.port().to_le_bytes();
                bytes.extend_from_slice(&(length as u32).to_le_bytes());
                bytes.extend_from_slice(&discriminator.to_le_bytes());
                bytes.extend_from_slice(&replica_id.to_le_bytes());
                bytes.extend_from_slice(&octets);
                bytes.extend_from_slice(&port);
                bytes
            }
        }
    }
}
