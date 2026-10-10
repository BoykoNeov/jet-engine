//! Controls crash map: random requests over the page's slider box; tally the outcome (complete / stop kind).
use std::collections::BTreeMap;
use turbojet::visuals::Json;
struct Lcg(u64);
impl Lcg {
    fn u(&mut self) -> f64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); (self.0 >> 11) as f64 / (1u64 << 53) as f64 }
    fn r(&mut self, a: f64, b: f64) -> f64 { a + (b - a) * self.u() }
    fn b(&mut self, p: f64) -> i64 { (self.u() < p) as i64 }
    fn step(&mut self, a: f64, b: f64, st: f64) -> f64 { ((self.r(a, b) / st).round() * st * 1e9).round() / 1e9 }
}
fn main() {
    let n: usize = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(1000);
    let seed: u64 = std::env::args().nth(2).and_then(|a| a.parse().ok()).unwrap_or(11);
    let mut g = Lcg(seed);
    let shapes = ["flow/press", "press/flow", "tilted", "flat-lp"];
    let levers = ["none", "stator", "bleed"];
    let mut tally: BTreeMap<String, usize> = BTreeMap::new();
    let mut ex: BTreeMap<String, String> = BTreeMap::new();
    std::panic::set_hook(Box::new(|_| {}));
    for _ in 0..n {
        let req = format!(r#"{{"op":"controls","controls":{{"shape":"{}","rho":{},"from":{},"to":{},"ramp":{},"settle":{},"redline_on":{},"redline":{},"gov_lag_on":{},"gov_lag":{},"accel_on":{},"accel_margin":{},"floor_on":{},"floor_spool":"{}","floor_phi":{},"floor_ref":"{}","release_on":{},"tau_att":{},"tau_rel":{},"lever":"{}","stator_spool":"{}","v_max":{},"v_n_lo":{},"b_max":{},"b_n_lo":{}}}}}"#,
            shapes[(g.u() * 4.0) as usize % 4], g.step(0.2, 5.0, 0.1), g.step(600.0, 1700.0, 10.0), g.step(600.0, 1700.0, 10.0),
            g.step(0.02, 3.0, 0.02), g.step(0.5, 9.0, 0.1), g.b(0.4), g.step(1200.0, 1800.0, 5.0), g.b(0.2), g.step(0.02, 1.0, 0.01),
            g.b(0.4), g.step(0.0, 0.6, 0.01), g.b(0.5), if g.u() < 0.5 { "lp" } else { "hp" }, g.step(0.55, 1.0, 0.005),
            if g.u() < 0.5 { "phi" } else { "incidence" }, g.b(0.3), g.step(0.02, 0.2, 0.01), g.step(0.02, 1.0, 0.01),
            levers[(g.u() * 3.0) as usize % 3], if g.u() < 0.5 { "lp" } else { "hp" }, g.step(0.0, 0.3, 0.01), g.step(0.4, 0.8, 0.01),
            g.step(0.0, 0.3, 0.01), g.step(0.4, 0.8, 0.01));
        let out = std::panic::catch_unwind(|| turbojet::sandbox::call(&req));
        let key = match out {
            Err(e) => format!("PANIC {}", e.downcast_ref::<String>().cloned().or(e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default().chars().take(60).collect::<String>()),
            Ok(s) => { let j = Json::parse(&s); match j.get("ok") {
                Some(Json::Int(1)) => match j.get("stop") { Some(Json::Null) | None => "complete".into(),
                    Some(st) => match st.get("kind") { Some(Json::Str(k)) => format!("stop {k}"), _ => "stop ?".into() } },
                _ => "refused".into() } }
        };
        *tally.entry(key.clone()).or_default() += 1;
        ex.entry(key).or_insert(req);
    }
    for (k, v) in &tally { println!("{v:5}  {k}"); }
    for (k, v) in &ex { if k.contains("schedule_check") || k.contains("other") || k.starts_with("PANIC") { println!("\n{k}\n{v}"); } }
}
