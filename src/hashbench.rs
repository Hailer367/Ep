mod ec;
mod ec51;

use ripemd::Ripemd160;
use sha2::Digest;
use sha2::Sha256;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let count: u64 = args
        .get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(5_000_000);

    // correctness: check each stage against crate implementations
    let mut seed = 0x12345678u32;
    let mut sha_bad = 0u64;
    let mut rm_bad = 0u64;
    let mut hf_bad = 0u64;
    for _ in 0..200_000 {
        let mut input = [0u8; 33];
        for b in input.iter_mut() {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            *b = (seed >> 24) as u8;
        }
        let mut mid = [0u8; 32];
        mid.copy_from_slice(&input[..32]);

        // reference: SHA-256 of the 33-byte message
        let ref_sha = Sha256::digest(&input);
        let got_sha = ec51::sha256_of_33(&input);
        if ref_sha.as_slice() != &got_sha[..] {
            sha_bad += 1;
            if sha_bad <= 3 {
                println!(
                    "SHA MISMATCH input={:02x?} ref={:02x?} got={:02x?}",
                    &input[..8],
                    &ref_sha[..4],
                    &got_sha[..4]
                );
            }
        }

        // reference: RIPEMD-160 of the 32-byte message
        let mut ref_rm = Ripemd160::new();
        ref_rm.update(&mid);
        let ref_rm = ref_rm.finalize();
        let got_rm = ec51::ripemd160_of_32(&mid);
        if ref_rm.as_slice() != &got_rm[..] {
            rm_bad += 1;
            if rm_bad <= 3 {
                println!(
                    "RIPEMD MISMATCH input={:02x?} ref={:02x?} got={:02x?}",
                    &mid[..8],
                    &ref_rm[..4],
                    &got_rm[..4]
                );
            }
        }

        let a = ec::hash160_from_compressed(&input);
        let b = ec51::hash160_fast33(&input);
        let c = ec51::hash160_fast(&input);
        if a == b && a == c {
        } else {
            hf_bad += 1;
            if hf_bad <= 3 {
                println!(
                    "HASH160 MISMATCH input={:02x?} a={:02x?} b={:02x?} c={:02x?}",
                    &input[..8],
                    &a[..4],
                    &b[..4],
                    &c[..4]
                );
            }
        }
    }
    println!("check: sha_bad={} ripemd_bad={} hash160_bad={}", sha_bad, rm_bad, hf_bad);
    if sha_bad != 0 || rm_bad != 0 || hf_bad != 0 {
        std::process::exit(1);
    }

    // 8x batch correctness
    let mut comps8 = [[0u8; 33]; 8];
    let mut bx_bad = 0u64;
    seed = 0x9e3779b9u32;

    // xv transpose debug: lane j = all bytes 0xBB
    let mut xv_shas = [[0xBBu8; 32]; 8];
    let xv = ec51::xv_debug(&xv_shas);
    for j in 0..8 {
        let expect: [u32; 16] = [
            0xBBBBBBBB, 0xBBBBBBBB, 0xBBBBBBBB, 0xBBBBBBBB,
            0xBBBBBBBB, 0xBBBBBBBB, 0xBBBBBBBB, 0xBBBBBBBB,
            0x80, 0, 0, 0, 0, 0, 256, 0,
        ];
        if xv[j] != expect {
            bx_bad += 1;
            println!("XV MISMATCH lane={} got={:08x?} expect={:08x?}", j, &xv[j][..4], &expect[..4]);
        }
    }
    // lane-varying: lane j = bytes all = j+1
    for j in 0..8 {
        for b in xv_shas[j].iter_mut() {
            *b = j as u8 + 1;
        }
    }
    let xv2 = ec51::xv_debug(&xv_shas);
    for j in 0..8 {
        let expect = (j as u32 + 1) * 0x01010101;
        if xv2[j][0] != expect || xv2[j][7] != expect || xv2[j][14] != 256 || xv2[j][8] != 0x80 || xv2[j][15] != 0 {
            bx_bad += 1;
            println!("XV2 MISMATCH lane={} got={:08x?}", j, &xv2[j][..8]);
        }
    }
    println!("xv check done");

    // ripemd-only 8x check vs scalar
    let mut r8_bad = 0u64;
    for _ in 0..20_000 {
        let mut shas = [[0u8; 32]; 8];
        for lane in 0..8 {
            for b in shas[lane].iter_mut() {
                seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                *b = (seed >> 24) as u8;
            }
        }
        let got8 = ec51::ripemd160_of_32_8x(&shas);
        for lane in 0..8 {
            let expect = ec51::ripemd160_of_32(&shas[lane]);
            if got8[lane] != expect {
                r8_bad += 1;
                if r8_bad <= 3 {
                    println!(
                        "R8 MISMATCH lane={} got={:02x?} expect={:02x?}",
                        lane,
                        &got8[lane][..4],
                        &expect[..4]
                    );
                }
            }
        }
    }
    println!("ripemd 8x check: r8_bad={}", r8_bad);

    // compress-only check vs scalar compress_ripemd
    {
        let mut shas = [[0u8; 32]; 8];
        for lane in 0..8 {
            for b in shas[lane].iter_mut() {
                seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                *b = (seed >> 24) as u8;
            }
        }
        let xvs = ec51::xv_debug(&shas);
        let simd = ec51::compress_8x_debug(&shas);
        for lane in 0..8 {
            let mut block = [0u8; 64];
            for i in 0..16 {
                block[i * 4..i * 4 + 4].copy_from_slice(&xvs[lane][i].to_le_bytes());
            }
            let mut h = [0x67452301u32, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
            ec51::compress_ripemd(&mut h, &block);
            if simd[lane] != h {
                println!(
                    "C8 MISMATCH lane={} got={:08x?} expect={:08x?}",
                    lane, &simd[lane], &h
                );
                std::process::exit(1);
            }
        }
        println!("compress 8x check OK");
    }

    for _ in 0..50_000 {
        for lane in 0..8 {
            let mut input = [0u8; 33];
            for b in input.iter_mut() {
                seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                *b = (seed >> 24) as u8;
            }
            comps8[lane] = input;
        }
        let got8 = ec51::hash160_fast33_8x(&comps8);
        for lane in 0..8 {
            let expect = ec::hash160_from_compressed(&comps8[lane]);
            if got8[lane] != expect {
                bx_bad += 1;
                if bx_bad <= 3 {
                    println!(
                        "8x MISMATCH lane={} got={:02x?} expect={:02x?}",
                        lane,
                        &got8[lane][..4],
                        &expect[..4]
                    );
                }
            }
        }
    }
    println!("8x check: bx_bad={}", bx_bad);
    if bx_bad != 0 {
        std::process::exit(1);
    }

    let input = [0x42u8; 33];
    let sha_only = count;
    let start = Instant::now();
    for _ in 0..sha_only {
        std::hint::black_box(Sha256::digest(std::hint::black_box(&input)));
    }
    let t = start.elapsed().as_secs_f64();
    println!("sha256 crate : {:.3}s {:.1} M/s", t, count as f64 / t / 1e6);

    let mut mid = [0u8; 32];
    mid.copy_from_slice(&input[..32]);
    let start = Instant::now();
    for _ in 0..sha_only {
        std::hint::black_box(ec51::sha256_of_33(std::hint::black_box(&input)));
    }
    let t = start.elapsed().as_secs_f64();
    println!("sha256 of33  : {:.3}s {:.1} M/s", t, count as f64 / t / 1e6);

    let start = Instant::now();
    for _ in 0..sha_only {
        std::hint::black_box(ec51::ripemd160_of_32(std::hint::black_box(&mid)));
    }
    let t = start.elapsed().as_secs_f64();
    println!("ripemd of32  : {:.3}s {:.1} M/s", t, count as f64 / t / 1e6);

    let start = Instant::now();
    for _ in 0..sha_only {
        std::hint::black_box(ec51::hash160_fast33(std::hint::black_box(&input)));
    }
    let t = start.elapsed().as_secs_f64();
    println!("hash160 f33  : {:.3}s {:.1} M/s", t, count as f64 / t / 1e6);

    let start = Instant::now();
    for _ in 0..sha_only {
        std::hint::black_box(ec51::hash160_fast(std::hint::black_box(&input)));
    }
    let t = start.elapsed().as_secs_f64();
    println!("hash160 fast : {:.3}s {:.1} M/s", t, count as f64 / t / 1e6);

    let mut comps8 = [[0u8; 33]; 8];
    for lane in 0..8 {
        comps8[lane] = input;
    }
    let n8 = sha_only / 8;
    let start = Instant::now();
    for _ in 0..n8 {
        std::hint::black_box(ec51::hash160_fast33_8x(&comps8));
    }
    let t = start.elapsed().as_secs_f64();
    println!("hash160 8x   : {:.3}s {:.1} M/s", t, (n8 * 8) as f64 / t / 1e6);
}