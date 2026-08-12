//! Rung-10 verification: the finite-rate quench — the RQL hazard, quantified.
//!
//! Rung 9 burned a RICH primary and froze NO through the IDEAL (infinitely-fast) quench: EI_NO
//! collapses on the rich flank of the NO-vs-φ bell. But a real quench mixes over a finite time,
//! and while it does the LOCAL mixture passes through STOICHIOMETRIC — the peak of the bell. So
//! a rich primary's temperature RISES through the stoich peak on the way down, and the
//! extended-Zeldovich rate RE-MAKES NO along that path. Rung 10 resolves the quench in TIME (a
//! `τ_q` knob + a linear mixing schedule) and integrates NO with a CLAMP-FREE integrator
//! (super-equilibrium NO on cooling must not be capped — Heywood). A slow quench dwells at
//! stoich and re-makes the NO a rich primary avoided; a fast quench escapes past the peak.
//! Still a pure diagnostic: bit-for-bit rung 6.
//!
//! Gates (`docs/rung10-spec.md`), priority order:
//!
//! 1. **reduce-to-rung-9 (LOAD-BEARING, exact by construction)** — no `tau_q` short-circuits to
//!    the rung-9 path; `ei_no`/`x_no_mix`/`T_primary`/`T_mix` bit-for-bit, the four quench
//!    fields `None`.
//! 2. **the smoking gun** — `T(β)` rises through the stoich peak for a RICH primary
//!    (`T_peak > T_primary`) and is monotone for lean/stoich (`T_peak == T_primary`).
//! 3. **the NO spike vs `τ_q`** — `ei_no_quenched` rises MONOTONICALLY with `τ_q`.
//! 4. **the finite-quench bell re-fills the rich flank** — to a ~φ_p-independent floor (NO
//!    re-made at the stoich crossing, not carried from the primary).
//! 5. **clamp dormancy is GUARDED** — `max_a_quench < 1` across the in-scope sweep.
//! 6. **the K-check + trace guard bind along the WHOLE trajectory** (asserted at every β).
//! 7. **the soot-bound guard** (φ_p ≤ 2.0) still trips, with a finite `τ_q`.
//!
//! The Python gates run at `ngrid = 32` because the SHAPE is settled there and a 240-point
//! build is ~25 s; these run at 33, which puts β on exact 1/32ths and matches the oracle.

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{equilibrium_composition, f_stoich, hf_fuel_default, Gas};
use turbojet::nox::{
    kcheck_ratio, primary_aft, quench_no, quench_trajectory, thermal_no, QuenchOpts, QuenchPoint,
    QuenchResult, ZonedNoxOpts,
};

const TAU: f64 = 3e-3;
const NG: usize = 33;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}
fn losses() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
        ..Losses::default()
    }
}
fn opts() -> ZonedNoxOpts {
    ZonedNoxOpts { tau: TAU, quench_ngrid: NG, ..ZonedNoxOpts::default() }
}

/// `(gas, Tt3, Tt4, far, pt4)` off a real equilibrium run — NO is trace, so bit-for-bit rung 6.
fn design_point() -> (Gas, f64, f64, f64, f64) {
    let g = Gas::reacting_equilibrium();
    let r = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, flight().p0, losses())
        .run(&flight(), 50.0);
    (g, r.station("3").tt, r.station("4").tt, r.station("4").far, r.station("4").pt)
}

