use std::fs;
use rand::RngExt;
use crate::bencode::{BType,bstring_to_ascii};
use std::net::UdpSocket;
use std::time::Duration;

const PROTOCOL_ID: u64 = 0x41727101980; // magic constant :)
const ACTION_CONNECT: u32 = 0;
const ACTION_ERROR: u32 = 3;

pub struct ResponseTracker {
    interval: usize,
    peers: String
}

pub fn read_torrent(path: &str) -> std::io::Result<Vec<u8>> {
    return fs::read(path);
}

pub fn gen_peer_id() -> String {
    let mut random_generated_vec = Vec::<u8>::new();
    let mut rng = rand::rng();
    let mut x: u8;
    let mut index = 0;
    while index < 20 {
        x = rng.random_range(97..123);
        random_generated_vec.push(x);
        index = index + 1;
    }
    return String::from_utf8(random_generated_vec).unwrap();
}

pub fn get_trackers(b_announce: BType, b_announce_list: BType) -> Result<Vec<String>, String> {
    let mut tracker_list = Vec::new();
    match b_announce {
        BType::BString(s) => {
            let announce = String::from_utf8(s).unwrap();
            tracker_list.push(announce);
        }
        _ => {
            return Err(String::from("get_trackers() error: Bad request"));
        }
    }
    match b_announce_list {
        BType::BList(l) => {
            for elem in l {
                match elem {
                    BType::BString(s) => {
                        tracker_list.push(String::from_utf8(s).unwrap());
                    }
                    BType::BList(ll) => {
                        for elemll in ll {
                            match elemll {
                                BType::BString(ss) => {
                                    tracker_list.push(String::from_utf8(ss).unwrap());
                                }
                                _ => {
                                    return Err(String::from("get_trackers() error: Bad request"));
                                }
                            }
                        }
                    }
                    _ => {
                        return Err(String::from("get_trackers() error: Bad request"));
                    }
                }
            }
        }
        _ => {
           return Err(String::from("get_trackers() error: Bad request"));
        }
    }
    println!("{:?}", tracker_list);
    return Ok(tracker_list);
}

pub fn connect_tracker_http(url: &String, info_hash: &String, peer_id: &String, port: usize, left: &String, compact: usize)
 -> Result<(), ureq::Error> {
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
            println!("Wow");
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
    Ok(())
}

pub fn udp_host(url: &str) -> Option<&str> {
    let rest = url.strip_prefix("udp://")?;
    Some(rest.split('/').next()?)
}

pub fn connect(url_list: &Vec<String>, info_hash: &String, peer_id: &String, port: usize, left: &String) -> Result<(), String> {
    for url in url_list {
        match udp_host(url.as_str()) {
            Some(udp_url) => {
                match udp_req_tracker_connect(udp_url) {
                    Ok(i) => {
                        return Ok(());
                    }
                    Err(e) => {
                        println!("{}", e);
                    }
                }
            }
            None => {
                connect_tracker_http(url, info_hash, peer_id, port, left, 1);
            }
        }
    }
    return Err("No url is responding".to_string());
}

pub fn udp_req_tracker_connect(addr: &str) -> Result<u64, String> {
    let socket = UdpSocket::bind("127.0.0.1:6879").map_err(|e| e.to_string())?;
    socket.set_read_timeout(Some(Duration::from_secs(5))).map_err(|e| e.to_string())?;
    socket.connect(addr).map_err(|e| e.to_string())?;

    let transaction_id: u32 = rand::rng().random();
    let mut req = [0u8; 16];
    req[0..8].copy_from_slice(&PROTOCOL_ID.to_be_bytes());
    req[8..12].copy_from_slice(&ACTION_CONNECT.to_be_bytes());
    req[12..16].copy_from_slice(&transaction_id.to_be_bytes());

    println!("Sending to {}: {:02x?}", addr, req);
    socket.send(&req).map_err(|e| e.to_string())?;

    let mut buf = [0u8; 16];
    let n = socket.recv(&mut buf).map_err(|e| format!("No response: {}", e))?;
    if n < 16 {
        return Err(format!("Response too short ({} bytes)", n));
    }

    let action = u32::from_be_bytes(buf[0..4].try_into().unwrap());
    let resp_tid = u32::from_be_bytes(buf[4..8].try_into().unwrap());
    
    if resp_tid != transaction_id {
        return Err("Transaction ID mismatch".to_string());
    }
    if action == ACTION_ERROR {
        return Err(format!("Tracker error: {}", String::from_utf8_lossy(&buf[8..n])));
    }
    if action != ACTION_CONNECT || n < 16 {
        return Err(format!("Unexpected response (action {}, {} bytes)", action, n));
    }

    println!("{:?}", buf);
    println!("AAAAAAAAAAA");
    Ok(u64::from_be_bytes(buf[8..16].try_into().unwrap()))
    
}