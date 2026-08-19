mod ec;
mod ec51;

use sha2::Digest;
use sha2::Sha256;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let count: u64 = args
        .get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(5_000_000);

    let input = [0x42u8; 33];
    let sha_only = count;
    let start = Instant::now();
    for _ in 0..sha_only {
        std::hint::black_box(Sha256::digest(std::hint::black_box(&input)));
    }
    let t = start.elapsed().as_secs_f64();
    println!("sha256 only : {:.3}s {:.1} M/s", t, count as f64 / t / 1e6);

    let mut mid = [0u8; 32];
    mid.copy_from_slice(&input[..32]);
    let start = Instant::now();
    for _ in 0..sha_only {
        let mut h = ripemd::Ripemd160::new();
        h.update(std::hint::black_box(&mid));
        std::hint::black_box(h.finalize());
    }
    let t = start.elapsed().as_secs_f64();
    println!("ripemd only : {:.3}s {:.1} M/s", t, count as f64 / t / 1e6);

    let start = Instant::now();
    for _ in 0..sha_only {
        std::hint::black_box(ec::hash160_from_compressed(std::hint::black_box(&input)));
    }
    let t = start.elapsed().as_secs_f64();
    println!("hash160     : {:.3}s {:.1} M/s", t, count as f64 / t / 1e6);

    let start = Instant::now();
    for _ in 0..sha_only {
        std::hint::black_box(ec51::ripemd160_opt(std::hint::black_box(&mid)));
    }
    let t = start.elapsed().as_secs_f64();
    println!("ripemd opt  : {:.3}s {:.1} M/s", t, count as f64 / t / 1e6);

    let start = Instant::now();
    for _ in 0..sha_only {
        std::hint::black_box(ec51::hash160_fast(std::hint::black_box(&input)));
    }
    let t = start.elapsed().as_secs_f64();
    println!("hash160 fast: {:.3}s {:.1} M/s", t, count as f64 / t / 1e6);
}