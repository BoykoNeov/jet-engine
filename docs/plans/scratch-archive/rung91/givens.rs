//! Rung 91 anchor § 0 — GIVENS only: the uncooled plant (rungs 39/42/90, existing code). Reads the
//! running-line state on CPG (flat + flow/press maps), the HP turbine's own operating point (the new
//! model needs it), rung 90's MEASURED customer-bleed response (the baseline a cooling stream is
//! compared against), and evaluates anchor § 1's linear model for the two cooling streams.
//! No cooling code exists yet.
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

/// Anchor § 1's linear model for one unit of cooling split (e1, e2): returns
/// (d ln pi_HPT, h, l, d ln phi_L, d ln phi_H).
#[allow(clippy::too_many_arguments)]
fn model(e1: f64, e2: f64, f: f64, kt3: f64, kt3p: f64, sigma: f64, rho: f64,
         al: f64, ah: f64, t_l: f64) -> [f64; 5] {
    let e = e1 + e2;
    let (m1, m2) = (e1 / (1.0 + f), e2 / (1.0 + f));
    let p = (e / (1.0 + f) + 0.5 * m1 * (kt3 - 1.0) + 0.5 * m2 * (kt3p - 1.0)) / (1.0 - 0.5 * sigma);
    let h = -e + m1 * kt3 - rho * p;
    let l = -e + m1 * kt3 + sigma * p + m2 * kt3p;
    let phi_l = al * l + ah * (h - t_l * l) + e - 0.5 * l;
    let phi_h = (ah - 0.5) * (h - t_l * l) + 0.5 * t_l * l + e;
    [p, h, l, phi_l, phi_h]
}

fn main() {
    let k = 3.5;
    let kappa = CP_C / CP_T;
    let g_h = 0.3 / 1.3;
    let maps: Vec<(&str, ComponentMap, ComponentMap)> = vec![
        ("flat", ComponentMap::flat(), ComponentMap::flat()),
        ("flow/press", ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() },
                       ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }),
    ];
    println!("kappa = cp_c/cp_t = {kappa:.4}");
    for (name, ml, mh) in maps {
        println!("\n== {name}");
        println!("Tt4  | Tt3    Tt45   f       | tau_hpt pi_hpt eta_hpt sigma  rho   | kT3/T4 kT3/T45 | tL     aL     aH     \
                  | customer model L,H | customer meas(0.02) L,H \
                  | VANE model: dlnpiT  h      l      L      H     | BLADE model: dlnpiT  h      l      L      H");
        let m0 = CustomerBleedMatcher::new(design(), flight(), 1.0, ml, mh, 0.0);
        let mc = CustomerBleedMatcher::new(design().with_customer_bleed(CustomerBleed::share(0.02)),
                                           flight(), 1.0, ml, mh, 0.0);
        for tt4 in [1500.0, 1400.0, 1300.0, 1200.0, 1100.0, 1000.0, 900.0, 850.0, 800.0, 750.0, 700.0, 650.0] {
            let Ok(r0) = m0.try_match_point(&flight(), tt4) else { println!("{tt4} uncooled fails"); continue };
            let s = &r0.base.base;
            let (t2, t25, t3, t45) = (s.station("2").tt, s.station("25").tt, s.station("3").tt,
                                      s.station("45").tt);
            let f = s.station("4").far;
            let (tl, th) = (t25 / t2, t3 / t25);
            let (al, ah) = (a_of(tl, r0.base.eta_lpc, k), a_of(th, r0.base.eta_hpc, k));
            let t_l = (tl - 1.0) / tl;
            let (tau, pi, eta) = (s.tau_hpt, s.pi_hpt, r0.base.eta_hpt);
            let sigma = eta * g_h * pi.powf(g_h) / tau;
            let rho = tau * sigma / (1.0 - tau);
            let (kt3, kt3p) = (kappa * t3 / tt4, kappa * t3 / t45);
            let cl = 1.5 - al - ah * (1.0 - t_l);
            let ch = (1.0 - t_l) * (1.0 - ah) + 0.5;
            let meas = match mc.try_match_point(&flight(), tt4) {
                Ok(r2) => format!("{:+.4} {:+.4}", (r2.base.phi_lp / r0.base.phi_lp).ln() / 0.02,
                                  (r2.base.phi_hp / r0.base.phi_hp).ln() / 0.02),
                Err(_) => "fails        ".into(),
            };
            let v = model(1.0, 0.0, f, kt3, kt3p, sigma, rho, al, ah, t_l);
            let b = model(0.0, 1.0, f, kt3, kt3p, sigma, rho, al, ah, t_l);
            println!("{tt4:.0} | {t3:.1} {t45:.1} {f:.5} | {tau:.4} {pi:.4} {eta:.4} {sigma:.4} {rho:.4} \
                      | {kt3:.4} {kt3p:.4} | {t_l:.4} {al:.4} {ah:.4} | {cl:+.4} {ch:+.4} | {meas} \
                      | {:+.4} {:+.4} {:+.4} {:+.4} {:+.4} | {:+.4} {:+.4} {:+.4} {:+.4} {:+.4}",
                     v[0], v[1], v[2], v[3], v[4], b[0], b[1], b[2], b[3], b[4]);
        }
    }
}
