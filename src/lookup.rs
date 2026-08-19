mod ec;
mod table;

use std::env;
use std::process;

use table::Table;

fn parse_arg(args: &[String], name: &str) -> Option<String> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == name {
            return args.get(i + 1).cloned();
        }
        i += 1;
    }
    None
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let address = match args.iter().find(|a| a.starts_with("bc1")) {
        Some(a) => a.clone(),
        None => {
            eprintln!("Usage: ephil-lookup <bech32-address> --table FILE");
            process::exit(1);
        }
    };
    let path = match parse_arg(&args, "--table") {
        Some(s) => s,
        None => {
            eprintln!("Usage: ephil-lookup <bech32-address> --table FILE");
            process::exit(1);
        }
    };

    let h160 = match ec::decode_bech32_address(&address) {
        Some(h) => h,
        None => {
            eprintln!("invalid bech32 address");
            process::exit(1);
        }
    };

    let table = Table::load(&path).unwrap_or_else(|e| {
        eprintln!("failed to open table: {}", e);
        process::exit(1);
    });

    match table.lookup(&h160) {
        Some(n) => println!("{}", n),
        None => {
            println!("not found");
            process::exit(1);
        }
    }
}