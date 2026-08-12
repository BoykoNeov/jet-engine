//! PHASE 1 SPINE — the reduce-to-prior contract on the Rust gas layer.
//!
//! `gas_oracle.rs` proves the Rust reproduces Python's VALUES at the probe points chosen.
//! That is the easier half. This file proves the project's actual spine: **each rung
//! collapses onto its predecessor when its effect is switched off.** A port can agree
//! numerically at every probe and still be structurally wrong — for instance by "helpfully"
//! unifying the calorically-perfect and thermally-perfect branches, which agree to ~0.05 %
//! and would sail through a 1e-3 comparison while destroying rung 1's exact reduction.
//!
//! Ported from `tests/test_rung6.py` GATE 3 / `test_pressure_suppresses_dissociation`, and
//! from the rung-3 module docstring's own § the trap.
//!
//! NOT here, and deliberately: rung 6's GATE 1 (the cold-`Tt4` cycle reduce, `fE == fB` to
//! 1e-6) needs `build_turbojet`, so it lands in phase 2 with the components. What IS here is
//! its gas-layer shadow — the composition itself reducing — which is portable today.

use turbojet::gas::*;

/// Relative gap, with an absolute fallback so an exact zero cannot divide.
fn rel(a: f64, b: f64) -> f64 {
    let scale = a.abs().max(b.abs());
    if scale > 0.0 { (a - b).abs() / scale } else { 0.0 }
}

// ======================================================================================
// RUNGS 1-2 — the calorically-perfect section keeps the CLOSED FORMS, exactly.
// ======================================================================================

/// `h = cp T` and `pr = T^(1/g)` must be the literal closed forms, to the last bit.
///
/// This is what makes reduce-to-ideal reproduce rung 1's published table to the digit. It
/// reads as near-tautological *today* — which is the point: it is a structural guard against
/// a later refactor routing the CPG branch through the integral path.
#[test]
fn cpg_keeps_the_closed_forms_bit_for_bit() {
    let s = CpgSection::new(1.4, 1004.0, 287.0);
    // `(1.4 - 1.0) / 1.4`, NOT `0.4 / 1.4`. Neither 1.4 nor 0.4 is exact in binary, and
    // `1.4 - 1.0` lands on 0.3999999999999999 — a different double from 0.4. Writing the
    // "obvious" form here failed on the first run, which is the same class of hazard as the
    // associativity rule in `gas.rs`'s header: transcribe the EXPRESSION, not its algebra.
    assert_eq!(s.g.to_bits(), ((1.4f64 - 1.0) / 1.4).to_bits(), "g = (gamma-1)/gamma");
    assert_ne!((1.4f64 - 1.0).to_bits(), 0.4f64.to_bits(), "…and this is why");
    for t in [200.0, 288.15, 500.0, 1000.0, 1500.0, 2200.0, 3000.0] {
        assert_eq!(s.h(t).to_bits(), (1004.0f64 * t).to_bits(), "h must be exactly cp*T at {t}");
        assert_eq!(s.pr(t).to_bits(), t.powf(1.0 / s.g).to_bits(), "pr must be exactly T^(1/g)");
        // Both inverses are closed-form too — no Newton, so they invert exactly.
        assert_eq!(s.t_from_h(s.h(t)).to_bits(), t.to_bits(), "T_from_h exact at {t}");
        assert!(rel(s.t_from_pr(s.pr(t)), t) < 1e-15, "T_from_pr round-trip at {t}");
        assert_eq!(s.cp(t).to_bits(), 1004.0f64.to_bits());
        assert_eq!(s.gamma_at(t).to_bits(), 1.4f64.to_bits());
    }
}

