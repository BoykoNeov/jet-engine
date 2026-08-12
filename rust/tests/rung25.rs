//! Rung-25 verification: FINITE-RATE nozzle chemistry — the Damköhler flow BETWEEN rung-14's
//! bounds.
//!
//! Rung 14 gave the frozen↔equilibrium bracket and named the seam: "the real Damköhler-number
//! flow between the bounds." Rung 25 builds it — and the build INVERTS the seam's framing into a
//! THREE-state picture:
//!
//! ```text
//! (F) frozen (Da→0)      — rung-14 lower bound.                    THE REDUCE (exact/convergent).
//! (I) irreversible-fast  — Da→∞, the ATTAINABLE ceiling. Closed    THE KEYSTONE (integrator → I).
//!     (Da→∞)               form: const-(H,pt9) entry re-equilibration then reversible shifting.
//! (R) reversible-shift   — rung-14 upper bound, a STRICT UNREACHABLE ceiling above (I).
//! ```
//!
//! The rung reduces to rung-14 FROZEN and DELIBERATELY does NOT reduce to equilibrium — the
//! `(R−I)` entry-irreversibility gap is the finding (dormant lean, ~7 % of the bracket hot). This
//! file certifies the ROBUST structure, not the gap magnitude or the interior-curve shape, which
//! ride on the cartoon `Da` and the frozen-entry choice.
//!
//! **WHERE THIS SUITE SAYS MORE THAN THE PYTHON'S.**
//!
//! 1. **The monotonicity gate sweeps five design points where the Python's sweeps one**, on the
//!    seven-point `Da` ladder rather than five. `V9(Da)` monotone at one `Tt4` is consistent with
//!    a curve that folds anywhere else on the ladder; measured, it does not.
//! 2. **The three-state ordering is checked at every design point, including the two the Python
//!    never visits** (1300 K, where the gaps are meant to have collapsed, and 2300 K above its
//!    hot end). An ordering that holds only where it was looked at is not an ordering.
//! 3. **The coarse-grid guard is gated on WHICH assert fires**, not merely that one does. Python's
//!    `pytest.raises(AssertionError)` cannot tell the 2nd-law refusal from the exit-temperature
//!    floor guard or a config validation — three different asserts reachable from one call — so
//!    it would pass if the march failed for the wrong reason.
//!
//! Gates, priority order (`docs/rung25-spec.md`).

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{equilibrium_composition, Gas};
use turbojet::march::{finite_rate_expand, irreversible_fast_expand, FiniteRate};
use turbojet::nox::expand_nozzle;

const PI_C: f64 = 10.0;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn losses() -> Losses {
    Losses {
        pi_d: 0.97,
        eta_c: 0.88,
        eta_b: 0.99,
        pi_b: 0.96,
        eta_t: 0.90,
        eta_m: 0.99,
        pi_n: 0.98,
        ..Losses::default()
    }
}

/// One design point's cycle state. Built fresh per `Tt4` — the equilibrium section caches the
/// burn condition it was frozen at, so a shared `Gas` cannot serve two combustor temperatures.
struct Dp {
    gas: Gas,
    far: f64,
    tt4: f64,
    pt4: f64,
    tt9: f64,
    pt9: f64,
    p9: f64,
    cycle_v9: f64,
}

fn dp(tt4: f64) -> Dp {
    let eng = build_turbojet(Gas::reacting_equilibrium(), PI_C, tt4, 50_000.0, losses());
    let r = eng.run(&flight(), 1.0);
    let (s4, s9) = (r.station("4"), r.station("9"));
    Dp {
        far: s4.far,
        tt4: s4.tt,
        pt4: s4.pt,
        tt9: s9.tt,
        pt9: s9.pt,
        p9: r.p9,
        cycle_v9: r.v9,
        gas: eng.gas,
    }
}

fn entry(d: &Dp) -> Vec<(&'static str, f64)> {
    equilibrium_composition(d.far, d.tt4, d.pt4)
}

fn nozzle(d: &Dp, da: f64, nstep: usize) -> turbojet::march::FiniteRateNozzleState {
    d.gas.finite_rate_nozzle(d.far, d.tt4, d.pt4, d.tt9, d.pt9, d.p9, FiniteRate { da, nstep })
}

// --- GATE 1: REDUCE — frozen dispatch == rung-14 frozen, exactly ----------------------------- //

