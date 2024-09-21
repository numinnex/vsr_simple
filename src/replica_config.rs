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

    pub fn primary_id(&self, view_number: usize) -> usize {
        let idx = view_number % self.replicas.len();
        self.replicas[idx]
    }
}
