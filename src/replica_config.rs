use std::net::IpAddr;

pub struct ReplicaConfig {
    pub addresses: Vec<IpAddr>,
    pub replicas: Vec<usize>,
}

impl ReplicaConfig {
    pub fn new() -> Self {
        Self {
            addresses: Default::default(),
            replicas: Default::default(),
        }
    }
}
