// Rung 104 probe P5 — the CLOSED prototype below nozzle choke (rung 103's branch on), every law.
// Probe cells only (anchor § 4): the Controls engine on map "tilted", flights SL+40 M0.85, SL M0.30, design;
// rung 38's engine (no map) on its design flight and hot M0.3. Prints, per law: status, pi_HPT, pi_LPT,
// tau_HPT, tau_LPT, both speeds (map), thrust, cost.
use turbojet::atmosphere::standard;
use turbojet::blade_speed::shape_maps;
use turbojet::engine::FlightCondition;
use turbojet::gas::Gas;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::turbine_law::TurbineLaw;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolLosses, TwoSpoolMapMatcher, TwoSpoolMatcher};

fn flight(dt: f64, h: f64, m0: f64) -> FlightCondition { let (t, p) = standard(h); FlightCondition::new(t + dt, p, m0) }
fn real() -> TwoSpoolLosses {
    TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92, eta_lpt: 0.90,
                     eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true }
}
const LAWS: [(&str, TurbineLaw); 4] = [("pin", TurbineLaw::Pin), ("ellipse", TurbineLaw::Ellipse),
                                       ("2rows", TurbineLaw::TwoRows), ("1row", TurbineLaw::OneRow)];
fn main() {
    let bs = BladeSettings::defaults();
    let (ml, mh) = shape_maps("tilted").unwrap();
    let m = match TwoSpoolMapMatcher::new(bs.design(), bs.flight(), 1.0, ml, mh) { TwoSpoolMapMatcher::Full(c) => c, _ => unreachable!() };
    m.base.below_choke.set(true);
    println!("== Controls engine, map tilted, rung 103 branch ON");
    for (fname, fl) in [("SL+40 M0.85", flight(40.0, 0.0, 0.85)), ("SL M0.30", flight(0.0, 0.0, 0.30)), ("design", bs.flight())] {
        for i in 0..9 {
            let tt4 = 550.0 + 50.0 * i as f64;
            for (ln, law) in LAWS {
                m.base.turbine_law.set(law);
                let t0 = std::time::Instant::now();
                let r = m.try_match_point(&fl, tt4);
                let dt = t0.elapsed().as_secs_f64();
                match r {
                    Ok(x) => println!("{fname:12} {tt4:5.0} {ln:8} M9 {:.3} piH {:.5} piL {:.5} tauH {:.5} tauL {:.5} NL {:.5} NH {:.5} F {:9.2} ({dt:.2}s)",
                        x.base.m9, x.base.pi_hpt, x.base.pi_lpt, x.base.tau_hpt, x.base.tau_lpt, x.n_lp_ratio, x.n_hp_ratio, x.base.thrust),
                    Err(e) => println!("{fname:12} {tt4:5.0} {ln:8} REFUSED {} ({dt:.2}s)", &e.0[..e.0.len().min(110)]),
                }
            }
        }
    }
    println!("== rung 38 engine (no map), rung 103 branch ON");
    let fd = FlightCondition::new(250.0, 50_000.0, 0.85);
    let m38 = TwoSpoolMatcher::new(build_two_spool_turbojet(Gas::reacting_equilibrium(), 3.0, 6.0, 1500.0, 50_000.0, real()), fd, 1.0);
    let c = m38.core();
    c.below_choke.set(true);
    for (fname, fl) in [("design", fd), ("hot M0.3", flight(40.0, 0.0, 0.30))] {
        for tt4 in [550.0, 600.0, 650.0, 700.0, 750.0, 800.0, 900.0, 1000.0] {
            for (ln, law) in LAWS {
                c.turbine_law.set(law);
                let t0 = std::time::Instant::now();
                let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| c.match_point(&fl, tt4)));
                let dt = t0.elapsed().as_secs_f64();
                match r {
                    Ok(x) => println!("{fname:12} {tt4:5.0} {ln:8} M9 {:.3} piH {:.5} piL {:.5} tauH {:.5} tauL {:.5} piLPC {:.4} piHPC {:.4} F {:9.2} ({dt:.2}s)",
                        x.m9, x.pi_hpt, x.pi_lpt, x.tau_hpt, x.tau_lpt, x.pi_lpc, x.pi_hpc, x.thrust),
                    Err(_) => println!("{fname:12} {tt4:5.0} {ln:8} PANIC ({dt:.2}s)"),
                }
            }
        }
    }
}
