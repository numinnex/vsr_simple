use client::ADDRESSES;
use message::Message;
use monoio::{
    io::AsyncReadRentExt,
    net::{TcpListener, TcpStream},
};
use replica::Replica;
use replica_config::ReplicaConfig;
use std::{rc::Rc, time::Duration};

const TWO_SECONDS: u64 = 2;

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
        config.append_new(id, addr);
    }

    for (id, addr) in ADDRESSES.into_iter().enumerate() {
        let builder = std::thread::Builder::new().name(format!("replica-{id}"));
        let config = config.clone();
        let thread = builder
            .spawn(move || {
                let mut rt = monoio::RuntimeBuilder::<monoio::FusionDriver>::new()
                    .with_entries(256)
                    .enable_timer()
                    .build()
                    .unwrap();
                rt.block_on(async {
                    let replica = Rc::new(Replica::new(id, config));
                    println!("Created node with addr: {}, id: {}", addr, id);
                    let listener = TcpListener::bind(addr).expect("Failed to bind to socketerino");
                    loop {
                        let replica = replica.clone();
                        match listener.accept().await {
                            Ok((mut stream, _)) => {
                                monoio::spawn(async move {
                                    handle_connection(&mut stream, replica).await
                                });
                            }
                            Err(e) => {
                                eprintln!("Error when accepting incomming connection: {}", e);
                            }
                        }
                    }
                });
            })
            .unwrap();
        threads.push(thread);
    }
    let _: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
}

async fn handle_connection(stream: &mut TcpStream, replica: Rc<Replica>) {
    loop {
        let init_buf = vec![0u8; 4];
        let read_fut = stream.read_exact(init_buf);
        let result = monoio::time::timeout(Duration::from_secs(TWO_SECONDS), read_fut).await;
        match result {
            Ok(val) => {
                let (res, init_buf) = val;
                if let Err(e) = res {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        break;
                    }
                }
                let len = u32::from_le_bytes(init_buf[..].try_into().unwrap());
                let buf = vec![0u8; len as _];
                let (res, buf) = stream.read_exact(buf).await;
                res.unwrap();

                let message = Message::parse_message(&buf);
                println!("Received message: {:?}", message);
                replica.on_message(message).await;
            }
            Err(_) => {
                let thread = std::thread::current();
                println!("Ticking timer on thread: {:?}", thread);
                replica.on_timer().await;
            }
        }
    }
}