/// (F) is the DISPATCHED rung-14 frozen value — bit-for-bit, not the integrator.
///
/// The failure this catches is a `finite_rate_nozzle` that computed its own lower bound by
/// marching at a tiny `Da` instead of calling the rung-14 expansion: that would agree to several
/// digits and never bit-for-bit.
#[test]
fn frozen_dispatch_is_rung14_exact() {
    let d = dp(2200.0);
    let comp_entry = entry(&d);
    let f = expand_nozzle(&comp_entry, d.far, d.tt9, d.pt9, d.p9, false);
    let fr = nozzle(&d, 3.0, 400);
    assert_eq!(fr.v9_frozen.to_bits(), f.v9.to_bits());
    assert_eq!(fr.t9_frozen.to_bits(), f.t9.to_bits());

    // …and rung-14's own `nozzle_flow` agrees — the method BESIDE us is untouched.
    let nf = d.gas.nozzle_flow(d.far, d.tt4, d.pt4, d.tt9, d.pt9, d.p9, None);
    assert_eq!(fr.v9_frozen.to_bits(), nf.v9_frozen.to_bits());
    assert_eq!(fr.v9_reversible.to_bits(), nf.v9_equilibrium.to_bits());
}

// --- GATE 2: REDUCE — the integrator at Da→0 CONVERGES to (F), 2nd-order in 1/nstep ----------- //

/// The march is a DIFFERENT computation from the dispatched bound, so it converges rather than
/// matching: the gate is that the error shrinks with resolution and lands at the bound.
#[test]
fn integrator_reduces_to_frozen() {
    let d = dp(2200.0);
    let v9f = nozzle(&d, 3.0, 400).v9_frozen;
    let comp_entry = entry(&d);
    let errs: Vec<f64> = [100usize, 400, 1600]
        .iter()
        .map(|&n| {
            let m = finite_rate_expand(&comp_entry, d.far, d.tt9, d.pt9, d.p9, 1e-4, n);
            (m.v9 - v9f).abs()
        })
        .collect();
    assert!(errs[2] < 3e-3, "Da→0 not at the frozen bound: {errs:?}");
    assert!(errs[0] > errs[1] && errs[1] > errs[2], "not converging in 1/nstep: {errs:?}");
}

// --- GATE 3: THE KEYSTONE — the integrator's Da→∞ asymptote == the closed-form (I) ------------ //

/// The marching integrator, pushed to large `Da` (with `Da·ds` kept small for accuracy),
/// converges on the rate-law-INDEPENDENT closed form. That is what certifies (I) as the true
/// finite-rate endpoint rather than an arbitrary construction.
#[test]
fn keystone_integrator_asymptotes_to_irrev_fast() {
    let d = dp(2200.0);
    let comp_entry = entry(&d);
    let (_, v9i, _, _) = irreversible_fast_expand(&comp_entry, d.far, d.tt9, d.pt9, d.p9);
    let m = finite_rate_expand(&comp_entry, d.far, d.tt9, d.pt9, d.p9, 300.0, 1200); // Da·ds = 0.25
    assert!(
        (m.v9 - v9i).abs() < 0.15,
        "integrator Da→∞ {:.4} vs closed-form (I) {v9i:.4}",
        m.v9
    );

    // …and (I) is STRICTLY BELOW (R): the entry-irreversibility gap is real, not a rounding.
    let fr = nozzle(&d, 3.0, 400);
    assert!(fr.v9_frozen < fr.v9_irrev_fast);
    assert!(fr.v9_irrev_fast < fr.v9_reversible);
}

// --- GATE 4: THE THREE-STATE ORDERING + the interior monotone in Da -------------------------- //

/// The ordering at EVERY design point on the ladder, and monotone `V9(Da)` at every one of them.
///
/// The Python checks both at 2200 K alone. An ordering that holds only where it was looked at is
/// not an ordering, and a curve monotone at one `Tt4` can fold at another — so this sweeps the
/// five points the oracle uses, including 1300 K (below where the gaps open) and 2300 K (above
/// the Python's hot end).
#[test]
fn three_state_ordering_and_monotone_everywhere() {
    for tt4 in [1300.0, 1500.0, 1800.0, 2200.0, 2300.0] {
        let d = dp(tt4);
        let fr = nozzle(&d, 3.0, 400);
        assert!(
            fr.v9_frozen <= fr.v9_finite
                && fr.v9_finite <= fr.v9_irrev_fast
                && fr.v9_irrev_fast <= fr.v9_reversible,
            "three-state ordering broken at Tt4={tt4}: F={} < D={} < I={} < R={}",
            fr.v9_frozen,
            fr.v9_finite,
            fr.v9_irrev_fast,
            fr.v9_reversible
        );
        assert!(fr.attainable_gap() >= 0.0 && fr.unreachable_gap() >= 0.0);

        let vs: Vec<f64> = [0.03, 0.3, 1.0, 3.0, 10.0, 30.0, 300.0]
            .iter()
            .map(|&da| nozzle(&d, da, 400).v9_finite)
            .collect();
        assert!(
            vs.windows(2).all(|w| w[0] < w[1]),
            "V9(Da) not monotone at Tt4={tt4}: {vs:?}"
        );
    }
}

