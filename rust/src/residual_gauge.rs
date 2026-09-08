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
//! attributes both frames to its FIRST line, so the tally shows a single key at `:20591` holding
//! 20 — ten central differences, twenty nests. The static census sees two calls; the counter sees
//! one site. They agree, and the row above says so rather than quoting 20 twice.
//! **The source handles the hazard three incompatible ways in one slice** — `leg_slopes` nests and
//! never re-freezes (safe only because the one statement after it is arithmetic over numbers
//! already computed); `root_census` nests and MUST re-freeze, and does, at `:20504`; and
//! `_c_on_frozen` closes its block BEFORE calling `_c_at` at all, its docstring recording that an
//! earlier version did not and shipped wrong.
//!
//! **THE CRITERION THAT MAKES THE PORT SAFE IS THE DEAD WINDOW, NOT THE `(q, v)` AGREEMENT.** The
//! first writing of this claim said each callee receives the enclosing block's own `(q, v)`. That
//! is FALSE at two of the six: `gauge_vs_device` hands `_phi_at` **`q ± dq`**, so there the two
//! restore semantics genuinely leave different values behind. What holds at all six is weaker and
//! checked per site: **the window in which the two semantics disagree is DEAD** — between an inner
//! freezer's return and the next set or `finally`, nothing reads the plant. Verified by reading
//! all six: `:19829` is followed by one arithmetic assignment; `:20412` and `:20691` by their
//! `finally`; `:20503` by the re-freeze at `:20504`; `:20591`/`:20592` are the two halves of one
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
//! NEW code: `_phi_at`, sites `:20591`/`:20592`, the two where the callee is handed `q ± dq` and the
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
    AccelSchedule, AsymmetricLag, Floor, FuelTransientCore, FuelTransientHooks,
};
use crate::gas::Abort;
use crate::map::ComponentMap;
use crate::shared_actuator::SharedRigArm;
use crate::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient, StatorTransientHooks};
use crate::three_loop::TripleHooks;
use crate::two_spool::TwoSpoolEngine;
use crate::two_spool_transient::TwoSpoolTransientHooks;

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
fn gauge_residual<'a>(
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
fn gauge_root(
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
const GAUGE_ROOT_REL: f64 = 1e-10;
const GAUGE_ROOT_N: usize = 400;
const GAUGE_ROOT_TRUST: f64 = 0.25;

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
