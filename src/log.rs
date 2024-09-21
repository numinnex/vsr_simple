use std::sync::atomic::Ordering;
use crate::{replica::Replica, Op};

impl Replica {
    pub fn append_to_log(&self, op: Op) {
        self.op_number.fetch_add(1, Ordering::AcqRel);
    }
}
