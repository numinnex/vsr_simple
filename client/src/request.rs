use client::Op;

pub struct Request {
    pub client_id: usize,
    pub request_number: usize,
    op: Op,
}

impl Request {
    pub fn new(client_id: usize, request_number: usize, op: Op) -> Self {
        Self {
            client_id,
            request_number,
            op,
        }
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let op_bytes = self.op.to_bytes();
        let op_len = op_bytes.len();
        let length = 1 + 8 + 8 + op_len;
        let mut bytes = Vec::with_capacity(length + 4);
        let discriminator = 1u8;
        bytes.extend_from_slice(&(length as u32).to_le_bytes());
        bytes.extend_from_slice(&discriminator.to_le_bytes());
        bytes.extend_from_slice(&self.client_id.to_le_bytes());
        bytes.extend_from_slice(&self.request_number.to_le_bytes());
        bytes.extend_from_slice(&op_bytes);
        bytes
    }
}
