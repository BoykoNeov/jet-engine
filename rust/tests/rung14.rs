//! Rung-14 verification: equilibrium-vs-frozen NOZZLE FLOW — the rung-6 cycle-side seam.
//!
//! The production nozzle FREEZES the station-4 equilibrium mixture through the whole expansion
//! (rungs 6–13). Real nozzle flow lies between FROZEN (chemistry infinitely slow — composition
//! fixed) and EQUILIBRIUM / SHIFTING (chemistry infinitely fast — composition = eq(T,p)
//! everywhere). As the exhaust cools, CO/H₂/OH/O/H recombine to CO₂/H₂O, releasing chemical
//! energy → a HIGHER V9. So equilibrium is an UPPER thrust bound and frozen a LOWER one.
//!
//! TWO complementary lessons (the honest arc, mirroring the rung-10 dropped clamp):
//!
//! * MAJOR-SPECIES / THRUST — the frozen↔equilibrium gap is NEGLIGIBLE at the cool lean design
//!   point (dissociation ≈ 0) and grows with combustor temperature.
//! * NO / THE CLAMP — on the SAME cooling path equilibrium NO COLLAPSES, so any realistic frozen
//!   exhaust NO is super-equilibrium and rung 7's DROPPED clamp earns its keep (`max_a ≫ 1`,
//!   against rung 10's dormant 0.677).
//!
//! **THREE GATES HERE SAY SOMETHING THE PYTHON'S CANNOT, and one it says is dropped.**
//!
//! 1. The reduce bar is **1e-12 RELATIVE, measured**, where the Python's is `1e-6` ABSOLUTE —
//!    six orders above the number it gates. The frozen branch is NOT bit-equal to the production
//!    nozzle at any of eight design points and the residual has a floor that is the entropy
//!    ROUTE rather than the bisection's stopping rule; that measurement lives in
//!    `nozzle_oracle.rs::the_frozen_reduce_is_inexact_and_the_floor_is_the_route`, because it
//!    needs the tolerance as a knob.
//! 2. The monotonicity gate runs **eleven** combustor temperatures where the Python's runs three.
//! 3. **The composition-freeze test is NOT transcribed as an equality.** Python rebinds the
//!    module-global `_equilibrium_composition` to a constant and asserts the shifting branch then
//!    equals the frozen one bit-for-bit. In Rust the `shifting` flag is consumed by the closure
//!    builder, so there is no branch left inside the expansion body and that assertion compares a
//!    function to itself (vacuity case #8 — one this port's own factorisation created; plan
//!    § 4.9). What ships instead keeps the equality only as the SETUP for an arm that can fail:
//!    fed a DIFFERENT constant pool, the same call must MOVE the answer. That is what catches a
//!    body which ignored `comp_at` — a defect that would silently collapse the whole bracket.
//!
//! Gates, priority order (`docs/rung14-spec.md`).

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{equilibrium_composition, Gas};
use turbojet::nox::{expand_nozzle, expand_nozzle_with, mix_entropy_molar, ZonedNoxOpts, TOL_REL};

const PI_C: f64 = 10.0;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn losses() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
        ..Losses::default()
    }
}

/// `(gas, far, Tt3, Tt4, pt4, Tt9, pt9, p9, V9_cycle, T9_cycle)` at one combustor temperature.
struct Dp {
    gas: Gas,
    far: f64,
    tt3: f64,
    tt4: f64,
    pt4: f64,
    tt9: f64,
    pt9: f64,
    p9: f64,
    v9: f64,
    t9: f64,
}

fn dp(tt4: f64, with_losses: bool) -> Dp {
    let l = if with_losses { losses() } else { Losses::default() };
    let eng = build_turbojet(Gas::reacting_equilibrium(), PI_C, tt4, 50_000.0, l);
    let r = eng.run(&flight(), 1.0);
    let (s3, s4, s9) = (r.station("3"), r.station("4"), r.station("9"));
    Dp {
        far: s4.far, tt3: s3.tt, tt4: s4.tt, pt4: s4.pt,
        tt9: s9.tt, pt9: s9.pt, p9: r.p9, v9: r.v9, t9: r.t9,
        gas: eng.gas,
    }
}

