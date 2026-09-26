//! RUNG 79 — **THE STATE COORDINATE.** `StateCoordinateTransient`, slice AI.
//!
//! Rung 78 re-wrote a leg's LAW and found its root preserved and its UNIQUENESS destroyed. This
//! rung re-writes the phi leg's STATE COORDINATE — rung 60's incidence `M_i` in place of rung 49's
//! `phi` — and asks the same question of the other side:
//!
//! ```text
//! Gs(w) = phi_lim − phi(w)                               [rung 49, SHIPPED]
//! Gi(w) = m_lim − M_i(w) = 1/phi(w) − 1/phi_lim           [rung 60's currency]
//!       = Gs(w)·h(w),     h(w) := 1/(phi(w)·phi_lim) > 0  STRICTLY
//! ```
//!
//! `T_c` and `v` cancel identically, so what is left is a multiplier with no zero: **a coordinate
//! change is a gauge that cannot go singular.** The root SET is preserved pointwise, the slope at
//! the root scales by exactly `1/phi_lim²`, and `dw*/dq` is invariant. What is at risk is only
//! that the SAME root comes back as DIFFERENT FLOATS out of a bracketed solve whose iterates
//! depend on the slope — and those floats feed a `min`.
//!
//! # WHAT STEP 1 OF THIS SLICE ADDS AT THIS RUNG — **FOUR RE-AIMED POINTERS, NO NEW TABLE FIELD, AND THE PLANT**
//!
//! | | Python | slot | table |
//! |---|---|---|---|
//! | swap | `at_lever` (`engine.py:20946`) | `LeverHooks::at_lever` | [`R79`] |
//! | swap | `_shared_rig` (`engine.py:20965`) | [`shared_rig`](TripleHooks::shared_rig) | [`R79_TRIPLE`] |
//! | swap | `_cap_fuel` (`engine.py:20904`) | [`cap_fuel`](TripleHooks::cap_fuel) | [`R79_TRIPLE`] |
//! | swap | `_with_coord` (`engine.py:20973`) | [`with_coord`](TripleHooks::with_coord) | [`R79_TRIPLE`] |
//!
//! Plan § 5.33 (ii): `0 ADD`, so `TripleHooks` stays at **18** fields and — as at slices AG and AH
//! — a forgotten re-aim COMPILES and answers the reduce arm. The instrument is function-pointer
//! identity in both directions, in `tests/slice_ai_cells.rs`.
//!
//! **THE STEP BOUNDARY IS RE-CUT BY ONE SECTION**, and for the reason slice AG re-cut its own: the
//! pre-flight's step list (§ 5.33 (ix) P7) put `_phi_residual`, `_phi_cap` and `_cap_fuel` at step
//! 2. A re-aimed pointer needs a body to point at, and a stub — or leaving the cell at rung 78's
//! body until step 2 — would make step 1's own identity gate report a swap that has not happened.
//! So the PLANT lands here ([`phi_residual`], [`phi_cap`], [`r79_cap_fuel`]) and step 2 is
//! `_coord_at`, `coord_scan` and `coord_census`. `_with_probe` stays at step 3; its flag and log
//! are declared here only because the plant READS the flag.
//!
//! # THE SETTER RE-AIM IS § 5.33 (i)'s LATENT PYTHON DEFECT, REPRODUCED ON PURPOSE
//!
//! Rung 74's `demand_gains` pins `m._lag_coord = "clip"` by assignment (`engine.py:18269`) and then
//! enters `m._with_coord("demand", …)` (`engine.py:18278`) — a DISPATCH. On a rung-79 or rung-80
//! machine that lands on [`r79_with_coord`], which writes `phi_ref`: inside the scope the pair is
//! `("clip", "demand")` where rung 74 meant `("demand", "phi")`. It is value-invisible behind two
//! masks (the pre-flight measured 0 of 196 keys moving over nine walls while this branch ran 128
//! times), and it is RECORDED, NOT REPAIRED. The port is a translation, and because
//! [`CoordScope`](crate::demand_coordinate::CoordScope) goes THROUGH the table, re-aiming the cell
//! reproduces it exactly. **If the Python is ever repaired** — for instance by giving rung 79's
//! setter its own name — this cell's re-aim disappears with it, and
//! `slice_ai_cells.rs`'s readback gate flips in exactly one assertion.
//!
//! # THE COUNTERS ARE THREAD-LOCAL — **THE OPPOSITE OF RUNG 78's DECISION, MADE ON MEASURED GROUNDS**
//!
//! Python's six counters, flag and log are CLASS attributes, for the reason its own comment gives
//! (`self._x += 1` would create an instance attribute and leave the class one at zero forever).
//! Rung 78 ported its two as process-global `static`s
//! ([`GAUGE_HITS`](crate::residual_gauge::GAUGE_HITS)); **this rung's are `thread_local!`**, on
//! three grounds from plan § 5.33 (iv):
//!
//! 1. **Nothing in the crate spawns a thread** — `rust/src`, `rust/tests`, `rust/oracle` and
//!    `rust/examples` hold zero `thread::spawn`/`scope`/`Builder`, zero `rayon`. A march runs on its
//!    caller's thread, so per-thread counts equal Python's per-process ones for every value a reader
//!    returns. **Falsified the day a march crosses a thread** (§ 5.33 (ix) P3).
//! 2. **A `static` is corrupted by any other test in the binary that marches an incidence
//!    machine**, and the plant bumps on EVERY incidence call — including every rung-80–84 march
//!    slice AJ will port. That is slice AH step 7's race at six counters instead of two, and the
//!    lock that answers it would have to spread into every later test file. A lock protects the
//!    resource you wrap it around; a thread-local removes the shared resource.
//! 3. **The crate's own counters are already thread-local** — nine modules use `thread_local!` +
//!    a `bump`. Rung 78's pair is the exception, and its reason (a per-core `Cell` would be Python's
//!    rejected instance attribute) is one a thread-local satisfies too.
//!
//! **Rung 78's pair is knowingly NOT converted.** They are shipped, slice AH step 7's `Mutex`
//! protects them, and converting them moves no value. So the crate carries two conventions for one
//! Python construct, and this paragraph is where that is said.
//!
//! **THE FLAG AND THE LOG ARE NOT CORE FIELDS EITHER.** Rung 79's own `_with_probe` docstring records
//! that shape shipping once: the flag written on the INSTANCE, the march building a NEW rig through
//! `at_lever`, and the log coming back EMPTY while `hits`/`binds` read 1366/1366. A core field is
//! the Rust spelling of that instance attribute.
//!
//! # `phi_ref` IS A STRING CELL, BECAUSE A THIRD VALUE IS REACHED
//!
//! Python declares `"phi" | "incidence"` and tests `== "phi"`, treating everything else as
//! incidence. The setter re-aim above writes `"demand"` there on a callable path, so the port keeps
//! the string and the `==` test; see
//! [`phi_ref`](crate::two_spool_transient::TwoSpoolTransientCore::phi_ref).

