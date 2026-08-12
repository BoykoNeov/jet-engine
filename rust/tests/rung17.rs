//! Rung 17 — the exhaust-NO clamp through the combustor-mixing-fidelity ladder (a rung-14
//! corollary).
//!
//! Carry the exhaust NO from three progressively-faithful combustor-mixing models through the
//! SAME rung-14 nozzle collapse to T9 and read the dropped-clamp margin `a = [NO]/[NO]_e(T9)`:
//!
//! * MIXED-OUT (rung 8) — the shortcut; at a RICH primary reads DORMANT (`a < 1`): mixing-out
//!   HIDES the NO.
//! * BULK QUENCH (rung 11) — the dilution re-making restored; FIRES (`a > 1`).
//! * PER-POCKET (rung 16) — the β-PDF segregation-raised mean; FIRES harder.
//!
//! THE CERTIFIED CONTENT (`docs/rung17-spec.md` § scope):
//!
//! 1. THE LADDER — the ORDERING `a_mixed ≤ a_bulk ≤ a_pocket` is STRUCTURAL and `a_mixed < 1` is
//!    robust; the IN-BAND firing holds at the RQL design point. Three INDEPENDENT physics
//!    composing.
//! 2. THE RUNG-14 CONTRAST — the SAME mixed-out-through-the-nozzle construction FIRES at
//!    φ_p = 1.0 and is DORMANT at φ_p = 1.5.
//! 3. SCALE-SENSITIVITY — the ORDERING survives every scale; the magnitudes and the gap MOVE.
//! 4. REDUCE-to-components, cycle-untouched, station-4 clamp dormancy, and the guards.
//!
//! **TWO GATES HERE SAY SOMETHING THE PYTHON'S CANNOT, and two of its own are dropped.**
//!
//! * [`the_firing_band_edge_is_located_and_moves_with_the_scale`] and
//!   [`the_ladder_does_not_go_dormant_with_the_bulk`] measure what the source's docstring only
//!   states — that the firing is in-band and not universal. The `a_bulk = 1` crossing sits at
//!   J ≈ 2460 at `C_e` = 0.20 and J ≈ 3990 at 0.15, ~11× past the shipped RQL band. And the
//!   ladder does NOT follow `a_bulk` down: `a_pocket` RISES over the same sweep, because rung
//!   16's `τ_core` is an ABSOLUTE residence its own docstring says survives J→∞. So the rung's
//!   headline predicate `hides_super_eq` is about `a_bulk` alone (plan § 4.9 probe 4).
//! * `test_identity_is_witnessed_not_a_test` is NOT transcribed (vacuity case #6). Its own file
//!   header calls it "witnessed, not gated … NOT a discriminating test", and it compares
//!   `a_pocket/a_bulk` to `gap_pocket_over_bulk` — both built from the same two EIs over the same
//!   `xe`. [`the_kappa_round_trip_and_the_independent_denominator`] replaces it with the two
//!   statements in that neighbourhood that CAN fail.
//! * `test_requires_both_configs` is NOT transcribed (vacuity case #7): `mixing` and
//!   `pocket_quench` are taken BY VALUE, so "needs both" is a compile error.
//!
//! **NO `at_most_one_closure` GATE HERE, and that is a statement rather than an omission.**
//! `exhaust_no_clamp` FIXES its own closure set — it builds the three `ZonedNoxOpts` itself — so
//! the mutual-exclusion guard is unreachable from this entry point. A test for it would be a bar
//! that cannot fail, which is the thing slice B refused to ship.
//!
//! Coarse grids — DIRECTION not digits (project ethos); the per-pocket MAGNITUDE is
//! grid-dependent.

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::Gas;
use turbojet::nox::{ExhaustClampOpts, ExhaustNoxClampState, JetMixing, PocketQuenchPdf, ZonedNoxOpts};

const TAU: f64 = 3e-3;
const PHI_P: f64 = 1.5; // the RQL rich primary — where the mixed-out shortcut HIDES the NO
const J: f64 = 225.0; // over-penetrating jet (the far-flank pockets are richest/hottest; rung 16)
// Coarse grids — per-pocket quench is the cost driver; DIRECTION not digits.
const NB: usize = 20; // per-pocket ξ-grid points
const NQ: usize = 64; // β-PDF quadrature nodes (≥ ~48 for mean-preservation)
const NG: usize = 24; // finite-quench trajectory points
const NSTEPS: usize = 200; // RK4 steps

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn losses() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
        ..Losses::default()
    }
}

