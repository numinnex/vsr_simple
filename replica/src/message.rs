use client::{Op, MAX_OP_SIZE};

// Discriminator table (singular byte)
// 1 => Request
// 2 => Prepare
// 3 => PrepareOk
// 4 => Commit
// 5 => StartViewChange
// 6 => DoViewChange
// 7 => StartView
// 8 => GetState
// 9 => NewState
// TODO: Add variants to handle state transfers.

#[derive(Debug, PartialEq)]
pub enum Message<Op: Clone> {
    Request {
        client_id: usize,
        request_number: usize,
        op: Op,
    },
    Prepare {
        view_number: usize,
        op: Op,
        op_number: usize,
        commit_number: usize,
    },
    PrepareOk {
        view_number: usize,
        op_number: usize,
    },
    Commit {
        view_number: usize,
        commit_number: usize,
    },
    GetState {
        replica_id: usize,
        view_number: usize,
        op_number: usize,
    },
    NewState {
        view_number: usize,
        log: Vec<Op>,
        op_number: usize,
        commit_number: usize,
    },
    StartViewChange {
        view_number: usize,
        replica_id: usize,
    },
    DoViewChange {
        view_number: usize,
        op_number: usize,
        replica_id: usize,
        commit_number: usize,
        log: Vec<Op>,
    },
    StartView {
        view_number: usize,
        op_number: usize,
        replica_id: usize,
        commit_number: usize,
        log: Vec<Op>,
    },
}

