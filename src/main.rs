pub mod bencode;
pub mod connect;
pub mod sha;
use bencode::{BType, decode, encode, get_value, bstring_to_ascii};

fn main() -> () {

    let torrent_file = connect::read_torrent(r"C:\Users\deniz\Downloads\26F1CE29E36B0B4B72A1B0974776AF082F97FCB0.torrent").unwrap();
    let decoded_file = decode(&torrent_file);
    match decoded_file {
        Ok(b) => {
            match b {
                BType::BDict(d) => {
                    let url = bstring_to_ascii(get_value(String::from("announce"), &d).unwrap());
                    let info_hash = sha::sha1_string(encode(get_value(String::from("info"), &d).unwrap()).unwrap());
                    let peer_id = connect::gen_peer_id();
                    //let length = bstring_to_ascii(get_value(String::from("length"), &d).unwrap());
                    match connect::get_trackers(get_value(String::from("announce"), &d).unwrap(), get_value(String::from("announce-list"), &d).unwrap()) {
                        Ok(v) => {
                            match connect::connect(&v, &info_hash, &peer_id, 6879, &"32768".to_string()) {
                                Ok(()) => {
                                    println!("Good");
                                }
                                Err(e) => {
                                    println!("{}", e);
                                }
                            }
                        }
                        Err(e) => println!("{}", e)
                    }
                }
                _ => {();}
            }
        }
        _ => ()
    }

}

