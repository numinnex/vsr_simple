use crate::{client_table::ClientTable, message::Message, replica_config::ReplicaConfig, Op};
use std::sync::atomic::AtomicUsize;

pub struct Replica {
    pub config: ReplicaConfig,
    pub clients_table: ClientTable,
    pub log: Vec<Op>,
    pub id: usize,
    pub view_number: usize,
    pub op_number: AtomicUsize,
    pub commit_number: AtomicUsize,
}

impl Replica {
    pub fn new(id: usize, config: ReplicaConfig) -> Self {
        Self {
            id,
            config,
            clients_table: Default::default(),
            log: Default::default(),
            view_number: Default::default(),
            op_number: Default::default(),
            commit_number: Default::default(),
        }
    }

    pub fn on_message(&self, message: Message<Op>) {
        match message {
            Message::Request { client_id, request_number, op } => {

            },
            Message::Prepare { view_number, op, op_number, commit_number } => todo!(),
            Message::PrepareOk { view_number, op_number } => todo!(),
            Message::Commit { view_number, commit_number } => todo!(),
        }
    }
}
