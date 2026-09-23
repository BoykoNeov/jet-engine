//! RUNG 78 — **THE RESIDUAL GAUGE.** `ResidualGaugeTransient`, slice AH.
//!
//! Rung 77 § 3 lists two routes to a singular set-point solve, `dw*/dq = −G_q/G_w` with
//! `G_w → 0`, and calls the first of them (`c → 1`) *unreachable in this family*. **This rung
//! reaches it in one line, at a FIXED set point, and finds that arriving costs nothing.** Anchor
//! the accel leg's cap at its own root and scale the departure:
//!
//! ```text
//! cap_k(w) = w0 + k·(cap(w) − w0)        w0 := the k = 1 root, cap(w0) = w0
//! ```
//!
//! `cap_k(w0) = w0` IDENTICALLY for every `k` — so the set point is invariant by construction, not
//! by tolerance — while `G_k' = 1 − k·c` is a free dial reaching ZERO at `k = 1/c`. `G_w` and
//! `G_q` scale in LOCKSTEP, so `dw*/dq = w0'` for every `k`: rung 77 § 3's first route is a
//! **REMOVABLE** singularity, and `1/(1−c)` is a GAUGE. What `c → 1` actually costs is not
//! sensitivity but WELL-POSEDNESS — a SECOND root sweeps in, collides with the true one exactly at
//! `k·c = 1`, and departs upward.
//!
//! # WHAT STEP 1 OF THIS SLICE ADDS AT THIS RUNG — **THREE RE-AIMED POINTERS AND NOT ONE NEW TABLE FIELD**
//!
//! | | Python | slot | table |
//! |---|---|---|---|
//! | swap | `at_lever` | `LeverHooks::at_lever` | [`R78`] |
//! | swap | `_shared_rig` | [`shared_rig`](crate::three_loop::TripleHooks::shared_rig) | [`R78_TRIPLE`] |
//! | swap | `_cap_fuel` | [`cap_fuel`](crate::three_loop::TripleHooks::cap_fuel) | [`R78_TRIPLE`] |
//!
//! `TripleHooks` stays at **18** fields and `LeverHooks` is untouched — plan § 5.32 (ii), and
//! probe 7 makes that durable past rungs 79–84 rather than merely correct today. The `0 ADD`
//! consequence — that a missed re-aim COMPILES and answers the reduce arm, with no `E0063` width
//! tripwire anywhere in the crate — is written once, at
//! [`stiffness_ledger`](crate::stiffness_ledger), and the function-pointer identity gates that
//! answer it cover both rungs in `tests/slice_ah_cells.rs`.
//!
//! # THE SIXTH DECLARED KNOB, AND THE SECOND THAT ADDS NO CONSTANT AT ALL
//!
//! `_gauge_k` joins `_share_law` (72), `_ref_law` (73), `_lag_coord` (74), `_windup_law` (75) and
//! `_cap_law` (76). `k` is SWEPT, never set, and `w0` is the plant's own already-solved set point,
//! so nothing here is a chosen number. **Its domain is DECLARED**: the gauge reaches the ACCEL leg
//! and nothing else, for rung 77 § 1's reason read one step on — `Tt4_max` and `phi_lim` are
//! CONSTANTS, so the governor and the phi leg have no cap to anchor and no `c` to scale. A gauge
//! needs a formula, and a floor on a STATE is not one.
//!
//! **The reduce is an `is`-exact comparison against `1.0`** ([`GAUGE_K_IDENTITY`]), not a
//! tolerance, so at the identity gauge not one float in this family moves.
//!
//! # THE TWO REFUSALS ARE PORTED FROM THE SOURCE, NOT TRANSLATED FROM A TEST
//!
//! Plan § 5.32 (v), probes 4 and 11: rung 78 ships **two** refusals (`engine.py:20173`,
//! `engine.py:20188`) and **neither suite file asserts either of them** — `tests/test_rung77.py`
//! and `tests/test_rung78.py` carry **zero** `raises` and **zero** `match=` needles between them.
//! Rung 77 ships none to assert. So this is the first ported gate file in the phase whose refusal
//! section has no Python precedent to copy: both refusals are written here from the source, and
//! their gates at step 3 are written from this module.
//!
//! # WHAT STEP 4 ADDS — **§§ 1–3, AND THE SIXTH KNOB'S SECOND RESTORE POLICY**
//!
//! [`gauge_scan`] (§ 1/§ 2), [`root_census`] (§ 3), [`root_count`], [`accel_cap_fn`] and
//! [`gauge_points`]. Rung 78's remaining sections (`gauge_vs_device`, `gauge_march`) are step 5's.
//!
//! **The step's finding is that this slice's leading hazard has a second instance on a different
//! variable.** Plan § 5.32 (i) opens on the `_b_state`/`_v_state` freeze being handled three
//! incompatible ways across these two classes; its census was scoped to that pair, and so could
//! not see that `_gauge_k` is too — `engine.py:20394` / `engine.py:20404` save and restore `prev`,
//! `engine.py:20508` / `engine.py:20514` clobber to the literal identity, and
//! `engine.py:20705` / `engine.py:20714` use the declared helper `_with_gauge` that the other two
//! ignore. Both policies are ported, as [`GaugeRestored`] and [`GaugeClobbered`], and the clobber
//! is measured to be reachable-wrong inside a single call.
//!
//! # THE COUNTERS ARE PROCESS-GLOBAL BECAUSE PYTHON'S ARE CLASS ATTRIBUTES
//!
//! [`GAUGE_HITS`] and [`GAUGE_BINDS`] are `static`s, not fields on the core. Python writes
//! `ResidualGaugeTransient._gauge_hits += 1` and its own comment says why: *"It MUST be written on
//! the CLASS: `self._x += 1` would create an instance attribute and leave the class one at zero
//! forever, so the instrument built to catch a vacuous section would itself have been vacuous."*
//! A per-core `Cell` would be exactly the instance attribute that comment rejects — the march
//! builds its rig with `at_lever`, so the object that runs is not the object a reader would then
//! interrogate. They are INSTRUMENTS, never read by the plant.
//!
//! # THE `_b_state`/`_v_state` NEST — **THE ONE BOOKING NO GATE CAN CLOSE**
//!
//! Slice AC (§ 5.19 (iv) / § 5.22 (vii)) booked a same-field nest of the two frozen-state carriers
//! to this slice off a NAME-BASED reachability graph, and called that an upper bound. It is now
//! measured twice — statically and at runtime, by two instruments that disagreed and then
//! converged (plan § 5.32 (i)). **There are six genuine nests at five sites, and they sit at BOTH
//! of this slice's rungs:**
//!
//! | site | rung | caller | callee | handed | runtime |
//! |---|---|---|---|---|---|
//! | `engine.py:19829` | 77 | `leg_slopes` | `_c_at` | the enclosing `q, v` | 120 + 30 |
//! | `engine.py:20412` | 78 | `gauge_scan` | `_c_at` | the enclosing `q, v` | 50 |
//! | `engine.py:20503` | 78 | `root_census` | `_c_at` | the enclosing `q, v` | 20 |
//! | `engine.py:20591` | 78 | `gauge_vs_device` | `_phi_at` | **`q + dq`, `v`** | 20 |
//! | `engine.py:20592` | 78 | `gauge_vs_device` | `_phi_at` | **`q − dq`, `v`** | (as above) |
//! | `engine.py:20691` | 78 | `gauge_march` | `_c_at` | the enclosing `b, v` | 1 |
//!
//! Every static site fires at runtime, which is the validation neither instrument has alone.
//! **The runtime counter cannot separate the last two**, and that is a property of the reading
//! rather than of the code: both `_phi_at` calls belong to one multi-line statement and Python
//! attributes both frames to its FIRST line, so the tally shows a single key at `engine.py:20591` holding
//! 20 — ten central differences, twenty nests. The static census sees two calls; the counter sees
//! one site. They agree, and the row above says so rather than quoting 20 twice.
//! **The source handles the hazard three incompatible ways in one slice** — `leg_slopes` nests and
//! never re-freezes (safe only because the one statement after it is arithmetic over numbers
//! already computed); `root_census` nests and MUST re-freeze, and does, at `engine.py:20504`; and
//! `_c_on_frozen` closes its block BEFORE calling `_c_at` at all, its docstring recording that an
//! earlier version did not and shipped wrong.
//!
//! **THE CRITERION THAT MAKES THE PORT SAFE IS THE DEAD WINDOW, NOT THE `(q, v)` AGREEMENT.** The
//! first writing of this claim said each callee receives the enclosing block's own `(q, v)`. That
//! is FALSE at two of the six: `gauge_vs_device` hands `_phi_at` **`q ± dq`**, so there the two
//! restore semantics genuinely leave different values behind. What holds at all six is weaker and
//! checked per site: **the window in which the two semantics disagree is DEAD** — between an inner
//! freezer's return and the next set or `finally`, nothing reads the plant. Verified by reading
//! all six: `engine.py:19829` is followed by one arithmetic assignment; `engine.py:20412` and `engine.py:20691` by their
//! `finally`; `engine.py:20503` by the re-freeze at `engine.py:20504`; `engine.py:20591`/`engine.py:20592` are the two halves of one
//! central difference with only a division and an assignment after them.
//!
//! **AND THIS IS WHY THE DISCHARGE IS A DOC CLAIM AND NOT A GATE.** The two restore semantics
//! produce identical numbers on identical inputs at every reachable site, so no value gate in the
//! port can assert which one was chosen. Writing one anyway would be an instrument fed by what it
//! certifies, arriving from the opposite side. **A booking no gate can close is not a gate nobody
//! wrote — it is a claim in the wrong currency.** What IS gated is the weaker, testable thing:
//! that the choice moves no key (§ 5.32 (vii) P3).
//!
//! # THE PRE-FLIGHT SAID THE REPAIR WAS **FREE**. IT IS NOT — **A SHIPPED GATE ALREADY PINS THE OTHER POLICY**
//!
//! § 5.32 (i) proposed one repair for all six sites: give `_b_state`/`_v_state` an RAII guard whose
//! `Drop` restores the PREVIOUS value, making site 1's accidental safety real, site 2's re-freeze a
//! no-op and site 3's defensive double-freeze unnecessary. **Four of the six nests are inside rung
//! 76's `_c_at`**, which in this port is [`c_at`](crate::sensed_cap::c_at) using
//! [`MarchedBleed`](crate::two_spool_transient::MarchedBleed) and
//! [`MarchedStator`](crate::two_spool_transient::MarchedStator) — so the proposal is a change to
//! those two `Drop`s, crate-wide across 66 call sites in 11 modules. Step 1 measured both halves of
//! whether that is admissible:
//!
//! * **It is value-invisible everywhere below rung 77, and that is now measured rather than
//!   argued.** § 5.25 (iii) had rungs ≤ 68 at 0 overwrites and § 5.26 (vii) rung 69; **rungs 70–76
//!   had never been measured**, and they hold the majority of those 66 sites. Re-pointing the
//!   pre-flight's probe 13 at them: **0 nests in 5 182 138 `_b_state` sets and 4 856 210 `_v_state`
//!   sets**, over 161 tests, nine per-worker tallies each with a passing positive control. The
//!   second shape — an explicit `= None` written over a live value, which the nest counter cannot
//!   see and which restore-previous would also change — is **5 018 510 events at 46 innermost
//!   sites, and every one of them is a `finally` restore. Zero guard entries.**
//! * **And it is blocked anyway.** `tests/slice_y_dispatch.rs`'s
//!   `the_b_state_guard_restores_none_which_is_the_opposite_policy` MANUFACTURES this exact nest
//!   and asserts the inner guard CLEARS the outer, carrying the message *"Two guards, two policies
//!   — do not unify them."* Slice Y did not restore `None` by inertia: it measured the choice
//!   value-invisible, chose Python's spelling, and built a hand nest to pin it. **Overturning that
//!   is its own decision with its own authorisation, not a side effect of a step 1.**
//!
//! **So § (i)'s "the repair is free" is measured FALSE, and the scope of its structural claim
//! shrinks from six sites to two.** Sites 1/2/3/6 (`leg_slopes`, `gauge_scan`, `root_census`,
//! `gauge_march`, all through `_c_at`) keep Python's clobber and are safe **by the dead-window
//! criterion, not by construction** — which is exactly the reading verified per site above, and it
//! is why that criterion had to be got right. Restore-previous is used only where this slice writes
//! NEW code: `_phi_at`, sites `engine.py:20591`/`engine.py:20592`, the two where the callee is handed `q ± dq` and the
//! two semantics genuinely differ. No shipped gate pins those, and the guard that carries them —
//! with § 5.26 (iii)'s owed message — lands **with `gauge_vs_device`'s `_phi_at`**, the rung-78 leaf
//! that is its only caller.
//!
//! > This sentence and plan § 5.32.1's both said *"at step 2"* when they were written. Step 2 is
//! > rung 77's reader layer and `_phi_at` is a rung-78 leaf, so the promise named a step that could
//! > not keep it. Re-aimed at the BODY rather than at a step number, which is the only form of the
//! > claim that cannot decay — this slice has already logged a third instance of *a documented
//! > gate that does not exist*, and a fourth authored in the port's own header would have been the
//! > cheapest possible one to avoid.
//!
//! **The lesson is the slice's own, from the other side: a booking that no gate can CLOSE can still
//! be BLOCKED by one.** The pre-flight looked for a gate that could state the claim and found none;
//! it did not look for a gate that had already decided it.
//!
//! # AND A DOCUMENTED MANUFACTURED NEST FOR `MarchedStator` DOES NOT EXIST
//!
//! Found while checking the above. `tests/slice_aa_oracle.rs`'s header lists, among the things no
//! value key can witness, *"`ForcedStator`'s and `MarchedStator`'s restore POLICY … Manufactured in
//! `slice_aa_cells.rs`, which is step 1's file."* **`slice_aa_cells.rs` manufactures nests for
//! `InitialStator` and `DeclaredOrder` only.** Its `MarchedStator` block is a single set entered
//! from `None`, which passes under EITHER policy and pins neither. The real pin for this pair is
//! `MarchedBleed`'s, one field over, in a different file and a different slice. Recorded, not
//! repaired: the claim is a doc comment's, the gate it names is real but does not cover the name it
//! is cited for, and adding one now would be a new gate rather than a corrected citation.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::bleed_transient::{LeverArm, LeverHooks};
use crate::demand_coordinate::cap_free;
use crate::engine::FlightCondition;
use crate::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelPoint, FuelTransientCore, FuelTransientHooks,
};
use crate::gas::Abort;
use crate::map::ComponentMap;
use crate::sensed_cap::{c_at, C_AT_REL};
use crate::shared_actuator::{riding4, SharedRigArm};
use crate::stiffness_ledger::SLOPE_AT_REL;
use crate::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient, StatorTransientHooks};
use crate::three_loop::TripleHooks;
use crate::two_spool::TwoSpoolEngine;
use crate::two_lag::{py_max_of, py_min_of};
use crate::two_spool_transient::{
    MarchedBleed, MarchedStator, TwoSpoolTransientCore, TwoSpoolTransientHooks,
};

