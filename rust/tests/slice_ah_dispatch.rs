//! SLICE AH step 7 — **THE DISPATCH GATES, AND P2: A CELL THAT CARRIES NOTHING NEW IS VISIBLE TO
//! POINTER IDENTITY AND TO NOTHING ELSE.**
//!
//! Four swaps — rung 77's `at_lever` (from rung 76's) and rung 78's `at_lever`, `shared_rig` and
//! `cap_fuel` (all from rung 77's) — each pointed back at the parent it was re-aimed FROM, and
//! scored against **eight seats**: the eight readers this slice publishes, four per rung, each
//! run on a machine of either rung. `slice_ah_cells.rs` gates the pointers,
//! `slice_ah_oracle.rs` 29 288 values against two interpreters, and `rung77.rs` / `rung78.rs`
//! the suites. All of them read a machine one of this slice's builders assembled, so none can
//! say which FUNCTION sat in a slot while a reader ran. That is this file's subject.
//!
//! # THE FINGERPRINT
//!
//! A seat's reading is `format!("{x:?}")` over the reader's own `Debug`: Rust's `{:?}` for `f64`
//! is shortest-round-trip and so INJECTIVE on bit patterns, `-0.0` included. AF's and AG's
//! precedent. Beside it every seat records TWO counts, because a verdict is one bit
//! (§ 5.31.7 (a)): how many times the observed CELL was entered, and how many times rung 78's
//! GAUGED branch ran (`GAUGE_HITS`, reset per seat). The second exists because a call count on
//! `cap_fuel` cannot separate its two branches — every march enters the identity branch, so the
//! count is nonzero everywhere whether or not the swapped branch is reachable.
//!
//! # THE MATRIX IS COMPUTED ONCE, BEHIND A `OnceLock`
//!
//! `GAUGE_HITS` is process-global and cargo runs tests on parallel threads, so every seat run
//! happens inside ONE computation and each gate asserts on its slice of it. It also means each
//! observer runs once, not once for its purity proof and again for its tally, and the shipped
//! step `ds` is affordable — no coarsening, so AG § 5.31.7 (d)'s RK4-floor trap does not arise.
//!
//! # THE REBUILD HELPER IS PROVED AGAINST THE SHIPPED CONSTRUCTOR BEFORE ANY ROW IS READ
//!
//! A table injection must survive a reader's rebuild, so the injected machines carry an
//! `at_lever` that rebuilds with the injected tables (AG's `injection!`). That puts every
//! macro-built row's `at_lever` in THIS FILE's hands, and at rung 78 the knob that matters,
//! `gauge_k`, is one AG's six-knob helper never carried — while the baseline cannot reveal the gap,
//! because `r78_shared_rig` re-sets `gauge_k` on the sibling anyway. So
//! [`the_rebuild_helper_is_the_shipped_constructor`] checks all seven knobs at a NON-identity
//! gauge against the shipped `r78_at_lever` (six against `r77_at_lever`), and the matrix carries a
//! `MacroNone` row per rung that must read `same` at every seat.

use std::cell::Cell;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::fn_addr_eq;
use std::sync::{Mutex, MutexGuard, OnceLock};