fn need(comp: &[(&str, f64)], name: &str) -> f64 {
    comp.iter().find(|&&(s, _)| s == name).expect("species present").1
}

// ------------------------------------------------------------------------------------- //
// GATE 1 — the LOAD-BEARING reduce: the frozen expansion IS the production nozzle.        //
// ------------------------------------------------------------------------------------- //

/// The frozen branch of `expand_nozzle` is the production nozzle re-derived on the molar
/// entropy/enthalpy scale, so it must reproduce the engine's V9 and T9.
///
/// **The bar is MEASURED, not transcribed.** `test_rung14.py` asserts `< 1e-6` ABSOLUTE; the
/// actual worst over eight design points is 2.46e-11 m/s in V9 and 2.80e-11 K in T9 — 1.75e-14
/// and 2.36e-14 RELATIVE. A bar six orders above the thing it measures cannot tell a defect from
/// noise, which is the lesson phase 2 paid a whole phase for. 1e-12 relative leaves ~40×.
#[test]
fn frozen_expansion_reproduces_production_nozzle() {
    for tt4 in [1500.0, 1800.0, 2200.0] {
        let d = dp(tt4, true);
        let nf = d.gas.nozzle_flow(d.far, d.tt4, d.pt4, d.tt9, d.pt9, d.p9, None);
        let (rv, rt) =
            ((nf.v9_frozen - d.v9).abs() / d.v9, (nf.t9_frozen - d.t9).abs() / d.t9);
        assert!(rv < 1e-12, "Tt4={tt4}: frozen V9 {} vs production {} (rel {rv:.2e})",
                nf.v9_frozen, d.v9);
        assert!(rt < 1e-12, "Tt4={tt4}: frozen T9 {} vs production {} (rel {rt:.2e})",
                nf.t9_frozen, d.t9);
    }
}

/// The expansion body genuinely CONSULTS its composition function — the half of the Python's
/// monkey-patch gate that survives the factorisation.
///
/// Arm 1 (the setup, and a tautology in Rust — see the module header): a constant closure at
/// `comp_entry` reproduces the frozen branch bit-for-bit, because that IS the frozen branch.
/// Arm 2 (the one that can fail): the SAME call fed a different constant pool — the shifted exit
/// mixture, a real composition rather than an invented perturbation — must MOVE both T9 and V9.
/// A body that ignored `comp_at` and read `comp_entry` throughout would pass arm 1, fail arm 2,
/// and would silently collapse the entire rung-14 bracket to a single number.
#[test]
fn the_expansion_body_consults_its_composition_function() {
    let d = dp(2200.0, true);
    let comp = equilibrium_composition(d.far, d.tt4, d.pt4);
    let frozen = expand_nozzle(&comp, d.far, d.tt9, d.pt9, d.p9, false);
    let shifted = expand_nozzle(&comp, d.far, d.tt9, d.pt9, d.p9, true);

    let entry = comp.clone();
    let via_closure =
        expand_nozzle_with(&comp, &|_t| entry.clone(), d.tt9, d.pt9, d.p9, TOL_REL);
    assert_eq!(via_closure.t9.to_bits(), frozen.t9.to_bits(), "constant-at-entry T9 must be frozen's");
    assert_eq!(via_closure.v9.to_bits(), frozen.v9.to_bits(), "constant-at-entry V9 must be frozen's");

    let other = shifted.comp9.clone();
    let via_other =
        expand_nozzle_with(&comp, &|_t| other.clone(), d.tt9, d.pt9, d.p9, TOL_REL);
    assert!(via_other.t9 != frozen.t9, "a DIFFERENT constant pool left T9 unmoved — comp_at unused");
    assert!(via_other.v9 != frozen.v9, "a DIFFERENT constant pool left V9 unmoved — comp_at unused");
}