/// The interior fraction is strictly inside the bracket where the bracket is OPEN — which is the
/// hot end. Split from the ordering gate because at 1300 K the bracket is closed to within
/// rounding and `finite_filled` is then a ratio of two numbers that are both ~0.
#[test]
fn finite_flow_sits_strictly_inside_the_open_bracket() {
    for tt4 in [1800.0, 2200.0, 2300.0] {
        let fr = nozzle(&dp(tt4), 3.0, 400);
        assert!(
            fr.finite_filled() > 0.0 && fr.finite_filled() < 1.0,
            "finite_filled={} outside (0,1) at Tt4={tt4}",
            fr.finite_filled()
        );
        assert!(fr.attainable_gap() > 0.0 && fr.unreachable_gap() > 0.0);
    }
}

// --- GATE 5: DORMANT LEAN, EARNS ITS KEEP HOT (rung-14's arc) -------------------------------- //

#[test]
fn dormant_lean_earns_keep_hot() {
    let cold = nozzle(&dp(1500.0), 3.0, 400);
    let hot = nozzle(&dp(2200.0), 3.0, 400);
    // Both gaps collapse at the cool lean design point — no entry non-equilibrium to recover.
    assert!(cold.attainable_gap() / cold.v9_frozen < 1e-4);
    assert!(cold.unreachable_gap() / cold.v9_frozen < 1e-4);
    // …and both grow hot; the unreachable (entry-irreversibility) gap is real but the smaller.
    assert!(hot.attainable_gap() > 100.0 * cold.attainable_gap());
    assert!(hot.attainable_gap() > hot.unreachable_gap());
    assert!(hot.unreachable_gap() > 0.0);
}

// --- GATE 6: 2nd LAW — entropy production ≥ 0, → 0 as Da→0 ----------------------------------- //

#[test]
fn entropy_production_nonneg() {
    let d = dp(2200.0);
    let ce = entry(&d);
    for da in [0.3, 1.0, 3.0, 10.0, 30.0] {
        let m = finite_rate_expand(&ce, d.far, d.tt9, d.pt9, d.p9, da, 400);
        assert!(m.ds > -1e-6, "2nd law violated at Da={da}: dS={}", m.ds);
    }
    let m0 = finite_rate_expand(&ce, d.far, d.tt9, d.pt9, d.p9, 1e-4, 400);
    assert!(m0.ds.abs() < 1e-3, "dS should → 0 as Da→0: {}", m0.ds);
}

/// A pathologically coarse grid overshoots — trapezoid truncation drives `dS < 0` and the exit
/// even creeps past the reversible ceiling — so the 2nd-law conservation assert must REFUSE it
/// rather than return a non-physical number.
///
/// **The `expected` string is the point.** Three different asserts are reachable from this one
/// call (the 2nd-law floor, the exit-temperature floor guard, and the config validation), and
/// Python's bare `pytest.raises(AssertionError)` cannot tell them apart — it would pass if the
/// march failed for entirely the wrong reason. Naming the message makes the gate say what it
/// means.
#[test]
#[should_panic(expected = "2nd law violated")]
fn second_law_guard_rejects_coarse_grid() {
    let d = dp(2200.0);
    let ce = entry(&d);
    finite_rate_expand(&ce, d.far, d.tt9, d.pt9, d.p9, 1e-4, 10);
}

/// …and the guard is the COARSE-GRID net, not a large-`Da` net: the exponential relaxation step
/// is unconditionally stable in `Da` (`relax ∈ [0,1]`), so cranking `Da` to 1e6 on a
/// well-resolved grid is safe and stays below the reversible ceiling.
#[test]
fn large_da_on_a_good_grid_is_not_refused() {
    let d = dp(2200.0);
    let ce = entry(&d);
    let m = finite_rate_expand(&ce, d.far, d.tt9, d.pt9, d.p9, 1e6, 400);
    assert!(m.ds > 0.0, "dS={} at Da=1e6, nstep=400", m.ds);
    assert!(m.v9 < nozzle(&d, 3.0, 400).v9_reversible);
}

