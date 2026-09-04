//! SLICE AF step 2 — **THE SIX DEMAND LAWS, AND THE ONE THING THE COORDINATE ACTUALLY CHANGES.**
//!
//! `_applied_demand`, `_demand_target`, `_demand_reference`, `_demand_tau`, `_demand_authority`
//! and `_demand_laws` — § 5.30 (v)'s step-2 list as re-cut by step 1 § (c).
//!
//! # AN AST CENSUS DECIDED THIS FILE's SHAPE BEFORE ANY OF IT WAS WRITTEN
//!
//! All six methods have **exactly one definer** across `engine.py`'s 58 classes, so step 2 adds no
//! `TripleHooks` field and pays none of step 1's width toll. Rung 72's siblings score the same way
//! (`_applied_clip`, `_authority`, `_quad_laws` are single-definer too) and are plain functions for
//! the same reason. The only DISPATCHED call inside this step's code is `_cap_fuel` — three
//! definers, rungs 74/78/79 — and [`f_reaches_cap_fuel_through_the_table`] drives it.
//!
//! # THE CONSTRAINT THAT DECIDES WHETHER THESE GATES ARE WORTH ANYTHING
//!
//! § 5.30 (i) measured **two disjoint regions on one plant**: on `_with_coord`'s path
//! `cap <= mf_sched` at 1 040 of 1 040 and 624 of 624 calls, while inside `_coord_march`
//! `cap > mf_sched` at 120 of 2 730 and 139 of 2 732, max ratio **1.3039**. The latch is
//! arithmetically invisible in the first region. **So any gate on `demand_target` that drives only
//! the interior-filter arm is VOID and a mutation sweep reports it green** — which is step 1
//! § (h)'s defect one method over. The truth table below therefore drives HAND-PICKED floats, where
//! the over-schedule column is reachable by construction rather than by luck.
//!
//! # WHAT THIS FILE DELIBERATELY DOES NOT GATE — pre-registered, so step 5 does not hunt them
//!
//! **`_with_coord` BY VALUE**, unchanged from step 1: § 5.30 (i)'s P5. Gating `demand_target`'s
//! truth table is the READER's law, not the scope's observability through a march, and it does not
//! falsify P5 in either direction.
//!
//! **The `b_state`/`v_state` boundary on `C`, `V` and `R`.** Only `F` reaches a spy, through the
//! injected `cap_fuel` table. Booked as a known blind spot rather than left to be found.
//!
//! **`V`'s copy of the demand-as-fuel spelling.** [`c_and_v_read_the_applied_demand`] drives `C`,
//! which needs only the bleed limiter; `V` carries the identical expression and is driven with it.
//!
//! # TWO PREDICTED SURVIVORS, WITH THEIR PROOFS WRITTEN BEFORE THE SWEEP RAN
//!
//! **The two `if`s of [`applied_demand`] swapped** is provably the same function on every input,
//! NaN included — each `if` fires iff its operand is strictly below the running minimum, and `<` is
//! false in both directions against a NaN. A survivor there is the port being right.
//!
//! **`F`/`R`'s regime label read off the LATCHED target instead of the cap** is a NO-OP, and
//! [`the_regime_label_cannot_be_moved_by_the_coordinate`] asserts the invariance that makes it one:
//! under `clip`/`demand` `tgt == cap`, and under `demand-latched`
//! `min(mf_sched, cap) < mf_sched  <=>  cap < mf_sched`. It was nominated as a must-kill and is
//! recorded here as inert instead, because a survivor is a question and this one has an answer.
//!
//! # A BOOKING FOR STEPS 3 AND 4 — **ALL SIX FUNCTIONS HAVE ZERO READERS IN `src` TODAY**
//!
//! Nothing in the library calls them yet; this file is their only caller, because rung 74's march
//! lands at step 3 and its readers at step 4. **So those steps must CALL them, never re-derive the
//! min-select or the latch inline** — a second spelling would leave all thirteen gates below green
//! while the march ran a different function. Booked here rather than left to be discovered.
//!
//! **NOTHING HERE READS A GOLDEN.** Every assertion is a hand-picked float, a same-run difference,
//! or a panic.

use std::cell::Cell;

