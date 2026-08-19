mod ec;
mod ec51;

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::Deserialize;

const BATCH: usize = 1024;
const CHUNK: u64 = 1_000_000;

extern "C" {
    fn signal(signum: i32, handler: usize) -> usize;
}

static STOP: AtomicBool = AtomicBool::new(false);

extern "C" fn on_stop_signal(_: i32) {
    STOP.store(true, Ordering::Relaxed);
}

fn install_signal_handlers() {
    unsafe {
        signal(2, on_stop_signal as usize);
        signal(15, on_stop_signal as usize);
    }
}

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

fn esc_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

fn tg_send(token: &str, chat: &str, text: &str) -> Result<(), String> {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let resp = ureq::post(&url)
        .timeout(Duration::from_secs(15))
        .send_form(&[
            ("chat_id", chat),
            ("text", text),
            ("parse_mode", "HTML"),
            ("disable_web_page_preview", "true"),
        ])
        .map_err(|e| format!("send failed: {}", e))?;
    let status = resp.status();
    if status == 200 {
        Ok(())
    } else {
        Err(format!("telegram responded {}", status))
    }
}

fn load_targets(path: &str) -> (HashSet<[u8; 20]>, HashMap<[u8; 20], String>, usize) {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("failed to open targets file: {}", e);
            process::exit(1);
        }
    };
    let reader = BufReader::new(file);
    let mut set = HashSet::new();
    let mut map = HashMap::new();
    let mut valid = 0usize;
    let mut invalid = 0usize;
    let mut line_no = 0usize;
    for line in reader.lines() {
        line_no += 1;
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match ec::decode_bech32_address(line) {
            Some(h) => {
                if set.insert(h) {
                    map.insert(h, line.to_string());
                    valid += 1;
                }
            }
            None => {
                invalid += 1;
                eprintln!(
                    "targets file {}:{}: skipping invalid address '{}'",
                    path, line_no, line
                );
            }
        }
    }
    eprintln!("targets: {} valid address(es) loaded from {}", valid, path);
    if invalid > 0 {
        eprintln!("targets: {} invalid line(s) skipped", invalid);
    }
    (set, map, valid)
}

#[derive(Deserialize)]
struct PgRun {
    id: i64,
    name: String,
    target_addr: String,
    chunk_size: i64,
    next_start: i64,
    to_n: Option<i64>,
}

#[derive(Deserialize)]
struct PgClaim {
    chunk_id: i64,
    start_n: u64,
    end_n: u64,
    target_addr: String,
}

fn pg_call(url: &str, key: &str, fname: &str, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let url = format!("{}/rest/v1/rpc/{}", url.trim_end_matches('/'), fname);
    let resp = ureq::post(&url)
        .set("apikey", key)
        .set("Authorization", &format!("Bearer {}", key))
        .timeout(Duration::from_secs(30))
        .send_json(args)
        .map_err(|e| format!("rpc {} failed: {}", fname, e))?;
    let status = resp.status();
    let body = resp
        .into_string()
        .map_err(|e| format!("read {}: {}", fname, e))?;
    if status >= 200 && status < 300 {
        serde_json::from_str(&body).map_err(|e| format!("json {}: {}", fname, e))
    } else {
        Err(format!("rpc {} responded {}: {}", fname, status, body))
    }
}

fn pg_get_run(url: &str, key: &str, run_id: i64) -> Result<Option<PgRun>, String> {
    let v = pg_call(url, key, "ephil_get_run", &serde_json::json!({ "p_run_id": run_id }))?;
    let arr: Vec<PgRun> = serde_json::from_value(v).map_err(|e| format!("parse run: {}", e))?;
    Ok(arr.into_iter().next())
}

#[derive(Deserialize, Clone, Default)]
struct RunRow {
    id: i64,
    name: String,
    target_addr: String,
    chunk_size: i64,
    status: String,
    from_n: i64,
    to_n: Option<i64>,
    created_at: String,
    done_chunks: i64,
    done_keys: i64,
    hits: i64,
    frontier: i64,
    keys_per_sec: f64,
}

#[derive(Deserialize, Clone)]
struct HitRow {
    id: i64,
    run_id: i64,
    n: i64,
    address: String,
    worker: Option<String>,
    found_at: String,
}

#[derive(Deserialize, Clone)]
struct WorkerRow {
    id: i64,
    name: String,
    status: String,
    lease_until: Option<String>,
}

#[derive(Deserialize, Clone, Default)]
struct ControllerState {
    worker_id: Option<i64>,
    last_offset: i64,
}

#[derive(Clone, Deserialize)]
struct Claim {
    run_id: i64,
    chunk_id: i64,
    start_n: u64,
    end_n: u64,
    target_addr: String,
}

fn pg_claim(url: &str, key: &str, run_id: i64, worker: &str, lease: i32) -> Result<Option<Claim>, String> {
    let v = pg_call(
        url,
        key,
        "ephil_claim_work",
        &serde_json::json!({ "p_run_id": run_id, "p_worker": worker, "p_lease_sec": lease }),
    )?;
    let arr: Vec<PgClaim> = serde_json::from_value(v).map_err(|e| format!("parse claim: {}", e))?;
    Ok(arr.into_iter().next().map(|c| Claim {
        run_id,
        chunk_id: c.chunk_id,
        start_n: c.start_n,
        end_n: c.end_n,
        target_addr: c.target_addr,
    }))
}

fn pg_finish(url: &str, key: &str, chunk_id: i64, worker: &str) -> Result<(), String> {
    pg_call(
        url,
        key,
        "ephil_finish_work",
        &serde_json::json!({ "p_chunk_id": chunk_id, "p_worker": worker }),
    )?;
    Ok(())
}

