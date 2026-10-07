pub mod bencode;
pub mod connecttracker;
pub mod sha;

fn main() -> () {
    let input = String::from("d4:spaml1:a1:bee");
    //match bencode::decode(&input) {
    //    Ok(b) => println!("{:?}", b),
    //    Err(x) => println!("{}", x)
    //}

    //let encoded = bencode::encode(bencode::BType::BList(Vec::from([bencode::BType::BInt(56), bencode::BType::BString(String::from("abc"))]))).unwrap();
//    println!("{:?}", encoded);

}

