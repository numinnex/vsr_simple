use crate::{client_table::ClientTable, message::Message, replica_config::ReplicaConfig, status::Status, Op};
use std::{cell::RefCell, collections::HashMap, sync::atomic::{AtomicUsize, Ordering}};

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

    acks: RefCell<HashMap<usize, usize>>
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

impl Replica {
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

        // Ack the op,

    }
}
