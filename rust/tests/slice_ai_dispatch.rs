//! SLICE AI step 7 — **THE DISPATCH GATES for rungs 79 and 80.**
//!
//! Six swaps — rung 79's `at_lever`, `shared_rig`, `cap_fuel` and `with_coord`, rung 80's `at_lever`
//! and `shared_rig` — each pointed back at the parent it was re-aimed FROM, and read at every seat
//! Python can call on that rung's machine. `slice_ai_cells.rs` gates the pointers,
//! `slice_ai_oracle.rs` 37 945 values against two interpreters, `rung79.rs` / `rung80.rs` the
//! suites; all of them read machines this slice's own builders assembled, so none can say which
//! FUNCTION sat in a slot while a reader ran. That is this file's subject (plan § 5.33.7).
//!
//! # A CELL IS A READING, A CALL COUNT, AND RUNG 79's SIX COUNTERS
//!
//! A reading is `format!("{x:?}")` over the reader's own `Debug` — shortest-round-trip, so
//! injective on bit patterns (AF/AG/AH's fingerprint). Beside it every cell records:
//!
//! * **how often each observed cell was ENTERED** (the `Count` row only) — AH § 5.32.7 (b): a
//!   one-bit verdict prints `same` both for *ran, and made no difference* and for *never entered*;
//! * **rung 79's six counters, reset before the seat and read after it.** This slice's leading
//!   hazard (plan § 5.33 (i)) is VALUE-INVISIBLE by construction — `with_coord`'s re-aim moves 0 of
//!   196 keys while its branch runs 128 times — so a row that reads `same` on values alone would
//!   print the headline and mean nothing. The counters are part of the purity proof too.
//!
//! # THE ROWS RUN ON PARALLEL THREADS, AND THAT IS SOUND ONLY BECAUSE EVERYTHING READ HERE IS PER-THREAD
//!
//! Rung 79's counters are `thread_local!` (§ 5.33 (iv), P3 confirmed at step 6), and so are this
//! file's call counters. Each row is built, reset, run and read on ONE spawned thread. **Nothing here
//! reads rung 78's process-global `GAUGE_HITS`** — that would need slice AH step 7's `Mutex` back.
//! The one process-wide thing touched is a `Once`-installed panic hook gated by a thread-local flag.
//!
//! # DECLARED NARROWINGS
//!
//! * **The PHI stator arm only** — the suites' rig. The oracle measured the incidence arm to be a
//!   different plant (2 riding points against 10), and it sweeps both.
//! * **Rung-80 readers are not seated on rung-79 machines.** Python cannot call them there, and the
//!   reading would only re-measure the rung-80 `SharedRig` row.
//! * **`demand_gains` sits at § 5.33 (i)'s settings** — wall 0.80, `every = 4`, `ds = 0.005` — and
//!   the shipped rows must reproduce the pre-flight's 128 incidence hits with `fb_inc = calls_inc`,
//!   or the seat is not on the measured rig ([`every_baseline_seat_reads_on_the_measured_rig`]).
//! * `split_gains` sits in `clip`, its non-vacuous coordinate on this arm (step 4: `demand` is
//!   `vacuous`); `split_liveness` carries both coordinates anyway.
//!
//! # WHAT THE MATRIX FOUND — every verdict row as pre-registered (`W:\temp\claude\slice-ai-step7`)
//!
//! * **Rung 79's three table swaps are ONE deletion to every seat** — `coord_march` reads one
//!   identical string under `at_lever`, `cap_fuel` and `with_coord` pointed back, and `demand_gains`
//!   reads the shipped values under all three with § 5.33 (i)'s 128 hits gone to 0.
//! * **THREE KINDS OF SILENCE, not AH's two.** *Ran, no difference* (`shared_rig → R78`, redundant
//!   behind `at_lever`'s carriage); *never entered* (`cap_fuel` at every CLIP-only seat); and a third,
//!   *entered, built a different machine, never dispatched it*: under `at_lever → R78` the three
//!   gauge-point readers march a rig carrying RUNG 78's `cap_fuel` and `with_coord` and read `same`,
//!   because they call the plant with an explicit coordinate and never touch the rig's table
//!   (*never dispatched* is read off the `Count` row — an inference across rows, sound because
//!   the dispatch sites belong to the readers, not the tables).
//! * **Rung 80's `at_lever` is invisible to every value-bearing seat** — silent at all nine seats,
//!   on values and counters, while entered at every one: `r80_shared_rig` re-reads `sm_air` off
//!   the CORE. The closing source mutation (plan § 5.33.7 (e)) is caught only by STRUCTURAL gates
//!   — pointer identity AND a readback of the sibling's `sm_air`. Measured in Rust through rung 80.
//!
//! # THE REBUILD HELPER IS PROVED AGAINST THE SHIPPED CONSTRUCTOR BEFORE ANY ROW IS READ
//!
//! An injected machine must survive a reader's rebuild, so every macro-built row carries an
//! `at_lever` that rebuilds with ITS OWN tables — carrying EIGHT knobs at rung 79 (`phi_ref` the
//! eighth) and NINE at rung 80 (`sm_air`).
//! [`the_rebuild_helper_is_the_shipped_constructor`] checks them off-default against the shipped
//! constructors, non-vacuity halves included, and a `MacroNone` row per rung must read `same`.

use std::cell::Cell;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::fn_addr_eq;
use std::sync::OnceLock;
use std::time::Instant;