fn pg_renew(url: &str, key: &str, chunk_id: i64, worker: &str, lease: i32) -> Result<(), String> {
    pg_call(
        url,
        key,
        "ephil_renew_lease",
        &serde_json::json!({ "p_chunk_id": chunk_id, "p_worker": worker, "p_lease_sec": lease }),
    )?;
    Ok(())
}

fn pg_report_hit(
    url: &str,
    key: &str,
    run_id: i64,
    n: u64,
    addr: &str,
    worker: &str,
) -> Result<bool, String> {
    let v = pg_call(
        url,
        key,
        "ephil_report_hit",
        &serde_json::json!({
            "p_run_id": run_id,
            "p_n": n,
            "p_address": addr,
            "p_worker": worker
        }),
    )?;
    serde_json::from_value(v).map_err(|e| format!("parse report_hit: {}", e))
}

fn pg_register_worker(url: &str, key: &str, name: &str, lease: i32) -> Result<i64, String> {
    let v = pg_call(
        url,
        key,
        "ephil_register_worker",
        &serde_json::json!({ "p_name": name, "p_lease_sec": lease }),
    )?;
    serde_json::from_value(v).map_err(|e| format!("parse register_worker: {}", e))
}

fn pg_worker_heartbeat(url: &str, key: &str, wid: i64, lease: i32) -> Result<(), String> {
    pg_call(
        url,
        key,
        "ephil_worker_heartbeat",
        &serde_json::json!({ "p_worker_id": wid, "p_lease_sec": lease }),
    )?;
    Ok(())
}

fn pg_release_worker(url: &str, key: &str, wid: i64) -> Result<(), String> {
    pg_call(
        url,
        key,
        "ephil_release_worker",
        &serde_json::json!({ "p_worker_id": wid }),
    )?;
    Ok(())
}

fn pg_abandon_work(url: &str, key: &str, chunk_id: i64, worker: &str) -> Result<(), String> {
    pg_call(
        url,
        key,
        "ephil_abandon_work",
        &serde_json::json!({ "p_chunk_id": chunk_id, "p_worker": worker }),
    )?;
    Ok(())
}

fn pg_start_run(
    url: &str,
    key: &str,
    name: &str,
    target: &str,
    chunk: i64,
    from: i64,
    to: Option<i64>,
) -> Result<i64, String> {
    let v = pg_call(
        url,
        key,
        "ephil_start_run",
        &serde_json::json!({
            "p_name": name,
            "p_target": target,
            "p_chunk_size": chunk,
            "p_from": from,
            "p_to": to
        }),
    )?;
    serde_json::from_value(v).map_err(|e| format!("parse start_run: {}", e))
}

fn pg_set_run_status(url: &str, key: &str, run: i64, status: &str) -> Result<bool, String> {
    let v = pg_call(
        url,
        key,
        "ephil_set_run_status",
        &serde_json::json!({ "p_run_id": run, "p_status": status }),
    )?;
    serde_json::from_value(v).map_err(|e| format!("parse set_run_status: {}", e))
}

fn pg_run_list(url: &str, key: &str) -> Result<Vec<RunRow>, String> {
    let v = pg_call(url, key, "ephil_run_list", &serde_json::json!({}))?;
    serde_json::from_value(v).map_err(|e| format!("parse run_list: {}", e))
}

fn pg_hits(url: &str, key: &str, run: Option<i64>) -> Result<Vec<HitRow>, String> {
    let v = pg_call(
        url,
        key,
        "ephil_hits",
        &serde_json::json!({ "p_run_id": run }),
    )?;
    serde_json::from_value(v).map_err(|e| format!("parse hits: {}", e))
}

fn pg_worker_list(url: &str, key: &str) -> Result<Vec<WorkerRow>, String> {
    let v = pg_call(url, key, "ephil_worker_list", &serde_json::json!({}))?;
    serde_json::from_value(v).map_err(|e| format!("parse worker_list: {}", e))
}

fn pg_claim_any(url: &str, key: &str, worker: &str, lease: i32) -> Result<Option<Claim>, String> {
    let v = pg_call(
        url,
        key,
        "ephil_claim_any",
        &serde_json::json!({ "p_worker": worker, "p_lease_sec": lease }),
    )?;
    let rows: Vec<Claim> = serde_json::from_value(v).map_err(|e| format!("parse claim_any: {}", e))?;
    Ok(rows.into_iter().next())
}

fn pg_acquire_controller(url: &str, key: &str, wid: i64, lease: i32) -> Result<bool, String> {
    let v = pg_call(
        url,
        key,
        "ephil_acquire_controller",
        &serde_json::json!({ "p_worker_id": wid, "p_lease_sec": lease }),
    )?;
    serde_json::from_value(v).map_err(|e| format!("parse acquire_controller: {}", e))
}

fn pg_release_controller(url: &str, key: &str, wid: i64) -> Result<(), String> {
    pg_call(
        url,
        key,
        "ephil_release_controller",
        &serde_json::json!({ "p_worker_id": wid }),
    )?;
    Ok(())
}

fn pg_get_controller(url: &str, key: &str) -> Result<ControllerState, String> {
    let v = pg_call(url, key, "ephil_get_controller", &serde_json::json!({}))?;
    let rows: Vec<ControllerState> =
        serde_json::from_value(v).map_err(|e| format!("parse get_controller: {}", e))?;
    Ok(rows.into_iter().next().unwrap_or_default())
}