use turbojet::bleed_transient::{LeverArm, LeverArming, LeverHooks};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelTransientCore, FuelTransientHooks,
};
use turbojet::gas::{Abort, Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::reference_split::StatorIncidenceLimiter;
use turbojet::residual_gauge::{
    build_residual_gauge_cascade, gauge_counters, gauge_march, gauge_scan, gauge_vs_device,
    reset_gauge_counters, root_census, GaugeRestored, DEVICE_DQ, DEVICE_EVERY, DEVICE_SPREAD,
    GAUGE_MARCH_MULTS, GAUGE_SCAN_DQ, GAUGE_SCAN_MULTS, R78, R78_FUEL, R78_STATOR, R78_TRIPLE,
    R78_TWO, ROOT_CENSUS_MULTS, ROOT_CENSUS_N, ROOT_COUNT_HI, ROOT_COUNT_LO,
};
use turbojet::sensed_cap::{accel_for, cap_march, R76, R76_TRIPLE};
use turbojet::shared_actuator::SharedRigArm;
use turbojet::stator_transient::{
    ScheduledStatorCore, ScheduledStatorTransient, StatorTransientHooks,
};
use turbojet::stiffness_ledger::{
    build_stiffness_ledger_cascade, leg_slopes, set_point_gains, singular_limit,
    stiffness_ledger, R77, R77_FUEL, R77_STATOR, R77_TRIPLE, R77_TWO,
};
use turbojet::three_loop::{StatorLimiter, TripleHooks};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::TwoSpoolTransientHooks;

// ============================================================================ the grid
//
// `slice_ah_oracle.rs`'s, which is both suites'.

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
const PHI_JAC: f64 = 0.80;
const MARGIN: f64 = 0.10;
const EVERY: usize = 8;
const DQ: f64 = 1e-5;
const SPREAD: f64 = 0.10;
const LEDGER_EVERY: usize = 16;

/// **THE ONE NARROWED SEAT, AND WHICH CELL IT KEEPS.** `stiffness_ledger`'s 24-cell default runs
/// here as ONE cell — declared, not silent. The cell is `margin = 0.40`, where § 4 found the accel
/// leg DORMANT and the raw order inverting: a cell at the suite's `margin = 0.10` would re-read
/// what `leg_slopes` / `set_point_gains` already read at `every = 16`, and its verdict would not be
/// independent evidence.
const SL_PHI_LIMS: [f64; 1] = [0.80];
const SL_MARGINS: [f64; 1] = [0.40];
const SL_TT4_MAXES: [f64; 1] = [1200.0];
const SL_ARMS: [bool; 1] = [false];

/// A NON-identity gauge for the knob-carriage proofs — any value but `1.0` separates "carried"
/// from "left at the default".
const K_PROBE: f64 = 2.5;

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

fn arm() -> LeverArm {
    let sm = PHI_JAC / FLOOR - 1.0;
    LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, sm, Some(TAU))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S))),
        ..Default::default()
    }
}

/// An incidence-arm `LeverArm` — used ONLY by the knob-carriage proof, so a sibling built from it
/// differs from the receiver in the stator arm and a helper that copied the ARM instead of
/// taking the argument would show.
fn inc_arm() -> LeverArm {
    let sm = PHI_JAC / FLOOR - 1.0;
    LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, sm, Some(TAU))),
        stator_inc: Some(StatorIncidenceLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S))),
        ..Default::default()
    }
}

/// The five knobs both suites' fixtures write by PLAIN ASSIGNMENT.
fn set_rig_knobs(m: &ScheduledStatorCore) {
    m.fuel.inner.lag_coord.set("demand");
    m.fuel.inner.ref_law.set("sched");
    m.fuel.inner.windup_law.set("none");
    m.fuel.inner.tau_t.set(None);
    m.fuel.inner.cap_law.set("solve");
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
    static N_AT77: Cell<usize> = const { Cell::new(0) };
    static N_RIG78: Cell<usize> = const { Cell::new(0) };
    static N_CAP78: Cell<usize> = const { Cell::new(0) };
}

fn reset_counts() {
    N_AT77.with(|n| n.set(0));
    N_RIG78.with(|n| n.set(0));
    N_CAP78.with(|n| n.set(0));
}

/// Counts, then delegates to the SHIPPED `r77_at_lever` through the `const` table. The sibling it
/// returns is the shipped one, so the count is *entered from the receiver*, which is the question.
fn counting_at_lever_77(core: &ScheduledStatorCore, a: &LeverArm) -> ScheduledStatorCore {
    N_AT77.with(|n| n.set(n.get() + 1));
    (R77.at_lever)(core, a)
}

fn counting_shared_rig_78(
    core: &ScheduledStatorCore, a: &SharedRigArm,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>) {
    N_RIG78.with(|n| n.set(n.get() + 1));
    (R78_TRIPLE.shared_rig)(core, a)
}

#[allow(clippy::too_many_arguments)]
fn counting_cap_fuel_78(
    ft: &FuelTransientCore, flight: &FlightCondition, a: f64, h: f64, mf_sched: f64,
    accel: Option<&AccelSchedule>, surge: Option<&Floor>, mf_app: Option<f64>,
) -> Result<f64, Abort> {
    N_CAP78.with(|n| n.set(n.get() + 1));
    (R78_TRIPLE.cap_fuel)(ft, flight, a, h, mf_sched, accel, surge, mf_app)
}

// -------------------------------------------------------------- the injected tables

static T78_SHARED_RIG: TripleHooks =
    TripleHooks { shared_rig: R77_TRIPLE.shared_rig, ..R78_TRIPLE };
static T78_CAP_FUEL: TripleHooks = TripleHooks { cap_fuel: R77_TRIPLE.cap_fuel, ..R78_TRIPLE };
static T78_COUNT_RIG: TripleHooks =
    TripleHooks { shared_rig: counting_shared_rig_78, ..R78_TRIPLE };
static T78_COUNT_CAP: TripleHooks = TripleHooks { cap_fuel: counting_cap_fuel_78, ..R78_TRIPLE };

