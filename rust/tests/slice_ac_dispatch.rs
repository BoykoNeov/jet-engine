//! SLICE AC step 7 — **THE FIVE SWAPS, EACH REPLACED BY ITS PARENT'S FUNCTION POINTER.**
//!
//! `slice_ac_oracle.rs` is green at **5 351 keys per arm** against two interpreters, and
//! `rung70.rs` / `rung71.rs` carry 57 ported gates between them. All of those are value
//! instruments, and **no value key can witness a hook table**. Slice AC's whole risk is a swap
//! whose Rust body is still effectively the parent's — which compiles, runs, and is caught by
//! nothing the ladder does automatically, because § 5.27 (i) measured that this slice adds **no
//! cell at all**: there is no first gate for a forgotten cell to fail.
//!
//! So the injection is the parent's own function pointer, as at slice AB, and for the same
//! reason: **the risk IS the parent.**
//!
//! # THE LAUNDERING, WHICH DECIDES WHERE EACH SWAP CAN BE SCORED AT ALL
//!
//! Slice AB recorded the sibling-constructor trap; this slice hits a **stronger** form of it and
//! it is the leading finding of this step.
//!
//! Every rung-70/71 reader begins by calling [`split_rig`] or [`full_rig`], and both of those call
//! `core.at_lever(…)`, whose shipped body routes through the cascade builder — which installs the
//! **shipped** `&R70_TRIPLE` / `&R70_FUEL`. So an injection into a core's `TripleHooks` or
//! `FuelTransientHooks` table is **laundered before any reader reads anything**:
//! [`the_rig_launders_a_triple_injection`] measures that, and it comes back bit-identical.
//!
//! **THIS IS AN ARCHITECTURAL DIFFERENCE FROM PYTHON AND NOT A PORT DEFECT.** In Python
//! `_triple_laws` is a METHOD ON THE CLASS, so a monkeypatch survives the rebuild — the sibling is
//! the same class. In Rust the sibling is a table pointer chosen by the builder. § 5.27 (v)'s
//! *"seen by 1 of 6 readers"* is therefore a **Python** measurement that does not transfer, and a
//! gate that scored `triple_laws` through `split_gains` on an injected core would report
//! UNOBSERVABLE for a reason about the builder rather than about the cell — the ordering artifact
//! § (v) already booked once for `at_lever`.
//!
//! Each swap is therefore scored where it is actually dispatched:
//!
//! | swap | scored on | why not the obvious reader |
//! |---|---|---|
//! | `r70.at_lever` | [`split_gains`], which dispatches it | — it IS the cell the rig calls |
//! | `r70.integrate_fuel` | a DIRECT march on the injected machine | a rig would launder it |
//! | `r70.triple_laws` | [`split_gains`] **through a declared CARRIER** | a rig would launder it |
//! | `r71.at_lever` | [`full_gains`], which dispatches it | — it IS the cell the rig calls |
//! | `r71.integrate_fuel` | a DIRECT march on the injected machine | a rig would launder it |
//!
//! # THE CARRIER IS DECLARED, AND IT HAS ITS OWN CONTROL
//!
//! [`R70_CARRIER`] is an `at_lever` that rebuilds through [`ScheduledStatorTransient::with_ref_tables`]
//! and **re-installs whichever triple table it was built around**. It is the Rust stand-in for
//! Python's class-wide patch, not part of the injection — and
//! [`the_carrier_alone_is_silent`] runs the carrier with the **shipped** triple table and
//! requires the reading to come back bit-identical to the shipped one, so the delta gate 3
//! measures is the triple cell and nothing else.
//!
//! # THE SHAPES ARE DERIVED, NEVER ANNOTATED
//!
//! Slice AB's doc records its first draft carrying a `shape` string per row and counting the
//! strings: *"the labels were mine, and nothing measured whether a break was a panic."* Here each
//! swap names ONE exercise, that exercise runs **twice** — shipped tables, then injected — and
//! [`Shape`] falls out of the pair. The control is the same machinery with the shipped tables on
//! BOTH sides: all five must come back [`Shape::Silent`].
//!
//! **AND THE PANIC MESSAGE IS THE EVIDENCE, NOT THE PANIC.** Three of these swaps are observable
//! only because the PARENT's `integrate_fuel` refuses this rung's arming, and the refusal names
//! the rung it came from. A bare "it panicked" would pass on any panic and could not tell a
//! wrong-class construction from an unrelated abort, so every panic gate asserts the message
//! identifies the rung whose table got built.
//!
//! # WHERE § 5.27 (v)'s PYTHON SHAPES DISAGREE WITH THE MEASURED RUST ONES
//!
//! Pre-registered before the run, on slice AB's precedent — AB predicted four panic-shaped cells
//! and **two broke by value instead**. **The measured Rust shape wins**, and any divergence is
//! recorded as a finding rather than quietly re-gated. § (v)'s `at_lever` rows read
//! *"PANIC + VALUE"* because a Python swap is observable in two ways at once; here the value half
//! is unreachable for the laundering reason above, so both are scored on the panic.
//!
//! [`split_rig`]: turbojet::cross_split::split_rig
//! [`full_rig`]: turbojet::full_split::full_rig
//! [`ScheduledStatorTransient::with_ref_tables`]: turbojet::stator_transient::ScheduledStatorTransient