fn mix(j: f64, c_e: f64) -> JetMixing {
    JetMixing { j, c_e, shape_n: 2.0, ..Default::default() }
}

fn pq() -> PocketQuenchPdf {
    PocketQuenchPdf { n_bell: NB, n_quad: NQ, ..Default::default() }
}

fn opts() -> ExhaustClampOpts {
    ExhaustClampOpts { tau: TAU, quench_ngrid: NG, quench_nsteps: NSTEPS, ..Default::default() }
}

/// The rung-17 design point — the stations the ladder rides on. NO is trace ⇒ the cycle is
/// bit-for-bit rung 6.
struct Dp {
    gas: Gas,
    far: f64,
    tt3: f64,
    tt4: f64,
    p: f64,
    tt9: f64,
    pt9: f64,
    p9: f64,
}

fn dp() -> Dp {
    let eng = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, 50_000.0, losses());
    let r = eng.run(&flight(), 50.0);
    let (s3, s4, s9) = (r.station("3"), r.station("4"), r.station("9"));
    Dp {
        far: s4.far, tt3: s3.tt, tt4: s4.tt, p: s4.pt,
        tt9: s9.tt, pt9: s9.pt, p9: r.p9,
        gas: eng.gas,
    }
}

fn clamp_at(d: &Dp, j: f64, c_e: f64) -> ExhaustNoxClampState {
    d.gas.exhaust_no_clamp(
        d.far, d.tt3, d.tt4, d.p, d.tt9, d.pt9, d.p9, PHI_P, mix(j, c_e), pq(), opts(),
    )
}

/// `a_bulk` on the CHEAP path — the bulk numerator against a denominator computed once.
///
/// The sizing lever `Gas::nozzle_flow`'s argument list makes available: it reads no mixing
/// config, so `x_no_e(T9)` is one solve for an entire J sweep.
fn a_bulk_at(d: &Dp, xe: f64, j: f64, c_e: f64) -> f64 {
    let zb = d.gas.zoned_nox(
        d.far, d.tt3, d.tt4, d.p, PHI_P,
        ZonedNoxOpts {
            tau: TAU,
            mixing: Some(mix(j, c_e)),
            quench_ngrid: NG,
            quench_nsteps: NSTEPS,
            ..Default::default()
        },
    );
    zb.x_no_quenched.expect("bulk quench ran") / xe
}

// ------------------------------------------------------------------------------------- //
// GATE 1 — THE LADDER. The ORDERING is STRUCTURAL (the quench only ADDS NO; the           //
// per-pocket excess is additive) + `a_mixed < 1` robust; the IN-BAND firing holds at the   //
// design point. Three INDEPENDENT physics: rung-8 mixed-out, rung-11 quench re-making,     //
// rung-16 segregation-raises-the-mean.                                                     //
// ------------------------------------------------------------------------------------- //
#[test]
fn ladder_direction_the_load_bearing_gate() {
    let d = dp();
    let s = clamp_at(&d, J, 0.20);
    // STRUCTURAL ordering (holds at ANY scale — same common denominator, additive excess):
    assert!(
        s.x_no_bulk_quench >= s.x_no_mixed_out,
        "the clamp-free quench only ADDS NO (x_no_quenched ≥ x_no_mix) — structural"
    );
    assert!(
        s.x_no_pocket >= s.x_no_bulk_quench,
        "the per-pocket excess is ADDITIVE (x_no_pocket ≥ x_no_bulk) — structural"
    );
    assert!(
        s.a_mixed_out < s.a_bulk_quench && s.a_bulk_quench < s.a_pocket,
        "the ORDERING must be monotone in fidelity: {:.4}, {:.4}, {:.4}",
        s.a_mixed_out, s.a_bulk_quench, s.a_pocket
    );
    // `a_mixed < 1` robust (a rich primary makes ≈0 NO), plus the IN-BAND firing at this point:
    assert!(s.a_mixed_out < 1.0, "mixed-out must read DORMANT at the rich primary: {}", s.a_mixed_out);
    assert!(s.a_bulk_quench > 1.0, "bulk quench must FIRE in-band: {}", s.a_bulk_quench);
    assert!(s.a_pocket > 1.0, "per-pocket must FIRE in-band: {}", s.a_pocket);
    assert!(s.hides_super_eq() && s.ladder_monotone(), "both headline predicates must hold");
}

