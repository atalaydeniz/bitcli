pub mod bencode;

fn main() -> () {
    let input = String::from("d4:spaml1:a1:bee");
    match bencode::parse_start(&input) {
        Ok(b) => println!("{:?}", b),
        Err(x) => println!("{}", x)
    }
    
}

