//! SLICE AB step 5 — **THE TEN CELLS, EACH SWAPPED FOR ITS PARENT'S FUNCTION POINTER.**
//!
//! `slice_ab_oracle.rs` is green at **15 957 keys per arm** against two interpreters, and
//! `rung69.rs`'s 25 ported gates are green. Both are value instruments, and **no value key can
//! witness a hook table**. That gap is worse at this slice than at any before it, because rung 69
//! **adds one cell and SWAPS TEN**: its failure mode is not a forgotten cell (which fails at its
//! own first gate) but *a swap whose Rust body is still effectively the parent's* — which
//! compiles, runs, and is caught by nothing the ladder does automatically.
//!
//! So the injection here is **not** a hand-written plausible bug the way `slice_aa_dispatch.rs`'s
//! nine were. AA's cells were NEW, so a manufactured body was the only injection available. AB's
//! risk IS the parent, so **the parent's own function pointer is the injection** — which is also
//! exactly what § 5.26 (ix)'s P2 pre-registered: *"swapping the cell for the parent's function
//! pointer breaks at least one gate."*
//!
//! # WHAT THE INJECTIONS MEASURED, AND WHERE THE PRE-FLIGHT WAS WRONG
//!
//! § 5.26 (ii) predicted the SHAPE of each break, and **two of its four panic-shaped cells break
//! BY VALUE instead**. Both are recorded here as findings rather than quietly re-gated:
//!
//! * **`triple_rig`** — the pre-flight said rung 68's body dereferences `stator_lim`, which is
//!   `None` under an incidence arming, so the parent pointer panics on all 60 calls. It does not.
//!   [`r68_triple_rig`](turbojet::three_loop) never READS `stator_lim`; it BUILDS a
//!   [`StatorLimiter`] from the map. The parent therefore hands back a perfectly well-formed
//!   sibling carrying the **wrong reference** — a `phi` stator where an incidence one was asked
//!   for — and nothing raises. That is the more dangerous shape, not the less.
//! * **`manifold_v`** — same prediction, same reason, also wrong. Rung 68's body is `V(g, q)[0]`:
//!   it ignores every argument but the law it is handed, so it reads no field at all. Its value is
//!   the stator's OWN root, where rung 69's is the SHARED manifold, and at the sampled point the
//!   two have **opposite signs**.
//!
//! The other two hold: `stator_leg` and `solve_v` really do break by panic, because rung 68's
//! bodies for those two DO dereference the floor this machine does not carry.
//!
//! # THE SIBLING-CONSTRUCTOR TRAP APPLIES HERE VERBATIM, AND IT DECIDES EVERY OBSERVABLE
//!
//! `slice_aa_dispatch.rs` recorded it: any reader reached through `_triple_rig` -> `at_lever`
//! builds a sibling through [`build_reference_split_cascade`], which installs the **SHIPPED**
//! `&R69_TRIPLE`. All six of rung 69's readers do exactly that. So an injection into a
//! `TripleHooks` cell is **invisible to every reader** — it must be scored on a march of the
//! injected machine itself, or on a `triple_gains_at` called against it directly. Measured, not
//! assumed: with `manifold_v` set to the parent, `reference_gains` returns the identical two rows.
//!
//! # P2, SETTLED PER CELL AND EMITTED
//!
//! [`the_ten_cells_are_all_observable`] re-runs every injection and tallies, so the count is
//! EMITTED and never typed — this phase has been caught six times on a tally written beside the
//! addends that disprove it, once inside AA's own oracle doc comment.
//!
//! **AND THE *SHAPE* OF EACH BREAK IS CLASSIFIED, NOT ANNOTATED.** This section's first draft
//! carried a `shape` string per row and counted the strings, which is the same defect one level
//! down: the labels were mine, and nothing measured whether a break was a panic. Each cell now
//! names ONE exercise, that exercise runs TWICE — shipped tables, then injected — and the shape
//! falls out of the pair. The control is the same machinery with the shipped tables on BOTH sides:
//! all ten must come back `Silent`.
//!
//! [`StatorLimiter`]: turbojet::three_loop::StatorLimiter
//! [`build_reference_split_cascade`]: turbojet::reference_split::build_reference_split_cascade

use std::panic::catch_unwind;

use turbojet::bleed_transient::{LeverArm, LeverArming, LeverHooks};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelPoint, PointExtra, SurgeLimiter,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::reference_split::{
    reference_bill, reference_gains, Census69, RefScope, StatorIncidenceLimiter, R69, R69_FUEL,
    R69_STATOR, R69_TRIPLE, R69_TWO,
};
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::three_loop::{triple_gains_at, v_at_point, TripleHooks, TripleRigArm, R68, R68_TRIPLE};
use turbojet::two_lag::violation;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::TwoSpoolTransientCore;

