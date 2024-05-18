mod protocol;
// Uncomment this block to pass the first stage

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> anyhow::Result<(), anyhow::Error> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let (stream, _) = listener.accept().await?;
        println!("accepted new connection");

        tokio::spawn(async move {
            if let Err(err) = handle_connection(stream).await {
                eprintln!("{:?}", err);
            }
        });
    }
}

async fn handle_connection(mut stream: TcpStream) -> anyhow::Result<(), anyhow::Error> {
    let mut recvbuf = [0; 1024];

    loop {
        let num_bytes = stream.read(&mut recvbuf).await?;

        if num_bytes == 0 {
            return Ok(());
        }

        println!(
            "handling conn: {}",
            std::str::from_utf8(&recvbuf[..num_bytes]).unwrap()
        );

        let _r = protocol::readnext_resp(&recvbuf)?;

        //match r {
        //    protocol::Resp::Integer(x) => println!("{}", x),
        //    protocol::Resp::BulkString(s) => println!("{:?}", s),
        //    protocol::Resp::Array(v) => println!("Got array {:?}", v),
        //    _ => println!("Unsupported type in main file"),
        //}

        stream.write_all(b"+PONG\r\n").await?;
        stream.flush().await?;
    }
}
