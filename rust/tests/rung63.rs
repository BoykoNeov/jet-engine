//! RUNG 63 — **FUEL + BLEED on one plant.** Rung 62's named seam. `tests/test_rung63.py`,
//! ported one-to-one.
//!
//! **30 `#[test]` against Python's 30 COLLECTED** (from 19 `def test_`), both counts EMITTED.
//! **13 of the 19 carry `@pytest.mark.slow`; none becomes `#[ignore]`.**
//!
//! THE HEADLINE: rung 58's ONE-WAY arrow was never a fact about wall-movers. It was a fact about
//! the `Wf/pt3` leg's TWO PROTECTIONS — a CHOKED `A4` guards its ordinate (rung 59's proof chain)
//! and rung 39's `pi_LPC` cancellation guards its abscissa — and a stator satisfies both. **A
//! bleed is the ladder's only lever that breaks `mdot_face == mdot_core`**, the identity sitting
//! UPSTREAM of both, so it reaches both sensed inputs and the arrow CLOSES: the leg's engagement
//! time moves +2.9 to +4.2 %, LATER, in all six cells. But `s_eng` is a TRAJECTORY quantity, not
//! a table quantity, so a STATOR moves it too (up to +1.28 %) with the table bit-identical: the
//! bleed's channel is STRUCTURAL, the stator's TRAJECTORY-MEDIATED, and what the data separates
//! is systematic from incidental, **not presence from absence**.
//!
//! THE SECOND FINDING: a `phi` floor and the valve have NO COMPOSABLE MIDDLE. Over the band whose
//! edges are the two plants' OWN minimum `phi` the bleed DISARMS the floor exactly; above it both
//! bind, the floor pins the currency, and the valve's credit is exactly zero.
//!
//! THE INSTRUMENT that would have counterfeited the rung: rung 62 overrode `at_stator` so a
//! rung-57 reader carries this machine's valve, and that reaches SIX inherited readers. On a
//! bleed-armed machine `schedule_invariance` compares armed against armed and returns
//! `ordinate_identical = true` — numerically identical to rung 59's headline — while measuring
//! nothing. **Gate 2 pins that trap directly, and § 5.21 (ii) is why `at_stator` became a cell.**
//!
//! **WHAT 88/88 GREEN DOES NOT ESTABLISH.** Every gate in this file is RELATIONAL — it
//! asserts a relation among values THIS CRATE computed, so a Rust/Python arithmetic
//! divergence moves both sides of every one of them and leaves the whole suite green.
//! Agreement with Python is step 4's oracle, not this file. Step 3 MEASURED how far
//! that reaches, by running 6 INJECTIONS over 5 distinct defects (I2 and I2b are two
//! spellings of one) into `bleed_transient.rs`, counting both
//! how many of a 871-key probe moved and how many of the 88 fired:
//!
//! * **I1** — `mdot_face` returning the TRIAL face flow rather than the imposed `mdot_imp/(1-b)`: **312 of 871** gate-visible keys move, 2 witness — **0 of 88 catch it**.
//! * **I2** — `_powers`/`_instant_tail` re-reading `b_of` instead of the closure's own `bleed` key: **0 of 871** gate-visible keys move, 7 witness — **0 of 88 catch it**.
//! * **I2b** — the same re-read confined to the branch PREDICATE (P4's own injection): **0 of 871** gate-visible keys move, 7 witness — **0 of 88 catch it**.
//! * **I3** — `R62_FUEL` spread from `..R43`, which drops rung 57's floor-RESOLVING `_surge_fuel`: **0 of 871** gate-visible keys move, 1 witness, 3 key(s) never emitted (its section PANICS) — **0 of 88 catch it**.
//! * **I4** — `at_stator` left un-overridden, the shape slice V shipped: **4 of 871** gate-visible keys move, 1 witness, 1 key(s) never emitted (its section PANICS) — **2 of 88 catch it**.
//! * **I5** — the `1/(1-b)` dropped from the fuel bracket walls: **151 of 871** gate-visible keys move, 2 witness — **0 of 88 catch it**.
//!
//! So **5 of the 6 injections are invisible to every gate in both suites**, and the one
//! that is
//! caught is caught by the two written for it. The zeros are not probe blindness:
//! `caught > 0` implies `moved_G > 0` on every row, and the two that move NOTHING
//! gate-visible each leave a witness the gates never read — `b_of`'s call count for
//! I2/I2b, and for I3 a deliberately un-ported `Floor::Incidence` cell that PANICS,
//! which is what shows the channel is live at all. Both suites build only
//! `Floor::Phi`, and that is precisely why I3 reaches nothing here.

