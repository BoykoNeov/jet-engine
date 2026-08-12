//! Rung-29 verification: THE SHIFTING TURBINE — is FREEZING the turbine earned?
//!
//! Every rung since 6 FREEZES the station-4 mixture through the turbine; rungs 14/25 then read
//! that frozen pool at the nozzle entry and call it super-equilibrium — the premise the whole
//! `(R−I)` entry-irreversibility gap rests on. This brackets the turbine the way rung 14 bracketed
//! the nozzle: frozen vs fully-shifting, at the SAME shaft-set `Δh`. Zero knobs, no rate.
//!
//! **THE ENDPOINT IS WORK-LIMITED, not pressure-limited** — the shaft fixes the enthalpy drop, so
//! both expansions give up the same work and what the chemistry changes is where they end up.
//!
//! **THE INVERSION WORTH CARRYING IS RATIO ≠ ENERGY.** The frozen station-5 pool is
//! super-equilibrium by a huge RATIO, which is what rungs 14/25 read off it; what a shifting
//! turbine can actually exploit is the radical INVENTORY, which is far smaller. This suite gates
//! that the two move independently — the point being that a bound sitting on an enormous ratio can
//! still be worth almost nothing in energy.

use turbojet::components::Turbine;
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{equilibrium_composition, FlowState, Gas};
use turbojet::march::{work_limited_expand, ShiftingTurbineState, SHIFT_EARNED_TOL};
use turbojet::nox::mix_mass_per_air;

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

struct Dp {
    gas: Gas,
    far: f64,
    tt4: f64,
    pt4: f64,
    delta_h: f64,
    cycle_v9: f64,
}

fn dp(tt4: f64) -> Dp {
    let eng = build_turbojet(Gas::reacting_equilibrium(), PI_C, tt4, 50_000.0, losses());
    let r = eng.run(&flight(), 1.0);
    let (s2, s3, s4) = (r.station("2"), r.station("3"), r.station("4"));
    let delta_h = (eng.gas.h_c(s3.tt) - eng.gas.h_c(s2.tt)) / (0.99 * (1.0 + s4.far));
    Dp { far: s4.far, tt4: s4.tt, pt4: s4.pt, delta_h, cycle_v9: r.v9, gas: eng.gas }
}

fn st(d: &Dp) -> ShiftingTurbineState {
    d.gas.shifting_turbine(d.far, d.tt4, d.pt4, d.delta_h)
}

// --- GATE 1: THE REDUCE — the frozen bound IS the shipped turbine, by construction ------------ //

/// (F) is the production turbine at `η_t = 1`, taken verbatim rather than re-solved — so the
/// bound cannot drift from the cycle it is bracketing.
///
/// Gated against an actual `Turbine` rather than against the same two lines retyped: that is the
/// difference between checking the delegation happened and asserting a function equals itself.
#[test]
fn the_frozen_bound_is_the_shipped_turbine_bit_for_bit() {
    for tt4 in [1500.0, 2200.0] {
        let d = dp(tt4);
        let s = st(&d);
        let ideal = Turbine::new(1.0, None); // eta_t = 1: the reversible bracket
        let inlet = FlowState { tt: d.tt4, pt: d.pt4, mdot: 1.0, far: d.far };
        let out = ideal.apply(&inlet, &d.gas, d.delta_h);
        assert_eq!(s.t5_frozen.to_bits(), out.tt.to_bits(), "T5 differs at Tt4={tt4}");
        assert_eq!(s.p5_frozen.to_bits(), out.pt.to_bits(), "p5 differs at Tt4={tt4}");
    }
}

/// …and the SOLVER's frozen branch — which production never takes — lands on the same expansion.
///
/// It converges rather than matching bit-for-bit: it is a bisection where the other is a closed
/// form, so this is the non-tautological check that the two-level solve is solving the right
/// problem. Without it the `shifting = false` arm would be unreachable code in the port.
#[test]
fn the_solver_frozen_branch_agrees_with_the_closed_form() {
    for tt4 in [1500.0, 2200.0] {
        let d = dp(tt4);
        let s = st(&d);
        let ce = equilibrium_composition(d.far, d.tt4, d.pt4);
        let m = mix_mass_per_air(&ce);
        let (t5, p5, _) = work_limited_expand(&ce, d.far, d.tt4, d.pt4, d.delta_h * m, false);
        assert!(
            (t5 - s.t5_frozen).abs() < 1e-8 * s.t5_frozen,
            "solved T5 {t5} vs closed-form {} at Tt4={tt4}",
            s.t5_frozen
        );
        assert!(
            (p5 - s.p5_frozen).abs() < 1e-6 * s.p5_frozen,
            "solved p5 {p5} vs closed-form {} at Tt4={tt4}",
            s.p5_frozen
        );
    }
}

