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
            _ => unreachable!(),
        }
    }
}
