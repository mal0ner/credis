mod command;
mod protocol;
// Uncomment this block to pass the first stage
use clap::Parser;
use clap_num::number_range;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::net::TcpListener;

fn port_range(s: &str) -> Result<u16, String> {
    number_range(s, 1024, 65535)
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of times to greet
    #[arg(long, default_value_t = 6379, value_parser=port_range)]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<(), anyhow::Error> {
    let args = Args::parse();
    let listener = TcpListener::bind(format!("127.0.0.1:{}", args.port)).await?;

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
