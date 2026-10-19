// Rung 103 probe 2b: is rung 39's "LP efficiency leaves the HP pressure ratio alone" EXACT through the full match on the
// choked branch? (CHOKED cells only — the switch stays OFF; unchoked cells are skipped, not read.)
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolLosses, TwoSpoolMapMatcher};

fn real() -> TwoSpoolLosses {
    TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92, eta_lpt: 0.90,
                     eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true }
}
fn cpg() -> Gas {
    Gas::new(GasSpec { gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9, gamma_t: 1.3, cp_t: 1239.0,
                       r_t: (1.3 - 1.0) / 1.3 * 1239.0, hpr: 42.8e6, ..GasSpec::default() })
}

fn main() {
    let fl = FlightCondition::new(250.0, 50_000.0, 0.85);
    let f = ComponentMap::flat();
    let shapes = [("flat", f, f),
                  ("mixed", ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f },
                            ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f })];
    for (gname, gas) in [("CPG", cpg()), ("reacting-eq", Gas::reacting_equilibrium())] {
        for (sn, ml, mh) in shapes.iter() {
            let design = build_two_spool_turbojet(gas.clone(), 3.0, 6.0, 1500.0, 50_000.0, real());
            let mut m = TwoSpoolMapMatcher::new(design, fl, 1.0, *ml, *mh);
            for tt4 in [700.0, 800.0, 900.0, 1000.0, 1200.0, 1400.0] {
                m.core_mut().base.eta_lpc = 0.90;
                let a = m.core().try_match_point(&fl, tt4);
                m.core_mut().base.eta_lpc = 0.91;
                let b = m.core().try_match_point(&fl, tt4);
                match (a, b) {
                    (Ok(a), Ok(b)) => println!("{gname} {sn} Tt4 {tt4:.0} choked: dpi_HPC {:+.3e} (bits equal {}) dpi_LPC {:+.3e}",
                        b.base.pi_hpc - a.base.pi_hpc, a.base.pi_hpc.to_bits() == b.base.pi_hpc.to_bits(),
                        b.base.pi_lpc - a.base.pi_lpc),
                    _ => println!("{gname} {sn} Tt4 {tt4:.0}: refused (skipped)"),
                }
            }
        }
    }
}
