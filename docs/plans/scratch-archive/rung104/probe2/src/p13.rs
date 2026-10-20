// Rung 104 probe P13 — is the two-row W45 SMOOTH at the 1e-13 scale? Rung 38's engine, equilibrium gas, the design
// point's Tt45/f: W45(x0 + k*h) for k = 0..20, h = 1e-12, and its second differences, each law.
use turbojet::engine::FlightCondition;
use turbojet::gas::Gas;
use turbojet::turbine_law::TurbineLaw;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolLosses, TwoSpoolMatcher};

fn real() -> TwoSpoolLosses {
    TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92, eta_lpt: 0.90,
                     eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true }
}
fn main() {
    let fd = FlightCondition::new(250.0, 50_000.0, 0.85);
    let m = TwoSpoolMatcher::new(build_two_spool_turbojet(Gas::reacting_equilibrium(), 3.0, 6.0, 1500.0, 50_000.0, real()), fd, 1.0);
    let c = m.core();
    let gas = c.gas();
    let s4 = *c.reference.station("4");
    gas.freeze_equilibrium(s4.far, s4.tt, s4.pt);
    let (t45, f) = (c.lp_capacity.td * 0.8, s4.far);
    for (n, law) in [("ellipse", TurbineLaw::Ellipse), ("1row", TurbineLaw::OneRow), ("2rows", TurbineLaw::TwoRows)] {
        c.turbine_law.set(law);
        for x0 in [0.66, 0.69, 0.72] {
            let h = 1e-12;
            let w: Vec<f64> = (0..21).map(|k| c.w45(gas, x0 + k as f64 * h, t45, f, c.eta_lpt).unwrap()).collect();
            let d2: Vec<f64> = w.windows(3).map(|v| (v[0] - 2.0 * v[1] + v[2]) / w[0]).collect();
            let worst = d2.iter().fold(0.0f64, |m, v| m.max(v.abs()));
            let slope = (w[20] - w[0]) / (20.0 * h) / w[0];
            println!("{n:8} x0 {x0}: rel. slope {slope:+.4}  worst |2nd diff|/W {worst:.2e}  (smooth would be ~{:.0e})", 1e-16);
        }
    }
}
