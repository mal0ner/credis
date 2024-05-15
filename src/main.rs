mod protocol;
// Uncomment this block to pass the first stage
use std::{
    io::{Read, Write},
    net::TcpListener,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                println!("accepted new connection");
                // s.write_all(b"+PONG\r\n").unwrap();
                let mut recvbuf = [0; 1024];
                while s.read(&mut recvbuf).is_ok() {
                    let _kind = protocol::parse_bytestream(&recvbuf).unwrap();
                    s.write_all(b"+PONG\r\n").unwrap();
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