use std::cell::{Cell, RefCell};

use crate::bleed_transient::{LeverArm, LeverHooks};
use crate::demand_coordinate::cap_free;
use crate::engine::FlightCondition;
use crate::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelTransientCore, FuelTransientHooks,
};
use crate::gas::Abort;
use crate::map::ComponentMap;
use crate::residual_gauge::{GAUGE_K_IDENTITY, R78, R78_FUEL, R78_STATOR, R78_TRIPLE, R78_TWO};
use crate::shared_actuator::SharedRigArm;
use crate::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient, StatorTransientHooks};
use crate::three_loop::TripleHooks;
use crate::two_spool::TwoSpoolEngine;
use crate::two_spool_transient::{TwoSpoolTransientCore, TwoSpoolTransientHooks};

// ---------------------------------------------------------------------------------------------
// THE DECLARED KNOB
// ---------------------------------------------------------------------------------------------

/// Python's `_phi_ref = "phi"` — **THE CLASS DEFAULT, AND IT IS THE REDUCE ARM.** At this value
/// [`r79_cap_fuel`] dispatches to rung 78's body on an exact comparison and not one float moves.
pub const PHI_REF_PHI: &str = "phi";

/// The one other value Python's class docstring declares — rung 60's currency. **Not the only
/// other value that arrives**: anything that is not [`PHI_REF_PHI`] takes the incidence branch,
/// including the `"demand"` that § 5.33 (i)'s re-aim writes.
pub const PHI_REF_INCIDENCE: &str = "incidence";

// ---------------------------------------------------------------------------------------------
// THE INSTRUMENTS — six counters, a flag and a log, per THREAD (module header)
// ---------------------------------------------------------------------------------------------

