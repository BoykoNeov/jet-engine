//! The step check on rung 106's shipped route, the floor and schedule cases (42 settings) × the lag 0.1–1.0 every 0.05,
//! read on EVERY point of the run being checked: step 0.01 (k=2) against 0.0025, and 0.005 (k=4) against 0.00125.
use rung106_rough::*;
use std::sync::Mutex;
use turbojet::sandbox_controls::{build, DS};
fn main() {
    let mut v = Vec::new();
    for i in 0..19 {
        let lag = ((0.1 + 0.05 * i as f64) * 100.0).round() / 100.0;
        v.extend(settings(lag).into_iter().filter(|(n, _)| !n.starts_with("red")));
    }
    let jobs = Mutex::new(v);
    std::thread::scope(|sc| {
        for _ in 0..8 {
            sc.spawn(|| loop {
                let Some((name, s)) = jobs.lock().unwrap().pop() else { break };
                let Ok(sv) = build(&s) else { println!("{name}: REFUSED"); continue };
                let n_full = s.expected_points();
                let r: Vec<_> = [2usize, 4, 8, 16].iter().map(|&k| lag_march(&sv, DS / k as f64)).collect();
                let full = |x: &Vec<turbojet::fuel_transient::FuelPoint>, k: usize| (x.len() - 1) as f64 / k as f64 + 1.0 == n_full as f64;
                let ends = full(&r[0], 2) && full(&r[1], 4) && full(&r[2], 8) && full(&r[3], 16);
                println!("{name} k2v8={:.3e} k4v16={:.3e} ends={ends}", gap(&r[0], 1, &r[2], 4), gap(&r[1], 1, &r[3], 4));
            });
        }
    });
}
