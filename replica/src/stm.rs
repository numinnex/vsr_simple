use std::cell::RefCell;

use client::Op;

#[derive(Default)]
pub struct StateMachine {
    inner: RefCell<u64>,
}

impl StateMachine {
    pub fn apply(&self, op: Op) {
        match op {
            Op::Add(val) => {
                let mut inner = self.inner.borrow_mut();
                *inner += val;
            }
            Op::Nop => {
                println!("Applying No operation to state machine");
            }
        }
    }
}