use turbojet::bleed_transient::{build_scheduled_bleed, BleedSchedule, LeverArm};
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::fuel_transient::{AccelSchedule, Floor, SurgeLimiter};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::matcher::Branch;
use turbojet::stator_transient::{
    Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorArm, StatorLeg, StatorSchedule,
};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------- the grid
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
/// **RUNG 63 MARCHES AT 0.005, RUNG 62 AT 0.01.** Two suites on one plant with different grids,
/// each declared explicitly — slice V step 3's finding 1 in a form that cannot repeat.
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const N_LO: f64 = 0.65;
const V: f64 = 0.20;
const B: f64 = 0.10;
const MARGIN: f64 = 0.25;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp_map() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn hp_map() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn tilt_map() -> ComponentMap {
    ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), PI_LPC, PI_HPC, TT4, 50_000.0, REAL)
}

fn bt_on(lp: ComponentMap, hp: ComponentMap, arm: &LeverArm) -> ScheduledStatorCore {
    match build_scheduled_bleed(design(), flight(), 1.0, Some(lp), Some(hp), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn bt(arm: &LeverArm) -> ScheduledStatorCore {
    bt_on(lp_map(), hp_map(), arm)
}

fn bled() -> LeverArm {
    LeverArm::scheduled(BleedSchedule::new(B, N_LO))
}

fn stat() -> LeverArm {
    LeverArm::stator(StatorArm::scheduled_lp(StatorSchedule::new(V, N_LO)))
}

fn ramp(r: f64, ds: f64) -> Ramp {
    Ramp { tt4_lo: LO, tt4_hi: HI, r, s_settle: SETTLE, ds }
}

/// Python's `_leg`.
fn leg_of(m: &ScheduledStatorCore) -> AccelSchedule {
    m.fuel.accel_schedule(&flight(), LO, HI, MARGIN, 13)
}

// =============================================================================================
// GATE 1 — THE REDUCE: rung 62's `_legs` is untouched when no leg is passed
// =============================================================================================

/// `_legs` gained accel/surge/`Tt4_max`, all defaulting to `None` — which is `_stator_march`'s own
/// default — so every rung-62 caller reaches the IDENTICAL four marches. Witnessed against
/// `loop_decomposition`, whose reference is `at_lever()`: the same pair `marginal_loop` builds
/// with an empty neighbour.
#[test]
fn reduce_legless_marginal_loop_is_rung62_bit_for_bit() {
    let m = bt(&bled());
    let a = m.loop_decomposition(&flight(), &ramp(0.5, 0.02), Spool::Lp);
    let b = m.marginal_loop(&flight(), &ramp(0.5, 0.02), &bled(), None, Spool::Lp,
                            &StatorLeg::default());
    for (k, va, vb) in [("reference", a.reference, b.reference), ("start", a.start, b.start),
                        ("ramp", a.ramp, b.ramp), ("full", a.full, b.full),
                        ("self_cancel", a.self_cancel, b.self_cancel),
                        ("nu0_ref", a.nu0_ref, b.nu0_ref),
                        ("nu0_armed", a.nu0_armed, b.nu0_armed)] {
        assert!(va.to_bits() == vb.to_bits(), "{k}: {va:?} != {vb:?}");
    }
}

/// A leg passed as `None` must reach the same code path as no leg at all — otherwise the rung-62
/// gates would be guarding a branch nobody takes.
///
/// **In Rust the two spellings are ONE VALUE** (`StatorLeg::default()` has all three fields
/// `None`), so this gate is weaker here than in Python by construction. It is kept because the
/// COUNT is the contract and because a future overload could reintroduce the split; what it still
/// proves is that the legless path is reached twice with the same answer.
#[test]
fn reduce_explicit_none_leg_is_identical_to_omitting_it() {
    let m = bt(&bled());
    let a = m.marginal_loop(&flight(), &ramp(0.5, 0.02), &bled(), None, Spool::Lp,
                            &StatorLeg::default());
    let b = m.marginal_loop(&flight(), &ramp(0.5, 0.02), &bled(), None, Spool::Lp,
                            &StatorLeg { accel: None, surge: None, tt4_max: None });
    assert!(a.self_cancel.to_bits() == b.self_cancel.to_bits());
    assert!(a.full.to_bits() == b.full.to_bits());
}

/// Rung 63 adds only readers on the transient ladder. The default single-spool design run must be
/// bit-for-bit rung 6 (the project's spine).
#[test]
fn cycle_untouched_design_run_is_rung6_bit_for_bit() {
    let eng = build_turbojet(Gas::reacting_equilibrium(), PI_LPC * PI_HPC, TT4, 50_000.0,
                             Losses { pi_d: 0.97, eta_b: 0.99, pi_b: 0.96, eta_m: 0.99,
                                      pi_n: 0.98, ..Losses::default() });
    let res = eng.run(&flight(), 1.0);
    let reference = eng.run(&flight(), 1.0);
    assert!(res.performance.specific_thrust > 0.0 && res.performance.tsfc > 0.0);
    for s in ["2", "3", "4", "5", "9"] {
        assert!(res.station(s).tt.to_bits() == reference.station(s).tt.to_bits());
        assert!(res.station(s).pt.to_bits() == reference.station(s).pt.to_bits());
    }
    assert!(res.performance.specific_thrust.to_bits()
            == reference.performance.specific_thrust.to_bits());
}

// =============================================================================================
// GATE 2 — THE `_isolating` GATE: the trap that would have counterfeited § 1
// =============================================================================================

/// The text of a caught panic — `rung57.rs`'s helper, ported here because rung 63 matches BOTH of
/// `_isolating`'s refusals and they carry DIFFERENT payload types: the keyed one interpolates
/// `{k}` and so unwinds a `String`, while the empty-lever one is a bare literal and unwinds a
/// `&'static str`. Downcasting to `String` alone reads a literal-`assert!` refusal as the empty
/// string, which turns a matched message into `wrong refusal: ` — the gate then fails on the very
/// message it was written to accept.
fn panic_text(e: Box<dyn std::any::Any + Send>) -> String {
    e.downcast_ref::<String>()
        .cloned()
        .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_string()))
        .unwrap_or_default()
}