use std::panic::{catch_unwind, AssertUnwindSafe};

use turbojet::bleed_transient::{LeverArm, LeverArming, LeverHooks};
use turbojet::cross_split::{split_gains, R70, R70_FUEL, R70_STATOR, R70_TRIPLE, R70_TWO};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{AccelSchedule, FuelTransientHooks};
use turbojet::full_split::{full_gains, R71, R71_FUEL, R71_STATOR, R71_TRIPLE, R71_TWO};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::reference_split::{StatorIncidenceLimiter, R69, R69_TRIPLE};
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::three_loop::{StatorLimiter, TripleHooks, R68_FUEL};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------------- the grid
//
// **COPIED FROM `tests/rung70.rs`, WHICH IS THE SHIPPED FIXTURE'S OWN GRID.** § 5.27.6 (a) is the
// reason this is spelled rather than remembered: the pre-flight's § (ii) table was measured at
// `every = 40` where the fixture passes `every = 10`, and every stride-dependent number in it is
// therefore a reading of a sample the suite never takes. A baseline copied from that table would
// make five deltas wrong and all five still pass.

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
const B: f64 = 0.10;
const PHI: f64 = 0.80;
const V_MAX: f64 = 0.20;
/// The EXPRESSION, never a typed decimal — the three floors being ONE PHYSICAL WALL is what makes
/// `pair_CV = 1` a measurement, and a rounded constant would break it silently.
const SM: f64 = PHI / FLOOR - 1.0;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TAU_GOV: f64 = 0.05;
const TT4_MAX: f64 = 1200.0;
/// **THE FIXTURE'S OWN STRIDE.** `tests/test_rung70.py:114` and `tests/rung70.rs`'s `gains()` both
/// pass 10; § 5.27 (ii)'s table was taken at 40. See § 5.27.6 (a).
const EVERY: usize = 10;

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn hp() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn valve() -> BleedLimiter { BleedLimiter::from_margin_tau(&lp(), B, SM, Some(TAU)) }

fn phi_stator() -> StatorLimiter { StatorLimiter::from_margin(&lp(), V_MAX, SM, Some(TAU_S)) }

fn inc() -> StatorIncidenceLimiter {
    StatorIncidenceLimiter::from_margin(&lp(), V_MAX, SM, Some(TAU_S))
}

/// Rung 70's arming — a `phi` stator beside the valve, which is what `_cross` carries.
fn cross_arm() -> LeverArm {
    LeverArm { bleed_lim: Some(valve()), stator_lim: Some(phi_stator()), ..Default::default() }
}

