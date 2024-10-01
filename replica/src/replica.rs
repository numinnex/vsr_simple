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

struct ViewSnapshot<Op> {
    view_number: usize,
    op_number: usize,
    commit_number: usize,
    log: Vec<Op>,
}

impl ViewSnapshot<Op> {
    pub fn new(view_number: usize, op_number: usize, commit_number: usize, log: Vec<Op>) -> Self {
        Self {
            view_number,
            op_number,
            commit_number,
            log,
        }
    }
}

pub struct Replica {
    pub id: usize,
    pub status: RefCell<Status>,
    pub config: ReplicaConfig,
    pub clients_table: ClientTable,
    //TODO: Op in the log should be ref counted.
    pub log: RefCell<Vec<Op>>,
    pub view_number: AtomicUsize,
    pub op_number: AtomicUsize,
    pub commit_number: AtomicUsize,

    // Used during view change to choose the new best log.
    view_snapshot: RefCell<Option<ViewSnapshot<Op>>>,
    acks: RefCell<HashMap<usize, usize>>,
    backup_idle_ticks: AtomicUsize,
    view_change_counter: RefCell<HashMap<usize, usize>>,
    do_view_change_counter: RefCell<HashMap<usize, usize>>,
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
            view_snapshot: Default::default(),
            acks: Default::default(),
            backup_idle_ticks: Default::default(),
            view_change_counter: Default::default(),
            do_view_change_counter: Default::default(),
            stm: Default::default(),
        }
    }

    fn send_msg_to_primary(&self, message: Message<Op>) {
        todo!();
    }

    fn send_msg_to_replica(&self, replica_id: usize, message: Message<Op>) {
        let addr = self.config.get_replica_address(replica_id);
        let mut stream = TcpStream::connect(addr).unwrap();
        let bytes = message.to_bytes();
        stream
            .write_all(&bytes)
            .expect("Failed to send message to replica");
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

                let mut len_bytes = vec![0u8; 4];
                stream
                    .read_exact(&mut len_bytes)
                    .expect("Failed to read response from replica.");
                let length = u32::from_le_bytes(len_bytes.try_into().unwrap());
                let mut buffer = vec![0u8; length as _];
                stream
                    .read_exact(&mut buffer)
                    .expect("Failed to read response from replica.");
                let message = Message::parse_message(&buffer);
                let thread_id = std::thread::current();
                println!(
                    "{:?} Received message from replica: {:?}",
                    thread_id, message
                );

                self.on_message(&mut stream, message);
            }
        }
    }

    fn number_of_replicas(&self) -> usize {
        self.config.replicas.len()
    }

    fn quorum(&self) -> usize {
        let replicas_count = self.number_of_replicas();
        replicas_count / 2 + 1
    }

    pub fn quorum_for_op(&self, op_number: usize) -> bool {
        let acks = *self.acks.borrow().get(&op_number).unwrap();
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
        self.id == self.config.primary_id(self.view_number())
    }

    pub fn commit_number(&self) -> usize {
        self.commit_number.load(Ordering::Acquire)
    }

    pub fn view_number(&self) -> usize {
        self.view_number.load(Ordering::Acquire)
    }

    pub fn op_number(&self) -> usize {
        self.op_number.load(Ordering::Acquire)
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
            Message::StartViewChange {
                view_number,
                replica_id,
            } => {
                self.on_start_view_change(view_number, replica_id);
            }
            Message::DoViewChange {
                view_number,
                op_number,
                replica_id,
                commit_number,
                log,
            } => {
                self.on_do_view_change(view_number, op_number, replica_id, commit_number, log);
            }
            Message::StartView {
                view_number,
                op_number,
                replica_id,
                commit_number,
                log,
            } => {
                self.on_start_view(view_number, op_number, replica_id, commit_number, log);
            }
            Message::GetState {
                replica_id,
                view_number,
                op_number,
            } => {
                self.on_get_state(replica_id, view_number, op_number);
            }
            Message::NewState {
                view_number,
                log,
                op_number,
                commit_number,
            } => {
                self.on_new_state(view_number, log, op_number, commit_number);
            }
        }
    }
}