use turbojet::applied_reference::build_applied_reference_cascade;
use turbojet::bleed_transient::{LeverArm, LeverArming};
use turbojet::demand_coordinate::{
    applied_demand, build_demand_coordinate_cascade, demand_authority, demand_laws,
    demand_reference, demand_tau, demand_target, CoordScope, DEMAND_AUTH_TOL, LAG_COORD_CLIP,
    LAG_COORD_DEMAND, LAG_COORD_LATCHED, R74, R74_FUEL, R74_STATOR, R74_TRIPLE, R74_TWO,
};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, Authority, AsymmetricLag, Floor, FuelTransientCore, SurgeLimiter,
};
use turbojet::gas::{Abort, Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::shared_actuator::{authority, build_shared_actuator_cascade, REF_LAW_DEFAULT};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::{LegRegime, StatorLimiter, TripleHooks};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------- the grid
//
// `tests/test_rung74.py`'s module constants — `slice_af_cells.rs`'s copy, plus the stator pair the
// laws file needs and that file did not. This slice adds no constant of its own.
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const B: f64 = 0.10;
const PHI: f64 = 0.80;
const V_MAX: f64 = 0.20;
/// `PHI / FLOOR - 1.0` — the expression Python spells, never a typed decimal.
const SM: f64 = PHI / FLOOR - 1.0;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TAU_ATT: f64 = 0.05;
const TAU_REL: f64 = 0.15;
const TT4_MAX: f64 = 1200.0;

/// The operating point every law gate is driven at — **MEASURED, and the first draft's guess sat
/// inside a shipped refusal.**
///
/// The draft copied `slice_af_cells.rs`'s `(1.0, 1.0, 0.02)`, where that file's only call was
/// `_cap_fuel` with ONE cap armed through an accel schedule. Through the SURGE floor at the design
/// speed pair, `_cap_free` cannot bracket at all and `F` raises rung 74's own *the UNFLOORED cap is
/// unreachable above mf_sched* — the same shipped guard § 5.30 (i)'s third `phi_lim` arm hit. Four
/// gates aborted there and a fifth read a valve sitting on its stop.
///
/// A Python sweep over the speed pair and the schedule (five pairs x six schedules) picked this
/// point instead. It is the only cell in that grid where **all four laws return, both fuel-side
/// legs are RIDING, and the valve and the stator are OFF their stops** — the last of which the
/// flatness gate needs, because two saturated zeros satisfy an equality for the wrong reason.
const A: f64 = 0.85;
const H: f64 = 0.90;
const MF: f64 = 0.040;

/// **§ 5.30 (i)'s MEASURED over-schedule ratio, and it is a reference from OUTSIDE the code under
/// test.** The pre-flight measured `max(cap / mf_sched) = 1.3039` inside `_coord_march`, which
/// independently reproduces `DemandCoordinateTransient`'s own docstring — *`1.303 * mf_sched` at
/// the start of the ramp*.
///
/// **WHAT IS MEASURED IS THE RATIO, NOT `OVER * MF`.** The truth table needs only a `cap` strictly
/// above the schedule, and this is the one such ratio the shipped plant is known to reach; the
/// product at this file's `MF` is not itself a march point and is not claimed to be one.
const OVER: f64 = 1.3039;

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

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

fn valve() -> BleedLimiter { BleedLimiter::from_margin_tau(&lp_map(), B, SM, Some(TAU)) }
fn surge() -> SurgeLimiter { SurgeLimiter::from_margin(&lp_map(), Spool::Lp, SM) }
fn lag() -> AsymmetricLag { AsymmetricLag::new(TAU_ATT, TAU_REL) }

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

/// The arming every law gate uses: the valve (so `C` has a limiter) and the stator loop (so `V`
/// has one). `slice_af_cells.rs` needed neither and armed neither.
fn arm() -> LeverArm {
    LeverArm {
        bleed_lim: Some(valve()),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU_S))),
        ..Default::default()
    }
}

fn demand(a: &LeverArm) -> ScheduledStatorCore {
    full_of(build_demand_coordinate_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, a))
}

fn applied(a: &LeverArm) -> ScheduledStatorCore {
    full_of(build_applied_reference_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, a))
}

/// Rung 72 — the grandparent, whose `_ref_law` class default is the OTHER one. It is the ONLY
/// machine in this file whose reference is disarmed, and it is disarmed **by its builder**.
fn shared(a: &LeverArm) -> ScheduledStatorCore {
    full_of(build_shared_actuator_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, a))
}

/// A flat `Wf/pt3` schedule, built by hand — `slice_af_cells.rs`'s, for its reason: a derived one
/// would make a dispatch gate depend on the running line too.
fn flat_schedule(kappa: f64) -> AccelSchedule {
    AccelSchedule { margin: 0.0, n_h: vec![0.5, 1.5], kappa: vec![kappa, kappa] }
}

/// Rebuild a machine with an injected third-loop table — `slice_af_cells.rs`'s `with_triple`.
fn with_triple(
    core: &ScheduledStatorCore, a: &LeverArm, triple: &'static TripleHooks,
) -> ScheduledStatorCore {
    let c = full_of(ScheduledStatorTransient::with_ref_tables(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(),
        a.stator, &R74_TWO, &R74_STATOR, &R74_FUEL, &R74,
        LeverArming { bleed: a.bleed, sched: a.bleed_sched, lim: a.bleed_lim },
        triple, a.stator_lim, a.stator_inc));
    c.fuel.inner.ref_law.set(core.fuel.inner.ref_law.get());
    c.fuel.inner.lag_coord.set(core.fuel.inner.lag_coord.get());
    c
}

// =============================================================================================
// 1 — `_applied_demand`: MIN-SELECT, AND THE FOLD THAT IS NOT `f64::min`
// =============================================================================================

