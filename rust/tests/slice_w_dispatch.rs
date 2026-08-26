//! SLICE W step 5 — **THE TWO GATES STEP 3 MEASURED TO BE MISSING, PLUS P2 AND P4.**
//!
//! Step 3 built P4's dispatch gate the way § 5.21 (v) pre-registered it and then measured that
//! the instrument P4 named **cannot see the defect it was written for**. Step 3's closing note
//! registered the two corrections so they could not be missed, and this file is them:
//!
//! * **the dispatch gate must count `b_of_calls`, not only the four reduced/bled pairs.** I2 and
//!   I2b move the call count `409 → 818` and leave all eight pairs untouched — the two spellings
//!   `b_of(nu_lp, tt2)` and `c.bleed.unwrap_or(0.0)` agree at EVERY call on this plant, not
//!   merely where `b` is 0, so the pairs never separate. [`b_of_is_the_only_counter_that_sees_a_reread`];
//! * **the carrier gate must build a `Floor::Incidence` cell.** `r57_try_surge_fuel` is a
//!   WRAPPER — it resolves the floor and delegates to `R43.try_surge_fuel` — and on a
//!   `Floor::Phi` that resolution is the IDENTITY. Both ported suites build only `Phi`, once, so
//!   "0 of 88 catch `..R43`" is a fact about the suites' INPUTS, not about the defect.
//!   [`the_r43_spread_is_invisible_on_phi_and_fatal_on_incidence`].
//!
//! # WHAT EACH OTHER INSTRUMENT ANSWERS, AND WHY NONE OF THEM ANSWERS THIS
//!
//! * the **88 ported gates** are RELATIONAL — step 3 measured five of six injections passing all
//!   88, two of them moving 312 and 151 gate-visible readings;
//! * `slice_w_smoke.rs` (step 2) is a STRUCTURAL check on a coarse grid, before the gates exist;
//! * `slice_w_oracle.rs` (step 4) DOES catch these, and its section K carries the same counts —
//!   but it is a **golden-comparison** gate, and a golden comparison is defeated by regenerating
//!   the golden against buggy code. This file manufactures each bug **inline, in Rust, on every
//!   run**. Nothing on disk can make it disappear.
//!
//! # ZERO SOURCE LINES
//!
//! Every injected table is built out of already-`pub` seams — [`R57`], [`R43`], [`R62_TWO`],
//! [`R62_FUEL`], [`R62`], [`with_tables`] and [`LeverArming`] — so `git diff -- rust/src/` is
//! empty for this step, as it was for step 4. In particular the `at_stator` injection needs no
//! new item at all: `R62_STATOR` **is** `{ at_stator: r62_at_stator, ..R57 }`, so the machine
//! with rung 62's override removed is the machine built on `R57` itself.
//!
//! # THE CLEAN SIDE IS PINNED AGAINST THE COMMITTED GOLDEN, BY KEY
//!
//! Every baseline reading is compared to `oracle/slice_w_pypy.tsv` by KEY, never against a
//! literal typed here. That is § 5.19 (xi)'s rule, and it also closes the obvious hole in a
//! manufactured gate: if the golden were ever regenerated against buggy code, the live clean
//! computation would stop matching it and this file fails — the one failure mode the oracle
//! alone has.
//!
//! # THE WORKLOAD IS `LO` AT `ds = 0.02`, AND THAT IS NOT A DETAIL
//!
//! Step 3's finding 5: § 5.21 (v)'s counts come off `equilibrium(flight, LO)` + one
//! `stator_march` at `ds = 0.02`. On a run at `Tt4 = 1200` the BARE machine's Newton takes three
//! closes fewer (62 against 65) while the scheduled one is unchanged, so a census on the wrong
//! throttle reproduces one row of the pre-registered table exactly and misses another by 3 —
//! which reads as a port defect and would open this gate red for the wrong reason.
//!
//! [`with_tables`]: turbojet::stator_transient::ScheduledStatorTransient::with_tables

use std::cell::Cell;
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

