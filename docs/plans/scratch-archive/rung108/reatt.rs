//! The crash map's 12 re-attacking requests (all open, all throttle PULL-BACKS starting above the redline). Counts the
//! clip's turns (falling → rising) on the two-speed route, read two ways: from its recorded clip g (exact) and from the
//! applied cut mf_sched − mf (the proxy rung 47's route allows — it does not record g); then the proxy on rung 47's single
//! lag at the cut-in time and at the give-back time (the same request with the tick box off).
use turbojet::fuel_transient::{FuelPoint, PointExtra};
use turbojet::sandbox_controls::{controls, ControlsSettings};
use turbojet::visuals::Json;
fn turns(v: &[f64]) -> usize {
    let (mut n, mut prev) = (0, 0i8);
    for w in v.windows(2) {
        let d = w[1] - w[0];
        let sg = if d > 1e-12 { 1 } else if d < -1e-12 { -1 } else { 0 };
        if sg != 0 { if prev == -1 && sg == 1 { n += 1 } prev = sg; }
    }
    n
}
fn run(s: &ControlsSettings) -> (Vec<FuelPoint>, f64) { let (_, o) = controls(s).unwrap(); let t = o.points[0].tt4; (o.points, t) }
fn proxy(p: &[FuelPoint]) -> usize { turns(&p.iter().map(|x| x.mf_sched - x.mf).collect::<Vec<_>>()) }
fn main() {
    for line in std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap().lines() {
        let s = ControlsSettings::from_json(Json::parse(line).get("controls").unwrap()).unwrap();
        let (p, t0) = run(&s);
        let g: Vec<f64> = p.iter().map(|x| match x.extra { PointExtra::Asym { g, .. } => g, _ => f64::NAN }).collect();
        let (pa, _) = run(&ControlsSettings { gov_lag_two: false, gov_lag: s.gov_lag_att, ..s });
        let (pr, _) = run(&ControlsSettings { gov_lag_two: false, ..s });
        println!("start Tt4 {:.0} redline {:.0} att {:.2} rel {:.2}: two-speed turns g {} proxy {} | rung 47 at att {} | at rel {}",
                 t0, s.redline, s.gov_lag_att, s.gov_lag, turns(&g), proxy(&p), proxy(&pa), proxy(&pr));
    }
}
