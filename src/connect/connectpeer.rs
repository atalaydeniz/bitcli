use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

pub fn get_peer_addresses(peers: &[u8]) -> Vec<String> {
    peers
        .chunks_exact(6)
        .map(|c| {
            let port = u16::from_be_bytes([c[4], c[5]]);
            format!("{}.{}.{}.{}:{}", c[0], c[1], c[2], c[3], port)
        })
        .collect()
}

pub fn connect_peer(hostport: &String, info_hash: &[u8], peer_id: &String) -> Result<(), String> {
    let addr = hostport
        .to_socket_addrs()
        .map_err(|e| e.to_string())?
        .next()
        .ok_or("could not resolve address")?;

    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5))
        .map_err(|e| format!("connect: {}", e))?;

    let mut message = [0u8; 68];
    message[0] = 19;
    message[1..20].copy_from_slice(b"BitTorrent protocol");
    message[28..48].copy_from_slice(info_hash);
    message[48..68].copy_from_slice(peer_id.as_bytes());

    stream.write_all(&message).map_err(|e| format!("send: {}", e))?;

    let mut reply = [0u8; 68];
    stream.read_exact(&mut reply).map_err(|e| format!("no handshake reply: {}", e))?;

    if reply[0] != 19 || &reply[1..20] != b"BitTorrent protocol" {
        return Err("response is not a BitTorrent protocol message".to_string());
    } 

    if &reply[28..48] != info_hash {
        return Err("info_hash mismatch".to_string());
    }

    println!("Handshake OK, peer id: {}", String::from_utf8_lossy(&reply[48..68]));
    Ok(())
}