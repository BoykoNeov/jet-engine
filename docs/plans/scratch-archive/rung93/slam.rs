//! Rung 93 anchor § 0 G3 — the sandbox Controls march (fuel-metered, rung 43's), EXISTING code: the
//! Tt4 overshoot and Tt3 along it, with and without turbine cooling. No metal code.
use turbojet::sandbox_controls::{self, ControlsSettings, CONTROL_SHAPES};

fn main() {
    for shape in 0..CONTROL_SHAPES.len() {
        for (rho, ramp) in [(1.0, 0.5), (1.0, 0.1), (0.5, 0.5), (2.0, 0.5)] {
            for (e1, e2) in [(0.0, 0.0), (0.05, 0.03)] {
                let mut s = ControlsSettings::defaults();
                s.shape = shape; s.rho = rho; s.ramp = ramp; s.cool_vane = e1; s.cool_blade = e2;
                let sv = match sandbox_controls::build(&s) { Ok(v) => v, Err(e) => { println!("{shape} build refused: {e:?}"); continue; } };
                let pts = sv.march();
                let n = pts.len();
                let mut rows = Vec::new();
                for p in &pts {
                    let i = sv.reread(p).expect("reread");
                    rows.push((p.s, p.tt4, i.base.close.tt3, i.base.close.f));
                }
                let (imax, pk) = rows.iter().enumerate().max_by(|a, b| a.1.1.partial_cmp(&b.1.1).unwrap()).unwrap();
                let end = rows[n - 1];
                let start = rows[0];
                println!("{:<10} rho {rho} ramp {ramp} cool ({e1},{e2}) n {n}: start Tt4 {:.1} Tt3 {:.1} | peak Tt4 {:.1} at s {:.2} (Tt3 there {:.1}) | end Tt4 {:.1} Tt3 {:.1} | overshoot {:.1} K, Tt3 lag {:.1} K",
                         CONTROL_SHAPES[shape], start.1, start.2, pk.1, pk.0, pk.2, end.1, end.2, pk.1 - end.1, end.2 - pk.2);
                let _ = imax;
            }
        }
    }
}