use turbojet::bleed_transient::{LeverArm, LeverArming, LeverHooks};
use turbojet::demand_coordinate::demand_gains;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelTransientCore, FuelTransientHooks,
};
use turbojet::gas::{Abort, Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::reference_split::StatorIncidenceLimiter;
use turbojet::residual_gauge::{gauge_points, R78, R78_TRIPLE};
use turbojet::shared_actuator::SharedRigArm;
use turbojet::split_wall::{
    build_split_wall_cascade, split_arrest, split_gains, split_liveness, split_march,
    split_saturation, walls_of, ARREST_WALLS, GAINS_EVERY, LIVENESS_COORDS, LIVENESS_PHI_AIRS,
    R80, R80_FUEL, R80_STATOR, R80_TRIPLE, R80_TWO, SATURATION_PHI_AIRS,
};
use turbojet::state_coordinate::{
    build_state_coordinate_cascade, coord_census, coord_counters, coord_forced, coord_march,
    coord_scan, reset_coord_counters, CoordCounters, COORD_AT_DQ, COORD_CENSUS_WALK,
    PHI_REF_INCIDENCE, PHI_REF_PHI, R79, R79_FUEL, R79_STATOR, R79_TRIPLE, R79_TWO,
};
use turbojet::stator_transient::{
    ScheduledStatorCore, ScheduledStatorTransient, StatorTransientHooks,
};
use turbojet::three_loop::{StatorLimiter, TripleHooks};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::{TwoSpoolTransientCore, TwoSpoolTransientHooks};

// ============================================================================ the grid
//
// `slice_ai_oracle.rs`'s, which is both suites'.

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
const V_MAX: f64 = 0.20;
const TT4_MAX: f64 = 1200.0;
const TAUS: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
/// Both suites' receiver wall.
const PHI_RIG: f64 = 0.80;
/// Rung 79's readers' `phi_lim`, `margin` and `every`.
const C_PHI: f64 = 0.80;
const C_MARGIN: f64 = 0.10;
const C_EVERY: usize = 8;
/// Rung 80's `PHI_FUEL`, `split_arrest`'s `phi_air_hi`, and the suite's gains walls.
const PHI_FUEL: f64 = 0.75;
const ARREST_AIR_HI: f64 = 0.80;
const GAIN_AIRS: [Option<f64>; 3] = [None, Some(0.77), Some(0.80)];
/// § 5.33 (i)'s `demand_gains` settings — section I's first wall.
const I_WALL: f64 = 0.80;
const I_EVERY: usize = 4;

/// Off-default values for the knob-carriage proof.
const SM_AIR_PROBE: f64 = 0.5;

// ============================================================================ the fixtures

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

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

/// The suites' arm: valve + PHI stator at the receiver wall.
fn arm() -> LeverArm {
    let sm = PHI_RIG / FLOOR - 1.0;
    LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, sm, Some(TAU))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S))),
        ..Default::default()
    }
}

/// The incidence arm — used ONLY by the knob-carriage proof, so a helper that copied the ARM
/// instead of taking the argument would show.
fn inc_arm() -> LeverArm {
    let sm = PHI_RIG / FLOOR - 1.0;
    LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, sm, Some(TAU))),
        stator_inc: Some(StatorIncidenceLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S))),
        ..Default::default()
    }
}

/// The five knobs both suites' fixtures write by PLAIN ASSIGNMENT — the oracle's `rig()` state.
fn set_rig_knobs(m: &ScheduledStatorCore) {
    let t = &m.fuel.inner;
    t.lag_coord.set("demand");
    t.ref_law.set("sched");
    t.windup_law.set("none");
    t.tau_t.set(None);
    t.cap_law.set("solve");
}

thread_local! {
    static QUIET: Cell<bool> = const { Cell::new(false) };
}

/// Silences a panic per THREAD, not per process — slice AE's recorded reason.
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

// =============================================================================================
// 1 — THE INJECTIONS AND THE OBSERVERS
// =============================================================================================

thread_local! {
    static N_AT: Cell<usize> = const { Cell::new(0) };
    static N_RIG: Cell<usize> = const { Cell::new(0) };
    static N_CAP: Cell<usize> = const { Cell::new(0) };
    static N_COORD: Cell<usize> = const { Cell::new(0) };
}

