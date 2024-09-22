use std::net::SocketAddr;

#[derive(Default)]
pub struct ReplicaConfig {
    pub addresses: Vec<SocketAddr>,
    pub replicas: Vec<usize>,
}

impl ReplicaConfig {
    pub fn append_new(&mut self, address: SocketAddr, id: usize) {
        self.addresses.push(address);
        self.replicas.push(id);
    }

    pub fn primary_id(&self, view_number: usize) -> usize {
        let idx = view_number % self.replicas.len();
        self.replicas[idx]
    }
}
