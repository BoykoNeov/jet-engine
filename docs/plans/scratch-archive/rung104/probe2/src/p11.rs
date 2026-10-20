// Rung 104 probe P11 — the C3 trigger fired (two rows on the equilibrium gas, up to 151x the ellipse): COUNT the calls
// per point before changing anything. Rung 38's engine, equilibrium gas, its design flight, each law.
use turbojet::engine::FlightCondition;
use turbojet::gas::Gas;
use turbojet::turbine_law::{census, TurbineLaw};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolLosses, TwoSpoolMatcher};

fn real() -> TwoSpoolLosses {
    TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92, eta_lpt: 0.90,
                     eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true }
}
fn main() {
    let fd = FlightCondition::new(250.0, 50_000.0, 0.85);
    let m = TwoSpoolMatcher::new(build_two_spool_turbojet(Gas::reacting_equilibrium(), 3.0, 6.0, 1500.0, 50_000.0, real()), fd, 1.0);
    let c = m.core();
    for tt4 in [700.0, 750.0, 800.0, 850.0, 900.0, 1000.0] {
        for (n, law) in [("ellipse", TurbineLaw::Ellipse), ("2rows", TurbineLaw::TwoRows)] {
            c.turbine_law.set(law);
            census::take();
            let t0 = std::time::Instant::now();
            let r = m.match_point(&fd, tt4).two();
            let dt = t0.elapsed().as_secs_f64();
            let k = census::take();
            println!("{tt4:5.0} {n:8} {dt:7.3} s  pairs {:6} pair-evals {:7} w45 {:8} interstage {:9}  piH {:.6}",
                     k[0], k[1], k[2], k[3], r.pi_hpt);
        }
    }
}
