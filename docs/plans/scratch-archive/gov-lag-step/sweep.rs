//! The step check on rung 106's shipped route: the 63 settings × lag {0.1, 0.2, 0.4, 0.8}, at steps 0.02/k for
//! k ∈ {1, 2, 4, 8, 16}; gaps k=1 vs 4, k=2 vs 8, k=4 vs 16 on the 0.02 grid; run lengths (in 0.02 points); seconds.
use rung106_rough::*;
use std::sync::Mutex;
use std::time::Instant;
use turbojet::sandbox_controls::{build, DS};
fn main() {
    let mut v = Vec::new();
    for lag in [0.1, 0.2, 0.4, 0.8] { v.extend(settings(lag)); }
    let jobs = Mutex::new(v);
    std::thread::scope(|sc| {
        for _ in 0..8 {
            sc.spawn(|| loop {
                let Some((name, s)) = jobs.lock().unwrap().pop() else { break };
                let Ok(sv) = build(&s) else { println!("{name}: REFUSED"); continue };
                let n_full = s.expected_points();
                let mut runs = Vec::new();
                let mut secs = Vec::new();
                for k in [1usize, 2, 4, 8, 16] {
                    let t = Instant::now();
                    runs.push(lag_march(&sv, DS / k as f64));
                    secs.push(t.elapsed().as_secs_f64());
                }
                let g14 = gap(&runs[0], 1, &runs[2], 4);
                let g28 = gap(&runs[1], 2, &runs[3], 8);
                let g416 = gap(&runs[2], 4, &runs[4], 16);
                let lens: Vec<String> = runs.iter().enumerate()
                    .map(|(i, r)| format!("{}", (r.len() - 1) as f64 / (1u32 << i) as f64 + 1.0)).collect();
                println!("{name} k1v4={g14:.1e} k2v8={g28:.1e} k4v16={g416:.1e} pts={} of {n_full} t1={:.2} t4={:.2}",
                         lens.join("/"), secs[0], secs[2]);
            });
        }
    });
}
