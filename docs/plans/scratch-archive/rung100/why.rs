// Why B misses: compare the true unchoked engine with the choked table row at the SAME corrected burner temperature.
#[path = "common.rs"]
mod common;
use common::*;
use turbojet::components::Nozzle;
use turbojet::engine::FlightCondition;
use turbojet::gas::{FlowState, Gas};
use turbojet::governor::thrust;
use turbojet::sandbox_controls::ControlsSettings;
fn push(gas: &Gas, f: f64, pt5: f64, p0: f64, a8: f64, pi_n: f64) -> f64 {
    let e = Nozzle::convergent(p0, pi_n).try_apply(&FlowState { tt: 1000.0, pt: pt5, mdot: 1.0, far: f }, gas).unwrap();
    a8 * (e.p9 * (1.0 + gas.gamma_t_at(e.t9, f) * e.m9 * e.m9) - p0)
}
fn main() {
    let s = ControlsSettings::defaults();
    let sv = machine(&s);
    let core = &sv.core.fuel.inner;
    let base = &core.inner.base;
    let fd = s.design_flight();
    let (tt2d, pt2d, _) = st2(&sv, &fd);
    println!("plain, flow/press map. True unchoked point vs the choked design-flight row at the same corrected burner temperature (carried).");
    println!("{:>5} {:>4} | {:>8} {:>8} | {:>8} {:>8} | {:>8} {:>8} | {:>8} {:>8} | {:>9}", "Mach", "T4", "fan true", "fan row", "pLPC tru", "pLPC row", "pt45 tru", "pt45 row", "πLPT tru", "πLPT row", "F̂(true HP)");
    for m0 in [0.1, 0.4] {
        let fl = FlightCondition::new(250.0, 50_000.0, m0);
        let (tt2, pt2, v0) = st2(&sv, &fl);
        let (th, de) = (tt2 / tt2d, pt2 / pt2d);
        for t4 in [650.0, 700.0] {
            let (i, _, _) = core.try_equilibrium(&fl, t4, None).unwrap();
            let (r, _, _) = core.try_equilibrium(&fd, t4 / th, None).unwrap();
            let gas = i.close.gas(&core.inner);
            // closure: LP turbine re-solved from the TRUE HP side reproduces the truth?
            let (pi, _, _) = turbojet::unchoked_lp::try_lp_turbine(base, gas, i.tt45, i.close.f, i.eta_lpt, i.pi_hpt * i.close.pt4, fl.p0).unwrap();
            let fhat = push(gas, i.close.f, pi * i.pi_hpt * i.close.pt4, fl.p0, base.a8, base.pi_n) - v0 * i.close.mdot_air;
            println!("{m0:>5} {t4:>4} | {:8.4} {:8.4} | {:8.4} {:8.4} | {:8.4} {:8.4} | {:8.4} {:8.4} | {:+.2e}",
                     i.nu_lp, r.nu_lp * th.sqrt(), i.close.pi_lpc, r.close.pi_lpc,
                     i.pi_hpt * i.close.pt4 / pt2, r.pi_hpt * r.close.pt4 / pt2d, i.pi_lpt, r.pi_lpt, (thrust(&i) - fhat) / thrust(&i));
        }
    }
}
