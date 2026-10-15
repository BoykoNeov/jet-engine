// D1' residual convergence probe (scratch).
use r89probe::*;
use turbojet::asymmetric_valve::{asym_valve, march_marked};
use turbojet::lagged_incidence::{lag_read, valve_at};
use turbojet::limited_bleed::ValveReference;
use turbojet::stator_transient::Ramp;
use turbojet::valve_reference::sibling;
fn main() {
    let m = bare(lp_map(), hp_map());
    for (sm, to, tc, rf) in [(0.38, 0.2, 0.01, ValveReference::Phi), (0.42, 0.05, 0.2, ValveReference::Phi), (0.3423, 0.01, 0.2, ValveReference::Phi),
                             (0.46, 0.2, 0.01, ValveReference::Incidence), (0.3812, 0.2, 0.01, ValveReference::Phi), (0.3812, 0.2, 0.2, ValveReference::Phi)] {
        let mm = sibling(&m, Some(&stat()), Some(asym_valve(phi_of(sm), BMAX, to, tc, rf)));
        for ds in [0.005, 0.0025, 0.00125, 0.000625, 0.0003125] {
            let rp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r: 0.5, s_settle: 1.2, ds };
            let (t, fc, cl) = march_marked(&mm, &flight(), &rp);
            let l = lag_read(&mm, &flight(), &t, cl);
            let sw = t.windows(2).filter(|w| { let (a, b) = (valve_at(&w[0]), valve_at(&w[1])); (a.0 - a.1).signum() != (b.0 - b.1).signum() }).count();
            let sat = t.iter().filter(|p| valve_at(p).1 >= BMAX - 1e-12 || valve_at(p).1 <= 1e-12).count();
            println!("sm {sm} ({to},{tc}) {:?} ds {ds}: resid {:.3e} b0 {:.3e} bend {:.3e} U {:.3e} D {:.3e} switches {sw} cmd-at-limit pts {sat}/{} fc {fc:.4} clamps {cl}",
                     rf, l.identity_residual, l.b0, l.b_end, l.rise, l.fall, t.len());
        }
    }
}