fn pg_set_offset(url: &str, key: &str, off: i64) -> Result<(), String> {
    let _ = pg_call(url, key, "ephil_set_offset", &serde_json::json!({ "p_offset": off }))?;
    Ok(())
}

fn tg_post(token: &str, method: &str, body: &serde_json::Value) -> Result<serde_json::Value, String> {
    let url = format!("https://api.telegram.org/bot{}/{}", token, method);
    let resp = ureq::post(&url)
        .timeout(Duration::from_secs(35))
        .send_json(body)
        .map_err(|e| format!("telegram {}: {}", method, e))?;
    let status = resp.status();
    let text = resp
        .into_string()
        .map_err(|e| format!("telegram {} read: {}", method, e))?;
    if status >= 200 && status < 300 {
        serde_json::from_str(&text).map_err(|e| format!("telegram json: {}", e))
    } else {
        Err(format!("telegram {} responded {}: {}", method, status, text))
    }
}

fn tg_get_me(token: &str) -> Result<i64, String> {
    let v = tg_post(token, "getMe", &serde_json::json!({}))?;
    Ok(v["result"]["id"].as_i64().unwrap_or(0))
}

fn tg_get_updates(token: &str, offset: i64) -> Result<Vec<(i64, i64, i64, String)>, String> {
    let v = tg_post(
        token,
        "getUpdates",
        &serde_json::json!({ "offset": offset, "timeout": 5, "allowed_updates": ["message"] }),
    )?;
    let mut out = Vec::new();
    if let Some(arr) = v["result"].as_array() {
        for u in arr {
            let uid = u["update_id"].as_i64().unwrap_or(0);
            let msg = &u["message"];
            if msg.is_null() {
                continue;
            }
            let from_id = msg["from"]["id"].as_i64().unwrap_or(0);
            let chat_id = msg["chat"]["id"].as_i64().unwrap_or(0);
            let text = msg["text"].as_str().unwrap_or("").to_string();
            if !text.is_empty() {
                out.push((uid, from_id, chat_id, text));
            }
        }
    }
    Ok(out)
}