// ---------------------------------------------------------------------------------------------
// THE DECLARED KNOB AND THE TWO INSTRUMENTS
// ---------------------------------------------------------------------------------------------

/// Python's `_gauge_k = 1.0` — **THE CLASS DEFAULT, AND IT IS THE REDUCE ARM.**
///
/// At this value [`r78_cap_fuel`] dispatches to the parent's body and the whole family is
/// bit-for-bit rung 77. The comparison is `==` against this constant and NOT a tolerance, which is
/// Python's own `if self._gauge_k == 1.0` — a gauge one ULP off the identity is a different
/// residual and must take the gauged branch.
pub const GAUGE_K_IDENTITY: f64 = 1.0;

/// How many times the GAUGED branch of [`r78_cap_fuel`] actually ran — Python's `_gauge_hits`.
///
/// § 5's first version reported a bit-identical trajectory (`0.000e+00`) under four different
/// gauges and it was VACUOUS: `_ledger_march` marches in the CLIP coordinate, and rung 76 § 0's
/// own refusal says `clip` dispatches out of this ladder BEFORE `_cap_fuel` is reached. The branch
/// never executed. **A counter makes that failure impossible to repeat silently.**
///
/// `static` for the reason in the module header: Python's is a CLASS attribute, deliberately.
pub static GAUGE_HITS: AtomicU64 = AtomicU64::new(0);

/// How many times the gauged cap's VALUE won the min-select — Python's `_gauge_binds`.
///
/// A DIFFERENT question from [`GAUGE_HITS`], and both failures look exactly like a pass: that
/// counter proves the gauged branch RAN, this one proves its value REACHED THE PLANT. If the phi
/// leg won every `min`, the accel cap would be computed 1 366 times a march and discarded 1 366
/// times, and *"the gauge is inert on the trajectory"* would be untested.
pub static GAUGE_BINDS: AtomicU64 = AtomicU64::new(0);

