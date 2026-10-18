// Rung 100 pre-anchor, unscored flights: two fan-speed approximations against the truth.
//  A (the user's): HP side read off the table AT the fan speed, LP turbine re-solved against p0.
//  B (LP-shaft balance on the table's gas-generator line): find the table row x whose HP side, with its LP turbine
//    re-solved against p0, gives the LP power the table needs at the fan speed (pt_lp at that fan speed); read push and
//    airflow at x.
#[path = "common.rs"]
mod common;
use common::*;
use turbojet::components::Nozzle;
use turbojet::engine::FlightCondition;
use turbojet::gas::{FlowState, Gas};
use turbojet::governor::thrust;
use turbojet::inferred_thrust::face;
use turbojet::sandbox_controls::{ControlsSettings, CONTROL_SHAPES};
use turbojet::two_spool_transient::Instant2;
fn push(gas: &Gas, f: f64, pt5: f64, p0: f64, a8: f64, pi_n: f64) -> f64 {
    let e = Nozzle::convergent(p0, pi_n).try_apply(&FlowState { tt: 1000.0, pt: pt5, mdot: 1.0, far: f }, gas).unwrap();
    a8 * (e.p9 * (1.0 + gas.gamma_t_at(e.t9, f) * e.m9 * e.m9) - p0)
}
fn f45_of(i: &Instant2) -> f64 {
    let c = &i.close;
    match c.cooling { Some(cool) => { let hpc = (1.0 - c.bleed.unwrap_or(0.0)) * c.mdot_face.unwrap(); c.f * c.mdot_air / (c.mdot_air + cool.total() * hpc) } None => c.f }
}
fn main() {
    let d = ControlsSettings::defaults();
    let v = variants();
    for (name, s) in [("plain", d), ("cool .05/.05", v[6].1), ("gen HP power", v[3].1), ("gen LP share", v[2].1), ("cust share", v[4].1), ("lever bleed", v[8].1)] {
        for shape in 0..4 {
            let sv = machine(&ControlsSettings { shape, ..s });
            let core = &sv.core.fuel.inner;
            let base = &core.inner.base;
            let fd = s.design_flight();
            let (tt2d, pt2d, _) = st2(&sv, &fd);
            let rows: Vec<Instant2> = (0..=105).filter_map(|k| core.try_equilibrium(&fd, 550.0 + 10.0 * k as f64, None).ok().map(|x| x.0)).collect();
            let by_nu = |v: f64, y: &dyn Fn(&Instant2) -> f64| { let k = rows.windows(2).position(|w| v <= w[1].nu_lp).unwrap_or(rows.len() - 2); let (a, b) = (&rows[k], &rows[k + 1]); y(a) + (y(b) - y(a)) * (v - a.nu_lp) / (b.nu_lp - a.nu_lp) };
            let by_t = |x: f64, y: &dyn Fn(&Instant2) -> f64| { let k = rows.windows(2).position(|w| x <= w[1].tt4).unwrap_or(rows.len() - 2); let (a, b) = (&rows[k], &rows[k + 1]); y(a) + (y(b) - y(a)) * (x - a.tt4) / (b.tt4 - a.tt4) };
            for &(t0, p0, m0) in &[(250.0, 50_000.0, 0.1), (250.0, 50_000.0, 0.3), (250.0, 50_000.0, 0.4), (288.15, 101_325.0, 0.1), (288.15, 101_325.0, 0.3)] {
                let fl = FlightCondition::new(t0, p0, m0);
                let (tt2, pt2, v0) = st2(&sv, &fl);
                let (th, de) = (tt2 / tt2d, pt2 / pt2d);
                for t4 in [650.0, 700.0, 750.0, 800.0, 900.0] {
                    let Ok((i, _, _)) = core.try_equilibrium(&fl, t4, None) else { continue };
                    if choked(&i) { continue }
                    let gas = i.close.gas(&core.inner);
                    let ft = thrust(&i);
                    let nd = i.nu_lp / th.sqrt();
                    // the LP turbine at row-parameter x (design Tt4), re-solved against p0: (push, LP power, airflow)
                    let at = |x: f64| -> Option<(f64, f64, f64)> {
                        let pt45 = de * by_t(x, &|w| w.pi_hpt * w.close.pt4);
                        let tt45 = th * by_t(x, &|w| w.tt45);
                        let tt5d = th * by_t(x, &|w| w.tt5);
                        let f45 = by_t(x, &f45_of);
                        let (pi, _, tt5) = turbojet::unchoked_lp::try_lp_turbine(base, gas, tt45, f45, by_t(x, &|w| w.eta_lpt), pt45, p0).ok()?;
                        let p = push(gas, f45, pi * pt45, p0, base.a8, base.pi_n);
                        Some((p, by_t(x, &|w| w.pt_lp) * (tt45 - tt5) / (tt45 - tt5d), by_t(x, &|w| face(w)) * de / th.sqrt()))
                    };
                    let x_nu = by_nu(nd, &|w| w.tt4);
                    let need = by_nu(nd, &|w| w.pt_lp);
                    let fa = at(x_nu).map(|(p, _, m)| p - v0 * m);
                    // B: bisect x in [x_nu, 1600] on power(x) - need
                    let fb = (|| {
                        let g = |x: f64| at(x).map(|(_, pw, _)| pw - need);
                        let (mut lo, mut hi) = (x_nu, x_nu);
                        if g(lo)? >= 0.0 { return at(lo).map(|(p, _, m)| (p - v0 * m, lo)); }
                        loop { hi += 10.0; if hi > 1590.0 { return None } if g(hi)? >= 0.0 { break } lo = hi; }
                        for _ in 0..60 { let m = 0.5 * (lo + hi); if g(m)? >= 0.0 { hi = m } else { lo = m } }
                        at(hi).map(|(p, _, m)| (p - v0 * m, hi))
                    })();
                    let pc = |f: Option<f64>| f.map_or("  refused".into(), |f| format!("{:+8.3}%", 100.0 * (ft - f) / ft));
                    println!("{name:13} {:10} {t0:.0}K M{m0:<4} T4 {t4:<4} | A {} | B {} | x_B {:6.1} vs true Tt4·k {:6.1} (x_ν {:6.1})",
                             CONTROL_SHAPES[shape], pc(fa), pc(fb.map(|q| q.0)), fb.map_or(f64::NAN, |q| q.1), t4 / th, x_nu);
                }
            }
        }
    }
}
