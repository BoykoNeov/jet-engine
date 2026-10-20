// Rung 104 probe P15 — THE SORT, rung 103's headline (H1): "below choke a better LP compressor LOWERS the HP pressure
// ratio". Its own instrument (Δπ_HPC for η_LPC 0.90 → 0.91 through the full match, rung 103's branch on) and its own
// cells (rung 39's engine, both gases, three maps, two flights), under each of rung 104's settings.
use turbojet::atmosphere::standard;
use turbojet::blade_speed::shape_maps;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::turbine_law::TurbineLaw;
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
    let fd = FlightCondition::new(250.0, 50_000.0, 0.85);
    let (t, p) = standard(0.0);
    let sl = FlightCondition::new(t, p, 0.50);
    let f = ComponentMap::flat();
    let shapes = [("flat", f, f),
                  ("mixed", ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f },
                            ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f }),
                  ("steep", shape_maps("steep").unwrap().0, shape_maps("steep").unwrap().1)];
    let laws = [("pin", TurbineLaw::Pin), ("1row", TurbineLaw::OneRow), ("2rows", TurbineLaw::TwoRows), ("ellipse", TurbineLaw::Ellipse)];
    let mut tally = [[0usize; 2]; 4];
    for (gn, gas) in [("CPG", cpg()), ("eq", Gas::reacting_equilibrium())] {
        for (sn, ml, mh) in shapes.iter() {
            let mut ms: Vec<TwoSpoolMapMatcher> = laws.iter().map(|(_, l)| TwoSpoolMapMatcher::new(
                build_two_spool_turbojet(gas.clone(), 3.0, 6.0, 1500.0, 50_000.0, real()).with_turbine_law(*l), fd, 1.0, *ml, *mh)).collect();
            for (fname, fl) in [("250K", fd), ("SL M0.50", sl)] {
                for tt4 in [450.0, 500.0, 550.0, 600.0, 650.0, 700.0, 750.0] {
                    let mut line = format!("{gn:3} {sn:5} {fname:8} {tt4:4.0}");
                    for (k, m) in ms.iter_mut().enumerate() {
                        let mut d = |eta: f64, m: &mut TwoSpoolMapMatcher| {
                            m.core_mut().base.eta_lpc = eta;
                            m.core().base.below_choke.set(true);
                            let r = m.core().try_match_point(&fl, tt4);
                            m.core().base.below_choke.set(false);
                            r.ok()
                        };
                        let a = d(0.90, m);
                        let b = d(0.91, m);
                        m.core_mut().base.eta_lpc = 0.90;
                        match (a, b) {
                            (Some(a), Some(b)) if a.base.m9 < 1.0 - 1e-9 => {
                                let dp = b.base.pi_hpc - a.base.pi_hpc;
                                line += &format!(" | {} M9 {:.3} {dp:+.2e}", laws[k].0, a.base.m9);
                                tally[k][(dp > 0.0) as usize] += 1;
                            }
                            (Some(_), Some(_)) => line += &format!(" | {} choked", laws[k].0),
                            _ => line += &format!(" | {} refused", laws[k].0),
                        }
                    }
                    println!("{line}");
                }
            }
        }
    }
    for (k, (n, _)) in laws.iter().enumerate() {
        println!("{n:8}: below choke, d piHPC < 0 at {} cells, > 0 at {}", tally[k][0], tally[k][1]);
    }
}