/// **PYTHON's LEFT FOLD, AND A NaN SEED IS WHAT SEPARATES IT FROM `f64::min`.**
///
/// `min(mf_sched, wf, wr)` seeds at `mf_sched` and moves only on a strict `<`, so a NaN operand is
/// never selected — and a NaN SEED is never displaced, because `wf < NaN` is false. `f64::min` has
/// the opposite rule: it returns the non-NaN operand. So `applied_demand(NaN, 1.0, 2.0)` is `NaN`
/// here and `1.0` under `mf_sched.min(wf).min(wr)`, which is the only input on which the two
/// spellings can be told apart at all.
///
/// The ordinary rows are there because the NaN row alone would pass for a body that returned its
/// first argument.
#[test]
fn applied_demand_is_pythons_left_fold_and_a_nan_seed_survives_it() {
    // The schedule wins.
    assert_eq!(applied_demand(0.02, 0.05, 0.09), 0.02);
    // The fuel leg wins.
    assert_eq!(applied_demand(0.02, 0.01, 0.09), 0.01);
    // The governor wins.
    assert_eq!(applied_demand(0.02, 0.05, 0.003), 0.003);
    // Both legs below: the LOWEST demand wins, which is min-select one level down.
    assert_eq!(applied_demand(0.02, 0.011, 0.010), 0.010);

    // THE DISCRIMINATOR. A NaN in a leg is not selected...
    assert_eq!(applied_demand(0.02, f64::NAN, 0.09), 0.02);
    // ...and a NaN SEED is not displaced, where `f64::min` would return `1.0`.
    assert!(applied_demand(f64::NAN, 1.0, 2.0).is_nan(),
            "`min(NaN, 1.0, 2.0)` is NaN in Python and 1.0 under an `f64::min` chain — this row \
             is the whole reason the fold is spelled out");
}

// =============================================================================================
// 2 — `_demand_target`: A THREE-VALUED TAG READ BY A TWO-VALUED TEST
// =============================================================================================

/// **THE LATCH's TRUTH TABLE — THREE TAGS x TWO REGIONS, AND EXACTLY ONE CELL MOVES.**
///
/// The coordinate is set through [`CoordScope`], which is the shipped setter and the only public
/// way to write the carrier, so the tag under test arrives the way the plant supplies it.
///
/// | | `cap < mf_sched` | `cap > mf_sched` |
/// |---|---|---|
/// | `clip` | `cap` | `cap` |
/// | `demand` | `cap` | `cap` |
/// | `demand-latched` | `cap` | **`mf_sched`** |
///
/// `clip` and `demand` are asserted **equal to each other**, not merely each correct: that is the
/// by-construction indistinguishability § 5.30 (i) found, and asserting it is what makes a
/// mutation widening the arm to `!= LAG_COORD_CLIP` fail rather than pass silently.
///
/// **THE OVER-SCHEDULE COLUMN IS THE POINT OF THE GATE.** It is unreachable on the interior-filter
/// arm the shipped reader walks (0 of 1 040 and 0 of 624), so a gate driven through a march there
/// would agree with a body that had no latch at all. [`OVER`] puts it where `_coord_march`
/// measured the plant actually going.
#[test]
fn the_latch_truth_table_is_three_tags_by_two_regions_and_one_cell_moves() {
    let a = arm();
    let m = demand(&a);
    let t = &m.fuel.inner;

    let under = 0.9 * MF;
    let over = OVER * MF;

    let read = |coord: &'static str, cap: f64| -> f64 {
        let _s = CoordScope::set(t, coord);
        demand_target(t, cap, MF)
    };

    // THE UNDER-SCHEDULE COLUMN — all three tags agree, and the latch is inert.
    for c in [LAG_COORD_CLIP, LAG_COORD_DEMAND, LAG_COORD_LATCHED] {
        assert_eq!(read(c, under), under, "under the schedule the latch cannot bind, on any tag");
    }

    // THE OVER-SCHEDULE COLUMN — and this is where the three tags become two.
    assert_eq!(read(LAG_COORD_CLIP, over), over, "`clip` passes the cap through");
    assert_eq!(read(LAG_COORD_DEMAND, over), over, "`demand` passes it through TOO");
    assert_eq!(read(LAG_COORD_CLIP, over), read(LAG_COORD_DEMAND, over),
               "AND THE TWO ARE EQUAL — a three-valued tag read by a two-valued test, which is \
                § 5.30 (i)'s finding stated as an assertion rather than as a comment");
    assert_eq!(read(LAG_COORD_LATCHED, over), MF,
               "the ONE cell that moves: the latch caps the target at the schedule, which is \
                rung 52's `max(0, .)` seen from the other coordinate");

    // AND THE INSTRUMENT CAN SEE — the two columns are not the same number.
    assert_ne!(read(LAG_COORD_LATCHED, over), read(LAG_COORD_LATCHED, under));

    // THE SCOPE RESTORED WHAT IT DISPLACED, so the table above measured three tags and not one.
    assert_eq!(t.lag_coord.get(), LAG_COORD_CLIP,
               "rung 74's class default is `clip`, and every scope above has been dropped");
}

// =============================================================================================
// 3 — `_demand_reference`: RUNG 73's HOOK, TERM FOR TERM, IN THE NEW COORDINATE
// =============================================================================================