/// THE TRAP, measured rather than asserted (rung-3 spec § the trap).
///
/// Rungs 1-2 pin `gamma = 1.4` with a rounded `R = 287` that is ~0.05 % off
/// `R = (gamma-1)/gamma * cp`. The integral path's constant-cp limit is `pr = T^(R/cp)`,
/// exponent `287/1004 = 0.285857`; the closed form uses `(gamma-1)/gamma = 0.285714`. If the
/// two branches were merged, a "constant-cp" gas would silently take the wrong exponent.
///
/// This measures how far apart they actually are, so the branch's existence is justified by
/// a number in the test output rather than by a claim in a comment.
#[test]
fn the_two_branches_really_do_disagree_so_the_split_is_load_bearing() {
    let cpg = CpgSection::new(1.4, 1004.0, 287.0);
    // A thermally-perfect section with FLAT cp: A0 = cp/R, higher coefficients zero.
    let a0 = 1004.0 / 287.0;
    let flat = TpgSection::new([a0, 0.0, 0.0, 0.0, 0.0], [a0, 0.0, 0.0, 0.0, 0.0], 287.0);

    let exp_closed = 1.0 / cpg.g;         // gamma/(gamma-1) = 3.5
    let exp_integral = a0;                // cp/R           = 3.49826...
    let exponent_gap = rel(exp_closed, exp_integral);
    println!("exponent  closed {exp_closed:.9}  integral {exp_integral:.9}  gap {exponent_gap:.3e}");
    assert!(exponent_gap > 3e-4 && exponent_gap < 8e-4,
            "the rounded-R gap should be ~0.05%, measured {exponent_gap:.3e}");

    let mut worst_pr = 0.0f64;
    let mut worst_h = 0.0f64;
    for t in [300.0, 800.0, 1500.0, 2500.0] {
        worst_pr = worst_pr.max(rel(cpg.pr(t), flat.pr(t)));
        worst_h = worst_h.max(rel(cpg.h(t), flat.h(t)));
    }
    println!("pr disagreement {worst_pr:.3e}   h disagreement {worst_h:.3e}");
    // Enthalpy is safe (datum-0 makes a flat-cp integral reduce to cp*T), pressure is NOT.
    assert!(worst_h < 1e-15, "flat-cp h SHOULD reduce to cp*T, got {worst_h:.3e}");
    assert!(worst_pr > 1e-3,
            "if pr agreed, merging the branches would be harmless — it is not; got {worst_pr:.3e}");
}

// ======================================================================================
// RUNG 5 — Fork B's DERIVED heating value reduces to Fork A's ASSUMED one.
// ======================================================================================

/// The single calibration input is pinned so the derived LHV falls out at Mattingly's
/// assumed 42.8 MJ/kg — which is what makes Fork B reproduce rung-4 Fork A for complete
/// combustion. If this drifts, every downstream fuel flow drifts with it.
#[test]
fn forkb_derived_lhv_reduces_to_the_assumed_hpr() {
    let lhv = lhv_from_fuel(hf_fuel_default());
    let gap = rel(lhv, HPR_MATTINGLY);
    println!("derived LHV {lhv:.6e}  assumed {HPR_MATTINGLY:.6e}  gap {gap:.3e}");
    assert!(gap < 1e-15, "derived LHV must round-trip the assumed hPR, gap {gap:.3e}");
}

// ======================================================================================
// RUNG 6 — the equilibrium substrate's self-checks, and dissociation's two signatures.
// ======================================================================================

/// GATE 3 of `test_rung6.py`: a6 is DERIVED from ΔHf and a7 from S298, so evaluating the
/// absolute enthalpy and entropy back at 298.15 K must return the inputs. Exact by
/// construction — which means a failure here is a transcription slip, not a tolerance issue.
#[test]
fn formation_and_entropy_self_check() {
    for &(sp, hf) in HF298 {
        let h_abs = RU * (sens_h(sp, T_REF) + a6_of(sp));
        let s_abs = s_molar(sp, T_REF);
        assert!(rel(h_abs, hf) < 1e-9 || (h_abs - hf).abs() < 1e-6,
                "{sp}: h(298) = {h_abs} != {hf}");
        let s0 = s298(sp);
        assert!(rel(s_abs, s0) < 1e-9 || (s_abs - s0).abs() < 1e-6,
                "{sp}: s(298) = {s_abs} != {s0}");
    }
    // The dissociation species carry the expected formation signs; H2 is an element.
    assert!(hf298("CO") < 0.0 && hf298("H2") == 0.0);
    assert!(hf298("OH") > 0.0 && hf298("O") > 0.0 && hf298("H") > 0.0);
}

