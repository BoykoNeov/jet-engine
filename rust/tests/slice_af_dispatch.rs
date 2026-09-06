//! SLICE AF step 6 — **TEN RE-AIMED POINTERS, AND FOUR PRODUCTION CALL SITES THAT DISPATCHED A
//! WRITE PYTHON MAKES BY PLAIN ASSIGNMENT.**
//!
//! Ten injections: this slice's **eight** cells (four ADD, four SWAP), plus **two counterfeits**
//! that exist to answer questions the eight cannot — a coordinate setter pointed at another
//! carrier (rung 79's shape, § 5) and rung 69's `_with_ref` (the control that makes *the
//! reference pins are inert* a measurement rather than an argument).
//!
//! `slice_af_oracle.rs` is green on 20 643 compared keys per arm against two interpreters,
//! `rung74.rs` carries the 17 ported gates, and `slice_af_cells.rs` / `slice_af_laws.rs` /
//! `slice_af_march.rs` gate the cell BODIES, the six demand laws and the six-state march. All
//! five are VALUE instruments, and none of them can see which FUNCTION POINTER sat in a slot:
//! every one of them reads a machine this rung's own builder assembled. That is this file's
//! subject, and § 5.30 (v)'s step 6.
//!
//! # OBLIGATION 1 — **§ 5.30 (ii)'s SEPARATE-FIELD GATE, THROUGH A REAL READER**
//!
//! `_with_coord` has **exactly two definers, rungs 74 and 79** (AST census, § 5.30 (ii)), and the
//! two write **different fields** — `_lag_coord` here, `_phi_ref` there. Both fields exist on a
//! rung-79 machine, so no signature comparison and no type error can reach the difference; the
//! only instrument that can is a DISPATCH.
//!
//! `slice_af_cells.rs` already gates the field two-sidedly — but it calls the cell BY HAND. § (ii)
//! asks the harder question: *is the slot READ, and by what?* — which needs the pointer replaced
//! and a shipped reader run over it.
//!
//! # OBLIGATION 2 — **P5, AND IT NEEDS TWO SIDES**
//!
//! § 5.30 (i) measured `_with_coord`'s one reader (`_demand_target`) arithmetically the identity
//! everywhere its one call site looks — `cap <= mf_sched` at 1 040 of 1 040 and 624 of 624 calls
//! — and predicted (**P5**) that the dispatch step would find the cell **unobservable by value**
//! and gate it structurally.
//!
//! **A SILENT ROW IS NOT THAT MEASUREMENT.** AE's own two all-silent rows are the recorded shape,
//! and § 5.30 (vii) item 2 is the recorded criticism of believing one. So the evidence is THREE-
//! sided: § 4 shows the reader can discriminate (a manufactured `cap > mf_sched` pair) and that
//! the plant REACHES that region (34 of 682 caps over the demand march's two legs), and § 5 then
//! reads the zero through the shipped reader itself. A zero taken from a population the effect
//! cannot be in is the shape this file's own first writing shipped and § 4 now names.
//!
//! # OBLIGATION 3 — **THE SEAT MATRIX, RUN WHOLE**
//!
//! AD step 6's rule, kept: with a parent-pointer injection a *silence* is either laundering (the
//! cell ran, on a machine `at_lever` rebuilt around the shipped tables) or a path that never
//! reaches the cell, and a did-it-break instrument cannot tell those apart. What separates them
//! is the OTHER seats in the same ROW, so § 3 runs all **ten** pointers against all **seven**
//! seats and prints the tally.
//!
//! # THE LEADING FINDING — **FOUR SITES THAT DISPATCH A PLAIN ASSIGNMENT, AND ONE OF THEM IS
//! REACHED BY A SHIPPED PYTHON TEST SIX RUNGS UP**
//!
//! Python writes the two knobs by PLAIN ASSIGNMENT in three places at this rung:
//!
//! | site | Python | what it writes |
//! |---|---|---|
//! | `at_lever` (`engine.py:17711`) | `m._lag_coord = self._lag_coord` | the field |
//! | `_shared_rig` (`:17722`) | `m._lag_coord = self._lag_coord` | the field |
//! | `_coord_march` (`:18031`) | `m._lag_coord, m._ref_law = coord, ref` | the field |
//! | `demand_gains` (`:18267`) | `m._lag_coord, m._ref_law = "clip", "sched"` | the field |
//!
//! and dispatches through `_with_coord` in exactly ONE place — `demand_gains`'s
//! `m._with_coord("demand", m._demand_gains_at, …)` at `:18276`, which is a SCOPE.
//!
//! **The port had the first two right and the last two wrong.** [`r74_at_lever`] and
//! [`r74_shared_rig`] spell it `lag_coord.set(…)`; `coord_march` and `demand_gains` routed the
//! same construct through `(triple_hooks.with_coord)(…)`, four hundred lines away — and
//! `rung74.rs`'s own `demand_rig` fixture states the rule the production code broke: *the two
//! knobs are set by PLAIN ASSIGNMENT … not through `_with_coord`. Routing them through the table
//! here would test a different line.* Three spellings against two, inside one slice.
//!
//! **AT RUNG 74 THE TWO SPELLINGS ARE THE SAME FUNCTION**, which is why every value instrument in
//! the slice is blind to it: `r74_with_coord` writes `lag_coord` and returns the displaced value
//! the pins discard. **AT RUNG 79 THEY ARE NOT.** `_coord_march`, `demand_law`, `demand_gains`,
//! `latch_discriminator`, `windup_law`, `flat_schedule_identity` and `forcing_openloop` are all
//! **single-definer** — rung 74's, inherited unchanged by rungs 75–84 — and
//! `tests/test_rung80.py:110` calls `m._coord_march(…, coord="demand", …)` on a
//! **`SplitWallTransient`**, which carries rung 79's `_with_coord`. Python writes `_lag_coord`
//! there and marches the demand arm; the port would have written `_phi_ref`, left the coordinate
//! at the class default `"clip"`, and marched **rung 73** — while that test's own
//! `len(r0) == len(r1) == 341` went on passing, because the clip march returns 341 points too.
//!
//! So it is a SILENT divergence, planted six rungs ahead, reachable by a shipped test, and
//! invisible to bit-exactness at the rung that introduced it. **The four sites are fixed in this
//! commit**, and the fix is a measured 2 + 2 partition rather than one uniform tidy-up:
//!
//! * the two `with_coord` pins are a **LATENT DIVERGENCE** — a second definer re-aims the field;
//! * the two `with_ref` pins are **STRUCTURALLY WRONG BUT INERT** — `_with_ref` has exactly two
//!   definers (rungs 69 and 73) and every machine at rung ≥ 74 carries rung 73's, so the cell and
//!   the raw write agree forever. Fixed anyway, and § 3's `WithRefR69` row is what makes *inert*
//!   a measurement instead of an argument.
//!
//! The crate's own rule — [`CoordScope`]'s *dispatch the setter iff a later rung overrides
//! `_with_*` to write a DIFFERENT field* — is correct and was **over-applied**: it governs the
//! `_with_coord` METHOD, not every write of `_lag_coord`.
//!
//! # THE FIX'S REACH, MEASURED — the matrix was run BEFORE the four lines were touched
//!
//! Seats in [`SEATS`] order. Only the four rows that moved are shown; the other six are cell for
//! cell identical, which is what makes these four attributable.
//!
//! | injection | before | after |
//! |---|---|---|
//! | `WithCoordElsewhere` | `same DIFF DIFF DIFF DIFF DIFF DIFF` — **6 of 7** | `same` x7 |
//! | `WithCoordParent` | `BROKE DIFF BROKE BROKE DIFF BROKE BROKE` — **7 of 7** | `BROKE` at `demand_gains` only |
//! | `WithRefR69` | one `DIFF`, at `windup_law` | `same` x7 |
//! | `AtLever` | `BROKE DIFF BROKE BROKE DIFF BROKE BROKE` | `DIFF DIFF BROKE DIFF BROKE DIFF same` |
//!
//! **THE FOURTH ROW IS THE ONE THAT WAS NOT PREDICTED, AND IT IS THE SECOND HALF OF THE FINDING.**
//! The registered prediction was *the other seven rows are unchanged, cell for cell*; `AtLever`
//! falsifies it, and the reason sharpens the diagnosis rather than softening it. A dispatched pin
//! on a machine whose table does not carry the cell hits the parent slot's **panic** — so the port
//! was turning a benign Python attribute assignment into a hard REFUSAL: `m._lag_coord = coord`
//! succeeds on any Python object, including a rung-73 one that never reads the attribute. Four of
//! `AtLever`'s seven seats went from refusing to READING once the pins became assignments.
//!
//! So the four sites were not merely spelling a no-op the long way. They changed **which
//! configurations are answerable at all**, in the direction of refusing more than Python does —
//! and no value instrument in the slice could see either half, because at rung 74 with rung 74's
//! own table the two spellings are the same function.
//!
//! # HOW AN INJECTION IS INSTALLED, AND WHY THERE IS NO HIDDEN STATE
//!
//! Every reader on this ladder rebuilds its machine through `_shared_rig`, which reaches
//! `at_lever`, and `build_demand_coordinate_cascade` hardcodes `&R74`/`&R74_FUEL`/`&R74_TRIPLE` —
//! so a lone table injection is LAUNDERED before any value is read (AC step 7's finding).
//! [`injection!`] therefore emits, per cell, **its own `at_lever` naming its own tables**. There
//! is no selector, no thread-local and no run-order coupling.
//!
//! [`triple_diff`] proves the install by an **exhaustive destructuring** of [`TripleHooks`], so
//! the claim *only this slot differs* is checked against all **eighteen** fields and goes `E0027`
//! when a nineteenth lands — `slice_af_cells.rs`'s tripwire, which fired for real when this slice
//! took the table from 14 to 18.
//!
//! # THE FINGERPRINT, AND ITS BLIND SPOTS — DECLARED
//!
//! The seven seats return seven different types, so a reading is fingerprinted by its `Debug`
//! string. Rust prints an `f64` as the shortest decimal that round-trips, so the map is injective
//! on every finite `f64` and separates `-0.0` from `0.0`. It collapses NaN payloads, which is the
//! one difference it cannot see. [`the_matrix_instrument_moves_when_the_plant_does`] perturbs the
//! plant and requires every seat to move, so the instrument proves it can see before it is read
//! ([[rust-port-slice-w-step3]]).
//!
//! **AND THE MATRIX IS BASELINE-RELATIVE**, so a source change that moves the baseline and all
//! ten rows together is invisible to it by construction. That is a property of the instrument and
//! it is why § 4 and § 6 do not go through the matrix.
//!
//! **THE FIXTURE'S OWN `at_lever` COPIES BOTH KNOBS**, on the line after it rebuilds — it must,
//! or a sibling built under a scoped coordinate would read the class default and every row would
//! measure that instead of the injection. So this file re-implements the statement a mutation of
//! the SHIPPED carry deletes, and no seat here can see such a mutation. `slice_af_cells.rs` § 4
//! owns that one, on the shipped machine.
//!
//! # WHAT THIS FILE DOES NOT GATE, AND WHY EACH IS A DECISION
//!
//! * **The cell BODIES.** `slice_af_cells.rs` owns the four ADDs, the five split refusals and the
//!   two carriers; `rung74.rs` owns the 17 ported claims. This file asks only whether a slot is
//!   READ at a seat, and what the answer is worth.
//! * **A CPython arm.** Nothing here reads a golden. Every assertion is a panic, a same-run
//!   difference, or a compile-time property.
//! * **`_ic_cap`'s carry.** Rung 74's `at_lever` does not copy it and rung 75's does
//!   (`engine.py:17711` against `:18671`); nothing at this rung writes the field, so the copy
//!   would be invisible. `slice_af_cells.rs` records it; this file adds nothing.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::fn_addr_eq;

