use client::Op;

// Discriminator table (singular byte)
// 1 => Request
// 2 => Prepare
// 3 => PrepareOk
// 4 => Commit
// TODO: Add variants to handle state transfers.

#[derive(Debug)]
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
    // TODO: Add variants to handle state tranfers.
}

impl Message<Op> {
    pub fn parse_message(buf: &[u8]) -> Self {
        let discriminator = buf[0];
        match discriminator {
            1 => {
                let client_id = usize::from_le_bytes(buf[1..9].try_into().unwrap());
                let request_number = usize::from_le_bytes(buf[9..17].try_into().unwrap());
                let remainder = &buf[17..];
                let op = Op::from_bytes(remainder);

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
                let op = Op::from_bytes(remainder);
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
                let mut bytes = Vec::with_capacity(length);
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
                let mut bytes = Vec::with_capacity(length);
                bytes.extend_from_slice(&(length as u32).to_le_bytes());
                bytes.extend_from_slice(&discriminator.to_le_bytes());
                bytes.extend_from_slice(&view_number.to_le_bytes());
                bytes.extend_from_slice(&op_number.to_le_bytes());

                bytes
            }
            Message::Commit {
                view_number,
                commit_number,
            } => todo!(),
        }
    }
}
