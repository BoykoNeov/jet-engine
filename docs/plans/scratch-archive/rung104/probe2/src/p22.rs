// Rung 104 probe P22 (advisor, close-out): where the LP turbine's efficiency comes off a map term (`a_t`), does the law
// differ from the pin on the PERFECT gas with the nozzle choked — for real, or only by search rounding?
// (1) rung 39's `turb` cells (a_t 0.02) at Mach 0.85 / 1.60; (2) the pages' own surge-realistic maps (a_t 0.02), steady,
// same engine; each: eta_lpt against its design value, and the largest relative difference law vs pin.
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::turbine_law::TurbineLaw;
use turbojet::two_spool::{build_two_spool_turbojet, MatchedMap, TwoSpoolLosses, TwoSpoolMapMatcher};

fn losses() -> TwoSpoolLosses {
    TwoSpoolLosses {
        pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
        eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98,
        p_exit: None, nozzle_convergent: true,
    }
}

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn run(law: TurbineLaw, ml: ComponentMap, mh: ComponentMap, m0: f64, tt4: f64) -> Option<[f64; 6]> {
    let d = build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, losses()).with_turbine_law(law);
    let m = TwoSpoolMapMatcher::new(d, FlightCondition::new(250.0, 50_000.0, 0.85), 1.0, ml, mh);
    let fl = FlightCondition::new(250.0, 50_000.0, m0);
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| m.match_point(&fl, tt4))) {
        Ok(MatchedMap::Two(od)) => Some([od.eta_lpt, od.n_lp, od.n_hp, od.base.pi_hpc, od.base.tau_lpt, od.base.thrust]),
        _ => None,
    }
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let flat = ComponentMap::flat();
    let turb = ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, a_t: 0.02, ..flat };
    let cases: Vec<(&str, ComponentMap, ComponentMap)> = vec![
        ("r39 turb", turb, turb),
        ("page flow", ComponentMap::surge_flow(), ComponentMap::surge_flow()),
        ("page press", ComponentMap::surge_pressure(), ComponentMap::surge_pressure()),
        ("page tilted", ComponentMap::surge_tilted(), ComponentMap::surge_tilted()),
    ];
    for (name, ml, mh) in cases {
        for m0 in [0.85, 1.60] {
            for tt4 in [600.0, 900.0, 1100.0, 1500.0] {
                let (Some(p), Some(e)) = (run(TurbineLaw::Pin, ml, mh, m0, tt4), run(TurbineLaw::Ellipse, ml, mh, m0, tt4))
                else { println!("{name:12} M{m0:.2} {tt4:5.0}  (refused on one side)"); continue };
                let names = ["eta_lpt", "n_lp", "n_hp", "pi_hpc", "tau_lpt", "thrust"];
                let mut worst = (0.0f64, "");
                for k in 0..6 {
                    let r = (e[k] / p[k] - 1.0).abs();
                    if r > worst.0 { worst = (r, names[k]); }
                }
                println!("{name:12} M{m0:.2} {tt4:5.0}  eta_lpt-0.90 {:+.2e}  worst law-vs-pin {:.2e} ({})  n_hp {:+.2e}  n_lp {:+.2e}",
                         p[0] - 0.90, worst.0, worst.1, e[2] / p[2] - 1.0, e[1] / p[1] - 1.0);
            }
        }
    }
}