use turbojet::applied_reference::{
    build_applied_reference_cascade, REF_LAWS_DECLARED, REF_LAW_APPLIED, R73, R73_FUEL,
    R73_TRIPLE,
};
use turbojet::bleed_transient::{LeverArm, LeverArming, LeverHooks};
use turbojet::demand_coordinate::{
    build_demand_coordinate_cascade, coord_march, demand_gains, demand_law, demand_target,
    flat_schedule_identity, forcing_openloop, latch_discriminator, windup_law, CoordScope,
    LAG_COORDS_DECLARED, LAG_COORD_CLIP, LAG_COORD_DEMAND, LAG_COORD_LATCHED, R74, R74_FUEL,
    R74_STATOR, R74_TRIPLE, R74_TWO,
};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{FuelTransientHooks, PointExtra};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::reference_split::R69_TRIPLE;
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::{StatorLimiter, TripleHooks};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::TwoSpoolTransientCore;

// ============================================================================== the grid
//
// `tests/rung74.rs`'s constants, which are `tests/test_rung74.py`'s. This file adds no physical
// constant; the one grid it DOES add is the matrix's, declared with its reason below.

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
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
const B: f64 = 0.10;
const V_MAX: f64 = 0.20;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TT4_MAX: f64 = 1200.0;

/// The three `phi` arms, `rung74.rs`'s names and values. **0.70 is where the shipped guard
/// refuses**, and § 3's `demand_law` seat is run over it deliberately.
const PHI_ARREST: f64 = 0.80;
const PHI_BOTH: f64 = 0.76;
const PHI_GOV: f64 = 0.70;
/// `forcing_openloop(..., ds=0.005)` and its four siblings — **the SHIPPED reader grid.** § 4 is a
/// VALUE question and runs on it; only the matrix coarsens.
const RDR_DS: f64 = 0.005;

/// `taus = (0.05,)*4`, `inc = False`, `r = 0.5`, `s_settle = 1.2`, `v_max = 0.20` — read off
/// `engine.py`'s `def` lines, as `rung74.rs`'s reader-default table is.
const CLOCK: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
const RDR_INC: bool = false;

/// `"sched"` — the first declared reference, and the one every rung-74 reader pins.
const REF_SCHED: &str = REF_LAWS_DECLARED[0];

/// **§ 3's DECLARED COARSE GRID.** The matrix runs ten injections x seven seats and only ever
/// asks *did this seat's reading move*, never *by how much*; the value questions are § 4's and
/// § 6's, at the shipped grids. Coarsening it is what makes the matrix affordable whole, and AD
/// step 6's rule is that running it whole is the point.
///
/// **THE TWO FLOORS ARE A CHOICE AND SO IS THE SECOND ONE.** `demand_law` ships three
/// (`0.80, 0.76, 0.70`); the matrix passes `[PHI_ARREST, PHI_GOV]` — the arm every plant marches
/// and the arm the shipped guard REFUSES — so the baseline fingerprint for that seat legitimately
/// contains a `Failed(…)` string and a row that stops containing one is a DIFF, not a break.
const MX_DS: f64 = 0.02;
const MX_EVERY: usize = 8;
const MX_FLOORS: [f64; 2] = [PHI_ARREST, PHI_GOV];

/// `flat_schedule_identity(..., Tt4_flat=1150.0, s_end=1.2, nu_offset=0.94)`.
const FSI_TT4_FLAT: f64 = 1150.0;
const FSI_S_END: f64 = 1.2;
const FSI_NU_OFFSET: f64 = 0.94;

// ============================================================================== the fixtures

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
    build_two_spool_turbojet(cpg(), PI_LPC, PI_HPC, TT4, 50_000.0, REAL)
}

fn sm_of(phi: f64) -> f64 { phi / FLOOR - 1.0 }

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

/// `rung74.rs`'s `suite_arm(sm, inc = false)`, spelled from this file's constants.
fn suite_arm(sm: f64) -> LeverArm {
    LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, sm, Some(TAU))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S))),
        ..Default::default()
    }
}

/// The arming every seat runs on — `rung74.rs`'s reader arm at the arrest floor.
fn arm() -> LeverArm { suite_arm(sm_of(PHI_ARREST)) }

