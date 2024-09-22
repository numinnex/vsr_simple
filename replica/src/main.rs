use client::ADDRESSES;
use replica::Replica;
use replica_config::ReplicaConfig;
use std::net::TcpListener;

pub(crate) mod client_table;
pub(crate) mod log;
pub(crate) mod message;
pub(crate) mod replica;
pub(crate) mod replica_config;
pub(crate) mod status;
pub(crate) mod stm;

fn main() {
    let mut config = ReplicaConfig::default();
    for (id, addr) in ADDRESSES.into_iter().enumerate() {
        config.append_new(addr, id);
    }

    let mut threads = Vec::new();
    for addr in ADDRESSES {
        let thread = std::thread::spawn(move || {
            let listener = TcpListener::bind(addr).expect("Failed to bind to socketerino");
        });
        threads.push(thread);
    }

    let _: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
}