/// Read both instruments — `(hits, binds)`, in Python's declaration order.
pub fn gauge_counters() -> (u64, u64) {
    (GAUGE_HITS.load(Ordering::Relaxed), GAUGE_BINDS.load(Ordering::Relaxed))
}

/// Zero both instruments, returning what they held.
///
/// Python's counters run for the life of the interpreter and its readers difference them; a gate
/// that wants an absolute count resets first. Exposed rather than left to `store` at call sites so
/// the pair can never be reset by halves.
pub fn reset_gauge_counters() -> (u64, u64) {
    (GAUGE_HITS.swap(0, Ordering::Relaxed), GAUGE_BINDS.swap(0, Ordering::Relaxed))
}

// ---------------------------------------------------------------------------------------------
// THE CASCADE BUILDER
// ---------------------------------------------------------------------------------------------

/// Build a rung-78 object, so every sibling re-asserts the whole chain's guards.
///
/// `_ref_law` is set for rung 73's reason, inherited through the chain rather than copied. The
/// sixth knob is NOT written here: `_gauge_k`'s class default is `1.0` and so is the core's, so a
/// set would be a line that looks like it is doing the `ref_law` job — [`build_sensed_cap_cascade`
/// ](crate::sensed_cap::build_sensed_cap_cascade)'s reason, one knob over.
pub fn build_residual_gauge_cascade(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    let built = crate::reference_split::build_split_family_cascade(
        design_engine, flight_design, mdot_design, map_lp, map_hp, rho, arm,
        &R78_TWO, &R78_STATOR, &R78_FUEL, &R78, &R78_TRIPLE);
    if let ScheduledStatorTransient::Full(c) = &built {
        c.fuel.inner.ref_law.set(crate::applied_reference::REF_LAW_APPLIED);
    }
    built
}

// ---------------------------------------------------------------------------------------------
// THE TABLES — five, and TWO of them carry something of this rung's own
// ---------------------------------------------------------------------------------------------

/// RUNG 78's lever table — ONE swap, `at_lever`, and the parent it must differ from is rung 77's.
///
/// The SIXTEENTH instance of the sibling-constructor trap. Rung 77 lost the CLASS here; this rung
/// would lose the knob on top of it, and every reader below would silently run at `k = 1` — which
/// is a legal plant, is rung 77, and passes every reduce gate in the crate.
pub const R78: LeverHooks = LeverHooks {
    at_lever: r78_at_lever,
    ..crate::stiffness_ledger::R77
};

/// RUNG 78's `TwoSpoolTransientHooks` — **ZERO cells swapped**, an alias.
pub const R78_TWO: TwoSpoolTransientHooks = crate::stiffness_ledger::R77_TWO;

/// RUNG 78's fuel table — **ZERO cells swapped**, an alias.
///
/// This rung's two refusals live in the cap, not in the march, so `integrate_fuel` stays rung
/// 76's — the body that carries the CAP-LAW refusals a gauged machine still owes.
pub const R78_FUEL: FuelTransientHooks = crate::stiffness_ledger::R77_FUEL;

/// RUNG 78's stator table — **ZERO cells swapped**, an alias.
pub const R78_STATOR: StatorTransientHooks = crate::stiffness_ledger::R77_STATOR;

/// RUNG 78's third-loop table — **TWO of rung 77's eighteen cells re-aimed, and NOTHING added.**
///
/// Spelled out field by field for [`R76_TRIPLE`](crate::sensed_cap::R76_TRIPLE)'s reason: with two
/// cells moving and sixteen staying put, and with **no width tripwire anywhere in this slice**, the
/// exhaustive literal is the only place a reader sees both facts at once. In particular
/// `sensed_cap` must stay pointed at RUNG 76's body here while `cap_fuel` moves to this one — the
/// two interact, because [`r78_cap_fuel`]'s gauged branch deliberately does NOT consult it.
pub const R78_TRIPLE: TripleHooks = TripleHooks {
    stator_leg: crate::stiffness_ledger::R77_TRIPLE.stator_leg,
    lagged_stator: crate::stiffness_ledger::R77_TRIPLE.lagged_stator,
    clamp_v: crate::stiffness_ledger::R77_TRIPLE.clamp_v,
    check_v0: crate::stiffness_ledger::R77_TRIPLE.check_v0,
    rk4_floor: crate::stiffness_ledger::R77_TRIPLE.rk4_floor,
    solve_v: crate::stiffness_ledger::R77_TRIPLE.solve_v,
    manifold_v: crate::stiffness_ledger::R77_TRIPLE.manifold_v,
    triple_laws: crate::stiffness_ledger::R77_TRIPLE.triple_laws,
    triple_rig: crate::stiffness_ledger::R77_TRIPLE.triple_rig,
    with_ref: crate::stiffness_ledger::R77_TRIPLE.with_ref,
    reference: crate::stiffness_ledger::R77_TRIPLE.reference,
    quad_gains_at: crate::stiffness_ledger::R77_TRIPLE.quad_gains_at,
    rk4_floor_shared: crate::stiffness_ledger::R77_TRIPLE.rk4_floor_shared,
    windup_tau: crate::stiffness_ledger::R77_TRIPLE.windup_tau,
    with_coord: crate::stiffness_ledger::R77_TRIPLE.with_coord,
    sensed_cap: crate::stiffness_ledger::R77_TRIPLE.sensed_cap,
    // THE TWO THIS RUNG RE-AIMS.
    shared_rig: r78_shared_rig,
    cap_fuel: r78_cap_fuel,
};

// ---------------------------------------------------------------------------------------------
// THE THREE RE-AIMED BODIES, AND THE TWO HELPERS THE THIRD CANNOT SHIP WITHOUT
// ---------------------------------------------------------------------------------------------

/// RUNG 78's `at_lever` — **rung 77's sibling constructor returning a RUNG-78 machine THAT CARRIES
/// SEVEN KNOBS.**
///
/// Sixteenth instance of the trap, and the seventh knob is the one this rung is about. See
/// [`r77_at_lever`](crate::stiffness_ledger)'s note for why the copy lines are not to be factored
/// onto the parent: they are redundant on the value and load-bearing on the pointer identity.
fn r78_at_lever(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    let m = match build_residual_gauge_cascade(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(), arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("at_lever never disables LP"),
    };
    m.fuel.inner.ref_law.set(core.fuel.inner.ref_law.get());
    m.fuel.inner.lag_coord.set(core.fuel.inner.lag_coord.get());
    m.fuel.inner.windup_law.set(core.fuel.inner.windup_law.get());
    m.fuel.inner.tau_t.set(core.fuel.inner.tau_t.get());
    m.fuel.inner.ic_cap.set(core.fuel.inner.ic_cap.get());
    m.fuel.inner.cap_law.set(core.fuel.inner.cap_law.get());
    m.fuel.inner.gauge_k.set(core.fuel.inner.gauge_k.get());
    m
}

/// RUNG 78's `_shared_rig` — **rung 77's rig with the GAUGE carried too, a sixth knob on the same
/// leak.**
///
/// Predicted a NO-OP for the structural reason its three predecessors were: rung 72's body reaches
/// its sibling through `at_lever`, which on a rung-78 receiver is [`r78_at_lever`], which has
/// already copied all seven. **Ported unchanged regardless** — a duplication the source makes is
/// not the port's to remove, and slice AG measured that deleting the equivalent line at rung 76
/// leaves every value gate green while killing the POINTER gate, because the remaining body
/// forwards straight to its parent's and the linker folds the two into one address.
fn r78_shared_rig(
    core: &ScheduledStatorCore, arm: &SharedRigArm,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>) {
    let (m, surge, lag) = (crate::stiffness_ledger::R77_TRIPLE.shared_rig)(core, arm);
    m.fuel.inner.gauge_k.set(core.fuel.inner.gauge_k.get());
    (m, surge, lag)
}

