// Probe: on the live page (warn), do the inferred-thrust tables a governed run reads stay one-way (thrust, pressure
// ratio and fan speed all rising row by row) where they now hold rows below idle? Governed EPR / N1 runs over fast
// flights, both table placements and speed corrections.
use turbojet::burner_hold::Hold;
use turbojet::sandbox_controls::{build, ControlsSettings, ThrustBy, CONTROL_SHAPES};
use turbojet::mach_axis::SpeedCorr;
use turbojet::inferred_thrust::Table;
fn check(t: &Table) -> (bool, bool) {
    let r = &t.rows;
    let up = |f: &dyn Fn(usize) -> f64| (1..r.len()).all(|k| f(k) > f(k - 1));
    let mono = up(&|k| r[k].thrust) && up(&|k| r[k].epr) && up(&|k| r[k].nu_lp);
    (mono, r.iter().any(|w| w.thrust <= 0.0))
}
fn main() {
    let d = ControlsSettings::defaults();
    let (t_sl, p_sl) = turbojet::atmosphere::standard(0.0);
    let (mut tables, mut with_neg, mut bad) = (0, 0, 0);
    for by in [ThrustBy::Epr, ThrustBy::N1] {
        for sc in [SpeedCorr::None, SpeedCorr::Grid] {
            for (t0, p0) in [(t_sl, p_sl), (d.t0, d.p0), (t_sl + 30.0, p_sl)] {
                for m0 in [1.6, 2.0, 2.5, 3.0, 3.3] {
                    for (k, _) in CONTROL_SHAPES.iter().enumerate() {
                        for from in [700.0, 850.0, 1000.0] {
                            let s = ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: by, speed_corr: sc, t0, p0, m0, shape: k, from, ..d };
                            let Ok(sv) = build(&s) else { continue };
                            for t in [sv.design_table.as_ref(), sv.governor.as_ref().and_then(|g| g.infer.as_ref())].into_iter().flatten() {
                                tables += 1;
                                let (mono, neg) = check(t);
                                if neg { with_neg += 1; }
                                if !mono { bad += 1; println!("NOT ONE-WAY {by:?} {sc:?} T0 {t0} M {m0} shape {k} from {from} (rows below idle: {neg})"); }
                            }
                        }
                    }
                }
            }
        }
    }
    println!("tables {tables}, holding rows below idle {with_neg}, not one-way {bad}");
}
