mod ec;
mod ec51;
mod key;
mod table;

use std::env;
use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use ec::{Fe, Jacobian};
use table::Table;

const BATCH: usize = 1024;

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

fn flag(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

struct Cfg {
    count: u64,
    threads: usize,
    load: f64,
    do_ec: bool,
    do_hash: bool,
    do_insert: bool,
    f51: bool,
    stride: usize,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if flag(&args, "--selftest") {
        selftest();
        selftest_bigkey();
        return;
    }

    let count: u64 = match parse_arg(&args, "--count").and_then(|s| s.parse().ok()) {
        Some(v) if v > 0 => v,
        _ => {
            eprintln!("Usage: ephil-bench --count N --threads T [--no-ec] [--no-hash] [--no-insert] [--f51]");
            process::exit(1);
        }
    };
    let threads: usize = parse_arg(&args, "--threads")
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        });
    let cfg = Cfg {
        count,
        threads,
        load: parse_arg(&args, "--load")
            .and_then(|s| s.parse().ok())
            .unwrap_or(2.0),
        do_ec: !flag(&args, "--no-ec"),
        do_hash: !flag(&args, "--no-hash"),
do_insert: !flag(&args, "--no-insert"),
        f51: flag(&args, "--f51"),
        stride: parse_arg(&args, "--stride")
            .and_then(|s| s.parse().ok())
            .unwrap_or(8),
    };
