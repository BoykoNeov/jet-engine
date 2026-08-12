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