/// The two `at_lever` rows — the PARENT's own shipped cell, a pointer and nothing this file built.
static L77_AT_LEVER: LeverHooks = LeverHooks { at_lever: R76.at_lever, ..R77 };
static L78_AT_LEVER: LeverHooks = LeverHooks { at_lever: R77.at_lever, ..R78 };
/// The observer on rung 77's one cell.
static L77_COUNT_AT: LeverHooks = LeverHooks { at_lever: counting_at_lever_77, ..R77 };

/// Build a machine on an arbitrary table quintuple — not through either cascade, which hardcode
/// their own tables.
#[allow(clippy::too_many_arguments)]
fn with_tables(
    core: &ScheduledStatorCore, a: &LeverArm, lever: &'static LeverHooks,
    fuel: &'static FuelTransientHooks, triple: &'static TripleHooks,
    two: &'static TwoSpoolTransientHooks, stator: &'static StatorTransientHooks,
) -> ScheduledStatorCore {
    full_of(ScheduledStatorTransient::with_ref_tables(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(),
        a.stator, two, stator, fuel, lever,
        LeverArming { bleed: a.bleed, sched: a.bleed_sched, lim: a.bleed_lim },
        triple, a.stator_lim, a.stator_inc))
}

/// An injection's `at_lever`: it rebuilds with **that injection's own tables** and carries the
/// knobs the shipped constructor carries — SIX at rung 77 (`r77_at_lever`), SEVEN at rung 78
/// (`r78_at_lever`, `gauge_k` the seventh). [`the_rebuild_helper_is_the_shipped_constructor`] is
/// the proof, and it runs before any row is read.
macro_rules! injection {
    ($lever:ident, $rebuild:ident, $base:expr, $fuel:expr, $triple:expr, $two:expr, $stator:expr,
     $gauge:expr) => {
        static $lever: LeverHooks = LeverHooks { at_lever: $rebuild, ..$base };
        fn $rebuild(core: &ScheduledStatorCore, a: &LeverArm) -> ScheduledStatorCore {
            let m = with_tables(core, a, &$lever, $fuel, $triple, $two, $stator);
            m.fuel.inner.ref_law.set(core.fuel.inner.ref_law.get());
            m.fuel.inner.lag_coord.set(core.fuel.inner.lag_coord.get());
            m.fuel.inner.windup_law.set(core.fuel.inner.windup_law.get());
            m.fuel.inner.tau_t.set(core.fuel.inner.tau_t.get());
            m.fuel.inner.ic_cap.set(core.fuel.inner.ic_cap.get());
            m.fuel.inner.cap_law.set(core.fuel.inner.cap_law.get());
            if $gauge {
                m.fuel.inner.gauge_k.set(core.fuel.inner.gauge_k.get());
            }
            m
        }
    };
}

injection!(L77_NONE, r77_none, R77, &R77_FUEL, &R77_TRIPLE, &R77_TWO, &R77_STATOR, false);
injection!(L78_NONE, r78_none, R78, &R78_FUEL, &R78_TRIPLE, &R78_TWO, &R78_STATOR, true);
injection!(L78_RIG, r78_rig, R78, &R78_FUEL, &T78_SHARED_RIG, &R78_TWO, &R78_STATOR, true);
injection!(L78_CAP, r78_cap, R78, &R78_FUEL, &T78_CAP_FUEL, &R78_TWO, &R78_STATOR, true);
injection!(L78_CRIG, r78_crig, R78, &R78_FUEL, &T78_COUNT_RIG, &R78_TWO, &R78_STATOR, true);
injection!(L78_CCAP, r78_ccap, R78, &R78_FUEL, &T78_COUNT_CAP, &R78_TWO, &R78_STATOR, true);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Rung { R77, R78 }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Row {
    /// The shipped cascade, untouched — the baseline.
    Shipped,
    /// The shipped tables through THIS FILE's rebuild helper — must be `same` everywhere.
    MacroNone,
    AtLever,
    /// Rung 78 only.
    SharedRig,
    CapFuel,
    /// The observers.
    CountAtLever,
    CountSharedRig,
    CountCapFuel,
}

const ROWS_77: [Row; 4] = [Row::Shipped, Row::MacroNone, Row::AtLever, Row::CountAtLever];
const ROWS_78: [Row; 7] = [Row::Shipped, Row::MacroNone, Row::AtLever, Row::SharedRig,
                           Row::CapFuel, Row::CountSharedRig, Row::CountCapFuel];

fn shipped_triple(rung: Rung) -> &'static TripleHooks {
    match rung { Rung::R77 => &R77_TRIPLE, Rung::R78 => &R78_TRIPLE }
}