fn scan_range(
    gx: &ec51::Fe51,
    gy: &ec51::Fe51,
    from: u64,
    to: u64,
    file_targets: &HashSet<[u8; 20]>,
    file_addr: &HashMap<[u8; 20], String>,
    run_targets: &[([u8; 20], String)],
    scanned: &AtomicU64,
    on_hit: &mut dyn FnMut(u64, &str),
) {
    let mut p = ec51::scalar_mult(&[from, 0, 0, 0], gx, gy);
    let mut n = from;
    let mut pts: Vec<ec51::Jacobian51> = Vec::with_capacity(BATCH);
    let mut zs: Vec<ec51::Fe51> = Vec::with_capacity(BATCH);
    while n < to {
        let take = std::cmp::min(BATCH as u64, to - n) as usize;
        pts.clear();
        zs.clear();
        let mut cur = p;
        for _ in 0..take {
            pts.push(cur);
            zs.push(cur.z);
            cur = ec51::point_add(&cur, gx, gy);
        }
        p = cur;
        ec51::batch_invert(&mut zs);

        for i in 0..take {
            let comp = ec51::to_compressed_inv(&pts[i], &zs[i]);
            let h160 = ec51::hash160_fast(&comp);
            if file_targets.contains(&h160) || run_targets.iter().any(|(h, _)| *h == h160) {
                let found = n + i as u64;
                let addr = file_addr
                    .get(&h160)
                    .cloned()
                    .or_else(|| {
                        run_targets
                            .iter()
                            .find(|(h, _)| *h == h160)
                            .map(|(_, a)| a.clone())
                    })
                    .unwrap_or_else(|| "unknown".to_string());
                on_hit(found, &addr);
            }
        }
        n += take as u64;
        scanned.fetch_add(take as u64, Ordering::Relaxed);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let agent = args.iter().any(|a| a == "--agent");
    let coordinator = parse_arg(&args, "--coordinator")
        .or_else(|| env::var("SUPABASE_URL").ok())
        .unwrap_or_default();
    let key = parse_arg(&args, "--key")
        .or_else(|| env::var("SUPABASE_KEY").ok())
        .unwrap_or_default();
    let worker_id = parse_arg(&args, "--worker-id")
        .or_else(|| env::var("EPHIL_WORKER").ok())
        .unwrap_or_else(|| "agent".to_string());
    let lease_sec: i32 = parse_arg(&args, "--lease-sec")
        .and_then(|s| s.parse().ok())
        .unwrap_or(900);
    let threads: usize = parse_arg(&args, "--threads")
        .and_then(|s| s.parse().ok())
        .or_else(|| env::var("EPHIL_THREADS").ok().and_then(|s| s.parse().ok()))
        .unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1));

    if agent {
        if coordinator.is_empty() || key.is_empty() {
            eprintln!("agent mode needs --coordinator URL and --key (or SUPABASE_URL / SUPABASE_KEY)");
            process::exit(1);
        }
        let token = match env::var("EPHIL_TG_TOKEN") {
            Ok(s) if !s.is_empty() => s,
            _ => {
                eprintln!("EPHIL_TG_TOKEN not set");
                process::exit(1);
            }
        };
        let chat = match env::var("EPHIL_TG_CHAT") {
            Ok(s) if !s.is_empty() => s,
            _ => {
                eprintln!("EPHIL_TG_CHAT not set");
                process::exit(1);
            }
        };
        let pinned = parse_arg(&args, "--run-id").and_then(|r| r.parse::<i64>().ok());
        let targets_file = parse_arg(&args, "--targets");
        run_agent(&coordinator, &key, &worker_id, lease_sec, &token, &chat, pinned, threads, targets_file.as_deref());
        return;
    }

    if let Some(rid) = parse_arg(&args, "--run-id") {
        let run_id: i64 = match rid.parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("invalid --run-id: {}", rid);
                process::exit(1);
            }
        };
        if coordinator.is_empty() || key.is_empty() {
            eprintln!("distributed mode needs --coordinator URL and --key (or SUPABASE_URL / SUPABASE_KEY)");
            process::exit(1);
        }
        let token = match env::var("EPHIL_TG_TOKEN") {
            Ok(s) if !s.is_empty() => s,
            _ => {
                eprintln!("EPHIL_TG_TOKEN not set");
                process::exit(1);
            }
        };
        let chat = match env::var("EPHIL_TG_CHAT") {
            Ok(s) if !s.is_empty() => s,
            _ => {
                eprintln!("EPHIL_TG_CHAT not set");
                process::exit(1);
            }
        };
        run_distributed(&coordinator, &key, run_id, &worker_id, lease_sec, &token, &chat, threads, parse_arg(&args, "--targets").as_deref());
        return;
    }

    let single = parse_arg(&args, "--target");
    let file = parse_arg(&args, "--file");
    if single.is_none() && file.is_none() {
        eprintln!("Usage: ephil-scan [--target ADDR | --file LIST] [--from N] [--to N] [--threads T]");
        eprintln!("Distributed: ephil-scan --run-id N [--worker-id NAME] [--coordinator URL] [--key KEY] [--lease-sec S] [--threads T]");
        eprintln!("Agent: ephil-scan --agent [--run-id N] [--worker-id NAME] [--coordinator URL] [--key KEY] [--threads T] [--targets FILE]");
        eprintln!("  (worker id is assigned by the coordinator; NAME is just a label; threads default to all cores)");
        eprintln!("Telegram config via env: EPHIL_TG_TOKEN, EPHIL_TG_CHAT");
        process::exit(1);
    }
    let (mut targets, mut addr_map, mut valid) = match &file {
        Some(p) => load_targets(p),
        None => (HashSet::new(), HashMap::new(), 0usize),
    };
    if let Some(a) = &single {
        match ec::decode_bech32_address(a) {
            Some(h) => {
                targets.insert(h);
                addr_map.insert(h, a.clone());
                valid += 1;
            }
            None => {
                eprintln!("invalid bech32 address: {}", a);
                process::exit(1);
            }
        }
    }
    if valid == 0 {
        eprintln!("no valid target addresses to watch");
        process::exit(1);
    }
    let targets = Arc::new(targets);
    let addr_map = Arc::new(addr_map);

    let from: u64 = parse_arg(&args, "--from")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let to: Option<u64> = parse_arg(&args, "--to").and_then(|s| s.parse().ok());
    if let Some(t) = to {
        if t < from {
            eprintln!("--to must be >= --from (or omit for infinite watch)");
            process::exit(1);
        }
    }
    let threads: usize = parse_arg(&args, "--threads")
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        });
    let token = match env::var("EPHIL_TG_TOKEN") {
        Ok(s) if !s.is_empty() => s,
        _ => {
            eprintln!("EPHIL_TG_TOKEN not set");
            process::exit(1);
        }
    };
    let chat = match env::var("EPHIL_TG_CHAT") {
        Ok(s) if !s.is_empty() => s,
        _ => {
            eprintln!("EPHIL_TG_CHAT not set");
            process::exit(1);
        }
    };

    match to {
        Some(t) => eprintln!(
            "watching [{}..{}] for {} targets, {} threads (Telegram on hit only)",
            from, t, valid, threads
        ),
        None => eprintln!(
            "watching {}..forever for {} targets, {} threads (Telegram on hit only)",
            from, valid, threads
        ),
    }

    let scanned = Arc::new(AtomicU64::new(0));
    let head = Arc::new(AtomicU64::new(from));
    let start = Instant::now();

    let mut handles = Vec::with_capacity(threads);
    for t in 0..threads {
        let start_scalar = from + (t as u64) * CHUNK;
        let to = to;
        let stride = threads as u64;
        let scanned = Arc::clone(&scanned);
        let head = Arc::clone(&head);
        let targets = Arc::clone(&targets);
        let addr_map = Arc::clone(&addr_map);
        let token = token.clone();
        let chat = chat.clone();
        handles.push(std::thread::spawn(move || {
            worker(
                start_scalar,
                to,
                stride,
                &targets,
                &addr_map,
                &scanned,
                &head,
                &token,
                &chat,
            );
        }));
    }

    loop {
        std::thread::sleep(Duration::from_secs(5));
        let s = scanned.load(Ordering::Relaxed);
        let h = head.load(Ordering::Relaxed);
        let elapsed = start.elapsed().as_secs_f64();
        let rate = s as f64 / elapsed / 1e6;
        eprintln!(
            "scanned {:.1} M keys, head ~ {}, {:.1} M/s",
            s as f64 / 1e6,
            h,
            rate
        );
        if to.is_some() && handles.iter().all(|h| h.is_finished()) {
            break;
        }
    }
    for h in handles {
        let _ = h.join();
    }
}

