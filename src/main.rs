pub(crate) mod client_table;
pub(crate) mod message;
pub(crate) mod replica;
pub(crate) mod replica_config;

#[derive(Clone)]
pub(crate) enum Op {
    Nop,
}

fn main() {
    println!("Hello, world!");
}