/// `G_k(w) = w − cap_k(w)` — and at `k = 1` it is the shipped residual EXPRESSION, not merely a
/// number equal to it: `w0` does not appear at all.
///
/// Returned as a boxed closure rather than an `enum` of two shapes because both callers use it
/// exactly once, through [`gauge_root`], and the branch is Python's own.
pub fn gauge_residual<'a>(
    k: f64, cap: &'a dyn Fn(f64) -> Result<f64, Abort>, w0: f64,
) -> Box<dyn Fn(f64) -> Result<f64, Abort> + 'a> {
    if k == GAUGE_K_IDENTITY {
        Box::new(move |w: f64| Ok(w - cap(w)?))
    } else {
        Box::new(move |w: f64| Ok(w - (w0 + k * (cap(w)? - w0))))
    }
}

/// DAMPED Newton with a differenced derivative — **the one root finder in this rung**, and it has
/// to survive BOTH SIGNS of `G_w`, which is why the shipped bracket is not reused.
///
/// Two damping rules, and both are SOLVER facts rather than plant ones:
///
/// * a TRUST REGION (`|dw| ≤ trust·w`). Near the singular gauge `|G'|` is small, so an undamped
///   step is enormous and lands off the modelled speed-line region — which the plant reports as
///   rung 62's bracket assertion, not as a root-finding failure.
/// * BACKTRACKING ON THE PLANT'S OWN REFUSAL: an [`Abort`] from an evaluation is a step that left
///   the model, so the step is halved and retried rather than propagated. **This is where Python's
///   `except AssertionError` becomes an `Err` match**, and the two are the same control flow —
///   slice L's rule read at the call site rather than at the definition.
///
/// Returns `(w, ok)`. **`ok` is NOT a correctness guard** — § 3's whole content is that inside the
/// collision band this converges cleanly onto the WRONG root. Every caller that needs correctness
/// checks `|w − w0|/w0` instead.
pub fn gauge_root(
    big_g: &dyn Fn(f64) -> Result<f64, Abort>, guess: f64, rel: f64, n: usize, trust: f64,
) -> (f64, bool) {
    let (mut w, mut prev): (f64, Option<f64>) = (guess, None);
    for _ in 0..n {
        // Python's `1e-7 * max(abs(w), 1e-9)` — argument 0 is an EXPRESSION, so `w.abs().max(1e-9)`
        // is the wrong spelling: it returns `1e-9` for a NaN `w` where Python's fold returns `nan`.
        // Step 1 shipped the `f64::max` form; step 2's re-run of slice AG step 1's census found
        // this site along with three others, against `c_at`'s claim that its own is the only one.
        // See [`slope_at`](crate::stiffness_ledger::slope_at).
        let aw = w.abs();
        let dw = 1e-7 * if 1e-9 > aw { 1e-9 } else { aw };
        // Python evaluates `G(w)` first, then `G(w + dw)`, then `G(w - dw)`, and ONE `except`
        // covers all three. `?` here would propagate where Python retreats.
        let trio = (|| -> Result<(f64, f64), Abort> {
            let g = big_g(w)?;
            let gp = (big_g(w + dw)? - big_g(w - dw)?) / (2.0 * dw);
            Ok((g, gp))
        })();
        let (g, gp) = match trio {
            Ok(v) => v,
            Err(_) => match prev {
                None => return (f64::NAN, false),
                Some(p) => {
                    w = 0.5 * (w + p); // retreat toward the last good point
                    continue;
                }
            },
        };
        if gp == 0.0 {
            return (f64::NAN, false);
        }
        let mut step = g / gp;
        let lim = trust * w.abs().max(1e-12);
        if step.abs() > lim {
            step = if step > 0.0 { lim } else { -lim };
        }
        prev = Some(w);
        w -= step;
        if !(w > 0.0) {
            return (f64::NAN, false);
        }
        if step.abs() <= rel * w.abs().max(1e-12) {
            return (w, true);
        }
    }
    (w, false)
}

/// Python's `_gauge_root` defaults, carried as named constants so the gate and the call site read
/// the same numbers — `rel`, `n`, `trust`.
pub const GAUGE_ROOT_REL: f64 = 1e-10;
pub const GAUGE_ROOT_N: usize = 400;
pub const GAUGE_ROOT_TRUST: f64 = 0.25;

/// The accel leg's set point UNDER THE GAUGE, solved from the plant's own guess — **and BOTH of
/// rung 78's shipped refusals.**
///
/// `w0` is the SHIPPED unfloored set point (rung 74's `cap_free` on rung 77's residual, untouched)
/// and it is the anchor. The gauged residual is then re-solved from `mf_sched`, **not** from `w0`:
/// a solver seeded with the answer would hand it back whether or not the gauge preserved the root,
/// and the whole rung is the question of whether it does.
///
/// # THE TWO REFUSALS, AND WHY THEY ARE `Abort` AND NOT `panic!`
///
/// Python spells both as `assert`, and this function is reached from `_cap_fuel` inside a march
/// whose derivative is wrapped in `except AssertionError: break` (`engine.py:17967`/`17991`). A
/// `panic!` would end the process where Python ends the march. That is slice L's rule applied per
/// CALL SITE, and it is the same decision `cap_free`'s own unreachable-cap refusal records.
///
/// * `sensed × a non-identity GAUGE` is REFUSED because this branch bypasses `_sensed_cap`, so the
///   cap would be evaluated at the TRIAL fuel instead of the APPLIED one and the march would
///   silently be rung-78-on-rung-75 while being reported as rung-78-on-rung-76.
/// * the ANCHORED-ROOT refusal is **not** a convergence check. `ok` is deliberately not the guard:
///   the gauge preserves `w0` as a root identically, so the only correctness question is whether
///   the solver found THAT root — and § 3 measures it converging cleanly onto a SECOND one inside
///   the collision band. A gate on `ok` would pass every wrong answer this rung is about.
fn gauged_accel_cap(
    ft: &FuelTransientCore, flight: &FlightCondition, a: f64, h: f64, mf_sched: f64,
    accel: &AccelSchedule,
) -> Result<f64, Abort> {
    let law = ft.inner.cap_law.get();
    let k = ft.inner.gauge_k.get();
    if law == crate::sensed_cap::CAP_LAW_SENSED {
        return Err(Abort(format!(
            "rung-78: `sensed` x a non-identity GAUGE is REFUSED. This branch bypasses \
             `_sensed_cap`, so the cap would be evaluated at the TRIAL fuel instead of the \
             APPLIED one and the march would silently be rung 78-on-rung-75 while being \
             reported as rung 78-on-rung-76 -- rung 76 s 0's own refusal of `clip x sensed`, \
             one knob over. Got _cap_law = {law:?}, _gauge_k = {k:?}.")));
    }
    let pi_b = ft.inner.inner.base.pi_b;
    let cap = |w: f64| -> Result<f64, Abort> {
        let i = ft.try_instant_fuel(flight, a, h, w)?;
        Ok(accel.cap(i.base.close.n_hp, i.base.close.pt4 / pi_b))
    };
    let big_g0 = |w: f64| -> Result<f64, Abort> { Ok(w - cap(w)?) };
    let w0 = cap_free(&big_g0, mf_sched, &|| ft.try_sched_fuel(flight, a, h, mf_sched, accel))?;
    let big_gk = gauge_residual(k, &cap, w0);
    let (w, ok) = gauge_root(
        &*big_gk, mf_sched, GAUGE_ROOT_REL, GAUGE_ROOT_N, GAUGE_ROOT_TRUST);
    if !(ok && (w - w0).abs() <= 1e-6 * w0.abs()) {
        return Err(Abort(format!(
            "rung-78: the GAUGED accel solve returned {w:?} at k = {k:.6} where the anchor \
             is w0 = {w0:.6e} (converged = {ok}). `w0` is a root of `G_k` IDENTICALLY, so \
             this is either a solver failure or -- inside the multi-root band around \
             `k*c = 1` -- the SECOND root (spec s 3). A march must not run on either.")));
    }
    Ok(w)
}