// ------------------------------------------------------------------------------------- //
// GATE 2 — reduce: dissociation → 0 ⇒ the bracket collapses (dV9 → 0).                    //
// ------------------------------------------------------------------------------------- //

/// At a cool combustor the station-4 pool is essentially complete-combustion (CO/(CO+CO2) → 0),
/// so the shifting expansion has nothing to recombine and `dV9 → 0`.
#[test]
fn cool_combustor_collapses_the_bracket() {
    let d = dp(1300.0, true);
    let nf = d.gas.nozzle_flow(d.far, d.tt4, d.pt4, d.tt9, d.pt9, d.p9, None);
    assert!(nf.co_fraction_entry < 1e-5, "cool combustor still dissociated: {}", nf.co_fraction_entry);
    assert!(nf.dv9_frac() < 1e-4, "cool-combustor bracket did not collapse: {}", nf.dv9_frac());
}

// ------------------------------------------------------------------------------------- //
// GATE 3 — direction: shifting is faster, hotter at exit, and recombines.                 //
// ------------------------------------------------------------------------------------- //

/// A shifting expansion is FASTER, HOTTER at the exit (recombination reheats), and genuinely
/// recombines.
///
/// Includes the DESIGN point (1500 K), where the shift is ~5e-6 scale — near the equilibrium
/// solver's underflow floor and so the case most likely to SILENTLY return the entry pool and
/// zero the bracket for the wrong reason.
///
/// **The Python's `comp_exit_eq is not comp_entry` is NOT transcribed** (vacuity case #5): in
/// Rust the exit pool is a fresh `Vec` by construction, so an identity check is green whatever
/// happened. What replaces it is the physics that check was proxying — the exit pool differs
/// from the entry pool AS VALUES, with CO down and CO2 up.
#[test]
fn equilibrium_is_faster_hotter_and_recombines() {
    for tt4 in [1500.0, 1800.0, 2200.0] {
        let d = dp(tt4, true);
        let nf = d.gas.nozzle_flow(d.far, d.tt4, d.pt4, d.tt9, d.pt9, d.p9, None);
        assert!(nf.v9_equilibrium > nf.v9_frozen, "Tt4={tt4}: recombination must add KE");
        assert!(nf.t9_equilibrium > nf.t9_frozen, "Tt4={tt4}: recombination must reheat the exit");
        let (co_in, co_out) = (need(&nf.comp_entry, "CO"), need(&nf.comp_exit_eq, "CO"));
        let (c2_in, c2_out) = (need(&nf.comp_entry, "CO2"), need(&nf.comp_exit_eq, "CO2"));
        assert!(co_out < co_in, "Tt4={tt4}: the equilibrium exit must recombine CO ({co_out} vs {co_in})");
        assert!(c2_out > c2_in, "Tt4={tt4}: the CO must have gone somewhere — CO2 did not rise");
        assert!(
            nf.comp_exit_eq != nf.comp_entry,
            "Tt4={tt4}: the exit pool is bit-identical to the entry pool — a silent solver return"
        );
    }
}

// ------------------------------------------------------------------------------------- //
// GATE 4 — magnitude: dormant at the design point, earns its keep hot.                    //
// ------------------------------------------------------------------------------------- //

