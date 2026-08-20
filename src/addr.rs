mod ec;
mod key;

use std::env;
use std::process;

use ec::{bech32_address, hash160_of_key};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: ephil-addr <number>");
        eprintln!("  <number> is a decimal private key in [1, 2^160) (Bech32/P2WPKH key space)");
        process::exit(1);
    }
    let k = match key::parse(&args[1]) {
        Some(k) => k,
        None => {
            eprintln!(
                "Invalid number (must be a decimal integer in [1, 2^160)): {}",
                args[1]
            );
            process::exit(1);
        }
    };

    let h160 = hash160_of_key(&k);
    let address = bech32_address(&h160);
    println!("{}", address);
}