/// Rung 71's arming — an INCIDENCE stator beside the valve, which is what `_full` carries and
/// what rung 70's guard A refuses.
fn full_arm() -> LeverArm {
    LeverArm { bleed_lim: Some(valve()), stator_inc: Some(inc()), ..Default::default() }
}

// ---------------------------------------------------------------- machines with CHOSEN tables
//
// **NOT through `build_cross_split_cascade` / `build_full_split_cascade`**, which hardcode the
// shipped tables — the whole point here is to install a table they would never install. The
// constructor guards those builders assert are not re-spelled: `slice_ac_cells.rs` is where the
// guards themselves are gated, and both armings below are ones they admit.

#[allow(clippy::too_many_arguments)]
fn build(
    arm: &LeverArm, lever: &'static LeverHooks, fuel: &'static FuelTransientHooks,
    tri: &'static TripleHooks,
) -> ScheduledStatorCore {
    match ScheduledStatorTransient::with_ref_tables(
        design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm.stator,
        &R70_TWO, &R70_STATOR, fuel, lever,
        LeverArming { bleed: arm.bleed, sched: arm.bleed_sched, lim: arm.bleed_lim },
        tri, arm.stator_lim, arm.stator_inc)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("neither arming disables LP"),
    }
}

/// A rung-70 machine, tables chosen.
fn cross(lever: &'static LeverHooks, fuel: &'static FuelTransientHooks,
         tri: &'static TripleHooks) -> ScheduledStatorCore {
    build(&cross_arm(), lever, fuel, tri)
}

/// A rung-71 machine, tables chosen. `R71_TWO`/`R71_STATOR` are aliases of rung 70's — measured,
/// see [`the_four_alias_tables_are_the_same_pointer_as_rung_70s`].
fn full(lever: &'static LeverHooks, fuel: &'static FuelTransientHooks,
        tri: &'static TripleHooks) -> ScheduledStatorCore {
    build(&full_arm(), lever, fuel, tri)
}

// ------------------------------------------------------------------------ the five injections

/// SWAP 1 — rung 70's `at_lever` replaced by **rung 69's**.
const INJ_R70_LEVER: LeverHooks = LeverHooks { at_lever: R69.at_lever, ..R70 };

/// SWAP 2 — rung 70's `integrate_fuel` replaced by **rung 68's**, which is the body `R69_FUEL`
/// aliases (`R69_FUEL = R68_FUEL`, measured at step 7 and the reason this names rung 68 directly).
const INJ_R70_FUEL: FuelTransientHooks =
    FuelTransientHooks { integrate_fuel: R68_FUEL.integrate_fuel, ..R70_FUEL };

/// SWAP 3 — rung 70's `triple_laws` replaced by the one it overrides. `R69_TRIPLE.triple_laws` is
/// spelled `R68_TRIPLE.triple_laws` at its own definition; naming rung 69's is what makes this the
/// PARENT pointer rather than a hand-picked ancestor.
const INJ_R70_TRIPLE: TripleHooks =
    TripleHooks { triple_laws: R69_TRIPLE.triple_laws, ..R70_TRIPLE };

/// SWAP 4 — rung 71's `at_lever` replaced by **rung 70's**.
const INJ_R71_LEVER: LeverHooks = LeverHooks { at_lever: R70.at_lever, ..R71 };

/// SWAP 5 — rung 71's `integrate_fuel` replaced by **rung 70's**.
const INJ_R71_FUEL: FuelTransientHooks =
    FuelTransientHooks { integrate_fuel: R70_FUEL.integrate_fuel, ..R71_FUEL };

// ---------------------------------------------------------------------------- THE CARRIER
//
// See the module header. This is NOT an injection; it is the Rust stand-in for Python's
// class-wide patch, and `the_carrier_alone_is_silent` is what keeps it out of the delta.

fn carrier_shipped(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    carry(core, arm, &R70_CARRIER, &R70_TRIPLE)
}

fn carrier_injected(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    carry(core, arm, &R70_CARRIER_INJ, &INJ_R70_TRIPLE)
}

