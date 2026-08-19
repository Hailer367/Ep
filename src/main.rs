use std::env;
use std::process;

use sha2::{Digest, Sha256};

const B58: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

fn base58_encode(data: &[u8]) -> String {
    let zeros = data.iter().take_while(|&&b| b == 0).count();
    let mut num = data.to_vec();
    let mut encoded = Vec::with_capacity(40);
    let mut start = zeros;
    while start < num.len() {
        let mut rem = 0u32;
        let mut i = start;
        while i < num.len() {
            let acc = (rem << 8) | (num[i] as u32);
            num[i] = (acc / 58) as u8;
            rem = acc % 58;
            i += 1;
        }
        encoded.push(B58[rem as usize]);
        while start < num.len() && num[start] == 0 {
            start += 1;
        }
    }
    for _ in 0..zeros {
        encoded.push(b'1');
    }
    encoded.reverse();
    String::from_utf8(encoded).unwrap()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: ephil <number>");
        process::exit(1);
    }
    let n = match u64::from_str_radix(&args[1], 10) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Invalid number: {}", e);
            process::exit(1);
        }
    };

    let mut key_bytes = [0u8; 32];
    let mut tmp = n;
    for i in (0..32).rev() {
        key_bytes[i] = (tmp & 0xFF) as u8;
        tmp >>= 8;
    }

    let mut wif_bytes = Vec::with_capacity(1 + 32 + 4);
    wif_bytes.push(0x80);
    wif_bytes.extend_from_slice(&key_bytes);

    let first = Sha256::digest(&wif_bytes);
    let second = Sha256::digest(&first);
    wif_bytes.extend_from_slice(&second[..4]);

    println!("{}", base58_encode(&wif_bytes));
}
