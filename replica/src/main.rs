use client::ADDRESSES;
use message::Message;
use replica::Replica;
use replica_config::ReplicaConfig;
use std::{
    io::Read,
    net::{TcpListener, TcpStream},
    rc::Rc,
};

pub(crate) mod client_table;
pub(crate) mod log;
pub(crate) mod message;
pub(crate) mod replica;
pub(crate) mod replica_config;
pub(crate) mod status;
pub(crate) mod stm;

fn main() {
    let mut config = ReplicaConfig::default();
    let mut threads = Vec::new();

    for (id, addr) in ADDRESSES.into_iter().enumerate() {
        let config_mut = &mut config;
        config_mut.append_new(id, addr);
        let config = config.clone();
        let thread = std::thread::spawn(move || {
            let replica = Rc::new(Replica::new(id, config));
            println!("Created node with addr: {}, id: {}", addr, id);
            let listener = TcpListener::bind(addr).expect("Failed to bind to socketerino");
            loop {
                let replica = replica.clone();
                match listener.accept() {
                    Ok((mut stream, client_addr)) => {
                        handle_connection(&mut stream, replica);
                    }
                    Err(e) => {
                        eprintln!("Error when accepting incomming connection: {}", e);
                    }
                }
            }
        });
        threads.push(thread);
    }
    let _: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
}

fn handle_connection(stream: &mut TcpStream, replica: Rc<Replica>) {
    loop {
        println!("XDD");
        let mut init_buf = [0u8; 4];
        stream.read_exact(&mut init_buf).unwrap();
        let len = u32::from_le_bytes(init_buf[..].try_into().unwrap());
        println!("len: {len}");

        let mut buf = vec![0u8; len as _];
        stream.read_exact(&mut buf).unwrap();
        println!("buf: {:?}", buf);

        let message = Message::parse_message(&buf);
        println!("Received message: {:?}", message);
        replica.on_message(message);
    }
}
