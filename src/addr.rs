mod ec;

use std::env;
use std::process;

use ec::{bech32_address, hash160_of_n};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: ephil-addr <number>");
        process::exit(1);
    }
    let n = match u64::from_str_radix(&args[1], 10) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Invalid number: {}", e);
            process::exit(1);
        }
    };

    let h160 = hash160_of_n(n);
    let address = bech32_address(&h160);
    println!("{}", address);
}