// ---------------------------------------------------------------------------- the grid
//
// `tests/rung69.rs`'s grid, verbatim, so a delta measured here is a delta on the machine the 25
// ported gates and the 15 957 oracle keys were taken on.
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
/// The expression, never a typed decimal — rung 69's guard D is that the three floors are ONE
/// PHYSICAL WALL, and a rounded constant would break it silently.
const SM: f64 = PHI / FLOOR - 1.0;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;

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

fn valve() -> BleedLimiter { BleedLimiter::with_tau(PHI, B, Some(TAU)) }

fn inc() -> StatorIncidenceLimiter {
    StatorIncidenceLimiter::from_margin(&lp(), V_MAX, SM, Some(TAU_S))
}

/// A rung-69 machine carrying `lever` and `tri` as its tables.
///
/// **NOT through [`build_reference_split_cascade`]**, which hardcodes `&R69` and `&R69_TRIPLE` —
/// the whole point here is to install a table it would never install. The four rung-69 guards the
/// builder asserts are not re-spelled: this arming (`stator_inc` armed, no `stator_lim`, no
/// constant or scheduled setting, an LP spool, and the valve built from the SAME `sm`) is the one
/// they admit, and `slice_ab_cells.rs` is where the guards themselves are gated.
///
/// [`build_reference_split_cascade`]: turbojet::reference_split::build_reference_split_cascade
fn machine(lever: &'static LeverHooks, tri: &'static TripleHooks) -> ScheduledStatorCore {
    let arm = LeverArm { bleed_lim: Some(valve()), stator_inc: Some(inc()), ..Default::default() };
    match ScheduledStatorTransient::with_ref_tables(
        design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm.stator,
        &R69_TWO, &R69_STATOR, &R69_FUEL, lever,
        LeverArming { bleed: arm.bleed, sched: arm.bleed_sched, lim: arm.bleed_lim },
        tri, arm.stator_lim, arm.stator_inc)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming never disables LP"),
    }
}

fn split(tri: &'static TripleHooks) -> ScheduledStatorCore { machine(&R69, tri) }

fn ramp(ds: f64) -> Ramp { Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds } }

fn fuel_floor() -> Floor { Floor::Phi(SurgeLimiter::from_margin(&lp(), Spool::Lp, SM)) }

fn march_of(m: &ScheduledStatorCore, ds: f64, scope: MarchScope) -> Vec<FuelPoint> {
    let leg = StatorLeg { accel: None::<&AccelSchedule>, surge: Some(fuel_floor()), tt4_max: None };
    m.stator_march_scoped(&flight(), &ramp(ds), None, &leg,
                          &MarchScope { lag: Some(AsymmetricLag::new(0.05, 0.15)), ..scope }).0
}

fn rig_arm() -> TripleRigArm { TripleRigArm { sm: SM, ..TripleRigArm::default() } }

/// The four numbers every value injection below is scored on.
///
/// **`v_max` AND NOT `v_min`, WHICH IS THE MIRROR OF SLICE AA's READING.** Rung 68's band is
/// `[-v_max, 0]`, so AA scored the DEEPEST excursion; rung 69's is `[0, +v_max]`, so the riding
/// setting is the HIGHEST one and the dormant stop is the bottom of the band. A reading copied
/// from AA would report `v_min = 0` on every machine here, injected or not.
#[derive(Clone, Copy, Debug)]
struct Reading {
    /// The violation integral of the `phi` floor over the ramp.
    viol: f64,
    /// The dormant stop — `0.0` on every rung-69 march, asserted so the band's orientation is a
    /// measurement and not an assumption.
    v_min: f64,
    /// The deepest RIDE, which is the positive end here.
    v_max: f64,
    /// How many points carried a FIVE-state record at all.
    five: usize,
}

