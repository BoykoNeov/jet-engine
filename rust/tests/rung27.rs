//! Rung-27 verification: NO FREEZE-OUT — is the frozen-NO assumption every NO number carries
//! EARNED?
//!
//! Every NO number since rung 7 ASSUMES the station-4 exhaust NO freezes through the nozzle, and
//! the rung-14/17 dropped-clamp corollary reads `max_a ≫ 1` OFF that assumption. Rung 26 then
//! showed the MAJOR pool freezes only partway down. Rung 27 asks the same of NO — and, applying
//! rung 26's machinery to a clock built from rung 7's OWN Zeldovich reverse rates (zero new
//! constants), finds the assumption is EARNED: `Da_NO ≪ 1` from entry at EVERY `Tt4`.
//!
//! The kill test INVERTS rung 26's: that clock is `Ea = 0` and termolecular, so density won
//! DESPITE an opposing rate constant; this one is Arrhenius and bimolecular, so both factors AGREE
//! and both drive freezing.
//!
//! **WHERE THIS SUITE SAYS MORE THAN THE PYTHON'S.**
//!
//! 1. **The `max_a` ARGMAX is gated, and the Python cannot gate it.** The source hedges that "a
//!    relaxed one may peak earlier" than the cold exit; measured over 5 design points × 4 rate
//!    scales spanning `1e-12 … 1e12`, it never does. The Python returns `max_a` without an index,
//!    so this is a claim only the port can make — and it is made here rather than in the oracle,
//!    because a dumped class only one side can produce is § 4.12 finding 5 repeated on purpose.
//! 2. **"Neighbours untouched" is rewritten so it can fail**, for the reason `rung26.rs` gives:
//!    both diagnostics take `&self` with no interior mutability, so the Python's version is a
//!    compiler guarantee here. What is gated instead is that the three diagnostics AGREE
//!    bit-for-bit on the references they each compute.
//! 3. **The frozen-from-entry census runs five design points against rung 26's two**, which is
//!    what makes the CONTRAST between the two clocks the measurement rather than the claim.

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{equilibrium_composition, Gas};
use turbojet::march::{
    no_freeze_out_expand, tau_chem_recomb, tau_no_destroy, FreezeOut, NoFreezeOut,
    NoFreezeOutNozzleState,
};
use turbojet::nox::ZonedNoxOpts;

const PI_C: f64 = 10.0;
const PHI_P: f64 = 1.0;

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
    tt3: f64,
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
    let (s3, s4, s9) = (r.station("3"), r.station("4"), r.station("9"));
    Dp {
        far: s4.far,
        tt3: s3.tt,
        tt4: s4.tt,
        pt4: s4.pt,
        tt9: s9.tt,
        pt9: s9.pt,
        p9: r.p9,
        cycle_v9: r.v9,
        gas: eng.gas,
    }
}

fn nf(d: &Dp, cfg: NoFreezeOut) -> NoFreezeOutNozzleState {
    d.gas.no_freeze_out_nozzle(d.far, d.tt3, d.tt4, d.pt4, d.tt9, d.pt9, d.p9, PHI_P, cfg)
}

fn entry(d: &Dp) -> Vec<(&'static str, f64)> {
    equilibrium_composition(d.far, d.tt4, d.pt4)
}

// --- GATE 1: NEIGHBOURS UNTOUCHED — rewritten so it can fail --------------------------------- //