fn shipped_lever(rung: Rung) -> &'static LeverHooks {
    match rung { Rung::R77 => &R77, Rung::R78 => &R78 }
}

/// `None` for the shipped row, which is built by the cascade itself.
fn tables_of(rung: Rung, row: Row) -> Option<(&'static LeverHooks, &'static TripleHooks)> {
    match (rung, row) {
        (_, Row::Shipped) => None,
        (Rung::R77, Row::MacroNone) => Some((&L77_NONE, &R77_TRIPLE)),
        (Rung::R77, Row::AtLever) => Some((&L77_AT_LEVER, &R77_TRIPLE)),
        (Rung::R77, Row::CountAtLever) => Some((&L77_COUNT_AT, &R77_TRIPLE)),
        (Rung::R78, Row::MacroNone) => Some((&L78_NONE, &R78_TRIPLE)),
        (Rung::R78, Row::AtLever) => Some((&L78_AT_LEVER, &R78_TRIPLE)),
        (Rung::R78, Row::SharedRig) => Some((&L78_RIG, &T78_SHARED_RIG)),
        (Rung::R78, Row::CapFuel) => Some((&L78_CAP, &T78_CAP_FUEL)),
        (Rung::R78, Row::CountSharedRig) => Some((&L78_CRIG, &T78_COUNT_RIG)),
        (Rung::R78, Row::CountCapFuel) => Some((&L78_CCAP, &T78_COUNT_CAP)),
        (r, w) => panic!("{w:?} is not a row at {r:?} — ROWS_77 / ROWS_78 are the only source"),
    }
}

fn cascade(rung: Rung, a: &LeverArm) -> ScheduledStatorCore {
    full_of(match rung {
        Rung::R77 => build_stiffness_ledger_cascade(
            design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, a),
        Rung::R78 => build_residual_gauge_cascade(
            design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, a),
    })
}

/// The injected machine. **The knobs are written by plain assignment afterwards, as both suites'
/// fixtures do**, so a machine reaching a reader is in the oracle's `rig()` state.
fn build(rung: Rung, row: Row) -> ScheduledStatorCore {
    let a = arm();
    let base = cascade(rung, &a);
    let m = match tables_of(rung, row) {
        None => base,
        Some((lever, triple)) => {
            let (fuel, two, stator) = match rung {
                Rung::R77 => (&R77_FUEL, &R77_TWO, &R77_STATOR),
                Rung::R78 => (&R78_FUEL, &R78_TWO, &R78_STATOR),
            };
            with_tables(&base, &a, lever, fuel, triple, two, stator)
        }
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
        Row::SharedRig | Row::CountSharedRig => vec!["shared_rig"],
        Row::CapFuel | Row::CountCapFuel => vec!["cap_fuel"],
        _ => vec![],
    }
}

/// **THE INSTALL PROOF.** An all-silent row's only evidence that the injection took is this.
///
/// **At the two `at_lever` rows the proof is the LEVER POINTER, in BOTH directions** — and not
/// AG's *the sibling's triple table is the parent's*, which at rung 77 is VACUOUS: `R77_TRIPLE` is
/// an alias of `R76_TRIPLE`, so that comparison holds whether or not the injection took. Step 6's
/// lesson, one step on: the negative half (`!= this rung's own cell`) is what the defect cannot
/// satisfy.
fn assert_installed(m: &ScheduledStatorCore, rung: Rung, row: Row) {
    assert_eq!(triple_diff(m.triple_hooks(), shipped_triple(rung)), slot_of(row),
               "{rung:?} {row:?}: the machine carries this row's slot and no other");
    let sib = m.at_lever(&arm());
    let (parent_at, own_at) = match rung {
        Rung::R77 => (R76.at_lever, R77.at_lever),
        Rung::R78 => (R77.at_lever, R78.at_lever),
    };
    if row == Row::AtLever {
        let at = sib.fuel.inner.lever_hooks.at_lever;
        assert!(fn_addr_eq(at, parent_at),
                "{rung:?}: the parent's constructor builds a PARENT machine — the whole injection");
        assert!(!fn_addr_eq(at, own_at),
                "{rung:?}: and NOT this rung's — the half a re-aimed `R7x.at_lever` cannot fake");
        return;
    }
    assert_eq!(triple_diff(sib.triple_hooks(), shipped_triple(rung)), slot_of(row),
               "{rung:?} {row:?}: AND THE SIBLING DOES TOO — otherwise the first rebuild inside \
                any reader launders the injection away");
    assert!(!fn_addr_eq(sib.fuel.inner.lever_hooks.at_lever, parent_at),
            "{rung:?} {row:?}: the sibling is not a parent machine");
}