/// Dissociation must FALL as pressure rises — the `(p/p0)^dnu` factor in Kp. A solve that
/// dropped the pressure term entirely would still converge, still conserve atoms, and still
/// look plausible; only this shape test catches it.
#[test]
fn pressure_suppresses_dissociation() {
    const P1: f64 = 101325.0;
    let f = f_stoich() * 0.98; // lean of stoich, so `products_composition` stays in scope
    let mut fracs = Vec::new();
    for p_atm in [1.0, 5.0, 13.0] {
        let c = equilibrium_composition(f, 2300.0, p_atm * P1);
        let get = |n: &str| c.iter().find(|&&(s, _)| s == n).unwrap().1;
        fracs.push(get("CO") / (get("CO") + get("CO2")));
    }
    println!("CO/(CO+CO2) at 1/5/13 atm: {:.5} {:.5} {:.5}", fracs[0], fracs[1], fracs[2]);
    assert!(fracs[0] > fracs[1] && fracs[1] > fracs[2],
            "dissociation must fall with pressure: {fracs:?}");
    assert!(fracs[0] > 0.05, "1 atm near-stoich should show real CO dissociation: {}", fracs[0]);
}

/// THE GAS-LAYER SPINE: rung 6 reduces to rung 4 as the temperature falls.
///
/// This is the portable half of `test_rung6.py`'s GATE 1. As `T` drops, dissociation → 0 and
/// the equilibrium composition must approach the complete-combustion one.
///
/// The second assertion is the load-bearing one and is lifted straight from GATE 1's
/// reasoning: the gap must SHRINK with falling T. A CONSTANT offset would betray a scale-A
/// enthalpy datum leaking into a scale-B balance — a bug that a "gap is small" test passes
/// happily, because a constant 1 % is also small.
#[test]
fn equilibrium_reduces_to_complete_combustion_as_it_cools() {
    let f = 0.025;
    let p = 1.0e6;
    let complete = products_composition(f);
    let cg = |n: &str| complete.iter().find(|&&(s, _)| s == n).unwrap().1;

    let mut gaps = Vec::new();
    for t in [2400.0, 2000.0, 1600.0, 1200.0, 900.0] {
        let eq = equilibrium_composition(f, t, p);
        let eg = |n: &str| eq.iter().find(|&&(s, _)| s == n).unwrap().1;
        // The three species complete combustion actually predicts. N2/Ar are inert and
        // identical by construction, so including them would only dilute the measurement.
        let gap = rel(eg("CO2"), cg("CO2"))
            .max(rel(eg("H2O"), cg("H2O")))
            .max(rel(eg("O2"), cg("O2")));
        println!("T = {t:>6.0} K   worst composition gap {gap:.3e}   CO {:.3e}", eg("CO"));
        gaps.push(gap);
    }

    for w in gaps.windows(2) {
        assert!(w[1] < w[0],
                "the gap must SHRINK as it cools — a CONSTANT offset would betray a datum \
                 leak, and 'small' alone would not catch it: {gaps:?}");
    }
    assert!(*gaps.last().unwrap() < 1e-6,
            "at 900 K dissociation should be spent; worst gap {:.3e}", gaps.last().unwrap());
    assert!(gaps[0] > 1e-4,
            "at 2400 K there must be REAL dissociation, else the test proves nothing: {:.3e}",
            gaps[0]);
}