/// **The energy bisection must CONVERGE, and that is invisible in every value the oracle holds.**
///
/// Each step runs a counted `for _ in 0..200` and then takes `0.5*(lo+hi)` from whatever bracket
/// it ends with. If the stopping rule never fires, that expression still returns a perfectly
/// plausible temperature — the march continues, the exit state is wrong by an amount nobody can
/// name, and `march_oracle.rs` cannot say so either, because the Python would sit at the same
/// unconverged number and the two would agree bit-for-bit.
///
/// § 4.11 probe 1 measured 36–37 halvings across all 70 marches, so the cap has ~5× headroom. The
/// band below is deliberately wider than the measurement (it must not fail when a design point
/// shifts a halving) but far below 200 (it must fail if the rule stops firing).
#[test]
fn the_energy_bisection_converges_far_inside_its_cap() {
    for tt4 in [1300.0, 1500.0, 1800.0, 2200.0, 2300.0] {
        let d = dp(tt4);
        let ce = entry(&d);
        for da in [0.03, 3.0, 300.0] {
            for nstep in [100usize, 400] {
                let m = finite_rate_expand(&ce, d.far, d.tt9, d.pt9, d.p9, da, nstep);
                assert!(
                    m.iters_max < 60,
                    "energy bisection took {} halvings at Tt4={tt4}, Da={da}, nstep={nstep} — \
                     measured 36–37, and 200 means it never converged at all",
                    m.iters_max
                );
                // …and the floor, so a loop that broke IMMEDIATELY (an inverted stopping rule,
                // which would also return a plausible number) fails just as loudly.
                assert!(
                    m.iters_min > 20,
                    "energy bisection took only {} halvings at Tt4={tt4}, Da={da}",
                    m.iters_min
                );
            }
        }
    }
}

// --- GATE 7: ATOM CONSERVATION (the vector-relaxation free invariant) ------------------------ //

/// Each element count is a linear invariant shared by `n` and `n_eq`, so the exact linear
/// relaxation conserves atoms whatever the rate — the free invariant that says the composition
/// step is a relaxation and not an approximation of one.
#[test]
fn atoms_conserved() {
    fn atoms(c: &[(&str, f64)]) -> (f64, f64, f64) {
        let g = |name: &str| c.iter().find(|&&(s, _)| s == name).map_or(0.0, |&(_, n)| n);
        let carbon = g("CO2") + g("CO");
        let hydrogen = 2.0 * g("H2O") + 2.0 * g("H2") + g("OH") + g("H");
        let oxygen =
            2.0 * g("CO2") + g("CO") + g("H2O") + g("OH") + g("O") + 2.0 * g("O2");
        (carbon, hydrogen, oxygen)
    }

    // Every design point and both ends of the rate ladder — atom conservation is `Da`-independent
    // by construction, so a version that held only at one rate would be evidence against it.
    for tt4 in [1300.0, 1500.0, 1800.0, 2200.0, 2300.0] {
        let d = dp(tt4);
        let ce = entry(&d);
        for da in [0.03, 3.0, 300.0] {
            let m = finite_rate_expand(&ce, d.far, d.tt9, d.pt9, d.p9, da, 400);
            let (c0, h0, o0) = atoms(&ce);
            let (c1, h1, o1) = atoms(&m.comp9);
            let worst = (c1 - c0).abs().max((h1 - h0).abs()).max((o1 - o0).abs());
            assert!(worst < 1e-12, "atoms not conserved at Tt4={tt4}, Da={da}: {worst:e}");
        }
    }
}

// --- GATE 8: CYCLE UNTOUCHED (a pure diagnostic) --------------------------------------------- //

#[test]
fn cycle_untouched() {
    let d = dp(2200.0);
    let far_before = d.far;
    let v9_before = d.cycle_v9;
    let _ = nozzle(&d, 3.0, 400);
    // Re-run the cycle: identical, because the diagnostic read only and mutated nothing.
    let r = build_turbojet(Gas::reacting_equilibrium(), PI_C, 2200.0, 50_000.0, losses())
        .run(&flight(), 1.0);
    assert_eq!(r.station("4").far.to_bits(), far_before.to_bits());
    assert_eq!(r.v9.to_bits(), v9_before.to_bits());
}

// --- GATE 9: GUARDS -------------------------------------------------------------------------- //

#[test]
#[should_panic(expected = "must be positive")]
fn guard_da_zero_is_refused() {
    FiniteRate { da: 0.0, nstep: 400 }.validate();
}

#[test]
#[should_panic(expected = "must be positive")]
fn guard_da_negative_is_refused() {
    FiniteRate { da: -1.0, nstep: 400 }.validate();
}

#[test]
#[should_panic(expected = "too coarse")]
fn guard_nstep_below_100_is_refused() {
    FiniteRate { da: 3.0, nstep: 99 }.validate();
}

#[test]
#[should_panic(expected = "needs the rung-6 equilibrium gas")]
fn guard_requires_the_equilibrium_gas() {
    let d = dp(2200.0);
    Gas::thermally_perfect().finite_rate_nozzle(
        d.far,
        d.tt4,
        d.pt4,
        d.tt9,
        d.pt9,
        d.p9,
        FiniteRate { da: 3.0, nstep: 400 },
    );
}

#[test]
#[should_panic(expected = "exceeds pt9")]
fn guard_rejects_back_pressure_above_total() {
    let d = dp(2200.0);
    d.gas.finite_rate_nozzle(
        d.far,
        d.tt4,
        d.pt4,
        d.tt9,
        d.pt9,
        d.pt9 * 1.5,
        FiniteRate { da: 3.0, nstep: 400 },
    );
}