fn worker(
    mut start_scalar: u64,
    to: Option<u64>,
    stride: u64,
    targets: &HashSet<[u8; 20]>,
    addr_map: &HashMap<[u8; 20], String>,
    scanned: &AtomicU64,
    head: &AtomicU64,
    token: &str,
    chat: &str,
) {
    let gx = ec51::fe_from_b32_limbs(&ec::GX);
    let gy = ec51::fe_from_b32_limbs(&ec::GY);

    loop {
        if let Some(ub) = to {
            if start_scalar > ub {
                return;
            }
        }
        let mut run_len = CHUNK;
        if let Some(ub) = to {
            run_len = std::cmp::min(run_len, ub - start_scalar + 1);
        }
        let end = start_scalar + run_len;
        let mut on_hit = |found: u64, addr: &str| {
            println!("HIT n = {} address = {}", found, addr);
            let msg = format!("Ephil scan hit\nn = {}\ntarget: {}", found, addr);
            match tg_send(token, chat, &msg) {
                Ok(()) => println!("telegram delivered"),
                Err(e) => eprintln!("telegram: {}", e),
            }
        };
        scan_range(&gx, &gy, start_scalar, end, targets, addr_map, &[], scanned, &mut on_hit);
        head.fetch_max(end, Ordering::Relaxed);

        match start_scalar.checked_add(CHUNK.checked_mul(stride).unwrap()) {
            Some(v) => start_scalar = v,
            None => {
                start_scalar = 1;
            }
        }
    }
}

const HELP: &str = "<b>Ephil bot</b> - distributed Bitcoin sequence scanner

<b>Commands</b>
/scan <i>&lt;address&gt;</i> [chunk] [from] [to] - start a scan run
/scan_txt [chunk] [from] [to] - scan against the downloaded txt targets
/stop <i>&lt;run_id&gt;</i> - stop a run
/resume <i>&lt;run_id&gt;</i> - resume a stopped run
/status - runs + workers summary
/workers - active workers
/hits [run_id] - recent hits
/shutdown - stop all active runs
/help - this message";

fn fmt_count(v: u64) -> String {
    const K: u64 = 1_000;
    const M: u64 = 1_000_000;
    const G: u64 = 1_000_000_000;
    const T: u64 = 1_000_000_000_000;
    if v >= T {
        format!("{:.2}T", v as f64 / T as f64)
    } else if v >= G {
        format!("{:.2}G", v as f64 / G as f64)
    } else if v >= M {
        format!("{:.2}M", v as f64 / M as f64)
    } else if v >= K {
        format!("{:.1}K", v as f64 / K as f64)
    } else {
        v.to_string()
    }
}

struct WorkerCtx {
    coordinator: String,
    key: String,
    token: String,
    chat: String,
    file_targets: HashSet<[u8; 20]>,
    file_addr: HashMap<[u8; 20], String>,
}

impl WorkerCtx {
    fn scan_chunk(
        &self,
        run_id: i64,
        cid: i64,
        cs: u64,
        ce: u64,
        target_addr: &str,
        total: &AtomicU64,
        lease_sec: i32,
        worker: &str,
    ) {
        let run_targets: Vec<([u8; 20], String)> = if target_addr.is_empty() {
            Vec::new()
        } else {
            match ec::decode_bech32_address(target_addr) {
                Some(h) => vec![(h, target_addr.to_string())],
                None => {
                    eprintln!("invalid target from coordinator: {}", target_addr);
                    return;
                }
            }
        };
        if self.file_targets.is_empty() && run_targets.is_empty() {
            eprintln!(
                "worker {}: run {} has no targets (no run address and no file targets)",
                worker, run_id
            );
            let _ = pg_abandon_work(&self.coordinator, &self.key, cid, worker);
            return;
        }

        let stop = Arc::new(AtomicBool::new(false));
        let s2 = Arc::clone(&stop);
        let url3 = self.coordinator.clone();
        let key3 = self.key.clone();
        let w3 = worker.to_string();
        let renewer = std::thread::spawn(move || {
            let interval = std::cmp::max(10u64, (lease_sec as u64) / 4);
            let mut last = Instant::now();
            loop {
                std::thread::sleep(Duration::from_secs(2));
                if s2.load(Ordering::Relaxed) {
                    break;
                }
                if last.elapsed().as_secs() >= interval {
                    let _ = pg_renew(&url3, &key3, cid, &w3, lease_sec);
                    last = Instant::now();
                }
            }
        });

        let gx = ec51::fe_from_b32_limbs(&ec::GX);
        let gy = ec51::fe_from_b32_limbs(&ec::GY);
        let mut on_hit = |found: u64, addr: &str| {
            match pg_report_hit(&self.coordinator, &self.key, run_id, found, addr, worker) {
                Ok(true) => {
                    println!("HIT n = {} address = {}", found, addr);
                    let msg = format!(
                        "<b>Ephil scan hit</b>\nn = <code>{}</code>\ntarget: <code>{}</code>",
                        found,
                        esc_html(addr)
                    );
                    match tg_send(&self.token, &self.chat, &msg) {
                        Ok(()) => println!("telegram delivered"),
                        Err(e) => eprintln!("telegram: {}", e),
                    }
                }
                Ok(false) => println!(
                    "HIT n = {} address = {} (already reported, no re-ping)",
                    found, addr
                ),
                Err(e) => {
                    println!("HIT n = {} address = {}", found, addr);
                    eprintln!("hit report failed ({}), pinging telegram anyway", e);
                    let msg = format!(
                        "<b>Ephil scan hit</b>\nn = <code>{}</code>\ntarget: <code>{}</code>",
                        found,
                        esc_html(addr)
                    );
                    match tg_send(&self.token, &self.chat, &msg) {
                        Ok(()) => println!("telegram delivered"),
                        Err(e2) => eprintln!("telegram: {}", e2),
                    }
                }
            }
        };

        scan_range(
            &gx,
            &gy,
            cs,
            ce,
            &self.file_targets,
            &self.file_addr,
            &run_targets,
            total,
            &mut on_hit,
        );

        stop.store(true, Ordering::Relaxed);
        let _ = renewer.join();
        if STOP.load(Ordering::Relaxed) {
            let _ = pg_abandon_work(&self.coordinator, &self.key, cid, worker);
            eprintln!(
                "worker {}: interrupted, chunk {} [{}..{}) reopened",
                worker, cid, cs, ce
            );
        } else {
            match pg_finish(&self.coordinator, &self.key, cid, worker) {
                Ok(()) => {}
                Err(e) => eprintln!("finish: {}", e),
            }
            eprintln!("worker {}: finished chunk {} [{}..{})", worker, cid, cs, ce);
        }
    }
}

