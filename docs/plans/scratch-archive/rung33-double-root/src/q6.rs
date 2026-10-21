// The oracle grids' cells: outcome of match_point (old or new code, by which crate builds this).
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};
use turbojet::matcher::OffDesignMatcher;
fn cpg() -> Gas {
    let (g, cp) = (1.3, 1239.0);
    Gas::new(GasSpec { gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9, gamma_t: g, cp_t: cp, r_t: (g - 1.0) / g * cp, hpr: 42.8e6, ..GasSpec::default() })
}
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let losses = Losses { pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, eta_m: 0.99, pi_n: 0.98, nozzle_convergent: true, ..Losses::default() };
    for (tag, gas) in [("cpg", cpg()), ("tpg", Gas::thermally_perfect()), ("eq", Gas::reacting_equilibrium())] {
        let m = OffDesignMatcher::new(build_turbojet(gas, 10.0, 1500.0, 50_000.0, losses), FlightCondition::new(250.0, 50_000.0, 0.85), 1.0);
        for m0 in [0.3, 0.5, 0.85, 1.2, 1.6, 2.0] {
            for tt4 in [400.0, 500.0, 600.0, 650.0, 900.0, 1100.0, 1500.0] {
                let fl = FlightCondition::new(250.0, 50_000.0, m0);
                let r = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| m.match_point(&fl, tt4))) {
                    Ok(od) => format!("{:?} pi_t {:.6} pi_c {:.5} F/mdot {:.2} M9 {:.3}", od.branch, od.pi_t, od.pi_c, od.performance.specific_thrust, od.m9),
                    Err(e) => { let s = e.downcast_ref::<String>().cloned().unwrap_or_default(); format!("FAIL {}", s.chars().take(110).collect::<String>()) }
                };
                println!("{tag} {m0:.2} {tt4}: {r}");
            }
        }
    }
}