/// **THE REFERENCE ARMS FROM THE BUILDER, AND RUNG 72's BUILDER DISARMS IT.**
///
/// § 5.30 (viii) item 1 — *what supplies the value under test?* — applied to the arming rather than
/// to the arithmetic: the `ref_law` is **not poked in by this test**. Rung 74's cascade builder
/// writes `"applied"` because Python's class attribute is inherited through the subclass; rung 72's
/// writes `"sched"`. Both machines are handed the identical three floats.
///
/// This is step 1 § (f)'s builder gate from the READER's side: a rung-74 builder that dropped the
/// overwrite would hand back a machine passing its own refusals while marching rung 72's reference,
/// and this gate is where that shows up as a value.
#[test]
fn the_reference_arms_from_the_builder_and_rung_72s_default_disarms_it() {
    let a = arm();
    let (cap, w_own, mf_app) = (0.011_f64, 0.013_f64, 0.010_f64);

    let m74 = demand(&a);
    assert_eq!(m74.fuel.inner.ref_law.get(), turbojet::applied_reference::REF_LAW_APPLIED,
               "the BUILDER armed it — nothing in this test wrote the field");
    assert_eq!(demand_reference(&m74.fuel.inner, cap, w_own, mf_app), (w_own + cap) - mf_app,
               "armed: rung 73's hook, term for term, in demand coordinates");

    let m72 = shared(&a);
    assert_eq!(m72.fuel.inner.ref_law.get(), REF_LAW_DEFAULT,
               "rung 72's builder leaves the class default, which is the OTHER law");
    assert_eq!(demand_reference(&m72.fuel.inner, cap, w_own, mf_app), cap,
               "disarmed: path 1 returns the cap and the formula never runs");

    // THE INSTRUMENT CAN SEE — the two answers are not the same number at these arguments.
    assert_ne!(demand_reference(&m74.fuel.inner, cap, w_own, mf_app),
               demand_reference(&m72.fuel.inner, cap, w_own, mf_app));

    // AND RUNG 73's OWN MACHINE IS ARMED TOO, which is what "inherited" means.
    assert_eq!(applied(&a).fuel.inner.ref_law.get(),
               turbojet::applied_reference::REF_LAW_APPLIED);
}

/// **THE FLOAT-IDENTITY BRANCH IS NOT THE FORMULA, AND TWO SEPARATE POINTS SAY SO.**
///
/// Rung 73's reason carries: when the leg HOLDS, `mf_app == w_own` and the method must return the
/// cap ITSELF, because `w_own + cap - w_own` is not `cap` in binary floating point and the
/// difference lands on the authoritative leg's own diagonal.
///
/// * **The `1e16` point** shows the branch is load-bearing: the formula there returns `0.0` where
///   the cap is `1.0`.
/// * **The `1e-13` point** shows the test must be an EXACT `==`: at `mf_app = w_own + 1e-13` the
///   two are unequal by less than any plausible epsilon, so an epsilonized test takes the identity
///   branch and returns `1.0` where the shipped body returns `1.0 - 1e-13`. Without this row an
///   epsilon mutation survives — which is why the row exists.
#[test]
fn the_float_identity_branch_is_not_the_formula_and_two_points_say_so() {
    let a = arm();
    let m = demand(&a);
    let t = &m.fuel.inner;

    // THE HOLDING LEG, at a scale where the formula visibly loses the cap.
    let big = 1e16_f64;
    assert_eq!(demand_reference(t, 1.0, big, big), 1.0,
               "the identity branch returns the cap ITSELF");
    assert_eq!((big + 1.0) - big, 0.0,
               "and the formula at the same arguments returns 0.0 — which is what the branch is \
                there to prevent landing on the diagonal");

    // THE EPSILON POINT — unequal, but by less than any epsilon a tidy-up would choose.
    let (w_own, mf_app) = (1.0_f64, 1.0_f64 + 1e-13);
    assert_ne!(w_own, mf_app, "unequal as floats");
    assert!((w_own - mf_app).abs() < 1e-12, "and closer than a 1e-12 epsilon would admit");
    let got = demand_reference(t, 1.0, w_own, mf_app);
    assert_eq!(got, (w_own + 1.0) - mf_app,
               "the EXACT `==` sends this through the formula");
    assert_ne!(got, 1.0,
               "an epsilonized test would have returned the cap here, and this is the row that \
                separates the two");
}

/// **PYTHON's ASSOCIATION IS PINNED: `(w_own + cap) - mf_app`, NEVER `cap + (w_own - mf_app)`.**
///
/// Rung 73's probe L4 measured the two disagreeing, and the rearrangement is exactly what a later
/// tidy-up writes. The arguments here are chosen so the two orders differ by a whole unit: at
/// `w_own = 1e16` the spacing is `2.0`, so `1e16 + 1.0` rounds back to `1e16` (ties to even) while
/// `w_own - mf_app` is exact.
#[test]
fn pythons_association_is_pinned_and_the_rearrangement_is_a_different_float() {
    let a = arm();
    let m = demand(&a);
    let t = &m.fuel.inner;

    let (cap, w_own, mf_app) = (1.0_f64, 1e16_f64, 1e16_f64 + 2.0);
    assert_ne!(w_own, mf_app, "not the identity branch");

    let shipped = (w_own + cap) - mf_app;
    let rearranged = cap + (w_own - mf_app);
    assert_ne!(shipped, rearranged,
               "the two associations are different floats at these arguments — without this the \
                gate below could not fail");
    assert_eq!(demand_reference(t, cap, w_own, mf_app), shipped,
               "and the port spells Python's");
}

// =============================================================================================
// 4 — `_demand_tau`: RUNG 52's LAG WITH THE ARGUMENTS SWAPPED
// =============================================================================================

