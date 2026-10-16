//! Rung 96 — the inferred-thrust governor's crash map (sandbox-plan § 14.3's instrument): 1 000 random Controls requests,
//! every one governed on thrust READ from the pressure ratio or the LP speed (by either law), the table clean or loaded,
//! beside random instant limiters, levers, the
//! generator, cooling and the metal cap, both throttle ends over the slider range; through `sandbox::call`, counted by
//! outcome (with the governor's clamps), and TIMED.
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
    let mut r = Lcg(13);
    let mut tally: BTreeMap<String, usize> = BTreeMap::new();
    let mut stops: BTreeMap<String, usize> = BTreeMap::new();
    let mut panics = Vec::new();
    let mut times: Vec<(f64, String)> = Vec::new();
    for i in 0..1000 {
        let hold = "thrust";
        let by = r.pick(&["epr", "epr", "n1", "n1", "true"]);
        let clean = (r.next() < 0.6) as i32;
        let law = r.pick(&["rate", "prop"]);
        let tau = r.range(0.04, 1.0, 0.01);
        let gain = r.range(1.0, 50.0, 1.0);
        let gen = r.pick(&["off", "hp", "hp", "lp", "lp"]);
        let gp = r.range(0.0, 0.1, 0.005);
        let cool = r.next() < 0.4;
        let (cv, cb) = if cool { (r.range(0.0, 0.15, 0.01), r.range(0.0, 0.15, 0.01)) } else { (0.0, 0.0) };
        let cap = r.next() < 0.1;
        let metal = cap || r.next() < 0.2;
        let shape = r.pick(&["flow/press", "press/flow", "tilted", "flat-lp"]);
        let lever = r.pick(&["none", "none", "stator", "bleed", "both"]);
        let mode = r.pick(&["schedule", "valve"]);
        let (red, acc, flo) = ((r.next() < 0.4) as i32, (r.next() < 0.3) as i32, (r.next() < 0.2) as i32);
        let from = r.range(600.0, 1700.0, 10.0);
        let to = r.range(600.0, 1700.0, 10.0);
        let ramp = r.range(0.02, 2.0, 0.02);
        let rho = r.range(0.5, 2.0, 0.1);
        let req = format!(r#"{{"op":"controls","controls":{{"govern":1,"hold":"{hold}","thrust_by":"{by}","table_clean":{clean},"speed_law":"{law}","gov_tau":{tau},"gov_gain":{gain},"generator":"{gen}","generator_power":{gp},"hold_cap":{},"metal_on":{},"cool_vane":{cv},"cool_blade":{cb},"shape":"{shape}","lever":"{lever}","bleed_mode":"{mode}","redline_on":{red},"accel_on":{acc},"floor_on":{flo},"from":{from},"to":{to},"ramp":{ramp},"rho":{rho}}}}}"#,
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
                let edge = if s.contains(r#""rich""#) && s.contains(r#""lean""#) { " (rich + lean edge)" }
                    else if s.contains(r#""lean""#) { " (lean edge)" } else if s.contains(r#""rich""#) { " (rich edge)" } else { "" };
                if s.contains(r#""stopped":1"#) {
                    let k = s.find(r#""kind":""#).map(|p| s[p + 8..].split('"').next().unwrap_or("").to_string()).unwrap_or_default();
                    if k == "other" { let m = s.find(r#""message":""#).map(|p| s[p..].chars().take(330).collect::<String>()).unwrap_or_default(); println!("OTHER #{i}: {m}
   {req}"); }
                    *stops.entry(k).or_default() += 1;
                    format!("ran, stopped early{edge}")
                } else { format!("ran to the end{edge}") }
            }
            Ok(s) => {
                let p = s.find(r#""reason":""#).map(|p| p + 10).unwrap_or(0);
                let words: String = s[p..].chars().take(70).collect();
                format!("refused: {words}")
            }
        };
        *tally.entry(key).or_default() += 1;
    }
    for (k, n) in &tally { println!("{n:>5}  {k}"); }
    println!("stop kinds: {stops:?}");
    times.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let total: f64 = times.iter().map(|t| t.0).sum();
    println!("{} panics; total {:.1} s, median {:.3} s, slowest {:.2} / {:.2} / {:.2} s", panics.len(), total,
             times[times.len() / 2].0, times[0].0, times[1].0, times[2].0);
    println!("slowest request: {}", times[0].1);
    for p in &panics { println!("{p}"); }
}