fn worker_loop(
    ctx: &WorkerCtx,
    worker: &str,
    lease_sec: i32,
    total: &AtomicU64,
    claim: &(dyn Fn(&str, i32) -> Result<Option<Claim>, String> + Sync),
) {
    loop {
        if STOP.load(Ordering::Relaxed) {
            break;
        }
        let c = match claim(worker, lease_sec) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("claim: {} (retrying)", e);
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }
        };
        match c {
            None => {
                for _ in 0..2 {
                    std::thread::sleep(Duration::from_secs(1));
                    if STOP.load(Ordering::Relaxed) {
                        break;
                    }
                }
            }
            Some(cl) => {
                eprintln!(
                    "worker {}: claimed chunk {} [{}..{}) run {}",
                    worker, cl.chunk_id, cl.start_n, cl.end_n, cl.run_id
                );
                ctx.scan_chunk(
                    cl.run_id, cl.chunk_id, cl.start_n, cl.end_n, &cl.target_addr, total,
                    lease_sec, worker,
                );
            }
        }
    }
}

fn register_and_join(coordinator: &str, key: &str, worker_name: &str, lease_sec: i32) -> i64 {
    let worker_id = match pg_register_worker(coordinator, key, worker_name, lease_sec) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("register worker: {}", e);
            process::exit(1);
        }
    };
    eprintln!(
        "worker '{}' assigned id {} (lease {}s)",
        worker_name, worker_id, lease_sec
    );
    worker_id
}

fn run_distributed(
    coordinator: &str,
    key: &str,
    run_id: i64,
    worker_name: &str,
    lease_sec: i32,
    token: &str,
    chat: &str,
    threads: usize,
    targets_file: Option<&str>,
) {
    install_signal_handlers();
    let worker_id = register_and_join(coordinator, key, worker_name, lease_sec);
    let worker = worker_id.to_string();
    let run = match pg_get_run(coordinator, key, run_id) {
        Ok(Some(r)) => r,
        Ok(None) => {
            eprintln!("run {} not found on coordinator", run_id);
            process::exit(1);
        }
        Err(e) => {
            eprintln!("coordinator: {}", e);
            process::exit(1);
        }
    };
    eprintln!(
        "run {} '{}' target {} chunk={}",
        run.id, run.name, run.target_addr, run.chunk_size
    );

    let (ft, fa) = match targets_file {
        Some(p) => {
            let (s, m, _) = load_targets(p);
            (s, m)
        }
        None => (HashSet::new(), HashMap::new()),
    };
    let ctx = WorkerCtx {
        coordinator: coordinator.to_string(),
        key: key.to_string(),
        token: token.to_string(),
        chat: chat.to_string(),
        file_targets: ft,
        file_addr: fa,
    };
    let total = Arc::new(AtomicU64::new(0));
    let t2 = Arc::clone(&total);
    let w2 = worker.clone();
    let progress = std::thread::spawn(move || {
        let start = Instant::now();
        loop {
            std::thread::sleep(Duration::from_secs(10));
            let s = t2.load(Ordering::Relaxed);
            let rate = s as f64 / start.elapsed().as_secs_f64() / 1e6;
            eprintln!("worker {}: {:.1} M keys, {:.1} M/s", w2, s as f64 / 1e6, rate);
        }
    });

    let hb_interval = std::cmp::max(10u64, (lease_sec as u64) / 3);
    let url2 = coordinator.to_string();
    let key2 = key.to_string();
    let hb = std::thread::spawn(move || {
        loop {
            for _ in 0..hb_interval {
                std::thread::sleep(Duration::from_secs(1));
                if STOP.load(Ordering::Relaxed) {
                    return;
                }
            }
            let _ = pg_worker_heartbeat(&url2, &key2, worker_id, lease_sec);
        }
    });

    let ccoord = coordinator.to_string();
    let ckey = key.to_string();
    let claim = move |w: &str, l: i32| pg_claim(&ccoord, &ckey, run_id, w, l);
    eprintln!("worker {}: scanning with {} thread(s)", worker_id, threads);
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| worker_loop(&ctx, &worker, lease_sec, &total, &claim));
        }
    });

    let _ = pg_release_worker(coordinator, key, worker_id);
    let _ = hb.join();
    eprintln!("worker {}: released worker slot, bye", worker_id);
}