/// One row of § 5's probe log — Python's 5-tuple `(a_cap, p_phi, p_cap, mf_sched, used_fb)`,
/// named. Appended only while the probe flag is up (`_with_probe`, step 3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoordLogRow {
    /// The accel leg's cap — rung 78's body with the phi leg disarmed.
    pub a_cap: f64,
    /// The SAME phi leg solved in the `phi` coordinate at the same frozen state — the instrument.
    pub p_phi: f64,
    /// The phi leg in the machine's own coordinate — the plant.
    pub p_cap: f64,
    /// Logged because `binds` is not the last selector: `_applied_demand` takes a further `min`.
    pub mf_sched: f64,
    /// Whether THIS call's incidence solve short-circuited to the shipped fallback.
    pub used_fb: bool,
}

thread_local! {
    static COORD_HITS: Cell<u64> = const { Cell::new(0) };
    static COORD_BINDS: Cell<u64> = const { Cell::new(0) };
    static COORD_FB_PHI: Cell<u64> = const { Cell::new(0) };
    static COORD_FB_INC: Cell<u64> = const { Cell::new(0) };
    static COORD_CALLS_PHI: Cell<u64> = const { Cell::new(0) };
    static COORD_CALLS_INC: Cell<u64> = const { Cell::new(0) };
    static COORD_PROBE: Cell<bool> = const { Cell::new(false) };
    static COORD_LOG: RefCell<Option<Vec<CoordLogRow>>> = const { RefCell::new(None) };
}

fn bump(c: &'static std::thread::LocalKey<Cell<u64>>) {
    c.with(|x| x.set(x.get() + 1));
}

fn read(c: &'static std::thread::LocalKey<Cell<u64>>) -> u64 {
    c.with(Cell::get)
}

/// Rung 79's six class counters, read together — Python's `_coord_hits`, `_coord_binds`,
/// `_coord_fb_phi`, `_coord_fb_inc`, `_coord_calls_phi` and `_coord_calls_inc`.
///
/// **THE PUBLIC READER IS A STEP-1 DELIVERABLE (§ 5.33 (ix) P2a).** `demand_gains` does not RETURN
/// these, so the oracle arm that pins § (i)'s break snapshots them around the call, and the Rust
/// side needs a way to read its thread-locals from outside the module.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CoordCounters {
    /// The incidence branch RAN.
    pub hits: u64,
    /// … and its value WON the min-select — compared BY NAME, never by position (Python's own
    /// warning against rung 78's `out == caps[0]`).
    pub binds: u64,
    /// `_cap_free` returned the SHIPPED fallback, in the `phi` coordinate.
    pub fb_phi: u64,
    /// … and in the incidence coordinate. `_surge_fuel` brackets its own HARDCODED `phi`
    /// residual, so a fallback here SUBSTITUTES the original coordinate rather than bypassing one.
    pub fb_inc: u64,
    /// Completed phi-leg set-point solves, in `phi`.
    pub calls_phi: u64,
    /// … and in incidence. `fb_inc < calls_inc` is § 5's non-vacuity condition.
    pub calls_inc: u64,
}

/// Read all six on THIS thread.
pub fn coord_counters() -> CoordCounters {
    CoordCounters {
        hits: read(&COORD_HITS),
        binds: read(&COORD_BINDS),
        fb_phi: read(&COORD_FB_PHI),
        fb_inc: read(&COORD_FB_INC),
        calls_phi: read(&COORD_CALLS_PHI),
        calls_inc: read(&COORD_CALLS_INC),
    }
}

/// Zero all six on THIS thread, returning what they held — Python's `coord_march` zeroes them as a
/// set before it reads them back, and a reset by halves would be a count from two different
/// starts. **Per-thread state survives from one test to the next on the same thread** (the harness
/// reuses threads, and `--test-threads=1` puts every test on one), so a gate that wants an absolute
/// count resets first and never assumes zero.
pub fn reset_coord_counters() -> CoordCounters {
    let out = coord_counters();
    for c in [&COORD_HITS, &COORD_BINDS, &COORD_FB_PHI, &COORD_FB_INC, &COORD_CALLS_PHI,
              &COORD_CALLS_INC] {
        c.with(|x| x.set(0));
    }
    out
}

/// Whether § 5's probe is armed on this thread — Python's `_coord_probe`. Read by the plant;
/// written only by `_with_probe` (step 3).
pub fn coord_probe_armed() -> bool {
    COORD_PROBE.with(Cell::get)
}

// ---------------------------------------------------------------------------------------------
// THE CASCADE BUILDER
// ---------------------------------------------------------------------------------------------

