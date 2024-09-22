pub struct Client {
    pub id: usize,
    pub request_number: usize,
}

impl Client {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            request_number: 0,
        }
    }
}