/// RUNG 78's `_cap_fuel` — **rung 74's min-select with the ACCEL branch GAUGED.**
///
/// At `k = 1` this dispatches to the parent and the whole family is bit-for-bit rung 77. The
/// comparison is `==` against [`GAUGE_K_IDENTITY`], which is Python's own `is`-exact test.
///
/// **THE GAUGED BRANCH DOES NOT CONSULT `sensed_cap`, AND THAT IS THE POINT OF ITS FIRST REFUSAL**
/// — see [`gauged_accel_cap`]. The parent's body does consult it, which is why `sensed_cap` had to
/// stay aimed at rung 76 in [`R78_TRIPLE`] while this cell moved.
fn r78_cap_fuel(
    ft: &FuelTransientCore, flight: &FlightCondition, a: f64, h: f64, mf_sched: f64,
    accel: Option<&AccelSchedule>, surge: Option<&Floor>, mf_app: Option<f64>,
) -> Result<f64, Abort> {
    if ft.inner.gauge_k.get() == GAUGE_K_IDENTITY {
        return (crate::stiffness_ledger::R77_TRIPLE.cap_fuel)(
            ft, flight, a, h, mf_sched, accel, surge, mf_app);
    }
    let mut caps: Vec<f64> = Vec::with_capacity(2);
    if let Some(accel) = accel {
        GAUGE_HITS.fetch_add(1, Ordering::Relaxed);
        caps.push(gauged_accel_cap(ft, flight, a, h, mf_sched, accel)?);
    }
    if let Some(surge) = surge {
        let phi = surge.phi();
        let big_gs = |w: f64| -> Result<f64, Abort> {
            Ok(phi.phi_lim - phi.read(&ft.try_instant_fuel(flight, a, h, w)?))
        };
        caps.push(cap_free(
            &big_gs, mf_sched, &|| ft.try_surge_fuel(flight, a, h, mf_sched, surge))?);
    }
    if caps.is_empty() {
        return Ok(f64::INFINITY);
    }
    let first = caps[0];
    let out = caps.iter().copied().reduce(|x, y| if y < x { y } else { x }).unwrap();
    if accel.is_some() && out == first {
        GAUGE_BINDS.fetch_add(1, Ordering::Relaxed);
    }
    Ok(out)
}

// =============================================================================================
// § 1 / § 2 — THE SET POINT, THE SLOPE, AND THE SENSITIVITY
// =============================================================================================

/// `cap(w)` for the accel leg — Python's `_accel_cap_fn`, and **the one cap in this family that
/// depends on the fuel it is asked about**, hence the only one that HAS a `c`.
///
/// Structurally the accel arm of [`residuals`](crate::stiffness_ledger::residuals) with the
/// `w −` removed: rung 77 wants a RESIDUAL, this rung wants the CAP itself, because the gauge is
/// applied to the cap and the residual is rebuilt around it.
pub fn accel_cap_fn<'a>(
    ft: &'a FuelTransientCore, flight: &'a FlightCondition, a: f64, h: f64,
    accel: &'a AccelSchedule,
) -> Box<dyn Fn(f64) -> Result<f64, Abort> + 'a> {
    let pi_b = ft.inner.inner.base.pi_b;
    Box::new(move |w: f64| {
        let i = ft.try_instant_fuel(flight, a, h, w)?;
        Ok(accel.cap(i.base.close.n_hp, i.base.close.pt4 / pi_b))
    })
}

/// The readers' refusal arm: rung 77's `.unwrap_or_else(|e| panic!("{}", e.0))`, generic so one
/// spelling serves every return type in § 1–§ 3. Python's readers catch nothing.
fn boom<T>(e: Abort) -> T {
    panic!("{}", e.0)
}

/// § 1's walk bounds and resolution — Python's `_root_count` defaults, which [`gauge_scan`] takes
/// by omission and [`root_census`] overrides (`n = 400`).
pub const ROOT_COUNT_LO: f64 = 0.2;
/// See [`ROOT_COUNT_LO`].
pub const ROOT_COUNT_HI: f64 = 3.0;
/// See [`ROOT_COUNT_LO`].
pub const ROOT_COUNT_N: usize = 120;
/// [`root_count`]'s refinement budget — a fixed iteration count, not a tolerance.
const ROOT_COUNT_BISECT: usize = 60;

/// **SIGN CHANGES OF `G` ON `[lo, hi]·w0` — A COUNT, NOT A SOLVE.** Python's `_root_count`.
///
/// § 3's whole content is that inside the collision band a solver converges cleanly onto the
/// WRONG root, so uniqueness cannot be tested with a solver: a root finder started anywhere
/// reports only the one it reached. The walk is therefore exhaustive on a grid.
///
/// # THE FALLIBILITY IS ASYMMETRIC IN THE SOURCE, AND THE PORT KEEPS IT THAT WAY
///
/// Python wraps **only the initial walk** in `except AssertionError`, storing `None` for a point
/// the model refuses; the 60-iteration bisection calls `G(xm)` **uncaught**, so a refusal there
/// propagates out of the reader entirely. Both arms are reproduced: `.ok()` on the walk, `?` in
/// the bisection. Making `G` uniformly fallible — the tidier Rust — would change behaviour at the
/// one site Python leaves propagating.
///
/// **AND BOTH ARMS WERE MEASURED BEFORE THEY WERE WRITTEN, WITH OPPOSITE ANSWERS.** Over the two
/// shipped grids the walk's refusal arm fires **6 171 of 39 930** points in [`gauge_scan`] and
/// **5 980 of 40 100** in [`root_census`] — a sixth of every walk, so the `None` sentinel is
/// load-bearing and a port that propagated there would abort on the first row. The bisection's
/// arm fires **0 of 25 200** and **0 of 9 720**. That zero is a property of THIS GRID and not of
/// the code — slice Q's rule — so it is recorded rather than relied on, and no gate can see a
/// wrong port of it.
///
/// The skip predicate is ported clause by clause in source order, including `g1 == 0.0`, which is
/// **not** redundant under a NaN `g2`: `0.0 * NaN` is `NaN` and `NaN >= 0.0` is false, so folding
/// it into the product test would bisect a bracket Python skips. Probe: it fires **0** times on
/// both grids and no residual is ever NaN there, so this clause too ships unexercised.
pub fn root_count(
    big_g: &dyn Fn(f64) -> Result<f64, Abort>, w0: f64, lo: f64, hi: f64, n: usize, locate: bool,
) -> Result<Vec<f64>, Abort> {
    let mut vals: Vec<(f64, Option<f64>)> = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let x = w0 * (lo + (hi - lo) * i as f64 / n as f64);
        vals.push((x, big_g(x).ok()));
    }
    let mut roots: Vec<f64> = Vec::new();
    for pair in vals.windows(2) {
        let ((mut x1, g1), (mut x2, g2)) = (pair[0], pair[1]);
        let (mut g1, g2) = match (g1, g2) {
            (Some(p), Some(q)) => (p, q),
            _ => continue,
        };
        if g1 == 0.0 || g1 * g2 >= 0.0 {
            continue;
        }
        if locate {
            for _ in 0..ROOT_COUNT_BISECT {
                let xm = 0.5 * (x1 + x2);
                // UNCAUGHT in Python — see this function's doc.
                let gm = big_g(xm)?;
                // Python also rebinds `g2` on the upper branch; it is never read again, so the
                // assignment is dropped here rather than kept as a dead binding.
                if g1 * gm <= 0.0 {
                    x2 = xm;
                } else {
                    x1 = xm;
                    g1 = gm;
                }
            }
        }
        roots.push(0.5 * (x1 + x2) / w0);
    }
    roots.sort_by(|a, b| a.partial_cmp(b).expect("§ 1 measured no NaN residual on either grid"));
    Ok(roots)
}

// ---------------------------------------------------------------------------------------------
// THE SIXTH KNOB'S TWO RESTORE POLICIES — **AND THEY DISAGREE INSIDE ONE CLASS**
// ---------------------------------------------------------------------------------------------

/// Python's `_with_gauge`: set `_gauge_k`, restore **the previous value** in a `finally`.
///
/// This is the policy [`gauge_scan`]'s inner solve hand-rolls (`engine.py:20394`
/// … `engine.py:20404`) and the one the declared helper `_with_gauge` implements. It is
/// correct under nesting.
pub struct GaugeRestored<'a> {
    core: &'a TwoSpoolTransientCore,
    prev: f64,
}

impl<'a> GaugeRestored<'a> {
    /// Set the gauge, remembering what was there.
    pub fn set(core: &'a TwoSpoolTransientCore, k: f64) -> Self {
        let prev = core.gauge_k.get();
        core.gauge_k.set(k);
        GaugeRestored { core, prev }
    }
}

impl Drop for GaugeRestored<'_> {
    fn drop(&mut self) {
        self.core.gauge_k.set(self.prev);
    }
}