/// Build a rung-79 object, so every sibling re-asserts the whole chain's guards.
///
/// `_ref_law` is set for rung 73's reason, inherited through the chain rather than copied. The new
/// knob is NOT written: `_phi_ref`'s class default is `"phi"` and so is the core's, so a set would
/// be a line that looks like it is doing the `ref_law` job — rung 78's builder's reason, one knob
/// over.
pub fn build_state_coordinate_cascade(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    let built = crate::reference_split::build_split_family_cascade(
        design_engine, flight_design, mdot_design, map_lp, map_hp, rho, arm,
        &R79_TWO, &R79_STATOR, &R79_FUEL, &R79, &R79_TRIPLE);
    if let ScheduledStatorTransient::Full(c) = &built {
        c.fuel.inner.ref_law.set(crate::applied_reference::REF_LAW_APPLIED);
    }
    built
}

// ---------------------------------------------------------------------------------------------
// THE TABLES
// ---------------------------------------------------------------------------------------------

/// RUNG 79's lever table — ONE swap, `at_lever`, against rung 78's.
///
/// The SEVENTEENTH instance of the sibling-constructor trap. Rung 78 lost the class and the gauge
/// here; this rung would lose the COORDINATE on top of both, and every reader below would silently
/// run at `"phi"` — rung 78's plant, reported as rung 79's.
pub const R79: LeverHooks = LeverHooks {
    at_lever: r79_at_lever,
    ..R78
};

/// RUNG 79's `TwoSpoolTransientHooks` — **ZERO cells swapped**, an alias.
pub const R79_TWO: TwoSpoolTransientHooks = R78_TWO;

/// RUNG 79's fuel table — **ZERO cells swapped**, an alias. The rung's one refusal lives in the
/// CAP (`engine.py:20910`), not in the march.
pub const R79_FUEL: FuelTransientHooks = R78_FUEL;

/// RUNG 79's stator table — **ZERO cells swapped**, an alias.
pub const R79_STATOR: StatorTransientHooks = R78_STATOR;

/// RUNG 79's third-loop table — **THREE of rung 78's eighteen cells re-aimed, and NOTHING added.**
///
/// Spelled out field by field, [`R78_TRIPLE`]'s reason: with no width tripwire the exhaustive
/// literal is the only place a reader sees the three moves and the fifteen stays at once. Two of
/// the stays matter here: `sensed_cap` remains RUNG 76's body (rung 78's `cap_fuel`, reached through
/// [`r79_cap_fuel`]'s accel call, consults it), and `quad_gains_at` remains rung 73's — the reader
/// through which § 5.33 (i)'s `demand_gains` reaches this table at all.
pub const R79_TRIPLE: TripleHooks = TripleHooks {
    stator_leg: R78_TRIPLE.stator_leg,
    lagged_stator: R78_TRIPLE.lagged_stator,
    clamp_v: R78_TRIPLE.clamp_v,
    check_v0: R78_TRIPLE.check_v0,
    rk4_floor: R78_TRIPLE.rk4_floor,
    solve_v: R78_TRIPLE.solve_v,
    manifold_v: R78_TRIPLE.manifold_v,
    triple_laws: R78_TRIPLE.triple_laws,
    triple_rig: R78_TRIPLE.triple_rig,
    with_ref: R78_TRIPLE.with_ref,
    reference: R78_TRIPLE.reference,
    quad_gains_at: R78_TRIPLE.quad_gains_at,
    rk4_floor_shared: R78_TRIPLE.rk4_floor_shared,
    windup_tau: R78_TRIPLE.windup_tau,
    sensed_cap: R78_TRIPLE.sensed_cap,
    // THE THREE THIS RUNG RE-AIMS.
    shared_rig: r79_shared_rig,
    cap_fuel: r79_cap_fuel,
    with_coord: r79_with_coord,
};

// ---------------------------------------------------------------------------------------------
// THE RE-AIMED BODIES
// ---------------------------------------------------------------------------------------------

