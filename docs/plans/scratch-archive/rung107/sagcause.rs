//! Rung 107, off protocol (reported, not scored): which edge of the lag makes the sag? The default thrust-hold slam with
//! the schedule, the march called directly (past the page's floors), attack and release varied separately; step 0.02 and
//! 0.005 (the fast lags are rough at 0.02). Prints the thrust share at the move's end, the sag (drawdown after it), the
//! deepest cut.
use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::{AsymmetricLag, FuelLimiters, FuelPoint, PointExtra};
use turbojet::governor as gv;
use turbojet::governor_lag::Demand;
use turbojet::governor_release as gr;
use turbojet::sandbox_controls::{build, ControlsSettings, CONTROL_SHAPES};

fn main() {
    for shape in [0usize, 3] {
        let s = ControlsSettings { govern: true, hold: Hold::Thrust, accel_on: true, shape, ..ControlsSettings::defaults() };
        let sv = build(&s).unwrap();
        let (f0, f1) = (gv::thrust(&sv.start), gv::thrust(&sv.end));
        for ds in [0.02, 0.005] {
            let k0 = (s.ramp / ds).round_ties_even() as usize;
            let read = |p: &[FuelPoint]| {
                if p.len() <= k0 { return format!("STOPPED at {} pts", p.len()); }
                let fr: Vec<f64> = p.iter().map(|x| (gv::thrust(&sv.reread(x).unwrap().base) - f0) / (f1 - f0)).collect();
                let (mut hi, mut dd) = (f64::NEG_INFINITY, 0.0f64);
                for &v in &fr[k0..] { hi = hi.max(v); dd = dd.max(hi - v); }
                let g = p.iter().map(|x| if let PointExtra::Asym { g, .. } = x.extra { g } else { 0.0 }).fold(0.0, f64::max);
                format!("end {:.4} sag {:.4} cut {:.3e} ({} pts)", fr[k0], dd, g, p.len())
            };
            let inst = gv::march(&sv.core.fuel, &sv.flight, sv.governor.as_ref().unwrap(), &sv.limiters(), None,
                                 (sv.start.nu_lp, sv.start.nu_hp), s.s_end(), ds).0;
            println!("{} ds {ds} instant: {}", CONTROL_SHAPES[shape], read(&inst));
            for (ta, tr) in [(0.005, 0.005), (0.005, 0.1), (0.005, 0.4), (0.005, 1.6), (0.02, 0.4), (0.05, 0.4), (0.1, 0.4),
                             (0.2, 0.005), (0.2, 0.02), (0.2, 0.1), (0.2, 0.4), (0.2, 1.6), (0.4, 0.005), (0.4, 0.4)] {
                let lim = FuelLimiters { lag: Some(AsymmetricLag::new(ta, tr)), ..sv.limiters() };
                let p = gr::march(&sv.core.fuel, &sv.flight, Demand::Governor(sv.governor.as_ref().unwrap()), &lim, None,
                                  (sv.start.nu_lp, sv.start.nu_hp), s.s_end(), ds).0;
                println!("{} ds {ds} attack {ta:5} release {tr:5}: {}", CONTROL_SHAPES[shape], read(&p));
            }
        }
    }
}
