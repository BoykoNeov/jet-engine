// Against the trial worktree (rung 34's step fix): (1) the page map's power balance on a fine speed grid at the
// throttles that still refuse — is there ANY accelerating speed (an equilibrium) or none (genuinely below the
// lowest running throttle)? (2) flat map vs rung 33's working root at 675/700 K (the skipped cells of the rung-33 gate).
use turbojet::engine::{build_turbojet, FlightCondition};
use turbojet::map::ComponentMap;
use turbojet::matcher::OffDesignMatcher;
use turbojet::sandbox::{FlySettings, GasModel};
use turbojet::spool::SpoolTransient;

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let fl = FlightCondition::new(318.15, 101_325.0, 0.30);
    for g in [GasModel::ThermallyPerfect, GasModel::Perfect] {
        let s = FlySettings { gas: g, ..FlySettings::defaults() };
        let d = s.capture();
        let mk = || build_turbojet(d.gas.gas(), d.pi_c, d.tt4, d.p0, d.losses());
        let page = SpoolTransient::new(mk(), d.flight(), d.mdot, s.map.map());
        for tt4 in [650.0, 675.0, 685.0, 690.0, 695.0, 700.0] {
            let (mut best, mut at, mut nerr) = (f64::NEG_INFINITY, 0.0, 0);
            for k in 0..=200 {
                let nu = 0.30 + 0.0025 * k as f64;
                match page.try_instant(&fl, nu, tt4, None) {
                    Ok(i) => if i.phi > best { best = i.phi; at = nu },
                    Err(_) => nerr += 1,
                }
            }
            println!("{g:?} page map Tt4 {tt4}: max Phi {best:+.5} at nu {at:.4} ({nerr} speeds refused)");
        }
        let flat = SpoolTransient::new(mk(), d.flight(), d.mdot, ComponentMap::flat());
        let r33 = OffDesignMatcher::new(mk(), d.flight(), d.mdot);
        for tt4 in [650.0, 660.0, 675.0, 700.0] {
            let a = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| flat.equilibrium(&fl, tt4, None)));
            let b = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| r33.match_point(&fl, tt4)));
            match (a, b) {
                (Ok(a), Ok(b)) => println!("{g:?} flat Tt4 {tt4}: r34 pi_t {:.12} pi_c {:.10} nu {:.4} | r33 pi_t {:.12} pi_c {:.10} | dev {:.2e}",
                                           a.pi_t, a.pi_c, a.nu, b.pi_t, b.pi_c, (a.pi_t - b.pi_t).abs().max((a.pi_c - b.pi_c).abs() / b.pi_c)),
                (a, b) => println!("{g:?} flat Tt4 {tt4}: r34 {} r33 {}", if a.is_ok() { "ok" } else { "FAIL" }, if b.is_ok() { "ok" } else { "FAIL" }),
            }
        }
    }
}
