// Current State: Not thoroughly tested, but it seems working.
// Todo: Write tests
//       Implement HashMap for dict type of bencode
//       Make the code concise: Last 40-50 lines looks disgusting.

#[derive(Debug, Eq, PartialEq)]
pub enum BType {
    BString(String),
    BInt(i128),
    BList(Vec<BType>),
    BDict(Vec<(BType, BType)>)
}

pub fn parse_start(input: &String) -> Result<BType, String> {
    match parse(input, 0) {
        Ok((b, _)) => return Ok(b),
        Err(x) => return Err(x)
    }
}

fn parse(input: &String, pos: usize) -> Result<(BType, usize), String> {
    match input.chars().nth(pos) {
        Some('i') => return parse_int(input, pos),
        Some('l') => return parse_list(input, pos),
        Some('d') => return parse_dict(input, pos),
        Some(x) if x.is_ascii_digit() => return parse_string(input, pos), 
        Some(_) => return Err(String::from("Error")),
        None => return Err(String::from("Error")) 
    }
}

fn parse_int(input: &String, mut pos: usize) -> Result<(BType, usize), String> {
    pos = pos + 1;
    let mut num = String::from("");
    let mut p = input.chars().nth(pos);
    if p == Some('-') {
        pos = pos + 1;
        p = input.chars().nth(pos);
        num.push('-');
    }
    if p == None {
        return Err(String::from("Bencode ERROR: Unexpected end."));
    }
    if p == Some('0') {
        match input.chars().nth(pos + 1) {
            Some('e') => {
                return Ok((BType::BInt(0), pos+1));
            }
            Some(i) if i.is_ascii_digit() => {
                return Err(String::from("Bencode ERROR: Int cannot start with 0."));
            }  
            Some(_) => {
                return Err(String::from("Bencode ERROR: Digit expected."));
            }
            None => {
                return Err(String::from("Bencode ERROR: Unexpected end."));
            }
        }
        
    } 
    else {
        loop {
            match p {
                Some(i) if i.is_ascii_digit() => {
                    num.push(i);
                }
                Some('e') => {
                    let output: i128 = num.parse()
                    .unwrap();
                    return Ok((BType::BInt(output), pos));
                }
                Some(_) => {
                    return Err(String::from("Bencode ERROR: Digit expected."));
                } 
                None => {
                    return Err(String::from("Bencode ERROR: Unexpected end."));
                }
            }
            pos = pos+1;
            p = input.chars().nth(pos);
        }
    } 
}

fn parse_string(input: &String, mut pos: usize) -> Result<(BType, usize), String> {
    let mut len = String::from("");
    let mut s = String::from("");
    let mut p = input.chars().nth(pos);

    loop {
        match p {
            Some(x) if x.is_ascii_digit() => len.push(x),
            Some(':') => break,
            Some(_) => return Err(String::from("Bencode ERROR: Expected \':\'")),
            None => return Err(String::from("Bencode ERROR: Unexpected end."))
        }
        pos = pos + 1;
        p = input.chars().nth(pos);
    }
    
    let len_int: usize = len.parse().unwrap();
    pos = pos + 1;
    let mut index = 0;
    while index < len_int {
        match input.chars().nth(pos) {
            Some(x) if x.is_alphanumeric() => s.push(x),
            Some(_) => return Err(String::from("Bencode ERROR: Expected an alphanumeric char.")),
            None => return Err(String::from("Bencode ERROR: Expected more characters."))
        }
        index = index + 1;
        pos = pos + 1;
    }
    return Ok((BType::BString(s), pos-1));
}

fn parse_list(input: &String, mut pos: usize) -> Result<(BType, usize), String> {
    pos = pos + 1;
    let mut v = Vec::new();
    loop {
        match input.chars().nth(pos) {
            Some('e') => {
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

fn parse_dict(input: &String, mut pos: usize) -> Result<(BType, usize), String> {
    pos = pos + 1;
    let mut d = Vec::new();
    loop {
        match input.chars().nth(pos) {
            Some('e') => {
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
