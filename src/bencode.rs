// Current State: Not thoroughly tested, but it seems working.
// Todo: Write tests
//       Implement HashMap for dict type of bencode
//       Make the code concise: Last 40-50 lines looks disgusting.

#[derive(Debug, Eq, PartialEq)]
pub enum BType {
    BString(Vec<u8>),
    BInt(i128),
    BList(Vec<BType>),
    BDict(Vec<(BType, BType)>)
}

pub fn decode(input: &Vec<u8>) -> Result<BType, String> {
    match parse(input, 0) {
        Ok((b, _)) => return Ok(b),
        Err(x) => return Err(x)
    }
}

fn parse(input: &Vec<u8>, pos: usize) -> Result<(BType, usize), String> {
    if pos >= input.len() {
        return Err(String::from("Bencode ERROR: Unexpected end."));
    }
    match input[pos] {
        105 => return parse_int(input, pos),
        108 => return parse_list(input, pos),
        100 => return parse_dict(input, pos),
        x if (x >= 48 && x <= 57) => return parse_string(input, pos), 
        _ => return Err(String::from("Error")), 
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
                    num.push((i-48) as char);
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
            Some(&x) if (x >= 48 && x <= 57) => len.push((x-48) as char),
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
