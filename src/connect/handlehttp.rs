use rand::RngExt;
use crate::bencode::{BType, decode, get_value};

pub fn connect_tracker_http(url: &String, info_hash: &String, peer_id: &String, port: usize, left: &String, compact: usize)
 -> Result<Vec<u8>, ureq::Error> {
    let sep = if url.contains('?') { '&' } else { '?' };
    let hex_info_hash: String = info_hash.as_bytes()
        .chunks(2)
        .map(|pair| {
            let byte = u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap();
            match byte {
                b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'-' | b'.' | b'_' | b'~' => {
                    (byte as char).to_string()
                }
                _ => format!("%{:02X}", byte),
            }
        })
        .collect();

    let full_url = format!(
        "{}{}info_hash={}&peer_id={}&port={}&left={}&compact=1",
        url,
        sep,
        hex_info_hash,
        peer_id,
        port.to_string(),
        left
    );
    println!("GET {}", full_url);

    match ureq::get(&full_url).call() {
        Ok(mut response) => {
            let res: Vec<u8> = response.body_mut().read_to_vec()?;
            return Ok(res);
        }
        Err(ureq::Error::StatusCode(code)) => {
            println!("Tracker returned HTTP status {}", code);
            return Err(ureq::Error::StatusCode(code));
        }
        Err(e) => {
            println!("Request failed: {}", e);
            return Err(e);
        }
    }
}

pub fn parse_response(encoded_res: &Vec<u8>) -> Result<(i128, Vec<u8>), String>{
    let decoded_btype: Vec<(BType, BType)>;
    match decode(encoded_res) {
        Err(e) => return Err(e),
        Ok(BType::BDict(d)) => decoded_btype = d,
        _ => return Err("Tracker response is not a dictionary".to_string()) 
    }
    let interval: i128;
    let peers: Vec<u8>;
    match get_value(String::from("interval"), &decoded_btype) {
        Ok(BType::BInt(i)) => interval = i,
        Err(e) => return Err(e),
        _ => return Err(String::from("Impossible"))
    }
    match get_value(String::from("peers"), &decoded_btype) {
        Ok(BType::BString(s)) => peers = s,
        Err(e) => return Err(e),
        _ => return Err(String::from("peers string couldn't parsed."))
    }
    return Ok((interval, peers));

}