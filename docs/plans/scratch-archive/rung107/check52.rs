//! G1: the prototype with the open ramp as the demand IS the Controls view's rung-52 route, bit for bit.
use rung107_probe::*;
use turbojet::fuel_transient::AsymmetricLag;
use turbojet::sandbox_controls::{build, ControlsSettings, CONTROL_SHAPES, DS};

fn main() {
    let d = ControlsSettings::defaults();
    let cases: Vec<(&str, ControlsSettings)> = vec![
        ("sched", ControlsSettings { accel_on: true, ..d }),
        ("floor", ControlsSettings { floor_on: true, ..d }),
        ("both+redline", ControlsSettings { accel_on: true, floor_on: true, redline_on: true, ..d }),
        ("floor decel", ControlsSettings { floor_on: true, from: 1400.0, to: 1000.0, ..d }),
    ];
    for (name, c) in &cases {
        for shape in 0..4 {
            for (ta, tr) in [(0.02, 0.1), (0.02, 0.8), (0.1, 0.1), (0.2, 0.05)] {
                let s = ControlsSettings { shape, release_on: true, tau_att: ta, tau_rel: tr, ..*c };
                let sv = match build(&s) {
                    Ok(v) => v,
                    Err(e) => {
                        println!("{name} {}: REFUSED {}", CONTROL_SHAPES[shape], e.0);
                        continue;
                    }
                };
                let page = sv.march();
                let (recs, stop) = march(&sv, AsymmetricLag::new(ta, tr), true, DS);
                let n = page.len().min(recs.len());
                let diff = (0..n)
                    .filter(|&k| !(page[k].nu_lp == recs[k].a && page[k].nu_hp == recs[k].b && page[k].mf == recs[k].mf))
                    .count();
                println!("{name:13} {:10} att {ta:4} rel {tr:4}: page {} proto {} pts, {diff} differ{}", CONTROL_SHAPES[shape],
                         page.len(), recs.len(), stop.map(|m| format!(" (proto STOP {m})")).unwrap_or_default());
            }
        }
    }
}