// Handlers
impl Replica {
    fn on_request(&self, client_id: usize, request_number: usize, op: Op) {
        assert!(self.is_primary());
        if *self.status.borrow() != Status::Normal {
            // TODO: Impl mechanism that teaches client to try again later on.
            return;
        }
        // Check in client table, whether the request_number is subsequent.
        // If it's smaller, drop the request (duplicate)
        // If it's equal to current request_number, resend the response.

        // Append to log
        self.append_to_log(op.clone());
        // Ack the op ourselves
        let op_number = self.op_number.load(Ordering::Acquire);
        self.acks.borrow_mut().insert(1, op_number);
        // Send `Prepare` message to backups
        let commit_number = self.commit_number();
        let view_number = self.view_number();
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
        self.backup_idle_ticks.store(0, Ordering::Relaxed);
        assert!(!self.is_primary());
        if self.view_number() != view_number {
            // This means that our backup has felt behind during the `ViewChange` protocol.
            // Initiate the recovery process.
        }

        let current_op_number = self.op_number.load(Ordering::Acquire);
        if op_number <= current_op_number {
            return;
        }
        if op_number > current_op_number + 1 {
            // Initiate state transfer
            self.state_transfer();
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
            view_number: self.view_number(),
            op_number: self.op_number.load(Ordering::Acquire),
        };
        println!(
            "Sending message: {:?} to replica as response for prepare message",
            message
        );
        let bytes = message.to_bytes();
        stream
            .write_all(&bytes)
            .expect("Failed to send PrepareOk back to leader");
    }

    fn on_prepare_ok(&self, view_number: usize, op_number: usize) {
        assert!(self.is_primary());
        assert_eq!(self.view_number(), view_number);

        self.ack_op(op_number);
        if self.quorum_for_op(op_number) {
            // Commit op
            self.commit_op(op_number - 1);
            // Send response to the client.
        }
    }

