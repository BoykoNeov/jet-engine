//! Rung 90 anchor § 0 — GIVENS only: the c = 0 plant (rung 39/42, existing code). Reads the
//! running-line state on CPG + flat maps and the rung-42 b-response (existing), and evaluates the
//! linear model of anchor § 1 for both b and c. No customer-bleed code exists yet.
use turbojet::bleed::TwoSpoolBleedMatcher;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

fn real() -> TwoSpoolLosses {
    TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
                     eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None,
                     nozzle_convergent: true }
}
fn cpg() -> Gas {
    Gas::new(GasSpec { gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
                       gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
                       hpr: 42.8e6, ..GasSpec::default() })
}
fn design() -> TwoSpoolEngine { build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, real()) }
fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

fn a_of(tau: f64, eta: f64, k: f64) -> f64 { k * eta * (tau - 1.0) / (1.0 + eta * (tau - 1.0)) }

fn main() {
    let k = 3.5;
    let maps: Vec<(&str, ComponentMap, ComponentMap)> = vec![
        ("flat", ComponentMap::flat(), ComponentMap::flat()),
        ("flow/press", ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() },
                       ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }),
    ];
    for (name, ml, mh) in maps {
        println!("== {name}");
        println!("Tt4  pi_lpc pi_hpc  tL    aL    aH   | model c: dlnphiL dlnphiH | model b: dlnphiL dlnphiH | meas b(0.02): dlnphiL dlnphiH");
        let m0 = TwoSpoolBleedMatcher::new(design(), flight(), 1.0, ml, mh, 0.0);
        let m2 = TwoSpoolBleedMatcher::new(design(), flight(), 1.0, ml, mh, 0.02);
        for tt4 in [1500.0, 1400.0, 1300.0, 1200.0, 1100.0, 1000.0, 900.0, 850.0, 800.0, 750.0, 700.0, 650.0] {
            let Ok(r0) = m0.try_match_point(&flight(), tt4) else { println!("{tt4} c=0 fails"); continue };
            let s = &r0.base.base;
            let (t2, t25, t3) = (s.station("2").tt, s.station("25").tt, s.station("3").tt);
            let (tl, th) = (t25 / t2, t3 / t25);
            let (al, ah) = (a_of(tl, r0.base.eta_lpc, k), a_of(th, r0.base.eta_hpc, k));
            let t_l = (tl - 1.0) / tl;
            let c_l = 1.5 - al - ah * (1.0 - t_l);
            let c_h = (1.0 - t_l) * (1.0 - ah) + 0.5;
            let b_l = 1.5 - al + ah * t_l;
            let b_h = t_l * (ah - 1.0);
            let meas = match m2.try_match_point(&flight(), tt4) {
                Ok(r2) => format!("{:+.4} {:+.4}", (r2.base.phi_lp / r0.base.phi_lp).ln() / 0.02,
                                  (r2.base.phi_hp / r0.base.phi_hp).ln() / 0.02),
                Err(_) => "fails".into(),
            };
            println!("{tt4:.0} {:.4} {:.4} {t_l:.4} {al:.4} {ah:.4} | {c_l:+.4} {c_h:+.4} | {b_l:+.4} {b_h:+.4} | {meas}  phi=({:.4},{:.4}) f={:.5}",
                     s.pi_lpc, s.pi_hpc, r0.base.phi_lp, r0.base.phi_hp, s.station("4").far);
        }
    }
}
