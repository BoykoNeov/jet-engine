//! Rung-26 verification: FREEZE-OUT — an anchored recombination clock that resolves WHERE
//! recombination quenches.
//!
//! Rung 25 resolved the finite-rate flow between rung-14's bounds with a single normalised `Da` —
//! a cartoon that slides the whole expansion uniformly and CANNOT show freeze-out. Rung 26
//! replaces it with a LOCAL `Da(T,p) = τ_res/τ_chem(T,p)` from an ANCHORED GRI-Mech 3.0
//! recombination clock (zero new constants), so the relaxation SHUTS OFF partway down the nozzle
//! — and the shut-off point MOVES with `Tt4`.
//!
//! THE HEADLINE IS A MOVING FREEZE POINT, NOT A NEW BOUND: the freeze-out flow lands inside
//! rung-25's `[V9_frozen, V9_irrev_fast]`. This file certifies the ROBUST structure and
//! DELIBERATELY does not assert the freeze LOCATION to any precision, nor the frozen-in
//! composition — both ride on the geometric knob `L` and the representative-reaction pick.
//!
//! **WHERE THIS SUITE SAYS MORE THAN THE PYTHON'S, AND WHERE IT SAYS LESS.**
//!
//! 1. **The load-bearing reduce runs 40 cells where the Python's runs 6.** This is the gate the
//!    oracle CANNOT provide: a Python↔Rust dump compares values, so a loop-shape error
//!    transcribed identically into both copies of the march would pass it and fail here. Plan
//!    § 4.11 probe 4 measured the claim on the Python at 40/40 bit-exact before this was written
//!    — the first "exactly"-class claim in this lineage to survive, after slices C, D and E each
//!    corrected one.
//! 2. **"Rung 25 untouched" is rewritten, because in Rust the Python's version cannot fail.**
//!    Python guards against a diagnostic that mutates shared state; `freeze_out_nozzle` takes
//!    `&self` and the type system already forbids that without interior mutability, so
//!    transcribing the assertion literally would be vacuity case #8 again (plan § 4.9). What
//!    ships instead is the arm that CAN fail: the two methods must agree BIT-FOR-BIT on the
//!    (F) and (I) references they each compute independently.
//! 3. **The freeze-motion gate runs seven combustor temperatures where the Python's runs five**,
//!    extended at both ends — 1300 K below where the relaxation ever switches on, 2300 K above
//!    the Python's hot end.
//!
//! Gates, priority order (`docs/rung26-spec.md`).

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{equilibrium_composition, powp, Gas, RU};
use turbojet::march::{
    finite_rate_expand, freeze_out_expand, tau_chem_recomb, FiniteRate, FreezeOut,
    FreezeOutNozzleState,
};

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

fn fz(d: &Dp, fo: FreezeOut) -> FreezeOutNozzleState {
    d.gas.freeze_out_nozzle(d.far, d.tt4, d.pt4, d.tt9, d.pt9, d.p9, fo)
}

// --- GATE 1: RUNG 25 UNTOUCHED — and rewritten so it can actually fail ----------------------- //

/// The two diagnostics sit BESIDE each other and must agree, bit-for-bit, on the references they
/// each compute independently.
///
/// The Python's version of this gate calls `finite_rate_nozzle`, then `freeze_out_nozzle`, then
/// `finite_rate_nozzle` again and asserts the rung-25 answer did not move. In Rust that cannot
/// fail: both methods take `&self` and neither has interior mutability, so the compiler already
/// guarantees it — transcribing it literally would assert a function equals itself. What is
/// gated instead is the claim that has content: `freeze_out_nozzle` reaches (F) and (I) by the
/// same route `finite_rate_nozzle` does, so their values must be identical to the last bit. A
/// freeze-out method that computed its own bracket — say by marching at a tiny `rate_scale`
/// instead of dispatching — would agree to several digits and fail here.
#[test]
fn the_two_diagnostics_share_their_references_bit_for_bit() {
    for tt4 in [1500.0, 2200.0] {
        let d = dp(tt4);
        let fr = d.gas.finite_rate_nozzle(
            d.far,
            d.tt4,
            d.pt4,
            d.tt9,
            d.pt9,
            d.p9,
            FiniteRate { da: 3.0, nstep: 400 },
        );
        let f = fz(&d, FreezeOut::default());
        assert_eq!(f.v9_frozen.to_bits(), fr.v9_frozen.to_bits());
        assert_eq!(f.t9_frozen.to_bits(), fr.t9_frozen.to_bits());
        assert_eq!(f.v9_irrev_fast.to_bits(), fr.v9_irrev_fast.to_bits());
        assert_eq!(f.t9_irrev_fast.to_bits(), fr.t9_irrev_fast.to_bits());
    }
}