/// The τ_q-INDEPENDENT trajectory, built ONCE per φ_p — the Python's `_reusable_traj`. The fast
/// chemistry is a function of β alone, so a whole τ_q sweep rides one build.
struct Traj {
    comp: Vec<(&'static str, f64)>,
    t_p: f64,
    alpha: f64,
    n0: f64,
    tab: Vec<QuenchPoint>,
    ei9: f64,
    tt3: f64,
    far: f64,
    p: f64,
}

fn reusable_traj(phi_p: f64) -> Traj {
    let (_g, tt3, _tt4, far, p) = design_point();
    let far_p = phi_p * f_stoich();
    let alpha = far / far_p;
    let t_p = primary_aft(far_p, p, tt3, hf_fuel_default());
    let comp = equilibrium_composition(far_p, t_p, p);
    let nox = thermal_no(&comp, t_p, p, TAU, far_p, 4000, 1.0);
    let ntot: f64 = comp.iter().map(|&(_, v)| v).sum();
    let n0 = alpha * nox.x_no * ntot;
    let tab = quench_trajectory(&comp, t_p, alpha, far, tt3, p, NG);
    Traj { comp, t_p, alpha, n0, tab, ei9: nox.ei_no, tt3, far, p }
}

impl Traj {
    fn quench(&self, tau_q: f64) -> QuenchResult {
        quench_no(
            &self.comp, self.t_p, self.alpha, self.far, self.tt3, self.p, self.n0, tau_q,
            QuenchOpts { ngrid: NG, tab: Some(&self.tab), ..QuenchOpts::default() },
        )
    }
}

// --------------------------------------------------------------------------------------
// GATE 1 — reduce-to-rung-9: no tau_q is exact (the short-circuit).
// --------------------------------------------------------------------------------------

/// `tau_q: None, mixing: None` must run the EXACT rung-9 path: the five quench fields stay
/// `None` and the rung-9 outputs are byte-identical whether or not the rung-10 branch exists.
///
/// In Rust the short-circuit is `if o.tau_q.is_none() && o.mixing.is_none() { return state }`,
/// so this is exact BY CONSTRUCTION — the same property the spike measured 28/28 for the
/// ladder. The test is still worth its cost because it pins the FIELDS: a port that populated
/// `tau_q` before the early return would leave every rung-9 number right and the contract
/// broken.
#[test]
fn reduce_ideal_quench_is_bit_for_bit_rung9() {
    let (g, tt3, tt4, far, p) = design_point();
    for phi_p in [0.8, 1.0, 1.5, 2.0] {
        let a = g.zoned_nox(far, tt3, tt4, p, phi_p, ZonedNoxOpts::default());
        let b = g.zoned_nox(far, tt3, tt4, p, phi_p, opts());
        for s in [&a, &b] {
            assert!(s.tau_q.is_none() && s.ei_no_quenched.is_none());
            assert!(s.x_no_quenched.is_none() && s.t_peak.is_none() && s.max_a_quench.is_none());
            assert!(s.mixing.is_none() && s.unmixedness.is_none());
        }
        assert_eq!(a.ei_no().to_bits(), b.ei_no().to_bits());
        assert_eq!(a.x_no_mix.to_bits(), b.x_no_mix.to_bits());
        assert_eq!(a.t_primary.to_bits(), b.t_primary.to_bits());
        assert_eq!(a.t_mix.to_bits(), b.t_mix.to_bits());
    }
}

/// A finite quench is still a pure diagnostic — it must not perturb the cycle.
#[test]
fn cycle_untouched_by_finite_quench() {
    let run = || {
        build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, flight().p0, losses())
            .run(&flight(), 50.0)
    };
    let r1 = run();
    let (tt3, tt4, far1, p) =
        (r1.station("3").tt, r1.station("4").tt, r1.station("4").far, r1.station("4").pt);
    let g = Gas::reacting_equilibrium();
    g.zoned_nox(far1, tt3, tt4, p, 1.5, ZonedNoxOpts { tau_q: Some(3e-3), ..opts() });
    assert_eq!(run().station("4").far.to_bits(), far1.to_bits(),
               "finite quench perturbed the cycle far — must stay rung-6");
}

// --------------------------------------------------------------------------------------
// GATE 2 — the smoking gun: T(β) rises through the stoich peak (rich primary).
// --------------------------------------------------------------------------------------

/// ONE public finite call at a rich φ_p, checking three things at once: the SMOKING GUN
/// (`T_peak` rises well above the primary AFT and sits at the slightly-rich stoich bell peak),
/// the WIRING (the quench fields are populated), and that the rung-9 ideal scalars are
/// UNTOUCHED — the finite quench is additive, not a replacement.
#[test]
fn public_wiring_and_rich_smoking_gun() {
    let (g, tt3, tt4, far, p) = design_point();
    let ideal = g.zoned_nox(far, tt3, tt4, p, 1.5, opts());
    let z = g.zoned_nox(far, tt3, tt4, p, 1.5, ZonedNoxOpts { tau_q: Some(1e-3), ..opts() });
    let t_peak = z.t_peak.expect("finite quench populates T_peak");
    assert!(t_peak > z.t_primary + 100.0,
            "rich φ_p=1.5: T_peak {t_peak:.1} must RISE well above T_primary {:.1}", z.t_primary);
    assert!((2400.0..2500.0).contains(&t_peak),
            "rich peak T {t_peak:.1} not at the stoich AFT maximum");
    assert!(z.ei_no_quenched.is_some() && z.max_a_quench.is_some());
    assert_eq!(z.tau_q.expect("tau_q").to_bits(), 1e-3f64.to_bits());
    assert_eq!(z.ei_no().to_bits(), ideal.ei_no().to_bits());
    assert_eq!(z.x_no_mix.to_bits(), ideal.x_no_mix.to_bits());
    assert_eq!(z.t_primary.to_bits(), ideal.t_primary.to_bits());
    assert_eq!(z.t_mix.to_bits(), ideal.t_mix.to_bits());
}