/// **THE SWAP IS THE WHOLE POINT, AND KEEPING THE SHIPPED ORDER IS A 3x CLOCK ERROR TOWARD LESS
/// PROTECTION.**
///
/// Attack in clip coordinates is `required > g`; substituting `w = mf_sched - g` and
/// `cap = mf_sched - required` gives `required > g  <=>  cap < w`. So the demand goes in the
/// `required` slot and the cap in the `g` slot.
///
/// **THE EXPECTATIONS ARE THE LITERAL CONSTANTS THIS TEST SET**, never a second call to
/// [`AsymmetricLag::tau`](turbojet::fuel_transient::AsymmetricLag::tau) — that is the code under
/// test, and step 1's `CAP_GROW` gate is the recorded cost of taking a reference from it. Both
/// sides are asserted, because a one-sided gate passes for a body that returns a constant.
///
/// The wrong-order control is asserted to give the OTHER constant at the same point, so the gate is
/// known to discriminate rather than merely to agree.
#[test]
fn demand_tau_swaps_the_arguments_and_both_sides_are_pinned_to_literals() {
    let l = lag();
    // The rig's ratio, and the size of the error at stake. **NOT `TAU_REL == 3.0 * TAU_ATT`** —
    // `3.0 * 0.05` is `0.15000000000000002`, so the exact form fails on a true statement about the
    // rig. The two law assertions below ARE exact; only this scene-setting line is not.
    assert!((TAU_REL / TAU_ATT - 3.0).abs() < 1e-12,
            "release is 3x attack on this rig, to within the float");

    // ATTACK: `cap < w`. The leg is being pulled DOWN toward a cap below where it sits.
    let (cap_a, w_a) = (0.010_f64, 0.015_f64);
    assert!(cap_a < w_a);
    assert_eq!(demand_tau(&l, cap_a, w_a), TAU_ATT,
               "attack selects the FAST constant, which is the whole protective content");

    // RELEASE: `cap > w`.
    let (cap_r, w_r) = (0.015_f64, 0.010_f64);
    assert!(cap_r > w_r);
    assert_eq!(demand_tau(&l, cap_r, w_r), TAU_REL);

    // THE CONTROL — the shipped argument ORDER, at the SAME attack point, selects the slow one.
    assert_eq!(l.tau(cap_a, w_a), TAU_REL,
               "a port that kept `lag.tau(cap, w)` would run attack on `tau_rel`: a 3x clock \
                error in the direction that SLOWS protection, and one that reads as a finding");
    assert_ne!(demand_tau(&l, cap_a, w_a), l.tau(cap_a, w_a),
               "so the two orders are distinguishable here, and this gate distinguishes them");
}

// =============================================================================================
// 5 — `_demand_authority`: RUNG 72's LABEL WITH BOTH SENSES INVERTED
// =============================================================================================

/// **BOTH SENSES INVERT, SO DELEGATING TO RUNG 72's `authority` WOULD BE WRONG — AND THE GATE
/// SHOWS THE TWO DISAGREEING RATHER THAN ASSERTING THEY DIFFER.**
///
/// | | rung 72, on clips | rung 74, on demands |
/// |---|---|---|
/// | dormant | `gf <= tol && gr <= tol` | `wf >= mf_sched - tol && wr >= mf_sched - tol` |
/// | holder | `fuel` iff `gf > gr` | `fuel` iff `wf < wr` |
///
/// Who holds the actuator is who DEMANDS LEAST, and `dormant` is now a statement about the
/// SCHEDULE rather than about a state sitting on a stop.
#[test]
fn both_senses_invert_from_rung_72_and_delegation_would_be_wrong() {
    // THE HOLDER SENSE. The fuel leg demands least, so it holds.
    assert_eq!(demand_authority(0.010, 0.018, MF), Authority::Fuel);
    assert_eq!(authority(0.010, 0.018), Authority::Gov,
               "rung 72 reads the SAME pair as the governor's, because there the bigger CUT wins");

    // The governor demands least.
    assert_eq!(demand_authority(0.018, 0.010, MF), Authority::Gov);
    assert_eq!(authority(0.018, 0.010), Authority::Fuel);

    // THE DORMANT SENSE. Neither leg is below the schedule.
    assert_eq!(demand_authority(MF, 1.5 * MF, MF), Authority::Dormant,
               "`>=` the schedule, so nobody is asking for less than it is offering");
    assert_eq!(authority(MF, 1.5 * MF), Authority::Gov,
               "rung 72 reads two large CUTS at the same numbers, which is not dormant at all");

    // ...and two zero demands are NOT dormant here, where at rung 72 they are the definition of it.
    assert_eq!(demand_authority(0.0, 0.0, MF), Authority::Tie);
    assert_eq!(authority(0.0, 0.0), Authority::Dormant);

    // THE TIE, below the schedule.
    assert_eq!(demand_authority(0.010, 0.010, MF), Authority::Tie);
}