// =============================================================================================
// 2 — THE SEATS, AND THE ONE MATRIX
// =============================================================================================

/// The eight readers this slice publishes — rung 77's four, then rung 78's four.
const SEATS: [&str; 8] = ["leg_slopes", "set_point_gains", "singular_limit", "stiffness_ledger",
                          "gauge_scan", "root_census", "gauge_vs_device", "gauge_march"];

#[derive(Clone, PartialEq, Eq, Debug)]
enum Seen { Read(String), Broke(String) }

/// One seat on one machine: the reading, the cell count, the gauged-branch count.
#[derive(Clone, Debug)]
struct Cellrun { seen: Seen, count: usize, gauged: u64 }

fn fingerprint<T: std::fmt::Debug>(x: &T) -> String { format!("{x:?}") }

fn run_seat(name: &str, m: &ScheduledStatorCore) -> Seen {
    quiet_hook();
    QUIET.with(|q| q.set(true));
    let f = flight();
    let out = catch_unwind(AssertUnwindSafe(|| match name {
        "leg_slopes" => fingerprint(&leg_slopes(
            m, &f, LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, false, R, SETTLE, DS, V_MAX, EVERY)),
        "set_point_gains" => fingerprint(&set_point_gains(
            m, &f, LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, false, R, SETTLE, DS, V_MAX, DQ,
            EVERY)),
        "singular_limit" => fingerprint(&singular_limit(
            m, &f, LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, false, R, SETTLE, DS, V_MAX, SPREAD,
            EVERY)),
        "stiffness_ledger" => fingerprint(&stiffness_ledger(
            m, &f, LO, HI, &SL_PHI_LIMS, &SL_MARGINS, &SL_TT4_MAXES, &SL_ARMS, TAUS, R, SETTLE,
            DS, V_MAX, LEDGER_EVERY)),
        "gauge_scan" => fingerprint(&gauge_scan(
            m, &f, LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, false, R, SETTLE, DS, V_MAX,
            GAUGE_SCAN_DQ, EVERY, &GAUGE_SCAN_MULTS)),
        "root_census" => fingerprint(&root_census(
            m, &f, LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, false, R, SETTLE, DS, V_MAX, EVERY,
            &ROOT_CENSUS_MULTS, ROOT_COUNT_LO, ROOT_COUNT_HI, ROOT_CENSUS_N)),
        "gauge_vs_device" => fingerprint(&gauge_vs_device(
            m, &f, LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, false, R, SETTLE, DS, V_MAX,
            DEVICE_EVERY, DEVICE_DQ, DEVICE_SPREAD)),
        "gauge_march" => fingerprint(&gauge_march(
            m, &f, LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, false, R, SETTLE, DS, V_MAX,
            &GAUGE_MARCH_MULTS)),
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

fn count_of(row: Row) -> usize {
    match row {
        Row::CountAtLever => N_AT77.with(|n| n.get()),
        Row::CountSharedRig => N_RIG78.with(|n| n.get()),
        Row::CountCapFuel => N_CAP78.with(|n| n.get()),
        _ => 0,
    }
}

type Matrix = Vec<((Rung, Row), Vec<Cellrun>)>;

/// **THE `OnceLock` WAS NOT ENOUGH, AND THE FIRST RUN SAID SO IN A NUMBER.** The matrix is computed
/// once, but ANOTHER gate in this binary —
/// [`the_rung_78_at_lever_row_loses_the_table_not_the_knob`] — marches a GAUGED machine, and it ran
/// on its own thread while the matrix was being built. `GAUGE_HITS` is process-global, so its hits
/// landed in whichever seat was running: the RUNG-77 baseline read **838** gauged hits at
/// `leg_slopes` and **528** at `set_point_gains`, on a machine that has no gauged branch at all.
/// **838 + 528 = 1 366 = exactly one gauged march** (4 RK4 stages × 341 points + 2). So every gate
/// that touches the counter takes this lock; the matrix holds it for its whole computation.
static GAUGED: Mutex<()> = Mutex::new(());

fn gauged() -> MutexGuard<'static, ()> { GAUGED.lock().unwrap_or_else(|e| e.into_inner()) }

/// Every row at every seat, ONCE. `GAUGE_HITS` is reset before each seat and read after it;
/// `gauge_march` resets it internally too, so its reading is its LAST gauge's march.
fn matrix() -> &'static Matrix {
    static M: OnceLock<Matrix> = OnceLock::new();
    M.get_or_init(|| {
        let _g = gauged();
        let mut out = Vec::new();
        for (rung, rows) in [(Rung::R77, &ROWS_77[..]), (Rung::R78, &ROWS_78[..])] {
            for &row in rows {
                let m = build(rung, row);
                assert_installed(&m, rung, row);
                let cells: Vec<Cellrun> = SEATS.iter().map(|s| {
                    reset_counts();
                    reset_gauge_counters();
                    let seen = run_seat(s, &m);
                    let gauged = gauge_counters().0;
                    Cellrun { seen, count: count_of(row), gauged }
                }).collect();
                out.push(((rung, row), cells));
            }
        }
        out
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

// =============================================================================================
// 3 — THE GATES
// =============================================================================================

/// **EACH INJECTION MOVES EXACTLY ITS OWN SLOT, AND THE TWO `at_lever` ROWS ARE THE PARENT'S OWN
/// CELL** — in both directions.
#[test]
fn each_injection_moves_exactly_its_own_slot() {
    for (rung, rows) in [(Rung::R77, &ROWS_77[..]), (Rung::R78, &ROWS_78[..])] {
        for &row in rows {
            if let Some((_, t)) = tables_of(rung, row) {
                assert_eq!(triple_diff(t, shipped_triple(rung)), slot_of(row), "{rung:?} {row:?}");
            }
        }
    }
    assert!(fn_addr_eq(L77_AT_LEVER.at_lever, R76.at_lever));
    assert!(!fn_addr_eq(L77_AT_LEVER.at_lever, R77.at_lever));
    assert!(fn_addr_eq(L78_AT_LEVER.at_lever, R77.at_lever));
    assert!(!fn_addr_eq(L78_AT_LEVER.at_lever, R78.at_lever));
    // and the shipped tables themselves differ from their parents where this slice says they do
    assert!(!fn_addr_eq(shipped_lever(Rung::R77).at_lever, R76.at_lever));
    assert_eq!(triple_diff(&R77_TRIPLE, &R76_TRIPLE), Vec::<&str>::new(),
               "rung 77's triple is rung 76's — which is why a triple-based install proof at the \
                rung-77 `at_lever` row would be vacuous");
}

/// **THE REBUILD HELPER IS THE SHIPPED CONSTRUCTOR** — every knob, at a NON-identity gauge, from
/// a receiver whose knobs are all off their defaults, into a sibling on a DIFFERENT arm.
#[test]
fn the_rebuild_helper_is_the_shipped_constructor() {
    for rung in [Rung::R77, Rung::R78] {
        let core = cascade(rung, &arm());
        core.fuel.inner.lag_coord.set("clip");
        core.fuel.inner.ref_law.set("applied");
        core.fuel.inner.windup_law.set("track");
        core.fuel.inner.tau_t.set(Some(0.0125));
        core.fuel.inner.ic_cap.set(123);
        core.fuel.inner.cap_law.set("sensed");
        core.fuel.inner.gauge_k.set(K_PROBE);
        let shipped = (shipped_lever(rung).at_lever)(&core, &inc_arm());
        let (mine_lever, _) = tables_of(rung, Row::MacroNone).unwrap();
        let mine = (mine_lever.at_lever)(&core, &inc_arm());
        let knobs = |m: &ScheduledStatorCore| (
            m.fuel.inner.lag_coord.get(), m.fuel.inner.ref_law.get(),
            m.fuel.inner.windup_law.get(), m.fuel.inner.tau_t.get(), m.fuel.inner.ic_cap.get(),
            m.fuel.inner.cap_law.get(), m.fuel.inner.gauge_k.get().to_bits());
        assert_eq!(knobs(&mine), knobs(&shipped), "{rung:?}: the helper carries what the shipped \
                   constructor carries, gauge_k included");
        assert_eq!(triple_diff(mine.triple_hooks(), shipped.triple_hooks()), Vec::<&str>::new());
        assert_eq!(mine.fuel.inner.stator.inc.is_some(), shipped.fuel.inner.stator.inc.is_some(),
                   "{rung:?}: both take the ARGUMENT's stator arm");
        assert!(mine.fuel.inner.stator.inc.is_some(), "{rung:?}: and that arm is the incidence one");
        // THE NON-VACUITY HALF: at rung 78 the gauge IS carried, at rung 77 it is NOT — so a
        // helper that always carried it, or never did, fails one of the two.
        let want = if rung == Rung::R78 { K_PROBE } else { 1.0 };
        assert_eq!(shipped.fuel.inner.gauge_k.get(), want, "{rung:?}: the shipped carriage");
    }
}

/// **THE OBSERVERS AND THE HELPER ARE PURE** — every reading bit-identical to the shipped row's,
/// asserted before any count is believed.
#[test]
fn the_observers_and_the_helper_are_pure() {
    for (rung, row) in [(Rung::R77, Row::MacroNone), (Rung::R77, Row::CountAtLever),
                        (Rung::R78, Row::MacroNone), (Rung::R78, Row::CountSharedRig),
                        (Rung::R78, Row::CountCapFuel)] {
        assert_eq!(row_verdicts(rung, row), vec!["same"; 8],
                   "{rung:?} {row:?} is NOT a pure observer — every count below would be taken \
                    from a different plant");
        let g: Vec<u64> = cells(rung, row).iter().map(|c| c.gauged).collect();
        let g0: Vec<u64> = cells(rung, Row::Shipped).iter().map(|c| c.gauged).collect();
        assert_eq!(g, g0, "{rung:?} {row:?}: and the gauged branch ran exactly as often");
    }
}

/// The baselines READ at every seat, on both rungs.
#[test]
fn every_baseline_seat_reads() {
    for rung in [Rung::R77, Rung::R78] {
        for (s, c) in SEATS.iter().zip(cells(rung, Row::Shipped)) {
            assert!(matches!(c.seen, Seen::Read(_)), "{rung:?} baseline {s}: {:?}", c.seen);
        }
    }
}

/// **P2 — RUNG 77's `at_lever`, POINTED AT RUNG 76's, IS SILENT AT EVERY SEAT, AND THE SILENCE IS
/// REDUNDANCY, NOT UNREACHABILITY.**
///
/// The row is `same` × 8. The positive control is NOT AG's *this row must move a seat* — P2
/// predicts exactly that it moves none, so that control would be designed to fail here. It is the
/// COUNTER: the cell is entered at every seat, so each `same` is *ran, and made no difference* —
/// plus the install proof, which reads the lever pointer in both directions.
#[test]
fn p2_the_rung_77_at_lever_row_is_silent_and_entered_everywhere() {
    let row = row_verdicts(Rung::R77, Row::AtLever);
    println!("R77 AtLever: {row:?}");
    assert_eq!(row, vec!["same"; 8], "P2: no VALUE sees rung 77's `at_lever`");
    let counts: Vec<usize> = cells(Rung::R77, Row::CountAtLever).iter().map(|c| c.count).collect();
    println!("R77 COUNT at_lever: {counts:?}");
    // PINNED, NOT PRINTED — transcribed from the run. Every seat ENTERS the cell (each reader
    // rebuilds its rig through `at_lever` from the receiver), so each `same` above is REDUNDANCY:
    // the cell ran and made no difference, which is P2's whole content. `stiffness_ledger` is 4
    // (two readers per cell); `gauge_march` 11 (its schedules and its five marches).
    assert_eq!(counts, vec![2, 2, 2, 4, 2, 2, 2, 11],
               "the tally moved — a ZERO would turn a `same` into unreachability");
}

/// **RUNG 78's THREE ROWS.** Predicted before the first run:
///
/// * `at_lever → R77`: `gauge_march` alone moves. The sibling is a RUNG-77 machine whose
///   `cap_fuel` is rung 76's, which has no gauged branch — the KNOB is not what is lost
///   (`r78_shared_rig` re-sets `gauge_k` on it); the TABLE is. See
///   [`the_rung_78_at_lever_row_loses_the_table_not_the_knob`].
/// * `shared_rig → R77`: silent everywhere — `r78_at_lever` already carried the gauge.
/// * `cap_fuel → R77`: `gauge_march` alone moves, and identically to `at_lever`'s.
///
/// The three VERDICT rows landed as predicted. The two tallies did not — see the `cap_fuel` pin.
#[test]
fn the_rung_78_rows() {
    let want = |g: &'static str| vec!["same", "same", "same", "same", "same", "same", "same", g];
    for (row, w) in [(Row::AtLever, want("DIFF")), (Row::SharedRig, want("same")),
                     (Row::CapFuel, want("DIFF"))] {
        let got = row_verdicts(Rung::R78, row);
        println!("R78 {row:?}: {got:?}");
        assert_eq!(got, w, "R78 {row:?}");
    }
    let gm = |row| cells(Rung::R78, row)[7].seen.clone();
    assert_eq!(gm(Row::AtLever), gm(Row::CapFuel),
               "the `at_lever` and `cap_fuel` rows are ONE deletion as far as `gauge_march` can see");
    let rig: Vec<usize> = cells(Rung::R78, Row::CountSharedRig).iter().map(|c| c.count).collect();
    let cap: Vec<usize> = cells(Rung::R78, Row::CountCapFuel).iter().map(|c| c.count).collect();
    println!("R78 COUNT shared_rig: {rig:?}\nR78 COUNT cap_fuel: {cap:?}");
    assert_eq!(rig, vec![2, 2, 2, 4, 2, 2, 2, 11],
               "`shared_rig` is ENTERED at every seat, so its all-`same` row is REDUNDANCY —                 `r78_at_lever` had already carried the gauge");
    // **`cap_fuel`'s SILENCE AT SEVEN SEATS IS UNREACHABILITY, AND THE PREDICTION SAID OTHERWISE.**
    // Pre-registered as "entered at every seat". MEASURED: entered ONLY at `gauge_march`, 6 830 =
    // five demand marches x 1 366. `_cap_fuel`'s only callers are the DEMAND march's closures
    // (`engine.py:17849`, the port's `demand_coordinate.rs`), and every other seat stands on
    // `_ledger_march`, which marches CLIP. So the same verdict `same` means *ran, no difference* at
    // `shared_rig`'s seven seats and *never entered* at `cap_fuel`'s — the two readings a
    // one-bit matrix cannot separate, separated by the counter.
    assert_eq!(cap, vec![0, 0, 0, 0, 0, 0, 0, 6830],
               "`cap_fuel` is entered by the DEMAND march alone");
}

/// **THE GAUGED BRANCH, SEPARATED FROM THE CALL.** A `cap_fuel` call count is nonzero at every seat
/// because every march enters the identity branch. Rung 78's swap is the GAUGED branch, and it
/// runs at `gauge_march` alone, on a rung-78 machine alone — zero on rung 77's machine, and zero
/// at rung 78's other three readers, which solve the gauged residual DIRECTLY and never reach it.
#[test]
fn the_gauged_branch_runs_only_where_the_swap_can_bite() {
    let g = |rung, row| -> Vec<u64> { cells(rung, row).iter().map(|c| c.gauged).collect() };
    println!("gauged R78 shipped: {:?}\ngauged R77 shipped: {:?}",
             g(Rung::R78, Row::Shipped), g(Rung::R77, Row::Shipped));
    let shipped = g(Rung::R78, Row::Shipped);
    assert!(shipped[7] > 0, "the gauged branch RAN at `gauge_march`: {shipped:?}");
    assert!(shipped[..7].iter().all(|&n| n == 0), "and nowhere else: {shipped:?}");
    assert_eq!(g(Rung::R77, Row::Shipped), vec![0; 8], "a rung-77 machine has no gauged branch");
    for row in [Row::AtLever, Row::CapFuel] {
        assert_eq!(g(Rung::R78, row), vec![0; 8], "{row:?} removed the gauged branch");
    }
    assert_eq!(g(Rung::R78, Row::SharedRig), shipped, "and `shared_rig` did not");
}

/// **THE MECHANISM, READ OFF THE MARCHED MACHINE.** Under the rung-78 `at_lever → R77` injection,
/// the machine `cap_march` actually marches carries the NON-identity gauge (so the knob survived,
/// re-set by `r78_shared_rig`) and RUNG 76's `cap_fuel` (so the table did not). On the shipped
/// machine the same march carries rung 78's.
#[test]
fn the_rung_78_at_lever_row_loses_the_table_not_the_knob() {
    let _g = gauged();
    let fl = flight();
    let sm = PHI_JAC / FLOOR - 1.0;
    for (row, want_cap) in [(Row::Shipped, R78_TRIPLE.cap_fuel),
                            (Row::AtLever, R76_TRIPLE.cap_fuel)] {
        let m = build(Rung::R78, row);
        let acc = accel_for(&m, &fl, LO, HI, sm, TT4_MAX, TAUS, V_MAX, false, MARGIN);
        let _gk = GaugeRestored::set(&m.fuel.inner, K_PROBE);
        let (marched, _, _, _) = cap_march(&m, &fl, LO, HI, TT4_MAX, sm, TAUS, R, SETTLE, DS,
                                           V_MAX, false, "demand", "sched", "none", None,
                                           "solve", &acc, None);
        assert_eq!(marched.fuel.inner.gauge_k.get(), K_PROBE, "{row:?}: the KNOB is carried");
        assert!(fn_addr_eq(marched.triple_hooks().cap_fuel, want_cap),
                "{row:?}: the marched machine's `cap_fuel`");
    }
    assert!(!fn_addr_eq(R76_TRIPLE.cap_fuel, R78_TRIPLE.cap_fuel), "the control");
}