// --- GATE 2: DIRECTION — recombination reheats, so the shifting exit is warmer and higher ----- //

#[test]
fn the_shifting_exit_is_warmer_and_at_higher_pressure() {
    for tt4 in [1300.0, 1500.0, 1800.0, 2200.0, 2300.0] {
        let s = st(&dp(tt4));
        assert!(s.t5_shifting >= s.t5_frozen, "shifting exit cooler at Tt4={tt4}");
        assert!(s.p5_shifting >= s.p5_frozen, "shifting exit at lower pressure at Tt4={tt4}");
        assert!(s.dt5() >= 0.0 && s.dp5_fraction() >= 0.0);
    }
}

// --- GATE 3: THE VERDICT — EARNED at design, BITES hot ---------------------------------------- //

/// The bracket's move rises monotonically with `Tt4` and crosses the earned threshold, so the
/// verdict MOVES along the ladder — which is what makes it a verdict rather than a constant.
#[test]
fn the_freeze_is_earned_at_design_and_bites_hot() {
    let mut fracs: Vec<f64> = Vec::new();
    let mut earned: Vec<bool> = Vec::new();
    for tt4 in [1300.0, 1500.0, 1800.0, 2200.0, 2300.0] {
        let s = st(&dp(tt4));
        fracs.push(s.dt5_fraction());
        earned.push(s.frozen_turbine_earned());
    }
    assert!(fracs.windows(2).all(|w| w[0] < w[1]), "dT5/T5 not rising with Tt4: {fracs:?}");
    assert!(earned[1], "the freeze must be EARNED at the 1500 K design point");
    assert!(!earned[4], "the freeze must BITE by 2300 K — else the verdict never moves");
    // …and the threshold is the one the code uses, not a number retyped here.
    assert!(fracs[1].abs() < SHIFT_EARNED_TOL);
    assert!(fracs[4].abs() > SHIFT_EARNED_TOL);
}

// --- GATE 4: RATIO ≠ ENERGY — the headline inversion ------------------------------------------ //

/// **The ratio is LARGEST exactly where the bracket is worth LEAST.** The two currencies move in
/// OPPOSITE directions along the ladder, while the radical INVENTORY tracks the energy almost
/// exactly. Measured:
///
/// ```text
/// Tt4                  1300      1500      1800      2200      2300      span
/// dT5/T5             1.69e-05  1.07e-04  7.65e-04  6.13e-03  1.04e-02   x616  (rises)
/// super-eq ratio        993.5     109.4      17.7      5.21      4.25   /234  (FALLS)
/// radical inventory  3.85e-06  3.18e-05  3.11e-04  2.28e-03  3.19e-03   x828  (rises)
/// ```
///
/// This is stronger than "the ratio does not predict the energy": it is ANTI-correlated with it.
/// A bound sitting on a 993× super-equilibrium ratio is worth 1.7e-05 in temperature, and the one
/// worth 600× more sits on a ratio 234× SMALLER. The inventory — what a shifting expansion can
/// actually burn — is the currency that tracks the answer, and it is the one rungs 14/25 do not
/// read.
///
/// Gated as the correlation rather than as bounds on each series, because a threshold on either
/// alone would pass on a model where both rose together, which is precisely the reading rung 29
/// corrected.
#[test]
fn the_super_equilibrium_ratio_is_anti_correlated_with_the_energy() {
    let (mut ratios, mut fracs, mut inventories) = (Vec::new(), Vec::new(), Vec::new());
    for tt4 in [1300.0, 1500.0, 1800.0, 2200.0, 2300.0] {
        let s = st(&dp(tt4));
        ratios.push(s.super_eq_ratio_max);
        fracs.push(s.dt5_fraction());
        inventories.push(s.radical_inventory);
    }

    // THE INVERSION: the ratio FALLS monotonically while the energy RISES monotonically.
    assert!(
        ratios.windows(2).all(|w| w[0] > w[1]),
        "the super-eq ratio should FALL with Tt4: {ratios:?}"
    );
    assert!(fracs.windows(2).all(|w| w[0] < w[1]), "the energy should RISE with Tt4: {fracs:?}");

    // …and the ratio is largest exactly where the bracket is worth least.
    assert!(ratios[0] > 100.0 * ratios[4], "the ratio's fall is not decisive: {ratios:?}");
    assert!(fracs[4] > 100.0 * fracs[0], "the energy's rise is not decisive: {fracs:?}");

    // THE CURRENCY THAT DOES TRACK IT: the radical inventory rises with the energy, and by a
    // comparable factor (828x against 616x), where the ratio moves 234x the other way.
    assert!(
        inventories.windows(2).all(|w| w[0] < w[1]),
        "the inventory should rise with Tt4: {inventories:?}"
    );
    let inv_span = inventories[4] / inventories[0];
    let energy_span = fracs[4] / fracs[0];
    assert!(
        (inv_span / energy_span).abs() > 0.2 && (inv_span / energy_span).abs() < 5.0,
        "the inventory should track the energy within a small factor: {inv_span:e} vs \
         {energy_span:e}"
    );
    assert!(inventories.iter().all(|&x| x < 1e-2), "inventory not small: {inventories:?}");
}