/// Rebuild on the receiver's own design references, keeping BOTH the carrier lever table and the
/// chosen triple table — so a nested `at_lever` inside a reader does not launder them either.
fn carry(
    core: &ScheduledStatorCore, arm: &LeverArm, lever: &'static LeverHooks,
    tri: &'static TripleHooks,
) -> ScheduledStatorCore {
    match ScheduledStatorTransient::with_ref_tables(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(),
        arm.stator, &R70_TWO, &R70_STATOR, &R70_FUEL, lever,
        LeverArming { bleed: arm.bleed, sched: arm.bleed_sched, lim: arm.bleed_lim },
        tri, arm.stator_lim, arm.stator_inc)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("the carrier never disables LP"),
    }
}

const R70_CARRIER: LeverHooks = LeverHooks { at_lever: carrier_shipped, ..R70 };
const R70_CARRIER_INJ: LeverHooks = LeverHooks { at_lever: carrier_injected, ..R70 };

// ------------------------------------------------------------------------- shapes, DERIVED

/// What one injection did, computed from a shipped/injected PAIR — never labelled by hand.
#[derive(Clone, Debug, PartialEq)]
enum Shape {
    /// The two readings are bit-identical. For an injection this is the failure: the cell is not
    /// dispatched, or the parent's body is indistinguishable here.
    Silent,
    /// Both sides returned, and the numbers differ.
    Value,
    /// The injected side aborted. The string is the panic message, which is the evidence of WHICH
    /// table set got built.
    Panic(String),
}

/// Run one closure, capturing a panic message instead of unwinding out of the test.
fn outcome<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let got = catch_unwind(AssertUnwindSafe(f));
    std::panic::set_hook(prev);
    got.map_err(|e| {
        e.downcast_ref::<String>().cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_string()))
            .unwrap_or_else(|| "<non-string panic>".to_string())
    })
}

/// The shape of one shipped/injected pair. **The shipped side must not abort** — if it does, the
/// exercise is broken rather than the cell, and saying so here stops that reading as a break.
fn shape<T: PartialEq + std::fmt::Debug>(
    shipped: impl FnOnce() -> T, injected: impl FnOnce() -> T,
) -> Shape {
    let a = outcome(shipped).expect("the SHIPPED side of an exercise must not abort");
    match outcome(injected) {
        Err(msg) => Shape::Panic(msg),
        Ok(b) if b == a => Shape::Silent,
        Ok(_) => Shape::Value,
    }
}

// ------------------------------------------------------------------------------- the readings

/// A `split_gains` reading, as EXACT BITS.
///
/// **THE COUNTS COME FIRST AND THEY ARE THE POINT.** § 5.27 (ii)'s registered break shape is an
/// EMPTY SAMPLE: every value key agrees between two empty tables, and the reader still returns
/// normally. § 5.27.6 (e) recorded the same blindness three ways in the ported gates, all three
/// because their bars were one-sided.
#[derive(Clone, Debug, PartialEq)]
struct Gains {
    n_riding: usize,
    n_sampled: usize,
    n_rows: usize,
    n_skipped: usize,
    max_pair_gap: Option<u64>,
    worst_cv: Option<u64>,
    pair_rc: Vec<u64>,
}

fn gains_of(m: &ScheduledStatorCore) -> Gains {
    let g = split_gains(m, &flight(), LO, HI, TT4_MAX, SM, R, SETTLE, DS, TAU, TAU_GOV, TAU_S,
                        V_MAX, EVERY);
    Gains {
        n_riding: g.n_riding,
        n_sampled: g.n_sampled,
        n_rows: g.rows.len(),
        n_skipped: g.skipped.len(),
        max_pair_gap: g.max_pair_gap.map(f64::to_bits),
        worst_cv: g.worst_cv.map(f64::to_bits),
        pair_rc: g.pair_rc.iter().map(|x| x.to_bits()).collect(),
    }
}

/// A `full_gains` reading, same shape and for the same reason.
#[derive(Clone, Debug, PartialEq)]
struct FullReading {
    n_rows: usize,
    n_skipped: usize,
    fingerprint: Vec<u64>,
}

