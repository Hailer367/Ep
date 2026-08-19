mod ec;
mod ec51;

use sha2::Digest;

fn main() {
    let mut ok = 0u64;
    let mut bad = 0u64;
    let mut first_bad: Option<(u64, [u8; 20], [u8; 20])> = None;

    for n in 1..=3000u64 {
        let a = ec::hash160_of_n(n);
        let b = ec51::hash160_of_n51(n);
        if a == b {
            ok += 1;
        } else {
            bad += 1;
            if first_bad.is_none() {
                first_bad = Some((n, a, b));
            }
        }
    }

    let randoms = [
        1234567u64,
        999999999u64,
        1_000_000_000u64,
        42_000_000u64,
        7u64,
        2u64,
    ];
    for n in randoms {
        let a = ec::hash160_of_n(n);
        let b = ec51::hash160_of_n51(n);
        if a == b {
            ok += 1;
        } else {
            bad += 1;
            if first_bad.is_none() {
                first_bad = Some((n, a, b));
            }
        }
    }

    println!("ok={} bad={}", ok, bad);
    if let Some((n, a, b)) = first_bad {
        println!("FIRST MISMATCH n={}", n);
        println!("  ec   = {:02x?}", a);
        println!("  ec51 = {:02x?}", b);
    } else {
        println!("ALL MATCH");
    }

    let mut rok = 0u64;
    let mut rbad = 0u64;
    let mut rng = 123456789u32;
    let mut mix = [0u8; 64];
    for i in 0..5000u64 {
        let len = (i % 64) as usize;
        for b in mix.iter_mut() {
            rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
            *b = (rng >> 24) as u8;
        }
        let a = ec51::ripemd160_opt(&mix[..len]);
        let mut hasher = ripemd::Ripemd160::new();
        hasher.update(&mix[..len]);
        let b = hasher.finalize();
        let mut b20 = [0u8; 20];
        b20.copy_from_slice(&b);
        if a == b20 {
            rok += 1;
        } else {
            rbad += 1;
            if rbad == 1 {
                println!("RIPEMD MISMATCH len={}", len);
                println!("  opt    = {:02x?}", a);
                println!("  crate  = {:02x?}", b20);
            }
        }
    }
    println!("ripemd ok={} bad={}", rok, rbad);
}