fn reading(tri: &'static TripleHooks) -> Reading { reading_with(&R69, tri) }

fn reading_with(lever: &'static LeverHooks, tri: &'static TripleHooks) -> Reading {
    let t = march_of(&machine(lever, tri), DS, MarchScope::DEFAULT);
    let vs: Vec<f64> = t.iter().filter(|p| matches!(p.extra, PointExtra::Triple { .. }))
        .map(v_at_point).collect();
    Reading {
        viol: violation(&t, PHI, R),
        v_min: vs.iter().cloned().fold(f64::INFINITY, f64::min),
        v_max: vs.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        five: vs.len(),
    }
}

fn message_of<F: FnOnce() + std::panic::UnwindSafe>(f: F) -> String {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = catch_unwind(f);
    std::panic::set_hook(prev);
    match out {
        Ok(()) => String::new(),
        Err(e) => e.downcast_ref::<String>().cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default(),
    }
}

// ============================================================================== the ten swaps
//
// Nine spreads of `R69_TRIPLE` with ONE cell taken from `R68_TRIPLE`, plus the tenth in the LEVER
// table. Written through a macro so that the injection is provably the parent's pointer and not a
// re-spelling of it: a hand-copied body would be a THIRD implementation and could agree with
// neither.

macro_rules! parent_swap {
    ($name:ident, $cell:ident) => {
        static $name: TripleHooks = TripleHooks { $cell: R68_TRIPLE.$cell, ..R69_TRIPLE };
    };
}
parent_swap!(P_STATOR_LEG, stator_leg);
parent_swap!(P_LAGGED_STATOR, lagged_stator);
parent_swap!(P_CLAMP_V, clamp_v);
parent_swap!(P_CHECK_V0, check_v0);
parent_swap!(P_RK4_FLOOR, rk4_floor);
parent_swap!(P_SOLVE_V, solve_v);
parent_swap!(P_MANIFOLD_V, manifold_v);
parent_swap!(P_TRIPLE_RIG, triple_rig);
parent_swap!(P_WITH_REF, with_ref);

/// The tenth cell lives in the LEVER table, so it needs its own spread.
static P_AT_LEVER: LeverHooks = LeverHooks { at_lever: R68.at_lever, ..R69 };

/// **THE ONE INJECTION THAT IS NOT A PARENT POINTER, AND IT IS DECLARED AS SUCH.**
///
/// Rung 68's `_with_ref` is a REFUSAL — there is no reference to select below rung 69 — so the
/// parent pointer settles P2's letter (a reader does dispatch through the cell) by making every
/// reader raise. That is a loud failure, and the failure this cell actually risks is a silent one:
/// a setter that does not set. Python's `prev, self._ref = self._ref, ref` is one statement, and a
/// port that returned `prev` without writing the field would leave every reader's scope inert,
/// `_triple_rig` falling through to `self._ref or (…)` and labelling the WRONG arm.
///
/// So this cell gets BOTH injections and the counterfeit carries the finding.
fn counterfeit_with_ref(_: &TwoSpoolTransientCore, _: Option<&'static str>) -> Option<&'static str> {
    None
}
static C_WITH_REF: TripleHooks = TripleHooks { with_ref: counterfeit_with_ref, ..R69_TRIPLE };

// ============================================================================== the baseline

/// **ASSERTED, NEVER ASSUMED.** Every gate below is a difference against these numbers, so a
/// baseline that had itself drifted would make ten gates report ten wrong deltas and all ten still
/// pass.
///
/// The `five` count and the band's orientation are the two the deltas lean on hardest.
fn baseline() -> Reading {
    Census69::reset();
    let r = reading(&R69_TRIPLE);
    assert_eq!(r.five, 341, "the five-state integrator must be entered");
    assert!(r.viol > 0.0, "the baseline march must VIOLATE the phi floor somewhere: {}", r.viol);
    assert_eq!(r.v_min, 0.0, "rung 69's dormant stop IS the bottom of its band");
    assert!(r.v_max > 0.01, "...and the stator must RIDE on the POSITIVE side: {}", r.v_max);
    let c = Census69::read();
    assert!(c.leg_inc > 0 && c.solve_v_calls > 0 && c.regime_riding > 0,
            "the baseline must run rung 69's OWN bodies, not a reduce arm: {c:?}");
    assert_eq!((c.leg_parent, c.solve_parent, c.clamp_parent, c.manifold_parent), (0, 0, 0, 0),
               "...and no reduce arm at all on an armed incidence machine");
    r
}

// ============================================================================== the ten gates

/// 1. `stator_leg` — **BY PANIC.** Rung 68's body reads `self.stator_lim`, which an incidence
///    arming leaves `None`, and the five-state integrator's very first line calls this cell.
#[test]
fn cell_1_stator_leg_is_reached() {
    baseline();
    let msg = message_of(|| { reading(&P_STATOR_LEG); });
    assert!(msg.contains("rung-68's march with no stator floor"),
            "the parent must be REACHED by the integrator's first line; got {msg:?}");
}

/// 2. `lagged_stator` — **BY VALUE, and the value is the number of STATES.** Rung 68's body asks
///    `stator_lim is not None`, which is `false` here, so the machine drops to rung 67's
///    three-state march: the third loop vanishes with nothing raising.
///
/// The violation FALLS by 4 % when the loop is removed, which is not a mistake — it is rung 69's
/// own headline in miniature. The incidence loop watches a coordinate the `phi` floor does not,
/// so in the `phi` currency it is a DEBIT. A gate that assumed "protection removed ⇒ more
/// violation" would have been written backwards.
#[test]
fn cell_2_lagged_stator_is_reached() {
    let base = baseline();
    let got = reading(&P_LAGGED_STATOR);
    assert_eq!(got.five, 0, "`lagged_stator = false` is the reduce to three states, by dispatch");
    let drop = (base.viol - got.viol) / base.viol;
    assert!((0.03..0.05).contains(&drop),
            "the third loop is a DEBIT in the phi currency; measured 4.1 %, got {drop}");
}

/// 3. `clamp_v` — **BY VALUE, and it is the silent one.** The band flips back to `[-v_max, 0]`,
///    so every positive command — which is every riding command here, `v > 0` on 25 364 of 25 371
///    inputs by § 5.26 (xi) — is clamped to `0`. The march still runs, still carries five states,
///    still reports a stator regime, and the actuator never leaves its stop.
#[test]
fn cell_3_clamp_v_is_reached_and_the_band_is_one_sided() {
    let base = baseline();
    let got = reading(&P_CLAMP_V);
    assert_eq!(got.five, 341, "the march still runs on five states -- that is the point");
    assert_eq!(got.v_max, 0.0, "...with the stator pinned at its stop by the mirrored band");
    let drop = (base.viol - got.viol) / base.viol;
    assert!((0.10..0.13).contains(&drop), "measured 11.5 %, got {drop}");
}

/// 4. `check_v0` — **BY REFUSAL, IN BOTH DIRECTIONS.** The band is the observable and the cell is
///    the only place it is asserted, so the gate reads all four cases: the shipped cell refuses
///    `v0 = -0.05` and admits `+0.05`, the parent does exactly the opposite.
///
/// Both directions, because a gate that only checked "the shipped cell refuses something" passes
/// with the parent installed — the parent refuses things too, just the mirror set.
#[test]
fn cell_4_check_v0_is_reached_and_the_band_is_mirrored() {
    let v0 = |tri: &'static TripleHooks, v: f64| {
        message_of(move || {
            march_of(&split(tri), DS, MarchScope { v0: Some(v), ..MarchScope::DEFAULT });
        })
    };
    assert!(v0(&R69_TRIPLE, -0.05).contains("rung-69 v0"),
            "rung 69 refuses a NEGATIVE initial position");
    assert!(v0(&R69_TRIPLE, 0.05).is_empty(), "...and admits a positive one");
    assert!(v0(&P_CHECK_V0, 0.05).contains("rung-68 v0"),
            "the parent refuses the POSITIVE one -- the mirror, and it names its own rung");
    assert!(v0(&P_CHECK_V0, -0.05).is_empty(), "...and admits the negative one");
}

/// 5. `rk4_floor` — **BY MESSAGE, AND BY NOTHING ELSE.** § 5.26 (ii) measured the condition
///    `ds * rate <= 2.0` identical in both rungs, character for character, over 77 calls with ZERO
///    disagreements. The entire difference is the reason the assertion gives: rung 68 justifies
///    the constant by *"J has rank one"*, rung 69 by *"the block is rank TWO and the dominant root
///    is a COMPLEX pair"*.
///
/// **So a gate that asserted only that the march panics at `ds = 0.04` would pass with the parent
/// installed**, and this cell would be the one swap in the slice with no live gate at all. The
/// needle is asserted present on the shipped table and ABSENT on the parent's, which is the only
/// assertion shape that can fail here.
#[test]
fn cell_5_rk4_floor_is_reached_and_only_its_message_can_say_so() {
    let good = message_of(|| { march_of(&split(&R69_TRIPLE), 0.04, MarchScope::DEFAULT); });
    assert!(good.contains("rank TWO"),
            "rung 69's floor must justify the constant by the SPLIT's spectrum: {good:?}");
    let bug = message_of(|| { march_of(&split(&P_RK4_FLOOR), 0.04, MarchScope::DEFAULT); });
    assert!(!bug.is_empty() && !bug.contains("rank TWO"),
            "the parent fires on the same step and gives rung 68's reason: {bug:?}");
    assert!(bug.contains("THE RATES ADD"), "...which is the rank-one one: {bug:?}");
}

/// 6. `solve_v` — **BY PANIC.** Rung 68's outer solve reads `self.stator_lim` for its band and its
///    set point; there is none. The reduce arms are counted, and the census shows the injected
///    march never reached rung 69's own solve.
#[test]
fn cell_6_solve_v_is_reached() {
    baseline();
    Census69::reset();
    let msg = message_of(|| { reading(&P_SOLVE_V); });
    assert!(msg.contains("`_solve_v` on a machine with no stator floor"),
            "the parent must be REACHED by the command solve; got {msg:?}");
    assert_eq!(Census69::read().solve_v_calls, 0,
               "and rung 69's own solve never ran on that march");
}

/// The march point every `manifold_v` reading below is taken at — index 37 of the injected
/// machine's own five-state trajectory, which is the first sampled point where BOTH the shipped
/// and the parent gains come back interior.
fn manifold_point() -> (ScheduledStatorCore, FuelPoint) {
    let m = split(&R69_TRIPLE);
    let t = march_of(&m, DS, MarchScope::DEFAULT);
    let p = t.iter().filter(|p| matches!(p.extra, PointExtra::Triple { .. }))
        .nth(37).expect("the march carries 341 five-state points").clone();
    assert!((p.s - 0.185).abs() < 1e-9, "the sampled point moved: s = {}", p.s);
    (m, p)
}

fn v_base_of(tri: &'static TripleHooks, p: &FuelPoint) -> (f64, bool) {
    let surge = fuel_floor();
    let g = triple_gains_at(&split(tri), &flight(), p, None, Some(&surge),
                            1e-7, 1e-5, 1e-4, true, 0.0, true)
        .expect("the gains at an interior point do not abort");
    (g.v_base, g.interior)
}

/// 7. `manifold_v` — **BY VALUE, AND BY A SIGN.** § 5.26 (ii) predicted a panic; there is none,
///    because rung 68's body is `V(g, q)[0]` and reads no field whatever. What it returns is the
///    stator's OWN root, where rung 69's is the SHARED manifold `phi = phi_lim`, rooted UNCLAMPED
///    on `[-0.6, +0.6]`. At the sampled point the shared manifold sits at `v < 0` — outside the
///    incidence loop's own band entirely — while the parent reports `+6.17e-3`.
///
/// **AND THE INJECTION IS INVISIBLE TO ALL SIX READERS**, measured: `reference_gains` with this
/// cell set to the parent returns the identical two rows, because it scores its gains on the
/// sibling `_triple_rig` builds, and that sibling carries the SHIPPED table. So the gate calls
/// `triple_gains_at` against the injected machine directly. `slice_aa_dispatch.rs` recorded this
/// trap one rung down; here it is the difference between a live gate and one that asserts nothing.
#[test]
fn cell_7_manifold_v_is_reached_and_it_breaks_by_value_not_by_panic() {
    let (_, p) = manifold_point();
    let (shipped, si) = v_base_of(&R69_TRIPLE, &p);
    let (parent, pi) = v_base_of(&P_MANIFOLD_V, &p);
    assert!(si && pi, "both arms are interior at the sampled point");
    assert!(shipped < 0.0, "the SHARED manifold sits below the band: {shipped}");
    assert!(parent > 0.0, "the stator's OWN root sits inside it: {parent}");
    assert!((shipped - -3.382951e-3).abs() < 1e-8, "the shared manifold moved: {shipped}");
    assert!((parent - 6.170912e-3).abs() < 1e-8, "the parent's root moved: {parent}");
    // AND THE READER CANNOT SEE IT — the sibling-constructor trap, asserted rather than described.
    let rows = |tri: &'static TripleHooks| {
        reference_gains(&split(tri), &flight(), &ramp(DS), SM, &rig_arm(), 40)
            .rows.iter().map(|r| r.v_base.to_bits()).collect::<Vec<_>>()
    };
    assert_eq!(rows(&R69_TRIPLE), rows(&P_MANIFOLD_V),
               "every reader scores the SIBLING, which carries the shipped table");
}

