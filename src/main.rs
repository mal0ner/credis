mod protocol;
// Uncomment this block to pass the first stage
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                println!("accepted new connection");
                // s.write_all(b"+PONG\r\n").unwrap();
                thread::spawn(move || {
                    handle_connection(s).unwrap_or_else(|err| eprintln!("{:?}", err));
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        };
    }
}

fn handle_connection(mut stream: std::net::TcpStream) -> Result<(), anyhow::Error> {
    let mut recvbuf = [0; 1024];
    loop {
        let _num_bytes = stream.read(&mut recvbuf)?;
        if _num_bytes == 0 {
            return Ok(());
        }
        stream.write_all(b"+PONG\r\n")?;
        stream.flush()?;
    }
}
