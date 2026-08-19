mod ec;
mod ec51;
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
}

fn main() {
    let args: Vec<String> = env::args().collect();
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
    let mut n = start_n;
    let gx = ec51::fe_from_b32_limbs(&ec::GX);
    let gy = ec51::fe_from_b32_limbs(&ec::GY);
    let mut p = if cfg.do_ec {
        let s = [start_n, 0, 0, 0];
        ec51::scalar_mult(&s, &gx, &gy)
    } else {
        ec51::INF
    };
    let mut pts: Vec<ec51::Jacobian51> = Vec::with_capacity(BATCH);
    let mut zs: Vec<ec51::Fe51> = Vec::with_capacity(BATCH);

    while n < end_n {
        let take = std::cmp::min(BATCH as u64, end_n - n) as usize;
        pts.clear();
        zs.clear();
        if cfg.do_ec {
            let mut cur = p;
            for _ in 0..take {
                pts.push(cur);
                zs.push(cur.z);
                cur = ec51::point_add(&cur, &gx, &gy);
            }
            p = cur;
            ec51::batch_invert(&mut zs);
        }

        for i in 0..take {
            let nn = n + i as u64;
            let comp = if cfg.do_ec {
                ec51::to_compressed_inv(&pts[i], &zs[i])
            } else {
                let mut c = [0u8; 33];
                c[1..9].copy_from_slice(&nn.to_le_bytes());
                c[9] = 1;
                c
            };
            let h160 = if cfg.do_hash {
                ec51::hash160_fast(&comp)
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