fn full_of(m: &ScheduledStatorCore) -> FullReading {
    let g = full_gains(m, &flight(), LO, HI, TT4_MAX, SM, R, SETTLE, DS, TAU, TAU_GOV, TAU_S,
                       V_MAX, EVERY);
    FullReading {
        n_rows: g.rows.len(),
        n_skipped: g.skipped.len(),
        fingerprint: g.rows.iter().map(|r| r.s.to_bits()).collect(),
    }
}

/// A DIRECT march on the machine handed in — **no `*_rig`, so nothing is rebuilt and nothing is
/// laundered.** This is where the two `integrate_fuel` swaps are dispatched.
fn march_of(m: &ScheduledStatorCore) -> Vec<u64> {
    let leg = StatorLeg { accel: None::<&AccelSchedule>, surge: None, tt4_max: Some(TT4_MAX) };
    let ramp = Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds: DS };
    let (traj, _) = m.stator_march_scoped(
        &flight(), &ramp, None, &leg,
        &MarchScope { tau_gov: Some(TAU_GOV), ..MarchScope::DEFAULT });
    let mut out = vec![traj.len() as u64];
    out.extend(traj.iter().map(|p| p.nu_lp.to_bits()));
    out
}

// -------------------------------------------------------------------------------- the exercises
//
// One per swap, each a closure PAIR over the same machinery with one table changed.

fn swap1() -> Shape {
    shape(|| gains_of(&cross(&R70, &R70_FUEL, &R70_TRIPLE)),
          || gains_of(&cross(&INJ_R70_LEVER, &R70_FUEL, &R70_TRIPLE)))
}

fn swap2() -> Shape {
    shape(|| march_of(&cross(&R70, &R70_FUEL, &R70_TRIPLE)),
          || march_of(&cross(&R70, &INJ_R70_FUEL, &R70_TRIPLE)))
}

fn swap3() -> Shape {
    shape(|| gains_of(&cross(&R70_CARRIER, &R70_FUEL, &R70_TRIPLE)),
          || gains_of(&cross(&R70_CARRIER_INJ, &R70_FUEL, &INJ_R70_TRIPLE)))
}

fn swap4() -> Shape {
    shape(|| full_of(&full(&R71, &R71_FUEL, &R71_TRIPLE)),
          || full_of(&full(&INJ_R71_LEVER, &R71_FUEL, &R71_TRIPLE)))
}

fn swap5() -> Shape {
    shape(|| march_of(&full(&R71, &R71_FUEL, &R71_TRIPLE)),
          || march_of(&full(&R71, &INJ_R71_FUEL, &R71_TRIPLE)))
}

/// The same five with the SHIPPED tables on BOTH sides. Every one must be [`Shape::Silent`], which
/// is what makes a `Silent` above a finding about the cell rather than about the machinery.
fn controls() -> Vec<Shape> {
    vec![
        shape(|| gains_of(&cross(&R70, &R70_FUEL, &R70_TRIPLE)),
              || gains_of(&cross(&R70, &R70_FUEL, &R70_TRIPLE))),
        shape(|| march_of(&cross(&R70, &R70_FUEL, &R70_TRIPLE)),
              || march_of(&cross(&R70, &R70_FUEL, &R70_TRIPLE))),
        shape(|| gains_of(&cross(&R70_CARRIER, &R70_FUEL, &R70_TRIPLE)),
              || gains_of(&cross(&R70_CARRIER, &R70_FUEL, &R70_TRIPLE))),
        shape(|| full_of(&full(&R71, &R71_FUEL, &R71_TRIPLE)),
              || full_of(&full(&R71, &R71_FUEL, &R71_TRIPLE))),
        shape(|| march_of(&full(&R71, &R71_FUEL, &R71_TRIPLE)),
              || march_of(&full(&R71, &R71_FUEL, &R71_TRIPLE))),
    ]
}

// ============================================================================== THE FIVE GATES