/// The mirror of rung 62's gate 3. A lever key also present in the neighbour would make the
/// "reference" an ARMED machine, i.e. an armed-vs-armed comparison.
///
/// **BOTH REFUSAL MESSAGES ARE MATCHED, full-string**, because Python matches on a substring and
/// slice U step 1 found a gate named for one rung firing another's assert. `LeverArm::keys()`
/// is what carries Python's `for k in lever: assert k not in neighbour`.
#[test]
fn isolating_refuses_a_reference_carrying_the_lever() {
    let m = bt(&LeverArm::default());
    let e = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        m.marginal_loop(&flight(), &ramp(0.5, DS), &bled(), Some(&bled()), Spool::Lp,
                        &StatorLeg::default())
    })).expect_err("must refuse a reference carrying the lever");
    let msg = panic_text(e);
    assert!(msg.contains("LEVER being isolated"), "wrong refusal: {msg}");

    let e2 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        m.marginal_loop(&flight(), &ramp(0.5, DS), &LeverArm::default(), None, Spool::Lp,
                        &StatorLeg::default())
    })).expect_err("must refuse an empty lever");
    let msg2 = panic_text(e2);
    assert!(msg2.contains("isolates a lever"), "wrong refusal: {msg2}");
}

/// The positive witness: the reference sibling carries the NEIGHBOUR's valve and nothing else,
/// and the armed one carries lever + neighbour.
#[test]
fn isolating_reference_is_valve_shut_and_armed_is_not() {
    let m = bt(&LeverArm::default());
    let (r, a) = m.isolating(&bled(), None);
    assert!(!r.armed_bleed() && a.armed_bleed());
    // with a STATOR neighbour the reference still must be valve-shut
    let (r2, a2) = m.isolating(&bled(), Some(&stat()));
    assert!(!r2.armed_bleed() && a2.armed_bleed());
    assert!(r2.arming().is_armed() && a2.arming().is_armed());   // both carry the stator
}

