use replica::Replica;
use replica_config::ReplicaConfig;

pub(crate) mod client_table;
pub(crate) mod log;
pub(crate) mod message;
pub(crate) mod replica;
pub(crate) mod replica_config;
pub(crate) mod status;
pub(crate) mod stm;

#[derive(Clone)]
pub(crate) enum Op {
    Nop,
    Add(u64),
}

fn main() {
    let config = ReplicaConfig::default();
    let replica = Replica::new(1, config);
}
