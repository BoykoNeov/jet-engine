// Against the repaired worktree: the lowest throttle that runs on the page's map (Fly-it engine, 45 °C sea level,
// Mach 0.3), per gas — bisected on "the equilibrium runs" — and the old search's lowest, for the docs.
use turbojet::engine::{build_turbojet, FlightCondition};
use turbojet::sandbox::{FlySettings, GasModel};
use turbojet::spool::SpoolTransient;

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let fl = FlightCondition::new(318.15, 101_325.0, 0.30);
    for g in [GasModel::ThermallyPerfect, GasModel::Perfect] {
        let s = FlySettings { gas: g, ..FlySettings::defaults() };
        let d = s.capture();
        let st = SpoolTransient::new(build_turbojet(d.gas.gas(), d.pi_c, d.tt4, d.p0, d.losses()), d.flight(), d.mdot, s.map.map());
        let new = |t: f64| std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| st.equilibrium(&fl, t, None))).is_ok();
        let old = |t: f64| std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| st.find_equilibrium_nu(|nu| Ok(st.try_instant(&fl, nu, t, None)?.phi)))).is_ok();
        for (name, f) in [("repaired", &new as &dyn Fn(f64) -> bool), ("old search", &old)] {
            // scan up in 5 K to the first that runs, then bisect the step below
            let mut t = 600.0;
            while !f(t) && t < 1000.0 { t += 5.0; }
            let (mut a, mut b) = (t - 5.0, t);
            while b - a > 0.05 { let c = 0.5 * (a + b); if f(c) { b = c } else { a = c } }
            // does every throttle above run, on a 5 K scan to 1000 K?
            let gaps: Vec<f64> = (0..).map(|k| b + 5.0 * k as f64).take_while(|&x| x <= 1000.0).filter(|&x| !f(x)).collect();
            println!("{g:?} {name}: lowest running throttle {b:.2} K (refuses {a:.2}); refusals above it: {gaps:?}");
        }
    }
}
