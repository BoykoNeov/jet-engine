//! H1's cells before freezing: at each 700 K start end (sea level +40 K and ISA, M 0.85, four maps) — the end's branch
//! and the pin-to-root relative pi gap (G4's kind of print). No speed, thrust, temperature or margin.
use turbojet::atmosphere::standard;
use turbojet::components::{try_choked_mfp, Nozzle};
use turbojet::gas::FlowState;
use turbojet::sandbox_controls::{build, ControlsSettings};

fn main() {
    let d = ControlsSettings::defaults();
    for (z, dt) in [(0.0, 40.0), (0.0, 0.0), (0.0, 20.0), (2000.0, 40.0)] {
        let (t, p) = standard(z);
        for shape in 0..4 {
            let s = ControlsSettings { shape, t0: t + dt, p0: p, from: 700.0, to: 1400.0, ..d };
            let sv = match build(&s) { Ok(b) => b, Err(e) => { println!("z {z} dT {dt:+} shape {shape}: REFUSED {}", e.0.chars().take(90).collect::<String>()); continue } };
            let (fl, core) = (sv.flight, &sv.core.fuel.inner);
            let base = &core.inner.base;
            let i = &sv.start;
            let (c, g) = (&i.close, i.close.gas(&core.inner));
            let f = c.f;
            let pt45 = i.pi_hpt * c.pt4;
            let supply = base.a45 * pt45 * try_choked_mfp(g, i.tt45, f).unwrap() / i.tt45.sqrt();
            let resid = |pi: f64| -> f64 {
                let (_, tt5) = base.try_tau_of(g, i.tt45, f, pi, i.eta_lpt).unwrap();
                match Nozzle::convergent(fl.p0, base.pi_n).try_apply(&FlowState { tt: tt5, pt: pi * pt45, mdot: 1.0, far: f }, g) {
                    Ok(e) => supply - base.a8 * (e.p9 / (g.r_t_at(f) * e.t9)) * e.v9, Err(_) => supply }
            };
            let (mut a, mut b) = (i.pi_lpt, 0.9995);
            let gap = if resid(a) <= 0.0 { 0.0 } else {
                for _ in 0..200 { let m = 0.5 * (a + b); if resid(m) > 0.0 { a = m } else { b = m } if b - a < 1e-13 { break } }
                0.5 * (a + b) / i.pi_lpt - 1.0 };
            println!("z {z:>5} dT {dt:+} shape {shape}: start end {:?}, pin-to-root pi gap {gap:.2e}", i.branch);
        }
    }
}