/// RUNG 79's `at_lever` — **rung 78's sibling constructor returning a RUNG-79 machine that carries
/// EIGHT knobs** (`engine.py:20960`–`engine.py:20962`).
///
/// See [`r77_at_lever`](crate::stiffness_ledger)'s note for why the copy lines are not factored onto
/// the parent: redundant on the value, load-bearing on the pointer identity.
fn r79_at_lever(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    let m = match build_state_coordinate_cascade(
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
    m.fuel.inner.phi_ref.set(core.fuel.inner.phi_ref.get());
    m
}

/// RUNG 79's `_shared_rig` — rung 78's rig with the COORDINATE carried too (`engine.py:20970`).
///
/// A plain `set`, not the [`with_coord`](TripleHooks::with_coord) cell: § 5.30.6's four-site rule
/// governs the SETTER, not every write of the field. Predicted a no-op for the reason its
/// predecessors were ([`r79_at_lever`] has already copied it) and ported regardless — slice AG
/// measured that deleting the equivalent line leaves every value gate green and kills the pointer
/// gate, because the linker folds a pure pass-through onto its parent's address.
fn r79_shared_rig(
    core: &ScheduledStatorCore, arm: &SharedRigArm,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>) {
    let (m, surge, lag) = (R78_TRIPLE.shared_rig)(core, arm);
    m.fuel.inner.phi_ref.set(core.fuel.inner.phi_ref.get());
    (m, surge, lag)
}

/// RUNG 79's `_with_coord` — **THE SETTER, AND IT WRITES `phi_ref`** (`engine.py:20973`).
///
/// Rung 74's body ([`r74_with_coord`](crate::demand_coordinate)) has the same arity and writes
/// `lag_coord`; the second parameter is renamed `coord` → `ref`, and every dispatch site passes it
/// positionally, so the pair is substitutable and the swap is legal (§ 5.33 (ii)). **Five dispatch
/// sites reach this body** — four in rung 79's own `coord_march` (one a bound-method reference
/// handed to `_with_probe`) and one in rung 74's `demand_gains`, which is the module header's
/// latent defect.
fn r79_with_coord(t: &TwoSpoolTransientCore, r: &'static str) -> &'static str {
    let prev = t.phi_ref.get();
    t.phi_ref.set(r);
    prev
}

// ---------------------------------------------------------------------------------------------
// THE PLANT — the knob in ONE place, the phi leg's set point, and the min-select
// ---------------------------------------------------------------------------------------------

/// `Gs` or `Gi` — Python's `_phi_residual` (`engine.py:20853`), **the ONE place the coordinate is
/// chosen**, so no reader can drift from the plant.
///
/// `coord` overrides the machine's own `phi_ref` (Python's `coord or self._phi_ref`). `phi_lim` is
/// read RAW off the floor, exactly as the shipped `_cap_fuel` reads it.
///
/// **THE INCIDENCE ARM COMPUTES `1/phi_lim` ONCE, OUTSIDE THE CLOSURE**, and subtracts it from
/// `1/phi(w)` — Python's `inv = 1.0 / surge.phi_lim` then `1.0 / … − inv`. Folding the two
/// reciprocals into one expression would round differently.
pub fn phi_residual<'a>(
    ft: &'a FuelTransientCore, flight: &'a FlightCondition, a: f64, h: f64, surge: &'a Floor,
    coord: Option<&'static str>,
) -> Box<dyn Fn(f64) -> Result<f64, Abort> + 'a> {
    let s = surge.phi();
    if coord.unwrap_or(ft.inner.phi_ref.get()) == PHI_REF_PHI {
        return Box::new(move |w: f64| Ok(s.phi_lim - s.read(&ft.try_instant_fuel(flight, a, h, w)?)));
    }
    let inv = 1.0 / s.phi_lim;
    Box::new(move |w: f64| Ok(1.0 / s.read(&ft.try_instant_fuel(flight, a, h, w)?) - inv))
}

/// The phi leg's unfloored set point in the chosen coordinate — Python's `_phi_cap`
/// (`engine.py:20867`).
///
/// **THE FALLBACK IS THE SHIPPED `_surge_fuel`, DELIBERATELY**: it solves the same root SET as `Gi`.
/// But it brackets its own HARDCODED `phi` residual, so on a binding leg the coordinate is not
/// inert, it is UNREACHABLE — and the fallback is counted, per coordinate, which is § 5.1.
///
/// **THE `calls_*` COUNTERS BUMP ONLY AFTER THE SOLVE RETURNS**: Python increments after
/// `_cap_free` comes back, so a solve that raises is never counted. `?` above the bump is that.
pub fn phi_cap(
    ft: &FuelTransientCore, flight: &FlightCondition, a: f64, h: f64, mf_sched: f64,
    surge: &Floor, coord: Option<&'static str>,
) -> Result<f64, Abort> {
    let is_phi = coord.unwrap_or(ft.inner.phi_ref.get()) == PHI_REF_PHI;
    let shipped = || -> Result<f64, Abort> {
        bump(if is_phi { &COORD_FB_PHI } else { &COORD_FB_INC });
        ft.try_surge_fuel(flight, a, h, mf_sched, surge)
    };
    let big_g = phi_residual(ft, flight, a, h, surge, coord);
    let out = cap_free(&*big_g, mf_sched, &shipped)?;
    bump(if is_phi { &COORD_CALLS_PHI } else { &COORD_CALLS_INC });
    Ok(out)
}

