pub mod handlehttp;
pub mod handleudp;
pub mod connectpeer;
use std::fs;
use std::fmt;
use rand::RngExt;
use crate::bencode::{BType,bstring_to_ascii};
use crate::connect::handleudp::{udp_req_tracker_connect};
use crate::connect::handlehttp::{connect_tracker_http};

pub enum ConnectionError {
    AnnounceUrlNotFound,
    UrlContainNonUTF8,
    NoUrlResponding, 
    UdpConnectionError(String),
    UdpResponseLength(usize, u32),
    UdpTransactionIdMismatch(String, String),
    UdpTrackerError(String),
    UdpUnknownActionCode(String)
    HttpConnectionError(String)
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ConnectionError::*;
        match self {
            AnnounceUrlNotFound => write!(f, "\'announce\' key not found in tracker response"),
            UrlContainNonUTF8 => write!(f, "URL in the announce field contain non utf-8 chars"),
            NoUrlResponding => write!(f, "No URL in the tracker is responding"),
            UdpConnectionError(e) => write!(f, "UDP connection error: {}", e),
            UdpResponseLength(i, code) => write!(f, "UDP response, for action {}, length too short: {}", code, i),
            UdpTransactionIdMismatch(s1, s2) => write!(f, "Transaction ID mismatch: {}, {}", s1, s2),
            UdpTrackerError(e) => write!(f, "Tracker error: {}", e),
            UdpUnknownActionCode(e) => write!(f, "Unexpected action value: {}", e), 
            HttpConnectionError(e) => write!(f, "HTTP connection error: {}", e),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerUrl {
    UdpUrl(String),
    HttpUrl(String),
}

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

pub fn get_trackers(b_announce: BType, b_announce_list: BType) -> Result<Vec<String>, ConnectionError> {
    let mut tracker_list = Vec::new();
    match b_announce {
        BType::BString(s) => tracker_list.push(String::from_utf8(s).map_err(|_| ConnectionError::UrlContainNonUTF8))?,
        _ => return Err(ConnectionError::AnnounceUrlNotFound),
    }
    match b_announce_list {
        BType::BList(l) => {
            for elem in l {
                match elem {
                    BType::BString(s) => tracker_list.push(String::from_utf8(s).map_err(|_| ConnectionError::UrlContainNonUTF8))?,
                    BType::BList(ll) => {
                        for elemll in ll {
                            match elemll {
                                BType::BString(ss) => tracker_list.push(String::from_utf8(ss).map_err(|_| ConnectionError::UrlContainNonUTF8))?,
                                _ => return Err(ConnectionError::AnnounceUrlNotFound),
                            }
                        }
                    }
                    _ => return Err(ConnectionError::AnnounceUrlNotFound),
                }
            }
        }
        _ => return Err(ConnectionError::AnnounceUrlNotFound),
    }
    println!("{:?}", tracker_list);
    return Ok(tracker_list);
}

pub fn connect(url_list: &Vec<String>, info_hash: &String, peer_id: &String, port: usize, left: &String) -> 
Result<Vec<u8>, ConnectionError> { 
    for url in url_list {
        match get_url_http_or_udp(url.as_str()) {
            Some(PeerUrl::UdpUrl(udp_url)) => { 
                match udp_req_tracker_connect(udp_url) {
                    Ok(i) => return Ok(v),
                    Err(e) => println!("{}", e),
                }
            }
            Some(PeerUrl::HttpUrl(http_url)) => {
                let response_result = connect_tracker_http(&http_url, info_hash, peer_id, port, left, 1);
                match response_result {
                    Ok(v) => return Ok(v),
                    Err(error) => println!("{}", error)
                }
            }
            None => println!("URL couldn't resolved")
        }
    }
    return Err(ConnectionError::NoUrlResponding);
}

pub fn get_url_http_or_udp(url: &str) -> Option<PeerUrl> {
    if let Some(rest) = url.strip_prefix("udp://") {
        let host_port = rest.split('/').next().unwrap_or("");
        if host_port.is_empty() {
            return None;
        }
        Some(PeerUrl::UdpUrl(host_port.to_string()))
    } else if url.starts_with("http://") || url.starts_with("https://") {
        Some(PeerUrl::HttpUrl(url.to_string()))
    } else {
        None
    }
}