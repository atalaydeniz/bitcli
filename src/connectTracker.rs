pub struct ResponseTracker {
    interval: usize,
    peers: String
}

pub fn connect_tracker_http1() -> Result<(), ureq::Error> {
    let res = ureq::get(String::from("https://doc.rust-lang.org/stable/"))
    .call()?
    .body_mut()
    .read_to_string()?;
    println!("{}", res);
    Ok(())
}

pub fn connect_tracker_http(url: String, info_hash: String, peer_id: String, ip: String, port: usize, left: u128, compact: usize)
 -> Result<(), ureq::Error> {
    let query = vec![("info_hash", info_hash), ("peer_id", peer_id), ("port", port.to_string()),
        ("left", left.to_string())];
    ureq::get(url)
    .query_pairs(query)
    .call();

    Ok(())
}