// --- GATE 5: BOTH EXPANSIONS GAVE UP THE SAME WORK, and the cycle is untouched ---------------- //

/// The work-limited endpoint, verified from OUTSIDE the method that asserts it internally.
#[test]
fn both_expansions_extract_the_same_shaft_work() {
    for tt4 in [1500.0, 2200.0] {
        let d = dp(tt4);
        let s = st(&d);
        let ce = equilibrium_composition(d.far, d.tt4, d.pt4);
        let m = mix_mass_per_air(&ce);
        // The shifting side, recomputed here on ABSOLUTE enthalpy — the composition changes, so
        // the formation enthalpy does not cancel out of the work balance.
        let w = (turbojet::nox::mix_h_abs_b(&ce, d.tt4)
            - turbojet::nox::mix_h_abs_b(
                &equilibrium_composition(d.far, s.t5_shifting, s.p5_shifting),
                s.t5_shifting,
            ))
            / m;
        assert!((w - d.delta_h).abs() < 1e-6 * d.delta_h, "work {w} != delta_h {}", d.delta_h);
    }
}

#[test]
fn cycle_untouched() {
    let d = dp(2200.0);
    let (far_before, v9_before) = (d.far, d.cycle_v9);
    let _ = st(&d);
    let r = build_turbojet(Gas::reacting_equilibrium(), PI_C, 2200.0, 50_000.0, losses())
        .run(&flight(), 1.0);
    assert_eq!(r.station("4").far.to_bits(), far_before.to_bits());
    assert_eq!(r.v9.to_bits(), v9_before.to_bits());
}

// ============================================================================================= //
// THE TWO MARGIN AXES — rung 29's "one design point" concession, re-checked where it was named.
//
// `test_rung29.py` is the largest suite in phase 4 (422 lines) because it carries BOTH margin
// sweeps as gates rather than as separate documents: the π_c axis (a CONFIRMATION plus a
// SHARPENING) and the M0 axis (a CONFIRMATION plus a CORRECTION to the π_c doc's framing).
//
// **These ten gates were missed on the first pass of this port**, which read `gas.py`'s docstrings
// and the spec but never opened the test file — so slice H shipped 6 of 16 rung-29 gates while its
// oracle read 270/270. That is the distinction worth keeping: **an oracle gates VALUES, and a
// missing gate is a missing CLAIM, not a wrong number.** No amount of bit-equality could have
// surfaced it.
// ============================================================================================= //

/// A run at `(Tt4, π_c, M0)`, or `None` when the cycle does not solve.
///
/// The Python wraps this in `except Exception` because two edges of the envelope legitimately have
/// no solution — the low-M0 ram edge and the high-`Tt4` equilibrium-burner ceiling — and NEITHER is
/// the turbine. Rust's equivalent is `catch_unwind`, since those failures arrive as conservation
/// asserts from inside the engine rather than as a returned error. The panic hook is silenced for
/// the duration so an EXPECTED non-solution does not print a backtrace and read as a failure.
fn try_run(tt4: f64, pi_c: f64, m0: f64) -> Option<(Gas, f64, f64, f64, f64)> {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = std::panic::catch_unwind(|| {
        let eng = build_turbojet(Gas::reacting_equilibrium(), pi_c, tt4, 50_000.0, losses());
        let fc = FlightCondition::new(250.0, 50_000.0, m0);
        let r = eng.run(&fc, 1.0);
        let (s2, s3, s4) = (r.station("2"), r.station("3"), r.station("4"));
        let delta_h = (eng.gas.h_c(s3.tt) - eng.gas.h_c(s2.tt)) / (0.99 * (1.0 + s4.far));
        (eng.gas, s4.far, s4.tt, s4.pt, delta_h)
    })
    .ok();
    std::panic::set_hook(prev);
    out
}