// ------------------------------------------------------------------------------------- //
// GATE 2 — THE RUNG-14 CONTRAST: the SAME mixed-out-through-the-nozzle construction FIRES  //
// at φ_p = 1.0 and is DORMANT at φ_p = 1.5. Two faces of the one dropped-clamp lesson.     //
// ------------------------------------------------------------------------------------- //
#[test]
fn rung14_contrast_mixed_out_fires_lean_dormant_rich() {
    let d = dp();
    let a_mixed = |phi: f64| {
        let zn = d.gas.zoned_nox(
            d.far, d.tt3, d.tt4, d.p, phi,
            ZonedNoxOpts { tau: TAU, ..Default::default() }, // rung-8 mixed-out (cheap)
        );
        d.gas
            .nozzle_flow(d.far, d.tt4, d.p, d.tt9, d.pt9, d.p9, Some(zn.x_no_mix))
            .max_a
            .expect("frozen NO supplied")
    };
    let (a10, a15) = (a_mixed(1.0), a_mixed(1.5));
    assert!(a10 > 1.0, "φ_p=1.0 mixed-out must FIRE (the rung-14 corollary): {a10:.2}");
    assert!(a15 < 1.0, "φ_p=1.5 mixed-out must be DORMANT (the rich primary hides it): {a15:.4}");
    assert!(a10 > 100.0 * a15, "the contrast must be stark — the shortcut is unconservative");
    // the dormant φ_p=1.5 value IS the ladder's bottom rung (reduce-to-components, mixed-out leg)
    let s = clamp_at(&d, J, 0.20);
    assert!(
        (a15 - s.a_mixed_out).abs() < 1e-9,
        "the φ_p=1.5 contrast must equal the ladder's a_mixed_out"
    );
}

// ------------------------------------------------------------------------------------- //
// GATE 3 — what survives of the Python's witnessed identity (vacuity case #6).             //
// ------------------------------------------------------------------------------------- //

/// The two statements near the Python's tautology that CAN fail.
///
/// `test_identity_is_witnessed_not_a_test` compares `a_pocket/a_bulk` to `gap_pocket_over_bulk`,
/// which are the same two EIs over the same `xe` — it cannot fail in either language. What can:
///
/// 1. **The κ ROUND-TRIP.** `x_no_pocket` is built as `κ·⟨EI⟩_pocket` with `κ = x_no_bulk/EI_bulk`.
///    Feeding the bulk EI back through that same κ must return the bulk mole fraction — which
///    fails if κ is ever formed from a mismatched pair (a different super-eq-O arm, a different
///    grid), the one way this construction can actually break.
/// 2. **AN INDEPENDENT DENOMINATOR.** `a_pocket` must equal `x_no_pocket / x_no_e(T9)` with the
///    `x_no_e(T9)` taken from a SEPARATE `nozzle_flow` call rather than read off the same state.
///    That is a real check that the ladder's three margins share the nozzle's denominator instead
///    of each carrying their own.
#[test]
fn the_kappa_round_trip_and_the_independent_denominator() {
    let d = dp();
    let s = clamp_at(&d, J, 0.20);
    let kappa = s.x_no_pocket / s.ei_no_pocket_quench;
    let bulk_again = kappa * s.ei_no_quenched;
    assert!(
        (bulk_again - s.x_no_bulk_quench).abs() <= 1e-15 * s.x_no_bulk_quench,
        "κ does not round-trip the bulk: {bulk_again:.6e} vs {:.6e}",
        s.x_no_bulk_quench
    );
    // an INDEPENDENT nozzle solve — same inputs, its own bisection
    let nf = d.gas.nozzle_flow(d.far, d.tt4, d.p, d.tt9, d.pt9, d.p9, None);
    assert_eq!(
        nf.x_no_e_exit.to_bits(),
        s.x_no_e_exit.to_bits(),
        "the ladder's denominator is not the rung-14 nozzle's"
    );
    for (name, a, x) in [
        ("mixed", s.a_mixed_out, s.x_no_mixed_out),
        ("bulk", s.a_bulk_quench, s.x_no_bulk_quench),
        ("pocket", s.a_pocket, s.x_no_pocket),
    ] {
        assert!(
            (a - x / nf.x_no_e_exit).abs() <= 1e-12 * a,
            "a_{name} is not x_no/x_no_e(T9) against an independent denominator"
        );
    }
}

