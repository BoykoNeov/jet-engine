//! Rung 94 — the Controls hold's crash map (sandbox-plan § 14.3's instrument): 1 000 random Controls requests, every one
//! with the generator on and a hold (thrust or HP speed), the cap on or off, with and without cooling, every map and
//! lever, both throttle ends over the slider range — through `sandbox::call`, counted by outcome, and TIMED.
use std::collections::BTreeMap;
use turbojet::sandbox::call;

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
    fn pick<'a>(&mut self, xs: &[&'a str]) -> &'a str { xs[(self.next() * xs.len() as f64) as usize % xs.len()] }
    fn range(&mut self, a: f64, b: f64, step: f64) -> f64 { ((a + (b - a) * self.next()) / step).round() * step }
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let mut r = Lcg(11);
    let mut tally: BTreeMap<String, usize> = BTreeMap::new();
    let mut panics = Vec::new();
    let mut times: Vec<(f64, String)> = Vec::new();
    for i in 0..1000 {
        let gen = r.pick(&["hp", "lp"]);
        let by = r.pick(&["power", "share"]);
        let gp = r.range(0.0, 0.1, 0.005);
        let gs = r.range(0.0, 0.3, 0.01);
        let hold = r.pick(&["thrust", "speed"]);
        let cool = r.next() < 0.5;
        let (cv, cb) = if cool { (r.range(0.0, 0.2, 0.01), r.range(0.0, 0.2, 0.01)) } else { (0.0, 0.0) };
        let cap = r.next() < 0.3;
        let metal = cap || r.next() < 0.3;
        let shape = r.pick(&["flow/press", "press/flow", "tilted", "flat-lp"]);
        let lever = r.pick(&["none", "none", "stator", "bleed", "both"]);
        let mode = r.pick(&["schedule", "valve"]);
        let from = r.range(600.0, 1700.0, 10.0);
        let to = r.range(600.0, 1700.0, 10.0);
        let rho = r.range(0.5, 2.0, 0.1);
        let req = format!(r#"{{"op":"controls","controls":{{"generator":"{gen}","generator_by":"{by}","generator_power":{gp},"generator_share":{gs},"hold":"{hold}","hold_cap":{},"metal_on":{},"cool_vane":{cv},"cool_blade":{cb},"shape":"{shape}","lever":"{lever}","bleed_mode":"{mode}","from":{from},"to":{to},"rho":{rho}}}}}"#,
                          cap as i32, metal as i32);
        let t0 = std::time::Instant::now();
        let out = std::panic::catch_unwind(|| call(&req));
        let dt = t0.elapsed().as_secs_f64();
        times.push((dt, req.clone()));
        let key = match out {
            Err(e) => {
                let m = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                panics.push(format!("#{i} {req}\n   {m}"));
                "PANIC".to_string()
            }
            Ok(s) if s.starts_with(r#"{"ok":1"#) => {
                let capped = s.contains(r#""hold_capped":1"#);
                if s.contains(r#""stopped":1"#) { format!("ran, stopped early{}", if capped { " (capped)" } else { "" }) }
                else { format!("ran to the end{}", if capped { " (capped)" } else { "" }) }
            }
            Ok(s) => {
                if s.contains("met a burner temperature") { println!("GAP #{i}: {}", &s[s.find("met a").unwrap_or(0)..s.len().min(s.find("met a").unwrap_or(0) + 260)]); }
                let why = if s.contains("below the least this load allows") { "hold: below the load's floor" }
                    else if s.contains("more than this loaded engine gives") { "hold: above the range" }
                    else if s.contains("met a burner temperature") { "hold: a gap" }
                    else if s.contains("no steady point anywhere") { "hold: no solve" }
                    else if s.contains("metal cap") || s.contains("over its limit") { "cap refusal" }
                    else if s.contains("sets no") { "unloaded end has no steady point" }
                    else if s.contains("generator would take") || s.contains("half or more") { "generator share past half" }
                    else if s.contains("cooling") && s.contains("half") { "cooling past half" }
                    else { "other refusal" };
                if why == "other refusal" && !s.contains("prechecks") { println!("OTHER #{i}: {}", &s[..s.len().min(300)]); }
                format!("refused: {why}")
            }
        };
        *tally.entry(key).or_default() += 1;
    }
    for (k, n) in &tally { println!("{n:>5}  {k}"); }
    times.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let total: f64 = times.iter().map(|t| t.0).sum();
    println!("{} panics; total {:.1} s, median {:.3} s, slowest {:.2} / {:.2} / {:.2} s", panics.len(), total,
             times[times.len() / 2].0, times[0].0, times[1].0, times[2].0);
    println!("slowest request: {}", times[0].1);
    for p in &panics { println!("{p}"); }
}