/// The three nozzle diagnostics must agree BIT-FOR-BIT on the references they each compute.
///
/// The Python's version calls `nozzle_flow`, then `no_freeze_out_nozzle`, then `nozzle_flow`
/// again and asserts nothing moved. In Rust that cannot fail — every method takes `&self` with no
/// interior mutability. What is gated instead has content: rung 27 reaches the rung-14 clamp and
/// the frozen exit through `nozzle_flow`, and rung 26 reaches the same frozen expansion, so a
/// rung-27 that recomputed either by its own route would agree to digits and fail here.
#[test]
fn the_diagnostics_share_their_references_bit_for_bit() {
    for tt4 in [1500.0, 2200.0] {
        let d = dp(tt4);
        let zn = d.gas.zoned_nox(d.far, d.tt3, d.tt4, d.pt4, PHI_P, ZonedNoxOpts::default());
        let flow = d.gas.nozzle_flow(d.far, d.tt4, d.pt4, d.tt9, d.pt9, d.p9, Some(zn.x_no_mix));
        let s = nf(&d, NoFreezeOut::default());
        assert_eq!(s.t9_frozen.to_bits(), flow.t9_frozen.to_bits());
        assert_eq!(s.x_no_e_entry.to_bits(), flow.x_no_e_entry.to_bits());
        assert_eq!(s.x_no_e_exit.to_bits(), flow.x_no_e_exit.to_bits());
        assert_eq!(s.max_a_frozen.to_bits(), flow.max_a.unwrap().to_bits());
        assert_eq!(s.x_no_frozen.to_bits(), zn.x_no_mix.to_bits());

        let fz = d.gas.freeze_out_nozzle(
            d.far,
            d.tt4,
            d.pt4,
            d.tt9,
            d.pt9,
            d.p9,
            FreezeOut::default(),
        );
        assert_eq!(fz.t9_frozen.to_bits(), flow.t9_frozen.to_bits());
    }
}

// --- GATE 2: REDUCE — Da_NO off ⇒ the clamp IS rung 14/17's, bit-for-bit ---------------------- //

