#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BType {
    BString(Vec<u8>),
    BInt(i128),
    BList(Vec<BType>),
    BDict(Vec<(BType, BType)>)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BencodeParseError {
    UnexpectedEnd,
    UnexpectedByte { byte: u8, pos: usize },
    IntLeadingZero { pos: usize },
    IntExpectedDigit { pos: usize },
    IntInvalid { pos: usize },
    StrExpectedColon { pos: usize },
    StrTooShort { pos: usize, expected: usize },
}

impl fmt::Display for BencodeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use BencodeParseError::*;
        match self {
            UnexpectedEnd => write!(f, "unexpected end of input"),
            UnexpectedByte { byte, pos } => {
                write!(f, "unexpected byte 0x{:02x} at position {}", byte, pos)
            }
            LeadingZero { pos } => write!(f, "integer with leading zero at position {}", pos),
            ExpectedDigit { pos } => write!(f, "expected a digit at position {}", pos),
            InvalidInteger { pos } => write!(f, "invalid integer at position {}", pos),
            ExpectedColon { pos } => write!(f, "expected ':' at position {}", pos),
            StringTooShort { pos, expected } => {
                write!(f, "string at position {} needs {} more bytes than available", pos, expected)
            }
        }
    }
}

pub fn decode(input: &[u8]) -> Result<BType, BencodeParseError> {
    let (parsed_btype, pos) = parse(input)?;
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

fn parse_int(input: &Vec<u8>, mut pos: usize) -> Result<(BType, usize), String> {
    pos = pos + 1;
    let mut num = String::from("");
    let mut p = input.get(pos);
    if p == Some(&45) {
        pos = pos + 1;
        p = input.get(pos);
        num.push('-');
    }
    if p == Some(&48) {
        match input.get(pos) {
            Some(&101) => {
                return Ok((BType::BInt(0), pos+1));
            }
            Some(&i) if (i >= 48 && i <= 57) => {
                return Err(String::from("Bencode ERROR: Int cannot start with 0."));
            }  
            None => {
                return Err(String::from("Bencode ERROR: Unexpected end."));
            }
            _ => {
                return Err(String::from("Bencode ERROR: Digit expected."));
            }
        }    
    } 
    else {
        loop {
            match p {
                Some(&i) if (i >= 48 && i <= 57) => {
                    num.push(i as char);
                }
                Some(&101) => {
                    let output: i128 = num.parse()
                    .unwrap();
                    return Ok((BType::BInt(output), pos));
                }
                None => {
                    return Err(String::from("Bencode ERROR: Unexpected end."));
                }
                _ => {
                    return Err(String::from("Bencode ERROR: Digit expected."));
                } 
            }
            pos = pos+1;
            p = input.get(pos);
        }
    } 
}

fn parse_string(input: &Vec<u8>, mut pos: usize) -> Result<(BType, usize), String> {
    let mut len = String::from("");
    let mut s = Vec::new();
    let mut p = input.get(pos);

    loop {
        match p {
            Some(&x) if (x >= 48 && x <= 57) => len.push(x as char),
            Some(&58) => break,
            None => return Err(String::from("Bencode ERROR: Unexpected end.")),
            _ => return Err(String::from("Bencode ERROR: Expected \':\'")),
        }
        pos = pos + 1;
        p = input.get(pos);
    }
    let len_int: usize = len.parse().unwrap();
    pos = pos + 1;
    let mut index = 0;
    while index < len_int {
        match input.get(pos) {
            Some(&x) => s.push(x),
            None => return Err(String::from("Bencode ERROR: Expected more characters."))
        }
        index = index + 1;
        pos = pos + 1;
    }
    return Ok((BType::BString(s), pos-1));
}

fn parse_list(input: &Vec<u8>, mut pos: usize) -> Result<(BType, usize), String> {
    pos = pos + 1;
    let mut v = Vec::new();
    loop {
        match input.get(pos) {
            Some(&101) => {
                return Ok((BType::BList(v), pos));
            }
            None => {
                return Err(String::from("Bencode ERROR: Unexpected end."));
            }
            Some(_) => {
                match parse(input, pos) {
                    Ok((b, i)) => {
                        v.push(b);
                        pos = i + 1;
                    }
                    Err(x) => {
                        return Err(x);
                    }
                }
            } 
        }
    }
}

fn parse_dict(input: &Vec<u8>, mut pos: usize) -> Result<(BType, usize), String> {
    pos = pos + 1;
    let mut d = Vec::new();
    loop {
        match input.get(pos) {
            Some(&101) => {
                return Ok((BType::BDict(d), pos));
            }
            None => {
                return Err(String::from("Bencode ERROR: Unexpected end."));
            }
            Some(_) => {
                match parse_string(input, pos) {
                    Ok((k, i)) => {
                        match parse(input, i+1) {
                            Ok((v, j)) => {
                                d.push((k, v));
                                pos = j + 1;
                            }
                            Err(z) => {
                                return Err(z);
                            }
                        }
                    }
                    Err(y) => {
                        return Err(y);
                    }
                }
            }
        }
    }

}

pub fn encode(btype: BType) -> Result<Vec<u8>, String> {
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
                encoded.push(c);
            }
        }
        BType::BList(v) => {
            encoded.push(b'l');
            for b in v {
                let encoded_in = encode(b);
                match encoded_in {
                    Ok(vec_in) => {
                        for c in vec_in {
                            encoded.push(c);
                        }
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            encoded.push(b'e');
        }
        BType::BDict(d) => {
            encoded.push(b'd');
            for (key, value) in d {
                let encoded_key = encode(key);
                match encoded_key {
                    Ok(vec_key) => {
                        for c in vec_key {
                            encoded.push(c);
                        }
                    }
                    Err(e) => return Err(e)
                }
                let encoded_value = encode(value); 
                match encoded_value {
                    Ok(vec_value) => {
                        for c in vec_value {
                            encoded.push(c);
                        }
                    }
                    Err(e) => return Err(e)
                }
            }
            encoded.push(b'e');
        }
    }
    return Ok(encoded);
}

pub fn print_decoded(btype: BType) -> () {
    match btype {
        BType::BInt(i) => {
            print!("{}", i);
        }
        BType::BString(s) => {
            if s.iter().all(|&x| is_printable(x)) {
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

pub fn print_vec_to_ascii(vec: &Vec<u8>) -> () {
    for elem in vec {
        if is_printable(*elem) {
            print!("{}", *elem as char);
        }
        else {
            print!("?");
        }
    }
}

pub fn bstring_to_ascii(btype: BType) -> String {
    match btype {
        BType::BString(s) => {
            let mut to_return = String::from("");
            for c in s {
                to_return.push(c as char);
            }
            return to_return;
        }
        _ => {
            return String::from("");
        }
    }
}

fn is_printable(c: u8) -> bool {
    if c <= 127 {
        return true;
    }
    return false;
}

pub fn get_value(key: String, dict: &Vec<(BType, BType)>) -> Result<BType, String> {
    let key_ext: &[u8] = key.as_bytes();
    for (k, v) in dict {
        match k {
            BType::BString(real_key) => {
                if vec_compare(&real_key, key_ext) {
                    let ret = v.clone();
                    return Ok(ret);
                }
                else {
                    continue;
                }
            }
            _ => {
                return Err(String::from("Key error: Key is not BString."));
            }
        }
    }
    return Err(String::from("Key not found."));
}

fn vec_compare(vec1: &Vec<u8>, vec2: &[u8]) -> bool {
    if vec1.len() != vec2.len() {
        return false;
    }
    let mut index = 0;
    while index < vec1.len() {
        if vec1[index] != vec2[index] {
            return false;
        }
        index = index + 1;
    }
    return true;
}