use turbojet::bleed_transient::{
    build_scheduled_bleed, counters, BleedSchedule, Census, LeverArm, LeverArming, R62, R62_FUEL,
    R62_TWO,
};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{Floor, FuelTransientHooks, SurgeLimiter, R43};
use turbojet::gas::{Abort, Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{
    CellRead, IncidenceLimiter, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorArm,
    StatorLeg, StatorSchedule, R57,
};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::{
    CloseState, Instant2, TwoSpoolTransientCore, TwoSpoolTransientHooks,
};

// ------------------------------------------------------------------------------- the golden
const ORACLE: &str = include_str!("../oracle/slice_w_pypy.tsv");

fn golden() -> BTreeMap<String, u64> {
    let mut m = BTreeMap::new();
    for line in ORACLE.lines() {
        if line.starts_with('#') { continue; }
        if let Some((k, v)) = line.split_once('\t') {
            if let Ok(u) = v.trim().parse::<u64>() { m.insert(k.to_string(), u); }
        }
    }
    assert!(m.len() > 9_000, "the committed slice-W golden did not parse ({} keys)", m.len());
    m
}

fn want_u(g: &BTreeMap<String, u64>, key: &str) -> u64 {
    *g.get(key).unwrap_or_else(|| panic!("no golden key {key}"))
}

// ------------------------------------------------------------------------------- the grid
// `slice_w_oracle.rs`'s grid verbatim — the same hardware, the same knee, the same ramp.
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const SETTLE: f64 = 1.2;
const N_LO: f64 = 0.65;
const V: f64 = 0.20;
const B: f64 = 0.10;
const MARGIN: f64 = 0.25;
const DS_62: f64 = 0.01;
const DS_63: f64 = 0.005;
const DS_CENSUS: f64 = 0.02;

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

/// `r_c` DERIVED — `0.4/1.4` is a different number and it drifts every reading in this file.
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

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn full(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("rungs 62-63 never disable LP"),
    }
}