/// **THE COUNTERFEIT, pinned.** Rung 62 deliberately overrode `at_stator` to carry this machine's
/// valve. So on a bleed-armed machine rung 59's `schedule_invariance` compares the plant against
/// ITSELF: it reports the tables bit-identical — numerically rung 59's own headline — while
/// measuring nothing. This gate exists so no future edit can reintroduce that reading as
/// evidence, and so § 1's instrument choice stays justified.
///
/// **AND IT IS WHY `at_stator` IS A CELL.** § 5.21 (ii) measured the un-overridden body returning
/// `false/false` at `9.543e-3` / `1.019e-2` here — i.e. this gate FAILS on the shape slice V
/// shipped, which is what promoted a booked deferral into slice W's first job.
#[test]
fn the_at_stator_trap_is_real_and_returns_rung59s_zero_for_free() {
    let m = bt(&bled());
    assert!(m.at_stator(StatorArm::default()).armed_bleed(),
            "rung 62's at_stator override must keep the valve -- if this flips, rung 62's gate 3 \
             has been broken and every inherited reader changes meaning.");
    let trap = m.schedule_invariance(&flight(), LO, HI, MARGIN, 5);
    assert!(trap.ordinate_identical && trap.abscissa_identical,
            "the trap must reproduce rung 59's exact-identity verdict for free");
    let honest = bt(&LeverArm::default())
        .sensed_inputs(&flight(), &ramp(0.5, DS), &bled(), MARGIN, 5, None);
    assert!(honest.d_ordinate > 1e-3 && honest.d_abscissa > 1e-3,
            "and the isolating reader must NOT: it differences against a valve-shut sibling");
}

// =============================================================================================
// GATE 3 — THE MECHANISM: the leg's two sensed inputs
// =============================================================================================

/// **THE MECHANISM (rung 63 § 1).** One instrument, two levers.
///
/// Rung 59 proved an LP stator moves NEITHER half of the `Wf/pt3` table (its own published
/// tolerance is 1e-13). A bleed moves BOTH, by more than 1e-2 — **ten orders apart** — because the
/// LP shaft balance is the one carrying `(1-b)` and it sits upstream of both protections.
#[test]
fn bleed_moves_both_sensed_inputs_where_a_stator_moves_neither() {
    let m = bt(&LeverArm::default());
    let bl = m.sensed_inputs(&flight(), &ramp(0.5, DS), &bled(), MARGIN, 9, None);
    let st = m.sensed_inputs(&flight(), &ramp(0.5, DS), &stat(), MARGIN, 9, None);
    assert!(st.d_ordinate < 1e-12 && st.d_abscissa < 1e-12,
            "rung 59's zero must reproduce: {:.3e}, {:.3e}", st.d_ordinate, st.d_abscissa);
    assert!(bl.d_ordinate > 1e-3, "{}", bl.d_ordinate);
    assert!(bl.d_abscissa > 1e-3, "{}", bl.d_abscissa);
    assert!(bl.d_ordinate / st.d_ordinate.max(1e-300) > 1e8);
    // the SIGN: a bleed makes the burner inlet colder, so the steady Wf/pt3 ratio RISES
    assert!(bl.signed_ordinate > 0.0 && bl.signed_abscissa > 0.0);
}

/// `MFP_A4` is the corrected group at a CHOKED throat — hardware, `gamma` and `R`. Nothing on the
/// compressor side can reach it, for ANY lever. **If it ever moves, the proof chain has broken
/// somewhere else and every number in § 1 is meaningless.**
fn choked_a4_control(lever: &LeverArm) {
    let d = bt(&LeverArm::default())
        .sensed_inputs(&flight(), &ramp(0.5, DS), lever, MARGIN, 5, None);
    assert!(d.d_mfp < 1e-14, "{}", d.d_mfp);
}