/// The recombination benefit is NEGLIGIBLE at the cool lean design point and grows monotonically
/// with combustor temperature — the "dormant here, earns its keep hot" arc.
///
/// **Eleven points, where the Python has three.** A monotonicity claim on three samples is the
/// shape slice B narrowed rung 12 on; this one survives the finer grid (9.86e-6 at 1300 K to
/// 7.90e-3 at 2300 K, strictly increasing).
///
/// BOTH bounds on the hot anchor, not just `>`: the frozen reduce validates the sensible
/// machinery, but formation enthalpy CANCELS in the frozen path, so it cannot catch a
/// recombination-ENERGY error — which is exactly what sets ΔV9. The band is the <1 %
/// air-breathing trend (Hill & Peterson / Sutton), the honest live-assertion substitute for a
/// bespoke published digit the project does not have.
#[test]
fn bracket_grows_with_combustor_temperature() {
    let grid: Vec<f64> =
        (0..11).map(|i| 1300.0 + 100.0 * i as f64).collect();
    let fracs: Vec<f64> = grid
        .iter()
        .map(|&tt4| {
            let d = dp(tt4, false);
            d.gas.nozzle_flow(d.far, d.tt4, d.pt4, d.tt9, d.pt9, d.p9, None).dv9_frac()
        })
        .collect();
    assert!(fracs[2] < 1e-4, "design-point bracket not dormant: {}", fracs[2]);
    let hot = fracs[9]; // Tt4 = 2200 K, the anchor the band is quoted at
    assert!(3e-3 < hot && hot < 8e-3, "hot-anchor bracket outside the <1% band: {hot}");
    for w in fracs.windows(2) {
        assert!(w[1] > w[0], "dV9 fraction not monotone in Tt4: {fracs:?}");
    }
}

// ------------------------------------------------------------------------------------- //
// GATE 5 — isentropic self-check: both expansions conserve mixture entropy.               //
// ------------------------------------------------------------------------------------- //

/// Reversible + adiabatic ⇒ the mixture entropy at the exit equals the entry entropy for BOTH
/// the frozen and the shifting expansion — the constraint each solve is built on.
#[test]
fn expansions_conserve_entropy() {
    let d = dp(2200.0, true);
    let nf = d.gas.nozzle_flow(d.far, d.tt4, d.pt4, d.tt9, d.pt9, d.p9, None);
    let s_entry = mix_entropy_molar(&nf.comp_entry, d.tt9, d.pt9);
    let s_froz = mix_entropy_molar(&nf.comp_entry, nf.t9_frozen, d.p9);
    let s_eq = mix_entropy_molar(&nf.comp_exit_eq, nf.t9_equilibrium, d.p9);
    assert!((s_froz - s_entry).abs() < 1e-6 * s_entry.abs(), "frozen expansion not isentropic");
    assert!((s_eq - s_entry).abs() < 1e-6 * s_entry.abs(), "shifting expansion not isentropic");
}

// ------------------------------------------------------------------------------------- //
// GATE 6 — the dropped clamp earns its keep on the cooling path.                          //
// ------------------------------------------------------------------------------------- //

/// The equilibrium NO mole fraction collapses from the nozzle entry to the exit (`Kp_NO` falls
/// steeply with T) — the frozen-NO-INDEPENDENT core of the clamp claim.
#[test]
fn equilibrium_no_collapse_is_frozen_independent() {
    for tt4 in [1500.0, 1800.0, 2200.0] {
        let d = dp(tt4, true);
        let nf = d.gas.nozzle_flow(d.far, d.tt4, d.pt4, d.tt9, d.pt9, d.p9, None);
        assert!(nf.x_no_e_exit < nf.x_no_e_entry, "Tt4={tt4}: exit eq NO must be below entry");
        assert!(nf.no_collapse_ratio > 10.0, "Tt4={tt4}: eq-NO collapse too weak: {}",
                nf.no_collapse_ratio);
    }
}

