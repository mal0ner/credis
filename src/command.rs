use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};

use tokio::sync::Mutex;

use crate::{protocol::Resp, server::Query};

#[derive(Debug, Clone)]
pub enum Command {
    Echo(String),
    Ping,
    Get(String),
    Set(String, String, Option<u64>), // <KEY> <VALUE> <TIMEOUT>
    Info(Option<String>),
    Replconf(ReplconfCommand),
}

#[derive(Debug, Clone)]
pub enum ReplconfCommand {
    Port(String),
    Capa(String),
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum CommandError {
    #[error("Command Error: Invalid Command - {}", .0)]
    InvalidCommand(&'static str),
    #[error("Command Error: Invalid Arguments - {}", .0)]
    InvalidArguments(&'static str),
}

impl Command {
    pub fn from_resp(resp: Resp) -> Result<Command, CommandError> {
        match resp {
            Resp::Array(args) => parse_command(args),
            _ => Err(CommandError::InvalidCommand("resp should be an array")),
        }
    }
}

fn parse_command(args: Vec<Resp>) -> Result<Command, CommandError> {
    use CommandError::*;
    let command_str = match args.first() {
        Some(Resp::Bulk(Some(string))) => string,
        _ => return Err(InvalidCommand("Command must be a bulk string")),
    };

    match command_str.to_uppercase().as_str() {
        "ECHO" => parse_echo(&args),
        "PING" => parse_ping(&args),
        "GET" => parse_get(&args),
        "SET" => parse_set(&args),
        "INFO" => parse_info(&args),
        "REPLCONF" => parse_replconf(&args),
        _ => Err(InvalidCommand("Unsupported command")),
    }
}

fn parse_echo(args: &[Resp]) -> Result<Command, CommandError> {
    use CommandError::*;
    match args.len() {
        2 => {
            if let Resp::Bulk(Some(data)) = &args[1] {
                Ok(Command::Echo(data.to_string()))
            } else {
                Err(InvalidArguments("Argument must be a bulk string"))
            }
        }
        _ => Err(InvalidArguments("command expects exactly one argument")),
    }
}

fn parse_ping(args: &[Resp]) -> Result<Command, CommandError> {
    use CommandError::*;
    match args.len() {
        1 => Ok(Command::Ping),
        _ => Err(InvalidArguments("PING command expects no arguments")),
    }
}

fn parse_get(args: &[Resp]) -> Result<Command, CommandError> {
    use CommandError::*;
    match args.get(1) {
        Some(Resp::Bulk(Some(key))) => Ok(Command::Get(key.to_string())),
        _ => Err(InvalidArguments("Usage: GET <key>")),
    }
}

fn parse_set(args: &[Resp]) -> Result<Command, CommandError> {
    // HACK: After careful reflection, this is awful...
    use CommandError::*;
    match args {
        [_, Resp::Bulk(Some(key)), Resp::Bulk(Some(val))] => {
            Ok(Command::Set(key.to_string(), val.to_string(), None))
        }
        [_, Resp::Bulk(Some(key)), Resp::Bulk(Some(val)), Resp::Bulk(Some(px)), Resp::Bulk(Some(millis))] => {
            if px.to_uppercase() == "PX" {
                match millis.parse::<u64>() {
                    Ok(ms) => Ok(Command::Set(key.to_string(), val.to_string(), Some(ms))),
                    Err(_) => Err(InvalidArguments("Invalid millisecond value")),
                }
            } else {
                Err(InvalidArguments("Unrecognized argument"))
            }
        }
        _ => Err(InvalidArguments(
            "Usage: SET <key> <value> <px> <milliseconds>",
        )),
    }
}

fn parse_info(args: &[Resp]) -> Result<Command, CommandError> {
    use CommandError::*;
    match args {
        [_, Resp::Bulk(Some(category))] => {
            if category.to_uppercase() == "REPLICATION" {
                Ok(Command::Info(Some(category.to_string())))
            } else {
                Err(InvalidArguments("Unrecognized argument"))
            }
        }
        _ => Err(InvalidArguments("Usage: INFO <category>")),
    }
}

fn parse_replconf(args: &[Resp]) -> Result<Command, CommandError> {
    use CommandError::*;
    match args {
        [_, Resp::Bulk(Some(replconfcmd)), Resp::Bulk(Some(arg))] => {
            match replconfcmd.to_lowercase().as_str() {
                "listening-port" => Ok(Command::Replconf(ReplconfCommand::Port(arg.to_string()))),
                "capa" => Ok(Command::Replconf(ReplconfCommand::Capa(arg.to_string()))),
                _ => Err(InvalidArguments("Unrecognized argument")),
            }
        }
        _ => Err(InvalidArguments(
            "Usage: REPLCONF listening-port | capa <ARGS>",
        )),
    }
}

// executes a command and returns the unencoded response.
pub async fn execute_command(
    cmd: Command,
    cache: Arc<Mutex<HashMap<String, Query>>>,
    info: Arc<crate::Info>,
) -> Result<Resp, CommandError> {
    match cmd {
        Command::Echo(arg) => Ok(Resp::Bulk(Some(arg))),
        Command::Ping => Ok(Resp::SimpleString("PONG".to_string())),
        Command::Get(key) => {
            let mut cache = cache.lock().await;
            let now = SystemTime::now();
            if let Some(value) = cache.get(&key) {
                let value = value.clone();
                if let Some(timeout) = value.expiry {
                    if timeout < now {
                        cache.remove(&key);
                        Ok(Resp::Null)
                    } else {
                        Ok(Resp::Bulk(Some(value.clone().value)))
                    }
                } else {
                    Ok(Resp::Bulk(Some(value.clone().value)))
                }
            } else {
                Ok(Resp::Null)
            }
        }
        Command::Set(key, value, timeout) => {
            let mut cache = cache.lock().await;
            let mut expiry = None::<SystemTime>;
            let now = SystemTime::now();

            if let Some(timeout_val) = timeout {
                expiry = Some(now + Duration::from_millis(timeout_val));
            }
            cache.insert(
                key,
                Query {
                    value,
                    created_at: now,
                    expiry,
                },
            );
            Ok(Resp::SimpleString("OK".to_string()))
        }
        Command::Info(category) => {
            if category.is_some() {
                Ok(Resp::Bulk(Some(info.replication())))
            } else {
                Ok(Resp::Null)
            }
        }
        Command::Replconf(c) => match c {
            ReplconfCommand::Port(_) => Ok(Resp::SimpleString("OK".to_string())),
            ReplconfCommand::Capa(_) => Ok(Resp::SimpleString("OK".to_string())),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Resp;

    #[test]
    fn test_parse_echo_command() {
        let input = Resp::Array(vec![
            Resp::Bulk(Some("ECHO".to_string())),
            Resp::Bulk(Some("hello".to_string())),
        ]);

        let command = Command::from_resp(input).unwrap();
        match command {
            Command::Echo(args) => assert_eq!(args, "hello".to_string()),
            _ => panic!("Expected Echo command"),
        }
    }

    #[test]
    fn test_invalid_command_type() {
        let input = Resp::SimpleString("ECHO".to_string());

        let result = Command::from_resp(input);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Command Error: Invalid Packet - RESP should be an array"
        );
    }

    #[test]
    fn test_invalid_command_argument_type() {
        let input = Resp::Array(vec![
            Resp::Bulk(Some("ECHO".to_string())),
            Resp::Integer(42),
        ]);

        let result = Command::from_resp(input);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Command Error: Invalid Arguments - All arguments must be bulk strings"
        );
    }
}
