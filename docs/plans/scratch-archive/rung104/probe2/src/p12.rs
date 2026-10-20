// Rung 104 PRE-FLIGHT of H2's equilibrium-gas cells — CLASS ONLY (does each law solve there, and is the nozzle choked);
// nothing measured is printed. Rung 39's engine (equilibrium gas, pi 3/6, Tt4 1500 design, 250 K / 50 kPa / M0.85).
use turbojet::atmosphere::standard;
use turbojet::blade_speed::shape_maps;
use turbojet::engine::FlightCondition;
use turbojet::gas::Gas;
use turbojet::map::ComponentMap;
use turbojet::turbine_law::TurbineLaw;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolLosses, TwoSpoolMapMatcher};

fn real() -> TwoSpoolLosses {
    TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92, eta_lpt: 0.90,
                     eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true }
}
fn main() {
    let fd = FlightCondition::new(250.0, 50_000.0, 0.85);
    let (t2, p2) = standard(2000.0);
    let f2 = FlightCondition::new(t2, p2, 0.80);
    let steep = shape_maps("steep").unwrap();
    for (mname, (ml, mh)) in [("flat", (ComponentMap::flat(), ComponentMap::flat())), ("steep", steep)] {
        let m = match TwoSpoolMapMatcher::new(build_two_spool_turbojet(Gas::reacting_equilibrium(), 3.0, 6.0, 1500.0, 50_000.0, real()),
                                              fd, 1.0, ml, mh) {
            TwoSpoolMapMatcher::Full(c) => c,
            _ => unreachable!(),
        };
        for (fname, fl) in [("design", fd), ("2km M0.80", f2)] {
            let mut line = format!("{mname:6} {fname:10}");
            for i in 0..7 {
                let tt4 = 800.0 + 100.0 * i as f64;
                let mut cls = String::new();
                for law in [TurbineLaw::Pin, TurbineLaw::OneRow, TurbineLaw::TwoRows, TurbineLaw::Ellipse] {
                    m.base.turbine_law.set(law);
                    cls.push(match m.try_match_point(&fl, tt4) {
                        Ok(r) if r.base.m9 > 1.0 - 1e-9 => 'c',
                        Ok(_) => 'u',
                        Err(_) => 'x',
                    });
                }
                line += &format!(" {tt4:.0}:{cls}");
            }
            println!("{line}");
        }
    }
    println!("per Tt4: pin / one row / two rows / ellipse — c choked, u unchoked, x refused");
}
