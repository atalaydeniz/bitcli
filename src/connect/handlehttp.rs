use rand::RngExt;
use std::time::Duration;
use crate::bencode::{BType, decode, get_value, BencodeParseError};
use crate::connect::{ConnectionError};
use crate::connect::connectpeer;
use crate::connect::TrackerResponse;

pub fn connect_tracker_http(url: &String, info_hash: &String, peer_id: &String, port: usize, downloaded: u64,
    left: u64, uploaded: u64, compact: usize)
 -> Result<TrackerResponse, ConnectionError> {
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
        "{}{}info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact=1&event=started",
        url,
        sep,
        hex_info_hash,
        peer_id,
        port,
        uploaded,
        downloaded,
        left
    );
    println!("GET {}", full_url);

    let mut response = ureq::get(&full_url).call().map_err(|e| ConnectionError::HttpConnectionError(e.to_string()))?; 
    let res: Vec<u8> = response.body_mut()
                                .read_to_vec()
                                .map_err(|e| ConnectionError::HttpConnectionError(e.to_string()))?;
    let (interval, peers) = parse_response(&res).map_err(|e| ConnectionError::GenericError(e.to_string()))?;
    let peers_string = connectpeer::get_peer_addresses(&peers);
    return Ok(TrackerResponse{interval: Duration::new(interval.into(), 0), peers: peers_string});
}

pub fn parse_response(encoded_res: &Vec<u8>) -> Result<(u64, Vec<u8>), BencodeParseError>{
    let decoded_btype: Vec<(BType, BType)>;
    match decode(encoded_res) {
        Err(e) => return Err(e),
        Ok(BType::BDict(d)) => decoded_btype = d,
        Ok(BType::BString(_)) => return Err(BencodeParseError::UnexpectedBencodeValue{msg: "Expected dict, found string".to_string()}),
        Ok(BType::BInt(_)) => return Err(BencodeParseError::UnexpectedBencodeValue{msg: "Expected dict, found int".to_string()}),
        Ok(BType::BList(_)) => return Err(BencodeParseError::UnexpectedBencodeValue{msg: "Expected dict, found list".to_string()}),
    }
    let interval: i64;
    let peers: Vec<u8>;
    match get_value(String::from("interval"), &decoded_btype) {
        Ok(BType::BInt(i)) => interval = i,
        Err(e) => return Err(e),
        Ok(BType::BString(_)) => return Err(BencodeParseError::UnexpectedBencodeValue{msg:"Expected int, found string".to_string()}),
        Ok(BType::BDict(_)) => return Err(BencodeParseError::UnexpectedBencodeValue{msg:"Expected int, found dict".to_string()}),
        Ok(BType::BList(_)) => return Err(BencodeParseError::UnexpectedBencodeValue{msg:"Expected int, found list".to_string()}), 
    }
    match get_value(String::from("peers"), &decoded_btype) {
        Ok(BType::BString(s)) => peers = s,
        Err(e) => return Err(e),
        Ok(BType::BDict(_)) => return Err(BencodeParseError::UnexpectedBencodeValue{msg:"Expected string, found dict".to_string()}),
        Ok(BType::BInt(_)) => return Err(BencodeParseError::UnexpectedBencodeValue{msg:"Expected string, found int".to_string()}),
        Ok(BType::BList(_)) => return Err(BencodeParseError::UnexpectedBencodeValue{msg:"Expected string, found list".to_string()}),
    }
    return Ok((interval as u64, peers));
}