if cfg.load <= 1.0 {
        eprintln!("--load must be > 1.0");
        process::exit(1);
    }

    let table = Arc::new(Table::new(count, cfg.load));
    let counter = Arc::new(AtomicU64::new(0));
    let start = Instant::now();

    let mut handles = Vec::with_capacity(cfg.threads);
    let chunk = count / cfg.threads as u64;
    let rem = count % cfg.threads as u64;
    let mut base = 1u64;

    for t in 0..cfg.threads {
        let start_n = base;
        let end_n = base + chunk + if (t as u64) < rem { 1 } else { 0 };
        base = end_n;
        let table = Arc::clone(&table);
        let counter = Arc::clone(&counter);
        let cfg = Cfg { ..cfg };
        handles.push(std::thread::spawn(move || {
            if cfg.f51 {
                worker51(start_n, end_n, &table, &counter, &cfg);
            } else {
                worker(start_n, end_n, &table, &counter, &cfg);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let elapsed = start.elapsed();
    let done = counter.load(Ordering::Relaxed);
    let rate = done as f64 / elapsed.as_secs_f64();
    println!(
        "count={} threads={} ec={} hash={} insert={} f51={} : {:.3}s {:.1} M/s",
        count,
        cfg.threads,
        cfg.do_ec,
        cfg.do_hash,
        cfg.do_insert,
        cfg.f51,
        elapsed.as_secs_f64(),
        rate / 1e6
    );
}

fn worker51(start_n: u64, end_n: u64, table: &Table, counter: &AtomicU64, cfg: &Cfg) {
    const MAX_STRIDE: usize = 16;
    let stride = cfg.stride.min(MAX_STRIDE);
    let gx = ec51::fe_from_b32_limbs(&ec::GX);
    let gy = ec51::fe_from_b32_limbs(&ec::GY);
    let step = ec51::scalar_mult(&[stride as u64, 0, 0, 0], &gx, &gy);
    let (step_x, step_y) = ec51::to_affine(&step);
    let mut chains: [ec51::Jacobian51; MAX_STRIDE] = [ec51::INF; MAX_STRIDE];
    if cfg.do_ec {
        chains[0] = ec51::scalar_mult(&[start_n, 0, 0, 0], &gx, &gy);
        for k in 1..stride {
            chains[k] = ec51::point_add(&chains[k - 1], &gx, &gy);
        }
    }
    let mut pts: Vec<ec51::Jacobian51> = Vec::with_capacity(BATCH * stride);
    let mut zs: Vec<ec51::Fe51> = Vec::with_capacity(BATCH * stride);
    let mut comps: Vec<[u8; 33]> = Vec::with_capacity(BATCH * stride);
    let mut hashes: Vec<[u8; 20]> = Vec::with_capacity(BATCH * stride);

    let mut n = start_n;
    while end_n - n >= stride as u64 {
        let groups = std::cmp::min(BATCH as u64, (end_n - n) / stride as u64) as usize;
        pts.clear();
        zs.clear();
        for _ in 0..groups {
            for k in 0..stride {
                pts.push(chains[k]);
                zs.push(chains[k].z);
                if cfg.do_ec {
                    chains[k] = ec51::point_add(&chains[k], &step_x, &step_y);
                }
            }
        }
        if cfg.do_ec {
            ec51::batch_invert(&mut zs);
        }
        comps.clear();
        hashes.clear();
        for i in 0..groups * stride {
            let nn = n + i as u64;
            let comp = if cfg.do_ec {
                ec51::to_compressed_inv(&pts[i], &zs[i])
            } else {
                let mut c = [0u8; 33];
                c[1..9].copy_from_slice(&nn.to_le_bytes());
                c[9] = 1;
                c
            };
            comps.push(comp);
        }
        let total = groups * stride;
        if cfg.do_hash {
            let mut ci = 0usize;
            while ci + 8 <= total {
                let mut comps8 = [[0u8; 33]; 8];
                comps8.copy_from_slice(&comps[ci..ci + 8]);
                let hs = ec51::hash160_fast33_8x(&comps8);
                for k in 0..8 {
                    hashes.push(hs[k]);
                }
                ci += 8;
            }
            while ci < total {
                hashes.push(ec51::hash160_fast33(&comps[ci]));
                ci += 1;
            }
        } else {
            for i in 0..total {
                let nn = n + i as u64;
                let mut h = [0u8; 20];
                h[0..8].copy_from_slice(&nn.to_le_bytes());
                hashes.push(h);
            }
        }
        for i in 0..groups * stride {
            let nn = n + i as u64;
            if cfg.do_insert {
                table.insert(nn, &hashes[i]);
            }
        }
        n += (groups * stride) as u64;
        counter.fetch_add((groups * stride) as u64, Ordering::Relaxed);
    }
    let tail = (end_n - n) as usize;
    for k in 0..tail {
        let nn = n + k as u64;
        let comp = if cfg.do_ec {
            let zi = ec51::fe_inv(&chains[k].z);
            ec51::to_compressed_inv(&chains[k], &zi)
        } else {
            let mut c = [0u8; 33];
            c[1..9].copy_from_slice(&nn.to_le_bytes());
            c[9] = 1;
            c
        };
        let h160 = if cfg.do_hash {
            ec51::hash160_fast33(&comp)
        } else {
            let mut h = [0u8; 20];
            h[0..8].copy_from_slice(&nn.to_le_bytes());
            h
        };
        if cfg.do_insert {
            table.insert(nn, &h160);
        }
    }
    counter.fetch_add(tail as u64, Ordering::Relaxed);
}

fn worker(start_n: u64, end_n: u64, table: &Table, counter: &AtomicU64, cfg: &Cfg) {
    let mut n = start_n;
    let mut p = if cfg.do_ec {
        let s = ec::scalar_from_u64(start_n);
        ec::scalar_mult(&s, &ec::GX, &ec::GY)
    } else {
        ec::INFINITY
    };
    let mut pts: Vec<Jacobian> = Vec::with_capacity(BATCH);
    let mut zs: Vec<Fe> = Vec::with_capacity(BATCH);

    while n < end_n {
        let take = std::cmp::min(BATCH as u64, end_n - n) as usize;
        pts.clear();
        zs.clear();
        if cfg.do_ec {
            let mut cur = p;
            for _ in 0..take {
                pts.push(cur);
                zs.push(cur.z);
                cur = ec::point_add(&cur, &ec::GX, &ec::GY);
            }
            p = cur;
            ec::batch_invert(&mut zs);
        }

        for i in 0..take {
            let nn = n + i as u64;
            let comp = if cfg.do_ec {
                ec::to_compressed_inv(&pts[i], &zs[i])
            } else {
                let mut c = [0u8; 33];
                c[1..9].copy_from_slice(&nn.to_le_bytes());
                c[9] = 1;
                c
            };
            let h160 = if cfg.do_hash {
                ec::hash160_from_compressed(&comp)
            } else {
                let mut h = [0u8; 20];
                h[0..8].copy_from_slice(&nn.to_le_bytes());
                h
            };
            if cfg.do_insert {
                table.insert(nn, &h160);
            }
        }
        n += take as u64;
        counter.fetch_add(take as u64, Ordering::Relaxed);
    }
}
fn selftest() {
    let gx = ec51::fe_from_b32_limbs(&ec::GX);
    let gy = ec51::fe_from_b32_limbs(&ec::GY);
    let ranges: &[(u64, u64)] = &[(1, 10), (4090, 4110), (0, 3001), (12345, 12346), (99990, 100003)];
    for &(f, t) in ranges {
const STRIDE: usize = 8;
        let step = ec51::scalar_mult(&[STRIDE as u64, 0, 0, 0], &gx, &gy);
        let (step_x, step_y) = ec51::to_affine(&step);
        let mut chains: [ec51::Jacobian51; STRIDE] = [ec51::INF; STRIDE];
        chains[0] = ec51::scalar_mult(&[f, 0, 0, 0], &gx, &gy);
        for k in 1..STRIDE {
            chains[k] = ec51::point_add(&chains[k - 1], &gx, &gy);
        }
        let mut pts: Vec<ec51::Jacobian51> = Vec::new();
        let mut zs: Vec<ec51::Fe51> = Vec::new();
        let mut comps8: [[u8; 33]; STRIDE] = [[0; 33]; STRIDE];
        let mut n = f;
        let mut mismatches = 0u64;
        let mut checked = 0u64;
        while t - n >= STRIDE as u64 {
            let groups = std::cmp::min(BATCH as u64, (t - n) / STRIDE as u64) as usize;
            pts.clear();
            zs.clear();
            for _ in 0..groups {
                for k in 0..STRIDE {
                    pts.push(chains[k]);
                    zs.push(chains[k].z);
                    chains[k] = ec51::point_add(&chains[k], &step_x, &step_y);
                }
            }
            ec51::batch_invert(&mut zs);
            for g in 0..groups {
                for k in 0..STRIDE {
                    comps8[k] = ec51::to_compressed_inv(&pts[g * STRIDE + k], &zs[g * STRIDE + k]);
                }
                let hashes = ec51::hash160_fast33_8x(&comps8);
                for k in 0..STRIDE {
                    let h = hashes[k];
                    let nn = n + (g * STRIDE + k) as u64;
                    let expect = ec51::hash160_of_n51(nn);
                    checked += 1;
                    if h != expect {
                        mismatches += 1;
                        if mismatches <= 3 {
                            println!(
                                "mismatch n={} got={:02x?} expect={:02x?}",
                                nn, &h[..4], &expect[..4]
                            );
                        }
                    }
                }
            }
            n += (groups * STRIDE) as u64;
        }
        let tail = (t - n) as usize;
        for k in 0..tail {
            let zi = ec51::fe_inv(&chains[k].z);
            let comp = ec51::to_compressed_inv(&chains[k], &zi);
            let h = ec51::hash160_fast33(&comp);
            let nn = n + k as u64;
            let expect = ec51::hash160_of_n51(nn);
            checked += 1;
            if h != expect {
                mismatches += 1;
                if mismatches <= 3 {
                    println!(
                        "mismatch n={} got={:02x?} expect={:02x?}",
                        nn, &h[..4], &expect[..4]
                    );
                }
            }
        }
        println!(
            "selftest range [{}, {}) checked={} mismatches={}",
            f, t, checked, mismatches
        );
        if mismatches != 0 {
            process::exit(1);
        }
    }
println!("selftest OK");
}

fn selftest_bigkey() {
    use key::Key;
    let gx = ec51::fe_from_b32_limbs(&ec::GX);
    let gy = ec51::fe_from_b32_limbs(&ec::GY);

    // key.rs sanity: parse/to_string round-trip at the top of the space
    let max = key::MAX_KEY; // 2^160
    assert!(
        key::parse("1461501637330902918203684832716283019655932542976").is_none(),
        "2^160 must be rejected as a key"
    );
    let last_s = key::to_string(&key::sub(&max, &key::ONE));
    assert_eq!(
        last_s,
        "1461501637330902918203684832716283019655932542975",
        "2^160 - 1 string"
    );
    let last_k = key::parse(&last_s).expect("parse 2^160-1");
    assert_eq!(last_k, key::sub(&max, &key::ONE), "2^160-1 round trip");
    assert!(key::parse("0").is_none(), "0 must be rejected");

// ec51 vs reference ec for 160-bit scalars (all keys < 2^160)
    let tests: [Key; 6] = [
        key::sub(&max, &key::ONE),
        key::sub(&max, &key::from_u64(2)),
        [0x1234_5678_9abc_def0, 0xfedc_ba98_7654_3210, 0xdead_beef_cafe_babe, 0],
        [1, 0, 0x0100_0000, 0],
        [0xffff_ffff_ffff_ffff, 0xffff_ffff_ffff_ffff, 0, 0],
        [7, 0, 0, 0],
    ];
    for &k in &tests {
        let a = ec::hash160_of_key(&k);
        let b = ec51::hash160_of_key51(&k);
        if a != b {
            println!(
                "bigkey mismatch k={} ec={:02x?} ec51={:02x?}",
                key::to_string(&k),
                &a[..4],
                &b[..4]
            );
            process::exit(1);
        }
    }
    println!("bigkey ec/ec51 cross-check OK ({} keys)", tests.len());

    // stride-walk just below the 2^160 ceiling, mirroring scan_range
    const STRIDE: usize = 8;
    let from = key::sub(&max, &key::from_u64(2000));
    let to = max;
    let step = ec51::scalar_mult(&[STRIDE as u64, 0, 0, 0], &gx, &gy);
    let (step_x, step_y) = ec51::to_affine(&step);
    let mut chains: [ec51::Jacobian51; STRIDE] = [ec51::INF; STRIDE];
    chains[0] = ec51::scalar_mult(&from, &gx, &gy);
    for k in 1..STRIDE {
        chains[k] = ec51::point_add(&chains[k - 1], &gx, &gy);
    }
    let mut n = from;
    let mut checked = 0u64;
    let mut mismatches = 0u64;
    let mut pts: Vec<ec51::Jacobian51> = Vec::new();
    let mut zs: Vec<ec51::Fe51> = Vec::new();
    let mut comps8: [[u8; 33]; STRIDE] = [[0; 33]; STRIDE];
    while key::lt(&n, &to) {
        let remaining = key::sub(&to, &n);
        if remaining[1] == 0 && remaining[2] == 0 && remaining[3] == 0 && remaining[0] < STRIDE as u64
        {
            break;
        }
        let (groups_key, _rem) = key::div_small(&remaining, STRIDE as u64);
        let groups = std::cmp::min(BATCH as u64, key::to_small(&groups_key)) as usize;
        pts.clear();
        zs.clear();
        for _ in 0..groups {
            for k in 0..STRIDE {
                pts.push(chains[k]);
                zs.push(chains[k].z);
                chains[k] = ec51::point_add(&chains[k], &step_x, &step_y);
            }
        }
        ec51::batch_invert(&mut zs);
        for g in 0..groups {
            for k in 0..STRIDE {
                comps8[k] = ec51::to_compressed_inv(&pts[g * STRIDE + k], &zs[g * STRIDE + k]);
            }
            let hashes = ec51::hash160_fast33_8x(&comps8);
            for k in 0..STRIDE {
                let nn = key::add(&n, (g * STRIDE + k) as u64);
                let expect = ec51::hash160_of_key51(&nn);
                checked += 1;
                if hashes[k] != expect {
                    mismatches += 1;
                    if mismatches <= 3 {
                        println!(
                            "mismatch n={} got={:02x?} expect={:02x?}",
                            key::to_string(&nn),
                            &hashes[k][..4],
                            &expect[..4]
                        );
                    }
                }
            }
        }
        n = key::add(&n, (groups * STRIDE) as u64);
    }
    let remaining = key::sub(&to, &n);
    let tail = key::to_small(&remaining) as usize;
    for k in 0..tail {
        let zi = ec51::fe_inv(&chains[k].z);
        let comp = ec51::to_compressed_inv(&chains[k], &zi);
        let h = ec51::hash160_fast33(&comp);
        let nn = key::add(&n, k as u64);
        let expect = ec51::hash160_of_key51(&nn);
        checked += 1;
        if h != expect {
            mismatches += 1;
            if mismatches <= 3 {
                println!(
                    "mismatch n={} got={:02x?} expect={:02x?}",
                    key::to_string(&nn),
                    &h[..4],
                    &expect[..4]
                );
            }
        }
    }
    println!("bigkey walk checked={} mismatches={}", checked, mismatches);
    if mismatches != 0 {
        process::exit(1);
    }
    println!("bigkey selftest OK");
}