/// **`dormant` IS TESTED BEFORE `tie`, AND A POINT THAT IS BOTH RETURNS `dormant`.**
///
/// The branch ORDER is the content of this method: two demands that are equal AND at the schedule
/// satisfy both tests, and reversing the two lines is a silent relabel that no aggregate over a
/// march would show. The tolerance is straddled rather than restated — one point just inside it and
/// one just outside.
#[test]
fn dormant_is_tested_before_tie_and_the_tolerance_is_straddled() {
    // BOTH conditions hold. Branch order alone decides.
    assert_eq!(demand_authority(MF, MF, MF), Authority::Dormant,
               "equal AND at the schedule: `dormant` wins because it is tested first");

    // JUST INSIDE the dormant tolerance — still dormant.
    let inside = MF - 0.5 * DEMAND_AUTH_TOL;
    assert_eq!(demand_authority(inside, inside, MF), Authority::Dormant);

    // JUST OUTSIDE it — and now the tie branch is reachable, which is how we know the point above
    // was decided by the ORDER and not by the dormant test being the only one that matched.
    let outside = MF - 10.0 * DEMAND_AUTH_TOL;
    assert_eq!(demand_authority(outside, outside, MF), Authority::Tie);
}

// =============================================================================================
// 6 — `_demand_laws`: THE FOUR CLOSURES
// =============================================================================================

/// **`C` READS THE APPLIED DEMAND AS THE FUEL, NOT RUNG 72's `mf_sched - clip` — AND IT IS FLAT IN
/// THE MASKED LEG.**
///
/// This is the one place the coordinate changes an ARGUMENT rather than a label, and the two
/// spellings are separated here by a pair of points that share an applied demand and nothing else:
///
/// * `(wf, wr) = (0.8 mf, 2.0 mf)` and `(0.8 mf, 0.9 mf)` have the SAME `applied_demand` — `0.8 mf`
///   — so the shipped body must return the SAME valve position. Under rung 72's spelling
///   (`mf_sched - max(wf, wr)`) the two would be `mf - 2 mf` and `mf - 0.9 mf`, which are a clamped
///   `1e-9` and `0.1 mf`: different plants. **The sweep killed that mutation two lines EARLIER than
///   this sentence predicted** — at the `.expect` below, because the `1e-9` clamp makes the valve
///   solve ABORT rather than return a different position. Verdict right, route wrong, and it is
///   corrected here rather than rounded up: step 1's sharpest lesson was a correct survivor
///   defended by a code comment that was false, and the comment shipped.
/// * `(0.7 mf, 2.0 mf)` has a DIFFERENT applied demand, and the answer must MOVE — without which
///   the first assertion would pass for a body that ignored its arguments entirely.
///
/// The flatness in the masked leg is § 1's whole reason the triangularity survives a change of
/// coordinate: `min()` where rung 72 had `max()`.
#[test]
fn c_and_v_read_the_applied_demand() {
    let a = arm();
    let m = demand(&a);
    let f = flight();
    let floor = Floor::Phi(surge());
    let laws = demand_laws(&m, &f, A, H, MF, None, Some(&floor), TT4_MAX);

    let masked_far = (laws.c)(0.8 * MF, 2.0 * MF, 0.0).expect("the valve solve closes here");
    let masked_near = (laws.c)(0.8 * MF, 0.9 * MF, 0.0).expect("the valve solve closes here");
    assert_eq!(applied_demand(MF, 0.8 * MF, 2.0 * MF), applied_demand(MF, 0.8 * MF, 0.9 * MF),
               "the two points share an applied demand — stated, not assumed");
    assert_ne!(masked_far.0, 0.0,
               "**AND THE VALVE IS OFF ITS STOP.** The first draft of this gate ran at a point \
                where every valve reading was 0.0, so the bit-equality below held between TWO \
                SATURATED ZEROS — an equality satisfied by an actuator that was not solving at \
                all. Only the `assert_ne` control further down could see it, and this line is \
                why it cannot recur.");
    assert_eq!(masked_far.0.to_bits(), masked_near.0.to_bits(),
               "so `C` returns the same valve position BIT FOR BIT: it sees the masked leg only \
                through a function that is FLAT in it");

    let moved = (laws.c)(0.7 * MF, 2.0 * MF, 0.0).expect("the valve solve closes here");
    assert_ne!(moved.0, masked_far.0,
               "THE INSTRUMENT CAN SEE — a different applied demand moves the answer, so the \
                agreement above is flatness and not a body that ignores its arguments");

    // AND `V` CARRIES THE IDENTICAL EXPRESSION.
    let v_far = (laws.v)(0.8 * MF, 2.0 * MF, 0.0).expect("the stator solve closes here");
    let v_near = (laws.v)(0.8 * MF, 0.9 * MF, 0.0).expect("the stator solve closes here");
    assert_eq!(v_far.0.to_bits(), v_near.0.to_bits());
    let v_moved = (laws.v)(0.7 * MF, 2.0 * MF, 0.0).expect("the stator solve closes here");
    assert_ne!(v_moved.0, v_far.0);
}