/// SWAP 1 — `r70.at_lever` ← rung 69's.
///
/// Rung 69's body routes through `build_reference_split_cascade`, so [`split_rig`]'s sibling comes
/// back carrying `R69_FUEL` — which is rung 68's `integrate_fuel` — and the march under `tau_gov`
/// is then **refused by rung 68**. The message is the evidence: it names the rung whose table got
/// built, which a bare `is_err()` could not.
///
/// [`split_rig`]: turbojet::cross_split::split_rig
#[test]
fn swap_1_r70_at_lever_is_reached_and_the_message_names_rung_68() {
    match swap1() {
        Shape::Panic(m) => assert!(
            m.contains("rung-68 is THREE LOOPS ON ONE VARIABLE"),
            "the sibling must be RUNG 69's, whose inherited march refuses this arming; got: {m}"),
        other => panic!("`r70.at_lever` came back {other:?} — the cell is not dispatched"),
    }
}

/// SWAP 2 — `r70.integrate_fuel` ← rung 68's, scored on a DIRECT march.
///
/// Rung 68's first guard is `assert tau_gov is None`, and a rung-70 march always carries one, so
/// the parent pointer refuses the march it was handed. Scored without a `*_rig` for the module
/// header's reason: a rig would rebuild through `at_lever` and install `R70_FUEL` again.
#[test]
fn swap_2_r70_integrate_fuel_is_reached_and_the_message_names_rung_68() {
    match swap2() {
        Shape::Panic(m) => assert!(
            m.contains("rung-68 is THREE LOOPS ON ONE VARIABLE"),
            "the parent pointer must refuse `tau_gov`; got: {m}"),
        other => panic!("`r70.integrate_fuel` came back {other:?} — the cell is not dispatched"),
    }
}

/// SWAP 3 — `r70.triple_laws` ← rung 69's (which is rung 68's), **AND IT BREAKS BY EMPTYING THE
/// SAMPLE**: § 5.27 (ii)'s registered shape, reproduced.
///
/// # BOTH ENDPOINTS ARE ASSERTED EXACTLY, AND THE COUNT IS **7**, NOT 2
///
/// § 5.27.6 (a) measured the pre-flight's row at `every = 40`; the fixture passes 10 and the
/// sample is **seven** points. Written as two `assert_eq!` and never as
/// `assert!(injected < shipped)` — a one-sided bar passes at every stride and on every empty
/// table, which is the exact blindness § 5.27.6 (e) recorded three times over in the ported gates.
///
/// The skipped count is asserted as the MIRROR (0 → 7): the rows do not vanish, they are all
/// declared off-regime, and a reader that lost them silently would show 0 → 0.
#[test]
fn swap_3_r70_triple_laws_empties_the_sample_seven_rows_to_zero() {
    let shipped = gains_of(&cross(&R70_CARRIER, &R70_FUEL, &R70_TRIPLE));
    let injected = gains_of(&cross(&R70_CARRIER_INJ, &R70_FUEL, &INJ_R70_TRIPLE));

    assert_eq!(shipped.n_riding, 61, "the shipped ride, at the FIXTURE's grid");
    assert_eq!(shipped.n_sampled, 7, "`every = 10` samples SEVEN of the 61 riding points");
    assert_eq!(shipped.n_rows, 7, "and all seven are interior");
    assert_eq!(shipped.n_skipped, 0);

    assert_eq!(injected.n_riding, 61, "the RIDE is unchanged — the break is downstream of it");
    assert_eq!(injected.n_sampled, 7, "and so is the SAMPLE — the same seven points are visited");
    assert_eq!(injected.n_rows, 0, "**every row is dropped**: rung 68's laws are off-regime here");
    assert_eq!(injected.n_skipped, 7, "and each one is DISCLOSED, not silently truncated");

    assert_eq!(injected.max_pair_gap, None,
               "every aggregate is `None` on an empty table — which is why the COUNTS carry this \
                gate and no value key can");
    assert_eq!(injected.worst_cv, None);
    assert!(injected.pair_rc.is_empty());
}

