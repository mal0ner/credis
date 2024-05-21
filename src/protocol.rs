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

    //fn byte(k: Kind) -> u8 {
    //    match k {
    //        Kind::SimpleString => b'+',
    //        Kind::Integer => b':',
    //        Kind::BulkString => b'$',
    //        Kind::Array => b'*',
    //        Kind::SimpleError => b'-',
    //        Kind::Null => b'_',
    //        Kind::Boolean => b'#',
    //        Kind::Double => b',',
    //        Kind::Big => b'(',
    //        Kind::BulkError => b'!',
    //        Kind::VerbatimString => b'=',
    //        Kind::Map => b'%',
    //        Kind::Set => b'~',
    //        Kind::Push => b'>',
    //    }
    //}

    fn byte_char(k: Kind) -> char {
        match k {
            Kind::SimpleString => '+',
            Kind::Integer => ':',
            Kind::BulkString => '$',
            Kind::Array => '*',
            Kind::SimpleError => '-',
            Kind::Null => '_',
            Kind::Boolean => '#',
            Kind::Double => ',',
            Kind::Big => '(',
            Kind::BulkError => '!',
            Kind::VerbatimString => '=',
            Kind::Map => '%',
            Kind::Set => '~',
            Kind::Push => '>',
        }
    }
}

#[derive(Debug, Clone)]
pub enum RespError {
    InvalidData(&'static str),
    InvalidType(&'static str),
}

impl std::fmt::Display for RespError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RespError::InvalidData(msg) => write!(f, "RESP Error: Invalid Data - {}", msg),
            RespError::InvalidType(msg) => write!(f, "RESP Error: Invalid Type - {}", msg),
        }
    }
}

impl std::error::Error for RespError {}

// pub struct Resp<'a> {
//     resp_type: Kind,
//     data: &'a [u8],
//     raw: &'a [u8],
//     size: usize,
// }

#[derive(Debug, PartialEq)]
pub enum Resp {
    SimpleString(String),
    Integer(i64),
    BulkString(Option<String>),
    Array(Vec<Resp>),
    //SimpleError(String),
    Null,
    //Boolean(bool),
    // Double(f64),
    // Big(String),
    // BulkError,
    // VerbatimString(String),
    // Map,
    // Set,
    // Push,
}

pub trait RespEncoding {
    fn encoded_string(&self) -> String;
    fn encode(&self) -> Vec<u8>;
}

impl RespEncoding for Resp {
    fn encoded_string(&self) -> String {
        match self {
            Resp::SimpleString(s) => {
                let mut result = Kind::byte_char(Kind::SimpleString).to_string();
                result.push_str(s);
                result.push_str("\r\n");
                result
            }
            //Resp::SimpleError(e) => {
            //    let mut result = Kind::byte_char(Kind::SimpleError).to_string();
            //    result.push_str(e);
            //    result.push_str("\r\n");
            //    result
            //}
            Resp::Integer(i) => {
                let mut result = Kind::byte_char(Kind::Integer).to_string();
                result.push_str(&i.to_string());
                result.push_str("\r\n");
                result
            }
            Resp::BulkString(data) => {
                let mut result = Kind::byte_char(Kind::BulkString).to_string();
                println!("bulk string encoding start: {}", result);
                if let Some(value) = data {
                    result.push_str(&value.len().to_string());
                    result.push_str("\r\n");
                    result.push_str(value);
                    result.push_str("\r\n");
                } else {
                    result.push_str("-1\r\n");
                }
                result
            }
            Resp::Array(list) => {
                let mut result = Kind::byte_char(Kind::Array).to_string();
                result.push_str(&list.len().to_string());
                result.push_str("\r\n");
                for item in list {
                    result.push_str(&item.encoded_string());
                }
                result
            }
            Resp::Null => "$-1\r\n".to_string(),
        }
    }
    fn encode(&self) -> Vec<u8> {
        self.encoded_string().into_bytes()
    }
}

// Parses data based on Resp kind as indicated by the first byte.
// Creates and returns corresponding Resp variant.
pub fn readnext_resp(b: &[u8]) -> Result<(Resp, usize), RespError> {
    if b.is_empty() {
        return Err(RespError::InvalidData("incoming bytestream empty"));
    }

    println!(
        "reading resp, check first char: {:?}",
        std::str::from_utf8(b).unwrap().chars().next()
    );
    let resp_kind =
        Kind::from_byte(b[0]).ok_or(RespError::InvalidType("unrecognized datatype prefix byte"))?;

    match resp_kind {
        Kind::SimpleString => parse_string(&b[1..]),
        Kind::Integer => parse_integer(&b[1..]),
        Kind::BulkString => parse_bulk(&b[1..]),
        Kind::Array => parse_array(&b[1..]),
        _ => Err(RespError::InvalidType("unsupported RESP type")),
    }
}