fn bump(c: &'static std::thread::LocalKey<Cell<usize>>) { c.with(|n| n.set(n.get() + 1)); }

fn reset_counts() {
    for c in [&N_AT, &N_RIG, &N_CAP, &N_COORD] {
        c.with(|n| n.set(0));
    }
}

/// `[at_lever, shared_rig, cap_fuel, with_coord]` entries on THIS thread.
fn counts() -> [usize; 4] {
    [N_AT.with(Cell::get), N_RIG.with(Cell::get), N_CAP.with(Cell::get), N_COORD.with(Cell::get)]
}

fn no_enter() {}
fn enter_at() { bump(&N_AT); }

/// The observers delegate to rung 79's SHIPPED bodies — which rung 80 inherits for `cap_fuel` and
/// `with_coord`, so one wrapper serves both rungs.
fn counting_rig_79(
    core: &ScheduledStatorCore, a: &SharedRigArm,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>) {
    bump(&N_RIG);
    (R79_TRIPLE.shared_rig)(core, a)
}

fn counting_rig_80(
    core: &ScheduledStatorCore, a: &SharedRigArm,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>) {
    bump(&N_RIG);
    (R80_TRIPLE.shared_rig)(core, a)
}

#[allow(clippy::too_many_arguments)]
fn counting_cap(
    ft: &FuelTransientCore, flight: &FlightCondition, a: f64, h: f64, mf_sched: f64,
    accel: Option<&AccelSchedule>, surge: Option<&Floor>, mf_app: Option<f64>,
) -> Result<f64, Abort> {
    bump(&N_CAP);
    (R79_TRIPLE.cap_fuel)(ft, flight, a, h, mf_sched, accel, surge, mf_app)
}

fn counting_coord(t: &TwoSpoolTransientCore, r: &'static str) -> &'static str {
    bump(&N_COORD);
    (R79_TRIPLE.with_coord)(t, r)
}

// -------------------------------------------------------------- the injected tables

/// Each swap is the PARENT's own shipped cell — a pointer, nothing this file built. Rung 79's
/// `with_coord` parent is rung 78's cell, which is rung 74's body (`r74_with_coord`).
static T79_SHARED_RIG: TripleHooks = TripleHooks { shared_rig: R78_TRIPLE.shared_rig, ..R79_TRIPLE };
static T79_CAP_FUEL: TripleHooks = TripleHooks { cap_fuel: R78_TRIPLE.cap_fuel, ..R79_TRIPLE };
static T79_WITH_COORD: TripleHooks = TripleHooks { with_coord: R78_TRIPLE.with_coord, ..R79_TRIPLE };
static T79_COUNT: TripleHooks = TripleHooks {
    shared_rig: counting_rig_79, cap_fuel: counting_cap, with_coord: counting_coord, ..R79_TRIPLE
};
static T80_SHARED_RIG: TripleHooks = TripleHooks { shared_rig: R79_TRIPLE.shared_rig, ..R80_TRIPLE };
static T80_COUNT: TripleHooks = TripleHooks {
    shared_rig: counting_rig_80, cap_fuel: counting_cap, with_coord: counting_coord, ..R80_TRIPLE
};

/// The two `at_lever` rows — the PARENT's own shipped constructor.
static L79_AT_LEVER: LeverHooks = LeverHooks { at_lever: R78.at_lever, ..R79 };
static L80_AT_LEVER: LeverHooks = LeverHooks { at_lever: R79.at_lever, ..R80 };

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Rung { R79, R80 }

/// The three alias tables — rung 80's are rung 79's, which are rung 78's; named per rung anyway.
fn aliases(rung: Rung) -> (&'static FuelTransientHooks, &'static TwoSpoolTransientHooks,
                           &'static StatorTransientHooks) {
    match rung {
        Rung::R79 => (&R79_FUEL, &R79_TWO, &R79_STATOR),
        Rung::R80 => (&R80_FUEL, &R80_TWO, &R80_STATOR),
    }
}

/// A machine on an arbitrary lever/triple pair — not through either cascade, which hardcode theirs.
fn with_tables(
    rung: Rung, core: &ScheduledStatorCore, a: &LeverArm, lever: &'static LeverHooks,
    triple: &'static TripleHooks,
) -> ScheduledStatorCore {
    let (fuel, two, stator) = aliases(rung);
    full_of(ScheduledStatorTransient::with_ref_tables(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(),
        a.stator, two, stator, fuel, lever,
        LeverArming { bleed: a.bleed, sched: a.bleed_sched, lim: a.bleed_lim },
        triple, a.stator_lim, a.stator_inc))
}

/// An injection's `at_lever`: it rebuilds with **that injection's own tables** and carries the
/// knobs the shipped constructor carries — EIGHT at rung 79 (`r79_at_lever`), NINE at rung 80
/// (`r80_at_lever`, `sm_air` the ninth). `$enter` is the `Count` row's observer, a no-op elsewhere.
macro_rules! injection {
    ($lever:ident, $rebuild:ident, $base:expr, $rung:expr, $triple:expr, $air:expr,
     $enter:expr) => {
        static $lever: LeverHooks = LeverHooks { at_lever: $rebuild, ..$base };
        fn $rebuild(core: &ScheduledStatorCore, a: &LeverArm) -> ScheduledStatorCore {
            ($enter)();
            let m = with_tables($rung, core, a, &$lever, $triple);
            let (t, c) = (&m.fuel.inner, &core.fuel.inner);
            t.ref_law.set(c.ref_law.get());
            t.lag_coord.set(c.lag_coord.get());
            t.windup_law.set(c.windup_law.get());
            t.tau_t.set(c.tau_t.get());
            t.ic_cap.set(c.ic_cap.get());
            t.cap_law.set(c.cap_law.get());
            t.gauge_k.set(c.gauge_k.get());
            t.phi_ref.set(c.phi_ref.get());
            if $air {
                t.sm_air.set(c.sm_air.get());
            }
            m
        }
    };
}

injection!(L79_NONE, r79_none, R79, Rung::R79, &R79_TRIPLE, false, no_enter);
injection!(L79_RIG, r79_rig, R79, Rung::R79, &T79_SHARED_RIG, false, no_enter);
injection!(L79_CAP, r79_cap, R79, Rung::R79, &T79_CAP_FUEL, false, no_enter);
injection!(L79_COORD, r79_coord, R79, Rung::R79, &T79_WITH_COORD, false, no_enter);
injection!(L79_COUNT, r79_count, R79, Rung::R79, &T79_COUNT, false, enter_at);
injection!(L80_NONE, r80_none, R80, Rung::R80, &R80_TRIPLE, true, no_enter);
injection!(L80_RIG, r80_rig, R80, Rung::R80, &T80_SHARED_RIG, true, no_enter);
injection!(L80_COUNT, r80_count, R80, Rung::R80, &T80_COUNT, true, enter_at);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Row {
    /// The shipped cascade, untouched — the baseline.
    Shipped,
    /// The shipped tables through THIS FILE's rebuild helper — must be `same` everywhere.
    MacroNone,
    AtLever,
    SharedRig,
    /// Rung 79 only.
    CapFuel,
    WithCoord,
    /// Every re-aimed cell wrapped in a counter at once — `at_lever`, `shared_rig`, `cap_fuel`,
    /// `with_coord` (rung 80's last two are rung 79's bodies, inherited).
    Count,
}

const ROWS_79: [Row; 7] = [Row::Shipped, Row::MacroNone, Row::AtLever, Row::SharedRig,
                           Row::CapFuel, Row::WithCoord, Row::Count];
const ROWS_80: [Row; 5] = [Row::Shipped, Row::MacroNone, Row::AtLever, Row::SharedRig,
                           Row::Count];

fn shipped_triple(rung: Rung) -> &'static TripleHooks {
    match rung { Rung::R79 => &R79_TRIPLE, Rung::R80 => &R80_TRIPLE }
}

fn shipped_lever(rung: Rung) -> &'static LeverHooks {
    match rung { Rung::R79 => &R79, Rung::R80 => &R80 }
}