/// **`F` REACHES `_cap_fuel` THROUGH THE TABLE**, so an inherited rung-74 reader run on a rung-78
/// or rung-79 machine takes that machine's body.
///
/// `_cap_fuel` has THREE definers (rungs 74, 78, 79), measured by AST census over all 58 classes,
/// which is what makes it a cell at all. A port calling `r74_cap_fuel` directly would compile, pass
/// every value gate at this rung and break two later ones silently — slice AE's recorded defect,
/// and step 1's own `sensed_cap` gate one level up.
///
/// The injected body also **spies the two march-state guards**, which is the only place in this
/// file that can see them: it asserts both are `Some` with the values `F` was handed, and the gate
/// asserts both are back to `None` afterwards.
#[test]
fn f_reaches_cap_fuel_through_the_table() {
    const SENTINEL: f64 = 0.987_654_321;
    thread_local! {
        static SEEN: Cell<(Option<f64>, Option<f64>)> = const { Cell::new((None, None)) };
    }
    #[allow(clippy::too_many_arguments)]
    fn injected_cap_fuel(
        ft: &FuelTransientCore, _: &FlightCondition, _: f64, _: f64, _: f64,
        _: Option<&AccelSchedule>, _: Option<&Floor>, _: Option<f64>,
    ) -> Result<f64, Abort> {
        SEEN.with(|s| s.set((ft.inner.b_state.get(), ft.inner.v_state.get())));
        Ok(SENTINEL)
    }
    static INJ: TripleHooks = TripleHooks { cap_fuel: injected_cap_fuel, ..R74_TRIPLE };

    let a = arm();
    let m = with_triple(&demand(&a), &a, &INJ);
    let f = flight();
    let floor = Floor::Phi(surge());
    let laws = demand_laws(&m, &f, A, H, MF, None, Some(&floor), TT4_MAX);

    // **THE ARGUMENT ORDER HERE IS LOAD-BEARING AND THE FIRST DRAFT HAD IT WRONG.** With
    // `(wf, wr) = (0.9 mf, 1.1 mf)` the applied demand is `0.9 mf`, which EQUALS `F`'s own `w_own`
    // — so the reference takes its float-identity branch and returns the cap untouched, and the
    // value assertion below would have been checking the formula against a body that never ran it.
    // `wf` is put ABOVE the schedule so `wr` supplies the demand and `w_own != mf_app`.
    let (q, v) = (0.037_f64, -0.041_f64);
    let (w, reg) = (laws.f)(1.1 * MF, 0.9 * MF, q, v).expect("the injected cap needs no solve");

    // THE DISPATCH.
    assert_eq!(SEEN.with(|s| s.get()), (Some(q), Some(v)),
               "`F` sets BOTH march states around the cap solve — rung 68's table, and a law that \
                trials neither actuator must see both");
    assert_eq!(m.fuel.inner.b_state.get(), None);
    assert_eq!(m.fuel.inner.v_state.get(), None,
               "and both are cleared on the way out, which is Python's `finally`");

    // THE VALUE CAME FROM THE INJECTED BODY, through the reference. `ref_law` is `applied` and
    // `mf_app != w_own` here, so the reference is the formula and not a pass-through.
    let ma = applied_demand(MF, 1.1 * MF, 0.9 * MF);
    assert_ne!(ma, 1.1 * MF, "the reference is on its FORMULA branch, not its identity one");
    assert_eq!(w, (1.1 * MF + SENTINEL) - ma,
               "so the sentinel reached the reference, which means the call went through the \
                receiver's TABLE and not through rung 74's function");
    assert_eq!(reg, LegRegime::Dormant,
               "and `SENTINEL > mf_sched`, so this cap is not riding");
}

/// **`_cap_fuel`'s `min` FOLD HAS SOMETHING TO ORDER HERE, WHICH DISCHARGES STEP 1 § (j)'s BOOKED
/// BLIND SPOT.**
///
/// Step 1's mutation reversing that fold SURVIVED, correctly: only one cap was ever armed at that
/// step, so the fold had nothing to order. `demand_laws`' `F` arms the accel schedule and the surge
/// floor together, and this is the first gate in the crate where both are live at once.
///
/// **THE REFERENCE COMES FROM TWO OTHER RUNS OF THE SAME CODE, NOT FROM THE FOLD** — § 5.30 (viii)
/// item 1. Each cap is measured on its own with the other disarmed, and the both-armed answer is
/// asserted to be the SMALLER of the two. A reversed fold returns the larger and dies here; a fold
/// that dropped one arm entirely also dies, because the two singletons are asserted unequal first.
#[test]
fn the_two_cap_fold_is_a_min_and_step_1s_blind_spot_is_discharged() {
    let a = arm();
    let m = demand(&a);
    let f = flight();
    let floor = Floor::Phi(surge());
    // A flat `Wf/pt3` schedule whose cap is not the same number as the surge one. **`1e-8`, and
    // it is MEASURED**: the Python sweep over `kappa` found every value from `1e-6` up aborting the
    // accel cap's own bracket at this point, and `1e-8` is the only one of seven that returns on
    // all three armings. `slice_af_cells.rs` uses the same constant, for the same reason.
    let sched = flat_schedule(1.0e-8);

    let one = |acc: Option<&AccelSchedule>, su: Option<&Floor>| -> f64 {
        let laws = demand_laws(&m, &f, A, H, MF, acc, su, TT4_MAX);
        (laws.f)(MF, MF, 0.0, 0.0).expect("the cap solve closes here").0
    };

    let accel_only = one(Some(&sched), None);
    let surge_only = one(None, Some(&floor));
    let both = one(Some(&sched), Some(&floor));

    assert_ne!(accel_only, surge_only,
               "THE INSTRUMENT CAN SEE — the two caps are different numbers, so a fold over them \
                is a real selection and not a tie");
    let smaller = if surge_only < accel_only { surge_only } else { accel_only };
    let larger = if surge_only < accel_only { accel_only } else { surge_only };
    assert_eq!(both.to_bits(), smaller.to_bits(),
               "both armed returns the SMALLER cap, bit for bit — min-select one level down");
    assert_ne!(both.to_bits(), larger.to_bits(),
               "and a reversed fold would have returned this one");
}