impl Message<Op> {
    pub fn parse_message(buf: &[u8]) -> Self {
        fn parse_op_bytes(buf: &[u8]) -> Vec<Op> {
            let mut position = 0;
            let mut log = Vec::new();

            while position < buf.len() {
                let (op, size) = Op::from_bytes(&buf[position..]);
                log.push(op);
                position += size;
            }
            log
        }

        let discriminator = buf[0];
        match discriminator {
            1 => {
                let client_id = usize::from_le_bytes(buf[1..9].try_into().unwrap());
                let request_number = usize::from_le_bytes(buf[9..17].try_into().unwrap());
                let remainder = &buf[17..];
                let (op, _) = Op::from_bytes(remainder);

                Message::Request {
                    client_id,
                    request_number,
                    op,
                }
            }
            2 => {
                let view_number = usize::from_le_bytes(buf[1..9].try_into().unwrap());
                let commit_number = usize::from_le_bytes(buf[9..17].try_into().unwrap());
                let op_number = usize::from_le_bytes(buf[17..25].try_into().unwrap());
                let remainder = &buf[25..];
                let (op, _) = Op::from_bytes(remainder);
                Message::Prepare {
                    view_number,
                    commit_number,
                    op_number,
                    op,
                }
            }
            3 => {
                let view_number = usize::from_le_bytes(buf[1..9].try_into().unwrap());
                let op_number = usize::from_le_bytes(buf[9..17].try_into().unwrap());
                Message::PrepareOk {
                    view_number,
                    op_number,
                }
            }
            4 => {
                let view_number = usize::from_le_bytes(buf[1..9].try_into().unwrap());
                let commit_number = usize::from_le_bytes(buf[9..17].try_into().unwrap());
                Message::Commit {
                    view_number,
                    commit_number,
                }
            }
            5 => {
                let view_number = usize::from_le_bytes(buf[1..9].try_into().unwrap());
                let replica_id = usize::from_le_bytes(buf[9..17].try_into().unwrap());
                Message::StartViewChange {
                    view_number,
                    replica_id,
                }
            }
            6 => {
                let view_number = usize::from_le_bytes(buf[1..9].try_into().unwrap());
                let op_number = usize::from_le_bytes(buf[9..17].try_into().unwrap());
                let replica_id = usize::from_le_bytes(buf[17..25].try_into().unwrap());
                let commit_number = usize::from_le_bytes(buf[25..33].try_into().unwrap());

                let remainder = &buf[33..];
                let log = parse_op_bytes(remainder);
                Message::DoViewChange {
                    view_number,
                    op_number,
                    replica_id,
                    commit_number,
                    log,
                }
            }
            7 => {
                let view_number = usize::from_le_bytes(buf[1..9].try_into().unwrap());
                let op_number = usize::from_le_bytes(buf[9..17].try_into().unwrap());
                let replica_id = usize::from_le_bytes(buf[17..25].try_into().unwrap());
                let commit_number = usize::from_le_bytes(buf[25..33].try_into().unwrap());

                let remainder = &buf[33..];
                let log = parse_op_bytes(remainder);
                Message::StartView {
                    view_number,
                    op_number,
                    replica_id,
                    commit_number,
                    log,
                }
            },
            8 => {
                Message::GetState {

                }
            },
            9 => {
                Message::NewState {

                }
            }
            _ => unreachable!(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Message::Request {
                client_id,
                request_number,
                op,
            } => {
                let op_bytes = op.to_bytes();
                let op_len = op_bytes.len();
                let length = 1 + 8 + 8 + op_len;
                let mut bytes = Vec::with_capacity(length + 4);
                let discriminator = 1u8;
                bytes.extend_from_slice(&(length as u32).to_le_bytes());
                bytes.extend_from_slice(&discriminator.to_le_bytes());
                bytes.extend_from_slice(&client_id.to_le_bytes());
                bytes.extend_from_slice(&request_number.to_le_bytes());
                bytes.extend_from_slice(&op_bytes);
                bytes
            }
            Message::Prepare {
                view_number,
                op,
                op_number,
                commit_number,
            } => {
                let op_bytes = op.to_bytes();
                let op_len = op_bytes.len();
                let length = 1 + 8 + 8 + 8 + op_len;
                let discriminator = 2u8;
                let mut bytes = Vec::with_capacity(length + 4);
                bytes.extend_from_slice(&(length as u32).to_le_bytes());
                bytes.extend_from_slice(&discriminator.to_le_bytes());
                bytes.extend_from_slice(&view_number.to_le_bytes());
                bytes.extend_from_slice(&commit_number.to_le_bytes());
                bytes.extend_from_slice(&op_number.to_le_bytes());
                bytes.extend_from_slice(&op_bytes);
                bytes
            }
            Message::PrepareOk {
                view_number,
                op_number,
            } => {
                let length = 1 + 8 + 8;
                let discriminator = 3u8;
                let mut bytes = Vec::with_capacity(length + 4);
                bytes.extend_from_slice(&(length as u32).to_le_bytes());
                bytes.extend_from_slice(&discriminator.to_le_bytes());
                bytes.extend_from_slice(&view_number.to_le_bytes());
                bytes.extend_from_slice(&op_number.to_le_bytes());

                bytes
            }
            Message::Commit {
                view_number,
                commit_number,
            } => {
                let length = 1 + 8 + 8;
                let discriminator = 4u8;
                let mut bytes = Vec::with_capacity(length + 4);
                bytes.extend_from_slice(&(length as u32).to_le_bytes());
                bytes.extend_from_slice(&discriminator.to_le_bytes());
                bytes.extend_from_slice(&view_number.to_le_bytes());
                bytes.extend_from_slice(&commit_number.to_le_bytes());

                bytes
            }
            Message::StartViewChange {
                view_number,
                replica_id,
            } => {
                let length = 1 + 8 + 8;
                let mut bytes = Vec::with_capacity(length + 4);
                let discriminator = 5u8;
                bytes.extend_from_slice(&(length as u32).to_le_bytes());
                bytes.extend_from_slice(&discriminator.to_le_bytes());
                bytes.extend_from_slice(&view_number.to_le_bytes());
                bytes.extend_from_slice(&replica_id.to_le_bytes());
                bytes
            }
            Message::DoViewChange {
                view_number,
                op_number,
                replica_id,
                commit_number,
                log,
            } => {
                let length = 1 + 8 + 8 + 8 + 8 + log.len() * MAX_OP_SIZE;
                let mut bytes = Vec::with_capacity(length + 4);
                let discriminator = 6u8;
                bytes.extend_from_slice(&(length as u32).to_le_bytes());
                bytes.extend_from_slice(&discriminator.to_le_bytes());
                bytes.extend_from_slice(&view_number.to_le_bytes());
                bytes.extend_from_slice(&op_number.to_le_bytes());
                bytes.extend_from_slice(&replica_id.to_le_bytes());
                bytes.extend_from_slice(&commit_number.to_le_bytes());
                let op_bytes = log.iter().flat_map(|op| op.to_bytes());
                bytes.extend(op_bytes);
                bytes
            }
            Message::StartView {
                view_number,
                op_number,
                replica_id,
                commit_number,
                log,
            } => {
                let length = 1 + 8 + 8 + 8 + 8 + log.len() * MAX_OP_SIZE;
                let mut bytes = Vec::with_capacity(length);
                let discriminator = 7u8;
                bytes.extend_from_slice(&(length as u32).to_le_bytes());
                bytes.extend_from_slice(&discriminator.to_le_bytes());
                bytes.extend_from_slice(&view_number.to_le_bytes());
                bytes.extend_from_slice(&op_number.to_le_bytes());
                bytes.extend_from_slice(&replica_id.to_le_bytes());
                bytes.extend_from_slice(&commit_number.to_le_bytes());
                let op_bytes = log.iter().flat_map(|op| op.to_bytes());
                bytes.extend(op_bytes);
                bytes
            }
            Message::GetState { replica_id, view_number, op_number } => todo!(),
            Message::NewState { view_number, log, op_number, commit_number } => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn generate_log() -> Vec<Op> {
        std::iter::repeat(Op::Add(69)).take(10).collect()
    }

    fn generate_start_view_message() -> Message<Op> {
        let view_number = 1;
        let op_number = 2;
        let replica_id = 3;
        let commit_number = 4;
        let log = generate_log();

        Message::StartView {
            view_number,
            op_number,
            replica_id,
            commit_number,
            log,
        }
    }

    fn generate_do_view_change_message() -> Message<Op> {
        let view_number = 1;
        let op_number = 2;
        let replica_id = 3;
        let commit_number = 4;
        let log = generate_log();

        Message::DoViewChange {
            view_number,
            op_number,
            replica_id,
            commit_number,
            log,
        }
    }

    #[test]
    fn serializing_and_deserializing_start_view_message_should_maintain_correct_schema() {
        let message = generate_start_view_message();
        let bytes = message.to_bytes();
        let message_deserialized = Message::parse_message(&bytes[4..]);

        assert_eq!(message, message_deserialized);
    }

    #[test]
    fn serializing_and_deserializing_do_view_change_message_should_maintain_correct_schema() {
        let message = generate_do_view_change_message();
        let bytes = message.to_bytes();
        let message_deserialized = Message::parse_message(&bytes[4..]);

        assert_eq!(message, message_deserialized);
    }
}