/// [`root_census`]'s policy: set `_gauge_k`, restore **the literal identity** in a `finally`.
///
/// # THIS IS A CLOBBER, NOT A RESTORE, AND IT IS REACHABLE-WRONG INSIDE ONE CALL
///
/// `engine.py:20514` writes `m._gauge_k = 1.0`, not `m._gauge_k = prev`. Because the write lands
/// on the MARCHED machine and `_shared_rig` propagates the caller's gauge onto it
/// (`engine.py:20332`), a `root_census` entered at a non-identity gauge builds its FIRST row's cap
/// under that gauge and every LATER row under the identity — different plants inside one table.
/// Measured, at `k = 2.5`: the gauge seen at each row's `_accel_cap_fn` is `[2.5, 1.0, 1.0]` for
/// [`root_census`] and `[2.5, 2.5, …]` for [`gauge_scan`].
///
/// It is invisible today only because every shipped caller enters at the identity. **That is this
/// slice's own leading finding on a SECOND variable**: plan § 5.32 (i) found one hazard given
/// three treatments across these two classes on `_b_state`/`_v_state`, and its census was scoped
/// to that pair, so it could not see that the SIXTH DECLARED KNOB has the same shape — a declared
/// helper (`_with_gauge`) that would make it structural, used by § 4 and by neither of the two
/// sections that need it most, with one of them substituting a clobber for a restore.
///
/// Ported as written. The gate is `the_census_clobbers_the_gauge_where_the_scan_restores_it`,
/// which manufactures the nesting no shipped caller creates.
pub struct GaugeClobbered<'a> {
    core: &'a TwoSpoolTransientCore,
}

impl<'a> GaugeClobbered<'a> {
    /// Set the gauge, forgetting what was there.
    pub fn set(core: &'a TwoSpoolTransientCore, k: f64) -> Self {
        core.gauge_k.set(k);
        GaugeClobbered { core }
    }
}

impl Drop for GaugeClobbered<'_> {
    fn drop(&mut self) {
        self.core.gauge_k.set(GAUGE_K_IDENTITY);
    }
}

// ---------------------------------------------------------------------------------------------
// THE POINTS BOTH READERS SHARE
// ---------------------------------------------------------------------------------------------

/// Python's `_gauge_points` — **rung 77's own march, at rung 77's own settings, read at its own
/// points**, so § 1's `k = 1` column IS rung 77 § 1 and can be differenced against it.
///
/// Rung 63's lesson in one call: a number quoted from another rung's settings is not a comparison.
#[allow(clippy::too_many_arguments)]
pub fn gauge_points(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    margin: f64, taus: (f64, f64, f64, f64), r: f64, s_settle: f64, ds: f64, v_max: f64,
    inc: bool, phi_lim: f64, every: usize,
) -> (ScheduledStatorCore, Option<Floor>, AccelSchedule, Vec<FuelPoint>) {
    let sm = phi_lim / core.arming().map_lp_design.phi_surge - 1.0;
    let (m, surge, _lag, traj, accel) = crate::stiffness_ledger::ledger_march(
        core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max, inc, margin);
    let b_max = m.fuel.inner.lever.lim.expect("`_shared_rig` arms the valve").b_max;
    // Python slices AFTER filtering (`_riding4(...)[::every]`), so the stride runs over the
    // FILTERED list and not over the trajectory.
    let pts: Vec<FuelPoint> = riding4(&traj, b_max).into_iter().step_by(every).collect();
    (m, surge, accel, pts)
}

/// ONE gauge's reading at one riding point — Python's `row["ks"][mult]`.
///
/// **A `Vec` and not a map**, because Python's dict is keyed by the sweep multiple and read in
/// insertion order everywhere it is read.
#[derive(Clone, Copy, Debug)]
pub struct GaugeCell {
    /// The sweep multiple. `k` is swept in multiples of `1/c`, **each point's own**, so the
    /// singular gauge sits at `k·c = 1` in every row.
    pub mult: f64,
    /// `mult / c`, or an exact `0.0` — Python's `mult / c if mult else 0.0` is a FALSY test on a
    /// float, so `mult = 0.0` short-circuits to a literal rather than dividing.
    pub k: f64,
    /// The gauged set point.
    pub w: f64,
    /// Whether this reading AND both of its `q ± dq` neighbours converged. **Not a correctness
    /// guard** — see [`gauge_root`].
    pub ok: bool,
    /// `G_w` at the anchor.
    pub gw: f64,
    /// `1 − k·c` — the prediction `gw` is scored against.
    pub gw_pred: f64,
    /// The `k = 1` root this gauge is anchored at.
    pub anchor: f64,
    /// How many roots the residual has here — [`root_count`], and the EXCLUSION's only input.
    pub n_roots: usize,
    /// `|w − w1| / |w1|` — P1's quantity.
    pub w_move: f64,
    /// `|gw − gw_pred| / max(|gw_pred|, 1e-30)` — P2's.
    pub gw_err: f64,
    /// `dw*/dq`, taken by RE-SOLVING the whole gauged set point at `q ± dq` rather than by
    /// reading `−G_q/G_w`, which would be computing the formula under test.
    pub direct: f64,
    /// **MEASURED, NOT CHOSEN**: excluded iff the residual is multi-rooted at this gauge or at
    /// either of its two neighbours.
    pub excluded: bool,
    /// `|direct − base| / max(|base|, 1e-30)` — P3's, filled in after the row's `base` is taken.
    pub gain_move: f64,
}

/// One riding point of [`gauge_scan`].
#[derive(Clone, Debug)]
pub struct GaugeRow {
    /// Path distance along the march.
    pub s: f64,
    /// Rung 76's `c` here, read by [`c_at`](crate::sensed_cap::c_at) at the `k = 1` set point.
    pub c: f64,
    /// `1/c` — the gauge at which `G_w` vanishes.
    pub k_crit: f64,
    /// The `k = 1` set point.
    pub w1: f64,
    /// `G_w` at it.
    pub gw1: f64,
    /// **THE NON-VACUITY ANCHOR**: `|(1 − gw1) − c|`. The `k = 1` column must BE rung 77 § 1's
    /// `G_a'` and rung 76's `c`, or this sweep is measuring its own solver.
    pub c_err: f64,
    /// The sweep, in order.
    pub ks: Vec<GaugeCell>,
    /// `dw*/dq` at `k = 1`, taken by the SAME `at` the sweep uses so the two share a differencing
    /// floor — rung 77 § 2.2's reason.
    pub base: f64,
}

/// § 1 / § 2's whole reading — Python's `gauge_scan` return dict.
#[derive(Clone, Debug)]
pub struct GaugeScan {
    pub phi_lim: f64,
    pub margin: f64,
    pub inc: bool,
    pub n: usize,
    pub rows: Vec<GaugeRow>,
    /// How many readings the multi-root test dropped, and how many it kept. **A rung that drops
    /// points must say what it dropped.**
    pub n_excluded: usize,
    /// See [`n_excluded`](Self::n_excluded).
    pub n_kept: usize,
    /// Which multiples were dropped, as a SET — Python's `sorted({...})`.
    pub excluded_mults: Vec<f64>,
    /// The worst `w_move` among the dropped. If the dropped points were harmless, excluding them
    /// bought the hold for nothing and § 1.2 is wrong.
    pub excluded_worst: Option<f64>,
    /// Worst [`c_err`](GaugeRow::c_err) over the rows.
    pub c_err: Option<f64>,
    /// **P1**: the set point does not move, anywhere outside the collision band.
    pub w_move: Option<f64>,
    /// **P2**: the slope IS `1 − k·c` …
    pub gw_err: Option<f64>,
    /// … and it spans BOTH signs.
    pub gw_span: Option<(f64, f64)>,
    /// See [`gw_span`](Self::gw_span).
    pub sign_change: Option<bool>,
    /// **P3 — THE RUNG**: and neither does the sensitivity, so the singularity is REMOVABLE.
    pub gain_move: Option<f64>,
    /// Kept readings whose solve did not converge.
    pub n_bad: usize,
    /// The span of rung 76's `c` over the march.
    pub c: Option<(f64, f64)>,
}

/// [`gauge_scan`]'s default sweep — and it **DELIBERATELY includes `1.05` and `1.1`**, where the
/// anchor's refuted `1e-3` window said nothing is wrong and § 3 says the residual is multi-rooted.
/// A sweep that stepped over them would report a clean hold by choosing where to look.
pub const GAUGE_SCAN_MULTS: [f64; 10] = [-0.5, 0.0, 0.25, 0.5, 0.9, 1.05, 1.1, 1.5, 2.0, 3.0];

/// [`gauge_scan`]'s perturbation for `dw*/dq`.
pub const GAUGE_SCAN_DQ: f64 = 1e-5;

