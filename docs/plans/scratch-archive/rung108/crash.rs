//! Rung 108 — the crash map of the two-speed temperature limiter (rung 95's instrument, rung 107's script, seed 108):
//! 1 000 random Controls requests with the limiter, its lag and "cuts fast, gives back slowly" all on — half governed
//! (thrust read directly / from the pressure ratio / the fan speed, HP speed by either law, or the burner temperature),
//! half open — cut-in 0.02–0.5 and give-back 0.02–1.0 (the sliders, so some fall under the floors), the redline
//! 1200–1800 K, the schedule / floor / both / neither, rung 52's release at random (10 %, the two-lag refusal), random
//! levers, generator, cooling and metal cap, both throttle ends over the slider range; through `sandbox::call`, counted
//! by outcome, TIMED. Every "other" stop and every minimum-fuel run is printed; every re-attack too.
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

/// Re-run an admitted request on the library and count the lag's sign changes after engagement (releases, re-attacks).
fn reattacks(req: &str) -> (usize, usize) {
    use turbojet::fuel_transient::PointExtra;
    use turbojet::sandbox_controls::{controls, ControlsSettings};
    use turbojet::visuals::Json;
    let j = Json::parse(req);
    let s = ControlsSettings::from_json(j.get("controls").unwrap()).unwrap();
    let (_, o) = controls(&s).unwrap();
    let (mut rel, mut re, mut att) = (0, 0, None::<bool>);
    for p in &o.points {
        let PointExtra::Asym { g, required: r } = p.extra else { panic!("the two-speed route records its clip") };
        if g == 0.0 && r == 0.0 { continue; }
        let a = r > g;
        if let Some(prev) = att { if prev && !a { rel += 1 } else if !prev && a { re += 1 } }
        att = Some(a);
    }
    (rel, re)
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let mut r = Lcg(108);
    let mut tally: BTreeMap<String, usize> = BTreeMap::new();
    let mut stops: BTreeMap<String, usize> = BTreeMap::new();
    let mut panics = Vec::new();
    let mut times: Vec<(f64, String)> = Vec::new();
    for i in 0..1000 {
        let hold = r.pick(&["thrust", "thrust", "speed", "speed", "tt4"]);
        let law = r.pick(&["rate", "prop"]);
        let tau = r.range(0.04, 1.0, 0.01);
        let gain = r.range(1.0, 50.0, 1.0);
        let gen = r.pick(&["off", "off", "hp", "lp"]);
        let gp = r.range(0.0, 0.1, 0.005);
        let cool = r.next() < 0.4;
        let (cv, cb) = if cool { (r.range(0.0, 0.15, 0.01), r.range(0.0, 0.15, 0.01)) } else { (0.0, 0.0) };
        let cap = r.next() < 0.25;
        let metal = cap || r.next() < 0.2;
        let shape = r.pick(&["flow/press", "press/flow", "tilted", "flat-lp"]);
        let lever = r.pick(&["none", "none", "stator", "bleed", "both"]);
        let mode = r.pick(&["schedule", "valve"]);
        let which = r.pick(&["none", "none", "sched", "floor", "both"]);
        let (acc, flo) = ((which == "sched" || which == "both") as i32, (which == "floor" || which == "both") as i32);
        let gov = (r.next() < 0.5) as i32;
        let rel_on = (r.next() < 0.1) as i32;
        let red = 1;
        let redline = r.range(1200.0, 1800.0, 10.0);
        let ta = r.range(0.02, 0.5, 0.02);
        let tr = r.range(0.02, 1.0, 0.02);
        let by = r.pick(&["true", "true", "epr", "n1"]);
        let from = r.range(600.0, 1700.0, 10.0);
        let to = r.range(600.0, 1700.0, 10.0);
        let ramp = r.range(0.02, 2.0, 0.02);
        let rho = r.range(0.5, 2.0, 0.1);
        let req = format!(r#"{{"op":"controls","controls":{{"govern":{gov},"hold":"{hold}","speed_law":"{law}","gov_tau":{tau},"gov_gain":{gain},"generator":"{gen}","generator_power":{gp},"hold_cap":{},"metal_on":{},"cool_vane":{cv},"cool_blade":{cb},"shape":"{shape}","lever":"{lever}","bleed_mode":"{mode}","redline_on":{red},"redline":{redline},"release_on":{rel_on},"tau_att":0.2,"tau_rel":0.4,"gov_lag_on":1,"gov_lag_two":1,"gov_lag_att":{ta},"gov_lag":{tr},"thrust_by":"{by}","accel_on":{acc},"floor_on":{flo},"from":{from},"to":{to},"ramp":{ramp},"rho":{rho}}}}}"#,
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
                let route = s.find(r#""route":""#).map(|p| s[p + 9..].split('"').next().unwrap_or("").to_string()).unwrap_or_default();
                let reatt = reattacks(&req);
                if reatt.1 > 0 { println!("RE-ATTACK #{i} ({} releases, {} re-attacks): {req}", reatt.0, reatt.1); }
                *tally.entry(format!("  (releases per admitted run: {})", reatt.0)).or_default() += 1;
                let mf = if s.contains(r#""min_fuel""#) { println!("MINFUEL #{i}: {req}"); " [MIN-FUEL STOP]" } else { "" };
                let edge = if s.contains(r#""rich""#) && s.contains(r#""lean""#) { " (rich + lean edge)" }
                    else if s.contains(r#""lean""#) { " (lean edge)" } else if s.contains(r#""rich""#) { " (rich edge)" } else { "" };
                if s.contains(r#""stopped":1"#) {
                    let k = s.find(r#""kind":""#).map(|p| s[p + 8..].split('"').next().unwrap_or("").to_string()).unwrap_or_default();
                    if k == "other" { let m = s.find(r#""message":""#).map(|p| s[p..].chars().take(330).collect::<String>()).unwrap_or_default(); println!("OTHER #{i}: {m}
   {req}"); }
                    *stops.entry(k).or_default() += 1;
                    format!("[{route}] ran, stopped early{edge}{mf}")
                } else { format!("[{route}] ran to the end{edge}{mf}") }
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
    for t in times.iter().take(5) { println!("slow {:.2} s: {}", t.0, t.1); }
    for p in &panics { println!("{p}"); }
}