/// **`R` IS INDEPENDENT OF `wf` ONCE THE APPLIED DEMAND IS PINNED, AND `F` IS NOT INDEPENDENT OF
/// IT — WHICH IS RUNG 72's `R_f = 0` SURVIVING THE COORDINATE.**
///
/// Both laws take all four states here, where rung 72's took three, and the reason is the
/// REFERENCE: each leg's own demand enters as `w_own`. `R`'s `w_own` is `wr` and `F`'s is `wf`, so
/// swapping either is a silent relabel that produces plausible numbers.
///
/// The two probe points hold `applied_demand` fixed by keeping BOTH legs above the schedule on one
/// axis, so the only channel left is `w_own`:
///
/// * `R(10 mf, 0.5 mf, ·)` and `R(20 mf, 0.5 mf, ·)` share `ma = 0.5 mf` and must agree BIT FOR
///   BIT — a body reading `wf` as `w_own` moves by `10 mf`.
/// * `F(10 mf, 0.5 mf, ·)` and `F(20 mf, 0.5 mf, ·)` share the same `ma` and must DIFFER, because
///   `wf` IS `F`'s own demand. That is the only route by which `F` depends on `wf` at all, and it
///   is exactly what `F_f` differences at step 4.
#[test]
fn r_is_blind_to_the_fuel_demand_and_f_is_not() {
    let a = arm();
    let m = demand(&a);
    let f = flight();
    let floor = Floor::Phi(surge());
    let laws = demand_laws(&m, &f, A, H, MF, None, Some(&floor), TT4_MAX);

    let (wf1, wf2, wr) = (10.0 * MF, 20.0 * MF, 0.5 * MF);
    assert_eq!(applied_demand(MF, wf1, wr), applied_demand(MF, wf2, wr),
               "the applied demand is pinned across the two probes — stated, not assumed");

    let r1 = (laws.r)(wf1, wr, 0.0, 0.0).expect("the governor cap solves here").0;
    let r2 = (laws.r)(wf2, wr, 0.0, 0.0).expect("the governor cap solves here").0;
    assert_eq!(r1.to_bits(), r2.to_bits(),
               "`R`'s own demand is `wr`; moving `wf` with the applied demand pinned must not \
                reach it");

    let f1 = (laws.f)(wf1, wr, 0.0, 0.0).expect("the fuel cap solves here").0;
    let f2 = (laws.f)(wf2, wr, 0.0, 0.0).expect("the fuel cap solves here").0;
    assert_ne!(f1, f2,
               "and `F`'s own demand IS `wf`, so the same move DOES reach it — without this the \
                assertion above would pass for a pair of laws that ignored their arguments");
}

/// **THE REGIME LABEL CANNOT BE MOVED BY THE COORDINATE, AND THAT IS WHY READING THE LATCHED
/// TARGET INSTEAD OF THE CAP IS A NO-OP.**
///
/// `F` and `R` label `riding` from `cap < mf_sched`. Reading the latched target instead was
/// nominated as a must-kill mutation and it is inert, provably: under `clip` and `demand`
/// `tgt == cap` identically, and under `demand-latched`
/// `min(mf_sched, cap) < mf_sched  <=>  cap < mf_sched`. So the two readings agree on all three
/// tags and every input.
///
/// **The survivor is recorded as a survivor with a proof, rather than left to be found and misread
/// as a blind spot** — step 1 § (h)'s lesson, which cost a wrong code comment there. What IS gated
/// is the invariance itself: the label is the same on all three coordinates at the same point, so a
/// port that made the label coordinate-dependent (in either direction) fails here.
#[test]
fn the_regime_label_cannot_be_moved_by_the_coordinate() {
    let a = arm();
    let m = demand(&a);
    let f = flight();
    let floor = Floor::Phi(surge());
    let t = &m.fuel.inner;

    let label = |coord: &'static str| -> (LegRegime, LegRegime) {
        let _s = CoordScope::set(t, coord);
        let laws = demand_laws(&m, &f, A, H, MF, None, Some(&floor), TT4_MAX);
        let fr = (laws.f)(MF, MF, 0.0, 0.0).expect("the fuel cap solves here").1;
        let rr = (laws.r)(MF, MF, 0.0, 0.0).expect("the governor cap solves here").1;
        (fr, rr)
    };

    let clip = label(LAG_COORD_CLIP);
    assert_eq!(label(LAG_COORD_DEMAND), clip);
    assert_eq!(label(LAG_COORD_LATCHED), clip,
               "the latch moves the TARGET and never the LABEL — which is the algebra above, \
                driven rather than asserted");

    // AND THE LABEL IS NOT A CONSTANT, so the invariance above is a measurement.
    assert_eq!(clip.0, LegRegime::Riding,
               "the fuel leg IS riding at this schedule — a gate over three equal `Dormant`s \
                would have measured nothing");
}