/// Fed the physically-realistic rung-8 zoned (ICAO-band) exhaust NO, the frozen exhaust NO is
/// wildly super-equilibrium at the exit — the dropped rung-7 clamp FIRES (`max_a ≫ 1`), unlike
/// rung 10's DORMANT combustor quench (`max_a` = 0.677 < 1). This is "where the dropped clamp
/// earns its keep".
#[test]
fn clamp_fires_with_realistic_zoned_no() {
    const RUNG10_DORMANT: f64 = 0.677;
    for tt4 in [1500.0, 1800.0] {
        let d = dp(tt4, true);
        let zn = d.gas.zoned_nox(
            d.far, d.tt3, d.tt4, d.pt4, 1.0,
            ZonedNoxOpts { tau: 3e-3, ..Default::default() },
        );
        let nf =
            d.gas.nozzle_flow(d.far, d.tt4, d.pt4, d.tt9, d.pt9, d.p9, Some(zn.x_no_mix));
        assert!(nf.clamp_fires(), "Tt4={tt4}: clamp should fire (max_a={:?})", nf.max_a);
        assert!(
            nf.max_a.unwrap() > 10.0 * RUNG10_DORMANT,
            "Tt4={tt4}: nozzle max_a={:?} not decisively past rung 10's dormant {RUNG10_DORMANT}",
            nf.max_a
        );
    }
}

/// With no frozen exhaust NO supplied, `max_a` is `None` and the clamp reports dormant — the
/// collapse ratio still stands as the frozen-NO-independent statement.
#[test]
fn clamp_dormant_without_frozen_no() {
    let d = dp(1800.0, true);
    let nf = d.gas.nozzle_flow(d.far, d.tt4, d.pt4, d.tt9, d.pt9, d.p9, None);
    assert!(nf.max_a.is_none() && !nf.clamp_fires());
    assert!(nf.no_collapse_ratio > 1.0);
}

// ------------------------------------------------------------------------------------- //
// GATE 7 — the diagnostic never feeds the cycle.                                          //
// ------------------------------------------------------------------------------------- //

/// A `nozzle_flow` call must not perturb station 4 — it is a pure diagnostic, so the cycle stays
/// bit-for-bit rung 6.
#[test]
fn cycle_untouched_by_nozzle_flow_call() {
    let run = || {
        build_turbojet(Gas::reacting_equilibrium(), PI_C, 1500.0, 50_000.0, losses())
            .run(&flight(), 1.0)
    };
    let r1 = run();
    let (s4, s9) = (r1.station("4"), r1.station("9"));
    let g = Gas::reacting_equilibrium();
    g.nozzle_flow(s4.far, s4.tt, s4.pt, s9.tt, s9.pt, r1.p9, Some(5e-4));
    let r2 = run();
    assert_eq!(
        r2.station("4").far.to_bits(),
        s4.far.to_bits(),
        "nozzle_flow perturbed the cycle far — must stay rung-6"
    );
    assert_eq!(r2.v9.to_bits(), r1.v9.to_bits(), "nozzle_flow perturbed the cycle V9");
}

// ------------------------------------------------------------------------------------- //
// GATE 8 — guards.                                                                        //
// ------------------------------------------------------------------------------------- //

#[test]
#[should_panic(expected = "needs the rung-6 equilibrium gas")]
fn requires_the_equilibrium_gas() {
    // Fork B has no dissociation machinery, so there is no station-4 mixture to freeze.
    Gas::reacting_forkb().nozzle_flow(0.03, 1500.0, 1.3e6, 1300.0, 4e5, 5e4, None);
}

#[test]
#[should_panic(expected = "exceeds pt9")]
fn rejects_a_back_pressure_above_the_total() {
    let d = dp(1500.0, true);
    d.gas.nozzle_flow(d.far, d.tt4, d.pt4, d.tt9, d.pt9, d.pt9 * 1.5, None);
}

/// The 500 K exit-bracket guard on the SHIPPED entry point, not just on the raw expansion.
///
/// `nozzle_oracle.rs` gates the census over a back-pressure ladder; this pins that the guard is
/// wired into `nozzle_flow` itself, at a back-pressure measured to be past the edge (`p9/pt9` =
/// 0.01 against a firing threshold of 0.025016 at this design point).
#[test]
#[should_panic(expected = "pinned at the 500 K bracket floor")]
fn rejects_an_expansion_that_pins_at_the_exit_floor() {
    let d = dp(1500.0, true);
    d.gas.nozzle_flow(d.far, d.tt4, d.pt4, d.tt9, d.pt9, d.pt9 * 0.01, None);
}