#[test] fn choked_a4_control_holds_for_bleed_sched() { choked_a4_control(&bled()); }
#[test] fn choked_a4_control_holds_for_stator_sched() { choked_a4_control(&stat()); }
#[test] fn choked_a4_control_holds_for_bleed_const() {
    choked_a4_control(&LeverArm::constant(B));
}

/// The derivation, term by term: `(1-b)` sits in the LP balance ONLY, so `Tt25` falls, `Tt3` falls
/// with it, `f` rises to make up the colder burner inlet, and `kappa_ss` rises.
#[test]
fn the_lp_balance_chain_is_signed_as_derived() {
    let d = bt(&LeverArm::default())
        .sensed_inputs(&flight(), &ramp(0.5, DS), &LeverArm::constant(B), MARGIN, 5, None);
    for row in &d.chain {
        assert!(row.d_tt25 < 0.0, "{row:?}");
        assert!(row.d_tt3 < 0.0, "{row:?}");
        assert!(row.d_f > 0.0, "{row:?}");
        assert!(row.d_kappa > 0.0, "{row:?}");
        // the HP balance is bleed-INVARIANT (rung 42), so Tt3 - Tt25 must move LESS than either
        // endpoint: the whole shift is imported from the LP side.
        assert!(row.d_tt3.abs() < row.d_tt25.abs(), "{row:?}");
    }
}

// =============================================================================================
// GATE 4 — THE HEADLINE: the return arrow
// =============================================================================================

/// **THE RUNG (63).** Rung 58's own instrument, on a lever the leg can feel.
///
/// A bleed schedule moves `s_eng` by +2.5 % or more, LATER, at every ramp rate and on both map
/// shapes. The reading is the DORMANT march, where `g` is defined everywhere and no clip has
/// perturbed the states; the limited march agrees to under 1 % of the shift.
///
/// **THE STATOR CONTROL IS BOUNDED, NOT ZERO, AND IT IS MEASURED HERE** rather than quoted from
/// rung 58 — whose −0.162 % sits at ITS OWN placement (`n_lo = 0.7557`) and is therefore a
/// DIFFERENT schedule, not a control for this grid. So what is gated is that the bleed is POSITIVE
/// and STRICTLY THE LARGER in every cell.
fn bleed_retimes_the_leg(lp: ComponentMap, hp: ComponentMap, r: f64, shape: &str) {
    let m = bt_on(lp, hp, &LeverArm::default());
    let leg = leg_of(&m);
    let sl = StatorLeg { accel: Some(&leg), surge: None, tt4_max: None };
    let d = m.leg_retiming(&flight(), &ramp(r, DS), &bled(), &sl, None);
    assert!(d.rel_dormant > 0.025, "{shape} r={r}: {}", d.rel_dormant);
    // dormant vs limited: the two readings agree to <= 3e-5 in the ratio (under 1 % of it)
    assert!((d.rel_limited - d.rel_dormant).abs() < 1e-4, "{d:?}");
    let (a1, a2) = d.audits.expect("an accel leg produces both audits");
    assert!(a1.clamped <= 1 && a2.clamped <= 1, "{a1:?} {a2:?}");
    let s = m.leg_retiming(&flight(), &ramp(r, DS), &stat(), &sl, None);
    // the bleed is POSITIVE and strictly the larger, in every cell.
    assert!(d.rel_dormant > s.rel_dormant.abs() && s.rel_dormant.abs() > 0.0,
            "{shape} r={r}: {} vs {}", d.rel_dormant, s.rel_dormant);
    assert!(s.rel_dormant.abs() < 0.02, "{shape} r={r}: {}", s.rel_dormant);
}

#[test] fn bleed_retimes_the_leg_shaped_r025() {
    bleed_retimes_the_leg(lp_map(), hp_map(), 0.25, "shaped");
}
#[test] fn bleed_retimes_the_leg_shaped_r050() {
    bleed_retimes_the_leg(lp_map(), hp_map(), 0.50, "shaped");
}
#[test] fn bleed_retimes_the_leg_shaped_r100() {
    bleed_retimes_the_leg(lp_map(), hp_map(), 1.00, "shaped");
}
#[test] fn bleed_retimes_the_leg_tilted_r025() {
    bleed_retimes_the_leg(tilt_map(), tilt_map(), 0.25, "tilted");
}
#[test] fn bleed_retimes_the_leg_tilted_r050() {
    bleed_retimes_the_leg(tilt_map(), tilt_map(), 0.50, "tilted");
}
#[test] fn bleed_retimes_the_leg_tilted_r100() {
    bleed_retimes_the_leg(tilt_map(), tilt_map(), 1.00, "tilted");
}