/// 8. `triple_rig` — **BY THE SIBLING'S ARMING, and § 5.26 (ii)'s panic prediction is FALSIFIED.**
///
/// Rung 68's rig does not read `stator_lim`; it BUILDS a [`StatorLimiter`] from the map. So the
/// parent returns a well-formed sibling that marches five states, reports a stator credit and
/// costs nothing — carrying the WRONG REFERENCE. That is the shape this rung exists to make
/// visible, and no float in the ledger reads it: [`Census69`]'s two rig counters are the
/// instrument, and they are declared beside the bodies for exactly this reason.
///
/// [`StatorLimiter`]: turbojet::three_loop::StatorLimiter
#[test]
fn cell_8_triple_rig_is_reached_and_it_breaks_by_the_reference_not_by_panic() {
    for (tri, phi_armed, inc_armed, counters) in
        [(&R69_TRIPLE, false, true, (1, 0)), (&P_TRIPLE_RIG, true, false, (0, 0))]
    {
        Census69::reset();
        let m = split(tri);
        let (sib, _, _) = m.triple_rig(&rig_arm());
        assert_eq!(sib.fuel.inner.stator.lim.is_some(), phi_armed);
        assert_eq!(sib.fuel.inner.stator.inc.is_some(), inc_armed);
        let c = Census69::read();
        assert_eq!((c.rig_inc, c.rig_phi), counters,
                   "the rig counters are the only witness of the reference");
    }
}