/// The bracket at `(Tt4, π_c, M0)`. Panics if the cycle does not solve — use [`try_run`] where
/// that is a question.
fn bracket(tt4: f64, pi_c: f64, m0: f64) -> (Gas, f64, ShiftingTurbineState) {
    let (gas, far, t4, p4, dh) = try_run(tt4, pi_c, m0).expect("the cycle should solve here");
    let s = gas.shifting_turbine(far, t4, p4, dh);
    (gas, far, s)
}

/// Fraction of the entry radical inventory that equilibrium at the FROZEN exit state wants gone —
/// how much of the pool the expansion actually ASKS for.
///
/// This is the second half of the currency: rung 29 proposed the entry INVENTORY as what the
/// super-equilibrium RATIO failed to be, and the π_c axis shows inventory alone fails the same way.
/// `inventory × completion` — the RECOMBINED inventory — is what actually tracks the shift.
fn completion(tt4: f64, pi_c: f64, m0: f64) -> f64 {
    let (_, far, s) = bracket(tt4, pi_c, m0);
    let c5 = equilibrium_composition(far, s.t5_frozen, s.p5_frozen);
    let n5: f64 = c5.iter().map(|&(_, n)| n).sum();
    let inv5: f64 = c5
        .iter()
        .filter(|&&(sp, _)| sp == "O" || sp == "H" || sp == "OH")
        .map(|&(_, n)| n)
        .sum::<f64>()
        / n5;
    1.0 - inv5 / s.radical_inventory
}

const PI_C_SCAN: [f64; 5] = [2.0, 5.0, 10.0, 20.0, 80.0];
const M0_SCAN: [f64; 5] = [0.3, 0.85, 1.6, 2.5, 3.0];

// --- THE π_c AXIS: a CONFIRMATION plus a SHARPENING ------------------------------------------ //

/// The verdict survives the `π_c` axis with margin, and the earned/not-earned boundary stays far
/// above the design point everywhere on it.
#[test]
fn earned_at_design_is_pi_c_robust() {
    for pi_c in PI_C_SCAN {
        let (_, _, s) = bracket(1500.0, pi_c, 0.85);
        assert!(s.frozen_turbine_earned(), "pi_c={pi_c}: design point should stay EARNED");
        assert!(
            s.dt5_fraction().abs() < 2e-4,
            "pi_c={pi_c}: design bound drifted: {:.3e}",
            s.dt5_fraction()
        );
        assert!(
            bracket(1800.0, pi_c, 0.85).2.frozen_turbine_earned(),
            "pi_c={pi_c}: Tt4=1800 K should still be earned"
        );
        assert!(
            !bracket(2200.0, pi_c, 0.85).2.frozen_turbine_earned(),
            "pi_c={pi_c}: Tt4=2200 K should NOT be earned"
        );
    }
}

/// The mechanism: raising `π_c` CUTS the inventory but RAISES how much of it gets spent. Two
/// opposed, comparable channels — which is what makes the shift's `π_c` dependence non-monotone.
#[test]
fn pi_c_channels_oppose() {
    let inv: Vec<f64> =
        PI_C_SCAN.iter().map(|&p| bracket(1500.0, p, 0.85).2.radical_inventory).collect();
    let comp: Vec<f64> = PI_C_SCAN.iter().map(|&p| completion(1500.0, p, 0.85)).collect();
    assert!(
        inv.windows(2).all(|w| w[0] > w[1]),
        "inventory should FALL with pi_c (pressure suppresses dissociation): {inv:?}"
    );
    assert!(
        comp.windows(2).all(|w| w[0] < w[1]),
        "completion should RISE with pi_c (deeper, colder expansion): {comp:?}"
    );
    // Comparable magnitudes — neither channel is a rounding correction to the other.
    let (i_swing, c_swing) = (inv[0] / inv[4], comp[4] / comp[0]);
    assert!(i_swing > 2.0 && i_swing < 6.0, "inventory swing {i_swing}");
    assert!(c_swing > 2.0 && c_swing < 4.0, "completion swing {c_swing}");
}

