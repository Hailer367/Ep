mod ec;
mod ec51;
mod table;

use std::env;
use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

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

fn main() {
    let args: Vec<String> = env::args().collect();

    let count: u64 = match parse_arg(&args, "--count").and_then(|s| s.parse().ok()) {
        Some(v) if v > 0 => v,
        _ => {
            eprintln!("Usage: ephil-build --count N --threads T --out FILE [--load X]");
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
    let out = match parse_arg(&args, "--out") {
        Some(s) => s,
        None => {
            eprintln!("Usage: ephil-build --count N --threads T --out FILE [--load X]");
            process::exit(1);
        }
    };
    let load: f64 = parse_arg(&args, "--load")
        .and_then(|s| s.parse().ok())
        .unwrap_or(2.0);
    if load <= 1.0 {
        eprintln!("--load must be > 1.0");
        process::exit(1);
    }

    let table = Arc::new(Table::new(count, load));

    let counter = Arc::new(AtomicU64::new(0));
    let start = Instant::now();

    let mut handles = Vec::with_capacity(threads);
    let chunk = count / threads as u64;
    let rem = count % threads as u64;
    let mut base = 1u64;

    for t in 0..threads {
        let start_n = base;
        let end_n = base + chunk + if (t as u64) < rem { 1 } else { 0 };
        base = end_n;
        let table = Arc::clone(&table);
        let counter = Arc::clone(&counter);
        handles.push(std::thread::spawn(move || {
            worker(start_n, end_n, &table, &counter);
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let elapsed = start.elapsed();
    let done = counter.load(Ordering::Relaxed);
    let rate = done as f64 / elapsed.as_secs_f64();

    table.save(&out).unwrap_or_else(|e| {
        eprintln!("failed to save table: {}", e);
        process::exit(1);
    });

    let slots = table.slots();
    let mb = slots as f64 * 8.0 / (1024.0 * 1024.0);
    eprintln!(
        "built {} entries in {:.3}s ({:.1} M/s), table: {} slots, {:.1} MB in RAM",
        done,
        elapsed.as_secs_f64(),
        rate / 1e6,
        slots,
        mb
    );
    eprintln!("saved to {}", out);
}

fn worker(start_n: u64, end_n: u64, table: &Table, counter: &AtomicU64) {
    let gx = ec51::fe_from_b32_limbs(&ec::GX);
    let gy = ec51::fe_from_b32_limbs(&ec::GY);
    let mut n = start_n;
    let mut p = ec51::scalar_mult(&[start_n, 0, 0, 0], &gx, &gy);
    let mut pts: Vec<ec51::Jacobian51> = Vec::with_capacity(BATCH);
    let mut zs: Vec<ec51::Fe51> = Vec::with_capacity(BATCH);

    while n < end_n {
        let take = std::cmp::min(BATCH as u64, end_n - n) as usize;
        pts.clear();
        zs.clear();
        let mut cur = p;
        for _ in 0..take {
            pts.push(cur);
            zs.push(cur.z);
            cur = ec51::point_add(&cur, &gx, &gy);
        }
        p = cur;
        ec51::batch_invert(&mut zs);

        for i in 0..take {
            let comp = ec51::to_compressed_inv(&pts[i], &zs[i]);
            let h160 = ec51::hash160_fast(&comp);
            table.insert(n + i as u64, &h160);
        }
        n += take as u64;
        counter.fetch_add(take as u64, Ordering::Relaxed);
    }
}