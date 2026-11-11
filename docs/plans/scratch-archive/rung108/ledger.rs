//! The ledger (D1), per phase, on recorded points: attack sum_{0..k-1} trapezoids of (r-g) vs tau_att*g_c, release
//! sum_{k..end} of (g-r) vs tau_rel*(g_c - g_end); g_c = g at k, the first recorded r <= g after g > 0. Relative misses.
//! Governed thrust law (fine step 0.005) and open (0.02), red case, design flight, four maps; also the step halved.
use rung108_probe::*;
use turbojet::burner_hold::Hold;
use turbojet::sandbox_controls::{build, ControlsSettings, DS};
fn ledger(p: &[P], ta: f64, tr: f64) -> Option<(f64, f64)> {
    let k = p.iter().enumerate().position(|(i, x)| i > 0 && p[i - 1].g > 0.0 && x.req <= x.g)?;
    let trap = |i: usize, f: &dyn Fn(&P) -> f64| 0.5 * (f(&p[i]) + f(&p[i + 1])) * (p[i + 1].s - p[i].s);
    let att: f64 = (0..k - 1).map(|i| trap(i, &|x| x.req - x.g)).sum();
    let rel: f64 = (k..p.len() - 1).map(|i| trap(i, &|x| x.g - x.req)).sum();
    let gc = p[k].g; let ge = p.last()?.g;
    Some(((att - ta * gc).abs() / (ta * gc), (rel - tr * (gc - ge)).abs() / (tr * gc)))
}
fn main() {
    for gov in [true, false] {
        for shape in 0..4 {
            let s = ControlsSettings { shape, redline_on: true, gov_lag_on: true, gov_lag: 0.2, govern: gov, hold: Hold::Thrust, ..ControlsSettings::defaults() };
            let Ok(sv) = build(&s) else { println!("REFUSED"); continue };
            for (ta, tr) in [(0.02, 0.2), (0.02, 0.8), (0.05, 0.4), (0.1, 1.6), (0.2, 0.2)] {
                let ds = if gov { DS / 4.0 } else { DS };
                let l1 = ledger(&march(&sv, ta, tr, ds), ta, tr);
                let l2 = ledger(&march(&sv, ta, tr, ds / 2.0), ta, tr);
                println!("{} map{shape} att={ta} rel={tr} miss(att,rel) step={ds}: {:?}  step/2: {:?}", if gov { "gov" } else { "open" }, l1, l2);
            }
        }
    }
}