/// 9a. `with_ref` — **BY PANIC, through a reader.** Rung 68's cell is a refusal: there is no
///     reference to select below rung 69. So the parent pointer settles P2's letter loudly — every
///     reader that opens a [`RefScope`] raises.
#[test]
fn cell_9a_with_ref_is_reached_by_every_reader() {
    let msg = message_of(std::panic::AssertUnwindSafe(|| {
        reference_gains(&split(&P_WITH_REF), &flight(), &ramp(DS), SM, &rig_arm(), 40);
    }));
    assert!(msg.contains("_with_ref is RUNG 69's and does not exist below it"),
            "the reader must dispatch through the cell; got {msg:?}");
}

/// 9b. **THE SILENT COUNTERFEIT, AND IT IS THIS FILE's SHARPEST GATE.** A setter that returns the
///     displaced value without writing the field leaves every reader's scope inert. Nothing
///     raises; `_triple_rig` falls through to `self._ref or ("phi" if stator_lim else "inc")` and
///     builds a SECOND incidence rig where the `phi` one was asked for.
///
/// **§ 5.26.1 (j) IS CONFIRMED AND SHARPENED.** It registered that the obvious ledger key cannot
/// see this: `reference_bill`'s `bare`/`F`/`V`/`FV` cells carry no stator and are identical
/// between the arms by construction, so `common_max_rel` reads `0` with the defect live. Measured
/// here — and the same run measures what the ledger CAN see: `delivered`, whose two arms are
/// `(93.83, 96.53)` shipped and the SAME NUMBER TWICE under the counterfeit. So the ledger is not
/// blind, only its self-check is; the discriminating keys are the ones that MUST differ.
///
/// The gains reader is the sharper witness of the two: `phi.pair_RV` is exactly `1.0` when the
/// `phi` rig is real — its two loops hold ONE wall — and becomes bit-for-bit the incidence rig's
/// own `-1.85…` when it is not.
#[test]
fn cell_9b_a_setter_that_does_not_set_labels_the_wrong_arm() {
    let read = |tri: &'static TripleHooks| {
        Census69::reset();
        let m = split(tri);
        let g = reference_gains(&m, &flight(), &ramp(DS), SM, &rig_arm(), 40);
        // Read BEFORE the ledger runs: `triple_bill` builds rigs of its own, and a census taken
        // after both readers would tally two questions at once.
        let c = Census69::read();
        let b = reference_bill(&m, &flight(), &ramp(DS), SM, &rig_arm());
        let pairs: Vec<(u64, u64)> = g.rows.iter()
            .map(|r| (r.inc.pair_rv.to_bits(), r.phi.pair_rv.to_bits())).collect();
        (c, pairs, b.common_max_rel, b.delivered)
    };
    let (cs, ps, common_s, del_s) = read(&R69_TRIPLE);
    let (cc, pc, common_c, del_c) = read(&C_WITH_REF);

    assert_eq!((cs.rig_inc, cs.rig_phi), (1, 1),
               "`reference_gains` ALONE -- the census is read before `reference_bill` runs, \n                because the ledger builds rigs of its own and would carry these to (5, 5)");
    assert_eq!((cc.rig_inc, cc.rig_phi), (2, 0),
               "`reference_gains` alone again: the counterfeit builds the incidence rig TWICE");
    assert!(cs.leg_parent > 0 && cc.leg_parent == 0,
            "and the phi rig is the ONLY thing on this machine that runs a reduce arm: {} vs {}",
            cs.leg_parent, cc.leg_parent);

    assert_eq!(ps.len(), pc.len(), "the two runs sample the same points");
    assert!(!ps.is_empty(), "and there are some");
    for ((i_s, p_s), (i_c, p_c)) in ps.iter().zip(&pc) {
        assert_eq!(i_s, i_c, "the INCIDENCE arm is untouched, bit for bit");
        assert_ne!(p_s, p_c, "the phi arm is not");
        // `1.0` to a central-difference residual, not exactly — the shipped pair reads
        // `1 + 2.6e-10`, which is the gains' own truncation and not a set-point offset.
        assert!((f64::from_bits(*p_s) - 1.0).abs() < 1e-8,
                "a real phi rig holds ONE wall with both loops: {}", f64::from_bits(*p_s));
        assert_eq!(p_c, i_c, "and the counterfeit's `phi` arm IS the incidence arm");
    }

    // The ledger: its self-check is blind, its delivered pair is not.
    assert_eq!((common_s, common_c), (0.0, 0.0),
               "the four stator-free cells agree whichever arm ran -- s 5.26.1 (j), measured");
    assert!((del_s.0 - del_c.0).abs() < 1e-9, "the incidence half of the bill is untouched");
    assert!((del_s.1 - del_s.0).abs() > 1.0, "the shipped arms deliver DIFFERENT credits");
    assert_eq!(del_c.1, del_c.0, "the counterfeit delivers the same number twice");
}

