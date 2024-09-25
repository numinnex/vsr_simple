use client::Op;

use crate::{
    client_table::ClientTable, message::Message, replica_config::ReplicaConfig, status::Status,
    stm::StateMachine,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct Replica {
    pub id: usize,
    pub status: Status,
    pub config: ReplicaConfig,
    pub clients_table: ClientTable,
    //TODO: Op in the log should be ref counted.
    pub log: RefCell<Vec<Op>>,
    pub view_number: usize,
    pub op_number: AtomicUsize,
    pub commit_number: AtomicUsize,

    acks: RefCell<HashMap<usize, usize>>,
    stm: StateMachine,
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
            stm: Default::default(),
        }
    }

    // Couldn't be bothered creating proper connections cache, instead just connect everytime
    // a new request is being made to the replica.
    fn send_msg_to_replicas(&self, message: Message<Op>) {
        for (replica_id, addr) in self
            .config
            .replicas
            .iter()
            .zip(self.config.addresses.iter())
        {
            if *replica_id != self.id {
                println!(
                    "Sending message: {:?} to replica with id: {}",
                    message, replica_id
                );
                let bytes = message.to_bytes();
                let mut stream = TcpStream::connect(addr).unwrap();
                stream
                    .write_all(&bytes)
                    .expect("Failed to send message to replica");
                println!("written bytes");
                let mut len_bytes = vec![0u8; 4];
                stream
                    .read_exact(&mut len_bytes)
                    .expect("Failed to read response from replica.");
                let length = u32::from_le_bytes(len_bytes.try_into().unwrap());
                let mut buffer = vec![0u8; length as _];
                println!("reading len: {}", length);
                stream
                    .read_exact(&mut buffer)
                    .expect("Failed to read response from replica.");
                let message = Message::parse_message(&buffer);
                println!("Received message from replica: {:?}", message);
                self.on_message(&mut stream, message);
            }
        }
    }

    fn quorum(&self) -> usize {
        let replicas_count = self.config.replicas.len();
        replicas_count / 2 + 1
    }

    pub fn quorum_for_op(&self, op_number: usize) -> bool {
        let acks = *self.acks.borrow().get(&op_number).unwrap();
        println!("acks: {}, acks required for quorum: {}", acks, self.quorum());
        acks == self.quorum()
    }

    pub fn commit_op(&self, op_number: usize) {
        let log = self.log.borrow();
        let op = &log[op_number];
        self.stm.apply(op.clone());
        self.commit_number.fetch_add(1, Ordering::AcqRel);
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

    pub fn on_message(&self, stream: &mut TcpStream, message: Message<Op>) {
        match message {
            Message::Request {
                client_id,
                request_number,
                op,
            } => {
                // Check if you are primary, otherwise drop the message.
                // Increment op-number.
                // Send `Prepare` message to other replicas.
                self.on_request(client_id, request_number, op);
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
                self.on_prepare(stream, view_number, op_number, op, commit_number)
            }
            Message::PrepareOk {
                view_number,
                op_number,
            } => {
                // Check if received a quorum of `PrepareOk`
                // Call the service code (app logic).
                // Increment the commit-number.
                // Reply to the client.
                // Update clients table.
                self.on_prepare_ok(view_number, op_number);
            }
            Message::Commit {
                view_number,
                commit_number,
            } => {
                // Check if the request is in the log
                // Call the service code (app logic).
                // Increment the commit-number.
                // Update clients table.
                self.on_commit(view_number, commit_number);
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
        self.append_to_log(op.clone());
        // Send `Prepare` message to backups.
        let op_number = self.op_number.load(Ordering::Acquire);
        let commit_number = self.commit_number();
        let view_number = self.view_number;
        let message = Message::Prepare {
            view_number,
            op,
            op_number,
            commit_number,
        };
        self.send_msg_to_replicas(message);
    }

    fn on_prepare(
        &self,
        stream: &mut TcpStream,
        view_number: usize,
        op_number: usize,
        op: Op,
        commit_number: usize,
    ) {
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
        self.append_to_log(op);
        for op_number in self.commit_number()..commit_number {
            // Commit op
            self.commit_op(op_number);
        }
        // Send message back to primary.
        let message = Message::PrepareOk {
            view_number: self.view_number,
            op_number: self.op_number.load(Ordering::Acquire),
        };
        let bytes = message.to_bytes();
        stream
            .write_all(&bytes)
            .expect("Failed to send PrepareOk back to leader");
        // stream
    }

    fn on_prepare_ok(&self, view_number: usize, op_number: usize) {
        assert!(self.is_primary());
        assert_eq!(self.view_number, view_number);

        println!("on_prepare_ok");
        self.ack_op(op_number);
        if self.quorum_for_op(op_number) {
            println!("quorum");
            // Commit op
            self.commit_op(op_number);
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
        if commit_number > current_commit_number {
            // Perform state transfer
            return;
        }

        for op_number in current_commit_number..commit_number {
            // Commit the op
            self.commit_op(op_number);
        }
    }
}