// ------------------------------------------------------------------------------------- //
// GATE 4 — SCALE-SENSITIVITY: the ORDERING holds structurally at every scale, but the      //
// MAGNITUDES and the GAP move with `C_e`. The firing is IN-BAND, not universal.            //
// ------------------------------------------------------------------------------------- //
#[test]
fn scale_sensitivity_ordering_robust_magnitude_not() {
    let d = dp();
    let (lo, hi) = (clamp_at(&d, J, 0.15), clamp_at(&d, J, 0.20));
    for s in [&lo, &hi] {
        assert!(
            s.a_mixed_out < 1.0 && 1.0 < s.a_bulk_quench && s.a_bulk_quench < s.a_pocket,
            "the ladder ordering must survive every scale: {:.4},{:.4},{:.4}",
            s.a_mixed_out, s.a_bulk_quench, s.a_pocket
        );
    }
    assert!(
        (hi.a_bulk_quench - lo.a_bulk_quench).abs() > 0.05 * lo.a_bulk_quench,
        "a_bulk must move with C_e (un-pinned): {:.3} vs {:.3}",
        lo.a_bulk_quench, hi.a_bulk_quench
    );
    assert!(
        (hi.gap_pocket_over_bulk - lo.gap_pocket_over_bulk).abs() > 0.05 * lo.gap_pocket_over_bulk,
        "the gap is NOT scale-invariant (rung-16's gap rides on C_e): {:.3} vs {:.3}",
        lo.gap_pocket_over_bulk, hi.gap_pocket_over_bulk
    );
    // `a_mixed_out` has no jet dependence — identical at both scales, and BIT-identical here
    assert_eq!(
        hi.a_mixed_out.to_bits(),
        lo.a_mixed_out.to_bits(),
        "a_mixed_out (no jet) must be scale-independent"
    );
}

// ------------------------------------------------------------------------------------- //
// GATE 5 — THE FIRING BAND EDGE, and where the ladder stops following it. Rust's own.      //
// ------------------------------------------------------------------------------------- //

/// The `a_bulk = 1` crossing is LOCATED, and it MOVES with an un-pinned entrainment scale.
///
/// The source's docstring says the firing "holds across the RQL J-band but is NOT universal —
/// as the quench gets FAST (J→∞) `x_no_quenched → x_no_mix` … so `a_bulk → a_mixed < 1`". The
/// Python's suite tests one J. Measured here: `a_bulk` falls monotonically and crosses 1 between
/// J = 2000 and 4000 at `C_e` = 0.20 (bracketed at ~2460) and only past 4000 at `C_e` = 0.15
/// (~3990) — the edge moves 1.6× on a scale nothing pins.
///
/// The bracket is deliberately COARSE. The crossing is a smooth root, so resolving it finely
/// would ship a digit that rides on `C_e`, `τ_res`, `H` — exactly what the rung says is
/// un-pinned. What is certifiable is that it EXISTS, is far outside the band, and MOVES.
#[test]
fn the_firing_band_edge_is_located_and_moves_with_the_scale() {
    let d = dp();
    let xe = d.gas.nozzle_flow(d.far, d.tt4, d.p, d.tt9, d.pt9, d.p9, None).x_no_e_exit;
    let ladder = [225.0f64, 1000.0, 2000.0, 4000.0];
    let a20: Vec<f64> = ladder.iter().map(|&j| a_bulk_at(&d, xe, j, 0.20)).collect();
    for w in a20.windows(2) {
        assert!(w[1] < w[0], "a_bulk must fall monotonically in J: {a20:?}");
    }
    assert!(a20[0] > 1.0, "the RQL band point must FIRE: {}", a20[0]);
    assert!(a20[2] > 1.0 && a20[3] < 1.0, "the crossing must be bracketed by J ∈ (2000, 4000]: {a20:?}");
    // the edge MOVES: a weaker entrainment scale keeps the bulk firing further out
    let a15_at_4000 = a_bulk_at(&d, xe, 4000.0, 0.15);
    assert!(
        a15_at_4000 > a20[3],
        "the crossing did not move with C_e: {a15_at_4000:.4} vs {:.4}",
        a20[3]
    );
    assert!(a15_at_4000 > 1.0, "at C_e=0.15 the bulk must still fire at J=4000: {a15_at_4000:.4}");
}

