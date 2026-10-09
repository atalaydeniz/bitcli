pub mod bencode;
pub mod connect;
pub mod sha;
use bencode::{BType, decode, encode, get_value, bstring_to_ascii};

fn main() -> Result<(), String> {

    let torrent_file = connect::read_torrent(r"C:\Users\deniz\Downloads\26F1CE29E36B0B4B72A1B0974776AF082F97FCB0.torrent").unwrap();
    let decoded_file = decode(&torrent_file);
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
    Ok(())
}

