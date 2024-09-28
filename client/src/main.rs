use ::client::{Op, ADDRESSES};
use client::Client;
use request::Request;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{net::TcpStream, thread, time::Duration};

pub(crate) mod client;
pub(crate) mod request;

const CLIENT_ID: usize = 69;

fn main() {
    let mut client = Client::new(CLIENT_ID);

    // TODO: Connect to the whole cluster.
    // Assume that the first replica in the list is primary.
    let primary_addr = ADDRESSES[0];
    let mut stream = TcpStream::connect(primary_addr).unwrap();
    let mut request_num = 0;
    /*
    loop {
        let client = &mut client;
        let stream = &mut stream;
        let value = generate_random_number();
        let request = Request::new(client.id, request_num, Op::Add(value));
        client.request_number = request_num;
        request_num += 1;

        let bytes = request.to_bytes();
        let _ = stream.write(&bytes).unwrap();
        // TODO: Read the response
        thread::sleep(Duration::from_millis(1000));
    }
    */
    loop {}
}

fn generate_random_number() -> u64 {
    // Get the current time in nanoseconds since UNIX_EPOCH
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();

    // Convert the u128 (nanoseconds) to u64 by shifting and XORing
    let random_number = (now as u64) ^ ((now >> 32) as u64);

    // Return the "random" number
    random_number % 2048
}