/// RUNG 79's `_cap_fuel` — **rung 78's min-select with the PHI branch re-coordinated**
/// (`engine.py:20904`).
///
/// At `"phi"` this dispatches to rung 78's body on an exact `==` and the whole family is
/// bit-for-bit rung 78. Anything else — `"incidence"`, and the `"demand"` § 5.33 (i) reaches — takes
/// the incidence branch.
///
/// **THE REFUSAL IS AN `Abort`, NOT A `panic!`** (`engine.py:20910`): an incidence phi leg × a
/// non-identity rung-78 gauge. Python spells it `assert` inside a function reached from marches that
/// wrap their derivative in `except AssertionError`, so a panic would end the process where Python
/// ends the march — rung 78's two refusals, one knob over, and slice L's per-call-site rule. The
/// `==` is written negated (`!(k == 1.0)`) so a NaN gauge is refused, as Python's `assert` refuses
/// it.
///
/// **THE ACCEL LEG IS RUNG 78's, UNTOUCHED** — taken by calling it with the phi leg disarmed, as
/// Python calls `super()` with `surge=None` rather than re-deriving the leg.
///
/// **THE `min` IS PYTHON's**: `min(a_cap, p_cap)` keeps the FIRST of equals and has no NaN rule, so
/// it is `if p_cap < a_cap { p_cap } else { a_cap }` and never `f64::min`. `binds` counts
/// `p_cap <= a_cap` — the phi leg compared BY NAME, not by position.
#[allow(clippy::too_many_arguments)]
fn r79_cap_fuel(
    ft: &FuelTransientCore, flight: &FlightCondition, a: f64, h: f64, mf_sched: f64,
    accel: Option<&AccelSchedule>, surge: Option<&Floor>, mf_app: Option<f64>,
) -> Result<f64, Abort> {
    let r = ft.inner.phi_ref.get();
    if r == PHI_REF_PHI {
        return (R78_TRIPLE.cap_fuel)(ft, flight, a, h, mf_sched, accel, surge, mf_app);
    }
    let k = ft.inner.gauge_k.get();
    if !(k == GAUGE_K_IDENTITY) {
        return Err(Abort(format!(
            "rung-79: an INCIDENCE phi leg x a non-identity rung-78 GAUGE is REFUSED. The two \
             knobs re-write DIFFERENT legs' residuals, so composing them is neither rung 78 nor \
             rung 79 and nothing measures it -- rung 78 s 0.3's refusal of `sensed x gauge`, one \
             knob over. Got _phi_ref = '{r}', _gauge_k = {k:?}.")));
    }
    let a_cap = (R78_TRIPLE.cap_fuel)(ft, flight, a, h, mf_sched, accel, None, mf_app)?;
    let Some(surge) = surge else {
        return Ok(a_cap);
    };
    bump(&COORD_HITS);
    let fb0 = read(&COORD_FB_INC);
    let p_cap = phi_cap(ft, flight, a, h, mf_sched, surge, None)?;
    // Differenced around the solve, not inferred from a total — § 5.3's complementarity is a
    // per-call claim. `fb_inc` ONLY, as Python's `used_fb`.
    let used_fb = read(&COORD_FB_INC) != fb0;
    if coord_probe_armed() {
        // AN INSTRUMENT, NEVER THE PLANT: the same leg in the `phi` coordinate at the same frozen
        // state. It bumps the PHI-side counters, as Python's second `_phi_cap` does.
        let p_phi = phi_cap(ft, flight, a, h, mf_sched, surge, Some(PHI_REF_PHI))?;
        COORD_LOG.with(|l| {
            l.borrow_mut()
                .as_mut()
                .expect("rung-79: the probe flag is up with no log -- Python's `None.append`, an \
                         AttributeError. Only `_with_probe` may raise the flag, and it sets both.")
                .push(CoordLogRow { a_cap, p_phi, p_cap, mf_sched, used_fb });
        });
    }
    if p_cap <= a_cap {
        bump(&COORD_BINDS);
    }
    Ok(if p_cap < a_cap { p_cap } else { a_cap })
}
