pub mod handlehttp;
pub mod handleudp;
pub mod connectpeer;
use std::fs;
use rand::RngExt;
use crate::bencode::{BType,bstring_to_ascii};
use crate::connect::handleudp::{udp_host, udp_req_tracker_connect};
use crate::connect::handlehttp::{connect_tracker_http};

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

pub fn connect(url_list: &Vec<String>, info_hash: &String, peer_id: &String, port: usize, left: &String) -> Result<Vec<u8>, String> {
    return Err("Try".to_string());
    
    for url in url_list {
        match udp_host(url.as_str()) {
            Some(udp_url) => {
                match udp_req_tracker_connect(udp_url) {
                    Ok(i) => {
                        println!("Udp works");
                    }
                    Err(e) => {
                        println!("{}", e);
                    }
                }
            }
            None => {
                let response_result = connect_tracker_http(url, info_hash, peer_id, port, left, 1) ;
                match response_result {
                    Ok(v) => return Ok(v),
                    Err(error) => println!("{}", error)
                }
            }
        }
    }
    return Err("No url is responding".to_string());
}
