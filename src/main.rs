pub mod bencode;

fn main() -> () {
    let input = String::from("i02e");
    match bencode::parse_start(&input) {
        Ok(b) => println!("{:?}", b),
        Err(x) => println!("{}", x)
    }
    
}

