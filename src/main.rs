pub mod bencode;
pub mod connecttracker;
pub mod sha;

fn main() -> () {

    let torrent_file = sha::read_torrent(r"C:\Users\deniz\Downloads\26F1CE29E36B0B4B72A1B0974776AF082F97FCB0.torrent").unwrap();
    let decoded_file = bencode::decode(&torrent_file);
    match decoded_file {
        Ok(b) => bencode::print_decoded(b),
        _ => ()
    }

    //let encoded = bencode::encode(bencode::BType::BList(Vec::from([bencode::BType::BInt(56), bencode::BType::BString(String::from("abc"))]))).unwrap();
//    println!("{:?}", encoded);

}