/// The other half of the smoking gun: a LEAN/stoich primary starts AT (or above) the peak, so
/// the quench only cools it — the trajectory T is monotone-falling and `T_peak == T(β=0)`.
#[test]
fn trajectory_monotone_fall_for_lean_stoich_primary() {
    let t = reusable_traj(1.0);
    let ts: Vec<f64> = t.tab.iter().map(|r| r.t).collect();
    let max = ts.iter().cloned().fold(f64::MIN, f64::max);
    assert_eq!(max.to_bits(), ts[0].to_bits(),
               "lean/stoich trajectory must be monotone-falling (peak at β=0): {:.1}", ts[0]);
    for w in ts.windows(2) {
        assert!(w[1] <= w[0] + 1e-9, "lean/stoich T must not rise along β");
    }
}

// --------------------------------------------------------------------------------------
// GATE 3 — the NO spike: monotone in τ_q; a slow quench re-makes NO.
// --------------------------------------------------------------------------------------

/// THE lesson: a rich primary that "avoided" NO re-makes it as the quench slows and the gas
/// dwells at the stoich crossing. One trajectory serves the whole sweep.
#[test]
fn no_spike_rises_monotonically_with_tau_q() {
    let t = reusable_traj(1.5);
    let taus = [1e-5, 1e-4, 1e-3, 3e-3, 1e-2];
    let eis: Vec<f64> = taus.iter().map(|&tq| t.quench(tq).ei).collect();
    for w in eis.windows(2) {
        assert!(w[1] > w[0], "EI_NO must rise monotonically with τ_q: {eis:?}");
    }
    // The spike spans orders of magnitude across the sweep — NOT a τ_q→0 reduce check (the
    // EXACT reduce is the short-circuit in gate 1; at φ_p=1.5 the rung-9 EI is ~0.001 g/kg, so
    // even a 0.01 ms window already re-makes several times that, the tiny-denominator artifact
    // the anchor warns not to chase).
    assert!(eis[4] > 50.0 * eis[0], "the spike must span orders of magnitude in τ_q: {eis:?}");
    assert!(eis[3] > 100.0 * t.ei9,
            "a slow (3 ms) quench must re-make ≫ the rung-9 frozen NO ({:.4e})", t.ei9);
}

// --------------------------------------------------------------------------------------
// GATE 4 — the finite-quench bell re-fills the rich flank.
// --------------------------------------------------------------------------------------

/// Rung-9 ideal EI_NO collapses on the rich flank. A 3 ms quench fills it back to a
/// ~φ_p-independent floor — because EVERY rich mixture passes through the SAME stoich peak on
/// the way down. This discriminates "NO re-made at the crossing" from "NO carried from the
/// primary", which is the whole content of rung 10.
#[test]
fn finite_quench_refills_the_rich_flank() {
    let (g, tt3, tt4, far, p) = design_point();
    let mut floor: Vec<f64> = Vec::new();
    for phi_p in [1.3, 1.5, 1.8] {
        let ideal = g.zoned_nox(far, tt3, tt4, p, phi_p, opts()).ei_no();
        let q = reusable_traj(phi_p).quench(3e-3).ei;
        assert!(q > 20.0 * ideal.max(1e-9),
                "φ_p={phi_p}: quench must re-fill the collapsed rich flank ({q:.3} vs ideal \
                 {ideal:.4e})");
        assert!((1.0..6.0).contains(&q),
                "φ_p={phi_p}: refilled floor {q:.3} outside the expected ~3 g/kg band");
        floor.push(q);
    }
    let (lo, hi) = (floor.iter().cloned().fold(f64::MAX, f64::min),
                    floor.iter().cloned().fold(f64::MIN, f64::max));
    assert!(hi < 2.0 * lo, "rich-flank floor not ~φ_p-independent: {floor:?}");
}

