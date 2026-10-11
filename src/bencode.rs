use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BType {
    BString(Vec<u8>),
    BInt(i64),
    BList(Vec<BType>),
    BDict(Vec<(BType, BType)>)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BencodeParseError {
    FileCouldNotRead {error: String},
    UnexpectedEnd,
    UnexpectedByte {byte: u8, pos: usize},
    IntLeadingZero {pos: usize},
    IntExpectedDigit {pos: usize},
    IntInvalid {pos: usize},
    IntNegativeZero {pos: usize},
    StrExpectedColon {pos: usize},
    StrTooShort {pos: usize, expected: usize},
    StrInvalidLength {pos: usize},
    UnexpectedBencodeValue {msg: String},
    BDictKeyNotFound { key: String },
    BDictKeyNotBString,
}

impl fmt::Display for BencodeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use BencodeParseError::*;
        match self {
            FileCouldNotRead {error} => write!(f, "File couldn't read: {}", error),
            UnexpectedEnd => write!(f, "unexpected end of input"),
            UnexpectedByte {byte, pos} => {
                write!(f, "unexpected byte 0x{:02x} at position {}", byte, pos)
            }
            IntLeadingZero {pos} => write!(f, "integer with leading zero at position {}", pos),
            IntExpectedDigit {pos} => write!(f, "expected a digit at position {}", pos),
            IntInvalid {pos} => write!(f, "invalid integer at position {}", pos),
            IntNegativeZero {pos} => write!(f, "negative zero at position {}", pos),
            StrExpectedColon {pos} => write!(f, "expected ':' at position {}", pos),
            StrTooShort {pos, expected} => {
                write!(f, "string at position {} needs {} more bytes than available", pos, expected)
            }
            StrInvalidLength {pos} => {
                write!(f, "length is invalid at position {}", pos)
            }
            UnexpectedBencodeValue {msg} => write!(f, "Unexpected bencode value: {}", msg),
            BDictKeyNotFound {key} => write!(f, "Key not found in bencode string: {}", key),
            BDictKeyNotBString => write!(f, "Key is not a encoded string"),
        }
    }
}

impl std::error::Error for BencodeParseError {}

fn read_torrent(path: &str) -> std::io::Result<Vec<u8>> {
    return std::fs::read(path);
}

pub fn read_and_decode(path: &str) -> Result<BType, BencodeParseError> {
    match read_torrent(path) {
        Ok(torrent_file) => decode(&torrent_file),
        Err(e) => return Err(BencodeParseError::FileCouldNotRead{error: e.to_string()}),
    }
}

pub fn decode(input: &[u8]) -> Result<BType, BencodeParseError> {
    let (parsed_btype, pos) = parse(input, 0)?;
    return Ok(parsed_btype);
}

fn parse(input: &[u8], pos: usize) -> Result<(BType, usize), BencodeParseError> {
    let &byte = input.get(pos).ok_or(BencodeParseError::UnexpectedEnd)?;
    match byte {
        b'i' => return parse_int(input, pos),
        b'l' => return parse_list(input, pos),
        b'd' => return parse_dict(input, pos),
        b'0'..=b'9' => return parse_string(input, pos), 
        _ => return Err(BencodeParseError::UnexpectedByte{byte, pos}), 
    }
}

fn parse_int(input: &[u8], mut pos: usize) -> Result<(BType, usize), BencodeParseError> {
    pos = pos + 1;
    let mut num = String::from("");
    let mut p = *input.get(pos).ok_or(BencodeParseError::UnexpectedEnd)?;
    let mut is_neg = false;
    if p == b'-' {
        is_neg = true;
        pos = pos + 1;
        p = *input.get(pos).ok_or(BencodeParseError::UnexpectedEnd)?;
        num.push('-');
    }
    if p == b'0' {
        match input.get(pos+1) {
            Some(&b'e') => {
                if is_neg == false {
                    return Ok((BType::BInt(0), pos+1));
                }
                else {
                    return Err(BencodeParseError::IntNegativeZero{pos});
                }
            }
            Some(&i) if (i >= 48 && i <= 57) => return Err(BencodeParseError::IntLeadingZero{pos}),  
            None => return Err(BencodeParseError::UnexpectedEnd),
            _ => return Err(BencodeParseError::IntExpectedDigit{pos}),
            
        }    
    } 
    else {
        loop {
            match p {
                b'0'..=b'9' => num.push(p as char),
                b'e' => {
                    let output: i64 = num.parse().map_err(|_| BencodeParseError::IntInvalid{pos})?;
                    return Ok((BType::BInt(output), pos));
                }
                _ => return Err(BencodeParseError::IntExpectedDigit{pos}), 
            }
            pos = pos+1;
            p = *input.get(pos).ok_or(BencodeParseError::UnexpectedEnd)?;
        }
    } 
}

