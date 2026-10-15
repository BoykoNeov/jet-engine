//! Rung 92 anchor § 0 — GIVENS only, on EXISTING code (rungs 42 / 90): no offtake code exists.
//! Per Tt4 on rig A (flat + flow/press): the compressor slopes a_L, a_H, t_L; each shaft's
//! compressor power (absolute, mdot_design = 1) and its ratio to the design HP compressor power;
//! rung 42's interstage-bleed response and rung 90's customer-bleed response, MEASURED
//! (Richardson 2D(0.01) - D(0.02)); and anchor § 1's linear model for a shaft-power offtake.
use turbojet::bleed::TwoSpoolBleedMatcher;
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
const CP_C: f64 = 1004.0;
const CP_T: f64 = 1239.0;
fn cpg() -> Gas {
    Gas::new(GasSpec { gamma_c: 1.4, cp_c: CP_C, r_c: (1.4 - 1.0) / 1.4 * CP_C,
                       gamma_t: 1.3, cp_t: CP_T, r_t: (1.3 - 1.0) / 1.3 * CP_T,
                       hpr: 42.8e6, ..GasSpec::default() })
}
fn design() -> TwoSpoolEngine { build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, real()) }
fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }
fn a_of(tau: f64, eta: f64, k: f64) -> f64 { k * eta * (tau - 1.0) / (1.0 + eta * (tau - 1.0)) }

fn main() {
    let k = 3.5;
    let g = cpg();
    let maps: Vec<(&str, ComponentMap, ComponentMap)> = vec![
        ("flat", ComponentMap::flat(), ComponentMap::flat()),
        ("flow/press", ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() },
                       ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }),
    ];
    for (name, ml, mh) in maps {
        println!("\n== {name}");
        let m0 = CustomerBleedMatcher::new(design(), flight(), 1.0, ml, mh, 0.0);
        let mb = |b: f64| TwoSpoolBleedMatcher::new(design(), flight(), 1.0, ml, mh, b);
        let mc = |c: f64| CustomerBleedMatcher::new(design().with_customer_bleed(CustomerBleed::share(c)),
                                                    flight(), 1.0, ml, mh, 0.0);
        let (mb1, mb2, mc1, mc2) = (mb(0.01), mb(0.02), mc(0.01), mc(0.02));
        // Design HP compressor power (absolute, mdot_design = 1), off the 1500 K match.
        let rd = m0.try_match_point(&flight(), 1500.0).unwrap();
        let sd = &rd.base.base;
        let p_ref_h = sd.mdot_air * (g.h_c(sd.station("3").tt) - g.h_c(sd.station("25").tt));
        let p_ref_l = sd.mdot_air * (g.h_c(sd.station("25").tt) - g.h_c(sd.station("2").tt));
        println!("P_ref_H = {p_ref_h:.1} W, P_ref_L = {p_ref_l:.1} W (design, mdot 1), ratio H/L {:.4}",
                 p_ref_h / p_ref_l);
        println!("Tt4  | pi_L   pi_H   | tL     aL     aH     | mdot   P_C,H/Pref  P_C,L/Pref \
                  | b meas L,H        | c meas L,H        \
                  | HPshaft model L,H | LPshaft model L,H \
                  | HPshaft from(c,b) L,H | LPshaft from b L,H");
        for tt4 in [1500.0, 1400.0, 1300.0, 1200.0, 1100.0, 1000.0, 900.0, 850.0, 800.0, 750.0, 700.0, 650.0] {
            let r0 = m0.try_match_point(&flight(), tt4).unwrap();
            let s = &r0.base.base;
            let (t2, t25, t3) = (s.station("2").tt, s.station("25").tt, s.station("3").tt);
            let (tl, th) = (t25 / t2, t3 / t25);
            let (al, ah) = (a_of(tl, r0.base.eta_lpc, k), a_of(th, r0.base.eta_hpc, k));
            let t_l = (tl - 1.0) / tl;
            let pch = s.mdot_air * (g.h_c(t3) - g.h_c(t25)) / p_ref_h;
            let pcl = s.mdot_air * (g.h_c(t25) - g.h_c(t2)) / p_ref_h;
            let rich = |p1: (f64, f64), p2: (f64, f64)| -> (f64, f64) {
                let d = |p: (f64, f64), h: f64| ((p.0 / r0.base.phi_lp).ln() / h,
                                                  (p.1 / r0.base.phi_hp).ln() / h);
                let (a, b) = (d(p1, 0.01), d(p2, 0.02));
                (2.0 * a.0 - b.0, 2.0 * a.1 - b.1)
            };
            let b1 = mb1.try_match_point(&flight(), tt4).unwrap();
            let b2 = mb2.try_match_point(&flight(), tt4).unwrap();
            let bm = rich((b1.base.phi_lp, b1.base.phi_hp), (b2.base.phi_lp, b2.base.phi_hp));
            let c1 = mc1.try_match_point(&flight(), tt4).unwrap();
            let c2 = mc2.try_match_point(&flight(), tt4).unwrap();
            let cm = rich((c1.base.phi_lp, c1.base.phi_hp), (c2.base.phi_lp, c2.base.phi_hp));
            // Linear model, per unit share of that shaft's turbine work.
            let hp = (-ah, 0.5 - ah);
            let lp = (0.5 - al + ah * t_l, t_l * (ah - 1.0));
            // First-order decomposition from MEASURED responses (flat maps: exact at first order).
            let lp_from_b = (bm.0 - 1.0, bm.1);
            let hp_from_cb = (cm.0 - bm.0, cm.1 - 1.0 - bm.1);
            println!("{tt4:.0} | {:.3} {:.3} | {t_l:.4} {al:.4} {ah:.4} | {:.4} {pch:.4} {pcl:.4} \
                      | {:+.4} {:+.4} | {:+.4} {:+.4} | {:+.4} {:+.4} | {:+.4} {:+.4} \
                      | {:+.4} {:+.4} | {:+.4} {:+.4}",
                     s.pi_lpc, s.pi_hpc, s.mdot_air, bm.0, bm.1, cm.0, cm.1, hp.0, hp.1, lp.0, lp.1,
                     hp_from_cb.0, hp_from_cb.1, lp_from_b.0, lp_from_b.1);
        }
    }
}
