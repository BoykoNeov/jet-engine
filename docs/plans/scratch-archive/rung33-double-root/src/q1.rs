// Does rung 33's double root reach the product? Fly it (rung 34's spool) vs rung 31/33's matcher, hot Mach 0.3.
use turbojet::engine::{build_turbojet, FlightCondition};
use turbojet::matcher::OffDesignMatcher;
use turbojet::sandbox::{fly, FlySettings, GasModel};
fn msg(e: Box<dyn std::any::Any + Send>) -> String {
    e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default()
}
fn cut(s: &str, n: usize) -> String { s.chars().take(n).collect() }
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let base = FlySettings::defaults();
    println!("design flight t0 {} p0 {} m0 {}", base.t0, base.p0, base.m0);
    for gas in [GasModel::ThermallyPerfect, GasModel::Perfect] {
        let mut s = base;
        s.gas = gas;
        s.t0 = 318.15;
        s.p0 = 101_325.0;
        s.m0 = 0.3;
        let d = s.capture();
        let m = OffDesignMatcher::new(build_turbojet(d.gas.gas(), d.pi_c, d.tt4, d.p0, d.losses()), d.flight(), d.mdot);
        let fl = FlightCondition::new(s.t0, s.p0, s.m0);
        for k in 0..=16 {
            let tt4 = 600.0 + 25.0 * k as f64;
            s.tt4 = tt4;
            let f = match std::panic::catch_unwind(|| fly(&s)) {
                Ok(Ok(o)) => format!("ok nu {:.4} pi_c {:.4} choked {}", o.point.nu, o.point.pi_c, o.point.choked),
                Ok(Err(e)) => format!("REFUSED {}", cut(&e.0, 110)),
                Err(p) => format!("PANIC {}", cut(&msg(p), 110)),
            };
            let r = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| m.match_point(&fl, tt4))) {
                Ok(od) => format!("ok {:?} pi_t {:.4} pi_c {:.4}", od.branch, od.pi_t, od.pi_c),
                Err(p) => format!("FAIL {}", cut(&msg(p), 90)),
            };
            println!("{gas:?} Tt4 {tt4}: FLY {f}\n      R33 {r}");
        }
    }
}
