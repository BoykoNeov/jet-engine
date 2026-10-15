// Rung 89's sandbox knob: the page's step (DS 0.02) against 0.005 for the valve CLOSING at its own speed (opening 0.05), per tau_close (scratch).
use turbojet::sandbox_controls::*;

fn main() {
    let d = ControlsSettings::defaults();
    let v = ControlsSettings { bleed_mode: BleedMode::Valve, valve_lag_on: true, valve_tau: 0.05, valve_close_own: true, ..d };
    let cases = [
        ("phi valve", ControlsSettings { lever: LeverChoice::Bleed, ..v }),
        ("inc valve + stators", ControlsSettings { lever: LeverChoice::Both, valve_ref: FloorRef::Incidence, ..v }),
        ("phi valve + stators + redline", ControlsSettings { lever: LeverChoice::Both, redline_on: true, ..v }),
        ("inc valve + stators, fast ramp 0.1", ControlsSettings { lever: LeverChoice::Both, valve_ref: FloorRef::Incidence, ramp: 0.1, ..v }),
        ("inc valve + stators, flat-lp", ControlsSettings { shape: 3, lever: LeverChoice::Both, valve_ref: FloorRef::Incidence, ..v }),
    ];
    for (name, c) in cases {
        for tau in [0.01, 0.015, 0.02, 0.03, 0.05, 0.1, 0.3] {
            let s = ControlsSettings { valve_tau_close: tau, ..c };
            let Ok(sv) = build(&s) else { println!("{name:38} tau={tau:<6} refused by the precheck"); continue };
            let coarse = sv.march();
            let sched = s.schedule(sv.mf_lo, sv.mf_hi);
            let fine = sv.core.fuel.integrate_fuel(&sv.flight, sched, (sv.start.nu_lp, sv.start.nu_hp), s.s_end(), 0.005,
                                                   &sv.limiters());
            let (mut dphi, mut db, mut dnu) = (0.0f64, 0.0f64, 0.0f64);
            let mut n = 0;
            for (i, p) in coarse.iter().enumerate() {
                let Some(q) = fine.get(4 * i) else { break };
                assert!((p.s - q.s).abs() < 1e-9, "{} {}", p.s, q.s);
                dphi = dphi.max((p.phi_lp - q.phi_lp).abs());
                dnu = dnu.max((p.nu_lp - q.nu_lp).abs());
                let (bp, bq) = match (p.extra, q.extra) {
                    (turbojet::fuel_transient::PointExtra::Valve { b: a, .. }, turbojet::fuel_transient::PointExtra::Valve { b, .. }) => (a, b),
                    _ => panic!(),
                };
                db = db.max((bp - bq).abs());
                n += 1;
            }
            let bmax = coarse.iter().map(|p| match p.extra { turbojet::fuel_transient::PointExtra::Valve { b, .. } => b, _ => 0.0 }).fold(0.0, f64::max);
            println!("{name:38} tau={tau:<6} ds/tau={:.2} pts {}/{} (fine {}) max|dphi|={dphi:.2e} max|db|={db:.2e} (b peak {bmax:.4}) max|dnuL|={dnu:.2e}",
                     DS / tau, coarse.len(), s.expected_points(), fine.len(), );
            let _ = n;
        }
    }
}