/// 10. `at_lever` — **BY THE SIBLING's ARMING, and no panic either.** Rung 68's `at_lever` copies
///     eight keywords forward and rung 69's copies nine; the ninth is `stator_inc`. So the parent
///     hands back a sibling with **no third loop at all** — rung 69's rig asked for an incidence
///     limiter (the census records that it did), and the constructor dropped it on the floor.
///
/// The sibling then marches perfectly happily on THREE states, which is the seventh instance of
/// the trap rungs 61–69 have each hit and the reason this cell is a cell.
#[test]
fn cell_10_at_lever_is_reached_and_the_sibling_keeps_the_reference() {
    for (lever, five, inc_armed) in [(&R69, 341usize, true), (&P_AT_LEVER, 0usize, false)] {
        Census69::reset();
        let m = machine(lever, &R69_TRIPLE);
        let (sib, surge, lag) = m.triple_rig(&rig_arm());
        assert_eq!(Census69::read().rig_inc, 1, "rung 69's rig asked for an INCIDENCE limiter");
        assert_eq!(sib.fuel.inner.stator.inc.is_some(), inc_armed,
                   "...and only rung 69's `at_lever` carries the ninth keyword through");
        assert!(!sib.fuel.inner.stator.lim.is_some(),
                "neither sibling carries a phi floor -- the loop is LOST, not swapped");
        let leg = StatorLeg { accel: None::<&AccelSchedule>, surge, tt4_max: None };
        let (t, _) = sib.stator_march_scoped(&flight(), &ramp(DS), None, &leg,
                                             &MarchScope { lag, ..MarchScope::DEFAULT });
        assert_eq!(t.iter().filter(|p| matches!(p.extra, PointExtra::Triple { .. })).count(), five,
                   "and the stripped sibling marches on THREE states with nothing raising");
    }
}


