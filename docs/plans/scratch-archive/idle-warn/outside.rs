// Probe (the advisor's check): on the runs `gap.rs` lists, how many points read the control's table OUTSIDE its rows
// (where its lookup carries the end segment on as a straight line), against the runs that reached before?
use turbojet::burner_hold::Hold;
use turbojet::sandbox_controls::{build, controls, ControlsSettings, ThrustBy, CONTROL_SHAPES};
use turbojet::mach_axis::SpeedCorr;
use turbojet::two_spool::IdleRule;
fn count(s: &ControlsSettings) -> Option<(usize, usize, f64)> {
    let (sv, o) = controls(s).ok()?;
    let t = sv.governor.as_ref()?.infer.as_ref()?;
    if t.formula.is_some() { return None; }
    let col = |k: usize| match s.thrust_by { ThrustBy::Epr => t.rows[k].epr, _ => t.rows[k].nu_lp };
    let (lo, hi) = (col(0), col(t.rows.len() - 1));
    let (mut out, mut worst) = (0, 0.0f64);
    for p in &o.points {
        let v = sv.read(p).ok()?.reading?;
        let d = if v < lo { (lo - v) / (hi - lo) } else if v > hi { (v - hi) / (hi - lo) } else { 0.0 };
        if d > 0.0 { out += 1; worst = worst.max(d); }
    }
    Some((out, o.points.len(), worst))
}
fn main() {
    let d = ControlsSettings::defaults();
    let (t_sl, p_sl) = turbojet::atmosphere::standard(0.0);
    for by in [ThrustBy::Epr, ThrustBy::N1] {
        for sc in [SpeedCorr::None, SpeedCorr::Grid] {
            let (mut new, mut old) = ([0usize; 4], [0usize; 4]);
            let (mut wn, mut wo) = (0.0f64, 0.0f64);
            for (t0, p0) in [(t_sl, p_sl), (d.t0, d.p0), (t_sl + 30.0, p_sl)] {
                for m0 in [2.0, 2.5, 2.8, 3.0, 3.3] {
                    for (k, _) in CONTROL_SHAPES.iter().enumerate() {
                        for from in [850.0, 1000.0, 1150.0] {
                            let s = ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: by, speed_corr: sc, t0, p0, m0, shape: k, from, ..d };
                            let b = build(&ControlsSettings { idle: IdleRule::Old, ..s });
                            let reach = matches!(&b, Err(e) if e.0.contains("Make the tables at this flight"));
                            let Some((out, n, w)) = count(&s) else { continue };
                            let (acc, ww) = if reach { (&mut new, &mut wn) } else if b.is_ok() { (&mut old, &mut wo) } else { continue };
                            if reach && w > 0.2 { println!("   {by:?} {sc:?} T0 {t0:.1} M {m0} shape {k} from {from}: {out} of {n} outside, furthest {w:.3}"); }
                            acc[0] += 1; acc[1] += (out > 0) as usize; acc[2] += out; acc[3] += n; *ww = ww.max(w);
                        }
                    }
                }
            }
            println!("{by:?} {sc:?}: past the old reach {} runs, {} with points outside the rows ({} of {} points, furthest {:.3} of the table's span); reached before {} runs, {} with ({} of {}, furthest {:.3})",
                     new[0], new[1], new[2], new[3], wn, old[0], old[1], old[2], old[3], wo);
        }
    }
}
