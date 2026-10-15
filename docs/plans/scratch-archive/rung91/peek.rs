use turbojet::customer_bleed::CustomerBleed;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::turbine_cooling::{TurbineCooling, TurbineCoolingMatcher};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolLosses};
fn main() {
    let real = TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
        eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true };
    let g = Gas::new(GasSpec { gamma_c: 1.4, cp_c: 1004.0, r_c: 0.4/1.4*1004.0, gamma_t: 1.3, cp_t: 1239.0,
        r_t: 0.3/1.3*1239.0, hpr: 42.8e6, ..GasSpec::default() });
    let fl = FlightCondition::new(250.0, 50_000.0, 0.85);
    let e = build_two_spool_turbojet(g, 3.0, 6.0, 1500.0, 50_000.0, real)
        .with_customer_bleed(CustomerBleed::share(0.3)).with_turbine_cooling(TurbineCooling::new(0.12, 0.1));
    let m = TurbineCoolingMatcher::new(e, fl, 1.0, ComponentMap::flat(), ComponentMap::flat(), 0.0);
    println!("{:?}", m.try_match_point(&fl, 1500.0).err().map(|e| e.0));
}
