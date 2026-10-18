//! Rung 99 — the crash map: rung 98's map plus the speed correction at random (grid 60 %, formula 20 %, none 20 %); one OUT line per request with an FNV hash of what it did (compared against a worktree of 7113ed3, which ignores the new key).
//! over altitude −1 … 25 km and ±45 K off the standard day, beside every other knob at random; through `sandbox::call`,
//! counted by outcome, TIMED; refusals grouped by their opening words, and each with its altitude.
use std::collections::BTreeMap;
use turbojet::atmosphere::standard;
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
    let seed: u64 = std::env::args().nth(1).map(|s| s.parse().unwrap()).unwrap_or(14);
    let n: usize = std::env::args().nth(2).map(|s| s.parse().unwrap()).unwrap_or(1000);
    let zlo: f64 = std::env::args().nth(3).map(|s| s.parse().unwrap()).unwrap_or(-1000.0);
    let zhi: f64 = std::env::args().nth(4).map(|s| s.parse().unwrap()).unwrap_or(25000.0);
    let dtm: f64 = std::env::args().nth(5).map(|s| s.parse().unwrap()).unwrap_or(45.0);
    let mmax: f64 = std::env::args().nth(6).map(|s| s.parse().unwrap()).unwrap_or(3.5);
    let mut r = Lcg(seed);
    let mut tally: BTreeMap<String, (usize, f64, f64)> = BTreeMap::new();
    let mut stops: BTreeMap<String, usize> = BTreeMap::new();
    let mut panics = Vec::new();
    let mut times: Vec<(f64, String)> = Vec::new();
    for i in 0..n {
        let z = r.range(zlo, zhi, 50.0);
        let dt = r.range(-dtm, dtm, 0.5);
        let (t, p) = standard(z);
        let shape = r.pick(&["flow/press", "press/flow", "tilted", "flat-lp"]);
        let lever = r.pick(&["none", "none", "stator", "bleed", "both"]);
        let mode = r.pick(&["schedule", "schedule", "valve"]);
        let vspeed = r.pick(&["instant", "instant", "slow"]);
        let (red, lag, acc, flo, rel) = ((r.next() < 0.4) as i32, (r.next() < 0.15) as i32, (r.next() < 0.3) as i32,
                                        (r.next() < 0.25) as i32, (r.next() < 0.15) as i32);
        let fref = r.pick(&["phi", "incidence"]);
        let floor_phi = r.range(0.6, 0.85, 0.005);
        let govern = (r.next() < 0.3) as i32;
        let hold = r.pick(&["tt4", "thrust", "speed"]);
        let by = r.pick(&["true", "epr", "n1"]);
        let law = r.pick(&["rate", "prop"]);
        let tau = r.range(0.04, 1.0, 0.01);
        let gain = r.range(1.0, 50.0, 1.0);
        let gen = r.pick(&["off", "off", "hp", "lp"]);
        let gp = r.range(0.0, 0.1, 0.005);
        let cust = r.pick(&["off", "off", "share", "amount"]);
        let cs = r.range(0.0, 0.2, 0.01);
        let cool = r.next() < 0.3;
        let (cv, cb) = if cool { (r.range(0.0, 0.15, 0.01), r.range(0.0, 0.15, 0.01)) } else { (0.0, 0.0) };
        let metal = (r.next() < 0.2) as i32;
        let from = r.range(600.0, 1700.0, 10.0);
        let to = r.range(600.0, 1700.0, 10.0);
        let ramp = r.range(0.02, 2.0, 0.02);
        let settle = r.range(0.5, 6.0, 0.1);
        let rho = r.range(0.2, 5.0, 0.1);
        let m0 = r.range(0.05, mmax, 0.01);
        let tat = if r.next() < 0.8 { "design" } else { "flight" };
        let (cp, ctt) = ((r.next() < 0.85) as i32, (r.next() < 0.85) as i32);
        let sc = { let u = r.next(); if u < 0.6 { "grid" } else if u < 0.8 { "formula" } else { "none" } };
        let req = format!(r#"{{"op":"controls","controls":{{"T0":{},"p0":{},"shape":"{shape}","lever":"{lever}","bleed_mode":"{mode}","valve_speed":"{vspeed}","redline_on":{red},"gov_lag_on":{lag},"accel_on":{acc},"floor_on":{flo},"floor_ref":"{fref}","floor_phi":{floor_phi},"release_on":{rel},"govern":{govern},"hold":"{hold}","thrust_by":"{by}","speed_law":"{law}","gov_tau":{tau},"gov_gain":{gain},"generator":"{gen}","generator_power":{gp},"customer":"{cust}","customer_share":{cs},"customer_amount":{cs},"cool_vane":{cv},"cool_blade":{cb},"metal_on":{metal},"from":{from},"to":{to},"ramp":{ramp},"settle":{settle},"rho":{rho},"M0":{m0},"table_at":"{tat}","corr_p":{cp},"corr_t":{ctt},"speed_corr":"{sc}"}}}}"#,
                          t + dt, p);
        if std::env::var("CRASH_REQS").is_ok() { println!("REQ	{i}	{req}"); continue; }
        let t0 = std::time::Instant::now();
        let out = std::panic::catch_unwind(|| call(&req));
        let el = t0.elapsed().as_secs_f64();
        times.push((el, req.clone()));
        let hash = match &out { Ok(s) => key_hash(s), Err(_) => 0 };
        let brief: String = match &out { Ok(s) => match s.find("\"reason\":\"") { Some(p) => s[p + 10..].chars().take(80).collect(), None => if s.contains("\"stopped\":1") { "stopped".into() } else { "ran".into() } }, Err(_) => "PANIC".into() };
        let key = match out {
            Err(e) => {
                let m = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                panics.push(format!("#{i} M {m0} z {z} dT {dt} {req}\n   {m}"));
                "PANIC".to_string()
            }
            Ok(s) if s.starts_with(r#"{"ok":1"#) => {
                if s.contains(r#""stopped":1"#) {
                    let k = s.find(r#""kind":""#).map(|p| s[p + 8..].split('"').next().unwrap_or("").to_string()).unwrap_or_default();
                    if k == "other" || k == "unknown" { let m = s.find(r#""message":""#).map(|p| s[p..].chars().take(300).collect::<String>()).unwrap_or_default(); println!("{k} #{i} z {z} dT {dt}: {m}
   REQ {req}"); }
                    *stops.entry(k).or_default() += 1;
                    "ran, stopped early".to_string()
                } else { "ran to the end".to_string() }
            }
            Ok(s) => {
                let p = s.find(r#""reason":""#).map(|p| p + 10).unwrap_or(0);
                let words: String = s[p..].chars().take(60).collect();
                // Group by the words after the throttle number.
                let full: String = s[p..].chars().take(900).collect();
                let why = ["customer bleed would take", "cooling together", "generator would take", "does not settle",
                           "off the part of their maps", "steady solve fails", "thrust asked", "HP speed asked", "With this",
                           "made at the design flight", "design-flight table", "design altitude's air", "recomputed", "blended", "acceleration schedule", "no table", "too little pressure", "Mach"]
                    .iter().find(|w| full.contains(*w)).map(|w| w.to_string());
                let lo = (m0 * 2.0).floor() as i32;
                let g = match why {
                    Some(w) => format!("[M {:.1}-{:.1}] {w}", lo as f64 / 2.0, lo as f64 / 2.0 + 0.5),
                    None => { println!("UNCLASSIFIED #{i} z {z} dT {dt}: {}
   REQ {req}", full.chars().take(700).collect::<String>()); words.chars().take(45).collect::<String>() }
                };
                if full.contains("off the part of their maps") || full.contains("does not settle") {
                    let at: String = full.chars().take(40).collect();
                    println!("DETAIL {} M {m0} z {z} dT {dt} T0 {:.0} from {from} to {to} lever {lever} gen {gen} {gp} cust {cust} {cs} cool {cv}/{cb} | {at}",
                             if full.contains("does not settle") { "SETTLE" } else { "OFFMAP" }, t + dt);
                }
                if full.contains("cannot push it out") { println!("SOLVEFAILS #{i} z {z} dT {dt}: {}
   REQ {req}", full.chars().take(400).collect::<String>()); }
                format!("refused: {g}")
            }
        };
        println!("OUT	{i}	{el:.4}	{hash:016x}	{key}	{brief}");
        let e = tally.entry(key).or_insert((0, f64::INFINITY, f64::NEG_INFINITY));
        e.0 += 1; e.1 = e.1.min(z); e.2 = e.2.max(z);
    }
    for (k, (n, lo, hi)) in &tally { println!("{n:>5}  [z {lo:>6}..{hi:>6}]  {k}"); }
    println!("stop kinds: {stops:?}");
    times.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let total: f64 = times.iter().map(|t| t.0).sum();
    println!("{} panics; total {:.1} s, median {:.3} s, slowest {:.2} / {:.2} / {:.2} s", panics.len(), total,
             times[times.len() / 2].0, times[0].0, times[1].0, times[2].0);
    println!("slowest request: {}", times[0].1);
    for p in &panics { println!("{p}"); }
}

/// A hash of what the run DID — the points, the stop, the refusal's words — not of the answer's spelling (the new
/// build adds keys the old one lacks).
pub fn key_hash(s: &str) -> u64 {
    let j = turbojet::visuals::Json::parse(s);
    let mut text = String::new();
    for k in ["ok", "reason", "s", "nu_lp", "nu_hp", "Tt4", "fuel", "thrust", "stopped", "stop", "phi_lp", "phi_hp"] {
        if let Some(v) = j.get(k) { text.push_str(k); text.push_str(&v.dump_compact()); }
    }
    text.bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3))
}