/// **THE LADDER DOES NOT GO DORMANT WITH THE BULK** — the half the source's caveat leaves out.
///
/// Past the `a_bulk = 1` crossing the rung's headline predicate `hides_super_eq` goes FALSE,
/// because it is DEFINED on `a_bulk`. But `a_pocket` RISES over the same sweep and the ORDERING
/// survives. The mechanism is rung 16's own: `ei_no_pocket_quench` = the mean-field bulk (term 1,
/// riding `τ_mean ∝ 1/√J`, collapsing) + a β-PDF integral at `τ_core = τ_res(1+b_u·u)` (term 2),
/// which `PocketQuenchPdf::core_dwell`'s docstring calls an ABSOLUTE residence whose penalty
/// "survives J→∞" — and `u` GROWS off-optimum. Both terms are checked separately, so the gate
/// says WHY and not only THAT.
#[test]
fn the_ladder_does_not_go_dormant_with_the_bulk() {
    let d = dp();
    let inband = clamp_at(&d, 225.0, 0.20);
    let past = clamp_at(&d, 4000.0, 0.20);
    let deep = clamp_at(&d, 16000.0, 0.20);

    // the bulk goes dormant, and the headline predicate goes with it
    assert!(inband.a_bulk_quench > 1.0 && past.a_bulk_quench < 1.0, "the crossing did not happen");
    assert!(inband.hides_super_eq(), "in band, mixing-out must hide the NO");
    assert!(!past.hides_super_eq(), "past the crossing hides_super_eq is defined on a DORMANT a_bulk");

    // the pocket does NOT — it rises, and the ordering survives everywhere
    for s in [&inband, &past, &deep] {
        assert!(s.ladder_monotone(), "the fidelity ordering broke at a_bulk={}", s.a_bulk_quench);
        assert!(s.a_pocket > 1.0, "the per-pocket margin must keep firing: {}", s.a_pocket);
    }
    assert!(
        deep.a_pocket > past.a_pocket && past.a_pocket > inband.a_pocket,
        "a_pocket must RISE as a_bulk falls: {:.3} → {:.3} → {:.3}",
        inband.a_pocket, past.a_pocket, deep.a_pocket
    );

    // the MECHANISM, term by term: term 1 collapses, term 2 grows
    let term2 = |s: &ExhaustNoxClampState| s.ei_no_pocket_quench - s.ei_no_quenched;
    assert!(
        deep.ei_no_quenched < past.ei_no_quenched && past.ei_no_quenched < inband.ei_no_quenched,
        "term 1 (the mean-field bulk) must collapse with J"
    );
    assert!(
        term2(&deep) > term2(&past) && term2(&past) > term2(&inband),
        "term 2 (the τ_core-dwelling pockets) must GROW with J: {:.4} → {:.4} → {:.4}",
        term2(&inband), term2(&past), term2(&deep)
    );
}

