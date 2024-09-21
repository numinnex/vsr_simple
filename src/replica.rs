use crate::{
    client_table::ClientTable, message::Message, replica_config::ReplicaConfig, status::Status, Op,
};
use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct Replica {
    pub id: usize,
    pub status: Status,
    pub config: ReplicaConfig,
    pub clients_table: ClientTable,
    // TODO: Add state-machine
    pub log: Vec<Op>,
    pub view_number: usize,
    pub op_number: AtomicUsize,
    pub commit_number: AtomicUsize,

    acks: RefCell<HashMap<usize, usize>>,
}

impl Replica {
    pub fn new(id: usize, config: ReplicaConfig) -> Self {
        Self {
            id,
            config,
            status: Default::default(),
            clients_table: Default::default(),
            log: Default::default(),
            view_number: Default::default(),
            op_number: Default::default(),
            commit_number: Default::default(),
            acks: Default::default(),
        }
    }

    pub fn quorum(&self) -> usize {
        let replicas_count = self.config.replicas.len();
        replicas_count / 2 + 1
    }

    pub fn ack_op(&self, op_number: usize) {
        self.acks
            .borrow_mut()
            .entry(op_number)
            .and_modify(|ack| *ack += 1)
            .or_insert(1);
    }

    pub fn is_primary(&self) -> bool {
        self.id == self.config.primary_id(self.view_number)
    }

    pub fn commit_number(&self) -> usize {
        self.commit_number.load(Ordering::Acquire)
    }

    pub fn on_message(&self, message: Message<Op>) {
        match message {
            Message::Request {
                client_id,
                request_number,
                op,
            } => {
                // Check if you are primary, otherwise drop the message.
                // Increment op-number.
                // Send `Prepare` message to other replicas.
            }
            Message::Prepare {
                view_number,
                op,
                op_number,
                commit_number,
            } => {
                // Incremenet op-number.
                // Append to log.
                // Update clients table.
                // Send `PrepareOk` to primary.
                self.on_prepare(view_number, op_number, op, commit_number)
            }
            Message::PrepareOk {
                view_number,
                op_number,
            } => {
                // Check if received a qourum of `PrepareOk`
                // Call the service code (app logic).
                // Increment the commit-number.
                // Reply to the client.
                // Update clients table.
            }
            Message::Commit {
                view_number,
                commit_number,
            } => {
                // Check if the request is in the log
                // Call the service code (app logic).
                // Increment the commit-number.
                // Update clients table.
            }
        }
    }
}

// Handlers
impl Replica {
    fn on_request(&self, client_id: usize, request_number: usize, op: Op) {
        assert!(self.is_primary());
        if self.status != Status::Normal {
            // TODO: Impl mechanism that teaches client to try again later on.
            return;
        }
        // Check in client table, whether the request_number is subsequent.
        // If it's smaller, drop the request (duplicate)
        // If it's equal to current request_number, resend the response.

        // Append to log.
        self.append_to_log(op);
        // Send `Prepare` message to backups.
    }

    fn on_prepare(&self, view_number: usize, op_number: usize, op: Op, commit_number: usize) {
        assert!(!self.is_primary());
        if self.view_number != view_number {
            // This means that our backup has felt behind during the `ViewChange` protocol.
            // Initiate the recovery process.
        }

        let current_op_number = self.op_number.load(Ordering::Acquire);
        if op_number <= current_op_number {
            return;
        }
        if op_number > current_op_number + 1 {
            // Initiate state transfer
            return;
        }

        // Append op to the log.
        for op_idx in self.commit_number()..commit_number {
            // Commit op
        }
        // Send message back to primary.
    }

    fn on_prepare_ok(&self, view_number: usize, op_number: usize) {
        assert!(self.is_primary());
        assert_eq!(self.view_number, view_number);

        self.ack_op(op_number);
        if *self.acks.borrow().get(&op_number).unwrap() == self.quorum() {
            // Commit op
            // Send response to the client.
        }
    }

    fn on_commit(&self, view_number: usize, commit_number: usize) {
        if self.status != Status::Normal {
            return;
        }
        if view_number < self.view_number {
            return;
        }
        assert_eq!(self.status, Status::Normal);
        assert_eq!(self.view_number, view_number);

        let current_commit_number = self.commit_number.load(Ordering::Acquire);
        if commit_number > current_commit_number{
            // Perform state transfer
            return;
        }

        for op_idx in current_commit_number..commit_number {
            // Commit the op
        }
    }
}
