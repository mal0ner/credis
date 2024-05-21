use std::fmt::{Display, Formatter};

use crate::protocol;
//use anyhow::{Ok, Result};

#[derive(Debug, Clone)]
pub enum Command {
    Echo(String),
    Ping,
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
        _ => Err(CommandError::InvalidCommand("Unsupported command")),
    }
}

pub fn execute_command(cmd: Command) -> Result<protocol::Resp, CommandError> {
    match cmd {
        Command::Echo(args) => Ok(protocol::Resp::BulkString(Some(args))),
        Command::Ping => Ok(protocol::Resp::SimpleString("PONG".to_string())),
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
    fn test_execute_echo_command() {
        let input = Resp::Array(vec![
            Resp::BulkString(Some("ECHO".to_string())),
            Resp::BulkString(Some("hello".to_string())),
        ]);

        let command = Command::from_resp(input).unwrap();

        let response = execute_command(command).unwrap();

        match response {
            Resp::BulkString(Some(s)) => assert_eq!(s, "hello".to_string()),
            _ => panic!("Expected hello response"),
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