/// § 1 / § 2: **`w*(k)`, `G_w(k)` and `dw*/dq(k)` at each riding point.** Python's `gauge_scan`.
///
/// # WHAT IS EXCLUDED IS MEASURED, NOT CHOSEN
///
/// The anchor registered the exclusion as `|1 − k·c| < 1e-3`, and that number is **REFUTED** — the
/// disturbed region is about 2.5 decades wider. Widening it to a round `0.30` would have been a
/// tuned pass wearing a pre-registered threshold's clothes, so the exclusion here is not a width
/// at all: a point is excluded iff `G_k` has MORE THAN ONE ROOT there, counted by [`root_count`].
/// § 1 then has no constant that could cover for § 3's content — the band is an OUTPUT of the
/// sweep rather than an input to it.
///
/// # EVERY READING IS TAKEN INSIDE ITS OWN FREEZE BLOCK, AND THAT IS THE MIRROR OF § 3 OF RUNG 77
///
/// [`singular_limit`](crate::stiffness_ledger::singular_limit) DEPENDS on a residual outliving its
/// block; this reader depends on the opposite. `at` rebuilds the cap, the anchor, the residual and
/// the root INSIDE each `qq`'s own block, because a closure carried out of one reads the plant with
/// the valve loop CLOSED whatever `qq` it was asked about — and it then returns a clean
/// `1.000e+00`. The first version of § 3 of this rung walked into exactly that and read a broken
/// identity that was a difference between two PLANTS.
///
/// **So here the natural Rust shape is the CORRECT one, which is why the gate has to prove it
/// rather than rely on it.** The discriminator is already in the arithmetic: a leaked freeze makes
/// `at` ignore `qq`, so `hi_` and `lo_` would agree bit-for-bit and [`direct`](GaugeCell::direct)
/// would be an exact `0.0`. Measured on the shipped grid it is `−2.64e-4` at every gauge in the
/// first row, and the gate reads the frozen cells at the instant the residual is built as well.
#[allow(clippy::too_many_arguments)]
pub fn gauge_scan(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, margin: f64, taus: (f64, f64, f64, f64), inc: bool, r: f64, s_settle: f64,
    ds: f64, v_max: f64, dq: f64, every: usize, mults: &[f64],
) -> GaugeScan {
    let (m, _surge, accel, pts) = gauge_points(
        core, flight, tt4_lo, tt4_hi, tt4_max, margin, taus, r, s_settle, ds, v_max, inc,
        phi_lim, every);
    let mut rows: Vec<GaugeRow> = Vec::new();
    for p in pts.iter() {
        let (a, h, ms) = (p.nu_lp, p.nu_hp, p.mf_sched);
        let (q, v) = crate::stiffness_ledger::bv_of(p);

        // THE WHOLE GAUGED SOLVE on the plant AS THE VALVE IS AT `qq`. Everything is rebuilt in
        // here; nothing escapes. The gauge guard is declared LAST so it drops FIRST, which is
        // Python's nesting (the inner `finally` restores `_gauge_k`, the outer clears the freeze).
        let at = |qq: f64, k: f64| -> Result<(f64, bool, f64, f64, usize), Abort> {
            let _sb = MarchedBleed::set(&m.fuel.inner, qq);
            let _sv = MarchedStator::set(&m.fuel.inner, v);
            let cap = accel_cap_fn(&m.fuel, flight, a, h, &accel);
            let g0 = |w: f64| -> Result<f64, Abort> { Ok(w - cap(w)?) };
            let w0 = cap_free(&g0, ms, &|| m.fuel.try_sched_fuel(flight, a, h, ms, &accel))?;
            let _gk = GaugeRestored::set(&m.fuel.inner, k);
            // Python's `_gauge_residual` reads `self._gauge_k` at BUILD time, so the knob is read
            // back out of the cell here rather than passed straight through from `k`.
            let big_g = gauge_residual(m.fuel.inner.gauge_k.get(), &*cap, w0);
            let (w, ok) = gauge_root(&*big_g, ms, GAUGE_ROOT_REL, GAUGE_ROOT_N, GAUGE_ROOT_TRUST);
            // THE SLOPE IS READ AT THE ANCHOR, which is a root at every gauge by construction.
            // Reading it at `w` would report the slope at whatever root the solver reached, and
            // inside the band that is the other one.
            let gw = crate::stiffness_ledger::slope_at(&*big_g, w0, SLOPE_AT_REL)?;
            let nr = root_count(&*big_g, w0, ROOT_COUNT_LO, ROOT_COUNT_HI, ROOT_COUNT_N, true)?
                .len();
            Ok((w, ok, gw, w0, nr))
        };
        // Python discards `ok1` and `w01`; only `w1` and `gw1` reach the row.
        let (w1, _ok1, gw1, _w01, _nr1) = at(q, 1.0).unwrap_or_else(boom);
        let c = {
            // The freeze here is Python's, and it is REDUNDANT: `c_at` sets both cells itself and
            // clears them in its own `finally`. Kept because the source keeps it.
            let _sb = MarchedBleed::set(&m.fuel.inner, q);
            let _sv = MarchedStator::set(&m.fuel.inner, v);
            c_at(&m, flight, a, h, &accel, w1, q, v, C_AT_REL).unwrap_or_else(boom)
        };
        let mut ks: Vec<GaugeCell> = Vec::with_capacity(mults.len());
        for &mult in mults {
            let k = if mult != 0.0 { mult / c } else { 0.0 };
            let (w, ok, gw, w0, nr) = at(q, k).unwrap_or_else(boom);
            let hi_ = at(q + dq, k).unwrap_or_else(boom);
            let lo_ = at(q - dq, k).unwrap_or_else(boom);
            let pred = 1.0 - k * c;
            // Python's `max(abs(1 - k*c), 1e-30)` — argument 0 is an EXPRESSION, so the fold is
            // written out rather than spelled `f64::max`.
            let ap = pred.abs();
            let den = if 1e-30 > ap { 1e-30 } else { ap };
            ks.push(GaugeCell {
                mult,
                k,
                w,
                ok: ok && hi_.1 && lo_.1,
                gw,
                gw_pred: pred,
                anchor: w0,
                n_roots: nr,
                w_move: (w - w1).abs() / w1.abs(),
                gw_err: (gw - pred).abs() / den,
                direct: (hi_.0 - lo_.0) / (2.0 * dq),
                // Integers, so `max` needs no expression-first care here.
                excluded: nr.max(hi_.4).max(lo_.4) > 1,
                gain_move: f64::NAN,
            });
        }
        let base = (at(q + dq, 1.0).unwrap_or_else(boom).0
            - at(q - dq, 1.0).unwrap_or_else(boom).0) / (2.0 * dq);
        let ab = base.abs();
        let bden = if 1e-30 > ab { 1e-30 } else { ab };
        for d in ks.iter_mut() {
            d.gain_move = (d.direct - base).abs() / bden;
        }
        rows.push(GaugeRow {
            s: p.s,
            c,
            k_crit: 1.0 / c,
            w1,
            gw1,
            c_err: ((1.0 - gw1) - c).abs(),
            ks,
            base,
        });
    }
    let keep: Vec<&GaugeCell> = rows.iter().flat_map(|x| x.ks.iter()).filter(|d| !d.excluded)
        .collect();
    let drop: Vec<&GaugeCell> = rows.iter().flat_map(|x| x.ks.iter()).filter(|d| d.excluded)
        .collect();
    let mut excluded_mults: Vec<f64> = drop.iter().map(|d| d.mult).collect();
    excluded_mults.sort_by(|a, b| a.partial_cmp(b).expect("the sweep's multiples are literals"));
    excluded_mults.dedup();
    let pick = |xs: Vec<f64>| -> Option<f64> {
        if xs.is_empty() { None } else { Some(py_max_of(&xs)) }
    };
    let gws: Vec<f64> = keep.iter().map(|d| d.gw).collect();
    GaugeScan {
        phi_lim,
        margin,
        inc,
        n: rows.len(),
        n_excluded: drop.len(),
        n_kept: keep.len(),
        excluded_mults,
        excluded_worst: pick(drop.iter().map(|d| d.w_move).collect()),
        c_err: pick(rows.iter().map(|x| x.c_err).collect()),
        w_move: pick(keep.iter().map(|d| d.w_move).collect()),
        gw_err: pick(keep.iter().map(|d| d.gw_err).collect()),
        gw_span: if gws.is_empty() { None } else { Some((py_min_of(&gws), py_max_of(&gws))) },
        sign_change: if keep.is_empty() {
            None
        } else {
            Some(keep.iter().any(|d| d.gw < 0.0) && keep.iter().any(|d| d.gw > 0.0))
        },
        gain_move: pick(keep.iter().map(|d| d.gain_move).collect()),
        n_bad: keep.iter().filter(|d| !d.ok).count(),
        c: if rows.is_empty() {
            None
        } else {
            let cs: Vec<f64> = rows.iter().map(|x| x.c).collect();
            Some((py_min_of(&cs), py_max_of(&cs)))
        },
        rows,
    }
}

