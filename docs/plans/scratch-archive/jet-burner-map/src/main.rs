//! Slice 5 crash map: the burner view through `sandbox::call`, over designs x burner knobs x every
//! mixing model. Each request's outcome: ok / refusal (pre-check) / panic (model message).
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;
use turbojet::sandbox::call;

thread_local! { static MSG: RefCell<String> = RefCell::new(String::new()); }

fn quiet() {
    std::panic::set_hook(Box::new(|info| {
        let t = match info.payload().downcast_ref::<String>() {
            Some(s) => s.clone(),
            None => info.payload().downcast_ref::<&str>().map(|s| s.to_string()).unwrap_or_default(),
        };
        MSG.with(|m| *m.borrow_mut() = t);
    }));
}

struct Rng(u64);
impl Rng {
    fn u(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 11) as f64) / ((1u64 << 53) as f64)
    }
    fn r(&mut self, a: f64, b: f64) -> f64 { a + (b - a) * self.u() }
    fn lg(&mut self, a: f64, b: f64) -> f64 { (a.ln() + (b.ln() - a.ln()) * self.u()).exp() }
    fn pick<T: Copy>(&mut self, v: &[T]) -> T { v[((self.u() * v.len() as f64) as usize).min(v.len() - 1)] }
}

fn norm(m: &str) -> String {
    let mut out = String::new();
    let mut last = false;
    for c in m.chars() {
        if c.is_ascii_digit() || (c == '.' && last) || (c == 'e' && last) {
            if !last { out.push('#'); }
            last = true;
        } else { last = false; out.push(c); }
    }
    out.chars().take(170).collect()
}

/// (design json, burner json)
fn sample(rng: &mut Rng, mode: &str) -> (String, String) {
    let pi_c = rng.lg(2.0, 40.0);
    let tt4 = rng.r(900.0, 2400.0);
    let (t0, p0) = rng.pick(&[(288.15, 101325.0), (250.0, 50000.0), (216.65, 22632.0), (216.65, 12000.0), (310.0, 101325.0)]);
    let m0 = rng.pick(&[0.1, 0.5, 0.85, 1.5, 2.5]);
    let design = format!("{{\"pi_c\":{pi_c},\"Tt4\":{tt4},\"T0\":{t0},\"p0\":{p0},\"M0\":{m0}}}");
    let phi_p = if rng.u() < 0.1 { 2.0 } else { rng.r(0.3, 2.0) };
    let tau = rng.lg(1e-4, 2e-2);
    let o = (rng.u() < 0.4) as i32;
    let pr = (rng.u() < 0.4) as i32;
    let peak = rng.lg(0.5, 10.0);
    let mut b = format!("\"phi_p\":{phi_p},\"tau\":{tau},\"super_eq_o\":{o},\"prompt\":{pr},\"prompt_peak\":{peak}");
    match mode {
        "instant" => b += ",\"quench\":\"instant\"",
        "time" => b += &format!(",\"quench\":\"time\",\"tau_q\":{}", rng.lg(1e-5, 1e-2)),
        model => {
            let j = rng.lg(1.0, 1000.0);
            b += &format!(",\"quench\":\"jets\",\"J\":{j},\"H\":{},\"U_c\":{},\"C_e\":{},\"shape_n\":{},\"closure\":\"{model}\"",
                          rng.lg(0.02, 0.3), rng.lg(20.0, 150.0), rng.lg(0.05, 0.5), rng.r(1.0, 4.0));
            let mut k: Vec<String> = vec![format!("\"s\":{}", rng.lg(0.02, 0.2))];
            let mut add = |n: &str, v: f64| k.push(format!("\"{n}\":{v}"));
            match model {
                "none" => {}
                "two_stream" => { add("c_opt", rng.r(1.0, 5.0)); add("tau_res", rng.lg(5e-4, 1e-2)); add("k_u", rng.r(0.0, 5.0)); add("b_u", rng.r(0.0, 6.0)); add("w_max", rng.r(0.1, 1.0)); }
                "pdf" => { add("c_opt", rng.r(1.0, 5.0)); add("k_g", rng.r(0.0, 1.0)); add("g_max", rng.r(0.05, 0.95)); }
                "pdf_quench" | "pocket" => { add("c_opt", rng.r(1.0, 5.0)); add("k_g", rng.r(0.0, 1.0)); add("g_max", rng.r(0.05, 0.95)); add("tau_res", rng.lg(5e-4, 1e-2)); add("b_u", rng.r(0.0, 6.0)); }
                "transported" => { add("c_opt", rng.r(1.0, 5.0)); add("c_phi", rng.r(0.5, 4.0)); add("da_opt", rng.r(0.5, 5.0)); add("w_cov", rng.r(0.3, 3.0)); add("tau_mix", rng.lg(5e-4, 1e-2)); }
                _ => { add("k_p", rng.r(0.15, 0.6)); add("k_y", rng.r(0.1, 0.5)); add("k_z", rng.r(0.1, 0.5)); }
            }
            if model == "none" { k.clear(); }
            if !k.is_empty() { b += &format!(",\"knobs\":{{{}}}", k.join(",")); }
        }
    }
    (design, format!("{{{b}}}"))
}