// --- GATE 2: REDUCE — constant Da_local ⇒ rung 25 BIT-FOR-BIT (LOAD-BEARING, NOT THE ORACLE) -- //

/// **The load-bearing reduce, and the one gate in this slice the oracle cannot supply.**
///
/// Driven with a CONSTANT `da_local_fn` — the literal `Da`, not `τ_res/τ_chem` — the rung-26 march
/// must reproduce `finite_rate_expand(Da)` to the ULP. The `Da → Da_local(s)` promotion is the
/// only change between the two loops, and it collapses back exactly.
///
/// This is also the DRIFT TRIPWIRE for the deliberate duplication. `freeze_out_expand` is
/// `finite_rate_expand` copied line for line; the Python does the same and says why (it keeps
/// rung 25 literally untouched). The `march_oracle.rs` gate cannot see an error made identically
/// in both copies — it compares values, and two identically-wrong loops produce identically-wrong
/// values on both sides of the Python↔Rust comparison only if the Python is wrong too, which it
/// is not; what it cannot see is a Rust-side edit that "tidies" the two into agreement with each
/// other while drifting from the source's loop shape. This gate is what stands there.
///
/// 40 cells against the Python's 6, and `Da = 300` is well past its ladder.
#[test]
fn constant_da_local_is_rung25_bit_for_bit() {
    let mut cells = 0usize;
    for tt4 in [1300.0, 1500.0, 1800.0, 2200.0, 2300.0] {
        let d = dp(tt4);
        let ce = entry(&d);
        for da in [0.5, 2.0, 10.0, 300.0] {
            for nstep in [100usize, 400] {
                let a = finite_rate_expand(&ce, d.far, d.tt9, d.pt9, d.p9, da, nstep);
                let konst = move |_: &[(&'static str, f64)], _: f64, _: f64| da;
                let (b, _, _, _) =
                    freeze_out_expand(&ce, d.far, d.tt9, d.pt9, d.p9, &konst, nstep, None);
                assert_eq!(
                    b.t9.to_bits(),
                    a.t9.to_bits(),
                    "T9 not bit-for-bit at Tt4={tt4}, Da={da}, nstep={nstep}: {} vs {}",
                    b.t9,
                    a.t9
                );
                assert_eq!(b.v9.to_bits(), a.v9.to_bits(), "V9 not bit-for-bit at Tt4={tt4}, Da={da}");
                assert_eq!(b.ds.to_bits(), a.ds.to_bits(), "dS not bit-for-bit at Tt4={tt4}, Da={da}");
                assert_eq!(b.comp9.len(), a.comp9.len());
                for (&(sp1, n1), &(sp2, n2)) in b.comp9.iter().zip(a.comp9.iter()) {
                    assert_eq!(sp1, sp2, "species ORDER differs at Tt4={tt4}, Da={da}");
                    assert_eq!(
                        n1.to_bits(),
                        n2.to_bits(),
                        "comp[{sp1}] not bit-for-bit at Tt4={tt4}, Da={da}"
                    );
                }
                cells += 1;
            }
        }
    }
    assert_eq!(cells, 40, "the reduce swept {cells} cells, expected 40");
}

/// The observer is PURE: recording the trajectory must not move a single bit of the result.
///
/// Rung 28 (slice G) reads that trajectory, so if recording perturbed the march, rung 28 would be
/// reading a different flow from the one rung 26 reports. Not in the Python's suite — its
/// equivalent is asserted in rung 28's file, which is one slice away; the property belongs to the
/// function that has the parameter.
#[test]
fn the_record_observer_is_bit_for_bit_pure() {
    let d = dp(2200.0);
    let ce = entry(&d);
    let da_fn = |_: &[(&'static str, f64)], _: f64, _: f64| 3.0;
    let (a, sa, ea, xa) = freeze_out_expand(&ce, d.far, d.tt9, d.pt9, d.p9, &da_fn, 100, None);
    let mut rec = Vec::new();
    let (b, sb, eb, xb) =
        freeze_out_expand(&ce, d.far, d.tt9, d.pt9, d.p9, &da_fn, 100, Some(&mut rec));
    assert_eq!(a.t9.to_bits(), b.t9.to_bits());
    assert_eq!(a.v9.to_bits(), b.v9.to_bits());
    assert_eq!(a.ds.to_bits(), b.ds.to_bits());
    assert_eq!((sa.to_bits(), ea.to_bits(), xa.to_bits()), (sb.to_bits(), eb.to_bits(), xb.to_bits()));
    // …and the record is the whole march plus its exit: one station per step, then station 1.0.
    assert_eq!(rec.len(), 101);
    assert_eq!(rec[0].t.to_bits(), d.tt9.to_bits());
    assert_eq!(rec[100].s.to_bits(), 1.0f64.to_bits());
    assert_eq!(rec[100].t.to_bits(), b.t9.to_bits());
}

// --- GATE 3: LIMITS — rate_scale→0 gives (F); rate_scale→∞ gives (I) ------------------------- //

#[test]
fn rate_scale_limits() {
    let d = dp(2200.0);
    let frozen_limit = fz(&d, FreezeOut { rate_scale: 1e-5, ..Default::default() });
    let fast_limit = fz(&d, FreezeOut { rate_scale: 1e5, ..Default::default() });
    assert!(
        frozen_limit.bracket_filled() < 0.02,
        "rate_scale→0 not at (F): filled={}",
        frozen_limit.bracket_filled()
    );
    assert!((frozen_limit.v9_freeze - frozen_limit.v9_frozen).abs() < 1e-2);
    assert!(
        fast_limit.bracket_filled() > 0.95,
        "rate_scale→∞ not at (I): filled={}",
        fast_limit.bracket_filled()
    );
}

// --- GATE 4: THE FREEZE EXISTS — dormant lean, earns its keep hot (composition space) --------- //

#[test]
fn freeze_dormant_lean_earns_hot() {
    let cold = fz(&dp(1500.0), FreezeOut::default());
    let hot = fz(&dp(2200.0), FreezeOut::default());

    // Lean: never switches on.
    assert!(cold.frozen_from_entry() && cold.da_entry < 1.0);
    assert_eq!(cold.s_freeze, 0.0);
    assert!(cold.bracket_filled() < 0.15);

    // Hot: switches on, then crosses mid-expansion.
    assert!(!hot.frozen_from_entry() && hot.da_entry > 1.0);
    assert!(hot.s_freeze > 0.0 && hot.s_freeze < 1.0);
    assert!(hot.bracket_filled() > cold.bracket_filled());

    // The V9 ordering holds, on a tiny (sub-percent) margin.
    for st in [&cold, &hot] {
        assert!(st.v9_frozen <= st.v9_freeze + 1e-6);
        assert!(st.v9_freeze + 1e-6 <= st.v9_irrev_fast + 2e-6);
    }

    // COMPOSITION is the load-bearing observable, not V9: recombination burns CO down, more hot.
    assert!(hot.co_fraction_freeze_exit < hot.co_fraction_entry);
    let hot_burn = 1.0 - hot.co_fraction_freeze_exit / hot.co_fraction_entry;
    let cold_burn = 1.0 - cold.co_fraction_freeze_exit / cold.co_fraction_entry;
    assert!(hot_burn > cold_burn, "hot burn {hot_burn} not above cold {cold_burn}");
}

// --- GATE 5: THE FREEZE POINT MOVES with Tt4 (THE RUNG) -------------------------------------- //

/// `s_freeze` rises with `Tt4` on the REAL self-quenching integrator. The certified claim is the
/// MONOTONE MOTION, not the `s` values — those ride on the geometric knob `L`.
///
/// Seven temperatures against the Python's five, extended at both ends: 1300 K is below where the
/// relaxation ever switches on (so it must still read 0) and 2300 K is above its hot end.
#[test]
fn freeze_point_moves_with_tt4() {
    let s: Vec<f64> = [1300.0, 1500.0, 1650.0, 1800.0, 2000.0, 2200.0, 2300.0]
        .iter()
        .map(|&tt4| fz(&dp(tt4), FreezeOut::default()).s_freeze)
        .collect();
    assert!(
        s.windows(2).all(|w| w[0] <= w[1] + 1e-12),
        "s_freeze not monotone in Tt4: {s:?}"
    );
    assert!(s[6] > s[0] + 1e-3, "the freeze point does not move hot vs lean: {s:?}");
    // The lean end is frozen-from-entry; the crossing then walks downstream as Tt4 climbs.
    assert_eq!(s[0], 0.0);
    assert_eq!(s[1], 0.0);
    assert!(s[3] > 0.0);
    assert!(s[5] > s[3]);
}

// --- GATE 6: THE KILL TEST — density drives the freeze DESPITE an opposing T effect ----------- //

/// On the STANDALONE clock (`x_OH` pinned at the frozen entry): killing the temperature in `k(T)`
/// leaves density alone driving, and it STILL freezes; pinning the density leaves temperature
/// alone, and `Da` RISES — no freeze. That is the OPPOSITE sign to Arrhenius intuition, which
/// predicts `Da` falls on cooling, and it is what makes the freeze density-driven.
///
/// Non-circular: the clock is evaluated directly rather than read off the march it explains.
#[test]
fn kill_test_density_drives_against_opposing_temperature() {
    let d = dp(2200.0);
    let ce = entry(&d);
    // The exit temperature by a plain CPG expansion — a float exponent, so libm `pow`.
    let t_ex = d.tt9 * powp(d.p9 / d.pt9, (1.30 - 1.0) / 1.30);
    let tau_res = 0.5 / (0.6 * d.cycle_v9); // FreezeOut default L=0.5, pinned to the cycle V9
    let da = |tau: f64| tau_res / tau;

    let da_entry = da(tau_chem_recomb(&ce, d.tt9, d.pt9, None, None));
    let da_real = da(tau_chem_recomb(&ce, t_ex, d.p9, None, None));
    let da_kill_t = da(tau_chem_recomb(&ce, t_ex, d.p9, Some(d.tt9), None)); // density alone
    let c_m_in = d.pt9 / (RU * d.tt9) / 1.0e6;
    let da_kill_p = da(tau_chem_recomb(&ce, t_ex, d.p9, None, Some(c_m_in))); // temperature alone

    assert!(da_entry > 1.0 && da_real < 1.0, "the real flow must freeze: {da_entry} → {da_real}");
    assert!(da_kill_t < 1.0, "kill-T should still freeze (density alone): {da_kill_t}");
    assert!(da_kill_p > 1.0, "kill-p should NOT freeze — Da rises (T alone): {da_kill_p}");
    assert!(da_kill_p > da_entry, "cooling should RAISE Da when p is pinned");
}

/// The clock returns `+∞` when there is no radical to recombine — so `Da_local → 0` and the flow
/// is frozen by construction. The zero-`x_OH` branch, which no march reaches.
#[test]
fn no_radical_means_an_infinite_clock() {
    let d = dp(2200.0);
    let ce = entry(&d);
    let no_oh: Vec<(&'static str, f64)> =
        ce.iter().map(|&(sp, n)| (sp, if sp == "OH" { 0.0 } else { n })).collect();
    assert!(tau_chem_recomb(&no_oh, 1800.0, 1.0e5, None, None).is_infinite());
    assert!(tau_chem_recomb(&ce, 1800.0, 1.0e5, None, None).is_finite());
}

// --- GATE 7: 2nd LAW — dS ≥ 0 for the freeze-out flow ---------------------------------------- //

/// **THE PYTHON'S BAR HERE IS NOT A PHYSICAL STATEMENT, and widening the sweep is what showed it.**
///
/// `test_rung26.py` asserts `dS_freeze > -1e-6` at `Tt4 ∈ {1500, 1800, 2200}`. Measured on the
/// same five design points the oracle uses, `dS` rises monotonically with `Tt4`:
///
/// ```text
/// Tt4    1300        1500        1800       2200       2300
/// dS    -2.077e-05  +1.211e-04  +3.202e-03 +3.947e-02 +7.056e-02
/// Da_e   0.0654      0.3098      1.4465     4.5012     5.1538
/// ```
///
/// At 1300 K the anchored clock never switches on (`Da_entry` = 0.065), so there is essentially no
/// relaxation and therefore essentially no entropy to produce — the trapezoid truncation is then
/// larger than the physical signal and sets the sign. `-1e-6` survives only because the Python
/// never evaluates it below 1500 K, where `dS` is already 1.2e-04. It is a bar fitted to the
/// points it was looked at, exactly as § 4.11 probe 3 predicted from the other side (the worst
/// truncation is at the FROZEN limit).
///
/// So this gate says the two things that are actually true, separately, instead of conflating
/// them into one threshold:
///
/// 1. the 2nd law holds everywhere against the code's OWN floor, which is the shipped physical
///    guard — and the worst point clears it by 240×;
/// 2. entropy is genuinely PRODUCED wherever there is relaxation to produce it (`Da_entry > 1`),
///    strictly and with margin; and
/// 3. `dS` is MONOTONE in `Tt4` across the whole ladder — a statement the Python never makes, and
///    a far sharper detector than any threshold, because it fails if a march goes wrong anywhere
///    on the ladder rather than only below a floor.
#[test]
fn entropy_production_is_monotone_and_positive_where_the_clock_runs() {
    let mut rows: Vec<(f64, f64, f64)> = Vec::new(); // (Tt4, dS, Da_entry)
    for tt4 in [1300.0, 1500.0, 1800.0, 2200.0, 2300.0] {
        let st = fz(&dp(tt4), FreezeOut::default());
        rows.push((tt4, st.ds_freeze, st.da_entry));
    }

    // (1) the code's own 2nd-law floor, everywhere on the ladder.
    for &(tt4, ds, _) in &rows {
        assert!(
            ds > turbojet::march::DS_FLOOR,
            "dS={ds:e} at Tt4={tt4} breaches the shipped floor {}",
            turbojet::march::DS_FLOOR
        );
    }

    // (2) strictly positive wherever the relaxation actually switches on.
    for &(tt4, ds, da_entry) in &rows {
        if da_entry > 1.0 {
            assert!(ds > 1e-4, "dS={ds:e} at Tt4={tt4} with Da_entry={da_entry} > 1");
        }
    }

    // (3) monotone in Tt4 — the sharp arm.
    assert!(
        rows.windows(2).all(|w| w[0].1 < w[1].1),
        "dS not monotone in Tt4: {:?}",
        rows.iter().map(|r| r.1).collect::<Vec<_>>()
    );
}

// --- GATE 8: ATOM CONSERVATION (the vector-relaxation free invariant) ------------------------ //

#[test]
fn atoms_conserved() {
    fn atoms(c: &[(&str, f64)]) -> (f64, f64, f64) {
        let g = |name: &str| c.iter().find(|&&(s, _)| s == name).map_or(0.0, |&(_, n)| n);
        (
            g("CO2") + g("CO"),
            2.0 * g("H2O") + 2.0 * g("H2") + g("OH") + g("H"),
            2.0 * g("CO2") + g("CO") + g("H2O") + g("OH") + g("O") + 2.0 * g("O2"),
        )
    }
    let d = dp(2200.0);
    let ce = entry(&d);
    let da_fn = |_: &[(&'static str, f64)], _: f64, _: f64| 3.0;
    let (m, _, _, _) = freeze_out_expand(&ce, d.far, d.tt9, d.pt9, d.p9, &da_fn, 400, None);
    let (c0, h0, o0) = atoms(&ce);
    let (c1, h1, o1) = atoms(&m.comp9);
    let worst = (c1 - c0).abs().max((h1 - h0).abs()).max((o1 - o0).abs());
    assert!(worst < 1e-12, "atoms not conserved: {worst:e}");
}

// --- GATE 9: CYCLE UNTOUCHED (a pure diagnostic) --------------------------------------------- //

#[test]
fn cycle_untouched() {
    let d = dp(2200.0);
    let (far_before, v9_before) = (d.far, d.cycle_v9);
    let _ = fz(&d, FreezeOut::default());
    let r = build_turbojet(Gas::reacting_equilibrium(), PI_C, 2200.0, 50_000.0, losses())
        .run(&flight(), 1.0);
    assert_eq!(r.station("4").far.to_bits(), far_before.to_bits());
    assert_eq!(r.v9.to_bits(), v9_before.to_bits());
}

// --- GATE 10: GUARDS ------------------------------------------------------------------------- //

#[test]
#[should_panic(expected = "FreezeOut.L=0 must be positive")]
fn guard_length_must_be_positive() {
    FreezeOut { l: 0.0, ..Default::default() }.validate();
}

#[test]
#[should_panic(expected = "too coarse")]
fn guard_nstep_below_100_is_refused() {
    FreezeOut { nstep: 99, ..Default::default() }.validate();
}

#[test]
#[should_panic(expected = "rate_scale=0 must be positive")]
fn guard_rate_scale_must_be_positive() {
    FreezeOut { rate_scale: 0.0, ..Default::default() }.validate();
}

#[test]
#[should_panic(expected = "needs the rung-6 equilibrium gas")]
fn guard_requires_the_equilibrium_gas() {
    let d = dp(2200.0);
    Gas::thermally_perfect().freeze_out_nozzle(
        d.far,
        d.tt4,
        d.pt4,
        d.tt9,
        d.pt9,
        d.p9,
        FreezeOut::default(),
    );
}

#[test]
#[should_panic(expected = "exceeds pt9")]
fn guard_rejects_back_pressure_above_total() {
    let d = dp(2200.0);
    d.gas.freeze_out_nozzle(d.far, d.tt4, d.pt4, d.tt9, d.pt9, d.pt9 * 1.5, FreezeOut::default());
}