// =============================================================================================
// § 3 — THE SECOND ROOT, AND WHERE IT COLLIDES
// =============================================================================================

/// One gauge's walk at one riding point — Python's `cells[mult]`.
#[derive(Clone, Debug)]
pub struct CensusCell {
    pub mult: f64,
    pub k: f64,
    /// `G_k(w0)`. **The construction, CHECKED**: `w0` is a root at every gauge, and this is the
    /// one thing that would make §§ 1–2 meaningless if it were false.
    pub g_at_w0: f64,
    pub n_roots: usize,
    /// Every root, as a fraction of the anchor.
    pub roots: Vec<f64>,
    /// The roots that are NOT the true one — everything further than `1e-6` from `1.0`.
    pub spurious: Vec<f64>,
}

/// One riding point of [`root_census`].
#[derive(Clone, Debug)]
pub struct CensusRow {
    pub s: f64,
    pub w0: f64,
    pub c: f64,
    pub k_crit: f64,
    pub cells: Vec<CensusCell>,
}

/// § 3's whole reading — Python's `root_census` return dict.
#[derive(Clone, Debug)]
pub struct RootCensus {
    pub phi_lim: f64,
    pub margin: f64,
    pub inc: bool,
    pub n: usize,
    pub rows: Vec<CensusRow>,
    /// Worst `|G_k(w0)|` anywhere — the construction, checked rather than assumed.
    pub g_at_w0: Option<f64>,
    /// Whether the walk found the true root at every gauge. Python's `all` over an EMPTY sequence
    /// is `True`, so an empty census reports `true` here — reproduced.
    pub true_found: bool,
    /// The distinct root counts seen, as a sorted SET.
    pub n_roots: Vec<usize>,
    /// The multiples at which the residual is multi-rooted.
    pub multi_mults: Vec<f64>,
    /// Their span …
    pub band: Option<(f64, f64)>,
    /// … and whether it BRACKETS the singular gauge. If it does not, the collision is not at
    /// `k·c = 1` and § 3's mechanism is wrong.
    pub brackets: Option<bool>,
    /// How close the spurious root gets to the true one, at the band's edges.
    pub approach: Option<f64>,
}

/// [`root_census`]'s default sweep — denser around `1.0` than [`GAUGE_SCAN_MULTS`], because this
/// section is looking for the COLLISION rather than for the slope.
pub const ROOT_CENSUS_MULTS: [f64; 10] = [0.5, 0.9, 0.99, 1.01, 1.05, 1.1, 1.2, 1.5, 2.0, 3.0];

/// [`root_census`]'s walk resolution — three times [`ROOT_COUNT_N`], because a collision is only
/// visible while the two roots are still resolvable on the grid.
pub const ROOT_CENSUS_N: usize = 400;

/// § 3: **the gauge preserves the root and destroys its UNIQUENESS.** Python's `root_census`.
///
/// `G_k(w0)` must be ZERO at every `k` — that is the construction, and it is checked rather than
/// assumed. Then the residual is walked across `[lo, hi]·w0` and its sign changes COUNTED, because
/// a root finder started anywhere would only ever report the one it happened to reach, which is
/// exactly the failure this section explains.
///
/// # THE RE-FREEZE AT `engine.py:20504` IS NOT OPTIONAL, AND THE CLOBBER AT `engine.py:20514` IS NOT A RESTORE
///
/// Two same-shaped hazards, one loop apart, treated differently:
///
/// * `c_at` clears both frozen cells in its own `finally`, and this block continues into real
///   plant work afterwards, so the freeze is **re-armed**. Plan § 5.32 (i) site 3. Rung 77's
///   `leg_slopes` omits the same re-arm and is safe only because the single statement after its
///   own nested call is arithmetic over numbers already computed.
/// * the gauge, by contrast, is restored to the **literal identity** rather than to what was
///   there — see [`GaugeClobbered`], where it is measured to be reachable-wrong inside one call.
#[allow(clippy::too_many_arguments)]
pub fn root_census(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, margin: f64, taus: (f64, f64, f64, f64), inc: bool, r: f64, s_settle: f64,
    ds: f64, v_max: f64, every: usize, mults: &[f64], lo: f64, hi: f64, n: usize,
) -> RootCensus {
    let (m, _surge, accel, pts) = gauge_points(
        core, flight, tt4_lo, tt4_hi, tt4_max, margin, taus, r, s_settle, ds, v_max, inc,
        phi_lim, every);
    let mut rows: Vec<CensusRow> = Vec::new();
    for p in pts.iter() {
        let (a, h, ms) = (p.nu_lp, p.nu_hp, p.mf_sched);
        let (q, v) = crate::stiffness_ledger::bv_of(p);
        let _sb = MarchedBleed::set(&m.fuel.inner, q);
        let _sv = MarchedStator::set(&m.fuel.inner, v);
        let cap = accel_cap_fn(&m.fuel, flight, a, h, &accel);
        let g0 = |w: f64| -> Result<f64, Abort> { Ok(w - cap(w)?) };
        let w0 = cap_free(&g0, ms, &|| m.fuel.try_sched_fuel(flight, a, h, ms, &accel))
            .unwrap_or_else(boom);
        let c = c_at(&m, flight, a, h, &accel, w0, q, v, C_AT_REL).unwrap_or_else(boom);
        // `c_at` un-freezes in its OWN `finally`, and this block keeps going — so re-arm.
        let _sb2 = MarchedBleed::set(&m.fuel.inner, q);
        let _sv2 = MarchedStator::set(&m.fuel.inner, v);
        let mut cells: Vec<CensusCell> = Vec::with_capacity(mults.len());
        for &mult in mults {
            let k = mult / c;
            let (at_w0, roots) = {
                let _gk = GaugeClobbered::set(&m.fuel.inner, k);
                let big_g = gauge_residual(m.fuel.inner.gauge_k.get(), &*cap, w0);
                let at_w0 = big_g(w0).unwrap_or_else(boom);
                (at_w0, root_count(&*big_g, w0, lo, hi, n, true).unwrap_or_else(boom))
            };
            cells.push(CensusCell {
                mult,
                k,
                g_at_w0: at_w0,
                n_roots: roots.len(),
                spurious: roots.iter().copied().filter(|x| (x - 1.0).abs() > 1e-6).collect(),
                roots,
            });
        }
        rows.push(CensusRow { s: p.s, w0, c, k_crit: 1.0 / c, cells });
    }
    let allc: Vec<&CensusCell> = rows.iter().flat_map(|x| x.cells.iter()).collect();
    let mut n_roots: Vec<usize> = allc.iter().map(|d| d.n_roots).collect();
    n_roots.sort_unstable();
    n_roots.dedup();
    let mut multi: Vec<f64> = allc.iter().filter(|d| d.n_roots > 1).map(|d| d.mult).collect();
    multi.sort_by(|a, b| a.partial_cmp(b).expect("the sweep's multiples are literals"));
    multi.dedup();
    let spur: Vec<f64> = allc.iter()
        .flat_map(|d| d.spurious.iter().map(|x| (x - 1.0).abs()))
        .collect();
    RootCensus {
        phi_lim,
        margin,
        inc,
        n: rows.len(),
        g_at_w0: if allc.is_empty() {
            None
        } else {
            Some(py_max_of(&allc.iter().map(|d| d.g_at_w0.abs()).collect::<Vec<f64>>()))
        },
        // Python's `all(...)` over an empty sequence is `True`.
        true_found: allc.iter().all(|d| d.roots.iter().any(|x| (x - 1.0).abs() <= 1e-6)),
        n_roots,
        band: if multi.is_empty() { None } else { Some((multi[0], multi[multi.len() - 1])) },
        brackets: if multi.is_empty() {
            None
        } else {
            Some(multi[0] < 1.0 && 1.0 < multi[multi.len() - 1])
        },
        multi_mults: multi,
        approach: if spur.is_empty() { None } else { Some(py_min_of(&spur)) },
        rows,
    }
}