/// `None` for the shipped row, which is built by the cascade itself.
fn tables_of(rung: Rung, row: Row) -> Option<(&'static LeverHooks, &'static TripleHooks)> {
    match (rung, row) {
        (_, Row::Shipped) => None,
        (Rung::R79, Row::MacroNone) => Some((&L79_NONE, &R79_TRIPLE)),
        (Rung::R79, Row::AtLever) => Some((&L79_AT_LEVER, &R79_TRIPLE)),
        (Rung::R79, Row::SharedRig) => Some((&L79_RIG, &T79_SHARED_RIG)),
        (Rung::R79, Row::CapFuel) => Some((&L79_CAP, &T79_CAP_FUEL)),
        (Rung::R79, Row::WithCoord) => Some((&L79_COORD, &T79_WITH_COORD)),
        (Rung::R79, Row::Count) => Some((&L79_COUNT, &T79_COUNT)),
        (Rung::R80, Row::MacroNone) => Some((&L80_NONE, &R80_TRIPLE)),
        (Rung::R80, Row::AtLever) => Some((&L80_AT_LEVER, &R80_TRIPLE)),
        (Rung::R80, Row::SharedRig) => Some((&L80_RIG, &T80_SHARED_RIG)),
        (Rung::R80, Row::Count) => Some((&L80_COUNT, &T80_COUNT)),
        (r, w) => panic!("{w:?} is not a row at {r:?} — ROWS_79 / ROWS_80 are the only source"),
    }
}

fn cascade(rung: Rung, a: &LeverArm) -> ScheduledStatorCore {
    full_of(match rung {
        Rung::R79 => build_state_coordinate_cascade(
            design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, a),
        Rung::R80 => build_split_wall_cascade(
            design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, a),
    })
}

/// The injected machine, knobs written by plain assignment afterwards as both suites' fixtures do.
fn build(rung: Rung, row: Row) -> ScheduledStatorCore {
    let a = arm();
    let base = cascade(rung, &a);
    let m = match tables_of(rung, row) {
        None => base,
        Some((lever, triple)) => with_tables(rung, &base, &a, lever, triple),
    };
    set_rig_knobs(&m);
    m
}

/// AG's field-by-field table comparison, exhaustive by destructuring.
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

fn slot_of(row: Row) -> Vec<&'static str> {
    match row {
        Row::SharedRig => vec!["shared_rig"],
        Row::CapFuel => vec!["cap_fuel"],
        Row::WithCoord => vec!["with_coord"],
        Row::Count => vec!["shared_rig", "cap_fuel", "with_coord"],
        _ => vec![],
    }
}

fn parent_and_own_at(rung: Rung) -> (fn(&ScheduledStatorCore, &LeverArm) -> ScheduledStatorCore,
                                     fn(&ScheduledStatorCore, &LeverArm) -> ScheduledStatorCore) {
    match rung {
        Rung::R79 => (R78.at_lever, R79.at_lever),
        Rung::R80 => (R79.at_lever, R80.at_lever),
    }
}

/// **THE INSTALL PROOF.** At the `at_lever` rows it reads the LEVER POINTER in both directions
/// (AH's lesson: the negative half is what a re-aimed `R8x.at_lever` cannot fake); elsewhere the
/// machine AND its sibling must carry exactly this row's slots, or the first rebuild inside any
/// reader launders the injection away.
fn assert_installed(m: &ScheduledStatorCore, rung: Rung, row: Row) {
    assert_eq!(triple_diff(m.triple_hooks(), shipped_triple(rung)), slot_of(row),
               "{rung:?} {row:?}: the machine carries this row's slot and no other");
    let sib = m.at_lever(&arm());
    let (parent_at, own_at) = parent_and_own_at(rung);
    let at = sib.fuel.inner.lever_hooks.at_lever;
    if row == Row::AtLever {
        assert!(fn_addr_eq(at, parent_at),
                "{rung:?}: the parent's constructor builds a PARENT machine — the whole injection");
        assert!(!fn_addr_eq(at, own_at), "{rung:?}: and NOT this rung's");
        return;
    }
    assert_eq!(triple_diff(sib.triple_hooks(), shipped_triple(rung)), slot_of(row),
               "{rung:?} {row:?}: AND THE SIBLING DOES TOO");
    assert!(!fn_addr_eq(at, parent_at), "{rung:?} {row:?}: the sibling is not a parent machine");
}

// =============================================================================================
// 2 — THE SEATS, AND THE ONE MATRIX
// =============================================================================================

/// Rung 79's four readers, rung 74's `demand_gains` (§ 5.33 (i)'s arm), then rung 80's four.
/// A rung-79 row runs the first [`N_SEATS_79`]; a rung-80 row runs all nine.
const SEATS: [&str; 9] = ["coord_scan", "coord_census", "coord_march", "coord_forced",
                          "demand_gains", "split_liveness", "split_arrest", "split_saturation",
                          "split_gains"];
const N_SEATS_79: usize = 5;

