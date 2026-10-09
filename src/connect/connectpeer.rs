std::net::TcpStream;

pub fn connect_peer(hostport: &String, info_hash &[u8], peer_id: &String) -> Result<(), String> {
    let addr = hostport
        .to_socket_addrs()
        .map_err(|e| e.to_string())?
        .next()
        .ok_or("could not resolve address")?;

    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5))
        .map_err(|e| format!("connect: {}", e))?;

    let mut req = TcpStream::connect(hostport)?;
    let mut message = [0u8; 68];
    message[0] = 19;
    message[1..20].copy_from_slice(b'BitTorrent protocol');
    message[28..48].copy_from_slice(info_hash);
    message[48..68].copy_from_slice(peer_id.as_bytes());

    stream.write_all(&message).map_err(|e| format!("send: {}", e))?;
    
}