/// FORBID the β-style reading. Unlike rung 28's β — which fell monotonically in `π_c` — the shift
/// TURNS OVER: it rises from `π_c` = 2 to ~10 and falls again out to 80.
#[test]
fn pi_c_is_not_simply_protective() {
    let lo = bracket(1800.0, 2.0, 0.85).2.dt5_fraction();
    let mid = bracket(1800.0, 10.0, 0.85).2.dt5_fraction();
    let hi = bracket(1800.0, 80.0, 0.85).2.dt5_fraction();
    assert!(lo < mid, "the shift should RISE from pi_c 2→10 (so pi_c is NOT protective)");
    assert!(hi < mid, "the shift should FALL from pi_c 10→80 — an INTERIOR maximum");
    assert!(mid / lo > 1.5, "the low-side rise should be substantial: {}", mid / lo);
}

/// **THE SHARPENING.** Rung 29 proposed the absolute radical INVENTORY as the currency the
/// super-equilibrium RATIO failed to be. On the `Tt4` axis that reads correctly. On the `π_c` axis
/// it does NOT: inventory falls while the shift rises — the same failure mode, now committed by
/// the replacement. The complete currency is inventory × COMPLETION.
#[test]
fn inventory_alone_fails_on_the_pi_c_axis() {
    let (_, _, lo) = bracket(1500.0, 2.0, 0.85);
    let (_, _, hi) = bracket(1500.0, 10.0, 0.85);
    assert!(hi.radical_inventory < lo.radical_inventory, "inventory should fall from pi_c 2→10");
    assert!(hi.dt5_fraction() > lo.dt5_fraction(), "yet the shift RISES — inventory alone fails");
    let rec_lo = lo.radical_inventory * completion(1500.0, 2.0, 0.85);
    let rec_hi = hi.radical_inventory * completion(1500.0, 10.0, 0.85);
    assert!(rec_hi > rec_lo, "the RECOMBINED inventory should track the shift: {rec_lo} {rec_hi}");
}

// --- THE M0 AXIS: a CONFIRMATION plus a CORRECTION to the π_c framing ------------------------- //

/// The margin-sweep helper reproduces the certified flight anchor BIT-FOR-BIT, and the anchor
/// itself.
///
/// Without this the whole margin family could be measuring a differently-built cycle.
///
/// **THE COMPARISON HAD TO BE RE-AIMED, and the reason is worth stating.** The Python runs two
/// helpers — one for the `π_c` sweep, one for the `M0` sweep — and gates that they agree at the
/// point they share. This port has ONE parameterised `bracket`, so transcribing that assertion
/// literally compares a function with itself: vacuity case #8 again, created by the port's own
/// factorisation, and it passed. What ships instead compares `bracket` against `dp` — which
/// really are two different constructions here (`dp` builds the engine directly, `bracket` goes
/// through `try_run`'s `catch_unwind` wrapper) — so a helper that drifted from the main path
/// would fail.
#[test]
fn the_margin_helper_reproduces_the_certified_flight_anchor() {
    let main_path = st(&dp(1500.0)); // the design-point construction the rest of this file uses
    let swept = bracket(1500.0, PI_C, 0.85).2; // the margin-sweep construction
    assert_eq!(swept.dt5_fraction().to_bits(), main_path.dt5_fraction().to_bits());
    assert_eq!(swept.t5_frozen.to_bits(), main_path.t5_frozen.to_bits());
    assert_eq!(swept.t5_shifting.to_bits(), main_path.t5_shifting.to_bits());
    assert_eq!(swept.delta_h.to_bits(), main_path.delta_h.to_bits());
    // The shipped anchor, as an ABSOLUTE value: 0.01067 % at the design point.
    assert!(
        (swept.dt5_fraction() * 100.0 - 0.01067).abs() < 5e-5,
        "the certified anchor moved: {:.5} %",
        swept.dt5_fraction() * 100.0
    );
}

/// The verdict survives the flight axis too.
#[test]
fn earned_at_design_is_m0_robust() {
    for m0 in M0_SCAN {
        let (_, _, s) = bracket(1500.0, PI_C, m0);
        assert!(s.frozen_turbine_earned(), "M0={m0}: the design point should stay EARNED");
    }
}

