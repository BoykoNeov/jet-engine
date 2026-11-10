//! Construction (anchor § 2, D1): the lag's ledger on recorded points, per phase, to set the gate's bar. The attack
//! phase is points 0..k−1 (the last point with req ≥ g before the first recorded crossing k), the release phase k..end;
//! the straddling interval [k−1, k] is in neither (the kink cell). Prints each phase's |trapz − τ·Δg| / (τ·g_c), the
//! crossing count, at step 0.02 and 0.01. No thrust, time or margin.
use rung107_probe::*;
use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::AsymmetricLag;
use turbojet::sandbox_controls::{build, ControlsSettings, CONTROL_SHAPES, DS};

fn trapz(r: &[Rec], f: impl Fn(&Rec) -> f64) -> f64 {
    r.windows(2).map(|w| 0.5 * (w[1].s - w[0].s) * (f(&w[0]) + f(&w[1]))).sum()
}

fn main() {
    let d = ControlsSettings { govern: true, accel_on: true, hold: Hold::Thrust, ..ControlsSettings::defaults() };
    let mut worst = 0.0f64;
    for case in ["sched", "floor"] {
        for shape in 0..4 {
            let s = match case {
                "sched" => ControlsSettings { shape, ..d },
                _ => ControlsSettings { shape, accel_on: false, floor_on: true, ..d },
            };
            if case == "floor" && shape == 3 {
                continue; // refused (start below the floor)
            }
            let sv = build(&s).unwrap();
            for (ta, tr) in [(0.2, 0.2), (0.2, 0.4), (0.2, 0.8), (0.2, 1.6), (0.4, 0.4), (0.4, 1.6)] {
                let mut line = Vec::new();
                for ds in [DS, DS / 2.0] {
                    let (r, stop) = march(&sv, AsymmetricLag::new(ta, tr), false, ds);
                    let Some(k) = r.iter().position(|p| p.g > 0.0 && p.req < p.g) else {
                        line.push(format!("ds {ds}: no crossing ({} pts{})", r.len(), if stop.is_some() { ", STOP" } else { "" }));
                        continue;
                    };
                    let gc = r[k].g;
                    let att = &r[..k];
                    let rel = &r[k..];
                    let ma = (trapz(att, |p| p.req - p.g) - ta * (att.last().unwrap().g - att[0].g)).abs() / (ta * gc);
                    let mr = (trapz(rel, |p| p.g - p.req) - tr * (rel[0].g - rel.last().unwrap().g)).abs() / (tr * gc);
                    let (nrel, nre) = crossings(&r);
                    if ds == DS {
                        worst = worst.max(ma).max(mr);
                    }
                    line.push(format!("ds {ds}: attack {ma:.2e} release {mr:.2e} (releases {nrel} re-attacks {nre}{})",
                                      if stop.is_some() { ", STOP" } else { "" }));
                }
                println!("{case} {:10} att {ta} rel {tr}: {}", CONTROL_SHAPES[shape], line.join(" | "));
            }
        }
    }
    println!("worst at step 0.02 (fraction of tau*g_c): {worst:.2e}");
}