/// Rung 74's own machine, built by the SHIPPED cascade. § 6 uses it deliberately.
fn demand_shipped(a: &LeverArm) -> ScheduledStatorCore {
    full_of(build_demand_coordinate_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, a))
}

/// Rung 73 — the parent, and the control the `AtLever` row lands on.
fn applied(a: &LeverArm) -> ScheduledStatorCore {
    full_of(build_applied_reference_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, a))
}

/// **THIS FILE SILENCES A PANIC PER THREAD, NOT PER PROCESS**, for AE's recorded reason: an
/// empty hook installed process-wide made two genuinely failing gates report `FAILED` with no
/// message at all. `Once` makes the install race-free; a thread inside [`caught`] or [`run_seat`]
/// is quiet and every other thread keeps the default hook.
fn quiet_hook() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if !QUIET.with(|q| q.get()) {
                prev(info);
            }
        }));
    });
}

thread_local! {
    static QUIET: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Run `f`, returning its panic message if it had one — `pytest.raises(…, match=…)` with the
/// message RETURNED, so a caller can put a second needle on it.
fn caught<F: FnOnce()>(f: F) -> Option<String> {
    quiet_hook();
    QUIET.with(|q| q.set(true));
    let out = catch_unwind(AssertUnwindSafe(f));
    QUIET.with(|q| q.set(false));
    match out {
        Ok(()) => None,
        Err(e) => Some(e.downcast_ref::<String>().cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default()),
    }
}

// =============================================================================================
// THE INJECTIONS — one `at_lever` per cell, so a rebuild installs the injected tables
// =============================================================================================

/// Build a rung-74 machine on an ARBITRARY table triple.
///
/// **NOT through [`build_demand_coordinate_cascade`]**, which hardcodes `&R74`, `&R74_FUEL` and
/// `&R74_TRIPLE` — the point is to install tables it would never install. The class attribute the
/// cascade applies (`_ref_law = "applied"`) is re-applied here, because a machine that kept the
/// core constructor's `"sched"` would march rung 72's reference while reporting rung 74; the
/// reader arm's own two knobs are then written over it, exactly as `rung74.rs`'s `demand_rig`
/// does and by the same PLAIN ASSIGNMENT.
fn with_tables(
    core: &ScheduledStatorCore, a: &LeverArm, lever: &'static LeverHooks,
    fuel: &'static FuelTransientHooks, triple: &'static TripleHooks,
) -> ScheduledStatorCore {
    let c = full_of(ScheduledStatorTransient::with_ref_tables(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(),
        a.stator, &R74_TWO, &R74_STATOR, fuel, lever,
        LeverArming { bleed: a.bleed, sched: a.bleed_sched, lim: a.bleed_lim },
        triple, a.stator_lim, a.stator_inc));
    c.fuel.inner.ref_law.set(REF_LAW_APPLIED);
    c
}

/// One injection: a [`LeverHooks`] whose `at_lever` rebuilds with **that injection's own tables**.
///
/// The sibling constructor is what launders a table injection (AC step 7), so an injection that
/// did not re-aim it would be undone the first time a reader rebuilt — and the gate would be
/// green and vacuous, which is this phase's most-repeated defect.
///
/// Both knobs are copied from the SOURCE core for [`r74_at_lever`]'s reason: a sibling built while
/// the receiver sits under a set coordinate must carry it and not the class default. That copy is
/// this file's declared blind spot (see the header).
macro_rules! injection {
    ($lever:ident, $rebuild:ident, $fuel:expr, $triple:expr) => {
        static $lever: LeverHooks = LeverHooks { at_lever: $rebuild, ..R74 };
        fn $rebuild(core: &ScheduledStatorCore, a: &LeverArm) -> ScheduledStatorCore {
            let m = with_tables(core, a, &$lever, $fuel, $triple);
            m.fuel.inner.ref_law.set(core.fuel.inner.ref_law.get());
            m.fuel.inner.lag_coord.set(core.fuel.inner.lag_coord.get());
            m
        }
    };
}

// **RUNG 79's SHAPE, STOOD IN FOR.** Rung 79's `_with_coord` writes `_phi_ref` — a field the
// port does not have yet, because rung 79 is slice AI's. So the stand-in writes a real second
// field of its own, which is what makes it a *setter pointed elsewhere* rather than a no-op:
// the displaced value round-trips, so `CoordScope`'s restore is symmetric on it and the only
// difference from the shipped cell is WHICH carrier moves.
thread_local! {
    static ELSEWHERE: std::cell::Cell<&'static str> = const { std::cell::Cell::new("phi") };
}

fn elsewhere_with_coord(_: &TwoSpoolTransientCore, coord: &'static str) -> &'static str {
    ELSEWHERE.with(|c| c.replace(coord))
}

/// The four ADDED cells, each replaced by the pointer **rung 73's own table holds** — which is the
/// panicking `no_triple_*` slot, because these four names do not exist below rung 74 in Python at
/// all. A refusal is a weaker rival than a body, and it is the ONLY rival there is: the row it
/// produces is a REACHABILITY census and is read as one.
static T_CAP_FUEL: TripleHooks = TripleHooks { cap_fuel: R73_TRIPLE.cap_fuel, ..R74_TRIPLE };
static T_SENSED_CAP: TripleHooks = TripleHooks { sensed_cap: R73_TRIPLE.sensed_cap, ..R74_TRIPLE };
static T_WINDUP_TAU: TripleHooks = TripleHooks { windup_tau: R73_TRIPLE.windup_tau, ..R74_TRIPLE };
static T_WITH_COORD: TripleHooks = TripleHooks { with_coord: R73_TRIPLE.with_coord, ..R74_TRIPLE };
/// **THE SEPARATE-FIELD INJECTION** — a setter with rung 74's arity that writes SOMEWHERE ELSE.
static T_WITH_COORD_79: TripleHooks =
    TripleHooks { with_coord: elsewhere_with_coord, ..R74_TRIPLE };
/// The two SWAPPED triple cells, each replaced by the parent pointer they were re-aimed from.
static T_RK4: TripleHooks =
    TripleHooks { rk4_floor_shared: R73_TRIPLE.rk4_floor_shared, ..R74_TRIPLE };
static T_SHARED_RIG: TripleHooks =
    TripleHooks { shared_rig: R73_TRIPLE.shared_rig, ..R74_TRIPLE };
/// **THE CONTROL FOR THE `with_ref` HALF OF THE 2 + 2 PARTITION.** Rung 69's `_with_ref` writes
/// `_ref`, rung 73's writes `_ref_law`; this is slice AE's `T_WITH_REF` pointed at a rung-74
/// machine, and its row is what makes *the `with_ref` pins are inert* a measurement.
static T_WITH_REF_69: TripleHooks = TripleHooks { with_ref: R69_TRIPLE.with_ref, ..R74_TRIPLE };
/// Rung 73's `integrate_fuel` — the port that drops all five of this rung's split refusals and
/// never enters the demand march.
static F_INTEGRATE: FuelTransientHooks =
    FuelTransientHooks { integrate_fuel: R73_FUEL.integrate_fuel, ..R74_FUEL };

injection!(L_NONE, none_at_lever, &R74_FUEL, &R74_TRIPLE);
injection!(L_CAP_FUEL, cap_fuel_at_lever, &R74_FUEL, &T_CAP_FUEL);
injection!(L_SENSED_CAP, sensed_cap_at_lever, &R74_FUEL, &T_SENSED_CAP);
injection!(L_WINDUP_TAU, windup_tau_at_lever, &R74_FUEL, &T_WINDUP_TAU);
injection!(L_WITH_COORD, with_coord_at_lever, &R74_FUEL, &T_WITH_COORD);
injection!(L_WITH_COORD_79, with_coord_79_at_lever, &R74_FUEL, &T_WITH_COORD_79);
injection!(L_RK4, rk4_at_lever, &R74_FUEL, &T_RK4);
injection!(L_SHARED_RIG, shared_rig_at_lever, &R74_FUEL, &T_SHARED_RIG);
injection!(L_WITH_REF_69, with_ref_69_at_lever, &R74_FUEL, &T_WITH_REF_69);
injection!(L_INTEGRATE, integrate_at_lever, &F_INTEGRATE, &R74_TRIPLE);

/// The `at_lever` cell itself — rung 73's sibling constructor, which builds a RUNG-73 machine.
/// It is the only row with no `injection!`: re-aiming `at_lever` at the parent is precisely *stop
/// carrying this rung's tables*, so a rebuild that carried them would be the opposite of the
/// injection.
static L_AT_LEVER: LeverHooks = LeverHooks { at_lever: R73.at_lever, ..R74 };

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Inj {
    None, AtLever, IntegrateFuel, Rk4FloorShared, SharedRig,
    CapFuel, SensedCap, WindupTau, WithCoordParent, WithCoordElsewhere, WithRefR69,
}

/// The ten injections plus the baseline, in the order § 3 prints them.
const INJS: [Inj; 11] = [
    Inj::None, Inj::AtLever, Inj::IntegrateFuel, Inj::Rk4FloorShared, Inj::SharedRig,
    Inj::CapFuel, Inj::SensedCap, Inj::WindupTau, Inj::WithCoordParent, Inj::WithCoordElsewhere,
    Inj::WithRefR69,
];

fn tables_of(inj: Inj) -> (&'static LeverHooks, &'static FuelTransientHooks, &'static TripleHooks) {
    match inj {
        Inj::None => (&L_NONE, &R74_FUEL, &R74_TRIPLE),
        Inj::AtLever => (&L_AT_LEVER, &R74_FUEL, &R74_TRIPLE),
        Inj::IntegrateFuel => (&L_INTEGRATE, &F_INTEGRATE, &R74_TRIPLE),
        Inj::Rk4FloorShared => (&L_RK4, &R74_FUEL, &T_RK4),
        Inj::SharedRig => (&L_SHARED_RIG, &R74_FUEL, &T_SHARED_RIG),
        Inj::CapFuel => (&L_CAP_FUEL, &R74_FUEL, &T_CAP_FUEL),
        Inj::SensedCap => (&L_SENSED_CAP, &R74_FUEL, &T_SENSED_CAP),
        Inj::WindupTau => (&L_WINDUP_TAU, &R74_FUEL, &T_WINDUP_TAU),
        Inj::WithCoordParent => (&L_WITH_COORD, &R74_FUEL, &T_WITH_COORD),
        Inj::WithCoordElsewhere => (&L_WITH_COORD_79, &R74_FUEL, &T_WITH_COORD_79),
        Inj::WithRefR69 => (&L_WITH_REF_69, &R74_FUEL, &T_WITH_REF_69),
    }
}

/// Which slot each injection names, for [`assert_installed`]. `AtLever` and `IntegrateFuel` name
/// none, because neither is a triple cell.
fn slot_of(inj: Inj) -> &'static [&'static str] {
    match inj {
        Inj::None | Inj::AtLever | Inj::IntegrateFuel => &[],
        Inj::Rk4FloorShared => &["rk4_floor_shared"],
        Inj::SharedRig => &["shared_rig"],
        Inj::CapFuel => &["cap_fuel"],
        Inj::SensedCap => &["sensed_cap"],
        Inj::WindupTau => &["windup_tau"],
        Inj::WithCoordParent | Inj::WithCoordElsewhere => &["with_coord"],
        Inj::WithRefR69 => &["with_ref"],
    }
}

/// A rung-74 machine carrying one injection, at the reader arm's own two knobs.
///
/// **[`Inj::None`] is a real machine and not a shortcut** — it goes through the same
/// `with_ref_tables` path with the shipped tables, so every baseline reading below differs from
/// an injected one in the POINTER and in nothing else.
fn build(inj: Inj, a: &LeverArm) -> ScheduledStatorCore {
    let (lever, fuel, triple) = tables_of(inj);
    let m = with_tables(&demand_shipped(a), a, lever, fuel, triple);
    // `rung74.rs`'s `demand_rig(sm, inc, "demand", "sched")`, by PLAIN ASSIGNMENT — which is what
    // Python's own fixture does and, as of this commit, what the two pins in `demand_coordinate.rs`
    // do too.
    m.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);
    m.fuel.inner.ref_law.set(REF_SCHED);
    m
}