/// **Opposite of `π_c`:** the shift is MONOTONE-PROTECTIVE in `M0`, with no interior turnover.
/// That is the `β`-like axis the `π_c` doc's unification framing predicted and `π_c` itself is not.
#[test]
fn the_m0_shift_is_monotone_protective() {
    let fr: Vec<f64> =
        M0_SCAN.iter().map(|&m| bracket(2100.0, PI_C, m).2.dt5_fraction()).collect();
    assert!(fr.windows(2).all(|w| w[0] > w[1]), "the shift should FALL monotonically in M0: {fr:?}");
    assert!(fr[0] / fr[4] > 1.8, "the fall should be substantial: {}", fr[0] / fr[4]);
}

/// The same inventory × completion currency, read where it is LOPSIDED: on the `M0` axis the
/// inventory swings far more than the completion does, which is why there is no turnover.
#[test]
fn the_m0_channels_are_lopsided() {
    let inv: Vec<f64> =
        M0_SCAN.iter().map(|&m| bracket(1500.0, PI_C, m).2.radical_inventory).collect();
    let comp: Vec<f64> = M0_SCAN.iter().map(|&m| completion(1500.0, PI_C, m)).collect();
    assert!(inv.windows(2).all(|w| w[0] > w[1]), "inventory should FALL with M0: {inv:?}");
    assert!(comp.windows(2).all(|w| w[0] < w[1]), "completion should RISE with M0: {comp:?}");
    let (inv_swing, comp_swing) = (inv[0] / inv[4], comp[4] / comp[0]);
    assert!(
        inv_swing > 3.0 * comp_swing,
        "the channels should be LOPSIDED here (unlike pi_c): {inv_swing} vs {comp_swing}"
    );
    let rec: Vec<f64> = inv.iter().zip(&comp).map(|(i, c)| i * c).collect();
    assert!(
        rec.windows(2).all(|w| w[0] > w[1]),
        "the recombined inventory should track the shift: {rec:?}"
    );
}

/// **THE CORRECTION.** The discriminator between the `π_c` turnover and the `M0` monotone is the
/// `Δh` SWING, not completion HEADROOM — proven by the `π_c` = 2 control, which HAS headroom and
/// still goes monotone.
#[test]
fn the_delta_h_swing_not_headroom_is_the_discriminator() {
    let comp_lo = completion(1500.0, 2.0, 0.3);
    assert!(comp_lo < 0.5, "pi_c=2 should leave completion headroom at low M0: {comp_lo}");
    // …and yet at pi_c = 2 the M0 axis is STILL monotone, so headroom is not what decides it.
    let fr2: Vec<f64> =
        M0_SCAN.iter().map(|&m| bracket(1500.0, 2.0, m).2.dt5_fraction()).collect();
    assert!(
        fr2.windows(2).all(|w| w[0] > w[1]),
        "pi_c=2 should still be monotone in M0 despite the headroom: {fr2:?}"
    );
    // The actual discriminator: delta_h barely swings across the M0 axis.
    let dh_lo = try_run(1500.0, PI_C, 0.3).expect("solves").4;
    let dh_hi = try_run(1500.0, PI_C, 3.0).expect("solves").4;
    assert!(dh_hi / dh_lo < 4.0, "delta_h swing across M0: {}", dh_hi / dh_lo);
}

/// **The flight axis is DOUBLE-EDGED**: protective per point, yet ram heating shrinks the earned
/// OPERATING band from both ends. This is the gate that needs [`try_run`]'s `None` — two of its
/// four corners have no cycle solution at all, and that absence IS the measurement.
#[test]
fn the_m0_envelope_band_is_squeezed() {
    assert!(try_run(1200.0, PI_C, 0.3).is_some(), "Tt4=1200 should run at M0=0.3");
    assert!(
        try_run(1200.0, PI_C, 3.0).is_none(),
        "Tt4=1200 should NOT run at M0=3.0 — the ram-lifted floor is above it"
    );
    // The upper end moves the other way: protective per point, so a Tt4 that is NOT earned slow
    // becomes earned fast.
    assert!(!bracket(1900.0, PI_C, 0.3).2.frozen_turbine_earned());
    assert!(bracket(1900.0, PI_C, 3.0).2.frozen_turbine_earned());
    // …and the burner ceiling lifts with M0 too.
    assert!(try_run(2500.0, PI_C, 0.85).is_none(), "Tt4=2500 should fail the burner balance");
    assert!(try_run(2500.0, PI_C, 2.5).is_some(), "…but should solve once ram-heated");
}