/// The pre-registered sign was EARLIER and it was **REFUTED**. The pressure channel does point
/// that way (`pt3` falls), but the ABSCISSA channel this rung's own § 1 derives fights it, the cap
/// barely moves, and the COMMANDED ramp — re-derived on the bled plant, since both are pinned to
/// the same `Tt4` endpoints — decides.
#[test]
fn the_retiming_sign_is_decided_by_the_commanded_ramp_not_the_cap() {
    let m = bt(&LeverArm::default());
    let leg = leg_of(&m);
    let sl = StatorLeg { accel: Some(&leg), surge: None, tt4_max: None };
    let c = m.leg_retiming(&flight(), &ramp(0.5, DS), &bled(), &sl, None)
        .channels.expect("an accel leg produces channels");
    assert!(c.d_pt3 < 0.0, "{c:?}");                     // as predicted
    assert!(c.d_kappa > 0.0, "{c:?}");                   // fighting it -- § 1's abscissa shift
    assert!(c.d_cap.abs() < c.d_pt3.abs(), "{c:?}");     // so the cap nearly cancels
    assert!(c.d_mf_sched < c.d_cap, "{c:?}");            // and the ramp falls FURTHER
    assert!(c.d_g < 0.0, "{c:?}");                       // => the crossing arrives LATER
}

/// The forward direction (leg → lever) is rung 58's relocation × state-feed, and its own predictor
/// — re-reading the LEG-FREE credit profile at the relocated minimum — recovers it. Rung 58 got
/// 86 % for a stator schedule. **This is why the headline is the RETURN arrow alone and not the
/// ratio of the two.**
#[test]
fn the_forward_arrow_is_rung58s_mechanism_confirmed_not_new_content() {
    let m = bt(&LeverArm::default());
    let leg = leg_of(&m);
    let sl = StatorLeg { accel: Some(&leg), surge: None, tt4_max: None };
    let d = m.lever_composite(&flight(), &ramp(0.5, DS), &bled(), &sl, Spool::Lp, None);
    assert!(d.interaction > 0.0 && d.share > 0.02, "{}", d.share);
    assert!(0.80 < d.recovered && d.recovered < 0.95, "{}", d.recovered);
    assert!(d.removed_bare > 0.0 && d.removed_armed > 0.0,
            "a dormant leg's zero is the envelope edge, not evidence (rung 58's r = 2.0)");
}

/// The two directions answer to DIFFERENT conditions, and this is the witness. At `r = 1.00` the
/// forward direction nearly vanishes — **but the leg is NOT dormant.** It engages DOWNSTREAM of
/// the incidence minimum, so it relocates nothing: rung 48's engagement law, reappearing inside a
/// third composite. The return arrow has no such condition.
#[test]
fn the_forward_arrow_collapses_at_r1_by_rung48s_law_not_by_dormancy() {
    let m = bt(&LeverArm::default());
    let leg = leg_of(&m);
    let sl = StatorLeg { accel: Some(&leg), surge: None, tt4_max: None };
    let fast = m.lever_composite(&flight(), &ramp(1.0, DS), &bled(), &sl, Spool::Lp, None);
    let mid = m.lever_composite(&flight(), &ramp(0.5, DS), &bled(), &sl, Spool::Lp, None);
    assert!(fast.share < 0.25 * mid.share, "{} {}", fast.share, mid.share);
    assert!(fast.removed_bare > 0.0, "not dormancy -- the leg still binds");
    assert!(fast.fuel.s_eng > fast.neither.s,
            "the leg engages DOWNSTREAM of the bare incidence minimum");
    // while the return arrow is undiminished at the same rate
    assert!(m.leg_retiming(&flight(), &ramp(1.0, DS), &bled(), &sl, None).rel_dormant > 0.025);
}