    fn on_commit(&self, view_number: usize, commit_number: usize) {
        self.backup_idle_ticks.store(0, Ordering::Relaxed);
        if *self.status.borrow() != Status::Normal {
            return;
        }
        let backup_view_number = self.view_number();
        if view_number < backup_view_number {
            return;
        }
        assert_eq!(*self.status.borrow(), Status::Normal);
        assert_eq!(backup_view_number, view_number);

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

    pub fn on_timer(&self) {
        if self.is_primary() {
            // Send the `Commit` message to our backups.
            let view_number = self.view_number();
            let commit_number = self.commit_number();
            let message = Message::Commit {
                view_number,
                commit_number,
            };

            self.send_msg_to_replicas(message);
        } else {
            let idle_ticks = self.backup_idle_ticks.fetch_add(1, Ordering::Relaxed);
            if idle_ticks > 0 {
                // Send the `StartViewChange` message to other backups.
                self.view_change();
            }
        }
    }

    fn state_transfer(&self) {
        self.status.replace(Status::Recovery);
        let message = Message::GetState {
            replica_id: self.id,
            view_number: self.view_number(),
            op_number: self.op_number(),
        };
        self.send_msg_to_primary(message);
    }

    fn ack_start_view_change(&self, view_number: usize) {
        let mut view_change_counter = self.view_change_counter.borrow_mut();
        view_change_counter
            .entry(view_number)
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    fn set_view_change_status(&self) {
        self.status.replace(Status::ViewChange);
    }

    fn set_view_number(&self, view_number: usize) {
        self.view_number.store(view_number, Ordering::Release);
    }

    fn set_op_number(&self, op_number: usize) {
        self.op_number.store(op_number, Ordering::Release);
    }

    fn enter_start_view_change_stage(&self, view_number: usize) {
        self.set_view_number(view_number);
        self.set_view_change_status();
    }

    fn view_change(&self) {
        let current_view_number = self.view_number.load(Ordering::Acquire);
        let view_number = (current_view_number + 1) % self.number_of_replicas();
        self.enter_start_view_change_stage(view_number);
        self.ack_start_view_change(view_number);
        let message = Message::StartViewChange {
            view_number,
            replica_id: self.id,
        };
        self.send_msg_to_replicas(message);
    }

    fn on_start_view(
        &self,
        view_number: usize,
        op_number: usize,
        replica_id: usize,
        commit_number: usize,
        log: Vec<Op>,
    ) {
        self.status.replace(Status::Normal);
        self.set_view_number(view_number);
        self.set_op_number(op_number);
        *self.log.borrow_mut() = log;
        // Commit uncommited ops.
        // There is no need to send `PrepareOk` messages to the primary
        // since we preemptively commit uncommited ops on primary
        // once quorum for `DoViewChange` is reached.
        let current_commit_number = self.commit_number();
        if current_commit_number < commit_number {
            for uncommited_op in current_commit_number..commit_number {
                self.commit_op(uncommited_op);
            }
        }
    }

    fn on_start_view_change(&self, view_number: usize, replica_id: usize) {
        assert!(self.id != replica_id);
        if view_number > self.view_number() {
            self.set_view_change_status();
            // Ack our own `StartViewChange`
            self.ack_start_view_change(view_number);
            // Broadcast the `StartViewChange` message to other backups.
            let message = Message::StartViewChange {
                view_number,
                replica_id: self.id,
            };
            // Send the message to ourselves..
            self.send_msg_to_replicas(message);
        }
        // Ack the incomming `StartViewChange`
        self.ack_start_view_change(view_number);

        let view_change_counter = self.view_change_counter.borrow();
        let start_view_changes = view_change_counter.get(&view_number).unwrap();
        if *start_view_changes >= self.quorum() {
            // Send message to new primary.
            let log = self.log.borrow().clone();
            let op_number = self.op_number();
            let message = Message::DoViewChange {
                view_number,
                op_number,
                commit_number: self.commit_number(),
                replica_id: self.id,
                log,
            };
            self.send_msg_to_primary(message);
        }
    }

    fn on_do_view_change(
        &self,
        view_number: usize,
        op_number: usize,
        replica_id: usize,
        commit_number: usize,
        log: Vec<Op>,
    ) {
        assert!(*self.status.borrow() != Status::Normal);
        let mut do_view_change_counter = self.do_view_change_counter.borrow_mut();
        let do_views = do_view_change_counter
            .entry(view_number)
            .and_modify(|v| *v += 1)
            .or_insert(1);

        // Store the best candidate for log transplant.
        let mut view_snapshot = self.view_snapshot.borrow_mut();
        if let Some(snapshot) = &mut *view_snapshot {
            if view_number > snapshot.view_number {
                *snapshot = ViewSnapshot::new(view_number, op_number, commit_number, log);
            } else if op_number > snapshot.op_number {
                *snapshot = ViewSnapshot::new(view_number, op_number, commit_number, log);
            }
        } else {
            *view_snapshot = Some(ViewSnapshot::new(
                view_number,
                op_number,
                commit_number,
                log,
            ));
        }

        if *do_views >= self.quorum() {
            assert!(self.view_snapshot.borrow().is_some());
            // Take log from the most up to date replica.
            let mut snapshot = self.view_snapshot.borrow_mut();
            let snapshot = snapshot.take().unwrap();
            let log = snapshot.log;
            let commit_number = snapshot.commit_number;
            let op_number = snapshot.op_number;
            *self.log.borrow_mut() = log.clone();
            // Set op number and commit number
            self.set_op_number(op_number);
            // Set the new view number.
            self.set_view_number(view_number);
            // Switch back to normal state.
            self.status.replace(Status::Normal);
            // Commit uncommited ops.
            let current_commit_number = self.commit_number();
            if current_commit_number < commit_number {
                for uncommited_op in current_commit_number..commit_number {
                    self.commit_op(uncommited_op);
                }
            }

            // Send `StartView` Message to other replicas.
            let message = Message::StartView {
                view_number,
                commit_number,
                op_number,
                replica_id: self.id,
                log,
            };
            self.send_msg_to_replicas(message);
        }
    }
    fn on_get_state(&self, replica_id: usize, view_number: usize, op_number: usize) {
        let current_view_number = self.view_number();
        if current_view_number != view_number {
            return;
        }
        if *self.status.borrow() == Status::ViewChange {
            return;
        }
        let log = self.log.borrow();
        let message = Message::NewState {
            view_number: current_view_number,
            log: log[..op_number].to_owned(),
            op_number: self.op_number(),
            commit_number: self.commit_number(),
        };
        self.send_msg_to_replica(replica_id, message);
    }
    fn on_new_state(
        &self,
        view_number: usize,
        log: Vec<Op>,
        op_number: usize,
        commit_number: usize,
    ) {
        assert_eq!(*self.status.borrow(), Status::Recovery);
        if self.view_number() != view_number {
            return;
        }
    }
}