fn run_agent(
    coordinator: &str,
    key: &str,
    worker_name: &str,
    lease_sec: i32,
    token: &str,
    chat: &str,
    pinned_run: Option<i64>,
    threads: usize,
    targets_file: Option<&str>,
) {
    install_signal_handlers();
    let worker_id = register_and_join(coordinator, key, worker_name, lease_sec);
    let worker = worker_id.to_string();

    let c1 = coordinator.to_string();
    let k1 = key.to_string();
    let t = token.to_string();
    let n1 = worker_name.to_string();
    let bot = std::thread::spawn(move || {
        bot_controller(&c1, &k1, worker_id, &t, &n1);
    });

    let (ft, fa) = match targets_file {
        Some(p) => {
            let (s, m, _) = load_targets(p);
            (s, m)
        }
        None => (HashSet::new(), HashMap::new()),
    };
    let ctx = WorkerCtx {
        coordinator: coordinator.to_string(),
        key: key.to_string(),
        token: token.to_string(),
        chat: chat.to_string(),
        file_targets: ft,
        file_addr: fa,
    };
    let total = Arc::new(AtomicU64::new(0));
    let t2 = Arc::clone(&total);
    let w2 = worker.clone();
    let progress = std::thread::spawn(move || {
        let start = Instant::now();
        loop {
            std::thread::sleep(Duration::from_secs(10));
            let s = t2.load(Ordering::Relaxed);
            let rate = s as f64 / start.elapsed().as_secs_f64() / 1e6;
            eprintln!("worker {}: {:.1} M keys, {:.1} M/s", w2, s as f64 / 1e6, rate);
        }
    });

    let hb_interval = std::cmp::max(10u64, (lease_sec as u64) / 3);
    let url2 = coordinator.to_string();
    let key2 = key.to_string();
    let hb = std::thread::spawn(move || {
        loop {
            for _ in 0..hb_interval {
                std::thread::sleep(Duration::from_secs(1));
                if STOP.load(Ordering::Relaxed) {
                    return;
                }
            }
            let _ = pg_worker_heartbeat(&url2, &key2, worker_id, lease_sec);
        }
    });

    let ccoord = coordinator.to_string();
    let ckey = key.to_string();
    let claim: Box<dyn Fn(&str, i32) -> Result<Option<Claim>, String> + Sync> =
        if let Some(run_id) = pinned_run {
            Box::new(move |w: &str, l: i32| pg_claim(&ccoord, &ckey, run_id, w, l))
        } else {
            Box::new(move |w: &str, l: i32| pg_claim_any(&ccoord, &ckey, w, l))
        };
    eprintln!("worker {}: scanning with {} thread(s)", worker_id, threads);
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| worker_loop(&ctx, &worker, lease_sec, &total, claim.as_ref()));
        }
    });

    let _ = pg_release_worker(coordinator, key, worker_id);
    let _ = hb.join();
    let _ = bot.join();
    eprintln!("worker {}: released worker slot, bye", worker_id);
}

fn bot_controller(coordinator: &str, key: &str, worker_id: i64, token: &str, name: &str) {
    let bot_id = tg_get_me(token).unwrap_or(0);
    eprintln!("bot controller '{}' up (bot id {})", name, bot_id);
    loop {
        if STOP.load(Ordering::Relaxed) {
            let _ = pg_release_controller(coordinator, key, worker_id);
            return;
        }
        match pg_acquire_controller(coordinator, key, worker_id, 30) {
            Ok(true) => {}
            _ => {
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }
        }
        let mut last_renew = Instant::now();
        while !STOP.load(Ordering::Relaxed) {
            if last_renew.elapsed().as_secs() >= 15 {
                if !pg_acquire_controller(coordinator, key, worker_id, 30).unwrap_or(false) {
                    break;
                }
                last_renew = Instant::now();
            }
            let offset = pg_get_controller(coordinator, key)
                .map(|c| c.last_offset)
                .unwrap_or(0);
            let updates = match tg_get_updates(token, offset) {
                Ok(u) => u,
                Err(e) => {
                    eprintln!("getUpdates: {}", e);
                    std::thread::sleep(Duration::from_secs(2));
                    continue;
                }
            };
            let mut next = offset;
            for (uid, from, chat_id, text) in updates {
                if uid >= next {
                    next = uid + 1;
                }
                if from == bot_id {
                    continue;
                }
                if text.starts_with('/') {
                    eprintln!("cmd from {}: {}", chat_id, text);
                    let reply = handle_command(&text, coordinator, key);
                    if !reply.is_empty() {
                        let _ = tg_send(token, &chat_id.to_string(), &reply);
                    }
                }
            }
            if next != offset {
                let _ = pg_set_offset(coordinator, key, next);
            }
        }
    }
}