fn parse_string(b: &[u8]) -> Result<(Resp, usize), RespError> {
    let end =
        find_clrf_index(b).ok_or(RespError::InvalidData("improperly terminated data payload"))?;
    let string = String::from_utf8(b[..end - 2].to_vec())
        .map_err(|_| RespError::InvalidData("Invalid UTF-8 in Simple String"))?;
    Ok((Resp::SimpleString(string), end))
}

fn parse_integer(b: &[u8]) -> Result<(Resp, usize), RespError> {
    let end = find_clrf_index(b).ok_or(RespError::InvalidData(
        "improperly terminated data payload for integer",
    ))?;
    let integer = std::str::from_utf8(&b[..end - 2])
        .map_err(|_| RespError::InvalidData("Invalid UTF-8 in Integer"))?
        .parse::<i64>()
        .map_err(|_| RespError::InvalidData("Invalid integer value"))?;
    Ok((Resp::Integer(integer), end))
}

fn parse_bulk(b: &[u8]) -> Result<(Resp, usize), RespError> {
    let len_end = find_clrf_index(b).ok_or(RespError::InvalidData(
        "CLRF not found in bulk string length specification",
    ))?;
    let len = std::str::from_utf8(&b[..len_end - 2])
        .map_err(|_| RespError::InvalidData("Invalid UTF-8 in bulk string length specification"))?
        .parse::<isize>()
        .map_err(|_| RespError::InvalidData("Invalid bulk string length"))?;

    if len == -1 {
        return Ok((Resp::Null, 0));
    }

    let data_start = len_end;
    let data_end = data_start + len as usize;

    if data_end + 2 > b.len() || &b[data_end..data_end + 2] != b"\r\n" {
        return Err(RespError::InvalidData(
            "Improperly terminated data payload for bulk string",
        ));
    }

    let data = std::str::from_utf8(&b[data_start..data_end])
        .map_err(|_| RespError::InvalidData("Invalid UTF-8 in bulk string"))?;
    //println!(
    //    "parsed bulk string: {}",
    //    std::str::from_utf8(&data).unwrap()
    //);
    // HACK: Add 2 to the buffer size as we remove two datatype specification
    // bytes with calls to readnext_resp (one for the $[length]), and one for the
    // actual string... or something like that...
    // This is terrible and a magic number?
    // I don't know...
    Ok((Resp::BulkString(Some(data.to_string())), data_end + 2))
}

fn parse_array(b: &[u8]) -> Result<(Resp, usize), RespError> {
    println!("parsing array: {}", std::str::from_utf8(b).unwrap());
    let len_end = find_clrf_index(b).ok_or(RespError::InvalidData(
        "CLRF not found in array length specification",
    ))?;
    let len = std::str::from_utf8(&b[..len_end - 2])
        .map_err(|_| RespError::InvalidData("Invalid UTF-8 in array length specification"))?
        .parse::<isize>()
        .map_err(|_| RespError::InvalidData("Invalid array length"))?;

    if len < -1 {
        return Err(RespError::InvalidData("array length cannot be < -1"));
    }

    if len == -1 {
        return Ok((Resp::Null, 0));
    }

    let mut items = Vec::with_capacity(len as usize);
    let mut rest = &b[len_end..];
    println!(
        "body of arr (length spec removed):\n{}",
        std::str::from_utf8(rest).unwrap()
    );
    for _ in 0..len {
        let (item, remaining) = parse_next_arr_value(rest)?;
        items.push(item);
        rest = remaining;
    }

    Ok((Resp::Array(items), b.len()))
}

fn parse_next_arr_value(b: &[u8]) -> Result<(Resp, &[u8]), RespError> {
    let (val, size) = readnext_resp(b)?;
    //let end = find_clrf_index(b).unwrap_or(0);
    println!(
        "remaining for next parse:\n{}",
        // HACK: Add 1 to the buffer size to account for the one taken off during
        // the call to the readnext_resp function. This is terrible and a magic number?
        // I don't know...
        std::str::from_utf8(&b[size + 1..]).unwrap()
    );
    Ok((val, &b[size + 1..]))
}

fn find_clrf_index(b: &[u8]) -> Option<usize> {
    b.windows(2)
        .position(|window| window == b"\r\n")
        .map(|pos| pos + 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_array() {
        let input = b"*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n";
        let (parsed, _) = readnext_resp(input).unwrap();
        assert_eq!(
            parsed,
            Resp::Array(vec![
                Resp::BulkString(Some("ECHO".to_string())),
                Resp::BulkString(Some("hey".to_string()))
            ])
        );

        //let response = handle_command(parsed).unwrap();
        //assert_eq!(response, RespValue::BulkString(Some(b"hey".to_vec())));
    }

    #[test]
    fn test_parse_int() {
        let input = b"*2\r\n:51\r\n:33\r\n";
        let (parsed, _) = readnext_resp(input).unwrap();

        assert_eq!(
            parsed,
            Resp::Array(vec![Resp::Integer(51), Resp::Integer(33),])
        );
    }
}
