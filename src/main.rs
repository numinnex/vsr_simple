pub(crate) mod client_table;
pub(crate) mod message;
pub(crate) mod replica;
pub(crate) mod replica_config;
pub(crate) mod status;

#[derive(Clone)]
pub(crate) enum Op {
    Nop,
}

fn main() {
    println!("Hello, world!");
}
