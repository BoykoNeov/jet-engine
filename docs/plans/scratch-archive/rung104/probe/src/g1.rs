// Rung 104 probe G1: can the LP turbine's NGV (A45) choke anywhere? A generous bound: the vane's largest
// static drop is pt45 down to the LP-turbine EXIT static at exit Mach M5 (0.6, generous), against the
// critical ratio ((g+1)/2)^(g/(g-1)) at the gas's gamma at Tt45 (the hottest, lowest-gamma choice: the
// smallest critical ratio, so the bound leans TOWARD choking). Same for the HP NGV (A4) for the record.
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolLosses, TwoSpoolMapMatcher, TwoSpoolResult};

fn real() -> TwoSpoolLosses {
    TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92, eta_lpt: 0.90,
                     eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true }
}
fn cpg() -> Gas {
    Gas::new(GasSpec { gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9, gamma_t: 1.3, cp_t: 1239.0,
                       r_t: (1.3 - 1.0) / 1.3 * 1239.0, hpr: 42.8e6, ..GasSpec::default() })
}
fn crit(g: f64) -> f64 { ((g + 1.0) / 2.0).powf(g / (g - 1.0)) }
fn stat(g: f64, m: f64) -> f64 { (1.0 + 0.5 * (g - 1.0) * m * m).powf(g / (g - 1.0)) }

fn report(tag: &str, gas: &Gas, r: &TwoSpoolResult, worst: &mut [f64; 2]) {
    let (s4, s45, s5) = (r.station("4"), r.station("45"), r.station("5"));
    if gas.is_equilibrium() { gas.freeze_equilibrium(s4.far, s4.tt, s4.pt); }
    let g45 = gas.gamma_t_at(s45.tt, s45.far);
    let g5 = gas.gamma_t_at(s5.tt, s5.far);
    let g4 = gas.gamma_t_at(s4.tt, s4.far);
    let lp_tot = s45.pt / s5.pt;
    let lp_bound = lp_tot * stat(g5, 0.6);
    let hp_tot = s4.pt / s45.pt;
    let hp_bound = hp_tot * stat(g45, 0.6);
    let (lpm, hpm) = (lp_bound / crit(g45), hp_bound / crit(g4));
    worst[0] = worst[0].max(lpm);
    worst[1] = worst[1].max(hpm);
    println!("{tag} Tt4 {:6.0}  LPT pt45/pt5 {:.4}  bound(M5=.6) {:.4}  crit {:.4}  LP bound/crit {:.3}  | HPT pt4/pt45 {:.4} bound {:.4} crit {:.4} HP bound/crit {:.3}",
             r.tt4, lp_tot, lp_bound, crit(g45), lpm, hp_tot, hp_bound, crit(g4), hpm);
}

fn main() {
    let f = ComponentMap::flat();
    let maps = [("flat", f, f),
                ("mixed", ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f },
                          ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f })];
    let flights = [("design M.85", FlightCondition::new(250.0, 50_000.0, 0.85)),
                   ("SLS M0", FlightCondition::new(288.15, 101_325.0, 0.0)),
                   ("hot M.3", FlightCondition::new(318.15, 101_325.0, 0.3)),
                   ("M2 36k", FlightCondition::new(216.65, 22_632.0, 2.0))];
    let mut worst = [0.0f64; 2];
    for (gname, gas) in [("CPG", cpg()), ("reacting-eq", Gas::reacting_equilibrium())] {
        for (mn, ml, mh) in maps.iter() {
            let design = build_two_spool_turbojet(gas.clone(), 3.0, 6.0, 1500.0, 50_000.0, real());
            let m = TwoSpoolMapMatcher::new(design, flights[0].1, 1.0, *ml, *mh);
            for (fname, fl) in flights.iter() {
                for tt4 in [600.0, 700.0, 800.0, 900.0, 1000.0, 1100.0, 1200.0, 1300.0, 1400.0, 1500.0, 1600.0] {
                    match m.core().try_match_point(fl, tt4) {
                        Ok(p) => report(&format!("{gname} {mn} {fname}"), m.core().gas(), &p.base, &mut worst),
                        Err(e) => println!("{gname} {mn} {fname} Tt4 {tt4:6.0}  refused: {}", &e.0[..e.0.len().min(70)]),
                    }
                }
            }
        }
    }
    println!("WORST LP bound/crit {:.4}   WORST HP bound/crit {:.4}", worst[0], worst[1]);
}
