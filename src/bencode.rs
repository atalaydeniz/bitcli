use std::collections::HashMap;

#[derive(Debug)]
pub enum BType {
    BString(String),
    BInt(i128),
    BList(Vec<BType>),
    BDict(HashMap<String, BType>)
}

pub fn parse_start(input: &String) -> Result<BType, String> {
    match parse(input, 0) {
        Ok((b, i)) => return Ok(b),
        Err(x) => return Err(x)
    }
}

fn parse(input: &String, mut pos: usize) -> Result<(BType, usize), String> {
    match input.chars().nth(pos) {
        Some('i') => return parse_int(input, pos+1),
        Some(x) => return Err(String::from("")), 
        None => return Err(String::from("Error")) 
    }
}

fn parse_int(input: &String, mut pos: usize) -> Result<(BType, usize), String> {
    let mut num = String::from("");
    let mut p = input.chars().nth(pos);
    if p == Some('-') {
        pos = pos + 1;
        p = input.chars().nth(pos);
        num.push('-');
    }
    if p == Some('0') {
        match input.chars().nth(pos + 1) {
            Some('e') => {
                return Ok((BType::BInt(0), pos+1));
            }
            Some(i) if i.is_ascii_digit() => {
                return Err(String::from("Bencode ERROR: Int cannot start with 0."));
            }  
            Some(i) => {
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
                Some(x) => {
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
