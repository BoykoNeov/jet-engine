//! The thrust sag the page would show, at finer steps: rung 107's sagcause cells (thrust hold, schedule, default slam,
//! maps flow/press and flat-LP), at steps DS/k, k in {1,4,16,32}. Sag = drawdown of the thrust share after the move's
//! end, read on the page's DS grid (every k-th point); "fine" = the same read on every point.
use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::{AsymmetricLag, FuelLimiters, FuelPoint, PointExtra};
use turbojet::governor as gv;
use turbojet::governor_lag::Demand;
use turbojet::governor_release as gr;
use turbojet::sandbox_controls::{build, ControlsSettings, CONTROL_SHAPES, DS};

fn main() {
    for shape in [0usize, 3] {
        let s = ControlsSettings { govern: true, hold: Hold::Thrust, accel_on: true, shape, ..ControlsSettings::defaults() };
        let sv = build(&s).unwrap();
        let (f0, f1) = (gv::thrust(&sv.start), gv::thrust(&sv.end));
        for (ta, tr) in [(0.005, 0.005), (0.005, 0.1), (0.005, 0.4), (0.005, 1.6), (0.01, 0.4), (0.02, 0.1), (0.02, 0.4),
                         (0.02, 1.6), (0.05, 0.4), (0.1, 0.4), (0.2, 0.4)] {
            let mut row = Vec::new();
            for k in [1usize, 4, 16, 32] {
                let ds = DS / k as f64;
                let k0 = (s.ramp / ds).round_ties_even() as usize;
                let lim = FuelLimiters { lag: Some(AsymmetricLag::new(ta, tr)), ..sv.limiters() };
                let p = gr::march(&sv.core.fuel, &sv.flight, Demand::Governor(sv.governor.as_ref().unwrap()), &lim, None,
                                  (sv.start.nu_lp, sv.start.nu_hp), s.s_end(), ds).0;
                if p.len() <= k0 {
                    row.push(format!("k{k}: STOPPED {}", p.len()));
                    continue;
                }
                let fr: Vec<f64> = p.iter().map(|x: &FuelPoint| (gv::thrust(&sv.reread(x).unwrap().base) - f0) / (f1 - f0)).collect();
                let dd = |step: usize| {
                    let (mut hi, mut dd) = (f64::NEG_INFINITY, 0.0f64);
                    for &v in fr[k0..].iter().step_by(step) { hi = hi.max(v); dd = dd.max(hi - v); }
                    dd
                };
                let g = p.iter().map(|x| if let PointExtra::Asym { g, .. } = x.extra { g } else { 0.0 }).fold(0.0, f64::max);
                row.push(format!("k{k}: end {:.4} sag {:.4} (fine {:.4}) cut {:.3e}", fr[k0], dd(k), dd(1), g));
            }
            println!("{} attack {ta:5} release {tr:5}: {}", CONTROL_SHAPES[shape], row.join(" | "));
        }
    }
}