// ------------------------------------------------------------------------------------- //
// GATE 6 — REDUCE-to-components (exact): `exhaust_no_clamp` COMPOSES the rung-8/11/16 +    //
// rung-14 outputs bit-for-bit; it never recomputes.                                        //
// ------------------------------------------------------------------------------------- //
#[test]
fn reduce_to_components_exact() {
    let d = dp();
    let s = clamp_at(&d, J, 0.20);
    let base = ZonedNoxOpts {
        tau: TAU,
        mixing: Some(mix(J, 0.20)),
        quench_ngrid: NG,
        quench_nsteps: NSTEPS,
        ..Default::default()
    };
    let zn_bulk = d.gas.zoned_nox(d.far, d.tt3, d.tt4, d.p, PHI_P, base);
    assert_eq!(
        s.x_no_bulk_quench.to_bits(),
        zn_bulk.x_no_quenched.unwrap().to_bits(),
        "x_no_bulk must BE the rung-11 x_no_quenched"
    );
    assert_eq!(
        s.ei_no_quenched.to_bits(),
        zn_bulk.ei_no_quenched.unwrap().to_bits(),
        "ei_no_quenched must BE the rung-11 value"
    );
    let nf = d.gas.nozzle_flow(
        d.far, d.tt4, d.p, d.tt9, d.pt9, d.p9, Some(s.x_no_bulk_quench),
    );
    assert_eq!(
        s.a_bulk_quench.to_bits(),
        nf.max_a.unwrap().to_bits(),
        "a_bulk must BE nozzle_flow(x_no_bulk).max_a"
    );
    assert_eq!(
        s.no_collapse_ratio.to_bits(),
        nf.no_collapse_ratio.to_bits(),
        "the collapse ratio must BE rung-14's"
    );
    let zn_pkt = d.gas.zoned_nox(
        d.far, d.tt3, d.tt4, d.p, PHI_P,
        ZonedNoxOpts { pocket_quench: Some(pq()), ..base },
    );
    assert_eq!(
        s.ei_no_pocket_quench.to_bits(),
        zn_pkt.ei_no_pocket_quench.unwrap().to_bits(),
        "ei_no_pocket_quench must BE the rung-16 value"
    );
}

// ------------------------------------------------------------------------------------- //
// GATE 7 — CYCLE UNTOUCHED + station-4 clamp dormancy.                                     //
// ------------------------------------------------------------------------------------- //

/// An `exhaust_no_clamp` call leaves the cycle `far` bit-identical — a pure diagnostic, rung 6.
#[test]
fn cycle_untouched_by_clamp_call() {
    let run = || {
        build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, 50_000.0, losses())
            .run(&flight(), 50.0)
    };
    let r1 = run();
    let (s3, s4, s9) = (r1.station("3"), r1.station("4"), r1.station("9"));
    let g = Gas::reacting_equilibrium();
    g.exhaust_no_clamp(
        s4.far, s3.tt, s4.tt, s4.pt, s9.tt, s9.pt, r1.p9, PHI_P, mix(J, 0.20), pq(), opts(),
    );
    assert_eq!(
        run().station("4").far.to_bits(),
        s4.far.to_bits(),
        "exhaust_no_clamp perturbed the cycle far — must stay rung-6"
    );
}

/// The combustor NO is SUB-equilibrium: the super-equilibrium is a NOZZLE phenomenon (the
/// collapse), not a burner one.
#[test]
fn clamp_dormant_at_station4() {
    let d = dp();
    let s = clamp_at(&d, J, 0.20);
    assert!(
        s.max_a_quench < 1.0,
        "the station-4 clamp must be DORMANT (super-eq is a nozzle effect): {:.3}",
        s.max_a_quench
    );
    assert!(s.no_collapse_ratio > 1.0, "the nozzle equilibrium-NO collapse must be > 1 (cooling)");
}

// ------------------------------------------------------------------------------------- //
// GATE 8 — GUARDS. "Requires both configs" is a COMPILE error here (vacuity case #7), so   //
// what remains runtime is the equilibrium gas and the inherited back-pressure guard.        //
// ------------------------------------------------------------------------------------- //
#[test]
#[should_panic(expected = "needs the rung-6 equilibrium gas")]
fn requires_the_equilibrium_gas() {
    let d = dp();
    let g = Gas::thermally_perfect(); // NOT the equilibrium gas
    g.exhaust_no_clamp(
        d.far, d.tt3, d.tt4, d.p, d.tt9, d.pt9, d.p9, PHI_P, mix(J, 0.20), pq(), opts(),
    );
}

#[test]
#[should_panic(expected = "exceeds pt9")]
fn back_pressure_guard_inherited() {
    let d = dp();
    d.gas.exhaust_no_clamp(
        d.far, d.tt3, d.tt4, d.p, d.tt9, d.pt9,
        d.pt9 * 1.5, // p9 > pt9 — cannot expand to it
        PHI_P, mix(J, 0.20), pq(), opts(),
    );
}
