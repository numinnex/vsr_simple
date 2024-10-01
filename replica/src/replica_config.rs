use std::net::SocketAddr;

#[derive(Default, Clone)]
pub struct ReplicaConfig {
    pub addresses: Vec<SocketAddr>,
    pub replicas: Vec<usize>,
}

impl ReplicaConfig {
    pub fn append_new(&mut self, id: usize, address: SocketAddr) {
        self.addresses.push(address);
        self.replicas.push(id);
    }

    pub fn get_replica_address(&self, replica_id: usize) -> SocketAddr {
        self.addresses[replica_id]
    }

    pub fn primary_id(&self, view_number: usize) -> usize {
        let idx = view_number % self.replicas.len();
        self.replicas[idx]
    }
}
