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

pub enum Role {
    Master,
    Slave,
}

pub struct Info {
    role: Role,
}

impl Info {
    pub fn new(role: Role) -> Self {
        Self { role }
    }
    pub fn role(&self) -> String {
        match self.role {
            Role::Master => "role:master".to_string(),
            Role::Slave => "role:slave".to_string(),
        }
    }
}

fn port_range(s: &str) -> Result<u16, String> {
    number_range(s, 1024, 65535)
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of times to greet
    #[arg(long, default_value_t = 6379, value_parser=port_range)]
    port: u16,

    #[arg(long, default_value = None)]
    replicaof: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<(), anyhow::Error> {
    let args = Args::parse();
    let listener = TcpListener::bind(format!("127.0.0.1:{}", args.port)).await?;
    let role = if args.replicaof.is_some() {
        Role::Slave
    } else {
        Role::Master
    };
    let info = Arc::new(Info::new(role));
    let cache = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let (stream, _) = listener.accept().await?;
        let cache = cache.clone();
        let server = info.clone();
        println!("accepted new connection");

        tokio::spawn(async move {
            let mut handler = protocol::Handler::new(stream, server);
            handler.handle_stream(cache).await;
        });
    }
}