fn main() {
    quiet();
    let args: Vec<String> = std::env::args().collect();
    let plan: Vec<(&str, usize)> = match args.get(1).map(|s| s.as_str()) {
        Some("fast") => vec![("instant", 4000), ("time", 800), ("none", 800), ("two_stream", 800)],
        Some("curve") => vec![("pdf", 250), ("pdf_quench", 250), ("transported", 250), ("spatial", 250)],
        Some("pocket") => vec![("pocket", 48), ("spatial_dwell", 48), ("spatial_local", 48)],
        _ => panic!("fast|curve|pocket"),
    };
    let nthreads: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(8);
    let ops: Vec<String> = args.get(3).map(|s| s.split(',').map(String::from).collect()).unwrap_or(vec!["burner".into()]);
    let mut jobs = Vec::new();
    let mut rng = Rng(0x5eed_5eed + args[1].len() as u64);
    for (mode, n) in &plan {
        for _ in 0..*n {
            let (d, b) = sample(&mut rng, mode);
            for op in &ops { jobs.push((mode.to_string(), op.clone(), d.clone(), b.clone())); }
        }
    }
    let next = AtomicUsize::new(0);
    let out: Mutex<Vec<(String, String, String, String, String, f64)>> = Mutex::new(Vec::new());
    std::thread::scope(|sc| {
        for _ in 0..nthreads {
            sc.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::SeqCst);
                if i >= jobs.len() { break; }
                let (mode, op, d, b) = &jobs[i];
                let req = format!("{{\"op\":\"{op}\",\"settings\":{d},\"burner\":{b}}}");
                let t = Instant::now();
                let r = std::panic::catch_unwind(|| call(&req));
                let ms = t.elapsed().as_secs_f64() * 1e3;
                let (kind, msg) = match r {
                    Ok(s) if s.contains("\"ok\":0") => ("refusal".to_string(), s),
                    Ok(s) => ("ok".to_string(), s.chars().take(400).collect()),
                    Err(_) => ("panic".to_string(), MSG.with(|m| m.borrow().clone())),
                };
                out.lock().unwrap().push((mode.clone(), op.clone(), kind, msg, req, ms));
            });
        }
    });
    let out = out.into_inner().unwrap();
    let mut classes: BTreeMap<(String, String, String, String), (usize, String)> = BTreeMap::new();
    let mut times: BTreeMap<(String, String), Vec<f64>> = BTreeMap::new();
    for (mode, op, kind, msg, req, ms) in &out {
        let key = if kind == "ok" { String::new() } else { norm(msg) };
        let e = classes.entry((mode.clone(), op.clone(), kind.clone(), key)).or_insert((0, req.clone()));
        e.0 += 1;
        if kind == "ok" { times.entry((mode.clone(), op.clone())).or_default().push(*ms); }
    }
    for ((mode, op, kind, key), (n, ex)) in &classes {
        println!("{mode:<13} {op:<14} {kind:<8} {n:>5}  {key}");
        if kind != "ok" { println!("      e.g. {ex}"); }
    }
    for ((mode, op), mut v) in times {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!("time {mode:<13} {op:<14} n={} median {:.1} ms  p90 {:.1}  max {:.1}", v.len(), v[v.len() / 2], v[v.len() * 9 / 10], v[v.len() - 1]);
    }
    let mut f = String::new();
    for (mode, op, kind, msg, req, ms) in &out { f += &format!("{mode}\t{op}\t{kind}\t{ms:.1}\t{}\t{req}\n", msg.replace('\n', " ")); }
    std::fs::write(format!("W:/temp/claude/jet-burner-map/out-{}.tsv", args[1]), f).unwrap();
}
