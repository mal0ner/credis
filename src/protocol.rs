#[derive(Debug, PartialEq)]
pub enum Kind {
    SimpleString,
    Integer,
    BulkString,
    Array,
    SimpleError,
    Null,
    Boolean,
    Double,
    Big,
    BulkError,
    VerbatimString,
    Map,
    Set,
    Push,
}

impl Kind {
    fn from_byte(byte: u8) -> Option<Kind> {
        match byte {
            b'+' => Some(Kind::SimpleString),
            b':' => Some(Kind::Integer),
            b'$' => Some(Kind::BulkString),
            b'*' => Some(Kind::Array),
            b'-' => Some(Kind::SimpleError),
            b'_' => Some(Kind::Null),
            b'#' => Some(Kind::Boolean),
            b',' => Some(Kind::Double),
            b'(' => Some(Kind::Big),
            b'!' => Some(Kind::BulkError),
            b'=' => Some(Kind::VerbatimString),
            b'%' => Some(Kind::Map),
            b'~' => Some(Kind::Set),
            b'>' => Some(Kind::Push),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RESPError {
    InvalidData(&'static str),
    InvalidType(&'static str),
}

impl std::fmt::Display for RESPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RESPError::InvalidData(msg) => write!(f, "RESP Error: Invalid Data - {}", msg),
            RESPError::InvalidType(msg) => write!(f, "RESP Error: Invalid Type - {}", msg),
        }
    }
}

impl std::error::Error for RESPError {}

pub fn parse_bytestream(b: &[u8]) -> Result<Kind, RESPError> {
    if b.is_empty() {
        return Err(RESPError::InvalidData("incoming bytestream empty"));
    }
    let resp_kind = match Kind::from_byte(b[0]) {
        Some(resp_kind) => resp_kind,
        None => return Err(RESPError::InvalidType("unrecognized data prefix")),
    };

    match resp_kind {
        Kind::SimpleString => {
            println!("Got simple string!");
            Ok(resp_kind)
        }
        _ => {
            println!("Got something else!");
            Ok(resp_kind)
        }
    }
}
