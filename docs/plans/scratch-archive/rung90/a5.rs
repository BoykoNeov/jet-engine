//! Rung 90 A5 re-read: the c = 0.05 HP point against the c = 0 line at the same x_H.
//! The scorer's first bracket (Tt4 ± 300 K) reached unchoked points below 1000 K; this one
//! brackets upward only (the customer bleed lowers Tt25, so x_H rises: the matching c = 0 point
//! sits at a HIGHER Tt4).
use turbojet::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
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

fn main() {
    let m0 = CustomerBleedMatcher::new(design(), flight(), 1.0, ComponentMap::flat(), ComponentMap::flat(), 0.0);
    let m5 = CustomerBleedMatcher::new(design().with_customer_bleed(CustomerBleed::share(0.05)), flight(), 1.0,
                                       ComponentMap::flat(), ComponentMap::flat(), 0.0);
    let xh = |t: f64| m0.try_match_point(&flight(), t).ok()
        .map(|r| (t / r.base.base.station("25").tt, r.base.phi_hp));
    for tt4 in [1500.0, 1400.0, 1300.0, 1200.0, 1100.0, 1000.0, 900.0, 850.0, 800.0, 750.0, 700.0, 650.0] {
        let Ok(r5) = m5.try_match_point(&flight(), tt4) else { println!("{tt4} c=.05 FAIL"); continue };
        let xt = tt4 / r5.base.base.station("25").tt;
        let (mut lo, mut hi) = (tt4, tt4 + 400.0);
        let (Some((xl, _)), Some((xu, _))) = (xh(lo), xh(hi)) else { println!("{tt4} bracket FAIL"); continue };
        if !((xl - xt) * (xu - xt) < 0.0) { println!("{tt4} not bracketed {xl} {xt} {xu}"); continue }
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            let (xm, _) = xh(mid).unwrap();
            if xm < xt { lo = mid } else { hi = mid }
        }
        let (xm, ph) = xh(0.5 * (lo + hi)).unwrap();
        println!("{tt4:.0}: x_H {xt:.6} (c=0 at Tt4' {:.2}, x {xm:.6}) phi_H c=.05 {:.6} vs line {ph:.6}: {:+.3} %",
                 0.5 * (lo + hi), r5.base.phi_hp, 100.0 * (r5.base.phi_hp / ph - 1.0));
    }
}
