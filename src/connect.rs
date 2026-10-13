pub mod handlehttp;
pub mod handleudp;
pub mod connectpeer;
use std::fs;
use std::fmt;
use rand::RngExt;
use std::time::{Duration, Instant};
use crate::bencode::{BType,bstring_to_ascii};
use crate::connect::handleudp::{udp_req_tracker_connect};
use crate::connect::handlehttp::{connect_tracker_http};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionError {
    AnnounceUrlNotFound,
    UrlContainNonUTF8,
    NoUrlResponding, 
    UdpConnectionError(String),
    UdpResponseLength(usize, u32),
    UdpTransactionIdMismatch(String, String),
    UdpTrackerError(String),
    UdpUnknownActionCode(String),
    HttpConnectionError(String),
    TcpConnectionError(String),
    TcpAddressNotResolved(String),
    TcpTimeout(String),
    TcpSendError(String),
    TcpNoHandshakeReply(String),
    TcpProtocolMismatch,
    TcpHashMismatch,
    GenericError(String)
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
            TcpConnectionError(e) => write!(f, "TCP connection error: {}", e),
            TcpAddressNotResolved(e) => write!(f, "TCP host address not resolved: {}", e),
            TcpTimeout(e) => write!(f, "TCP timeout: {}", e),
            TcpSendError(e) => write!(f, "TCP could not send: {}", e),
            TcpNoHandshakeReply(e) => write!(f, "TCP no handshake reply: {}", e),
            TcpProtocolMismatch => write!(f, "Response is not a BitTorrent protocol message"),
            TcpHashMismatch => write!(f, "info_hash mismatch"),
            GenericError(e) => write!(f, "generic error: {}", e)
        }
    }
}

impl std::error::Error for ConnectionError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerUrl {
    UdpUrl(String),
    HttpUrl(String),
}

pub struct Trackers {
    pub tracker_list: Vec<TrackerState>
}

pub struct TrackerState {
    pub tracker_id: String,
    pub last_connection: Instant,
    pub last_response: Option<TrackerResponse>,
}

pub struct TrackerResponse {
    pub interval: Duration,
    pub peers: Vec<String>,
}

impl Trackers {
    pub fn find(&mut self, searched_tracker_id: &str) -> Option<&mut TrackerState> {
        self.tracker_list
            .iter_mut()
            .find(|t| t.tracker_id == searched_tracker_id)
    }
}

impl TrackerState {
    pub fn can_connectable(&mut self) -> bool {
        let now = Instant::now();
        match &self.last_response {
            None => true,
            Some(res) => {
                return now.duration_since(self.last_connection) >= res.interval;
            }
        }
    }
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
        BType::BString(s) => tracker_list.push(String::from_utf8(s).map_err(|_| ConnectionError::UrlContainNonUTF8)?),
        _ => return Err(ConnectionError::AnnounceUrlNotFound),
    }
    match b_announce_list {
        BType::BList(l) => {
            for elem in l {
                match elem {
                    BType::BString(s) => tracker_list.push(String::from_utf8(s).map_err(|_| ConnectionError::UrlContainNonUTF8)?),
                    BType::BList(ll) => {
                        for elemll in ll {
                            match elemll {
                                BType::BString(ss) => tracker_list.push(String::from_utf8(ss)
                                    .map_err(|_| ConnectionError::UrlContainNonUTF8)?),
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

pub fn connect(all_trackers: &mut Trackers, url_list: &Vec<String>, info_hash: &String, peer_id: &String, port: usize, left: u64) -> 
Result<Vec<String>, ConnectionError> { 
    for url in url_list {
        match get_url_http_or_udp(url.as_str()) {
            Some(PeerUrl::UdpUrl(udp_url)) => {
                match all_trackers.find(&udp_url) {
                    Some(tracker_state) => {
                        if !tracker_state.can_connectable() { continue; }
                        match udp_req_tracker_connect(&udp_url, info_hash, peer_id, 0, left, 0) {
                            Ok(tracker_response) => {
                                let peer_list = (&tracker_response).peers.clone();
                                tracker_state.last_connection = Instant::now();
                                tracker_state.last_response = Some(tracker_response);
                                println!("Getting peers from the tracker: {}", &url);
                                return Ok(peer_list);
                            }
                            Err(e) => println!("{}", e),
                        }
                    }
                    None => {
                        let mut new_tracker_state = TrackerState{tracker_id: (&udp_url).clone(), last_connection: Instant::now(), last_response: None};
                        match udp_req_tracker_connect(&udp_url, info_hash, peer_id, 0, left, 0) {
                            Ok(tracker_response) => {
                                let peer_list = (&tracker_response).peers.clone();
                                new_tracker_state.last_connection = Instant::now();
                                new_tracker_state.last_response = Some(tracker_response);
                                all_trackers.tracker_list.push(new_tracker_state);
                                println!("Getting peers from the tracker: {}", &url);
                                return Ok(peer_list);
                            }
                            Err(e) => println!("{}", e),
                        }
                    }
                }
            }
            Some(PeerUrl::HttpUrl(http_url)) => {
                let Some(tracker_state) = all_trackers.find(&http_url) else { continue };
                if !tracker_state.can_connectable() { continue; }
                match connect_tracker_http(&http_url, info_hash, peer_id, 6900, 0, left, 0, 1) {
                    Ok(tracker_response) => {
                        let peer_list = (&tracker_response).peers.clone();
                        tracker_state.last_connection = Instant::now();
                        tracker_state.last_response = Some(tracker_response);
                        println!("Getting peers from the tracker: {}", &url);
                        return Ok(peer_list);
                    }
                    Err(e) => println!("{}", e),
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