// ============================================================================== P2, MEASURED
//
// **THE SHAPE OF A BREAK IS AN OBSERVATION HERE, NEVER A LABEL.** The first draft of this section
// carried a `shape` string per row and then counted the strings — which is the addends and the
// tally in the same hand, the defect this file's own header says it exists to avoid. Nothing in it
// measured whether a break was a panic. So each cell now names ONE exercise, that exercise is run
// TWICE (shipped tables, then injected), and the shape is classified from what the two runs did.

/// What one run of a cell's exercise did: whether it raised, with what message, and a digest of
/// the value it produced if it did not.
struct Run {
    msg: String,
    digest: u64,
}

fn run_exercise(f: impl FnOnce() -> u64) -> Run {
    let d = std::cell::Cell::new(u64::MAX);
    let msg = message_of(std::panic::AssertUnwindSafe(|| d.set(f())));
    Run { msg, digest: d.get() }
}

/// How a cell's swap becomes visible — **derived from two runs, not written down.**
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    /// One run raised and the other did not.
    Panic,
    /// BOTH raised, and the messages differ. `rk4_floor`'s shape, and the reason a
    /// fact-of-a-panic gate would leave that cell ungated.
    Message,
    /// Neither raised and the values differ.
    Value,
    /// Nothing at all — an UNOBSERVABLE cell, which is a result to report, never one to re-gate.
    Silent,
}

fn classify(shipped: &Run, injected: &Run) -> Shape {
    match (shipped.msg.is_empty(), injected.msg.is_empty()) {
        (a, b) if a != b => Shape::Panic,
        (false, false) if shipped.msg != injected.msg => Shape::Message,
        (true, true) if shipped.digest != injected.digest => Shape::Value,
        _ => Shape::Silent,
    }
}

// ---- the ten exercises. Each takes the tables to run under, so the shipped and the injected run
// ---- are the SAME code down to the pointer being tested.

fn ex_five(tri: &'static TripleHooks, lev: &'static LeverHooks) -> u64 {
    reading_with(lev, tri).five as u64
}
fn ex_v_max(tri: &'static TripleHooks, lev: &'static LeverHooks) -> u64 {
    reading_with(lev, tri).v_max.to_bits()
}
fn ex_v0_positive(tri: &'static TripleHooks, lev: &'static LeverHooks) -> u64 {
    march_of(&machine(lev, tri), DS, MarchScope { v0: Some(0.05), ..MarchScope::DEFAULT }).len()
        as u64
}
fn ex_coarse_step(tri: &'static TripleHooks, lev: &'static LeverHooks) -> u64 {
    march_of(&machine(lev, tri), 0.04, MarchScope::DEFAULT).len() as u64
}
fn ex_manifold(tri: &'static TripleHooks, _: &'static LeverHooks) -> u64 {
    let (_, p) = manifold_point();
    v_base_of(tri, &p).0.to_bits()
}
fn ex_sibling_inc(tri: &'static TripleHooks, lev: &'static LeverHooks) -> u64 {
    let (sib, _, _) = machine(lev, tri).triple_rig(&rig_arm());
    sib.fuel.inner.stator.inc.is_some() as u64
}
fn ex_sibling_in_a_phi_scope(tri: &'static TripleHooks, lev: &'static LeverHooks) -> u64 {
    let m = machine(lev, tri);
    let _r = RefScope::set(&m.fuel.inner, Some("phi"));
    let (sib, _, _) = m.triple_rig(&rig_arm());
    sib.fuel.inner.stator.lim.is_some() as u64
}

type Exercise = fn(&'static TripleHooks, &'static LeverHooks) -> u64;

/// The ten cells, each with the tables that inject it and the exercise that reveals it.
///
/// The last column is § 5.26 (ii)'s *"how a dispatch gate can see it"* column, transcribed — it is
/// the PRIOR, and it is the only typed thing in this section. Everything it is compared against is
/// measured.
#[allow(clippy::type_complexity)]
fn cells() -> Vec<(&'static str, &'static TripleHooks, &'static LeverHooks, Exercise, bool)> {
    vec![
        ("stator_leg", &P_STATOR_LEG, &R69, ex_five, false),
        ("lagged_stator", &P_LAGGED_STATOR, &R69, ex_five, false),
        ("clamp_v", &P_CLAMP_V, &R69, ex_v_max, false),
        ("check_v0", &P_CHECK_V0, &R69, ex_v0_positive, false),
        ("rk4_floor", &P_RK4_FLOOR, &R69, ex_coarse_step, false),
        ("solve_v", &P_SOLVE_V, &R69, ex_five, true),
        ("manifold_v", &P_MANIFOLD_V, &R69, ex_manifold, true),
        ("triple_rig", &P_TRIPLE_RIG, &R69, ex_sibling_inc, true),
        ("with_ref", &C_WITH_REF, &R69, ex_sibling_in_a_phi_scope, false),
        ("at_lever", &R69_TRIPLE, &P_AT_LEVER, ex_sibling_inc, false),
    ]
}

