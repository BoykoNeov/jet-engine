// Rung 100 pre-anchor bars. (a) The push identity's ABSOLUTE worst (N per kg/s) and smallest |thrust| at the points,
// every variant/map, flights incl. deep unchoke. (b) B at CHOKED readings vs rung 99's formula column (a dense
// design table read at the fan speed), relative to thrust: what a choked fan-formula run moves by.
#[path = "common.rs"]
mod common;
use common::*;
use turbojet::components::Nozzle;
use turbojet::engine::FlightCondition;
use turbojet::gas::{FlowState, Gas};
use turbojet::governor::thrust;
use turbojet::inferred_thrust::{epr, face};
use turbojet::sandbox_controls::ControlsSettings;
use turbojet::two_spool_transient::Instant2;
fn push(gas: &Gas, f: f64, pt5: f64, p0: f64, a8: f64, pi_n: f64) -> f64 {
    let e = Nozzle::convergent(p0, pi_n).try_apply(&FlowState { tt: 1000.0, pt: pt5, mdot: 1.0, far: f }, gas).unwrap();
    a8 * (e.p9 * (1.0 + gas.gamma_t_at(e.t9, f) * e.m9 * e.m9) - p0)
}
fn f45_of(i: &Instant2) -> f64 { let c = &i.close; match c.cooling { Some(cool) => { let hpc = (1.0 - c.bleed.unwrap_or(0.0)) * c.mdot_face.unwrap(); c.f * c.mdot_air / (c.mdot_air + cool.total() * hpc) } None => c.f } }
fn main() {
    let flights = [(250.0, 50_000.0, 0.05), (250.0, 50_000.0, 0.5), (250.0, 50_000.0, 1.4), (288.15, 101_325.0, 0.05), (288.15, 101_325.0, 0.4), (318.15, 101_325.0, 0.1)];
    let (mut wa, mut fmin) = (0.0f64, f64::INFINITY);
    for (_, s) in variants() { for shape in 0..4 {
        let sv = machine(&ControlsSettings { shape, ..s });
        let base = &sv.core.fuel.inner.inner.base;
        for &(t0, p0, m0) in &flights { let fl = FlightCondition::new(t0, p0, m0); let (_, pt2, v0) = st2(&sv, &fl);
            for k in 0..=20 { let t = 550.0 + 50.0 * k as f64;
                let Ok((i, _, _)) = sv.core.fuel.inner.try_equilibrium(&fl, t, None) else { continue };
                let gas = i.close.gas(&sv.core.fuel.inner.inner);
                let r = (thrust(&i) - (push(gas, f45_of(&i), epr(&i, pt2) * pt2, p0, base.a8, base.pi_n) - v0 * face(&i))).abs();
                wa = wa.max(r); fmin = fmin.min(thrust(&i).abs());
            } } } }
    println!("(a) push identity: absolute worst {wa:.3e} N per kg/s; smallest |thrust| seen {fmin:.3}");
    // (b)
    let v = variants();
    let mut wb = 0.0f64;
    for (name, s) in [("plain", v[0].1), ("cool", v[6].1), ("cust", v[4].1), ("gen LP share", v[2].1), ("lever bleed", v[8].1), ("gen HP power", v[3].1)] {
        let mut wv = 0.0f64;
        for shape in 0..4 {
            let sv = machine(&ControlsSettings { shape, ..s });
            let core = &sv.core.fuel.inner; let base = &core.inner.base;
            let fd = s.design_flight(); let (tt2d, pt2d, v0d) = st2(&sv, &fd);
            let rows: Vec<Instant2> = (0..=13).filter_map(|k| core.try_equilibrium(&fd, 800.0 + 50.0 * k as f64, None).ok().map(|x| x.0)).collect();
            let by_nu = |v: f64, y: &dyn Fn(&Instant2) -> f64| { let k = rows.windows(2).position(|w| v <= w[1].nu_lp).unwrap_or(rows.len() - 2); let (a, b) = (&rows[k], &rows[k + 1]); y(a) + (y(b) - y(a)) * (v - a.nu_lp) / (b.nu_lp - a.nu_lp) };
            let by_t = |x: f64, y: &dyn Fn(&Instant2) -> f64| { let k = rows.windows(2).position(|w| x <= w[1].tt4).unwrap_or(rows.len() - 2); let (a, b) = (&rows[k], &rows[k + 1]); y(a) + (y(b) - y(a)) * (x - a.tt4) / (b.tt4 - a.tt4) };
            for &(t0, p0, m0) in &[(250.0, 50_000.0, 0.5), (250.0, 50_000.0, 1.4), (288.15, 101_325.0, 0.85)] {
                let fl = FlightCondition::new(t0, p0, m0); let (tt2, pt2, v0) = st2(&sv, &fl); let (th, de) = (tt2 / tt2d, pt2 / pt2d);
                for t4 in [1000.0, 1100.0, 1200.0, 1300.0] {
                    let Ok((i, _, _)) = core.try_equilibrium(&fl, t4, None) else { continue }; if !choked(&i) { continue }
                    let gas = i.close.gas(&core.inner); let nd = i.nu_lp / th.sqrt();
                    let mh = by_nu(nd, &|w| face(w)) * de / th.sqrt();
                    let f99 = de * by_nu(nd, &|w| thrust(w)) + de * v0d * by_nu(nd, &|w| face(w)) - v0 * mh + base.a8 * (de * fd.p0 - p0);
                    let at = |x: f64| -> Option<(f64, f64, f64)> {
                        let pt45 = de * by_t(x, &|w| w.pi_hpt * w.close.pt4); let tt45 = th * by_t(x, &|w| w.tt45); let tt5d = th * by_t(x, &|w| w.tt5); let f45 = by_t(x, &f45_of);
                        let (pi, _, tt5) = turbojet::unchoked_lp::try_lp_turbine(base, gas, tt45, f45, by_t(x, &|w| w.eta_lpt), pt45, p0).ok()?;
                        Some((push(gas, f45, pi * pt45, p0, base.a8, base.pi_n), by_t(x, &|w| w.pt_lp) * (tt45 - tt5) / (tt45 - tt5d), by_t(x, &|w| face(w)) * de / th.sqrt()))
                    };
                    let x_nu = by_nu(nd, &|w| w.tt4); let (p, _, m) = at(x_nu).unwrap();
                    let fb = p - v0 * m;   // the shortcut: pinned at x_nu
                    let r = ((fb - f99) / thrust(&i)).abs(); wv = wv.max(r); wb = wb.max(r);
                }
            }
        }
        println!("(b) {name:13} choked: |F̂_B(x_ν) − F̂_99| / F worst {wv:.3e}");
    }
    println!("(b) all: {wb:.3e}");
}