/// Which of [`TripleHooks`]' fields differ between two tables, **by exhaustive destructuring**.
///
/// A `fn_addr_eq` on the one slot an injection names proves the slot moved; it does not prove
/// nothing else did. This lists all **eighteen** — and the destructuring goes `E0027` when a
/// nineteenth field lands, which is the tripwire this slice itself fired when the table went
/// 14 → 18.
fn triple_diff(a: &'static TripleHooks, b: &'static TripleHooks) -> Vec<&'static str> {
    let TripleHooks {
        stator_leg, lagged_stator, clamp_v, check_v0, rk4_floor, solve_v, manifold_v, triple_laws,
        triple_rig, with_ref, reference, rk4_floor_shared, shared_rig, quad_gains_at,
        cap_fuel, sensed_cap, windup_tau, with_coord,
    } = *a;
    let mut out = Vec::new();
    let mut chk = |name, same: bool| if !same { out.push(name) };
    chk("stator_leg", fn_addr_eq(stator_leg, b.stator_leg));
    chk("lagged_stator", fn_addr_eq(lagged_stator, b.lagged_stator));
    chk("clamp_v", fn_addr_eq(clamp_v, b.clamp_v));
    chk("check_v0", fn_addr_eq(check_v0, b.check_v0));
    chk("rk4_floor", fn_addr_eq(rk4_floor, b.rk4_floor));
    chk("solve_v", fn_addr_eq(solve_v, b.solve_v));
    chk("manifold_v", fn_addr_eq(manifold_v, b.manifold_v));
    chk("triple_laws", fn_addr_eq(triple_laws, b.triple_laws));
    chk("triple_rig", fn_addr_eq(triple_rig, b.triple_rig));
    chk("with_ref", fn_addr_eq(with_ref, b.with_ref));
    chk("reference", fn_addr_eq(reference, b.reference));
    chk("rk4_floor_shared", fn_addr_eq(rk4_floor_shared, b.rk4_floor_shared));
    chk("shared_rig", fn_addr_eq(shared_rig, b.shared_rig));
    chk("quad_gains_at", fn_addr_eq(quad_gains_at, b.quad_gains_at));
    chk("cap_fuel", fn_addr_eq(cap_fuel, b.cap_fuel));
    chk("sensed_cap", fn_addr_eq(sensed_cap, b.sensed_cap));
    chk("windup_tau", fn_addr_eq(windup_tau, b.windup_tau));
    chk("with_coord", fn_addr_eq(with_coord, b.with_coord));
    out
}

/// The install proof every injected reading is preceded by: rebuild a sibling through the injected
/// `at_lever`, and assert the tables it hands back are the injected ones and differ from the
/// shipped ones in EXACTLY the named slot.
///
/// **`ptr::eq` IS NOT A TABLE-IDENTITY TEST HERE** — AE step 5's finding. Every table in this
/// family is a `pub const`, so `&R73_TRIPLE` is a fresh rvalue promotion at each use site and two
/// of them need not share an address; worse, on most rows the machine holds the pointer *this
/// fixture handed the builder*, so such an assertion compares the instrument against itself.
/// [`triple_diff`] compares FUNCTION addresses, which are stable across promotions.
fn assert_installed(m: &ScheduledStatorCore, inj: Inj) {
    let (_, fuel, triple) = tables_of(inj);
    let sib = m.at_lever(&arm());
    assert_eq!(triple_diff(sib.triple_hooks(), &R74_TRIPLE), slot_of(inj),
               "{inj:?}: the REBUILT sibling must carry the injection and nothing else — a lone \
                table injection is laundered by `at_lever` before any value is read");
    assert_eq!(triple_diff(sib.triple_hooks(), triple), Vec::<&str>::new(),
               "{inj:?}: and it must agree with the intended table in all eighteen slots");
    assert_eq!(fn_addr_eq(sib.fuel.hooks.integrate_fuel, R74_FUEL.integrate_fuel),
               inj != Inj::IntegrateFuel,
               "{inj:?}: the fuel table travels with the rebuild too");
    assert!(fn_addr_eq(sib.fuel.hooks.integrate_fuel, fuel.integrate_fuel));
}