fn handle_command(text: &str, coordinator: &str, key: &str) -> String {
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.is_empty() {
        return String::new();
    }
    let cmd = parts[0].to_lowercase();
    match cmd.as_str() {
        "/start" | "/help" => HELP.to_string(),
        "/ping" => "pong".to_string(),
        "/scan" => {
            if parts.len() < 2 {
                return "usage: /scan <i>&lt;address&gt;</i> [chunk] [from] [to]".to_string();
            }
            let addr = parts[1];
            if ec::decode_bech32_address(addr).is_none() {
                return format!("invalid address: {}", esc_html(addr));
            }
            let chunk: i64 = parts
                .get(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(10_000_000);
            if chunk <= 0 {
                return "chunk must be > 0".to_string();
            }
            let from: i64 = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);
            let to: Option<i64> = parts.get(4).and_then(|s| s.parse().ok());
            let name = if addr.len() > 12 {
                format!("{}...", &addr[..12])
            } else {
                addr.to_string()
            };
            match pg_start_run(coordinator, key, &name, addr, chunk, from, to) {
                Ok(rid) => format!(
                    "<b>Run {} started</b>\ntarget: <code>{}</code>\nfrom: {} | chunk: {}{}\nAll agents will pick it up automatically.",
                    rid,
                    esc_html(addr),
                    from,
                    chunk,
                    to.map(|t| format!(" | to: {}", t)).unwrap_or_default()
                ),
                Err(e) => format!("failed to start run: {}", esc_html(&e)),
            }
        }
        "/scan_txt" => {
            let chunk: i64 = parts
                .get(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(10_000_000);
            if chunk <= 0 {
                return "chunk must be > 0".to_string();
            }
            let from: i64 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
            let to: Option<i64> = parts.get(3).and_then(|s| s.parse().ok());
            match pg_start_run(coordinator, key, "txt", "", chunk, from, to) {
                Ok(rid) => format!(
                    "<b>Run {} started (txt file targets)</b>\nfrom: {} | chunk: {}{}\nAgents will match generated keys against the address file they downloaded.",
                    rid,
                    from,
                    chunk,
                    to.map(|t| format!(" | to: {}", t)).unwrap_or_default()
                ),
                Err(e) => format!("failed to start run: {}", esc_html(&e)),
            }
        }
        "/stop" => {
            let run = match parts.get(1).and_then(|s| s.parse::<i64>().ok()) {
                Some(r) => r,
                None => return "usage: /stop <i>&lt;run_id&gt;</i>".to_string(),
            };
            match pg_set_run_status(coordinator, key, run, "stopped") {
                Ok(true) => format!("<b>Run {} stopped</b>", run),
                Ok(false) => format!("run {} not found", run),
                Err(e) => format!("failed to stop run: {}", esc_html(&e)),
            }
        }
        "/resume" => {
            let run = match parts.get(1).and_then(|s| s.parse::<i64>().ok()) {
                Some(r) => r,
                None => return "usage: /resume <i>&lt;run_id&gt;</i>".to_string(),
            };
            match pg_set_run_status(coordinator, key, run, "active") {
                Ok(true) => format!("<b>Run {} resumed</b>", run),
                Ok(false) => format!("run {} not found", run),
                Err(e) => format!("failed to resume run: {}", esc_html(&e)),
            }
        }
        "/shutdown" => {
            let runs = match pg_run_list(coordinator, key) {
                Ok(r) => r,
                Err(e) => return format!("failed: {}", esc_html(&e)),
            };
            let mut n = 0;
            for r in runs {
                if r.status == "active" {
                    let _ = pg_set_run_status(coordinator, key, r.id, "stopped");
                    n += 1;
                }
            }
            format!("<b>Stopped {} active run(s)</b>", n)
        }
        "/status" => status_text(coordinator, key),
        "/workers" => workers_text(coordinator, key),
        "/hits" => {
            let run = parts.get(1).and_then(|s| s.parse::<i64>().ok());
            hits_text(coordinator, key, run)
        }
        _ => HELP.to_string(),
    }
}

fn status_text(coordinator: &str, key: &str) -> String {
    let mut out = String::from("<b>Ephil status</b>\n\n<b>Runs</b>\n");
    match pg_run_list(coordinator, key) {
        Ok(runs) => {
            if runs.is_empty() {
                out.push_str("  none\n");
            }
            for r in runs {
                let rate = if r.keys_per_sec > 0.0 {
                    format!(" | rate: {}keys/s", fmt_count(r.keys_per_sec as u64))
                } else {
                    String::new()
                };
                out.push_str(&format!(
                    "#{} <i>{}</i> [{}] target <code>{}</code> chunk {}\n",
                    r.id,
                    esc_html(&r.name),
                    r.status,
                    if r.target_addr.is_empty() {
                        "(txt file)".to_string()
                    } else {
                        esc_html(&r.target_addr)
                    },
                    fmt_count(r.chunk_size as u64)
                ));
                out.push_str(&format!(
                    "  done {} keys ({} chunks) | hits {} | frontier {}{}\n",
                    fmt_count(r.done_keys as u64),
                    r.done_chunks,
                    r.hits,
                    fmt_count(r.frontier as u64),
                    rate
                ));
            }
        }
        Err(e) => out.push_str(&format!("  run_list failed: {}\n", esc_html(&e))),
    }
    out.push_str("\n<b>Workers</b>\n");
    match pg_worker_list(coordinator, key) {
        Ok(ws) => {
            if ws.is_empty() {
                out.push_str("  none\n");
            }
            for w in ws {
                out.push_str(&format!(
                    "#{} <code>{}</code> [{}]\n",
                    w.id,
                    esc_html(&w.name),
                    w.status
                ));
            }
        }
        Err(e) => out.push_str(&format!("  worker_list failed: {}\n", esc_html(&e))),
    }
    out
}

fn workers_text(coordinator: &str, key: &str) -> String {
    let mut out = String::from("<b>Workers</b>\n");
    match pg_worker_list(coordinator, key) {
        Ok(ws) => {
            if ws.is_empty() {
                out.push_str("  none\n");
            }
            for w in ws {
                out.push_str(&format!(
                    "#{} <code>{}</code> [{}]\n",
                    w.id,
                    esc_html(&w.name),
                    w.status
                ));
            }
        }
        Err(e) => out.push_str(&format!("  failed: {}\n", esc_html(&e))),
    }
    out
}

fn hits_text(coordinator: &str, key: &str, run: Option<i64>) -> String {
    let mut out = String::from("<b>Hits</b>\n");
    match pg_hits(coordinator, key, run) {
        Ok(hs) => {
            if hs.is_empty() {
                out.push_str("  none\n");
            }
            for h in hs {
                out.push_str(&format!(
                    "#{} run {} n=<code>{}</code> by <code>{}</code> at {}\n<code>{}</code>\n",
                    h.id,
                    h.run_id,
                    h.n,
                    esc_html(h.worker.as_deref().unwrap_or("?")),
                    esc_html(&h.found_at),
                    esc_html(&h.address)
                ));
            }
        }
        Err(e) => out.push_str(&format!("  failed: {}\n", esc_html(&e))),
    }
    out
}