/// The SHIPPED rung-62 machine.
fn clean(arm: &LeverArm) -> ScheduledStatorCore {
    full(build_scheduled_bleed(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

/// A rung-62 machine with ONE table swapped. Everything else is `build_scheduled_bleed`'s own
/// argument list, so the injected machine differs from the shipped one in exactly that table.
fn injected(
    arm: &LeverArm,
    two: &'static TwoSpoolTransientHooks,
    stator: &'static turbojet::stator_transient::StatorTransientHooks,
    fuel: &'static FuelTransientHooks,
) -> ScheduledStatorCore {
    full(ScheduledStatorTransient::with_tables(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm.stator,
        two, stator, fuel, &R62,
        LeverArming { bleed: arm.bleed, sched: arm.bleed_sched, lim: None }))
}

fn bsched() -> BleedSchedule { BleedSchedule::new(B, N_LO) }
fn vsched() -> StatorSchedule { StatorSchedule::new(V, N_LO) }
fn bleed_arm() -> LeverArm { LeverArm::scheduled(bsched()) }
fn stat_arm() -> LeverArm { LeverArm::stator(StatorArm::scheduled_lp(vsched())) }
fn const_arm() -> LeverArm { LeverArm::constant(B) }

fn both_arm() -> LeverArm {
    LeverArm { bleed_sched: Some(bsched()), stator: StatorArm::scheduled_lp(vsched()),
               ..Default::default() }
}

fn ramp(r: f64, ds: f64) -> Ramp {
    Ramp { tt4_lo: LO, tt4_hi: HI, r, s_settle: SETTLE, ds }
}

/// The text of a caught panic. **BOTH payload types, and that is not defensiveness.** Step 3
/// finding 1: `_isolating`'s two refusals unwind DIFFERENT types — an interpolated `assert!`
/// unwinds a `String`, a bare literal one a `&'static str` — and a gate downcasting only to
/// `String` read a CORRECTLY MATCHED refusal back as the empty string and failed on the very
/// message it was written to accept. `Floor::phi()`'s refusal, which this file's carrier gate
/// depends on, is the literal kind.
fn panic_text(e: Box<dyn std::any::Any + Send>) -> String {
    e.downcast_ref::<String>()
        .cloned()
        .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_string()))
        .unwrap_or_default()
}

/// Runs `f` with the panic hook silenced, returning the refusal text (or `None` if it did not
/// refuse). The hook is silenced so a deliberately refusing cell does not print a backtrace into
/// a passing run.
fn refusal(f: impl FnOnce()) -> Option<String> {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = catch_unwind(AssertUnwindSafe(f));
    std::panic::set_hook(prev);
    out.err().map(panic_text)
}

// =============================================================================================
// INJECTION 1 — `powers`/`try_instant_tail` RE-READ `b_of`
// =============================================================================================

thread_local! {
    /// Calls where the RE-READ predicate and the closure's OWN key selected the same branch.
    static AGREE: Cell<u64> = const { Cell::new(0) };
    /// Calls where they did NOT. **This is the counter that makes the gate mean something**: it
    /// is what would have to be non-zero for the eight reduced/bled pairs to be able to move.
    static DISAGREE: Cell<u64> = const { Cell::new(0) };
}

fn note(c: &CloseState, reread: f64) {
    let own = c.bleed.unwrap_or(0.0);
    if (reread == 0.0) == (own == 0.0) {
        AGREE.with(|x| x.set(x.get() + 1));
    } else {
        DISAGREE.with(|x| x.set(x.get() + 1));
    }
}

/// **THE INJECTED CELL — I2b, the "simplification" § 5.21 (v) predicted and P4 mis-instrumented.**
///
/// It re-reads `b_of(nu_lp, c.tt2)` where the shipped cell reads the closure's OWN `bleed` key,
/// then delegates. **The delegation is deliberate**: the shipped cell dispatches on its own key
/// internally, so re-implementing the branch here would double-bump the very pair counters this
/// gate asserts are still. What the injection actually costs is ONE EXTRA `b_of` CALL per
/// invocation — which is the whole observable footprint of the real defect on this plant — and
/// [`DISAGREE`] is what licenses that claim rather than asserting it: if the two predicates ever
/// selected different branches the pairs COULD move, and the gate says so out loud.
fn inj_powers(
    t: &TwoSpoolTransientCore, c: &CloseState, flight: &FlightCondition, nu_lp: f64, nu_hp: f64,
    tt4: f64,
) -> Result<(f64, f64), Abort> {
    note(c, t.b_of(nu_lp, Some(c.tt2)));
    (R62_TWO.powers)(t, c, flight, nu_lp, nu_hp, tt4)
}

fn inj_tail(
    t: &TwoSpoolTransientCore, flight: &FlightCondition, c: &CloseState, nu_lp: f64, nu_hp: f64,
    tt4: f64, v0: f64,
) -> Result<Instant2, Abort> {
    note(c, t.b_of(nu_lp, Some(c.tt2)));
    (R62_TWO.try_instant_tail)(t, flight, c, nu_lp, nu_hp, tt4, v0)
}

static INJ_TWO: TwoSpoolTransientHooks = TwoSpoolTransientHooks {
    try_close: R62_TWO.try_close,
    try_instant_tail: inj_tail,
    powers: inj_powers,
};

/// `R62_FUEL` spread from `..R43` instead of `..R57_FUEL` — step 3's I3. The ONE cell rung 62
/// overrides is kept; what the wrong spread silently drops is rung 60's floor-RESOLVING
/// `try_surge_fuel`.
static INJ_FUEL: FuelTransientHooks = FuelTransientHooks {
    try_close_fuel: R62_FUEL.try_close_fuel,
    ..R43
};

// =============================================================================================
// THE CENSUS HELPERS
// =============================================================================================
fn march_census(m: &ScheduledStatorCore) -> Census {
    counters::reset();
    m.fuel.inner.equilibrium(&flight(), LO);
    m.stator_march(&flight(), &ramp(0.5, DS_CENSUS), None, &StatorLeg::default());
    counters::take()
}

fn sibling_census(m: &ScheduledStatorCore) -> Census {
    counters::reset();
    m.loop_decomposition(&flight(), &ramp(0.5, DS_CENSUS), Spool::Lp);
    m.marginal_loop(&flight(), &ramp(0.5, DS_CENSUS), &bleed_arm(), None, Spool::Lp,
                    &StatorLeg::default());
    m.schedule_invariance(&flight(), LO, HI, MARGIN, 5);
    counters::take()
}

const PAIRS: [&str; 8] = ["close_reduced", "close_bled", "close_fuel_reduced",
                          "close_fuel_bled", "powers_reduced", "powers_bled", "tail_reduced",
                          "tail_bled"];
const ALL: [&str; 17] = ["close_reduced", "close_bled", "close_fuel_reduced",
                         "close_fuel_bled", "powers_reduced", "powers_bled", "tail_reduced",
                         "tail_bled", "b_of_calls", "b_of_constant", "b_of_sched_zero",
                         "b_of_sched_open", "at_lever_calls", "at_stator_r62",
                         "isolating_calls", "legs_calls", "legs_lever_bleed"];

fn field(n: &Census, k: &str) -> u64 {
    match k {
        "close_reduced" => n.close_reduced, "close_bled" => n.close_bled,
        "close_fuel_reduced" => n.close_fuel_reduced, "close_fuel_bled" => n.close_fuel_bled,
        "powers_reduced" => n.powers_reduced, "powers_bled" => n.powers_bled,
        "tail_reduced" => n.tail_reduced, "tail_bled" => n.tail_bled,
        "b_of_calls" => n.b_of_calls, "b_of_constant" => n.b_of_constant,
        "b_of_sched_zero" => n.b_of_sched_zero, "b_of_sched_open" => n.b_of_sched_open,
        "at_lever_calls" => n.at_lever_calls, "at_stator_r62" => n.at_stator_r62,
        "isolating_calls" => n.isolating_calls, "legs_calls" => n.legs_calls,
        "legs_lever_bleed" => n.legs_lever_bleed,
        _ => unreachable!("{k}"),
    }
}

// =============================================================================================
// GATE 1 — THE DISPATCH COUNTS, AGAINST PYTHON'S OWN MEASUREMENT
// =============================================================================================

/// § 5.21 (v)'s table, reproduced by the crate and checked **against the committed golden by
/// KEY**, not against numbers typed here. Seven censuses × seventeen counters.
///
/// It also asserts what step 3 measured about the census itself: `at_lever`, `at_stator`,
/// `isolating` and `legs` are ZERO on the march workload and NON-ZERO on the sibling one, so
/// their zeros are measured zeros rather than a dead instrument. That check is the reason the
/// second workload exists at all.
#[test]
fn the_dispatch_census_reproduces_pythons_own_counts() {
    let g = golden();
    let cases: [(&str, LeverArm, bool); 7] = [
        ("bare", LeverArm::default(), false),
        ("stator", stat_arm(), false),
        ("sched", bleed_arm(), false),
        ("const", const_arm(), false),
        ("both", both_arm(), false),
        ("sib_sched", bleed_arm(), true),
        ("sib_const", const_arm(), true),
    ];
    let mut bad = Vec::new();
    for (tag, arm, siblings) in cases {
        let m = clean(&arm);
        let n = if siblings { sibling_census(&m) } else { march_census(&m) };
        for k in ALL {
            let (got, want) = (field(&n, k), want_u(&g, &format!("K/{tag}/{k}")));
            if got != want {
                bad.push(format!("K/{tag}/{k}: rust {got} vs python {want}"));
            }
        }
        // The FIVE sibling counters, from both ends.
        for k in ["at_lever_calls", "at_stator_r62", "isolating_calls", "legs_calls",
                  "legs_lever_bleed"] {
            let v = field(&n, k);
            if siblings {
                assert!(v > 0, "{tag}: {k} is 0 on the SIBLING workload -- a counter no \
                                workload reaches cannot report an inert path");
            } else {
                assert_eq!(v, 0, "{tag}: {k} moved on the march workload, which builds no \
                                  sibling at all");
            }
        }
    }
    assert!(bad.is_empty(), "{} dispatch counts differ from Python:\n  {}", bad.len(),
            bad.join("\n  "));
    counters::reset();
}

// =============================================================================================
// GATE 2 — **P4, CORRECTED.** `b_of_calls` SEES THE RE-READ; THE EIGHT PAIRS DO NOT
// =============================================================================================

/// Step 3's finding 3, manufactured inline and asserted from both sides.
///
/// § 5.21 (v) predicted a `powers` re-reading `b_of` would be visible **only** to a dispatch gate
/// counting reduced-vs-bled per cell, and P4 registered that gate as the one slice W owes.
/// Measured: all eight of those counters DO NOT MOVE, and the only thing in the whole instrument
/// that does is the CALL COUNT of the re-read function. The verdict survived and its instrument
/// did not — so this gate asserts **both halves**, and without the second half it would be the
/// very instrument that measured nothing.
#[test]
fn b_of_is_the_only_counter_that_sees_a_reread() {
    for (tag, arm) in [("bare", LeverArm::default()), ("sched", bleed_arm()),
                       ("const", const_arm()), ("stator", stat_arm())] {
        let base = march_census(&clean(&arm));
        AGREE.with(|x| x.set(0));
        DISAGREE.with(|x| x.set(0));
        let inj = march_census(&injected(&arm, &INJ_TWO, &R57_STATOR_SHIPPED, &R62_FUEL));
        let (agree, disagree) = (AGREE.with(|x| x.get()), DISAGREE.with(|x| x.get()));

        // (a) THE EIGHT PAIRS ARE UNCHANGED — P4's named instrument, blind.
        for k in PAIRS {
            assert_eq!(field(&inj, k), field(&base, k),
                       "{tag}/{k}: the reduced/bled pair MOVED under the re-read. P4's \
                        instrument is not supposed to be able to see this defect -- if it \
                        now can, the plant changed and finding 3 needs re-measuring.");
        }
        // (b) AND IT IS BLIND BECAUSE THE TWO PREDICATES NEVER SEPARATE. This is finding 3's
        //     actual content, measured live rather than remembered.
        assert_eq!(disagree, 0,
                   "{tag}: the re-read and the closure's own key selected DIFFERENT branches \
                    {disagree} times. On this plant they agree at every call -- that is WHY \
                    the pairs cannot see the defect, and it is no longer true.");
        assert!(agree > 0, "{tag}: the injected cell was never called ({agree} observations) \
                            -- an injection that never applies reports every zero below");

        // (c) THE CALL COUNT IS THE WITNESS, and it rises by EXACTLY the number of extra
        //     consultations: one per `powers` call and one per `try_instant_tail` call.
        let extra = field(&base, "powers_reduced") + field(&base, "powers_bled")
            + field(&base, "tail_reduced") + field(&base, "tail_bled");
        assert_eq!(inj.b_of_calls, base.b_of_calls + extra,
                   "{tag}: b_of_calls {} -> {} against an expected +{extra}",
                   base.b_of_calls, inj.b_of_calls);
        assert!(inj.b_of_calls > base.b_of_calls,
                "{tag}: the ONE counter that can see the re-read did not move");
        assert_eq!(agree + disagree, extra,
                   "{tag}: the wrapper observed {agree} calls but the counters say {extra}");
    }
    // § 5.21 (v)'s own machine, at step 3's own numbers: 409 -> 818, and the two schedule
    // classifications 12 -> 24 and 397 -> 794. Pinned because they are the figures the write-up
    // quotes; the clean side of each is `K/sched/*` in the committed golden.
    let g = golden();
    let base = march_census(&clean(&bleed_arm()));
    assert_eq!(base.b_of_calls, want_u(&g, "K/sched/b_of_calls"));
    let inj = march_census(&injected(&bleed_arm(), &INJ_TWO, &R57_STATOR_SHIPPED, &R62_FUEL));
    assert_eq!(inj.b_of_calls, 2 * base.b_of_calls, "409 -> 818 on the scheduled machine");
    assert_eq!(inj.b_of_sched_zero, 2 * base.b_of_sched_zero, "12 -> 24");
    assert_eq!(inj.b_of_sched_open, 2 * base.b_of_sched_open, "397 -> 794");
    counters::reset();
}

/// Rung 62's SHIPPED stator table, named so the two injections above can hold it fixed while
/// swapping the table they are actually about. It is not `R62_STATOR` re-declared — it IS
/// `R62_STATOR`, and gate 3 below is the one that replaces it.
static R57_STATOR_SHIPPED: turbojet::stator_transient::StatorTransientHooks =
    turbojet::bleed_transient::R62_STATOR;

// =============================================================================================
// GATE 3 — **P2: THE MANUFACTURED `at_stator` BUG**
// =============================================================================================

/// § 5.21 (ii)'s table, manufactured. `R62_STATOR` is `{ at_stator: r62_at_stator, ..R57 }`, so
/// the machine with rung 62's override removed is the machine built on **`R57` itself** — no new
/// item, no source change.
///
/// The counterfeit this pins: rung 62 overrode `at_stator` so a rung-57 reader on a bleed-armed
/// machine differences against a sibling CARRYING THIS MACHINE'S VALVE. That makes rung 59's
/// `schedule_invariance` compare the plant with ITSELF and return `ordinate_identical = true` —
/// **numerically identical to rung 59's published headline while measuring nothing at all.**
///
/// The clean side is read from the committed golden by KEY. The injected side's two bars are
/// § 5.21 (ii)'s own printed numbers at the bar a printed value licenses — *half a unit in its
/// own last printed decimal place* — and the margins are recorded at each row.
#[test]
fn the_at_stator_override_is_what_makes_rung59s_reader_measure_anything() {
    let g = golden();
    let fl = flight();
    let m = clean(&bleed_arm());

    // --- the CLEAN side, against the golden.
    let sib = m.at_stator(StatorArm::default());
    assert!(sib.armed_bleed(),
            "the shipped sibling must CARRY this machine's valve -- that is the whole override");
    assert_eq!(sib.armed_bleed() as u64, want_u(&g, "G/trap/sched/sibling_armed_bleed"));
    let inv = m.schedule_invariance(&fl, LO, HI, MARGIN, 13);
    assert_eq!(inv.ordinate_identical as u64, want_u(&g, "G/trap/sched/ordinate_identical"));
    assert_eq!(inv.abscissa_identical as u64, want_u(&g, "G/trap/sched/abscissa_identical"));
    assert!(inv.ordinate_identical && inv.abscissa_identical,
            "the counterfeit reads TRUE/TRUE on a bleed-armed machine -- if it does not, the \
             golden was regenerated against a build that had already lost the override");
    assert_eq!(inv.d_ordinate.to_bits(), want_u(&g, "G/trap/sched/d_ordinate"));
    assert_eq!(inv.d_abscissa.to_bits(), want_u(&g, "G/trap/sched/d_abscissa"));

    // --- the INJECTED side: rung 57's `at_stator`, i.e. the table without the override.
    let inj = injected(&bleed_arm(), &R62_TWO, &R57, &R62_FUEL);
    // THE STRUCTURAL WITNESS, and it is SHARPER than § 5.21 (ii)'s Python row predicted. The
    // plan measured `at_stator()._armed_bleed()` as *"no such method"* — an `AttributeError`,
    // because Python's un-overridden `at_stator` hands back a `ScheduledStatorTransient`. The
    // Rust does the same thing with a TABLE: `r57_at_stator` builds a core carrying `NO_LEVER`,
    // whose `armed_bleed` cell PANICS rather than answering `false`. That refusal IS the port of
    // the AttributeError, and it is a stronger witness than a `false` would be — `false` is a
    // claim, and the crate's own comment at the refusal site says why it declines to make one.
    let sib_text = refusal(|| { inj.at_stator(StatorArm::default()).armed_bleed(); })
        .expect("without the override the sibling is a RUNG-57 object, which has no valve to \
                 report on -- if it now answers, the override is no longer what distinguishes \
                 the two machines");
    assert!(sib_text.contains("no lever table"),
            "the refusal must be the missing-lever-table one; got: {sib_text:?}");
    let inv_i = inj.schedule_invariance(&fl, LO, HI, MARGIN, 13);
    assert!(!inv_i.ordinate_identical && !inv_i.abscissa_identical,
            "§ 5.21 (ii): the two identities must flip true/true -> false/false, got {}/{}",
            inv_i.ordinate_identical, inv_i.abscissa_identical);

    // § 5.21 (ii)'s printed values, at a printed value's own bar. Measured at step 3:
    //   d_ordinate  9.54314506e-3  vs  9.543e-3  -> 1.45e-7 inside a 5.0e-7 bar
    //   d_abscissa  1.01882344e-2  vs  1.019e-2  -> 1.77e-6 inside a 5.0e-6 bar
    for (name, got, want, bar) in [
        ("d_ordinate", inv_i.d_ordinate, 9.543e-3, 0.5e-6),
        ("d_abscissa", inv_i.d_abscissa, 1.019e-2, 0.5e-5)] {
        assert!((got - want).abs() < bar,
                "{name}: {got:.8e} is {:.2e} from § 5.21 (ii)'s printed {want:.4e}, outside the \
                 {bar:.1e} that value's own last printed decimal licenses",
                (got - want).abs());
    }

    // AND THE CLEAN NUMBERS ARE NOT THE INJECTED ONES — the two runs must actually differ, or
    // the whole gate is comparing one computation with itself.
    assert!(inv.d_ordinate != inv_i.d_ordinate && inv.d_abscissa != inv_i.d_abscissa,
            "the injected and clean readers returned the same numbers -- the injection did not \
             apply");
    counters::reset();
}

// =============================================================================================
// GATE 4 — **THE CARRIER GATE, WITH THE `Floor::Incidence` CELL STEP 3 SAID IT NEEDS**
// =============================================================================================

fn cell_fields(c: &CellRead) -> [(&'static str, f64); 13] {
    [("m_i", c.m_i), ("m_i_grid", c.m_i_grid), ("m_phi", c.m_phi), ("s", c.s), ("v", c.v),
     ("s_grid", c.s_grid), ("min_phi", c.min_phi), ("nu0", c.nu0),
     ("nu_lp_end", c.nu_lp_end), ("nu_hp_end", c.nu_hp_end), ("Tt4_peak", c.tt4_peak),
     ("fuel_removed", c.fuel_removed), ("s_eng", c.s_eng)]
}

/// Step 3's finding 4, both halves.
///
/// `..R43` versus `..R57_FUEL` swaps `try_surge_fuel`, and the two ARE different functions — but
/// `r57_try_surge_fuel` is a **wrapper**: it resolves the floor and then delegates to
/// `R43.try_surge_fuel`. On a `Floor::Phi` that resolution is the IDENTITY, so the two bodies
/// agree exactly. **Both ported suites build only `Floor::Phi`** — once, in rung 63's
/// `every_march_stays_choked` — and never `Floor::Incidence`, the one input the resolution step
/// changes. So "0 of 88 catch it" is a fact about the suites' INPUTS, not about the defect.
///
/// The `Phi` half of this gate is therefore not filler: it is the finding. Asserting only that
/// `Incidence` breaks would leave the reason unrecorded, and a later slice could "fix" the
/// coverage gap by adding a `Phi` gate that still cannot see anything.
#[test]
fn the_r43_spread_is_invisible_on_phi_and_fatal_on_incidence() {
    let fl = flight();
    let arm = bleed_arm();
    let m_clean = clean(&arm);
    let m_inj = injected(&arm, &R62_TWO, &R57_STATOR_SHIPPED, &INJ_FUEL);

    // --- (a) ON `Floor::Phi` THE TWO BODIES ARE BIT-IDENTICAL. This is the finding.
    //
    // **AND THE SET POINT IS CHOSEN SO THE LEG BINDS.** At `sm = 0.40` the floor is DORMANT on
    // a bleed-armed plant — the valve has already lifted `min_phi` to 0.7887 against a
    // `phi_lim` of 0.7700 — which is rung 63's own dichotomy, and exactly the disarming the
    // oracle's `E/fd/bled/row2` records. Two inert paths agreeing is not an agreement, so each
    // cell below asserts its own `fuel_removed > 0`, and the two armings carry DIFFERENT set
    // points for that reason: a bare-lever plant binds at 0.43, a bleed-armed one only at 0.46.
    for (tag, arm_here, sm) in [("bare", LeverArm::default(), 0.43),
                                ("bled", bleed_arm(), 0.46)] {
        let c1 = clean(&arm_here);
        let c2 = injected(&arm_here, &R62_TWO, &R57_STATOR_SHIPPED, &INJ_FUEL);
        let lim = SurgeLimiter::from_margin(&lp_map(), Spool::Lp, sm);
        let phi_leg = StatorLeg { accel: None, surge: Some(Floor::Phi(lim)), tt4_max: None };
        let a = c1.cell(&fl, &ramp(0.5, DS_63), Spool::Lp, &phi_leg);
        let b = c2.cell(&fl, &ramp(0.5, DS_63), Spool::Lp, &phi_leg);
        assert!(a.fuel_removed > 0.0,
                "{tag}: the floor leg at sm={sm} removed no fuel, so this comparison exercised \
                 no BINDING `try_surge_fuel` call at all -- two inert paths agreeing is not an \
                 agreement");
        for ((k, va), (_, vb)) in cell_fields(&a).into_iter().zip(cell_fields(&b)) {
            assert_eq!(va.to_bits(), vb.to_bits(),
                       "{tag}/phi/{k}: {va:.17e} vs {vb:.17e}. On a `Floor::Phi` the rung-57 \
                        wrapper's resolution is the IDENTITY, so these two tables MUST agree \
                        exactly -- if they no longer do, finding 4's reason has changed and the \
                        `Phi`-only blindness needs re-measuring.");
        }
    }

    // --- (b) ON `Floor::Incidence` THE WRONG SPREAD REFUSES. The `Incidence` kind reaches
    // --- `try_surge_fuel` only through rung 57's resolving wrapper; `R43`'s body indexes
    // --- `surge.phi_lim` directly and `Floor::phi()` refuses a rung-60 object.
    let cmap = m_clean.arming().design_map(Spool::Lp);
    let inc = IncidenceLimiter::from_margin(&cmap, Spool::Lp, 0.40);
    let inc_leg = StatorLeg { accel: None, surge: Some(Floor::Incidence(inc)), tt4_max: None };

    let clean_ok = refusal(|| { m_clean.cell(&fl, &ramp(0.5, DS_62), Spool::Lp, &inc_leg); });
    assert!(clean_ok.is_none(),
            "the SHIPPED table must RESOLVE an incidence floor, but it refused: {}",
            clean_ok.unwrap_or_default());

    let text = refusal(|| { m_inj.cell(&fl, &ramp(0.5, DS_62), Spool::Lp, &inc_leg); })
        .expect("the `..R43` spread must REFUSE an incidence floor -- if it no longer does, the \
                 wrong table is no longer distinguishable from the right one by ANY input, and \
                 this gate has stopped testing anything");
    assert!(text.contains("_resolve_floor") || text.contains("IncidenceLimiter"),
            "the refusal must be rung 43's own, naming the floor it cannot read; got: {text:?}");
    counters::reset();
}

// =============================================================================================
// GATE 5 — THE INJECTIONS ARE NOT NO-OPS, AND EACH TOUCHES ONLY ITS OWN TABLE
// =============================================================================================

/// **A MANUFACTURED-BUG GATE IS CODE TOO** — slice V step 5's closing lesson, applied to this
/// file's own three injections. Each of them must (a) actually change the machine, and (b) leave
/// the other two channels alone, or a green run above could be a green run of the shipped code
/// three times over.
#[test]
fn each_injection_changes_its_own_channel_and_nothing_else() {
    let fl = flight();
    let arm = bleed_arm();
    let base = clean(&arm);

    // The three injected machines, one table swapped each.
    let i_two = injected(&arm, &INJ_TWO, &R57_STATOR_SHIPPED, &R62_FUEL);
    let i_stator = injected(&arm, &R62_TWO, &R57, &R62_FUEL);
    let i_fuel = injected(&arm, &R62_TWO, &R57_STATOR_SHIPPED, &INJ_FUEL);

    // The un-injected control: the same `with_tables` call with the SHIPPED tables. It must be
    // bit-identical to `build_scheduled_bleed`, or the harness itself is the variable.
    let control = injected(&arm, &R62_TWO, &R57_STATOR_SHIPPED, &R62_FUEL);
    let e0 = base.fuel.inner.equilibrium(&fl, 1200.0);
    let ec = control.fuel.inner.equilibrium(&fl, 1200.0);
    assert_eq!(e0.close.n_lp.to_bits(), ec.close.n_lp.to_bits(),
               "the injection harness moved the machine with NO table swapped -- every \
                difference measured in this file would be the harness's");

    // (1) INJ_TWO changes the `b_of` call count and NOTHING a value key can see.
    let n0 = march_census(&base);
    AGREE.with(|x| x.set(0));
    DISAGREE.with(|x| x.set(0));
    let n1 = march_census(&i_two);
    assert!(n1.b_of_calls > n0.b_of_calls, "INJ_TWO did not apply");
    let e1 = i_two.fuel.inner.equilibrium(&fl, 1200.0);
    assert_eq!(e0.close.n_lp.to_bits(), e1.close.n_lp.to_bits(),
               "INJ_TWO moved a VALUE -- it is supposed to be value-invisible, which is the \
                entire reason gate 2 counts calls instead");

    // (2) INJ_STATOR changes the sibling and nothing on the forward closure.
    let e2 = i_stator.fuel.inner.equilibrium(&fl, 1200.0);
    assert_eq!(e0.close.n_lp.to_bits(), e2.close.n_lp.to_bits(),
               "the `at_stator` swap reached the forward closure, which it does not touch");
    assert!(base.at_stator(StatorArm::default()).armed_bleed(),
            "the SHIPPED sibling carries the valve");
    assert!(refusal(|| { i_stator.at_stator(StatorArm::default()).armed_bleed(); }).is_some(),
            "INJ_STATOR did not apply -- its sibling still has a lever table");

    // (3) INJ_FUEL is invisible everywhere the suites look, which is the point of gate 4.
    let e3 = i_fuel.fuel.inner.equilibrium(&fl, 1200.0);
    assert_eq!(e0.close.n_lp.to_bits(), e3.close.n_lp.to_bits(),
               "the fuel-table spread reached the Tt4-pinned closure, which it does not touch");
    let lim = SurgeLimiter::from_margin(&lp_map(), Spool::Lp, 0.40);
    let leg = StatorLeg { accel: None, surge: Some(Floor::Phi(lim)), tt4_max: None };
    assert_eq!(base.cell(&fl, &ramp(0.5, DS_62), Spool::Lp, &leg).m_i.to_bits(),
               i_fuel.cell(&fl, &ramp(0.5, DS_62), Spool::Lp, &leg).m_i.to_bits(),
               "INJ_FUEL moved a `Floor::Phi` cell -- finding 4 says it cannot");
    counters::reset();
}