fn parse_string(input: &[u8], mut pos: usize) -> Result<(BType, usize), BencodeParseError> {
    let mut len = String::from("");
    let mut s = Vec::new();
    let mut p = *input.get(pos).ok_or(BencodeParseError::UnexpectedEnd)?;
    loop {
        match p {
            b'0'..=b'9' => len.push(p as char),
            b':' => break,
            _ => return Err(BencodeParseError::StrExpectedColon{pos}),
        }
        pos = pos + 1;
        p = *input.get(pos).ok_or(BencodeParseError::UnexpectedEnd)?;
    }
    let len_int: usize = len.parse().map_err(|_| BencodeParseError::StrInvalidLength{pos})?;
    pos = pos + 1;
    let mut index = 0;
    while index < len_int {
        match input.get(pos) {
            Some(&x) => s.push(x),
            None => return Err(BencodeParseError::StrTooShort{pos: pos, expected: len_int})
        }
        index = index + 1;
        pos = pos + 1;
    }
    return Ok((BType::BString(s), pos-1));
}

fn parse_list(input: &[u8], mut pos: usize) -> Result<(BType, usize), BencodeParseError> {
    pos = pos + 1;
    let mut v = Vec::new();
    loop {
        match input.get(pos) {
            Some(&b'e') => return Ok((BType::BList(v), pos)),
            None => return Err(BencodeParseError::UnexpectedEnd),
            Some(_) => {
                let (btype, new_pos) = parse(input, pos)?;
                v.push(btype);
                pos = new_pos + 1; 
            } 
        }
    }
}

fn parse_dict(input: &[u8], mut pos: usize) -> Result<(BType, usize), BencodeParseError> {
    pos = pos + 1;
    let mut d = Vec::new();
    loop {
        match *input.get(pos).ok_or(BencodeParseError::UnexpectedEnd)? {
            b'e' => return Ok((BType::BDict(d), pos)),
            _ => {
                let (key, new_pos) = parse_string(input, pos)?;
                let (value, new_new_pos) = parse(input, new_pos + 1)?;
                d.push((key, value));
                pos = new_new_pos + 1;    
            }
        }
    }
}

pub fn encode(btype: &BType) -> Vec<u8> {
    let mut encoded = Vec::new();
    match btype {
        BType::BInt(i) => {
            encoded.push(b'i');
            for c in i.to_string().chars() {
                encoded.push(c as u8);
            }
            encoded.push(b'e')
        }
        BType::BString(s) => {
            for c in s.len().to_string().chars() {
                encoded.push(c as u8);
            }
            encoded.push(b':');
            for c in s {
                encoded.push(*c);
            }
        }
        BType::BList(v) => {
            encoded.push(b'l');
            for b in v {
                let encoded_in = encode(b);
                for c in encoded_in {
                    encoded.push(c);
                }
            }
            encoded.push(b'e');
        }
        BType::BDict(d) => {
            encoded.push(b'd');
            for (b_key, b_value) in d {
                let encoded_key = encode(&b_key);
                for c in encoded_key {
                    encoded.push(c);
                }
                let encoded_value = encode(&b_value); 
                for c in encoded_value {
                    encoded.push(c);
                }
            }
            encoded.push(b'e');
        }
    }
    return encoded;
}

pub fn print_decoded(btype: &BType) -> () {
    match btype {
        BType::BInt(i) => {
            print!("{}", i);
        }
        BType::BString(s) => {
            if s.iter().all(|&x| x <= 127) {
                for &x in s.iter() {
                    print!("{}", x as char);
                }
            }
            else {
                print!("String(");
                for &x in s.iter() {
                    print!("{}", x);
                }
                print!(")");
            }
        }
        BType::BList(v) => {
            print!("[");
            for elem in v {
                print_decoded(elem);
                print!(", ")
            }
            print!("]");
        }
        BType::BDict(d) => {
            print!("{{");
            for (key, value) in d {
                print_decoded(key);
                print!(": ");
                print_decoded(value);
                print!(", ");
            }
            print!("}}");
        }
    }
}

pub fn print_bytes_to_ascii(vec: &[u8]) -> () {
    for elem in vec {
        if *elem <= 127 {
            print!("{}", *elem as char);
        }
        else {
            print!("?");
        }
    }
}

pub fn bstring_to_ascii(btype: &BType) -> String {
    match btype {
        BType::BString(s) => {
            let mut to_return = String::from("");
            for c in s {
                to_return.push(*c as char);
            }
            return to_return;
        }
        _ => {
            return String::from("");
        }
    }
}

pub fn get_value_announce(btype: &BType) -> Result<(BType, BType), BencodeParseError> {
    match btype {
        BType::BDict(d) => return Ok((get_value("announce".to_string(), &d)?, get_value("announce-list".to_string(), &d)?)),
        _ => return Err(BencodeParseError::UnexpectedBencodeValue{msg: "Expected dict, found else".to_string()}),
    }
}

pub fn get_value_info(btype: &BType) -> Result<BType, BencodeParseError> {
    match btype {
        BType::BDict(d) => return Ok(get_value("info".to_string(), &d)?),
        _ => return Err(BencodeParseError::UnexpectedBencodeValue{msg: "Expected dict, found else".to_string()}),
    }
}

pub fn get_value(key: String, dict: &Vec<(BType, BType)>) -> Result<BType, BencodeParseError> {
    for (k, v) in dict {
        match k {
            BType::BString(real_key) if real_key == key.as_bytes() => return Ok(v.clone()),
            BType::BString(_) => continue,
            _ => return Err(BencodeParseError::BDictKeyNotBString),
        }
    }
    return Err(BencodeParseError::BDictKeyNotFound{key});
}