#[test]
fn da_off_is_the_rung14_clamp_bit_for_bit() {
    let d = dp(2200.0);
    let s0 = nf(&d, NoFreezeOut { rate_scale: 1e-12, ..Default::default() });
    assert_eq!(s0.x_no_relaxed.to_bits(), s0.x_no_frozen.to_bits(), "NO moved at rate_scale→0");
    assert_eq!(s0.max_a.to_bits(), s0.max_a_frozen.to_bits(), "clamp is not rung 14/17's");

    // …and the same through the marcher directly, with a literal zero rate: exit == entry
    // EXACTLY, and the exit temperature matches the rung-14 bisection so the clamp denominator is
    // the same number rather than merely a close one.
    let ce = entry(&d);
    let zero = |_: &[(&'static str, f64)], _: f64, _: f64| 0.0;
    let (t9m, x_out, _, max_a_m, da_e, da_x) =
        no_freeze_out_expand(&ce, d.tt9, d.pt9, d.p9, s0.x_no_frozen, &zero, 400);
    assert_eq!(x_out.to_bits(), s0.x_no_frozen.to_bits());
    assert_eq!(t9m.to_bits(), s0.t9_frozen.to_bits());
    assert_eq!(da_e, 0.0);
    assert_eq!(da_x, 0.0);
    assert_eq!(max_a_m.to_bits(), s0.max_a_frozen.to_bits());
}

// --- GATE 3: LIMIT — rate_scale→∞ ⇒ NO tracks equilibrium ⇒ the clamp goes DORMANT ------------ //

#[test]
fn rate_scale_infinity_makes_the_clamp_dormant() {
    let d = dp(2200.0);
    let fast = nf(&d, NoFreezeOut { rate_scale: 1e12, ..Default::default() });
    assert!(
        fast.relaxed_fraction() > 0.99,
        "rate_scale→∞ did not equilibrate: {}",
        fast.relaxed_fraction()
    );
    assert!((fast.max_a - 1.0).abs() < 0.05, "clamp not dormant: max_a={}", fast.max_a);
    assert!(fast.max_a < nf(&d, NoFreezeOut::default()).max_a);
}

// --- GATE 4: THE FINDING — FROZEN FROM ENTRY at EVERY Tt4, unlike the major pool -------------- //

/// The rung, as a contrast rather than a claim: NO is frozen from entry at all five design points
/// while the MAJOR pool relaxes at three of them. Rung 26's census on the same ladder is 2 of 5.
#[test]
fn no_is_frozen_from_entry_at_every_tt4_unlike_the_major_pool() {
    let mut no_frozen = 0usize;
    let mut pool_frozen = 0usize;
    for tt4 in [1300.0, 1500.0, 1800.0, 2200.0, 2300.0] {
        let d = dp(tt4);
        let s = nf(&d, NoFreezeOut::default());
        assert!(s.frozen_from_entry(), "NO not frozen from entry at Tt4={tt4}");
        assert!(s.da_entry < 1.0);
        no_frozen += 1;
        let major =
            d.gas.freeze_out_nozzle(d.far, d.tt4, d.pt4, d.tt9, d.pt9, d.p9, FreezeOut::default());
        if major.frozen_from_entry() {
            pool_frozen += 1;
        }
    }
    assert_eq!(no_frozen, 5, "the NO census must be 5 of 5 — that is the rung");
    assert_eq!(pool_frozen, 2, "the major-pool census must be 2 of 5 — that is the CONTRAST");
}

// --- GATE 5: THE KILL TEST — both terms AGREE, INVERTING rung 26 ------------------------------ //

/// Both of this clock's factors drive freezing, where rung 26's oppose. Evaluated on the
/// STANDALONE clock so the test is not read off the march it explains.
#[test]
fn kill_test_both_terms_drive_inverting_rung26() {
    let d = dp(2200.0);
    let ce = entry(&d);
    let t_ex_temp = 900.0; // a representative cold exhaust state
    let (t_in, p_in) = (d.tt9, d.pt9);
    let (t_ex, p_ex) = (t_ex_temp, d.p9);

    let tau_in = tau_no_destroy(&ce, t_in, p_in, None, None);
    let tau_ex = tau_no_destroy(&ce, t_ex, p_ex, None, None);
    let tau_kill_t = tau_no_destroy(&ce, t_ex, p_ex, Some(t_in), None); // density alone
    let c_in = p_in / (turbojet::gas::RU * t_in);
    let tau_kill_c = tau_no_destroy(&ce, t_ex, p_ex, None, Some(c_in)); // temperature alone

    assert!(tau_ex > tau_in, "net: τ_NO must GROW on cooling (freezes harder)");
    assert!(tau_kill_t > tau_in, "kill-T (density alone) should DRIVE freezing (τ grows)");
    assert!(tau_kill_c > tau_in, "kill-c (T alone) should DRIVE freezing — THE INVERSION");

    // …and rung 26's clock, on the same states, goes the OTHER way when its density is pinned:
    // τ SHRINKS, which is the sign rung 27 inverts. Asserted here so the inversion is a
    // measurement against the other clock rather than a statement about this one alone.
    let r_in = tau_chem_recomb(&ce, t_in, p_in, None, None);
    let c_m_in = p_in / (turbojet::gas::RU * t_in) / 1.0e6;
    let r_kill_m = tau_chem_recomb(&ce, t_ex, p_ex, None, Some(c_m_in));
    assert!(r_kill_m < r_in, "rung 26's kill-M should make τ SHRINK — the sign rung 27 inverts");
}

// --- GATE 6: MARGIN TREND — the separation NARROWS with Tt4, without crossing ----------------- //

#[test]
fn the_margin_narrows_with_tt4_without_crossing() {
    let mut da_no: Vec<f64> = Vec::new();
    let mut seps: Vec<f64> = Vec::new();
    for tt4 in [1500.0, 1800.0, 2200.0] {
        let d = dp(tt4);
        let s = nf(&d, NoFreezeOut::default());
        let pool =
            d.gas.freeze_out_nozzle(d.far, d.tt4, d.pt4, d.tt9, d.pt9, d.p9, FreezeOut::default());
        da_no.push(s.da_entry);
        seps.push(pool.da_entry / s.da_entry);
    }
    assert!(seps.windows(2).all(|w| w[0] > w[1]), "separation not narrowing: {seps:?}");
    assert!(seps[0] / seps[2] > 1e3, "separation should collapse orders: {seps:?}");
    assert!(da_no.iter().all(|&x| x < 1.0), "Da_NO crossed 1 — no crossing is claimed: {da_no:?}");
    assert!(da_no.windows(2).all(|w| w[0] < w[1]), "Da_NO entry not monotone: {da_no:?}");
}

// --- GATE 7/8: the clamp fires on the MARCHED NO, and scales linearly with the NO level ------- //

#[test]
fn the_clamp_fires_on_the_marched_no() {
    for tt4 in [1500.0, 1800.0, 2200.0, 2300.0] {
        let s = nf(&dp(tt4), NoFreezeOut::default());
        assert!(s.clamp_fires(), "clamp does not fire at Tt4={tt4}: max_a={}", s.max_a);
        // The marched clamp is the frozen one to within the anchored margin, which is ≪ 1.
        let rel = (s.max_a - s.max_a_frozen).abs() / s.max_a_frozen;
        assert!(rel < 1e-2, "marched clamp differs from frozen by {rel:e} at Tt4={tt4}");
    }
}

/// The clock is `[NO]`-INDEPENDENT (the `[NO]_e` in the reverse rates cancels the `a` in the
/// linearised numerator), so the clamp scales with the NO level fed in.
///
/// **THE PROPORTIONALITY IS EXACT ONLY IN THE REDUCE, and this gate says both halves.** The
/// Python drives the march at a literal ZERO rate, where `x_no` never changes and the clamp is
/// therefore strictly linear — that is arm 1, kept at its `1e-9` bar. At the ANCHORED rate the
/// relaxation step `x_no += relax·(x_no_e − x_no)` is AFFINE rather than linear, because the
/// `relax·x_no_e` term does not scale with the input.
///
/// Measured departure of `a(2x)/a(x)` from 2, at the zoned NO level:
///
/// ```text
/// Tt4        1500       1800       2200       2300
/// departure  3.11e-10   8.81e-07   6.76e-04   2.18e-03
/// ```
///
/// **Nine orders across the ladder**, rising monotonically — it tracks the residual relaxation,
/// which is why it is invisible lean and visible hot. So arm 2 gates the SHAPE rather than a
/// threshold: a single bar would only say where it was fitted, which is the mistake § 4.12
/// finding 3 records against the source's own 2nd-law gate. (A first draft of this arm did fit a
/// bar, at the wrong NO level, and it failed — the departure is ~10× larger at `x_no` = 1e-4 than
/// at the zoned value, because the affine offset matters more when the level is smaller.)
#[test]
fn the_clamp_scales_with_the_no_level_exactly_frozen_and_affine_when_relaxing() {
    let d = dp(2200.0);
    let ce = entry(&d);

    // Arm 1 — the FROZEN reduce: a literal zero rate, so `x_no` never moves and the clamp is
    // strictly proportional to the level fed in.
    let zero = |_: &[(&'static str, f64)], _: f64, _: f64| 0.0;
    let (_, _, _, a1, _, _) = no_freeze_out_expand(&ce, d.tt9, d.pt9, d.p9, 1e-4, &zero, 400);
    let (_, _, _, a2, _, _) = no_freeze_out_expand(&ce, d.tt9, d.pt9, d.p9, 2e-4, &zero, 400);
    assert!((a2 / a1 - 2.0).abs() < 1e-9, "frozen clamp not linear in the NO level: {}", a2 / a1);

    // Arm 2 — the ANCHORED rate: affine, and the departure RISES with Tt4 alongside the residual
    // relaxation. The shape is the claim; the endpoints only bound it.
    let mut departures: Vec<f64> = Vec::new();
    for tt4 in [1500.0, 1800.0, 2200.0, 2300.0] {
        let dd = dp(tt4);
        let cc = entry(&dd);
        let zn = dd.gas.zoned_nox(dd.far, dd.tt3, dd.tt4, dd.pt4, PHI_P, ZonedNoxOpts::default());
        let flow = dd.gas.nozzle_flow(dd.far, dd.tt4, dd.pt4, dd.tt9, dd.pt9, dd.p9, None);
        let tau_res = 0.5 / (0.6 * flow.v9_frozen);
        let da_no = move |comp: &[(&'static str, f64)], t: f64, p: f64| {
            tau_res / tau_no_destroy(comp, t, p, None, None)
        };
        let x = zn.x_no_mix;
        let (_, _, _, b1, _, _) =
            no_freeze_out_expand(&cc, dd.tt9, dd.pt9, dd.p9, x, &da_no, 400);
        let (_, _, _, b2, _, _) =
            no_freeze_out_expand(&cc, dd.tt9, dd.pt9, dd.p9, 2.0 * x, &da_no, 400);
        departures.push((b2 / b1 - 2.0).abs());
    }
    assert!(
        departures.windows(2).all(|w| w[0] < w[1]),
        "the affine departure should RISE with Tt4 alongside the relaxation: {departures:?}"
    );
    assert!(
        departures[0] < 1e-8,
        "lean, the march is effectively frozen and the clamp should be proportional: {}",
        departures[0]
    );
    assert!(
        departures[3] > 1e-4,
        "hot, the relaxation term must be LIVE — if the departure vanishes it is dead code: {}",
        departures[3]
    );
}

// --- THE ARGMAX TRIPWIRE — a claim only the port can make ------------------------------------ //

/// **The `max_a` peak is at the EXIT, always — and the source hedges that it might not be.**
///
/// `no_freeze_out_expand`'s comment says "equilibrium NO is monotone in T, so a frozen NO peaks at
/// the cold exit; a relaxed one may peak earlier". Measured over 5 design points × 4 rate scales
/// spanning `1e-12` to `1e12` — including cells where NO is 70–97 % relaxed — the hedged case
/// never occurs.
///
/// This is a TRIPWIRE rather than a discriminator, and it is recorded as one: `max_a` is already
/// gated at bit-equality by the oracle, so what the location adds is that a march which started
/// peaking mid-path would fail HERE, naming the trajectory, instead of showing up as a changed
/// number. It cannot live in the oracle: the Python returns `max_a` without an index and would
/// need instrumenting to report one, and dumping a class only one side can produce is exactly the
/// defect § 4.12 finding 5 records.
#[test]
fn the_clamp_peak_is_always_at_the_exit() {
    for tt4 in [1300.0, 1500.0, 1800.0, 2200.0, 2300.0] {
        let d = dp(tt4);
        let ce = entry(&d);
        let s = nf(&d, NoFreezeOut::default());
        let flow = d.gas.nozzle_flow(d.far, d.tt4, d.pt4, d.tt9, d.pt9, d.p9, None);
        let tau_res = 0.5 / (0.6 * flow.v9_frozen);
        for rs in [1e-12, 1.0, 1e6, 1e12] {
            let da_no = move |comp: &[(&'static str, f64)], t: f64, p: f64| {
                rs * tau_res / tau_no_destroy(comp, t, p, None, None)
            };
            // `max_a` is the max over the whole trajectory INCLUDING the exit, and the exit's own
            // ratio is `x_no/x_no_e` from the same two returned values — the identical
            // expression. So they are BIT-equal exactly when the exit won, and `max_a` is
            // strictly larger exactly when some interior step did.
            let (_, x_exit, x_e_exit, max_all, _, _) =
                no_freeze_out_expand(&ce, d.tt9, d.pt9, d.p9, s.x_no_frozen, &da_no, 400);
            let a_exit = x_exit / x_e_exit;
            assert_eq!(
                max_all.to_bits(),
                a_exit.to_bits(),
                "the clamp peaked BEFORE the exit at Tt4={tt4}, rate_scale={rs}: \
                 trajectory max {max_all} vs exit {a_exit}"
            );
        }
    }
}

// --- GATE 9: CYCLE UNTOUCHED, and GATE 10: GUARDS -------------------------------------------- //

#[test]
fn cycle_untouched() {
    let d = dp(2200.0);
    let (far_before, v9_before) = (d.far, d.cycle_v9);
    let _ = nf(&d, NoFreezeOut::default());
    let r = build_turbojet(Gas::reacting_equilibrium(), PI_C, 2200.0, 50_000.0, losses())
        .run(&flight(), 1.0);
    assert_eq!(r.station("4").far.to_bits(), far_before.to_bits());
    assert_eq!(r.v9.to_bits(), v9_before.to_bits());
}

#[test]
#[should_panic(expected = "too coarse")]
fn guard_nstep_below_100_is_refused() {
    NoFreezeOut { nstep: 99, ..Default::default() }.validate();
}

#[test]
#[should_panic(expected = "rate_scale=0 must be positive")]
fn guard_rate_scale_must_be_positive() {
    NoFreezeOut { rate_scale: 0.0, ..Default::default() }.validate();
}

#[test]
#[should_panic(expected = "needs the rung-6 equilibrium gas")]
fn guard_requires_the_equilibrium_gas() {
    let d = dp(2200.0);
    Gas::thermally_perfect().no_freeze_out_nozzle(
        d.far,
        d.tt3,
        d.tt4,
        d.pt4,
        d.tt9,
        d.pt9,
        d.p9,
        PHI_P,
        NoFreezeOut::default(),
    );
}

#[test]
#[should_panic(expected = "exceeds pt9")]
fn guard_rejects_back_pressure_above_total() {
    let d = dp(2200.0);
    d.gas.no_freeze_out_nozzle(
        d.far,
        d.tt3,
        d.tt4,
        d.pt4,
        d.tt9,
        d.pt9,
        d.pt9 * 1.5,
        PHI_P,
        NoFreezeOut::default(),
    );
}