fn seats_of(rung: Rung) -> &'static [&'static str] {
    match rung { Rung::R79 => &SEATS[..N_SEATS_79], Rung::R80 => &SEATS[..] }
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum Seen { Read(String), Broke(String) }

/// One seat on one machine: the reading, the four call counts, rung 79's six counters.
#[derive(Clone, Debug)]
struct Cellrun { seen: Seen, counts: [usize; 4], coord: CoordCounters }

fn fingerprint<T: std::fmt::Debug>(x: &T) -> String { format!("{x:?}") }

fn run_seat(name: &str, m: &ScheduledStatorCore) -> Seen {
    quiet_hook();
    QUIET.with(|q| q.set(true));
    let f = flight();
    let out = catch_unwind(AssertUnwindSafe(|| match name {
        "coord_scan" => fingerprint(&coord_scan(
            m, &f, LO, HI, TT4_MAX, C_PHI, C_MARGIN, TAUS, false, R, SETTLE, DS, V_MAX,
            COORD_AT_DQ, C_EVERY)),
        "coord_census" => {
            let (lo, hi, n) = COORD_CENSUS_WALK;
            fingerprint(&coord_census(
                m, &f, LO, HI, TT4_MAX, C_PHI, C_MARGIN, TAUS, false, R, SETTLE, DS, V_MAX,
                C_EVERY, lo, hi, n))
        }
        "coord_march" => fingerprint(&coord_march(
            m, &f, LO, HI, TT4_MAX, C_PHI, C_MARGIN, TAUS, false, R, SETTLE, DS, V_MAX)),
        "coord_forced" => fingerprint(&coord_forced(
            m, &f, LO, HI, TT4_MAX, C_PHI, C_MARGIN, TAUS, false, R, SETTLE, DS, V_MAX, C_EVERY)),
        "demand_gains" => fingerprint(&demand_gains(
            m, &f, LO, HI, TT4_MAX, I_WALL, TAUS, false, R, SETTLE, DS, V_MAX, I_EVERY)),
        "split_liveness" => fingerprint(&split_liveness(
            m, &f, LO, HI, TT4_MAX, PHI_FUEL, &LIVENESS_PHI_AIRS, &LIVENESS_COORDS, TAUS, false,
            R, SETTLE, DS, V_MAX)),
        "split_arrest" => fingerprint(&split_arrest(
            m, &f, LO, HI, TT4_MAX, &ARREST_WALLS, PHI_FUEL, ARREST_AIR_HI, "demand", TAUS,
            false, R, SETTLE, DS, V_MAX)),
        "split_saturation" => fingerprint(&split_saturation(
            m, &f, LO, HI, TT4_MAX, PHI_FUEL, &SATURATION_PHI_AIRS, "demand", TAUS, false, R,
            SETTLE, DS, V_MAX)),
        "split_gains" => match split_gains(m, &f, LO, HI, TT4_MAX, PHI_FUEL, &GAIN_AIRS, "clip",
                                           TAUS, false, R, SETTLE, DS, V_MAX, GAINS_EVERY) {
            Ok(g) => fingerprint(&g),
            Err(e) => panic!("{}", e.0),
        },
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

type Matrix = Vec<((Rung, Row), Vec<Cellrun>)>;

/// One row, on the CALLING thread: installed once, then every seat on a FRESH machine with the
/// counters reset before it and read after it.
fn run_row(rung: Rung, row: Row) -> Vec<Cellrun> {
    assert_installed(&build(rung, row), rung, row);
    let t0 = Instant::now();
    let out = seats_of(rung).iter().map(|s| {
        let m = build(rung, row);
        reset_counts();
        reset_coord_counters();
        let seen = run_seat(s, &m);
        Cellrun { seen, counts: counts(), coord: coord_counters() }
    }).collect();
    println!("{rung:?} {row:?}: {:.1} s", t0.elapsed().as_secs_f64());
    out
}

/// Every row at every seat, ONCE — each row on its OWN spawned thread (module header).
fn matrix() -> &'static Matrix {
    static M: OnceLock<Matrix> = OnceLock::new();
    M.get_or_init(|| {
        let keys: Vec<(Rung, Row)> = ROWS_79.iter().map(|&w| (Rung::R79, w))
            .chain(ROWS_80.iter().map(|&w| (Rung::R80, w)))
            .collect();
        std::thread::scope(|s| {
            let hs: Vec<_> = keys.iter().map(|&(rung, row)| s.spawn(move || run_row(rung, row)))
                .collect();
            keys.iter().zip(hs)
                .map(|(&k, h)| (k, h.join().unwrap_or_else(|_| panic!("{k:?}: the row panicked"))))
                .collect()
        })
    })
}

fn cells(rung: Rung, row: Row) -> &'static [Cellrun] {
    &matrix().iter().find(|(k, _)| *k == (rung, row)).expect("every row ran").1
}

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

fn row_verdicts(rung: Rung, row: Row) -> Vec<&'static str> {
    cells(rung, Row::Shipped).iter().zip(cells(rung, row).iter())
        .map(|(b, g)| verdict(&b.seen, &g.seen)).collect()
}

fn coords_of(rung: Rung, row: Row) -> Vec<CoordCounters> {
    cells(rung, row).iter().map(|c| c.coord).collect()
}

fn hits_of(rung: Rung, row: Row) -> Vec<u64> {
    cells(rung, row).iter().map(|c| c.coord.hits).collect()
}

fn col(rung: Rung, i: usize) -> Vec<usize> {
    cells(rung, Row::Count).iter().map(|c| c.counts[i]).collect()
}

// =============================================================================================
// 3 — THE GATES
// =============================================================================================

/// **EACH INJECTION MOVES EXACTLY ITS OWN SLOT, AND THE TWO `at_lever` ROWS ARE THE PARENT'S OWN
/// CELL** — in both directions. Pointers only; nothing runs.
#[test]
fn each_injection_moves_exactly_its_own_slot() {
    for (rung, rows) in [(Rung::R79, &ROWS_79[..]), (Rung::R80, &ROWS_80[..])] {
        for &row in rows {
            if let Some((_, t)) = tables_of(rung, row) {
                assert_eq!(triple_diff(t, shipped_triple(rung)), slot_of(row), "{rung:?} {row:?}");
            }
        }
    }
    assert!(fn_addr_eq(L79_AT_LEVER.at_lever, R78.at_lever));
    assert!(!fn_addr_eq(L79_AT_LEVER.at_lever, R79.at_lever));
    assert!(fn_addr_eq(L80_AT_LEVER.at_lever, R79.at_lever));
    assert!(!fn_addr_eq(L80_AT_LEVER.at_lever, R80.at_lever));
    // the parent cells ARE different bodies, so no swap is a relabelling
    assert!(!fn_addr_eq(R78_TRIPLE.with_coord, R79_TRIPLE.with_coord));
    assert!(!fn_addr_eq(R78_TRIPLE.cap_fuel, R79_TRIPLE.cap_fuel));
    assert!(!fn_addr_eq(R78_TRIPLE.shared_rig, R79_TRIPLE.shared_rig));
    assert!(!fn_addr_eq(R79_TRIPLE.shared_rig, R80_TRIPLE.shared_rig));
    assert!(!fn_addr_eq(shipped_lever(Rung::R80).at_lever, R79.at_lever));
}

/// **THE REBUILD HELPER IS THE SHIPPED CONSTRUCTOR** — every knob off its default, from a receiver
/// on the phi arm into a sibling on the INCIDENCE arm. Non-vacuity: rung 79's shipped constructor
/// does NOT carry `sm_air` and rung 80's does, so a helper that always or never carried it fails.
#[test]
fn the_rebuild_helper_is_the_shipped_constructor() {
    for rung in [Rung::R79, Rung::R80] {
        let core = cascade(rung, &arm());
        let t = &core.fuel.inner;
        t.lag_coord.set("clip");
        t.ref_law.set("applied");
        t.windup_law.set("track");
        t.tau_t.set(Some(0.0125));
        t.ic_cap.set(123);
        t.cap_law.set("sensed");
        t.gauge_k.set(2.5);
        t.phi_ref.set(PHI_REF_INCIDENCE);
        t.sm_air.set(Some(SM_AIR_PROBE));
        let shipped = (shipped_lever(rung).at_lever)(&core, &inc_arm());
        let (mine_lever, _) = tables_of(rung, Row::MacroNone).unwrap();
        let mine = (mine_lever.at_lever)(&core, &inc_arm());
        let knobs = |m: &ScheduledStatorCore| {
            let t = &m.fuel.inner;
            (t.lag_coord.get(), t.ref_law.get(), t.windup_law.get(), t.tau_t.get(),
             t.ic_cap.get(), t.cap_law.get(), t.gauge_k.get().to_bits(), t.phi_ref.get(),
             t.sm_air.get().map(f64::to_bits))
        };
        assert_eq!(knobs(&mine), knobs(&shipped),
                   "{rung:?}: the helper carries what the shipped constructor carries");
        assert_eq!(triple_diff(mine.triple_hooks(), shipped.triple_hooks()), Vec::<&str>::new());
        assert!(fn_addr_eq(shipped.fuel.inner.lever_hooks.at_lever,
                           shipped_lever(rung).at_lever), "{rung:?}: a machine of its own rung");
        assert!(mine.fuel.inner.stator.inc.is_some() && shipped.fuel.inner.stator.inc.is_some(),
                "{rung:?}: both take the ARGUMENT's stator arm");
        assert_eq!(shipped.fuel.inner.phi_ref.get(), PHI_REF_INCIDENCE, "{rung:?}: phi_ref carried");
        let want_air = if rung == Rung::R80 { Some(SM_AIR_PROBE) } else { None };
        assert_eq!(shipped.fuel.inner.sm_air.get(), want_air,
                   "{rung:?}: the shipped carriage of `sm_air` — the non-vacuity half");
    }
}

/// **THE OBSERVERS AND THE HELPER ARE PURE** — every reading bit-identical to the shipped row's,
/// AND rung 79's six counters identical at every seat, before any count is believed.
#[test]
fn the_observers_and_the_helper_are_pure() {
    for (rung, row) in [(Rung::R79, Row::MacroNone), (Rung::R79, Row::Count),
                        (Rung::R80, Row::MacroNone), (Rung::R80, Row::Count)] {
        let n = seats_of(rung).len();
        assert_eq!(row_verdicts(rung, row), vec!["same"; n],
                   "{rung:?} {row:?} is NOT a pure observer");
        assert_eq!(coords_of(rung, row), coords_of(rung, Row::Shipped),
                   "{rung:?} {row:?}: and rung 79's counters moved exactly as often");
    }
}

/// The baselines READ at every seat, and the `demand_gains` seat is on the pre-flight's rig: 128
/// incidence-branch entries at wall 0.80 on BOTH rungs, every one short-circuited
/// (`fb_inc = calls_inc`). `coord_march` reproduces step 3's 1 366 / 1 363.
#[test]
fn every_baseline_seat_reads_on_the_measured_rig() {
    for rung in [Rung::R79, Rung::R80] {
        for (s, c) in seats_of(rung).iter().zip(cells(rung, Row::Shipped)) {
            assert!(matches!(c.seen, Seen::Read(_)), "{rung:?} baseline {s}: {:?}", c.seen);
        }
        let dg = cells(rung, Row::Shipped)[4].coord;
        println!("{rung:?} shipped demand_gains counters: {dg:?}");
        assert_eq!(dg.hits, 128, "{rung:?}: § 5.33 (i)'s 128 — else the seat is off the rig");
        assert!(dg.calls_inc > 0 && dg.fb_inc == dg.calls_inc,
                "{rung:?}: every incidence call short-circuited (mask 2): {dg:?}");
        let cm = cells(rung, Row::Shipped)[2].coord;
        assert_eq!((cm.hits, cm.fb_inc), (1366, 1363), "{rung:?}: step 3's march counters");
    }
}

fn print_row(rung: Rung, row: Row) {
    println!("{rung:?} {row:?}: {:?}\n    hits {:?}", row_verdicts(rung, row), hits_of(rung, row));
}

/// **RUNG 79's FOUR ROWS — every verdict landed as predicted before the first run.**
///
/// | row | scan | census | march | forced | demand_gains |
/// |---|---|---|---|---|---|
/// | `at_lever → R78` | same | same | **DIFF** | same | same, **hits 128 → 0** |
/// | `shared_rig → R78` | same | same | same | same | same, hits 128 |
/// | `cap_fuel → R78` | same | same | **DIFF** | same | same, **hits 128 → 0** |
/// | `with_coord → R78 (74's body)` | same | same | **DIFF** | same | same, **hits 128 → 0** |
///
/// * **The three `coord_march` DIFFs are ONE STRING** — a `phi` march compared with a `phi` march,
///   an empty log, all six counters zero. To the only value seat that can see any of them, the
///   three deletions are one deletion (AH's `at_lever`/`cap_fuel` shape, at three cells).
/// * **At `demand_gains` the three are VALUE-silent and COUNTER-loud** — P2b's masks hold on every
///   row: each deletion removes § 5.33 (i)'s branch entirely (hits 0) and no value moves. The
///   `with_coord` row is, in effect, the Python repair § 5.33 (i)(c) declined to make; at this
///   seat it moves no value (the pre-flight's nine walls say the same in Python).
/// * **`shared_rig → R78` is REDUNDANCY**: the injected `at_lever` had already carried `phi_ref`, so
///   the row matches the shipped one on values AND on all six counters at every seat.
#[test]
fn the_rung_79_rows() {
    for row in [Row::AtLever, Row::SharedRig, Row::CapFuel, Row::WithCoord] {
        print_row(Rung::R79, row);
    }
    let want = |m: &'static str| vec!["same", "same", m, "same", "same"];
    for (row, w) in [(Row::AtLever, want("DIFF")), (Row::SharedRig, want("same")),
                     (Row::CapFuel, want("DIFF")), (Row::WithCoord, want("DIFF"))] {
        assert_eq!(row_verdicts(Rung::R79, row), w, "R79 {row:?}");
    }
    let march = |row| cells(Rung::R79, row)[2].seen.clone();
    assert_eq!(march(Row::AtLever), march(Row::CapFuel), "one deletion, as `coord_march` sees it");
    assert_eq!(march(Row::AtLever), march(Row::WithCoord), "one deletion, as `coord_march` sees it");

    let shipped = coords_of(Rung::R79, Row::Shipped);
    assert_eq!(coords_of(Rung::R79, Row::SharedRig), shipped, "the redundant row, counters too");
    for row in [Row::AtLever, Row::CapFuel, Row::WithCoord] {
        let got = coords_of(Rung::R79, row);
        for i in [0, 1, 3] {
            assert_eq!(got[i], shipped[i], "{row:?} seat {i}: scan/census/forced call the plant \
                       with an EXPLICIT coordinate, table-free, so their counters cannot move");
        }
        for i in [2, 4] {
            assert_eq!(got[i], CoordCounters::default(),
                       "{row:?} seat {i}: the deletion removes rung 79's branch ENTIRELY");
        }
    }
}

/// **RUNG 80's TWO ROWS — both landed as predicted.**
///
/// * **`at_lever → R79` is silent at all NINE seats, on values and on all six counters, and it is
///   REDUNDANCY, not unreachability**: the `Count` row enters the cell at every seat, and
///   [`the_rung_80_at_lever_row_keeps_the_split_on_a_rung_79_rig`] reads the mechanism off the rig
///   — `r80_shared_rig` re-reads `sm_air` off the CORE and splits the parent's machine, and no
///   reader dispatches that rig's `shared_rig` again. So rung 80's `at_lever` re-aim is invisible
///   to every value-bearing seat and caught only structurally — pointer identity and the
///   sibling-knob readback (the closing mutation, plan § 5.33.7 (e)).
/// * **`shared_rig → R79`**: rung 79's four readers and `demand_gains` are exact dispatch
///   (`sm_air = None` on their rigs); the four split readers DIFF — the rig marches the SHARED
///   wall while the reader reports a split one — and none refuses, since `engine.py:21486`'s guard lives in
///   the deleted cell. Counters identical to shipped everywhere: rung 80 marches in `phi`.
#[test]
fn the_rung_80_rows() {
    for row in [Row::AtLever, Row::SharedRig] {
        print_row(Rung::R80, row);
    }
    assert_eq!(row_verdicts(Rung::R80, Row::AtLever), vec!["same"; 9], "R80 AtLever");
    assert_eq!(row_verdicts(Rung::R80, Row::SharedRig),
               vec!["same", "same", "same", "same", "same", "DIFF", "DIFF", "DIFF", "DIFF"],
               "R80 SharedRig");
    let shipped = coords_of(Rung::R80, Row::Shipped);
    for row in [Row::AtLever, Row::SharedRig] {
        assert_eq!(coords_of(Rung::R80, row), shipped, "R80 {row:?}: counters");
    }
    assert_eq!(&shipped[..5], &coords_of(Rung::R79, Row::Shipped)[..],
               "the five shared seats count identically on a rung-80 machine");
    assert!(shipped[5..].iter().all(|c| *c == CoordCounters::default()),
            "step 4: rung 80's readers never enter rung 79's branch");
}

/// **THE TALLIES — PINNED FROM THE RUN**, columns `[scan, census, march, forced, demand_gains,
/// liveness, arrest, saturation, gains]`.
///
/// * **`with_coord` reads 8 and 32, not the 4 and 16 predicted**: the prediction counted SCOPES,
///   and every `CoordScope` dispatches the cell TWICE — on set AND on drop, the restore going back
///   THROUGH the table. That second dispatch is exactly the path item L's survivor bypassed (step
///   6). `coord_march`'s four scopes → 8; `demand_gains`' sixteen interior cells → 32.
/// * **`cap_fuel` at `demand_gains` is 128 = the incidence hits**: every entry there took rung 79's
///   branch. `coord_march`'s 2 732 = two marches × 1 366. Zero at every CLIP-only seat — the three
///   gauge-point readers and `split_gains` in `clip` — AH's *only the DEMAND march calls it*.
/// * **`split_saturation` enters `cap_fuel` 10 931 times, not 8 × 1 366 = 10 928.** Not a longer
///   march (a point is 4 entries): a throwaway probe measured all eight at 341 points, with the
///   three highest walls (0.855, 0.86, 0.88) at 1 367 entries each. One extra entry per march at
///   the top walls, from a site not traced.
/// * `at_lever` and `shared_rig` are one column: every rig is built by `shared_rig` through
///   `at_lever`, once per march.
#[test]
fn the_count_rows() {
    for rung in [Rung::R79, Rung::R80] {
        for (i, name) in ["at_lever", "shared_rig", "cap_fuel", "with_coord"].iter().enumerate() {
            println!("{rung:?} COUNT {name}: {:?}", col(rung, i));
        }
    }
    assert_eq!(col(Rung::R79, 0), vec![2, 2, 4, 2, 1]);
    assert_eq!(col(Rung::R79, 1), vec![2, 2, 4, 2, 1]);
    assert_eq!(col(Rung::R79, 2), vec![0, 0, 2732, 0, 128]);
    assert_eq!(col(Rung::R79, 3), vec![0, 0, 8, 0, 32]);
    assert_eq!(col(Rung::R80, 0), vec![2, 2, 4, 2, 1, 6, 18, 8, 3]);
    assert_eq!(col(Rung::R80, 1), vec![2, 2, 4, 2, 1, 6, 18, 8, 3]);
    assert_eq!(col(Rung::R80, 2), vec![0, 0, 2732, 0, 128, 4098, 24588, 10931, 0]);
    assert_eq!(col(Rung::R80, 3), vec![0, 0, 8, 0, 32, 0, 0, 0, 0]);
    assert!(col(Rung::R80, 0).iter().all(|&n| n > 0),
            "rung 80's `at_lever` is ENTERED at every seat — so its all-`same` row is redundancy");
}

/// **THE THIRD SILENCE, MEASURED.** Under rung 79's `at_lever → R78` the rig `gauge_points` builds
/// for `coord_scan` / `coord_census` / `coord_forced` IS a different machine — rung 78's `cap_fuel`
/// and `with_coord` — and those readers still read `same`, because they never dispatch either
/// cell of the rig (the `Count` row's zeros at those seats).
#[test]
fn the_rung_79_at_lever_rig_differs_where_no_reader_dispatches_it() {
    for (row, want) in [(Row::Shipped, &R79_TRIPLE), (Row::AtLever, &R78_TRIPLE)] {
        let m = build(Rung::R79, row);
        let (rig, _, _, pts) = gauge_points(&m, &flight(), LO, HI, TT4_MAX, C_MARGIN, TAUS, R,
                                            SETTLE, DS, V_MAX, false, C_PHI, C_EVERY);
        assert!(!pts.is_empty());
        assert!(fn_addr_eq(rig.triple_hooks().cap_fuel, want.cap_fuel), "{row:?}: rig cap_fuel");
        assert!(fn_addr_eq(rig.triple_hooks().with_coord, want.with_coord),
                "{row:?}: rig with_coord");
    }
    for i in [2, 3] {
        let c = col(Rung::R79, i);
        assert_eq!((c[0], c[1], c[3]), (0, 0, 0),
                   "the scan/census/forced seats never dispatch the rig's cell {i}: {c:?}");
    }
}

/// **THE MECHANISM OF RUNG 80's `at_lever` ROW.** Under `at_lever → R79` the rig `split_march`
/// builds is a RUNG-79 machine (its `shared_rig` and `at_lever` are rung 79's), and it still carries
/// the requested `sm_air` with the airflow wall above the fuel wall — `r80_shared_rig` re-reads the
/// knob off the CORE and splits the parent's machine. On the shipped row the rig is rung 80's.
#[test]
fn the_rung_80_at_lever_row_keeps_the_split_on_a_rung_79_rig() {
    let pa = 0.77;
    for (row, want_rig, want_at) in [(Row::Shipped, R80_TRIPLE.shared_rig, R80.at_lever),
                                     (Row::AtLever, R79_TRIPLE.shared_rig, R79.at_lever)] {
        let m = build(Rung::R80, row);
        let ps = m.arming().map_lp_design.phi_surge;
        let (rig, surge, _, traj) = split_march(&m, &flight(), LO, HI, TT4_MAX, PHI_FUEL,
                                                Some(pa), "demand", TAUS, R, SETTLE, DS, V_MAX,
                                                false);
        assert!(!traj.is_empty());
        assert_eq!(rig.fuel.inner.sm_air.get(), Some(pa / ps - 1.0), "{row:?}: the KNOB is carried");
        let w = walls_of(&rig, surge.as_ref());
        assert!(w.phi_air.unwrap() > w.phi_lim.unwrap(), "{row:?}: and the split reached the plant");
        assert!(fn_addr_eq(rig.triple_hooks().shared_rig, want_rig), "{row:?}: rig shared_rig");
        assert!(fn_addr_eq(rig.fuel.inner.lever_hooks.at_lever, want_at), "{row:?}: rig at_lever");
        assert_eq!(m.fuel.inner.sm_air.get(), None, "{row:?}: the caller's knob restored");
        assert_eq!(rig.fuel.inner.phi_ref.get(), PHI_REF_PHI);
    }
}
