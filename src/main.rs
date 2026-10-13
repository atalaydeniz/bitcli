pub mod bencode;
pub mod connect;
pub mod sha;
use bencode::{BType, read_and_decode, decode, encode, bstring_to_ascii, get_value_announce, get_value_info};
use crate::connect::{get_trackers, Trackers, connect};
use std::error::Error;
use std::env;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        return Err("usage: program <torrent file>".into());
    }
    let torrent_path = &args[1];
    let torrent_file = read_and_decode(torrent_path)?;
    let (btype_announce, btype_announce_list) = get_value_announce(&torrent_file)?;
    let announce_urls = get_trackers(btype_announce, btype_announce_list)?;
    let mut tracker_list = Trackers{tracker_list: Vec::new()};

    let info_hash = sha::sha1_string(encode(&get_value_info(&torrent_file)?));
    let peer_id = connect::gen_peer_id();

    let peer_list = connect(&mut tracker_list, &announce_urls, &info_hash, &peer_id, 6900, 32768)?;
    for peer in peer_list {
        match connect::connectpeer::connect_peer(&peer, &sha::sha1_to_bytes(&info_hash), &peer_id) {
            Err(e) => println!("{}: {}", &peer, e.to_string()),
            Ok(()) => println!("Success"),
        }   
    }

    /*
    match decoded_file {
        Ok(b) => {
            match b {
                BType::BDict(d) => {
                    let url = bstring_to_ascii(get_value(String::from("announce"), &d).unwrap());
                    let info_hash = sha::sha1_string(encode(get_value(String::from("info"), &d).unwrap()).unwrap());
                    let peer_id = connect::gen_peer_id();
                    let tracker_list = connect::get_trackers(get_value(String::from("announce"), &d).unwrap(), get_value(String::from("announce-list"), &d).unwrap())?; 
                    let track_response = connect::connect(&tracker_list, &info_hash, &peer_id, 6879, &"32768".to_string())?;
                    let (interval, peers) = connect::handlehttp::parse_response(&track_response)?; 
                    let list_of_peers = connect::connectpeer::get_peer_addresses(&peers);
                    for peer in list_of_peers {
                        match connect::connectpeer::connect_peer(&peer, &sha::sha1_to_bytes(&info_hash), &peer_id) {
                            Ok(()) => println!("WHAT"),
                            Err(e) => println!("{}", peer)
                        }
                    }
                }
                _ => {return Err(format!("Dictionary error"));}
            }
        }
        _ => return Err(format!("Decoding error"))
    }
    */
    Ok(())
}

