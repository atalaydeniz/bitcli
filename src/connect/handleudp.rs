use std::net::UdpSocket;
use std::time::Duration;
use rand::RngExt;

const PROTOCOL_ID: u64 = 0x41727101980; // magic constant :)
const ACTION_CONNECT: u32 = 0;
const ACTION_ERROR: u32 = 3;

pub fn udp_host(url: &str) -> Option<&str> {
    let rest = url.strip_prefix("udp://")?;
    Some(rest.split('/').next()?)
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