//! Worst-case cost of a finer step beside the governor + release: rung 107's crash-map requests (seed 13, the same
//! generator), the attack / release lifted to the shipped floors (0.2 / 0.1) so they are admitted; each admitted one
//! timed natively, SINGLE-THREADED: build (the ends) and the march at DS. The 10 slowest (build + march) are then
//! re-timed with the attack set to 0.05 / 0.02 / 0.005 and the march at DS/4, DS/10, DS/40.
use turbojet::sandbox_controls::{build, ControlsSettings, DS};
use turbojet::visuals::Json;
use gov_fast_attack::march;
use std::time::Instant;

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
    let mut rows: Vec<(f64, f64, f64, String)> = Vec::new(); // (total, build, march, req)
    let (mut admitted, mut refused) = (0, 0);
    for _ in 0..1000 {
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
        let which = r.pick(&["sched", "sched", "floor", "both"]);
        let (acc, flo) = ((which != "floor") as i32, (which != "sched") as i32);
        let red = (r.next() < 0.5) as i32;
        let redline = r.range(1200.0, 1800.0, 10.0);
        let ta = r.range(0.02, 0.5, 0.02).max(0.2);
        let tr = r.range(0.02, 1.0, 0.02).max(0.1);
        let by = r.pick(&["true", "true", "epr", "n1"]);
        let from = r.range(600.0, 1700.0, 10.0);
        let to = r.range(600.0, 1700.0, 10.0);
        let ramp = r.range(0.02, 2.0, 0.02);
        let rho = r.range(0.5, 2.0, 0.1);
        let req = format!(r#"{{"govern":1,"hold":"{hold}","speed_law":"{law}","gov_tau":{tau},"gov_gain":{gain},"generator":"{gen}","generator_power":{gp},"hold_cap":{},"metal_on":{},"cool_vane":{cv},"cool_blade":{cb},"shape":"{shape}","lever":"{lever}","bleed_mode":"{mode}","redline_on":{red},"redline":{redline},"release_on":1,"tau_att":{ta},"tau_rel":{tr},"thrust_by":"{by}","accel_on":{acc},"floor_on":{flo},"from":{from},"to":{to},"ramp":{ramp},"rho":{rho}}}"#,
                          cap as i32, metal as i32);
        let Ok(s) = ControlsSettings::from_json(&Json::parse(&req)) else { refused += 1; continue };
        let t0 = Instant::now();
        let Ok(Ok(sv)) = std::panic::catch_unwind(|| build(&s)) else { refused += 1; continue };
        let tb = t0.elapsed().as_secs_f64();
        if sv.release_lag().is_none() { refused += 1; continue; }
        let t1 = Instant::now();
        let _ = sv.march();
        let tm = t1.elapsed().as_secs_f64();
        admitted += 1;
        rows.push((tb + tm, tb, tm, req));
    }
    rows.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let med = &rows[rows.len() / 2];
    println!("admitted {admitted}, refused/not-release {refused}; median total {:.3} s (build {:.3}, march {:.3})", med.0, med.1, med.2);
    let marches: Vec<f64> = rows.iter().map(|r| r.2).collect();
    println!("march at DS: max {:.3} s, median {:.3} s", marches.iter().cloned().fold(0.0, f64::max), {
        let mut m = marches.clone(); m.sort_by(|a, b| a.partial_cmp(b).unwrap()); m[m.len() / 2] });
    // The slowest by march alone too (a finer step multiplies the march, not the ends).
    let mut by_march = rows.clone();
    by_march.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    let mut pick: Vec<_> = rows.iter().take(6).cloned().collect();
    for x in by_march.iter().take(6) { if !pick.iter().any(|p| p.3 == x.3) { pick.push(x.clone()); } }
    for (tot, tb, tm, req) in pick {
        let s = ControlsSettings::from_json(&Json::parse(&req)).unwrap();
        let mut sv = build(&s).unwrap();
        let mut cells = Vec::new();
        for (ta, k) in [(0.05, 2usize), (0.02, 4), (0.005, 8)] {
            sv.settings.tau_att = ta;
            let t = Instant::now();
            let p = march(&sv, DS / k as f64);
            let dt = t.elapsed().as_secs_f64();
            cells.push(format!("att {ta} k{k}: march {dt:.2} s ({}x), total {:.2} s, pts {}", (dt / tm).round(), tb + dt, (p.len() - 1) / k + 1));
        }
        println!("total {tot:.2} s = build {tb:.2} + march {tm:.3} | {}\n   {req}", cells.join(" | "));
    }
}