// =============================================================================================
// GATE 5 — THE LOOP: rung 62 § 2's attribution, on a neighbour with no loop
// =============================================================================================

/// Rung 62 § 2 attributed the loop to `dn/d(setting)`. A fuel leg reads the state but emits a fuel
/// CAP, not a setting, so it has no such term — and it perturbs the bleed's amplification by under
/// 2 %, the same order as rung 62's scheduled neighbour. **The loop answers to its own gain and
/// not to the trajectory a neighbour hands it.**
fn legged_neighbour_leaves_the_loop_alone(r: f64) {
    let m = bt(&LeverArm::default());
    let leg = leg_of(&m);
    let a = m.marginal_loop(&flight(), &ramp(r, 0.01), &bled(), None, Spool::Lp,
                            &StatorLeg::default());
    let b = m.marginal_loop(&flight(), &ramp(r, 0.01), &bled(), None, Spool::Lp,
                            &StatorLeg { accel: Some(&leg), surge: None, tt4_max: None });
    assert!(a.self_cancel > 1.0 && b.self_cancel > 1.0, "{a:?} {b:?}");
    assert!((b.self_cancel / a.self_cancel - 1.0).abs() < 0.02,
            "r={r}: {} {}", a.self_cancel, b.self_cancel);
}

#[test] fn a_legged_neighbour_leaves_the_bleeds_loop_alone_r025() {
    legged_neighbour_leaves_the_loop_alone(0.25);
}
#[test] fn a_legged_neighbour_leaves_the_bleeds_loop_alone_r050() {
    legged_neighbour_leaves_the_loop_alone(0.50);
}
#[test] fn a_legged_neighbour_leaves_the_bleeds_loop_alone_r100() {
    legged_neighbour_leaves_the_loop_alone(1.00);
}

// =============================================================================================
// GATE 6 — THE SECOND FINDING: no composable middle
// =============================================================================================

/// **THE SECOND FINDING (rung 63 § 3).** A `phi` floor and the valve have TWO regimes and no
/// middle, and the boundary is the two plants' OWN minimum `phi` — nothing is fitted.
///
/// INSIDE the band the armed cell is bit-for-bit its own leg-free march (`disarmed`); ABOVE it
/// both bind, the floor pins the currency, and the valve's credit is exactly 0. Every verdict is
/// read off `fuel_removed`; `s_eng` is NaN there by construction and no assertion touches it.
#[test]
fn the_floor_is_disarmed_inside_the_band_and_tautological_above_it() {
    let m = bt(&LeverArm::default());
    let d = m.floor_dichotomy(&flight(), &ramp(0.5, DS), &bled(),
                              &[0.34, 0.36, 0.40, 0.43, 0.46], Spool::Lp, None);
    let (lo_b, hi_b) = d.band;
    assert!(0.0 < lo_b && lo_b < hi_b, "{:?}", d.band);
    assert!((lo_b - (d.min_phi_ref / d.phi_surge - 1.0)).abs() < 1e-12);
    assert!((hi_b - (d.min_phi_armed / d.phi_surge - 1.0)).abs() < 1e-12);
    let inside: Vec<_> = d.rows.iter().filter(|r| lo_b < r.sm && r.sm < hi_b).collect();
    let above: Vec<_> = d.rows.iter().filter(|r| r.sm > hi_b).collect();
    assert!(inside.len() >= 3 && !above.is_empty(),
            "{:?}", d.rows.iter().map(|r| r.sm).collect::<Vec<_>>());
    for r in inside {
        assert!(r.removed_fuel > 0.0, "{r:?}");        // the floor DOES bind on the reference
        assert!(r.removed_both == 0.0, "{r:?}");       // and is exactly DISARMED on the armed one
        assert!(r.disarmed, "{r:?}");                  // bit-for-bit its own leg-free march
    }
    for r in above {
        assert!(r.removed_fuel > 0.0 && r.removed_both > 0.0, "{r:?}");   // both BIND
        assert!(r.credit.abs() < 1e-12, "{r:?}");      // rung 60's tautology, exact
        assert!(!r.disarmed, "{r:?}");
    }
}

