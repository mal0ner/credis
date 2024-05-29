mod command;
mod protocol;
mod server;
// Uncomment this block to pass the first stage
use clap::Parser;
use clap_num::number_range;
use server::{Handler, HostSpec, Info, Query, Role};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

async fn repl_handshake(address: HostSpec) -> anyhow::Result<()> {
    let mut stream = TcpStream::connect(address.to_string()).await?;
    stream.write_all(b"*1\r\n$4\r\nping\r\n").await?;
    Ok(())
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

    let role = if let Some(address) = args.replicaof {
        let master = address
            .parse::<HostSpec>()
            .expect("failed to parse master address");
        repl_handshake(master)
            .await
            .expect("failed to perform handshake");
        Role::Slave
    } else {
        Role::Master
    };

    let listener = TcpListener::bind(format!("127.0.0.1:{}", args.port)).await?;
    let info = Arc::new(Info::new(role));
    let cache: Arc<Mutex<HashMap<String, Query>>> = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let (stream, _) = listener.accept().await?;
        let cache = cache.clone();
        let server = info.clone();
        println!("accepted new connection");

        tokio::spawn(async move {
            let mut handler = Handler::new(stream, server);
            handler.handle_stream(cache).await;
        });
    }
}
