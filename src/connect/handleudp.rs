use std::net::UdpSocket;
use std::time::Duration;
use rand::RngExt;
use crate::connect::{ConnectionError};

const PROTOCOL_ID: u64 = 0x41727101980; // magic constant :)
const ACTION_CONNECT: u32 = 0;
const ACTION_ANNOUNCE: u32 = 1;
const ACTION_ERROR: u32 = 3;
const EVENT_STARTED: u32 = 2;

pub fn udp_req_tracker_connect(addr: &String, info_hash: &String, peer_id: &String, downloaded: u64,
    left: u64, uploaded: u64) -> Result<Vec<u8>, ConnectionError> {
    let socket = UdpSocket::bind("0.0.0.0:0").
        map_err(|e| ConnectionError::UdpConnectionError(e.to_string()))?;

    socket.set_read_timeout(Some(Duration::from_secs(5))).
        map_err(|e| ConnectionError::UdpConnectionError(e.to_string()))?;

    socket.connect(addr).
        map_err(|e| ConnectionError::UdpConnectionError(e.to_string()))?;

    let mut transaction_id: u32 = rand::rng().random();
    let mut req = [0u8; 16];

    req[0..8].copy_from_slice(&PROTOCOL_ID.to_be_bytes());
    req[8..12].copy_from_slice(&ACTION_CONNECT.to_be_bytes());
    req[12..16].copy_from_slice(&transaction_id.to_be_bytes());

    println!("Sending to {}: {:02x?}", addr, req);
    socket.send(&req).
        map_err(|e| ConnectionError::UdpConnectionError(e.to_string()))?;

    let mut buf = [0u8; 1024];
    let mut n = socket.recv(&mut buf).
        map_err(|e| ConnectionError::UdpConnectionError(e.to_string()))?;
    
    if n < 16 {
        return Err(ConnectionError::UdpResponseLength(n, ACTION_CONNECT));
    }

    let mut action = u32::from_be_bytes(buf[0..4].try_into().unwrap());
    let mut resp_tid = u32::from_be_bytes(buf[4..8].try_into().unwrap());
    
    if resp_tid != transaction_id {
        return Err(ConnectionError::UdpTransactionIdMismatch(resp_tid.to_string(), transaction_id.to_string()));
    }
    if action == ACTION_ERROR {
        return Err(ConnectionError::UdpTrackerError(String::from_utf8_lossy(&buf[8..n])));
    }
    if action != ACTION_CONNECT {
        return Err(ConnectionError::UdpUnknownActionCode(String::from_utf8_lossy(&buf[0..4])));
    }

    transaction_id = rand::rng().random();
    let info_hash_bytes = sha::sha1_to_bytes(info_hash);

    let mut announce_req = [0u8; 98];
    announce_req[0..8].copy_from_slice(&buf[8..16]);
    announce_req[8..12].copy_from_slice(&ACTION_ANNOUNCE.to_be_bytes());
    announce_req[12..16].copy_from_slice(&transaction_id.to_be_bytes());
    announce_req[16..36].copy_from_slice(&info_hash_bytes.as_slice());
    announce_req[36..56].copy_from_slice(peer_id.as_bytes());
    announce_req[56..64].copy_from_slice(downloaded.to_be_bytes());
    announce_req[64..72].copy_from_slice(left.to_be_bytes());
    announce_req[72..80].copy_from_slice(uploaded.to_be_bytes());
    announce_req[80..84].copy_from_slice(&EVENT_STARTED.to_be_bytes());
    // Use this for IP address later
    announce_req[84..88].copy_from_slice(&0u32.to_be_bytes());
    announce_req[92..96].copy_from_slice(&-1i32.to_be_bytes());
    announce_req[96..98].copy_from_slice(&6886u16.to_be_bytes());

    println!("Sending {:02x?}", announce_req);
    socket.send(&announce_req).
        map_err(|e| ConnectionError::UdpConnectionError(e.to_string()))?;

    let mut announce_buf = [0u8; 4096];
    n = socket.recv(&mut announce_buf)
        .map_err(|e| ConnectionError::UdpConnectionError(e.to_string()))?;

    if n < 20 {
        return Err(ConnectionError::UdpResponseLength(n, ACTION_ANNOUNCE));
    }

    action = u32::from_be_bytes(announce_buf[0..4].try_into().unwrap());
    resp_tid = u32::from_be_bytes(announce_buf[4..8].try_into().unwrap());
    let interval: u32 = u32::from_be_bytes(announce_buf[8..12].try_into().unwrap());
    let leechers: u32 = u32::from_be_bytes(announce_buf[12..16].try_into().unwrap());
    let seeders: u32 = u32::from_be_bytes(announce_buf[16..20].try_into().unwrap());
    let peers: Vec<u8> = announce_buf[20..n].to_vec();

    if action == ACTION_ERROR {
        return Err(ConnectionError::UdpTrackerError(String::from_utf8_lossy(&announce_buf[8..n])));
    }

    if resp_tid != transaction_id {
        return Err(ConnectionError::UdpTransactionIdMismatch(resp_tid.to_string(), transaction_id.to_string()));
    }

    if action != ACTION_ANNOUNCE {
        return Err(ConnectionError::UdpUnknownActionCode(String::from_utf8_lossy(&announce_buf[0..4])));
    }

    Ok(peers)   
}