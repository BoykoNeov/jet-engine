// Rung 105 probe P4 — sign changes of rung 33's (★★) residual on a fine pi_t grid, one shaft, hot Mach 0.3, 650-900 K:
// pin vs reaction 0.3, perfect and thermally perfect gas. Count the roots.
use turbojet::engine::{build_turbojet, FlightCondition};
use turbojet::gas::Gas;
use turbojet::hp_vane::HpVane;
use turbojet::matcher::OffDesignMatcher;
use turbojet::components::ram_recovery;
use turbojet::sandbox::Settings;
use turbojet::sandbox_blades::BladeGas;
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let d = Settings::defaults();
    let fd = FlightCondition::new(250.0, 50_000.0, 0.85);
    for (gn, gas) in [("CPG", BladeGas::Perfect.gas()), ("TPG", Gas::thermally_perfect())] {
        let e = build_turbojet(gas, d.pi_c, d.tt4, fd.p0, { let mut l = d.losses(); l.nozzle_convergent = true; l.p_exit = None; l });
        let m = OffDesignMatcher::new(e, fd, 1.0);
        for (fname, fl) in [("hot M.3", FlightCondition::new(318.15, 101_325.0, 0.3)), ("design", fd)] {
            let pi_d = m.pi_d_max * ram_recovery(fl.m0);
            let (s0, _) = m.freestream_for(&fl);
            let (tt2, pt2) = (s0.tt, pi_d * s0.pt);
            for law in [HpVane::Pin, HpVane::Rows { reaction: 0.3 }] {
                m.set_hp_vane(law);
                for tt4 in [600.0, 650.0, 700.0, 750.0, 800.0, 850.0, 900.0] {
                    let mut prev: Option<(f64, f64)> = None;
                    let mut roots = Vec::new();
                    let (mut first, mut last) = (f64::NAN, f64::NAN);
                    for i in 0..=400 {
                        let y = 0.15 + (0.9995 - 0.15) * i as f64 / 400.0;
                        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| m.try_subsonic_operating(tt4, tt2, pt2, y, fl.p0)));
                        if let Ok(Ok(op)) = r {
                            if first.is_nan() { first = y; }
                            last = y;
                            if let Some((py, pr)) = prev { if pr * op.resid < 0.0 { roots.push(format!("{:.3}(m9 {:.3})", 0.5 * (py + y), op.m9)); } }
                            prev = Some((y, op.resid));
                        } else { prev = None; }
                    }
                    println!("{gn} {fname:<7} {law:?} Tt4 {tt4}: evaluable {first:.3}..{last:.3}, sign changes {} {:?}", roots.len(), roots);
                }
            }
        }
    }
}
