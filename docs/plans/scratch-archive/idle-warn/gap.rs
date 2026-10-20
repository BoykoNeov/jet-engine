// Probe: governed runs the old code refused because the control's tables could not reach the flight
// ("Make the tables at this flight"), which the live page (idle: warn) now runs. How far is the control's belief
// from the real thrust there, against runs at the same flights that ran before?
use turbojet::burner_hold::Hold;
use turbojet::sandbox_controls::{build, controls, ControlsSettings, ThrustBy, CONTROL_SHAPES};
use turbojet::mach_axis::SpeedCorr;
use turbojet::two_spool::IdleRule;
fn worst(s: &ControlsSettings) -> Option<(f64, f64, usize)> {
    let (sv, o) = controls(s).ok()?;
    let (mut g, mut f) = (0.0f64, f64::MAX);
    for p in &o.points {
        let r = sv.read(p).ok()?;
        if let Some(b) = r.inferred { g = g.max((r.thrust - b).abs()); }
        f = f.min(r.thrust);
    }
    Some((g, f, (o.points.len() == s.expected_points()) as usize))
}
fn main() {
    let d = ControlsSettings::defaults();
    let (t_sl, p_sl) = turbojet::atmosphere::standard(0.0);
    for by in [ThrustBy::Epr, ThrustBy::N1] {
        for sc in [SpeedCorr::None, SpeedCorr::Grid] {
            let (mut newly, mut before) = (Vec::new(), Vec::new());
            for (t0, p0) in [(t_sl, p_sl), (d.t0, d.p0), (t_sl + 30.0, p_sl)] {
                for m0 in [2.0, 2.5, 2.8, 3.0, 3.3] {
                    for (k, _) in CONTROL_SHAPES.iter().enumerate() {
                        for from in [850.0, 1000.0, 1150.0] {
                            let s = ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: by, speed_corr: sc, t0, p0, m0, shape: k, from, ..d };
                            let old = build(&ControlsSettings { idle: IdleRule::Old, ..s });
                            let reach = matches!(&old, Err(e) if e.0.contains("Make the tables at this flight"));
                            let Some(w) = worst(&s) else { continue };
                            if reach { newly.push((w, t0, m0, k, from)); } else if old.is_ok() { before.push(w); }
                        }
                    }
                }
            }
            let mx = |v: &[(f64, f64, usize)]| v.iter().map(|w| w.0).fold(0.0, f64::max);
            let med = |mut v: Vec<f64>| { v.sort_by(f64::total_cmp); if v.is_empty() { f64::NAN } else { v[v.len() / 2] } };
            let nw: Vec<_> = newly.iter().map(|x| x.0).collect();
            println!("{by:?} {sc:?}: ran before {} (worst gap median {:.1}, max {:.1}); reach-refused now run {} (median {:.1}, max {:.1}, finished {})",
                     before.len(), med(before.iter().map(|w| w.0).collect()), mx(&before),
                     nw.len(), med(nw.iter().map(|w| w.0).collect()), mx(&nw), nw.iter().map(|w| w.2).sum::<usize>());
            for ((g, fmin, fin), t0, m0, k, from) in &newly {
                println!("   T0 {t0:.1} M {m0} shape {k} from {from}: worst gap {g:.1}, lowest thrust {fmin:.1}, finished {fin}");
            }
        }
    }
}
