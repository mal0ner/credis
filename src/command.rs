use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    sync::{Arc, Mutex},
};

use crate::protocol;

#[derive(Debug, Clone)]
pub enum Command {
    Echo(String),
    Ping,
    Get(String),
    Set(String, String),
}

#[derive(Debug, Clone)]
pub enum CommandError {
    InvalidCommand(&'static str),
    InvalidArguments(&'static str),
}

impl Display for CommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::InvalidCommand(msg) => {
                write!(f, "Command Error: Invalid Command - {}", msg)
            }
            CommandError::InvalidArguments(msg) => {
                write!(f, "Command Error: Invalid Arguments - {}", msg)
            }
        }
    }
}

impl std::error::Error for CommandError {}

impl Command {
    pub fn from_resp(resp: protocol::Resp) -> Result<Command, CommandError> {
        match resp {
            protocol::Resp::Array(args) => parse_command(args),
            _ => Err(CommandError::InvalidCommand("resp should be an array")),
        }
    }
}

fn parse_command(args: Vec<protocol::Resp>) -> Result<Command, CommandError> {
    let command_str = match args.first() {
        Some(protocol::Resp::BulkString(Some(string))) => string,
        _ => {
            return Err(CommandError::InvalidCommand(
                "Command must be a bulk string",
            ))
        }
    };

    match command_str.to_uppercase().as_str() {
        "ECHO" => match args.len() {
            2 => {
                if let protocol::Resp::BulkString(Some(data)) = &args[1] {
                    Ok(Command::Echo(data.to_string()))
                } else {
                    Err(CommandError::InvalidArguments(
                        "Argument must be a bulk string",
                    ))
                }
            }
            _ => Err(CommandError::InvalidArguments(
                "ECHO command expects exactly one argument",
            )),
        },
        "PING" => match args.len() {
            1 => Ok(Command::Ping),
            _ => Err(CommandError::InvalidArguments(
                "PING command expects no arguments",
            )),
        },
        "GET" => match args.get(1) {
            Some(protocol::Resp::BulkString(Some(key))) => Ok(Command::Get(key.to_string())),
            _ => Err(CommandError::InvalidArguments("Usage: GET <key>")),
        },
        "SET" => match (args.get(1), args.get(2)) {
            (
                Some(protocol::Resp::BulkString(Some(key))),
                Some(protocol::Resp::BulkString(Some(value))),
            ) => Ok(Command::Set(key.to_string(), value.to_string())),
            _ => Err(CommandError::InvalidArguments("Usage: SET <key> <value>")),
        },
        _ => Err(CommandError::InvalidCommand("Unsupported command")),
    }
}

// executes a command and returns the unencoded response.
pub fn execute_command(
    cmd: Command,
    cache: &Arc<Mutex<HashMap<String, String>>>,
) -> Result<protocol::Resp, CommandError> {
    match cmd {
        Command::Echo(arg) => Ok(protocol::Resp::BulkString(Some(arg))),
        Command::Ping => Ok(protocol::Resp::SimpleString("PONG".to_string())),
        Command::Get(key) => {
            let cache = cache.lock().unwrap();
            if let Some(value) = cache.get(&key) {
                Ok(protocol::Resp::BulkString(Some(value.clone())))
            } else {
                Ok(protocol::Resp::Null)
            }
        }
        Command::Set(key, value) => {
            let mut cache = cache.lock().unwrap();
            cache.insert(key, value);
            Ok(protocol::Resp::SimpleString("OK".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Resp;

    #[test]
    fn test_parse_echo_command() {
        let input = Resp::Array(vec![
            Resp::BulkString(Some("ECHO".to_string())),
            Resp::BulkString(Some("hello".to_string())),
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
            Resp::BulkString(Some("ECHO".to_string())),
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
