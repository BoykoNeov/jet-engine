//! Plan § 45 bar B3's second half: among rung 107's crash-map requests (seed 13, the same generator as crash.rs), the
//! governed-release runs with a FAST cut (attack < 0.2, the fine step) that stop early — does each stop's diagnosis
//! re-run its last fine step and find a failure (`Stop.failure` Some), and is `s_last` the last point's `s`?
use turbojet::sandbox_controls::{controls, stop_reason, ControlsSettings, Route};
use turbojet::visuals::Json;

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
    let (mut fast_runs, mut fast_stops, mut with_failure, mut s_ok) = (0, 0, 0, 0);
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
        let which = r.pick(&["sched", "sched", "floor", "both"]);
        let (acc, flo) = ((which != "floor") as i32, (which != "sched") as i32);
        let red = (r.next() < 0.5) as i32;
        let redline = r.range(1200.0, 1800.0, 10.0);
        let ta = r.range(0.02, 0.5, 0.02);
        let tr = r.range(0.02, 1.0, 0.02);
        let by = r.pick(&["true", "true", "epr", "n1"]);
        let from = r.range(600.0, 1700.0, 10.0);
        let to = r.range(600.0, 1700.0, 10.0);
        let ramp = r.range(0.02, 2.0, 0.02);
        let rho = r.range(0.5, 2.0, 0.1);
        let req = format!(r#"{{"govern":1,"hold":"{hold}","speed_law":"{law}","gov_tau":{tau},"gov_gain":{gain},"generator":"{gen}","generator_power":{gp},"hold_cap":{},"metal_on":{},"cool_vane":{cv},"cool_blade":{cb},"shape":"{shape}","lever":"{lever}","bleed_mode":"{mode}","redline_on":{red},"redline":{redline},"release_on":1,"tau_att":{ta},"tau_rel":{tr},"thrust_by":"{by}","accel_on":{acc},"floor_on":{flo},"from":{from},"to":{to},"ramp":{ramp},"rho":{rho}}}"#,
                          cap as i32, metal as i32);
        let Ok(s) = ControlsSettings::from_json(&Json::parse(&req)) else { continue };
        if !(s.tau_att < 0.2) { continue; }
        let Ok(Ok((sv, o))) = std::panic::catch_unwind(|| controls(&s)) else { continue };
        if sv.route() != Route::GovernorRelease { continue; }
        fast_runs += 1;
        if o.points.len() >= s.expected_points() { continue; }
        fast_stops += 1;
        let st = stop_reason(&sv, &o.points).expect("a short run has a stop");
        let last_s = o.points.last().map(|p| p.s).unwrap_or(f64::NAN);
        if st.failure.is_some() { with_failure += 1; }
        if st.s_last == last_s || (o.points.is_empty() && st.s_last.is_nan()) { s_ok += 1; }
        println!("#{i} ds {} pts {}/{} s_last {} last.s {} failure {}\n   {req}", s.march_ds(), o.points.len(),
                 s.expected_points(), st.s_last, last_s,
                 st.failure.as_ref().map(|f| f.message.chars().take(100).collect::<String>()).unwrap_or("NONE".into()));
    }
    println!("fast-cut governed-release runs {fast_runs}; stopped {fast_stops}; diagnosis found a failure {with_failure}; s_last = last point's s {s_ok}");
}