/// SWAP 4 — `r71.at_lever` ← rung 70's.
///
/// [`full_rig`] arms an INCIDENCE stator, so rung 70's sibling meets rung 70's guard A — the one
/// refusal that exists precisely because rung 71 is the cell rung 70 asserted against.
///
/// [`full_rig`]: turbojet::full_split::full_rig
#[test]
fn swap_4_r71_at_lever_is_reached_and_the_message_names_rung_70() {
    match swap4() {
        Shape::Panic(m) => assert!(
            m.contains("rung-70 is THREE loops on TWO variables"),
            "the sibling must be RUNG 70's, whose guard A refuses an incidence stator; got: {m}"),
        other => panic!("`r71.at_lever` came back {other:?} — the cell is not dispatched"),
    }
}

/// SWAP 5 — `r71.integrate_fuel` ← rung 70's, scored on a DIRECT march.
#[test]
fn swap_5_r71_integrate_fuel_is_reached_and_the_message_names_rung_70() {
    match swap5() {
        Shape::Panic(m) => assert!(
            m.contains("rung-70 is THREE loops on TWO variables"),
            "the parent pointer must refuse the incidence arming; got: {m}"),
        other => panic!("`r71.integrate_fuel` came back {other:?} — the cell is not dispatched"),
    }
}

// ================================================================== THE LAUNDERING, AND ITS CONTROL

/// **THE LEADING FINDING OF THIS STEP, MEASURED RATHER THAN ASSERTED IN PROSE.**
///
/// A `TripleHooks` injection into a CORE is invisible to `split_gains`, because the reader's first
/// act is [`split_rig`] → `core.at_lever(…)` → the cascade builder → the **shipped** table. The
/// two readings must be bit-identical, and the shipped one must be non-trivial (seven rows), so
/// this cannot pass by both sides being empty.
///
/// This is why gate 3 needs the carrier, and why § 5.27 (v)'s Python *"seen by 1 of 6 readers"*
/// does not transfer to the port.
///
/// [`split_rig`]: turbojet::cross_split::split_rig
#[test]
fn the_rig_launders_a_triple_injection() {
    let shipped = gains_of(&cross(&R70, &R70_FUEL, &R70_TRIPLE));
    let laundered = gains_of(&cross(&R70, &R70_FUEL, &INJ_R70_TRIPLE));
    assert_eq!(shipped.n_rows, 7, "the shipped side must be non-trivial for this to mean anything");
    assert_eq!(shipped, laundered,
               "a triple injection into the core survived the rig — if this ever fails, gate 3's \
                carrier is no longer needed and the module header is wrong");
}

/// The carrier's own control: with the **shipped** triple table it must reproduce the shipped
/// reading exactly. Without this, gate 3's delta could be the carrier rather than the cell.
#[test]
fn the_carrier_alone_is_silent() {
    let shipped = gains_of(&cross(&R70, &R70_FUEL, &R70_TRIPLE));
    let carried = gains_of(&cross(&R70_CARRIER, &R70_FUEL, &R70_TRIPLE));
    assert_eq!(shipped, carried,
               "the carrier rebuilds through a different constructor and must be OBSERVATIONALLY \
                identical; it is a stand-in for Python's class-wide patch, not an injection");
}

// ======================================================================== THE TALLY AND THE LEDGER

