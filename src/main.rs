mod command;
mod protocol;
// Uncomment this block to pass the first stage

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<(), anyhow::Error> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    let cache = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let (stream, _) = listener.accept().await?;
        let cache = cache.clone();
        println!("accepted new connection");

        tokio::spawn(async move {
            let mut handler = protocol::Handler::new(stream);
            handler.handle_stream(cache).await;
        });
    }
}