// --------------------------------------------------------------------------------------
// GATE 5 — clamp dormancy is GUARDED (max_a < 1 across the in-scope sweep).
// --------------------------------------------------------------------------------------

/// The dropped equilibrium clamp is correct-on-principle but DORMANT-on-numbers at this lean
/// design point: NO lags BELOW equilibrium the whole way. Guarding the whole in-scope sweep is
/// what makes a future super-equilibrium operating point FLAG the regime change instead of
/// silently passing — that is the teaching payoff of exposing `max_a` at all.
#[test]
fn clamp_dormancy_max_a_below_one() {
    let mut overall = 0.0f64;
    for phi_p in [0.9, 1.0, 1.1, 1.5, 2.0] {
        let t = reusable_traj(phi_p);
        for tau_q in [1e-3, 1e-2] {
            overall = overall.max(t.quench(tau_q).max_a);
        }
    }
    assert!(overall < 1.0,
            "max_a={overall:.3} ≥ 1 — the super-eq regime; the dropped clamp is now load-bearing");
    assert!(overall > 0.5,
            "max_a={overall:.3} unexpectedly small — the dormancy sweep may be mis-sampled");
}

// --------------------------------------------------------------------------------------
// GATE 6 — the K-check + trace guard bind along the WHOLE trajectory.
// --------------------------------------------------------------------------------------

/// `quench_trajectory` asserts the K-check AND the trace guard at every β, so a passing finite
/// call IS the gate. This checks the constant directly across the trajectory's FULL T range —
/// from the peak down to the cold mixed-out T, the coldest temperature the quench visits and
/// the one rung 7's single-T check never saw.
#[test]
fn kcheck_binds_along_the_trajectory() {
    let t = reusable_traj(1.5);
    let ts: Vec<f64> = t.tab.iter().map(|r| r.t).collect();
    let (lo, hi) = (ts.iter().cloned().fold(f64::MAX, f64::min),
                    ts.iter().cloned().fold(f64::MIN, f64::max));
    for tt in [lo, hi] {
        let r = kcheck_ratio(tt);
        assert!(0.90 < r && r < 1.15, "K-check {r:.4} at trajectory T={tt:.0} K out of band");
    }
    assert!(lo < 1600.0 && hi > 2400.0,
            "trajectory should span the cold mix ({lo:.0} K) to the stoich peak ({hi:.0} K)");
}

// --------------------------------------------------------------------------------------
// GATE 7 — the scope guards still trip, with a finite τ_q.
// --------------------------------------------------------------------------------------

/// The φ_p ≤ 2 soot guard fires at the TOP of `zoned_nox`, before any trajectory is built, so
/// it is independent of `τ_q`.
#[test]
fn soot_bound_guard_with_finite_quench() {
    let (g, tt3, tt4, far, p) = design_point();
    g.zoned_nox(far, tt3, tt4, p, 2.0, opts()); // at the bound: accepted
    for bad in [2.2, 3.0] {
        let r = std::panic::catch_unwind(|| {
            let (g, tt3, tt4, far, p) = design_point();
            g.zoned_nox(far, tt3, tt4, p, bad, ZonedNoxOpts { tau_q: Some(3e-3), ..opts() })
        });
        assert!(r.is_err(), "φ_p={bad} > 2 should be rejected (soot / C(s) basis limit)");
    }
}

/// A non-positive `tau_q` is rejected — `None` is how the ideal quench is spelled, and the
/// distinction matters because `Some(0.0)` would divide by zero in the β map rather than
/// short-circuiting.
#[test]
fn non_positive_tau_q_rejected() {
    for bad in [0.0, -1e-3] {
        let r = std::panic::catch_unwind(|| {
            let (g, tt3, tt4, far, p) = design_point();
            g.zoned_nox(far, tt3, tt4, p, 1.5, ZonedNoxOpts { tau_q: Some(bad), ..opts() })
        });
        assert!(r.is_err(), "tau_q={bad} should be rejected (use None for the ideal quench)");
    }
}
