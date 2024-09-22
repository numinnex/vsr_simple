use client::Op;

use crate::replica::Replica;
use std::sync::atomic::Ordering;

impl Replica {
    pub fn append_to_log(&self, op: Op) {
        self.op_number.fetch_add(1, Ordering::AcqRel);
    }
}