/// **P2 FOR THIS SLICE, SETTLED PER SWAP AND EMITTED.**
///
/// The count is computed from re-running the five injections — never typed. This phase has been
/// caught repeatedly on a tally written beside the addends that disprove it, most recently at
/// § 5.27.6 (g) where a constant was typed at `39_099` against a measured 5 351.
#[test]
fn all_five_swaps_are_observable_and_the_five_controls_are_silent() {
    let shapes = vec![
        ("r70.at_lever", swap1()),
        ("r70.integrate_fuel", swap2()),
        ("r70.triple_laws", swap3()),
        ("r71.at_lever", swap4()),
        ("r71.integrate_fuel", swap5()),
    ];

    let silent: Vec<&str> =
        shapes.iter().filter(|(_, s)| *s == Shape::Silent).map(|(n, _)| *n).collect();
    assert!(silent.is_empty(), "these swaps are NOT observable: {silent:?}");
    assert_eq!(shapes.len(), 5, "§ 5.27 (iii) counted five distinct function pointers");

    // The SHAPES, derived from the pairs above and reported so a change in one is visible.
    let panics = shapes.iter().filter(|(_, s)| matches!(s, Shape::Panic(_))).count();
    let values = shapes.iter().filter(|(_, s)| *s == Shape::Value).count();
    assert_eq!(panics + values, 5, "every swap breaks one way or the other");
    assert_eq!(
        (panics, values), (4, 1),
        "MEASURED at step 7: the four table-identity swaps break by PANIC (the parent's march \
         refuses this rung's arming and the message names the rung), and `triple_laws` breaks by \
         VALUE — an EMPTY SAMPLE, § 5.27 (ii). § (v)'s Python column reads `PANIC + VALUE` for the \
         two `at_lever` rows because a class-wide patch is observable both ways; here the value \
         half is unreachable for the laundering reason in this file's header. shapes: {shapes:?}");

    for (n, s) in controls().iter().enumerate() {
        assert_eq!(*s, Shape::Silent, "control {n} was not silent: {s:?}");
    }
}

/// **THE STEP's OWN FINDING ABOUT THE TABLES, TURNED INTO A TRIPWIRE.**
///
/// `R70_TWO`, `R70_STATOR`, `R71_TWO` and `R71_STATOR` are whole-const ALIASES of their parents,
/// and their doc comments said they were *named rather than reached through a `..` spread* so that
/// the NEXT addition to those tables would not be silent here. **Measured at step 7 by adding a
/// probe field to each of the five hook structs and reading which sites raise `E0063`: an alias is
/// exactly as silent as a spread.** Only the five `TripleHooks` consts, which spell all ten fields
/// out, are loud — and `R66_TWO`, the precedent all four comments cite, is itself an alias.
///
/// The comments are corrected in the same pass; this is what makes the claim true rather than
/// prose. A new field on any of these four types now fails HERE.
#[test]
fn the_four_alias_tables_are_the_same_pointer_as_rung_70s() {
    // Widths, pinned by exhaustive destructuring: adding a field to any of these four types
    // stops compiling at this test.
    let turbojet::two_spool_transient::TwoSpoolTransientHooks {
        try_close: _, try_instant_tail: _, powers: _,
    } = R71_TWO;
    let turbojet::stator_transient::StatorTransientHooks {
        stator_march: _, v_of: _, arm: _, at_stator: _,
    } = R71_STATOR;

    // And the aliases really are the parents', field by field, so the four consts carry no
    // independent decision. **NOT  on the consts** — a reference to a  is a fresh
    // temporary each time it is written, so that comparison tests the optimiser and not the table
    // ([[rust-port-slice-aa-step1]] recorded it). Every cell is compared as a function ADDRESS,
    // and every cell of both types is listed, so a new field fails the destructuring above first.
    assert_eq!(R70_TWO.try_close as usize, R71_TWO.try_close as usize);
    assert_eq!(R70_TWO.try_instant_tail as usize, R71_TWO.try_instant_tail as usize);
    assert_eq!(R70_TWO.powers as usize, R71_TWO.powers as usize,
               "R71_TWO is rung 70's table, all three cells");
    assert_eq!(R70_STATOR.stator_march as usize, R71_STATOR.stator_march as usize);
    assert_eq!(R70_STATOR.v_of as usize, R71_STATOR.v_of as usize);
    assert_eq!(R70_STATOR.arm as usize, R71_STATOR.arm as usize);
    assert_eq!(R70_STATOR.at_stator as usize, R71_STATOR.at_stator as usize,
               "R71_STATOR is rung 70's table, all four cells");
}