// =============================================================================================
// 1 — THE TEN POINTERS ARE TEN REAL SWAPS
// =============================================================================================

/// **EVERY INJECTION BELOW MUST BE A DIFFERENT BODY FROM THE ONE IT REPLACES**, or its row in § 3
/// is a table compared with itself — [[rust-port-slice-aa-step1]]'s rule, and the equality
/// control rides beside each inequality so a broken instrument fails visibly instead of passing.
#[test]
fn every_injection_is_a_real_swap() {
    // the four ADDs: rung 73's slot is the panicking refusal, rung 74's is a body
    assert!(!fn_addr_eq(R74_TRIPLE.cap_fuel, R73_TRIPLE.cap_fuel));
    assert!(!fn_addr_eq(R74_TRIPLE.sensed_cap, R73_TRIPLE.sensed_cap));
    assert!(!fn_addr_eq(R74_TRIPLE.windup_tau, R73_TRIPLE.windup_tau));
    assert!(!fn_addr_eq(R74_TRIPLE.with_coord, R73_TRIPLE.with_coord));
    // the two triple SWAPs
    assert!(!fn_addr_eq(R74_TRIPLE.rk4_floor_shared, R73_TRIPLE.rk4_floor_shared));
    assert!(!fn_addr_eq(R74_TRIPLE.shared_rig, R73_TRIPLE.shared_rig));
    // the two non-triple SWAPs
    assert!(!fn_addr_eq(R74.at_lever, R73.at_lever));
    assert!(!fn_addr_eq(R74_FUEL.integrate_fuel, R73_FUEL.integrate_fuel));
    // the separate-field stand-in, and rung 69's setter
    assert!(!fn_addr_eq(R74_TRIPLE.with_coord, T_WITH_COORD_79.with_coord));
    assert!(!fn_addr_eq(R74_TRIPLE.with_ref, R69_TRIPLE.with_ref));

    // THE EQUALITY CONTROL — the twelve inherited slots really are inherited, so the twelve
    // `same` readings in `triple_diff` are a measurement and not a broken comparison.
    assert_eq!(triple_diff(&R74_TRIPLE, &R73_TRIPLE),
               vec!["rk4_floor_shared", "shared_rig", "cap_fuel", "sensed_cap", "windup_tau",
                    "with_coord"],
               "rung 74 re-aims two of rung 73's cells and adds four — the census, read off the \
                tables rather than off the plan's own column");

    // and every injected table differs from the shipped one in exactly its named slot
    for inj in INJS {
        let (_, _, t) = tables_of(inj);
        assert_eq!(triple_diff(t, &R74_TRIPLE), slot_of(inj), "{inj:?}");
    }
}

// =============================================================================================
// 2 — THE SEATS
// =============================================================================================

fn fingerprint<T: std::fmt::Debug>(x: &T) -> String { format!("{x:?}") }

/// What a seat did. **`refused` is separated from `BROKE-OTHER` by the MESSAGE**: a seat that
/// already refuses on the baseline and refuses identically under an injection has told us
/// nothing, and one that refuses differently has.
#[derive(Clone, PartialEq, Eq, Debug)]
enum Seen { Read(String), Broke(String) }

const SEATS: [&str; 7] = ["coord_march", "demand_law", "demand_gains", "latch_discriminator",
                          "windup_law", "flat_schedule_identity", "forcing_openloop"];

fn run_seat(name: &str, m: &ScheduledStatorCore, clock: (f64, f64, f64, f64)) -> Seen {
    quiet_hook();
    QUIET.with(|q| q.set(true));
    let f = flight();
    let sm = sm_of(PHI_ARREST);
    let out = catch_unwind(AssertUnwindSafe(|| match name {
        "coord_march" => fingerprint(&coord_march(
            m, &f, LO, HI, TT4_MAX, sm, clock, R, SETTLE, MX_DS, V_MAX, RDR_INC,
            LAG_COORD_DEMAND, REF_SCHED, None).3
            .iter().map(|p| (p.s.to_bits(), p.tt4.to_bits(), p.mf.to_bits())).collect::<Vec<_>>()),
        "demand_law" => fingerprint(&demand_law(
            m, &f, LO, HI, TT4_MAX, sm, clock, &MX_FLOORS, R, SETTLE, MX_DS, V_MAX)),
        "demand_gains" => fingerprint(&demand_gains(
            m, &f, LO, HI, TT4_MAX, PHI_ARREST, clock, RDR_INC, R, SETTLE, MX_DS, V_MAX,
            MX_EVERY)),
        "latch_discriminator" => fingerprint(&latch_discriminator(
            m, &f, LO, HI, TT4_MAX, PHI_ARREST, clock, RDR_INC, R, SETTLE, MX_DS, V_MAX)),
        "windup_law" => fingerprint(&windup_law(
            m, &f, LO, HI, TT4_MAX, PHI_ARREST, clock, RDR_INC, R, SETTLE, MX_DS, V_MAX)),
        "flat_schedule_identity" => fingerprint(&flat_schedule_identity(
            m, &f, FSI_TT4_FLAT, PHI_ARREST, clock, RDR_INC, FSI_S_END, MX_DS, V_MAX, TT4_MAX,
            FSI_NU_OFFSET)),
        "forcing_openloop" => fingerprint(&forcing_openloop(
            m, &f, LO, HI, TT4_MAX, PHI_ARREST, clock, RDR_INC, R, SETTLE, MX_DS, V_MAX)),
        _ => unreachable!("SEATS is the only source of these names"),
    }));
    QUIET.with(|q| q.set(false));
    match out {
        Ok(s) => Seen::Read(s),
        Err(e) => Seen::Broke(e.downcast_ref::<String>().cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default()),
    }
}

/// One cell of the matrix, as printed.
fn verdict(base: &Seen, got: &Seen) -> &'static str {
    match (base, got) {
        (Seen::Read(a), Seen::Read(b)) if a == b => "same",
        (Seen::Read(_), Seen::Read(_)) => "DIFF",
        (Seen::Read(_), Seen::Broke(_)) => "BROKE",
        (Seen::Broke(_), Seen::Read(_)) => "RETURNS",
        (Seen::Broke(a), Seen::Broke(b)) if a == b => "refused",
        (Seen::Broke(_), Seen::Broke(_)) => "BROKE-OTHER",
    }
}

// =============================================================================================
// 3 — THE SEAT MATRIX, RUN WHOLE
// =============================================================================================

