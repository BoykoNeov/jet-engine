//! **Rung 33's double root — the repair's gate (2026-10-21; `docs/rung33-spec.md` § The double root).**
//!
//! Off the design flight, rung 33's subsonic residual over `pi_t` has two roots between a lowest
//! throttle (where they merge, ~646 K on the Fly-it engine on a 45 °C sea-level day at Mach 0.3) and
//! near choke: the working point and a weak one where the turbine barely expands. The two-ended
//! march then read the same sign at both ends and refused SUB-IDLE. Where it does that, the bracket
//! is now walked up from the low end to the first sign change.
//!
//! 1. **Non-vacuous:** on these cells the march's ends DO share a sign and the grid DOES show two
//!    roots — so the walk, not the old march, produced every answer below.
//! 2. **The working root:** the answer is the LOWER root (the grid's, bisected), with thrust.
//! 3. **It is rung 34's answer:** the flat-map spool equilibrium — reached through the forward
//!    closure, never through this matcher — lands on the same `pi_t` and `pi_c`, away from choke.
//! 4. **Below the lowest throttle it still refuses**, in the old words first.

#[path = "common/double_root.rs"]
#[allow(dead_code)]
mod double_root;

use turbojet::components::ram_recovery;
use turbojet::engine::{build_turbojet, FlightCondition};
use turbojet::map::ComponentMap;
use turbojet::matcher::{Branch, OffDesignMatcher};
use turbojet::sandbox::{FlySettings, GasModel};
use turbojet::spool::SpoolTransient;

const GASES: [GasModel; 2] = [GasModel::ThermallyPerfect, GasModel::Perfect];
/// Throttles inside the two-root window on the hot day, all with the nozzle well unchoked.
const TT4S: [f64; 4] = [675.0, 700.0, 800.0, 875.0];

fn hot() -> FlightCondition {
    FlightCondition::new(318.15, 101_325.0, 0.30)
}

/// The Fly-it engine's design (the page's opening one), on `gas`.
fn engine(gas: GasModel) -> (turbojet::engine::Engine, FlightCondition, f64) {
    let d = FlySettings { gas, ..FlySettings::defaults() }.capture();
    (build_turbojet(d.gas.gas(), d.pi_c, d.tt4, d.p0, d.losses()), d.flight(), d.mdot)
}

fn matcher(gas: GasModel) -> OffDesignMatcher {
    let (e, fl, mdot) = engine(gas);
    OffDesignMatcher::new(e, fl, mdot)
}

/// Every root of the residual on a 1 600-interval grid over the march's range, each bisected.
fn roots(m: &OffDesignMatcher, fl: &FlightCondition, tt4: f64) -> Vec<f64> {
    let (s0, _) = m.freestream_for(fl);
    let (tt2, pt2) = (s0.tt, m.pi_d_max * ram_recovery(fl.m0) * s0.pt);
    let r = |y: f64| m.try_subsonic_operating(tt4, tt2, pt2, y, fl.p0).map(|op| op.resid).ok();
    let mut out = Vec::new();
    let mut prev: Option<(f64, f64)> = None;
    for i in 0..=1600 {
        let y = 0.15 + (0.9995 - 0.15) * i as f64 / 1600.0;
        let Some(v) = r(y) else { prev = None; continue };
        if let Some((py, pv)) = prev {
            if pv * v < 0.0 {
                let (mut a, mut fa, mut b) = (py, pv, y);
                while b - a > 1e-14 {
                    let c = 0.5 * (a + b);
                    let fc = r(c).expect("inside an evaluable interval");
                    if fa * fc <= 0.0 { b = c } else { (a, fa) = (c, fc) }
                }
                out.push(0.5 * (a + b));
            }
        }
        prev = Some((y, v));
    }
    out
}

#[test]
fn the_cells_have_two_roots_and_the_march_ends_share_a_sign() {
    for g in GASES {
        let m = matcher(g);
        for tt4 in TT4S {
            assert!(double_root::two_roots(&m, &hot(), tt4), "{g:?} {tt4}: the march's ends straddle — vacuous");
            assert_eq!(roots(&m, &hot(), tt4).len(), 2, "{g:?} {tt4}: not two roots");
        }
    }
}

#[test]
fn the_answer_is_the_working_root_with_thrust() {
    for g in GASES {
        let m = matcher(g);
        for tt4 in TT4S {
            let r = roots(&m, &hot(), tt4);
            let od = m.match_point(&hot(), tt4);
            assert_eq!(od.branch, Branch::Subsonic, "{g:?} {tt4}");
            assert!((od.pi_t - r[0]).abs() < 1e-10, "{g:?} {tt4}: pi_t {} is not the lower root {} (the other: {})",
                    od.pi_t, r[0], r[1]);
            assert!(od.performance.specific_thrust > 0.0 && od.m9 < 0.95, "{g:?} {tt4}");
        }
    }
}

#[test]
fn it_is_rung_34s_flat_map_equilibrium() {
    let mut worst: f64 = 0.0;
    for g in GASES {
        let m = matcher(g);
        let (e, fl, mdot) = engine(g);
        let st = SpoolTransient::new(e, fl, mdot, ComponentMap::flat());
        for tt4 in TT4S {
            let od = m.match_point(&hot(), tt4);
            let eq = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| st.equilibrium(&hot(), tt4, None)));
            // Rung 34's own speed search stops short of the lowest throttles on this day (675/700 K: it cannot evaluate
            // below ~0.6 of design speed, the next repair's seam). Measured worst where it runs: 3.9e-11.
            let Ok(eq) = eq else { continue };
            assert_eq!(eq.branch, Branch::Subsonic, "{g:?} {tt4}");
            let d = (od.pi_t - eq.pi_t).abs().max((od.pi_c - eq.pi_c).abs() / od.pi_c);
            println!("{g:?} {tt4}: pi_t {:.12} vs {:.12}, pi_c {:.10} vs {:.10}, worst {d:.2e}", od.pi_t, eq.pi_t, od.pi_c, eq.pi_c);
            worst = worst.max(d);
        }
    }
    assert!(worst > 0.0, "every cell skipped, or bit-equal — the comparison ran nowhere it could disagree");
    assert!(worst < 1e-9, "rung 33's repaired root is not rung 34's flat-map equilibrium: worst {worst:.2e}");
}

#[test]
fn below_the_lowest_throttle_it_still_refuses_in_the_old_words() {
    for g in GASES {
        let m = matcher(g);
        assert!(roots(&m, &hot(), 640.0).is_empty(), "{g:?}: 640 K has a root — pick a lower throttle");
        let msg = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| m.match_point(&hot(), 640.0)))
            .err().and_then(|e| e.downcast_ref::<String>().cloned()).expect("640 K refused");
        assert!(msg.starts_with("rung-33 subsonic match does not bracket") && msg.contains("SUB-IDLE"), "{msg}");
        assert!(msg.contains("Both ends were positive"), "the walk's reason is missing: {msg}");
    }
}
