use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
    sync::Arc,
    time::SystemTime,
};

use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

use crate::{
    command,
    protocol::{readnext_resp, Resp, RespEncoding},
};

pub enum Role {
    Master,
    Slave,
}

pub struct Info {
    pub role: Role,
    pub master_replid: String,
    pub master_repl_offset: u64,
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

pub async fn _repl_handshake(address: HostSpec) -> anyhow::Result<()> {
    let mut stream = TcpStream::connect(address.to_string()).await?;
    stream.write_all(b"*1\r\n$4\r\nping\r\n").await?;
    Ok(())
}

#[derive(Clone)]
pub struct Query {
    pub value: String,
    pub created_at: SystemTime,
    pub expiry: Option<SystemTime>,
}

pub struct Handler {
    stream: TcpStream,
    info: Arc<Mutex<Info>>,
    buf: BytesMut,
}

impl Handler {
    pub fn new(stream: TcpStream, server: Arc<Mutex<Info>>) -> Self {
        Self {
            stream,
            info: server,
            buf: BytesMut::with_capacity(1024),
        }
    }
    pub async fn handle_stream(&mut self, cache: Arc<Mutex<HashMap<String, Query>>>) {
        loop {
            let req = self.read_resp().await.unwrap();

            let response = if let Some(req) = req {
                let cmd = command::Command::from_resp(req).unwrap();
                command::execute_command(cmd, cache.clone(), self.info.clone())
                    .await
                    .unwrap()
            } else {
                break;
            };
            println!("sending response: {:?}", response);
            self.write_resp(response).await.unwrap();
        }
    }
    pub async fn read_resp(&mut self) -> Result<Option<Resp>, anyhow::Error> {
        let bytes_read = self.stream.read_buf(&mut self.buf).await?;
        if bytes_read == 0 {
            return Ok(None);
        }
        let (resp, _) = readnext_resp(&self.buf.split())?;
        Ok(Some(resp))
    }
    pub async fn write_resp(&mut self, resp: Resp) -> anyhow::Result<()> {
        self.stream.write_all(&resp.encode()).await?;
        Ok(())
    }
}
