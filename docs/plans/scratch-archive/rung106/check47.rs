//! The prototype's own check: with the open ramp as the demand, its lagged march is rung 47's (the Controls view's
//! GovLag route) — speeds and applied fuel compared bit for bit at every point, four maps, four lags.
use rung106_probe::*;
use turbojet::sandbox_controls::{build, ControlsSettings, CONTROL_SHAPES, DS};

fn main() {
    OPEN.store(true, std::sync::atomic::Ordering::Relaxed);
    for shape in 0..4 {
        for tau in [0.02, 0.05, 0.2, 0.8] {
            let s = ControlsSettings { shape, redline_on: true, gov_lag_on: true, gov_lag: tau, ..ControlsSettings::defaults() };
            let sv = build(&s).unwrap();
            let pts = sv.march();
            let (recs, stop) = march(&sv, s.redline, tau, DS);
            let n = pts.len().min(recs.len());
            let diff = (0..n).filter(|&k| recs[k].a != pts[k].nu_lp || recs[k].b != pts[k].nu_hp || recs[k].mf != pts[k].mf).count();
            let maxd = (0..n).map(|k| (recs[k].a - pts[k].nu_lp).abs().max((recs[k].b - pts[k].nu_hp).abs())).fold(0.0, f64::max);
            println!("{:10} tau {tau}: rung47 {} pts, prototype {} pts{}; {diff} points differ, max speed diff {maxd:.1e}",
                     CONTROL_SHAPES[shape], pts.len(), recs.len(), stop.map(|m| format!(" STOP {m}")).unwrap_or_default());
        }
    }
    // a start above the redline, slowing down: what rung 47's own march does there, and the prototype's lean count
    for (from, to, rl) in [(1450.0, 1000.0, 1300.0), (1700.0, 900.0, 1480.0)] {
        for tau in [0.02, 0.2, 0.8] {
            let s = ControlsSettings { from, to, redline: rl, ramp: 0.5, redline_on: true, gov_lag_on: true, gov_lag: tau, ..ControlsSettings::defaults() };
            match build(&s) {
                Err(e) => println!("{from}->{to} rl {rl} tau {tau}: page REFUSES: {}", e.0),
                Ok(sv) => {
                    let pts = sv.march();
                    let (recs, stop) = march(&sv, s.redline, tau, DS);
                    let la = recs.iter().filter(|r| r.lean_applied).count();
                    println!("{from}->{to} rl {rl} tau {tau}: rung47 {} of {} pts; prototype (open) {} pts, applied-lean {la}{}", pts.len(), s.expected_points(), recs.len(), stop.map(|m| format!(" STOP {m}")).unwrap_or_default());
                }
            }
        }
    }
}