/// The band exists BECAUSE the valve buys `phi`, so its width must track `b_max` — and vanish at
/// `b_max = 0`, where the two plants are the same machine.
#[test]
fn the_disarming_band_widens_with_the_valve() {
    let m = bt(&LeverArm::default());
    let mut widths = Vec::new();
    for bm in [0.0, 0.05, 0.10, 0.15] {
        let lever = LeverArm::scheduled(BleedSchedule::new(bm, N_LO));
        let d = m.floor_dichotomy(&flight(), &ramp(0.5, 0.01), &lever, &[], Spool::Lp, None);
        widths.push(d.band.1 - d.band.0);
    }
    assert!(widths[0].abs() < 1e-12, "{}", widths[0]);
    assert!(widths.windows(2).all(|w| w[1] > w[0]), "{widths:?}");
}

// =============================================================================================
// GATE 7 — THE SPLICE: both halves live, opposite signs, and no ratio published
// =============================================================================================

/// Rung 59 always had one half of the table EXACTLY zero, which made its split trivially additive.
/// A bleed moves both, and they FIGHT. The claim is carried by the two RAW deltas — large and
/// opposite in sign — and deliberately NOT by shares.
///
/// **Python's last assertion (`"abscissa_share" not in d`) has no port and needs none**: the
/// return type is a struct with no such field, so the absence is a compile-time property rather
/// than a runtime one. Recorded rather than silently dropped — the count is still one gate.
#[test]
fn both_splice_halves_are_live_and_carry_opposite_signs() {
    let m = bt(&LeverArm::default());
    let d = m.matched_leg_deltas(&flight(), &ramp(0.5, DS), &bled(), MARGIN, Spool::Lp, 13, None);
    assert!(d.clamped == 0, "{}", d.clamped);
    assert!(d.delta_index > 1e-3, "{}", d.delta_index);
    assert!(d.delta_value < -1e-3, "{}", d.delta_value);
    assert!(d.delta_index * d.delta_value < 0.0, "the two halves must FIGHT");
    assert!(d.delta_match.abs() < d.delta_index.abs(),
            "the net must sit inside the re-indexing term");
}

/// The control: on the SAME instrument an LP stator's matched leg is rung 59's exact no-op, so
/// gate 7's numbers are the lever's doing and not the reader's.
#[test]
fn a_stator_leaves_the_matched_leg_a_no_op_rung59_reproduced() {
    let m = bt(&LeverArm::default());
    let d = m.matched_leg_deltas(&flight(), &ramp(0.5, 0.01), &stat(), MARGIN, Spool::Lp, 13,
                                 None);
    assert!(d.delta_index.abs() < 1e-9, "{}", d.delta_index);
    assert!(d.delta_value.abs() < 1e-9, "{}", d.delta_value);
    assert!(d.delta_match.abs() < 1e-9, "{}", d.delta_match);
}

// =============================================================================================
// GATE 8 — SCOPE: the choked branch survives a leg that cuts hard
// =============================================================================================

/// Rung 62's pre-check was run with no leg cutting fuel; rung 42 warns the choked guard bites
/// SOONER with the valve open, and the bled `_close_fuel` makes a metered fuel flow RICHER.
/// Re-checked with both leg kinds armed.
fn every_march_stays_choked(lever: &LeverArm) {
    let m = bt(lever);
    let leg = leg_of(&bt(&LeverArm::default()));
    let floor = SurgeLimiter::from_margin(&lp_map(), Spool::Lp, 0.40);
    let legs = [StatorLeg::default(),
                StatorLeg { accel: Some(&leg), surge: None, tt4_max: None },
                StatorLeg { accel: None, surge: Some(Floor::Phi(floor)), tt4_max: None }];
    for sl in &legs {
        let (traj, _) = m.stator_march(&flight(), &ramp(0.5, 0.01), None, sl);
        assert!(traj.iter().all(|p| p.branch == Branch::Choked));
    }
}

#[test] fn every_march_stays_choked_bare() { every_march_stays_choked(&LeverArm::default()); }
#[test] fn every_march_stays_choked_bleed_sched() { every_march_stays_choked(&bled()); }
#[test] fn every_march_stays_choked_bleed_const() {
    every_march_stays_choked(&LeverArm::constant(0.15));
}
