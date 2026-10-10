//! Time the slowest successful requests of a crash-map TSV ONE AT A TIME (no contention).
use std::time::Instant;
use turbojet::sandbox::call;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let k: usize = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(3);
    let text = std::fs::read_to_string(&a[1]).unwrap();
    let mut rows: Vec<(String, f64, String)> = text.lines().filter_map(|l| {
        let f: Vec<&str> = l.split('\t').collect();
        (f[2] == "ok").then(|| (f[0].to_string(), f[3].parse::<f64>().unwrap(), f[5].to_string()))
    }).collect();
    rows.sort_by(|x, y| y.1.partial_cmp(&x.1).unwrap());
    let mut modes: Vec<String> = rows.iter().map(|r| r.0.clone()).collect();
    modes.sort();
    modes.dedup();
    for m in modes {
        for (_, ms, req) in rows.iter().filter(|r| r.0 == m).take(k) {
            let t = Instant::now();
            let _ = call(req);
            let alone = t.elapsed().as_secs_f64() * 1e3;
            let b = &req[req.find("\"burner\"").unwrap()..];
            println!("{m:<14} crash-map {ms:>9.0} ms  alone {alone:>8.0} ms  {}", &b[..b.len().min(330)]);
        }
    }
}