/// **TEN POINTERS x SEVEN SEATS.**
///
/// AD step 6's rule: a parent-pointer injection that produces no observable at a seat is either
/// laundered or on a path that never reaches the cell, and *nothing moved* is the same reading
/// for both. What separates them is the OTHER seats in the same ROW.
///
/// The rows are transcribed from the run, never predicted into it.
#[test]
fn the_seat_matrix() {
    let base: Vec<Seen> = SEATS.iter()
        .map(|s| run_seat(s, &build(Inj::None, &arm()), CLOCK)).collect();
    for (i, b) in base.iter().enumerate() {
        assert!(matches!(b, Seen::Read(_)),
                "the baseline seat {} must READ, or every verdict below is against a broken \
                 fixture: {b:?}", SEATS[i]);
    }
    // **THE 0.70 ARM DOES NOT REFUSE HERE, AND THAT WAS PREDICTED WRONG.** § 5.30 (i)'s refusal
    // table is `demand_gains`'s, not `demand_law`'s: the reader that raises is the one that walks
    // the unfloored cap, while `demand_law` reaches the same floor through `try_coord_march` and,
    // on this grid, gets a clean march. The first writing of this gate asserted a `Failed(...)`
    // into the baseline and went red. It is REPORTED instead — a recorded failure appearing or
    // disappearing is a DIFF like any other, which is all the matrix needs.
    match &base[1] {
        Seen::Read(s) => println!("demand_law baseline carries a recorded failure: {}",
                                  s.contains("Failed")),
        Seen::Broke(_) => unreachable!("checked above"),
    }

    let mut tally: Vec<(Inj, Vec<&'static str>)> = Vec::new();
    for inj in INJS.iter().skip(1) {
        let m = build(*inj, &arm());
        // THE INSTALL PROOF, BEFORE THE ROW IS READ. An all-silent row's ONLY evidence that the
        // injection took at all is this one.
        match *inj {
            // `at_lever` re-aimed at the parent is *stop carrying this rung's tables*, so its
            // sibling is a RUNG-73 machine by construction and there is no rung-74 slot to check.
            Inj::AtLever => {
                assert!(fn_addr_eq(m.fuel.inner.lever_hooks.at_lever, R73.at_lever));
                let sib = m.at_lever(&arm());
                assert_eq!(triple_diff(sib.triple_hooks(), &R73_TRIPLE), Vec::<&str>::new(),
                           "rung 73's sibling constructor installs rung 73's table, which is the \
                            whole of this injection");
                // and it is the SHIPPED rung-73 table, not a rung-73-shaped one this file
                // assembled — compared against a machine the rung-73 cascade built.
                assert_eq!(triple_diff(sib.triple_hooks(), applied(&arm()).triple_hooks()),
                           Vec::<&str>::new());
            }
            _ => assert_installed(&m, *inj),
        }
        let row: Vec<&'static str> = SEATS.iter().zip(base.iter())
            .map(|(s, b)| verdict(b, &run_seat(s, &m, CLOCK))).collect();
        println!("{inj:?}: {row:?}");
        tally.push((*inj, row));
    }
    let row_of = |i: Inj| tally.iter().find(|(k, _)| *k == i).expect("every injection ran").1
        .clone();
    let seat = |name: &str| SEATS.iter().position(|s| *s == name)
        .unwrap_or_else(|| panic!("no seat {name:?}"));

    // ------------------------------------------------------------------ THE MEASURED PARTITION
    //
    // Which injections are observable at a seat AT ALL. **Not a universal that happens to hold** —
    // AE's file asserted *every injection breaks somewhere* with two counterexamples already in
    // its own table. Five of the ten are silent at all seven seats, and each has a gate or a
    // sentence of its own below saying WHY, because the two pointers with no behavioural control
    // are exactly the two that would most benefit from one.
    let live: Vec<Inj> = tally.iter()
        .filter(|(_, row)| row.iter().any(|v| *v != "same" && *v != "refused"))
        .map(|(i, _)| *i).collect();
    assert_eq!(live, vec![Inj::AtLever, Inj::IntegrateFuel, Inj::CapFuel, Inj::WindupTau,
                          Inj::WithCoordParent],
               "five of the ten re-aimed pointers are observable at some seat");

    // --------------------------------------------- THE COORDINATE CELL HAS EXACTLY ONE READER
    //
    // **AND THIS IS THE ROW THE FOUR-SITE FIX IS MEASURED BY.** Before it, the parent's panicking
    // slot was reached at SIX of the seven seats, because `coord_march` and `demand_gains` pinned
    // the field through the cell. After it, the only dispatch left is [`CoordScope`] inside
    // `demand_gains` — which is Python's one call site, `engine.py:18276` — and the row says so.
    let p = row_of(Inj::WithCoordParent);
    assert_eq!(p[seat("demand_gains")], "BROKE");
    assert!(p.iter().enumerate().all(|(i, v)| i == seat("demand_gains") || *v == "same"),
            "the coordinate SETTER is dispatched at exactly one seat, which is the one Python \
             dispatches at: {p:?}");

    // **P5, AS A ROW.** The same slot pointed at a working setter that writes ANOTHER field is
    // silent everywhere — § 5's gate is the value form of this line, and § 4 is why the silence
    // is a property of the plant rather than of a blind reader.
    assert!(row_of(Inj::WithCoordElsewhere).iter().all(|v| *v == "same"),
            "P5: {:?}", row_of(Inj::WithCoordElsewhere));

    // **THE `with_ref` HALF OF THE 2 + 2 PARTITION, MEASURED.** Rung 69's setter writes `_ref`
    // instead of `_ref_law`, and with the pins direct it reaches nothing: `_with_ref` has exactly
    // two definers and every machine at rung >= 74 carries rung 73's. Before the fix this row was
    // live at `windup_law`, whose four cells are the only place the pinned reference VARIES.
    assert!(row_of(Inj::WithRefR69).iter().all(|v| *v == "same"),
            "{:?}", row_of(Inj::WithRefR69));

    // ------------------------------------------------ THE REACHABILITY CENSUS, PER ADDED CELL
    //
    // Recovered by RUNNING the matrix, never by grepping for the slot.
    let cf = row_of(Inj::CapFuel);
    assert_eq!(cf[seat("forcing_openloop")], "same",
               "`forcing_openloop` consults `cap_gov`, not the fuel leg's cap");
    assert!(cf.iter().enumerate().all(|(i, v)| i == seat("forcing_openloop") || *v != "same"),
            "and every other reader reaches it: {cf:?}");

    let wt = row_of(Inj::WindupTau);
    for s in ["demand_gains", "forcing_openloop"] {
        assert_eq!(wt[seat(s)], "same",
                   "{s} marches the CLIP arm, which does not enter this rung's own integrator — \
                    so the windup hook at its top is never called");
    }
    assert!(wt.iter().enumerate()
              .all(|(i, v)| i == seat("demand_gains") || i == seat("forcing_openloop")
                            || *v != "same"), "{wt:?}");

    // **`sensed_cap` IS UNREACHABLE FROM EVERY RUNG-74 READER**, and that is a measurement rather
    // than a suspicion: `r74_cap_fuel` reaches it only inside `if let Some(accel)`, and not one of
    // the seven seats arms an `AccelSchedule`. So the cell's dispatch cannot be gated by value at
    // this rung at all — `slice_af_cells.rs`'s `cap_fuel_reaches_sensed_cap_through_the_table`
    // owns it structurally, by injecting a table and calling the cap directly. Named here so a
    // later reader does not mistake this row's silence for a cell that does not matter: rung 76
    // replaces this body with rung 48's schedule, and the accel arm is where it lands.
    assert!(row_of(Inj::SensedCap).iter().all(|v| *v == "same"),
            "{:?}", row_of(Inj::SensedCap));

    // and the two SWAPPED triple cells, silent for two DIFFERENT reasons — § 6 owns the first,
    // and the second is `slice_af_cells.rs`'s driven no-op (the parent's rig reaches its sibling
    // through `at_lever`, which at rung 74 has already carried both knobs).
    assert!(row_of(Inj::Rk4FloorShared).iter().all(|v| *v == "same"));
    assert_eq!(row_of(Inj::SharedRig), row_of(Inj::Rk4FloorShared),
               "two different reasons, one reading");

    for (inj, row) in &tally {
        println!("ROW {inj:?} = {row:?}");
    }
}

/// **THE MATRIX's INSTRUMENT, PROVED ABLE TO SEE BEFORE IT IS READ.**
///
/// Every `same` in § 3 is a string equality between two `Debug` fingerprints. A fingerprint that
/// was constant — a reader that returned an empty struct, a seat wired to the wrong machine —
/// would make the whole matrix read `same` and look like a clean result. So the plant is
/// perturbed (the clock moved, nothing else) and **every seat is required to move**.
#[test]
fn the_matrix_instrument_moves_when_the_plant_does() {
    let m = build(Inj::None, &arm());
    // **THE PERTURBATION HAS TO STAY INSIDE THIS RUNG'S OWN FLOOR**, and the first one written
    // here did not: `(0.20, 0.01, 0.50, 0.05)` puts `ds*sum(1/tau)` at 2.540 against the shipped
    // bar of 2.0, so `coord_march` REFUSED and the control reported a seat that "changed kind"
    // when what had actually happened is that the fixture drove the plant out of the region the
    // rung declares. `r74_rk4_floor_shared` caught it by name, which is § 6's cell doing its job
    // on this file's own instrument.
    let other = (0.06, 0.05, 0.05, 0.05);
    assert_ne!(CLOCK, other);
    let rate: f64 = [other.0, other.1, other.2, other.3].iter().map(|t| 1.0 / t).sum();
    assert!(MX_DS * rate <= 2.0,
            "the perturbed clock must be admissible at the matrix grid: ds*rate = {}",
            MX_DS * rate);
    for s in SEATS.iter() {
        let a = run_seat(s, &m, CLOCK);
        let b = run_seat(s, &m, other);
        match (&a, &b) {
            (Seen::Read(x), Seen::Read(y)) => {
                assert!(!x.is_empty() && x != y, "seat {s} did not move with the plant");
            }
            _ => panic!("seat {s} refused on one of the two plants: {a:?} / {b:?}"),
        }
    }
}

// =============================================================================================
// 4 — P5: THE COORDINATE'S ONE READER, AND ITS ZERO IS ARITHMETIC
// =============================================================================================

/// **§ 5.30 (i)'s MEASUREMENT, REPRODUCED IN THE CRATE — AND IT IS THREE-SIDED, NOT TWO.**
///
/// `_demand_target` is `min(mf_sched, cap) if coord == "demand-latched" else cap`: a THREE-valued
/// tag read by a TWO-valued test, so `clip` and `demand` are indistinguishable to it BY
/// CONSTRUCTION, and the third value is distinguishable only where `cap > mf_sched`.
///
/// 1. **THE INSTRUMENT CAN SEE.** On a manufactured pair with `cap > mf_sched` the latched
///    coordinate returns something else. Asserted FIRST, so a reader that ignored its tag could
///    never be reported below as a clean zero — § 5.30 (vii) item 2's whole subject.
/// 2. **THE READER IS LIVE ON THIS PLANT.** Over the caps the demand march itself records, the
///    over-schedule region is NOT empty, and where it is non-empty the latch really does differ.
///    § 5.30 (i) measured 139 of 2 732 there; the number here is this file's own coarse grid's
///    and is printed rather than pinned.
/// 3. **AND IT IS THE IDENTITY WHERE THE SCOPE LOOKS**, which is P5 and is § 5's subject — a
///    VALUE reading through the shipped reader, not an absence.
///
/// The three together are § 5.30 (i)'s *two disjoint regions on one plant*, recovered inside the
/// crate: the over-schedule region and the interior-filter region do not meet.
#[test]
fn the_coordinates_one_reader_can_discriminate_and_the_march_reaches_the_region() {
    let m = build(Inj::None, &arm());
    let c = &m.fuel.inner;
    let keep = c.lag_coord.get();

    // 1 — THE POSITIVE CONTROL, before any zero is believed.
    let (cap_hi, ms) = (2.0, 1.0);
    assert!(cap_hi > ms);
    c.lag_coord.set(LAG_COORD_LATCHED);
    let latched = demand_target(c, cap_hi, ms);
    c.lag_coord.set(LAG_COORD_DEMAND);
    let plain = demand_target(c, cap_hi, ms);
    c.lag_coord.set(LAG_COORD_CLIP);
    let clip = demand_target(c, cap_hi, ms);
    assert_eq!(latched.to_bits(), ms.to_bits(), "the latch is `min(mf_sched, cap)`");
    assert_eq!(plain.to_bits(), cap_hi.to_bits(), "and the unlatched coordinates are the cap");
    assert_eq!(clip.to_bits(), plain.to_bits(),
               "`clip` and `demand` are ONE reading to this reader — a three-valued tag read by a \
                two-valued test");
    assert_ne!(latched.to_bits(), plain.to_bits(),
               "the reader CAN discriminate, so a zero below is a property of the plant");

    // 2 — THE MARCH REACHES THE REGION.
    //
    // **ON THE SHIPPED READER GRID AND AT THE FLOOR WHERE THE PLANT ACCELERATES**, and both
    // halves of that are corrections this gate's first writing needed. At `PHI_ARREST` the surge
    // cap sits AT the scheduled fuel from `s = 0` and the leg permits no acceleration at all
    // (`rung74.rs`'s own disclosure), so the over-schedule region is EMPTY there — measured, 0 of
    // 86 — and a gate that had only run that arm would have reported § 5.30 (i)'s 139 as
    // unreproducible. `PHI_BOTH` is the arm all three plants march.
    //
    // **AND BOTH CAPS ARE CENSUSED.** `_demand_target` is called once per LEG, so taking only
    // `cap_fuel` would make the answer depend on which leg happens to hold the actuator.
    //
    // `LAG_COORDS_DECLARED` is `[clip, demand, demand-latched]`; the three indices are resolved
    // by NAME rather than typed, because an index is exactly the kind of thing that goes stale
    // silently.
    let i_clip = idx_of(LAG_COORD_CLIP);
    let i_demand = idx_of(LAG_COORD_DEMAND);
    let i_latch = idx_of(LAG_COORD_LATCHED);
    let traj = coord_march(&m, &flight(), LO, HI, TT4_MAX, sm_of(PHI_BOTH), CLOCK, R, SETTLE,
                           RDR_DS, V_MAX, RDR_INC, LAG_COORD_DEMAND, REF_SCHED, None).3;
    let (mut seen, mut over) = (0usize, 0usize);
    for p in traj.iter() {
        let (cf, cg) = match p.extra {
            PointExtra::Demand { cap_fuel, cap_gov, .. } => (cap_fuel, cap_gov),
            _ => panic!("a demand march must emit rung-74 points"),
        };
        let mf_sched = p.mf_sched;
        for cap in [cf, cg] {
            if !cap.is_finite() { continue; }
            seen += 1;
            let mut bits: Vec<u64> = Vec::new();
            for coord in LAG_COORDS_DECLARED {
                c.lag_coord.set(coord);
                bits.push(demand_target(c, cap, mf_sched).to_bits());
            }
            assert_eq!(bits[i_clip], bits[i_demand],
                       "`clip` and `demand` are indistinguishable BY CONSTRUCTION");
            if cap > mf_sched {
                over += 1;
                assert_ne!(bits[i_latch], bits[i_demand],
                           "cap = {cap} > mf_sched = {mf_sched}, so the latch MUST bite");
            } else {
                assert_eq!(bits[i_latch], bits[i_demand],
                           "and it must not bite where `cap <= mf_sched`: {cap} / {mf_sched}");
            }
        }
    }
    c.lag_coord.set(keep);
    assert!(seen > 100, "the census must be over a real march: seen = {seen}");
    assert!(over > 0,
            "the OVER-SCHEDULE region must be non-empty on this plant, or the claim that the two \
             regions are DISJOINT is vacuous on one side: {over} of {seen}");
    println!("P5 census: {seen} finite caps over the demand march's two legs, {over} over schedule");
}

/// [`LAG_COORDS_DECLARED`]'s index for a coordinate, by NAME.
fn idx_of(coord: &str) -> usize {
    LAG_COORDS_DECLARED.iter().position(|c| *c == coord)
        .unwrap_or_else(|| panic!("{coord:?} is not a declared coordinate"))
}

// =============================================================================================
// 5 — THE SEPARATE-FIELD GATE, DRIVEN THROUGH A REAL READER
// =============================================================================================

/// **§ 5.30 (ii)'s OBLIGATION.** `slice_af_cells.rs` gates the field two-sidedly by calling the
/// cell BY HAND; this drives the same distinction through the shipped setter and the shipped
/// guard, on a machine whose table has been re-aimed.
///
/// The two halves are: the shipped cell moves `lag_coord` and NOTHING else; and a setter with the
/// same arity pointed at a different carrier leaves `lag_coord` where it was — which is what
/// rung 79 will be, and why the slot cannot be a name a later rung re-aims freely.
#[test]
fn the_coordinate_setter_writes_this_rungs_own_field_and_a_re_aimed_one_does_not() {
    let m = build(Inj::None, &arm());
    let c = &m.fuel.inner;
    c.lag_coord.set(LAG_COORD_CLIP);
    let before_ref = c.ref_law.get();

    {
        let g = CoordScope::set(c, LAG_COORD_DEMAND);
        assert_eq!(g.displaced(), LAG_COORD_CLIP);
        assert_eq!(c.lag_coord.get(), LAG_COORD_DEMAND, "the POSITIVE half");
        assert_eq!(c.ref_law.get(), before_ref, "THE NEGATIVE half — rung 73's carrier is untouched");
    }
    assert_eq!(c.lag_coord.get(), LAG_COORD_CLIP, "restore-previous, through the same cell");

    // AND THROUGH THE RE-AIMED TABLE — the scope runs, the guard restores, and this rung's field
    // never moves, because the setter is writing somewhere else.
    let n = build(Inj::WithCoordElsewhere, &arm());
    assert_installed(&n, Inj::WithCoordElsewhere);
    let d = &n.fuel.inner;
    d.lag_coord.set(LAG_COORD_CLIP);
    {
        let g = CoordScope::set(d, LAG_COORD_DEMAND);
        assert_eq!(d.lag_coord.get(), LAG_COORD_CLIP,
                   "a setter pointed at another carrier leaves this rung's field at its default \
                    — which is exactly rung 79's `_phi_ref` seen from rung 74");
        assert_eq!(g.displaced(), "phi", "and it hands back ITS OWN previous value");
        assert_eq!(ELSEWHERE.with(|x| x.get()), LAG_COORD_DEMAND, "the other carrier moved");
    }
    assert_eq!(ELSEWHERE.with(|x| x.get()), "phi", "and the guard restored the other carrier");
}

/// **P5, AS A VALUE READING THROUGH THE SHIPPED READER — NOT AS AN ABSENCE.**
///
/// `demand_gains` is `_with_coord`'s ONE call site (`engine.py:18276`). Point the setter at
/// another carrier and the reader's scope becomes a no-op, so `_demand_target` sees `"clip"`
/// where it would have seen `"demand"` — **and every published number is bit-identical**, because
/// this reader's interior filter admits only points at which `cap <= mf_sched`, where the latch
/// cannot bite. § 5.30 (i) counted that 1 040 of 1 040 and 624 of 624; § 4 is why the count is
/// evidence — it shows the reader moving where the inequality reverses, and the plant reaching
/// that region on another arm.
///
/// **THAT IS THE MEASUREMENT P5 NAMED**, and it is worth more than the matrix's `same`: the
/// matrix compares two `Debug` strings from two machines, while this compares the reader that
/// owns the cell against itself with the cell re-aimed, having already shown (§ 4) that the
/// reader can discriminate and that the plant reaches the region where it does.
///
/// **AND THE CONTROL IS THE OTHER INJECTION.** Pointing the same slot at rung 73's PANICKING slot
/// makes this same reader refuse by name — so the scope is genuinely entered, and the zero above
/// is not *the cell is never reached*.
#[test]
fn the_coordinate_is_unobservable_by_value_in_the_reader_that_sets_it() {
    let base = demand_gains(&build(Inj::None, &arm()), &flight(), LO, HI, TT4_MAX, PHI_ARREST,
                            CLOCK, RDR_INC, R, SETTLE, MX_DS, V_MAX, MX_EVERY);
    assert!(base.n > 0, "the reader must produce interior rows, or this gate compares two empties");

    let n = build(Inj::WithCoordElsewhere, &arm());
    assert_installed(&n, Inj::WithCoordElsewhere);
    let got = demand_gains(&n, &flight(), LO, HI, TT4_MAX, PHI_ARREST, CLOCK, RDR_INC, R, SETTLE,
                           MX_DS, V_MAX, MX_EVERY);
    assert_eq!(fingerprint(&got), fingerprint(&base),
               "P5: re-aiming the coordinate setter moves NOTHING this reader publishes");

    // THE CONTROL — the same slot, pointed at the refusal, on the same grid.
    let p = build(Inj::WithCoordParent, &arm());
    assert_installed(&p, Inj::WithCoordParent);
    let msg = caught(|| {
        let _ = demand_gains(&p, &flight(), LO, HI, TT4_MAX, PHI_ARREST, CLOCK, RDR_INC, R,
                             SETTLE, MX_DS, V_MAX, MX_EVERY);
    }).expect("the scope IS entered — a re-aimed slot that were never read could not refuse");
    assert!(msg.contains("_with_coord"), "and it refuses by the cell's own name: {msg:?}");
}

// =============================================================================================
// 6 — THE TWO SWAPS WITH NO VALUE BREAK, RECORDED RATHER THAN HUNTED
// =============================================================================================

/// **THE FLOOR SWAP IS A MESSAGE AND NOT A VALUE, AND THE CONDITION IS CHARACTER FOR CHARACTER
/// THE PARENT's.**
///
/// [`r74_rk4_floor_shared`]'s own doc comment says it: the condition is `ds * rate <= 2.0` in
/// rungs 72, 73 and 74 character for character, so the MESSAGE is the entire cell. § 3's
/// `Rk4FloorShared` row is silent for exactly that reason and not because the injection failed to
/// take, and this closes the distinction by driving both bodies past the boundary.
///
/// The shipped Python needle `"FOUR actuator states"` reaches nine classes back to rung 43, so it
/// is asserted to be USELESS rather than read; the tokens that discriminate are `rung-74` and
/// `ACTIVE lag`.
#[test]
fn the_floor_swap_is_a_message_and_not_a_value() {
    assert_eq!(0.5f64 * 4.0, 2.0);
    for (ds, rate) in [(0.5, 3.5), (0.5, 4.0)] {
        assert!(caught(|| (R74_TRIPLE.rk4_floor_shared)(ds, rate)).is_none());
        assert!(caught(|| (R73_TRIPLE.rk4_floor_shared)(ds, rate)).is_none());
    }
    let a = caught(|| (R74_TRIPLE.rk4_floor_shared)(0.5, 4.5)).expect("past the boundary");
    let b = caught(|| (R73_TRIPLE.rk4_floor_shared)(0.5, 4.5)).expect("past the boundary");
    assert!(a.contains("rung-74") && a.contains("ACTIVE lag"), "{a:?}");
    assert!(b.contains("rung-73") && !b.contains("rung-74"), "{b:?}");
    assert_ne!(a, b);
    assert!(a.contains("FOUR actuator states") && b.contains("FOUR actuator states"),
            "the shipped Python needle is in BOTH messages, which is why it discriminates \
             nothing and why the gate above reads the rung tag");
}

/// **THE FOUR ADDED CELLS' PARENT SLOTS ARE REFUSALS, AND THE REFUSAL NAMES ITSELF.**
///
/// A rung-40..73 machine has none of these four names, so the parent slot is a panic rather than a
/// rival body. § 3's four rows are therefore a REACHABILITY census — which readers dispatch which
/// cell — and this gate is what makes each `BROKE` attributable: the message carries the cell's
/// own Python name, so a row that broke for an unrelated reason is distinguishable from one that
/// reached the slot.
#[test]
fn the_four_added_slots_refuse_by_name_below_this_rung() {
    let m = build(Inj::None, &arm());
    let c = &m.fuel.inner;
    let msgs = [
        caught(|| { let _ = (R73_TRIPLE.windup_tau)(c); }).expect("_windup_tau refuses"),
        caught(|| { let _ = (R73_TRIPLE.with_coord)(c, LAG_COORD_DEMAND); })
            .expect("_with_coord refuses"),
    ];
    for (msg, name) in msgs.iter().zip(["_windup_tau", "_with_coord"]) {
        assert!(msg.contains("RUNG 74's") || msg.contains("RUNG 74"), "{msg:?}");
        assert!(msg.contains(name), "the refusal names the cell it stands in for: {msg:?}");
    }
}
