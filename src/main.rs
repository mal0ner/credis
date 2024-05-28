mod command;
mod protocol;
// Uncomment this block to pass the first stage
use clap::Parser;
use clap_num::number_range;
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
    sync::{Arc, Mutex},
};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

pub enum Role {
    Master,
    Slave,
}

pub struct Info {
    role: Role,
    master_replid: String,
    master_repl_offset: usize,
}

impl Info {
    pub fn new(role: Role) -> Self {
        Self {
            role,
            master_replid: "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_string(),
            master_repl_offset: 0,
        }
    }
    pub fn role(&self) -> String {
        match self.role {
            Role::Master => "role:master".to_string(),
            Role::Slave => "role:slave".to_string(),
        }
    }
    pub fn id(&self) -> String {
        self.master_replid.to_string()
    }
    pub fn replication(&self) -> String {
        format!(
            "# Replication\nrole:{}\nmaster_replid:{}\nmaster_repl_offset:{}",
            self.role(),
            self.master_replid,
            self.master_repl_offset
        )
    }
}

pub struct HostSpec {
    host: String,
    port: u16,
}

impl FromStr for HostSpec {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let components: Vec<&str> = s.split_whitespace().collect();

        if components.len() != 2 {
            return Err("Invalid Master Host and Port specification.".to_string());
        }

        let host = components[0];
        let port = components[1].parse::<u16>().map_err(|_| "Invalid Port")?;

        if !(1024..=65535).contains(&port) {
            return Err("Port must be between 1024 and 65535".to_string());
        }

        let ip_addr: IpAddr = match host.parse() {
            Ok(addr) => addr,
            Err(_) => {
                if host != "localhost" {
                    return Err("Invalid host".to_string());
                }
                IpAddr::V4(Ipv4Addr::LOCALHOST)
            }
        };

        Ok(HostSpec {
            host: ip_addr.to_string(),
            port,
        })
    }
}

impl ToString for HostSpec {
    fn to_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

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