fn measured_shapes() -> Vec<(&'static str, Shape, bool)> {
    cells().into_iter()
        .map(|(name, tri, lev, ex, predicted_panic)| {
            let shipped = run_exercise(|| ex(&R69_TRIPLE, &R69));
            let injected = run_exercise(|| ex(tri, lev));
            (name, classify(&shipped, &injected), predicted_panic)
        })
        .collect()
}

/// **P2, SETTLED: ten cells, ten observable — and the count is EMITTED, not typed.**
///
/// A cell that could not be broken is reported UNOBSERVABLE here rather than quietly re-gated on
/// something else — slice Z step 5's shape.
#[test]
fn the_ten_cells_are_all_observable() {
    let shapes = measured_shapes();
    let silent: Vec<&str> = shapes.iter().filter(|(_, s, _)| *s == Shape::Silent)
        .map(|(n, _, _)| *n).collect();
    assert!(silent.is_empty(), "UNOBSERVABLE cells: {silent:?}");
    let observed = shapes.iter().filter(|(_, s, _)| *s != Shape::Silent).count();
    assert_eq!(observed, shapes.len(), "P2 over {} cells", shapes.len());
}

/// **THE SHAPES, MEASURED AGAINST § 5.26 (ii)'s PREDICTION — AND THE PREDICTION IS WRONG.**
///
/// The pre-flight named three cells that break by PANIC (`solve_v`, `manifold_v`, `triple_rig`)
/// and one that breaks only through its panic STRING (`rk4_floor`). Two of the three do not panic
/// at all, for the same reason both times: rung 68's bodies for those two read no field that an
/// incidence arming leaves empty — one BUILDS a limiter from the map, the other is `V(g, q)[0]`.
///
/// The assertion is per NAME and against a MEASURED shape, so nothing here counts a label it wrote
/// itself. The disagreeing names are emitted in the message.
#[test]
fn two_of_the_predicted_panics_are_not_panics() {
    let shapes = measured_shapes();
    let disagree: Vec<(&str, Shape, bool)> = shapes.iter().cloned()
        .filter(|(_, s, predicted)| (*s == Shape::Panic) != *predicted)
        .collect();
    assert!(!disagree.is_empty(),
            "s 5.26 (ii)'s shape column is refuted somewhere; measured {shapes:?}");
    for name in ["triple_rig", "manifold_v"] {
        let (_, shape, predicted) = shapes.iter().find(|(n, _, _)| *n == name).unwrap();
        assert!(*predicted, "the pre-flight predicted a PANIC for {name}");
        assert_eq!(*shape, Shape::Value,
                   "{name} breaks by VALUE: rung 68's body reads no field an incidence arming \
                    leaves empty. Disagreements over the whole table: {disagree:?}");
    }
    // And the one cell whose ONLY observable is the text of its own refusal, which is why a
    // fact-of-a-panic gate would have left it ungated.
    let (_, floor, _) = shapes.iter().find(|(n, _, _)| *n == "rk4_floor").unwrap();
    assert_eq!(*floor, Shape::Message, "`rk4_floor` fires on BOTH tables and differs only in text");
}

/// **THE CONTROL: the same ten exercises with NOTHING injected must be SILENT.**
///
/// A dispatch gate's whole content is a difference, so an exercise that reports one against the
/// SHIPPED table is measuring itself. This slice shipped two gates that could not fail at its own
/// step 1 (§ 5.26.1 (b), (i)) and both were caught by hand; running the classification with the
/// shipped tables on both sides makes the check structural, and it costs one call.
#[test]
fn the_ten_exercises_are_silent_when_nothing_is_injected() {
    let noisy: Vec<(&str, Shape)> = cells().into_iter()
        .map(|(name, _, _, ex, _)| {
            let a = run_exercise(|| ex(&R69_TRIPLE, &R69));
            let b = run_exercise(|| ex(&R69_TRIPLE, &R69));
            (name, classify(&a, &b))
        })
        .filter(|(_, s)| *s != Shape::Silent)
        .collect();
    assert!(noisy.is_empty(), "these exercises report a difference with none injected: {noisy:?}");
    // The coverage bar, and it is the SAME list on both sides rather than a lone literal.
    assert_eq!(cells().len(), measured_shapes().len(),
               "the control covers every cell the tally does");
}
