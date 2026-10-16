//! Rung 93 probe 1 — does rig A's steady two-spool engine (rung 91's cooled matcher) solve ABOVE
//! its design Tt4 (1500 K)? No cooling and 5 % vane air; flat and flow/press maps.
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::turbine_cooling::{TurbineCooling, TurbineCoolingMatcher};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

fn real() -> TwoSpoolLosses {
    TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
                     eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None,
                     nozzle_convergent: true }
}
fn cpg() -> Gas {
    let (cp_c, cp_t) = (1004.0, 1239.0);
    Gas::new(GasSpec { gamma_c: 1.4, cp_c, r_c: 0.4 / 1.4 * cp_c, gamma_t: 1.3, cp_t, r_t: 0.3 / 1.3 * cp_t,
                       hpr: 42.8e6, ..GasSpec::default() })
}
fn design() -> TwoSpoolEngine { build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, real()) }
fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

fn main() {
    let maps: Vec<(&str, ComponentMap, ComponentMap)> = vec![
        ("flat", ComponentMap::flat(), ComponentMap::flat()),
        ("flow/press", ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() },
                       ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }),
    ];
    for (name, ml, mh) in &maps {
        for vane in [0.0, 0.05] {
            println!("\n== {name}, vane {vane}");
            let eng = if vane > 0.0 { design().with_turbine_cooling(TurbineCooling::new(vane, 0.0)) } else { design() };
            let m = TurbineCoolingMatcher::new(eng, flight(), 1.0, ml.clone(), mh.clone(), 0.0);
            println!("Tt4  | N_L    N_H    | phi_L  phi_H  | pi_L   pi_H  | Tt3    Tt41   | F/mdot  mdot");
            for tt4 in [1400.0, 1500.0, 1550.0, 1600.0, 1700.0, 1800.0, 1900.0, 2000.0] {
                match m.try_match_point(&flight(), tt4) {
                    Ok(r) => {
                        let b = &r.base;
                        let s = &b.base;
                        let t3 = s.station("3").tt;
                        let t41 = r.booking.map(|k| k.tt41).unwrap_or(tt4);
                        let st = r.booking.map(|k| k.st_inlet).unwrap_or(s.performance.specific_thrust);
                        println!("{tt4:.0} | {:.4} {:.4} | {:.4} {:.4} | {:.3} {:.3} | {t3:.1} {t41:.1} | {st:.2} {:.4}",
                                 b.n_lp_ratio, b.n_hp_ratio, b.phi_lp, b.phi_hp, s.pi_lpc, s.pi_hpc, s.mdot_air);
                    }
                    Err(e) => println!("{tt4:.0} | REFUSED: {}", e.0),
                }
            }
        }
    }
}
