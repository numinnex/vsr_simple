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
