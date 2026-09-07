//! SLICE AG step 7 — **A COUNTING POINTER ANSWERS, PER CELL, THE QUESTION EVERY DISPATCH MATRIX
//! SINCE SLICE AD HAS ANSWERED BY LOOKING AT THE REST OF THE ROW.**
//!
//! Eight injections — this slice's eight swaps, each pointed back at the parent it was re-aimed
//! FROM — scored against **seven seats**, which are slice AG's OWN readers and not slice AF's
//! rung-74 names. `slice_ag_cells.rs` gates the pointers, `slice_ag_laws.rs` the laws,
//! `slice_ag_oracle.rs` 38 100 values against two interpreters, and `rung75.rs` / `rung76.rs` the
//! two suites. **All five read a machine this rung's own builder assembled**, so not one of them
//! can say which FUNCTION POINTER sat in a slot. That is this file's subject, and § 5.31 (vi)'s
//! step 7.
//!
//! # OBLIGATION 1 — **§ 5.30.6 (v)'s BOOKING, AND IT NEEDS THREE SIDES**
//!
//! Slice AF measured `sensed_cap` **unreachable from every rung-74 reader** — `r74_cap_fuel`
//! reaches it only inside `if let Some(accel)`, and not one of that rung's seven seats arms an
//! `AccelSchedule` — and booked the closure here: *rung 76 replaces the body, and the accel arm
//! is where it lands.*
//!
//! **The booking is right and it is not sufficient**, which § 5.31 (i) measured before a line of
//! this slice was written. Arming the arm dispatches the cell at every call; whether a VALUE moves
//! is decided by the `min`-select one level down; and whether the PLANT moves is decided by which
//! leg wins that `min`, which is a property of the surge floor. Three questions, three answers,
//! and at `PHI_JAC` they are *dispatched*, *2.32 % below the solve*, and *exactly zero*.
//!
//! So § 4 is three-sided and the sides are scored at the arms they are true of:
//!
//! | side | arm | instrument |
//! |---|---|---|
//! | DISPATCHED | either | the counting pointer's tally |
//! | DISCRIMINATING | `PHI_JAC` | the returned cap against the solve |
//! | PLANT-REACHABLE | `PHI_BOTH` | the marched trajectory |
//!
//! **AND THE TRAP IS NAMED BEFORE IT IS SPRUNG.** A parent-pointer injection scored on the
//! TRAJECTORY at `PHI_JAC` reads an exact zero — the same reading a pointer dropped from the
//! table gives, and the same reading a machine that never reached the cell gives. Slice T step 1's
//! *an EXACT ZERO blinds its own gate*, one phase on: the `PHI_JAC` row is scored on the value and
//! the count, the `PHI_BOTH` row on the trajectory, and neither substitutes for the other.
//!
//! # OBLIGATION 2 — **THE SEAT MATRIX, RUN WHOLE, ON BOTH RUNGS**
//!
//! AD step 6's rule, kept: with a parent-pointer injection a *silence* is either laundering (the
//! cell ran, on a machine `at_lever` rebuilt around the shipped tables) or a path that never
//! reaches the cell, and a did-it-break instrument cannot tell those apart. Every seat therefore
//! runs on BOTH machines — a rung-76 cell that leaks into a rung-75 reader is visible, and so is a
//! rung-75 cell that only bites through a rung-76 one.
//!
//! # THE LEADING FINDING — **THE ROW WAS NEVER THE ONLY WAY TO SEPARATE THEM**
//!
//! AD step 6's rule is a workaround for an instrument that reports only *did the reading move*.
//! A pointer that COUNTS and then delegates to the shipped body reports *was this cell entered*,
//! which is the other half, and it is available at every cell for the price of one `Cell<usize>`.
//! Every counting reading in § 2 is asserted **bit-identical to the baseline** first, so the tally
//! is taken from a machine that is otherwise the shipped one — a pure observer, and a reading that
//! moved would mean the tally belonged to a different plant.
//!
//! With it, a silent row splits in two by MEASUREMENT rather than by inference:
//!
//! * count `> 0`, reading unchanged ⇒ the cell RAN and is REDUNDANT here;
//! * count `== 0` ⇒ the cell was never entered, and the row says nothing about it.
//!
//! Both `_shared_rig` carries are the first case, which `slice_ag_cells.rs`'s
//! `both_shared_rig_carries_are_no_ops_because_at_lever_already_did_them` PRE-REGISTERED as a
//! measured no-op — so their silence here is a confirmation with a number under it rather than an
//! absence. `sensed_cap` is the second case on a machine with no schedule armed and the first the
//! moment one is, which is § 5.30.6 (v)'s obligation stated as a measurement instead of as a
//! reachability argument.
//!
//! # WHAT THIS FILE'S FINGERPRINT IS, AND WHAT IT LOSES
//!
//! A seat's reading is `format!("{x:?}")` over the reader's own `Debug`. Rust's `{:?}` for `f64`
//! is shortest-round-trip and therefore INJECTIVE on bit patterns, `-0.0` included — the one
//! collapse is NaN payloads, which no reader in this slice publishes. AF's precedent, with the
//! reason written down rather than inherited.

use std::cell::Cell;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::fn_addr_eq;

use turbojet::anti_windup::{
    build_anti_windup_cascade, contraction_law, device_control, windup_bill, windup_gains,
    R75, R75_FUEL, R75_STATOR, R75_TRIPLE, R75_TWO, WINDUP_LAW_NONE,
};
use turbojet::applied_reference::REF_LAW_APPLIED;
use turbojet::bleed_transient::{LeverArm, LeverArming, LeverHooks};
use turbojet::demand_coordinate::{LAG_COORD_DEMAND, R74, R74_FUEL, R74_TRIPLE};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelPoint, FuelTransientCore, FuelTransientHooks,
    PointExtra,
};
use turbojet::gas::{Abort, Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::sensed_cap::{
    accel_for, build_sensed_cap_cascade, cap_bill, cap_gains, cap_march, solve_gain, CapScope,
    CAP_LAW_SENSED, CAP_LAW_SOLVE, R76, R76_FUEL, R76_STATOR, R76_TRIPLE, R76_TWO,
};
use turbojet::shared_actuator::{SharedRigArm, REF_LAW_DEFAULT};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::reference_split::StatorIncidenceLimiter;
use turbojet::three_loop::{StatorLimiter, TripleHooks};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------- the grid
//
// `tests/test_rung75.py` and `tests/test_rung76.py`'s module constants, spelled as
// `slice_ag_oracle.rs` spells them. Nothing here is a number this file chose except the two
// COARSENINGS, which are named as such.
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
const TAU_T: f64 = 0.05;
const MARGIN: f64 = 0.10;

/// **THE TWO ARMS THE WHOLE SLICE TURNS ON.** `PHI_JAC` is where the two cap laws march
/// bit-identically and `PHI_BOTH` is where every one of the 341 points moves — § 5.31 (i).
const PHI_JAC: f64 = 0.80;
const PHI_BOTH: f64 = 0.76;

/// The readers' own declared defaults, `slice_ag_oracle.rs`'s names and values.
const WG_REFS: [&str; 2] = [REF_LAW_APPLIED, REF_LAW_DEFAULT];
const CL_RES0: f64 = 2.898e-3;
const CL_TOL: f64 = 1e-12;
const CL_IC_CAP: usize = 400;
const DC_REFS: [&str; 2] = [REF_LAW_DEFAULT, REF_LAW_APPLIED];
const WB_REF: &str = REF_LAW_APPLIED;
const CAP_GAINS_REFS: [&str; 1] = [REF_LAW_DEFAULT];
const CAP_GAINS_LAWS: [&str; 2] = [CAP_LAW_SOLVE, CAP_LAW_SENSED];
const CAP_BILL_TAIL: f64 = 0.2;
const SOLVE_GAIN_DQ: f64 = 1e-4;

/// **THE MATRIX's COARSE GRID, AND IT WAS TYPED WRONG FIRST — SLICE AF's OWN INSTRUMENT DEFECT
/// NUMBER ONE, ONE SLICE LATER, AT THE SAME PLACE.**
///
/// The matrix asks *did this seat's reading move*, never *by how much*; the value questions are
/// § 4's and § 5's, at the shipped grids. Five injections x seven seats x two rungs is seventy
/// readings, each of them one or more full marches, so the step wants coarsening.
///
/// **The first writing typed `ds = 0.02` and the baseline REFUSED**, by name:
/// `rung-74: ds*sum(1/tau_i) = 2.400 is outside the explicit RK4 stability region`. AF § 5.30.6
/// (vi) item 1 records the identical failure — *the matrix's perturbation left the region the
/// rung declares* — and its repair was *assert admissibility before it perturbs*. That repair is
/// what the baseline assertion in [`matrix`] is; what was missing is that the STEP itself is
/// bounded, and the bound is rung 75's own:
///
/// ```text
/// ds * (1/tau_gov + 1/tau_lag + 1/tau_q + 1/tau_s + 2/tau_t) <= 2
/// ```
///
/// with `2/tau_t` — TWO, because the device adds two states — and every clock on this rig at
/// `0.05`, that is `ds * 120 <= 2`, so `ds <= 0.01667`. **`0.0125` is the coarsest eighth-step
/// under it**, 2.5x the shipped `0.005`, and `MX_TAU_TS` drops the fast clock for the same
/// reason: `tau_t = 0.0125` puts `2/tau_t` at 160 and forces `ds` back BELOW the shipped step,
/// which is a coarsening that costs more than it saves.
///
/// **What a coarsening can cost is a cell going silent that the shipped grid would have moved**,
/// which is why § 4 and § 5 run at `DS` and the full lists, and why the matrix's verdicts are
/// never quoted as *this cell is inert*.
const MX_DS: f64 = 0.0125;
const MX_EVERY: usize = 8;
const MX_TAU_TS: [f64; 1] = [TAU_T];

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
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn sm_of(phi: f64) -> f64 { phi / FLOOR - 1.0 }

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

/// `slice_ag_oracle.rs`'s `suite_arm`, which is both suites' fixture arm.
fn suite_arm(sm: f64, inc: bool) -> LeverArm {
    LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, sm, Some(TAU))),
        stator_lim: if inc { None }
                    else { Some(StatorLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S))) },
        stator_inc: if inc {
            Some(StatorIncidenceLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S)))
        } else { None },
        ..Default::default()
    }
}

/// The four knobs both suites' fixtures write by PLAIN ASSIGNMENT, plus rung 76's fifth.
fn set_rig_knobs(m: &ScheduledStatorCore, cap: bool) {
    m.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);
    m.fuel.inner.ref_law.set(REF_LAW_DEFAULT);
    m.fuel.inner.windup_law.set(WINDUP_LAW_NONE);
    m.fuel.inner.tau_t.set(None);
    if cap {
        m.fuel.inner.cap_law.set(CAP_LAW_SOLVE);
    }
}

/// **THIS FILE SILENCES A PANIC PER THREAD, NOT PER PROCESS**, for slice AE's recorded reason: an
/// empty hook installed process-wide made two genuinely failing gates report `FAILED` with no
/// message at all.
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
    static QUIET: Cell<bool> = const { Cell::new(false) };
}

/// Run `f`, returning its panic message if it had one.
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
// 1 — THE INJECTIONS
// =============================================================================================

// The counters the observer pointers write. **`Cell`, not `AtomicUsize`**, because every
// seat runs on the thread that reads its tally and a cross-thread counter would report
// another gate's marches.
thread_local! {
    static N_SENSED: Cell<usize> = const { Cell::new(0) };
    static N_SHARED_75: Cell<usize> = const { Cell::new(0) };
    static N_SHARED_76: Cell<usize> = const { Cell::new(0) };
    static N_WINDUP_TAU: Cell<usize> = const { Cell::new(0) };
}

thread_local! {
    static PROBING: Cell<bool> = const { Cell::new(false) };
    static SCALE: Cell<f64> = const { Cell::new(1.0) };
    static LEGS: std::cell::RefCell<Vec<(f64, f64)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

fn reset_counts() {
    N_SENSED.with(|n| n.set(0));
    N_SHARED_75.with(|n| n.set(0));
    N_SHARED_76.with(|n| n.set(0));
    N_WINDUP_TAU.with(|n| n.set(0));
}

/// **THE OBSERVERS.** Each increments and then calls the SHIPPED body through the `const` table,
/// never through the machine's own slot — a delegation through the receiver would find this same
/// wrapper and recur.
fn counting_sensed_cap(
    c: &FuelTransientCore, f: &FlightCondition, a: f64, b: f64, sch: &AccelSchedule,
    mf: Option<f64>,
) -> Result<Option<f64>, Abort> {
    N_SENSED.with(|n| n.set(n.get() + 1));
    (R76_TRIPLE.sensed_cap)(c, f, a, b, sch, mf)
}

fn counting_shared_rig_75(
    core: &ScheduledStatorCore, arm: &SharedRigArm,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>) {
    N_SHARED_75.with(|n| n.set(n.get() + 1));
    (R75_TRIPLE.shared_rig)(core, arm)
}

fn counting_shared_rig_76(
    core: &ScheduledStatorCore, arm: &SharedRigArm,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>) {
    N_SHARED_76.with(|n| n.set(n.get() + 1));
    (R76_TRIPLE.shared_rig)(core, arm)
}

/// **THE ACCEL LEG, ISOLATED, AT THE REAL CALL SITE.** A `cap_fuel` that records what the two cap
/// laws would return **with the surge leg omitted**, then delegates and returns the shipped answer.
///
/// # WHY IT EVALUATES MORE THAN IT REPORTS, AND WHY THAT IS NOT § 5.31 (viii) ITEM 2
///
/// That item records a spy whose COUNT was double because the spy called the method it was
/// counting. This one does the same thing on purpose and is **never used for a count** — every
/// tally in this file comes from a pure delegating observer. What it is used for is a COMPARISON,
/// and a comparison has to evaluate both sides. The re-entrancy flag stops the two probe calls
/// from probing themselves.
///
/// **The alternative was to replay the call afterwards from recorded arguments, and it is wrong
/// here**: `sensed_cap` reaches `try_instant_fuel`, which reads the marched bleed and stator states
/// that only exist inside the live call. A replay would answer at a state the march never had.
fn probing_cap_fuel(
    ft: &FuelTransientCore, flight: &FlightCondition, a: f64, h: f64, mf_sched: f64,
    accel: Option<&AccelSchedule>, surge: Option<&Floor>, mf_app: Option<f64>,
) -> Result<f64, Abort> {
    if let (Some(acc), Some(_)) = (accel, mf_app) {
        if !PROBING.with(|q| q.get()) {
            PROBING.with(|q| q.set(true));
            let one = |law: &'static str| -> Option<f64> {
                let _c = CapScope::set(&ft.inner, law);
                (R76_TRIPLE.cap_fuel)(ft, flight, a, h, mf_sched, Some(acc), None, mf_app).ok()
            };
            let (sv, sn) = (one(CAP_LAW_SOLVE), one(CAP_LAW_SENSED));
            if let (Some(x), Some(y)) = (sv, sn) {
                LEGS.with(|l| l.borrow_mut().push((x, y)));
            }
            PROBING.with(|q| q.set(false));
        }
    }
    (R76_TRIPLE.cap_fuel)(ft, flight, a, h, mf_sched, accel, surge, mf_app)
}

fn counting_windup_tau(c: &turbojet::two_spool_transient::TwoSpoolTransientCore) -> Option<f64> {
    N_WINDUP_TAU.with(|n| n.set(n.get() + 1));
    (R75_TRIPLE.windup_tau)(c)
}

/// **THE REACHABILITY PROBE, AND IT IS NOT A PLAUSIBLE DEFECT — that is the point.**
/// The two defects above ask *does a realistic transcription slip reach the untagged
/// message?* This asks the prior question — *is the site reachable AT ALL?* — by scaling
/// the shipped cap until one law's march aborts and the other's does not. A gate that
/// concluded *unreachable* from two tries would be a zero taken from a population that
/// was never swept, which is § 5.31 (viii) item 5.
fn scaled_sensed_cap(
    ft: &FuelTransientCore, flight: &FlightCondition, a: f64, h: f64, sch: &AccelSchedule,
    mf: Option<f64>,
) -> Result<Option<f64>, Abort> {
    match (R76_TRIPLE.sensed_cap)(ft, flight, a, h, sch, mf)? {
        Some(v) => Ok(Some(v * SCALE.with(|x| x.get()))),
        None => Ok(None),
    }
}

/// [`T76_SWAPPED`]'s body — the shipped cell with `cap`'s two arguments exchanged.
fn swapped_sensed_cap(
    ft: &FuelTransientCore, flight: &FlightCondition, a: f64, h: f64, sch: &AccelSchedule,
    mf: Option<f64>,
) -> Result<Option<f64>, Abort> {
    if ft.inner.cap_law.get() != CAP_LAW_SENSED {
        return Ok(None);
    }
    let mf_app = mf.expect("the shipped cell's own refusal, kept");
    let i = ft.try_instant_fuel(flight, a, h, 1e-9f64.max(mf_app))?;
    // THE DEFECT — the shipped line is `cap(n_hp, pt4 / pi_b)`.
    Ok(Some(sch.cap(i.base.close.pt4 / ft.inner.inner.base.pi_b, i.base.close.n_hp)))
}

/// [`T76_NO_PI_B`]'s body — the shipped cell with `pt4 / pi_b` written `pt4`.
fn no_pi_b_sensed_cap(
    ft: &FuelTransientCore, flight: &FlightCondition, a: f64, h: f64, sch: &AccelSchedule,
    mf: Option<f64>,
) -> Result<Option<f64>, Abort> {
    if ft.inner.cap_law.get() != CAP_LAW_SENSED {
        return Ok(None);
    }
    let mf_app = mf.expect("the shipped cell's own refusal, kept");
    let i = ft.try_instant_fuel(flight, a, h, 1e-9f64.max(mf_app))?;
    // THE DEFECT — the shipped line divides by `pi_b` here.
    Ok(Some(sch.cap(i.base.close.n_hp, i.base.close.pt4)))
}

// -------------------------------------------------------------- the injected TripleHooks
//
// Each is the rung's own table with ONE cell moved — to the parent it was re-aimed from, or to an
// observer.

static T75_WINDUP_TAU: TripleHooks =
    TripleHooks { windup_tau: R74_TRIPLE.windup_tau, ..R75_TRIPLE };
static T75_SHARED_RIG: TripleHooks =
    TripleHooks { shared_rig: R74_TRIPLE.shared_rig, ..R75_TRIPLE };
static T75_COUNT_SHARED: TripleHooks =
    TripleHooks { shared_rig: counting_shared_rig_75, ..R75_TRIPLE };
static T75_COUNT_TAU: TripleHooks =
    TripleHooks { windup_tau: counting_windup_tau, ..R75_TRIPLE };

static T76_SENSED_CAP: TripleHooks =
    TripleHooks { sensed_cap: R75_TRIPLE.sensed_cap, ..R76_TRIPLE };
static T76_SHARED_RIG: TripleHooks =
    TripleHooks { shared_rig: R75_TRIPLE.shared_rig, ..R76_TRIPLE };
static T76_COUNT_SENSED: TripleHooks =
    TripleHooks { sensed_cap: counting_sensed_cap, ..R76_TRIPLE };
static T76_COUNT_SHARED: TripleHooks =
    TripleHooks { shared_rig: counting_shared_rig_76, ..R76_TRIPLE };

static T76_PROBE_LEGS: TripleHooks =
    TripleHooks { cap_fuel: probing_cap_fuel, ..R76_TRIPLE };
/// **A PLAUSIBLE PORT DEFECT, NOT AN ARBITRARY ONE.** Rung 76's body ends
/// `accel.cap(n_hp, pt4 / pi_b)` — the burner pressure ratio turning `pt4` into the `pt3` the
/// schedule is written on. Dropping that ONE division is a transcription slip of exactly the
/// kind slice AG's own step 1 found 18 of in its line citations, and it inflates the sensed cap
/// by `1/pi_b`.
static T76_NO_PI_B: TripleHooks =
    TripleHooks { sensed_cap: no_pi_b_sensed_cap, ..R76_TRIPLE };

/// **THE SECOND PLAUSIBLE DEFECT, AND THE ONE A TYPE SYSTEM CANNOT SEE.**
/// `AccelSchedule::cap(n_h: f64, pt3: f64)` takes two `f64`, so calling it
/// `cap(pt3, n_h)` COMPILES. An argument-order slip between two same-typed scalars is
/// the defect class this phase keeps finding, and it is the one a `Result` cannot
/// report and a reduce gate cannot see.
static T76_SWAPPED: TripleHooks =
    TripleHooks { sensed_cap: swapped_sensed_cap, ..R76_TRIPLE };
static T76_SCALED: TripleHooks =
    TripleHooks { sensed_cap: scaled_sensed_cap, ..R76_TRIPLE };

static F75_INTEGRATE: FuelTransientHooks =
    FuelTransientHooks { integrate_fuel: R74_FUEL.integrate_fuel, ..R75_FUEL };
static F76_INTEGRATE: FuelTransientHooks =
    FuelTransientHooks { integrate_fuel: R75_FUEL.integrate_fuel, ..R76_FUEL };

/// Build a machine on an ARBITRARY table quintuple — **not through either cascade**, which
/// hardcode their own tables. The point is to install tables neither would.
#[allow(clippy::too_many_arguments)]
fn with_tables(
    core: &ScheduledStatorCore, a: &LeverArm, lever: &'static LeverHooks,
    fuel: &'static FuelTransientHooks, triple: &'static TripleHooks,
    two: &'static turbojet::two_spool_transient::TwoSpoolTransientHooks,
    stator: &'static turbojet::stator_transient::StatorTransientHooks,
) -> ScheduledStatorCore {
    full_of(ScheduledStatorTransient::with_ref_tables(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(),
        a.stator, two, stator, fuel, lever,
        LeverArming { bleed: a.bleed, sched: a.bleed_sched, lim: a.bleed_lim },
        triple, a.stator_lim, a.stator_inc))
}

/// One injection's `at_lever`: it rebuilds with **that injection's own tables** and carries the
/// knobs the shipped cell carries.
///
/// The sibling constructor is what launders a table injection (AC step 7), so an injection that
/// did not re-aim it would be undone the first time a reader rebuilt — green and vacuous, which
/// is this phase's most-repeated defect.
macro_rules! injection {
    ($lever:ident, $rebuild:ident, $base:expr, $fuel:expr, $triple:expr, $two:expr, $stator:expr,
     $cap:expr) => {
        static $lever: LeverHooks = LeverHooks { at_lever: $rebuild, ..$base };
        fn $rebuild(core: &ScheduledStatorCore, a: &LeverArm) -> ScheduledStatorCore {
            let m = with_tables(core, a, &$lever, $fuel, $triple, $two, $stator);
            m.fuel.inner.lag_coord.set(core.fuel.inner.lag_coord.get());
            m.fuel.inner.ref_law.set(core.fuel.inner.ref_law.get());
            m.fuel.inner.windup_law.set(core.fuel.inner.windup_law.get());
            m.fuel.inner.tau_t.set(core.fuel.inner.tau_t.get());
            m.fuel.inner.ic_cap.set(core.fuel.inner.ic_cap.get());
            if $cap {
                m.fuel.inner.cap_law.set(core.fuel.inner.cap_law.get());
            }
            m
        }
    };
}

injection!(L75_NONE, r75_none, R75, &R75_FUEL, &R75_TRIPLE, &R75_TWO, &R75_STATOR, false);
injection!(L75_TAU, r75_tau, R75, &R75_FUEL, &T75_WINDUP_TAU, &R75_TWO, &R75_STATOR, false);
injection!(L75_RIG, r75_rig, R75, &R75_FUEL, &T75_SHARED_RIG, &R75_TWO, &R75_STATOR, false);
injection!(L75_FUEL, r75_fuel, R75, &F75_INTEGRATE, &R75_TRIPLE, &R75_TWO, &R75_STATOR, false);
injection!(L75_CRIG, r75_crig, R75, &R75_FUEL, &T75_COUNT_SHARED, &R75_TWO, &R75_STATOR, false);
injection!(L75_CTAU, r75_ctau, R75, &R75_FUEL, &T75_COUNT_TAU, &R75_TWO, &R75_STATOR, false);

injection!(L76_NONE, r76_none, R76, &R76_FUEL, &R76_TRIPLE, &R76_TWO, &R76_STATOR, true);
injection!(L76_SENSED, r76_sensed, R76, &R76_FUEL, &T76_SENSED_CAP, &R76_TWO, &R76_STATOR, true);
injection!(L76_RIG, r76_rig, R76, &R76_FUEL, &T76_SHARED_RIG, &R76_TWO, &R76_STATOR, true);
injection!(L76_FUEL, r76_fuel, R76, &F76_INTEGRATE, &R76_TRIPLE, &R76_TWO, &R76_STATOR, true);
injection!(L76_CSENSED, r76_csensed, R76, &R76_FUEL, &T76_COUNT_SENSED, &R76_TWO, &R76_STATOR,
           true);
injection!(L76_CRIG, r76_crig, R76, &R76_FUEL, &T76_COUNT_SHARED, &R76_TWO, &R76_STATOR, true);
injection!(L76_PROBE, r76_probe, R76, &R76_FUEL, &T76_PROBE_LEGS, &R76_TWO, &R76_STATOR,
           true);
injection!(L76_NOPIB, r76_nopib, R76, &R76_FUEL, &T76_NO_PI_B, &R76_TWO, &R76_STATOR, true);
injection!(L76_SWAP, r76_swap, R76, &R76_FUEL, &T76_SWAPPED, &R76_TWO, &R76_STATOR, true);
injection!(L76_SCALE, r76_scale, R76, &R76_FUEL, &T76_SCALED, &R76_TWO, &R76_STATOR, true);

/// The `at_lever` cells themselves — the parent's sibling constructor, which builds a PARENT
/// machine. The only rows with no `injection!`: re-aiming `at_lever` at the parent is precisely
/// *stop carrying this rung's tables*, so a rebuild that carried them would be the opposite of
/// the injection.
static L75_AT_LEVER: LeverHooks = LeverHooks { at_lever: R74.at_lever, ..R75 };
static L76_AT_LEVER: LeverHooks = LeverHooks { at_lever: R75.at_lever, ..R76 };

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Inj {
    /// The baseline — this rung's own tables, reached through the injection machinery so the
    /// fixture is the same machinery every other row uses.
    None,
    AtLever,
    IntegrateFuel,
    SharedRig,
    /// Rung 75 only.
    WindupTau,
    /// Rung 76 only.
    SensedCap,
    /// The observers.
    CountSharedRig,
    CountWindupTau,
    CountSensedCap,
    /// The accel leg, isolated at the real call site — a COMPARISON instrument, never a count.
    ProbeLegs,
    /// The two plausible port defects this file drives, for P6.
    NoPiB,
    Swapped,
    /// The reachability probe, with its factor in [`SCALE`].
    Scaled,
}

/// The rows § 3 prints, per rung. The observers are NOT in the matrix — their readings must be
/// identical by construction and § 2 asserts exactly that.
const INJS_75: [Inj; 5] =
    [Inj::None, Inj::AtLever, Inj::IntegrateFuel, Inj::SharedRig, Inj::WindupTau];
const INJS_76: [Inj; 5] =
    [Inj::None, Inj::AtLever, Inj::IntegrateFuel, Inj::SharedRig, Inj::SensedCap];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Rung { R75, R76 }

fn tables_of(rung: Rung, inj: Inj)
    -> (&'static LeverHooks, &'static FuelTransientHooks, &'static TripleHooks)
{
    match (rung, inj) {
        (Rung::R75, Inj::None) => (&L75_NONE, &R75_FUEL, &R75_TRIPLE),
        (Rung::R75, Inj::AtLever) => (&L75_AT_LEVER, &R75_FUEL, &R75_TRIPLE),
        (Rung::R75, Inj::IntegrateFuel) => (&L75_FUEL, &F75_INTEGRATE, &R75_TRIPLE),
        (Rung::R75, Inj::SharedRig) => (&L75_RIG, &R75_FUEL, &T75_SHARED_RIG),
        (Rung::R75, Inj::WindupTau) => (&L75_TAU, &R75_FUEL, &T75_WINDUP_TAU),
        (Rung::R75, Inj::CountSharedRig) => (&L75_CRIG, &R75_FUEL, &T75_COUNT_SHARED),
        (Rung::R75, Inj::CountWindupTau) => (&L75_CTAU, &R75_FUEL, &T75_COUNT_TAU),
        (Rung::R76, Inj::None) => (&L76_NONE, &R76_FUEL, &R76_TRIPLE),
        (Rung::R76, Inj::AtLever) => (&L76_AT_LEVER, &R76_FUEL, &R76_TRIPLE),
        (Rung::R76, Inj::IntegrateFuel) => (&L76_FUEL, &F76_INTEGRATE, &R76_TRIPLE),
        (Rung::R76, Inj::SharedRig) => (&L76_RIG, &R76_FUEL, &T76_SHARED_RIG),
        (Rung::R76, Inj::SensedCap) => (&L76_SENSED, &R76_FUEL, &T76_SENSED_CAP),
        (Rung::R76, Inj::CountSharedRig) => (&L76_CRIG, &R76_FUEL, &T76_COUNT_SHARED),
        (Rung::R76, Inj::CountSensedCap) => (&L76_CSENSED, &R76_FUEL, &T76_COUNT_SENSED),
        (Rung::R76, Inj::ProbeLegs) => (&L76_PROBE, &R76_FUEL, &T76_PROBE_LEGS),
        (Rung::R76, Inj::NoPiB) => (&L76_NOPIB, &R76_FUEL, &T76_NO_PI_B),
        (Rung::R76, Inj::Swapped) => (&L76_SWAP, &R76_FUEL, &T76_SWAPPED),
        (Rung::R76, Inj::Scaled) => (&L76_SCALE, &R76_FUEL, &T76_SCALED),
        (r, i) => panic!("{i:?} is not an injection at {r:?} — the two INJS lists are the only \
                          source of these pairs"),
    }
}

/// Build the injected machine at one arm. **The knobs are written by plain assignment afterwards,
/// exactly as both suites' fixtures do**, so a machine reaching a reader is in the state the
/// oracle's `windup_rig` / `cap_rig` put it in.
fn build(rung: Rung, inj: Inj, phi: f64, inc: bool) -> ScheduledStatorCore {
    let a = suite_arm(sm_of(phi), inc);
    let (lever, fuel, triple) = tables_of(rung, inj);
    let base = match rung {
        Rung::R75 => full_of(build_anti_windup_cascade(
            design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &a)),
        Rung::R76 => full_of(build_sensed_cap_cascade(
            design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &a)),
    };
    let (two, stator) = match rung {
        Rung::R75 => (&R75_TWO, &R75_STATOR),
        Rung::R76 => (&R76_TWO, &R76_STATOR),
    };
    let m = with_tables(&base, &a, lever, fuel, triple, two, stator);
    set_rig_knobs(&m, rung == Rung::R76);
    m
}

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

fn slot_of(inj: Inj) -> Vec<&'static str> {
    match inj {
        Inj::None | Inj::AtLever | Inj::IntegrateFuel => vec![],
        Inj::SharedRig | Inj::CountSharedRig => vec!["shared_rig"],
        Inj::WindupTau | Inj::CountWindupTau => vec!["windup_tau"],
        Inj::SensedCap | Inj::CountSensedCap => vec!["sensed_cap"],
        Inj::NoPiB | Inj::Swapped | Inj::Scaled => vec!["sensed_cap"],
        Inj::ProbeLegs => vec!["cap_fuel"],
    }
}

fn shipped_triple(rung: Rung) -> &'static TripleHooks {
    match rung { Rung::R75 => &R75_TRIPLE, Rung::R76 => &R76_TRIPLE }
}

/// **THE INSTALL PROOF, BEFORE ANY ROW IS READ.** An all-silent row's only evidence that the
/// injection took at all is this one — rebuild a sibling through the injected `at_lever` and
/// assert the tables it hands back are the injected ones.
fn assert_installed(m: &ScheduledStatorCore, rung: Rung, inj: Inj, a: &LeverArm) {
    let want = tables_of(rung, inj);
    assert_eq!(triple_diff(m.triple_hooks(), shipped_triple(rung)), slot_of(inj),
               "the machine itself carries {inj:?}'s slot and no other");
    let sib = m.at_lever(a);
    if inj == Inj::AtLever {
        // **THE ONE ROW WHERE THE SIBLING IS SUPPOSED TO BE A DIFFERENT RUNG.** Re-aiming
        // `at_lever` at the parent IS *stop carrying this rung's tables*, so demanding the
        // injected table back would be the opposite of the injection. What is checked instead is
        // that the sibling carries the PARENT's own shipped table, exactly — not a
        // parent-shaped one this file assembled.
        assert_eq!(triple_diff(sib.triple_hooks(), parent_triple(rung)), Vec::<&str>::new(),
                   "the parent's sibling constructor installs the PARENT's table, which is the \
                    whole of this injection");
        assert_eq!(triple_diff(sib.triple_hooks(), shipped_triple(rung)),
                   this_rungs_own_cells(rung),
                   "and it therefore differs from this rung's by exactly this rung's own two \
                    re-aimed cells — the positive control on the row");
        return;
    }
    assert_eq!(triple_diff(sib.triple_hooks(), shipped_triple(rung)), slot_of(inj),
               "AND THE SIBLING DOES TOO — without this the first rebuild inside any reader \
                launders {inj:?} away and every verdict below is against the shipped machine");
    assert!(fn_addr_eq(sib.fuel.hooks.integrate_fuel, want.1.integrate_fuel),
            "the sibling carries {inj:?}'s fuel table");
}

fn parent_triple(rung: Rung) -> &'static TripleHooks {
    match rung { Rung::R75 => &R74_TRIPLE, Rung::R76 => &R75_TRIPLE }
}

/// The two cells each rung re-aims, in [`triple_diff`]'s field order.
fn this_rungs_own_cells(rung: Rung) -> Vec<&'static str> {
    match rung {
        Rung::R75 => vec!["shared_rig", "windup_tau"],
        Rung::R76 => vec!["shared_rig", "sensed_cap"],
    }
}

/// **EIGHT SWAPS, EIGHT INJECTIONS, AND EACH MOVES EXACTLY ITS OWN SLOT.**
///
/// The equality half is the one that matters: an injection that moved a second cell would make
/// every verdict in § 3 a reading of two changes at once, and at a `0 ADD` slice nothing would
/// go loud.
#[test]
fn each_injection_moves_exactly_its_own_slot() {
    for (rung, injs) in [(Rung::R75, &INJS_75), (Rung::R76, &INJS_76)] {
        for inj in injs.iter() {
            let (_, _, t) = tables_of(rung, *inj);
            assert_eq!(triple_diff(t, shipped_triple(rung)), slot_of(*inj), "{rung:?} {inj:?}");
        }
    }
    for (rung, inj) in [(Rung::R75, Inj::CountSharedRig), (Rung::R75, Inj::CountWindupTau),
                        (Rung::R76, Inj::CountSharedRig), (Rung::R76, Inj::CountSensedCap)] {
        let (_, _, t) = tables_of(rung, inj);
        assert_eq!(triple_diff(t, shipped_triple(rung)), slot_of(inj), "{rung:?} {inj:?}");
    }
    // AND THE TWO `at_lever` ROWS ARE THE PARENT's OWN CELL, not a lookalike this file built.
    assert!(fn_addr_eq(L75_AT_LEVER.at_lever, R74.at_lever));
    assert!(fn_addr_eq(L76_AT_LEVER.at_lever, R75.at_lever));
    assert!(!fn_addr_eq(L75_AT_LEVER.at_lever, R75.at_lever),
            "the control — rung 75's own cell is a DIFFERENT function, so the row above is a \
             measurement");
    assert!(!fn_addr_eq(L76_AT_LEVER.at_lever, R76.at_lever));
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

/// **SLICE AG's OWN READERS**, not slice AF's rung-74 names. Copying the previous slice's grid is
/// the defect § 5.31.5 wrote up — two of sixteen sweep verdicts there were properties of a grid
/// taken from somewhere else — so this list is derived from what rungs 75 and 76 publish.
const SEATS: [&str; 7] = ["windup_gains", "contraction_law", "device_control", "windup_bill",
                          "cap_gains", "cap_bill", "solve_gain"];

/// The arm each seat's shipped drive uses — `slice_ag_oracle.rs`'s sections A–G.
fn seat_phi(name: &str) -> f64 {
    match name {
        "windup_gains" | "cap_gains" | "solve_gain" => PHI_JAC,
        "contraction_law" | "device_control" | "windup_bill" | "cap_bill" => PHI_BOTH,
        _ => unreachable!("SEATS is the only source of these names"),
    }
}

fn run_seat(name: &str, m: &ScheduledStatorCore, inc: bool) -> Seen {
    quiet_hook();
    QUIET.with(|q| q.set(true));
    let f = flight();
    let phi = seat_phi(name);
    let out = catch_unwind(AssertUnwindSafe(|| match name {
        "windup_gains" => fingerprint(&windup_gains(
            m, &f, LO, HI, TT4_MAX, phi, TAUS, &MX_TAU_TS, &WG_REFS, inc, R, SETTLE, MX_DS,
            V_MAX, MX_EVERY)),
        "contraction_law" => fingerprint(&contraction_law(
            m, &f, LO, HI, TT4_MAX, phi, TAUS, &MX_TAU_TS, CL_RES0, CL_TOL, CL_IC_CAP, inc, R,
            SETTLE, MX_DS, V_MAX)),
        "device_control" => fingerprint(&device_control(
            m, &f, LO, HI, TT4_MAX, phi, TAUS, &MX_TAU_TS, &DC_REFS, inc, R, SETTLE, MX_DS,
            V_MAX)),
        "windup_bill" => fingerprint(&windup_bill(
            m, &f, LO, HI, TT4_MAX, phi, TAUS, &MX_TAU_TS, WB_REF, inc, R, SETTLE, MX_DS,
            V_MAX)),
        "cap_gains" => fingerprint(&cap_gains(
            m, &f, LO, HI, TT4_MAX, phi, MARGIN, TAUS, TAU_T, &CAP_GAINS_REFS, &CAP_GAINS_LAWS,
            inc, R, SETTLE, MX_DS, V_MAX, MX_EVERY)),
        "cap_bill" => fingerprint(&cap_bill(
            m, &f, LO, HI, TT4_MAX, phi, MARGIN, TAUS, TAU_T, REF_LAW_DEFAULT, WINDUP_LAW_NONE,
            inc, R, SETTLE, MX_DS, V_MAX, CAP_BILL_TAIL)),
        "solve_gain" => fingerprint(&solve_gain(
            m, &f, LO, HI, TT4_MAX, phi, MARGIN, TAUS, REF_LAW_DEFAULT, inc, R, SETTLE, MX_DS,
            V_MAX, SOLVE_GAIN_DQ, MX_EVERY)),
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

/// **THE OBSERVERS ARE PURE, AND THAT IS ASSERTED BEFORE ANY TALLY IS BELIEVED.**
///
/// A counting pointer is only an instrument if the machine it sits in is otherwise the shipped
/// one. Each observer's reading at every seat must be BIT-IDENTICAL to the baseline's; a `DIFF`
/// here would mean every count in § 3 was taken from a different plant, which is
/// [[instrument-fed-by-what-it-certifies]] in its dispatch-shaped form.
#[test]
fn the_counting_pointers_are_pure_observers() {
    for (rung, obs) in [(Rung::R75, vec![Inj::CountSharedRig, Inj::CountWindupTau]),
                        (Rung::R76, vec![Inj::CountSharedRig, Inj::CountSensedCap])] {
        for seat in SEATS.iter() {
            let phi = seat_phi(seat);
            let base = run_seat(seat, &build(rung, Inj::None, phi, false), false);
            for o in obs.iter() {
                let got = run_seat(seat, &build(rung, *o, phi, false), false);
                assert_eq!(verdict(&base, &got), match &base {
                    Seen::Read(_) => "same",
                    Seen::Broke(_) => "refused",
                }, "{rung:?} {o:?} at {seat} is NOT a pure observer");
            }
        }
    }
}

// =============================================================================================
// 3 — THE SEAT MATRIX, RUN WHOLE, WITH THE COUNT BESIDE THE VERDICT
// =============================================================================================

/// One row of the matrix: the verdict at each seat, and — for the two cells that have an
/// observer — how many times the cell was actually entered.
fn matrix(rung: Rung, injs: &[Inj], observers: &[(Inj, &'static str, [usize; 7])],
          rows: &[(Inj, [&'static str; 7])]) {
    let base: Vec<Seen> = SEATS.iter()
        .map(|s| run_seat(s, &build(rung, Inj::None, seat_phi(s), false), false)).collect();
    for (i, b) in base.iter().enumerate() {
        assert!(matches!(b, Seen::Read(_)),
                "the baseline seat {} must READ, or every verdict below is against a broken \
                 fixture: {b:?}", SEATS[i]);
    }

    let mut tally: Vec<(Inj, Vec<&'static str>)> = Vec::new();
    for inj in injs.iter().skip(1) {
        let a = suite_arm(sm_of(PHI_JAC), false);
        let row: Vec<&'static str> = SEATS.iter().zip(base.iter())
            .map(|(s, b)| {
                let m = build(rung, *inj, seat_phi(s), false);
                assert_installed(&m, rung, *inj, &a);
                verdict(b, &run_seat(s, &m, false))
            }).collect();
        println!("{rung:?} {inj:?}: {row:?}");
        tally.push((*inj, row));
    }

    // THE COUNTS — the half AD step 6's rule had to infer from the rest of the row.
    for (obs, label, want) in observers.iter() {
        let counts: Vec<usize> = SEATS.iter().map(|s| {
            reset_counts();
            let _ = run_seat(s, &build(rung, *obs, seat_phi(s), false), false);
            match *obs {
                Inj::CountSharedRig => match rung {
                    Rung::R75 => N_SHARED_75.with(|n| n.get()),
                    Rung::R76 => N_SHARED_76.with(|n| n.get()),
                },
                Inj::CountWindupTau => N_WINDUP_TAU.with(|n| n.get()),
                Inj::CountSensedCap => N_SENSED.with(|n| n.get()),
                _ => unreachable!("only the observers are counted"),
            }
        }).collect();
        println!("{rung:?} COUNT {label}: {counts:?}");
        assert_eq!(counts.as_slice(), want.as_slice(),
                   "{rung:?}'s {label} tally moved. **A COUNT IS THE HALF A VERDICT CANNOT \
                    CARRY** — this row is what separates *the cell RAN and was redundant* from \
                    *the cell was never entered*, so it is pinned, not printed.");
    }

    // THE ROWS, TRANSCRIBED FROM THE RUN AND NEVER PREDICTED INTO IT.
    for (inj, want) in rows.iter() {
        let got = tally.iter().find(|(k, _)| k == inj).expect("every injection ran").1.clone();
        assert_eq!(got.as_slice(), want.as_slice(), "{rung:?} {inj:?}");
    }

    // **THE `at_lever` ROW IS THE POSITIVE CONTROL, AND IT IS READ OFF THE RUN.**
    //
    // Re-aiming the sibling constructor at the parent stops the machine carrying this rung's
    // tables at all, so a seat that is silent there is silent because it never rebuilds — and
    // every OTHER row's silence has to be read against it. **The first writing took this row from
    // `rows`**, the table of EXPECTED verdicts two lines above — which is rung 70's *a gate
    // computing my own formula twice*, and it would have passed on a machine where nothing ran.
    // It is taken from `tally`, which is what the seats returned.
    let at_lever = &tally.iter().find(|(k, _)| *k == Inj::AtLever).expect("always driven").1;
    assert!(at_lever.iter().any(|v| *v != "same"),
            "{rung:?}: pointing `at_lever` at the parent moved NOTHING at seven seats, which \
             would mean no reader in this slice rebuilds a sibling — {at_lever:?}");
}

/// **RUNG 75's MATRIX.** `windup_gains` / `contraction_law` / `device_control` / `windup_bill` are
/// this rung's four readers; `cap_gains` / `cap_bill` / `solve_gain` are rung 76's, run here so a
/// cell that only bites one rung down is visible.
///
/// # WHAT THE ROWS SAY
///
/// `at_lever` and `windup_tau` produce the **IDENTICAL** row — `DIFF, DIFF, BROKE, BROKE` on the
/// four rung-75 readers and silence on rung 76's three. They are not the same injection and the
/// agreement is the finding: rung 75's sibling constructor exists to carry the DEVICE, so
/// removing the constructor and removing the device's hook are the same deletion as far as every
/// reader in the slice can tell.
///
/// `integrate_fuel` is silent at all seven, at BOTH rungs, and the install proof ran on every
/// cell. **That swap is a pure REFUSAL carrier on this grid**: rung 75's body adds guards and a
/// call made for its side effect, and on arms where no guard fires the parent's march is the same
/// march. `slice_ag_cells.rs` gates it by hand, which is the only instrument that can.
#[test]
fn the_seat_matrix_at_rung_75() {
    matrix(Rung::R75, &INJS_75,
           &[(Inj::CountSharedRig, "shared_rig", [2, 1, 4, 2, 3, 3, 2]),
             (Inj::CountWindupTau, "windup_tau", [14, 2, 6, 3, 16, 2, 0])],
           &[(Inj::AtLever, ["DIFF", "DIFF", "BROKE", "BROKE", "same", "same", "same"]),
             (Inj::IntegrateFuel, ["same"; 7]),
             (Inj::SharedRig, ["same"; 7]),
             (Inj::WindupTau, ["DIFF", "DIFF", "BROKE", "BROKE", "same", "same", "same"])]);
}

/// **RUNG 76's MATRIX, AND THE ROW THAT CLOSES § 5.30.6 (v).**
///
/// `sensed_cap` pointed at rung 75's body is silent at the four rung-75 readers and `DIFF` at all
/// three of rung 76's — and the tally beside it reads `[0, 0, 0, 0, 144, 1100, 24]`. **The four
/// zeros and the three silences are the SAME CELLS**, so that silence is UNREACHABILITY; the
/// `shared_rig` row is silent at all seven with a tally of `[2, 1, 4, 2, 3, 3, 2]`, so that
/// silence is REDUNDANCY. Two identical rows, two different facts, separated by a number rather
/// than by an argument about the rest of the row.
#[test]
fn the_seat_matrix_at_rung_76() {
    matrix(Rung::R76, &INJS_76,
           &[(Inj::CountSharedRig, "shared_rig", [2, 1, 4, 2, 3, 3, 2]),
             (Inj::CountSensedCap, "sensed_cap", [0, 0, 0, 0, 144, 1100, 24])],
           &[(Inj::AtLever, ["same", "same", "same", "same", "DIFF", "DIFF", "DIFF"]),
             (Inj::IntegrateFuel, ["same"; 7]),
             (Inj::SharedRig, ["same"; 7]),
             (Inj::SensedCap, ["same", "same", "same", "same", "DIFF", "DIFF", "DIFF"])]);
}

// =============================================================================================
// 4 — THE ARMING GATE — § 5.30.6 (v)'s OBLIGATION, THREE-SIDED
// =============================================================================================

/// Every float a marched point records, as bits — the WHOLE-POINT population.
/// `branch` is an enum and is compared through its label.
fn point_bits(p: &FuelPoint) -> Vec<u64> {
    let mut v = vec![p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.tt4.to_bits(),
                     p.f.to_bits(), p.pi_lpc.to_bits(), p.pi_hpc.to_bits(), p.phi_lp.to_bits(),
                     p.phi_hp.to_bits(), p.mdot_air.to_bits(), p.sp_thrust.to_bits(),
                     p.mf.to_bits(), p.mf_sched.to_bits(),
                     p.branch.label().as_bytes().iter().fold(0u64, |h, b| h * 131 + *b as u64)];
    // **AND THE `extra`, WHICH IS WHERE THE EIGHT LIVE.** § 5.31.4 (e): the eight points that
    // agree on `Tt4` are the first eight, `s = 0 … 0.035`, where the two LEG STATES have already
    // parted. A tuple that stops at the base fields reads 333 and is not the population § 5.31
    // (i)'s `341 of 341` is about — which the first writing of this helper found out by
    // measuring 333 twice and calling one of them the tuple.
    if let PointExtra::Demand { g, required, b, b_cmd, v: vv, v_cmd, ic_res, g_fuel, g_gov,
                                required_fuel, required_gov, w_fuel, w_gov, cap_fuel, cap_gov,
                                ic_iters, .. } = p.extra {
        v.extend([g.to_bits(), required.to_bits(), b.to_bits(), b_cmd.to_bits(), vv.to_bits(),
                  v_cmd.to_bits(), ic_res.to_bits(), g_fuel.to_bits(), g_gov.to_bits(),
                  required_fuel.to_bits(), required_gov.to_bits(), w_fuel.to_bits(),
                  w_gov.to_bits(), cap_fuel.to_bits(), cap_gov.to_bits(), ic_iters as u64]);
    }
    v
}

/// The shipped march at one arm under one cap law, at the FULL grid.
fn march_at(m: &ScheduledStatorCore, phi: f64, law: &'static str, inc: bool) -> Vec<FuelPoint> {
    let f = flight();
    let sm = sm_of(phi);
    let acc = accel_for(m, &f, LO, HI, sm, TT4_MAX, TAUS, V_MAX, inc, MARGIN);
    cap_march(m, &f, LO, HI, TT4_MAX, sm, TAUS, R, SETTLE, DS, V_MAX, inc, LAG_COORD_DEMAND,
              REF_LAW_DEFAULT, WINDUP_LAW_NONE, None, law, &acc, None).3
}

/// **SIDE 1 — DISPATCHED.** Slice AF's row for this cell was silent at all seven of its seats
/// because not one armed an `AccelSchedule`. Arming one is what rung 76's readers do, and the
/// count is the closure § 5.30.6 (v) booked.
///
/// The `solve` arm is the control and it is the sharper half: the cell is dispatched **just as
/// often** there and returns `None`, so **the reduce is a DISPATCH and not a tolerance** (P3) —
/// and a gate that counted only on the `sensed` arm could not have said so.
#[test]
fn sensed_cap_is_dispatched_once_an_accel_is_armed_and_the_reduce_is_by_dispatch() {
    let mut seen: Vec<(&str, usize, usize)> = Vec::new();
    for law in [CAP_LAW_SOLVE, CAP_LAW_SENSED] {
        reset_counts();
        let m = build(Rung::R76, Inj::CountSensedCap, PHI_JAC, false);
        let traj = march_at(&m, PHI_JAC, law, false);
        seen.push((law, N_SENSED.with(|n| n.get()), traj.len()));
    }
    println!("sensed_cap dispatch: {seen:?}");
    assert!(seen[0].1 > 0, "the cell is entered on the SOLVE arm too — `_cap_fuel` calls it \
                            before the law is consulted");
    assert_eq!(seen[0].1, seen[1].1,
               "THE REDUCE IS BY DISPATCH: the cell is entered exactly as often under `solve` as \
                under `sensed`, and what differs is what it RETURNS");
    assert_eq!(seen[0].2, seen[1].2, "and both arms march the same number of points at PHI_JAC");

    // THE CONTROL THAT MAKES THE COUNT A MEASUREMENT — with no schedule armed the cell is
    // UNREACHABLE, which is slice AF's row for it, reproduced here rather than quoted.
    reset_counts();
    let m = build(Rung::R76, Inj::CountSensedCap, PHI_JAC, false);
    let f = flight();
    let sm = sm_of(PHI_JAC);
    let (_, _, _, traj) = turbojet::demand_coordinate::coord_march(
        &m, &f, LO, HI, TT4_MAX, sm, TAUS, R, SETTLE, DS, V_MAX, false, LAG_COORD_DEMAND,
        REF_LAW_DEFAULT, None);
    assert!(!traj.is_empty(), "the control march has to actually march");
    assert_eq!(N_SENSED.with(|n| n.get()), 0,
               "SLICE AF's ROW, REPRODUCED: with no `AccelSchedule` armed the cell is never \
                entered, so its silence there was UNREACHABILITY and not redundancy");
}

/// **SIDE 2 — DISCRIMINATING, at `PHI_JAC`.** The cell is entered at every call and returns a
/// value 2.32 % below the solve at every one of them — and the plant does not move, because the
/// `min` one level down discards it. Scoring THIS at the trajectory reads an exact zero.
#[test]
fn the_sensed_cap_returns_a_different_number_at_every_call_where_the_plant_does_not_move() {
    let m = build(Rung::R76, Inj::None, PHI_JAC, false);
    let solve = march_at(&m, PHI_JAC, CAP_LAW_SOLVE, false);
    let sensed = march_at(&m, PHI_JAC, CAP_LAW_SENSED, false);
    assert_eq!(solve.len(), sensed.len());
    let n_diff = solve.iter().zip(sensed.iter())
        .filter(|(a, b)| a.tt4.to_bits() != b.tt4.to_bits()).count();
    let d_tt4 = solve.iter().zip(sensed.iter())
        .map(|(a, b)| (a.tt4 - b.tt4).abs()).fold(0.0_f64, f64::max);
    println!("PHI_JAC: {} points, {n_diff} differ, max|dTt4| = {d_tt4:e}", solve.len());
    assert_eq!(n_diff, 0, "§ 5.31 (i): the two laws march BIT-IDENTICALLY at this arm");
    assert_eq!(d_tt4, 0.0);

    // AND THE ACCEL LEG's TWO ANSWERS AT THE SAME CALL ARE NOT THE SAME NUMBER — the
    // discrimination the trajectory hid, read at the real call site through the shipped body.
    LEGS.with(|l| l.borrow_mut().clear());
    let probed = build(Rung::R76, Inj::ProbeLegs, PHI_JAC, false);
    let _ = march_at(&probed, PHI_JAC, CAP_LAW_SOLVE, false);
    let legs: Vec<(f64, f64)> = LEGS.with(|l| l.borrow().clone());
    assert!(!legs.is_empty(),
            "the probe must have SEEN the accel leg — an empty sample is the vacuity this whole \
             section exists to avoid");
    let rel: Vec<f64> = legs.iter().filter(|(sv, _)| *sv != 0.0)
        .map(|(sv, sn)| (sn - sv) / sv.abs()).collect();
    let lo = rel.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = rel.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let n_cuts = rel.iter().filter(|x| **x < 0.0).count();
    println!("PHI_JAC accel leg: {} probed calls, (sensed-solve)/|solve| in [{lo:e}, {hi:e}], \
              sensed LOWER at {n_cuts}", legs.len());
    assert_eq!(n_cuts, rel.len(),
               "THE SENSED LEG CUTS HARDER AT EVERY CALL — the shipped claim, measured at the leg \
                rather than at the plant that discards it");
    assert!(hi < 0.0, "and not one call is a tie, so the break is not an artefact of a stalled \
                       march");
}

/// **SIDE 3 — PLANT-REACHABLE, at `PHI_BOTH`, AND THE SAME INJECTION IS INVISIBLE AT THE OTHER
/// ARM.**
///
/// The two arms differ in one scalar — the surge floor the `min` is referenced to — and that
/// scalar decides whether a cell that is dispatched, and returns a different number, reaches the
/// plant at all. § 5.31 (i)'s `1.025497e+01` independently reproduces `docs/rung76-spec.md`
/// § 1.3's `1179.24 -> 1168.98 K`, so it is an ANCHOR and is asserted as one rather than as
/// `> 0`.
#[test]
fn the_trajectory_moves_only_at_phi_both_and_the_injection_is_silent_at_the_other_arm() {
    let m = build(Rung::R76, Inj::None, PHI_BOTH, false);
    let solve = march_at(&m, PHI_BOTH, CAP_LAW_SOLVE, false);
    let sensed = march_at(&m, PHI_BOTH, CAP_LAW_SENSED, false);
    assert_eq!(solve.len(), sensed.len());
    let n_diff = solve.iter().zip(sensed.iter())
        .filter(|(a, b)| a.tt4.to_bits() != b.tt4.to_bits()).count();
    let d_tt4 = solve.iter().zip(sensed.iter())
        .map(|(a, b)| (a.tt4 - b.tt4).abs()).fold(0.0_f64, f64::max);
    println!("PHI_BOTH: {} points, {n_diff} differ, max|dTt4| = {d_tt4:e}", solve.len());
    // **TWO POPULATIONS, AND § 5.31 (i)'s `341 of 341` IS THE SECOND ONE.** Counting on `Tt4`
    // alone gives 333; counting on the whole recorded point gives 341. § 5.31.4 (e) measured
    // exactly this pair (`n_diff` 333, `n_diff_tuple` 341) and named it as the slice's own
    // leading lesson happening inside the slice — a claim is about a POPULATION, and the two
    // numbers are both right about different ones. The first writing of this gate asserted 341
    // against the `Tt4` count and went red.
    let n_diff_tuple = solve.iter().zip(sensed.iter())
        .filter(|(a, b)| point_bits(a) != point_bits(b)).count();
    println!("PHI_BOTH: {n_diff} differ on Tt4, {n_diff_tuple} on the whole point");
    assert_eq!(n_diff, 333, "the `Tt4` population — § 5.31.4 (e)'s `n_diff`");
    assert_eq!(n_diff_tuple, solve.len(), "the WHOLE-POINT population — its `n_diff_tuple`, and \
                                           the `341 of 341` § 5.31 (i) publishes");
    assert_eq!(d_tt4.to_bits(), 1.0254967637311893e1_f64.to_bits(),
               "the anchor: § 5.31 (i)'s `1.025497e+01`, which independently reproduces \
                `docs/rung76-spec.md` § 1.3's `1179.24 -> 1168.98 K`");

    // **THE INJECTION, SCORED ON THE PLANT AT BOTH ARMS.** Rung 75's `sensed_cap` returns
    // `Ok(None)`, so the caller falls back to the solve — which at `PHI_JAC` is what the shipped
    // machine marches ANYWAY.
    let inj_both = build(Rung::R76, Inj::SensedCap, PHI_BOTH, false);
    let t_both = march_at(&inj_both, PHI_BOTH, CAP_LAW_SENSED, false);
    let moved_both = sensed.iter().zip(t_both.iter())
        .filter(|(a, b)| a.tt4.to_bits() != b.tt4.to_bits()).count();

    let m_jac = build(Rung::R76, Inj::None, PHI_JAC, false);
    let base_jac = march_at(&m_jac, PHI_JAC, CAP_LAW_SENSED, false);
    let inj_jac = build(Rung::R76, Inj::SensedCap, PHI_JAC, false);
    let t_jac = march_at(&inj_jac, PHI_JAC, CAP_LAW_SENSED, false);
    let moved_jac = base_jac.iter().zip(t_jac.iter())
        .filter(|(a, b)| a.tt4.to_bits() != b.tt4.to_bits()).count();

    println!("the SAME injection: PHI_BOTH moved {moved_both} of {}, PHI_JAC moved {moved_jac} \
              of {}", t_both.len(), t_jac.len());
    assert!(moved_both > 0,
            "the injection has to be visible SOMEWHERE, or § 3's silence is the only reading");
    assert_eq!(moved_jac, 0,
            "**THE TRAP**: the same dropped body is EXACTLY INVISIBLE at the other arm, and a \
             trajectory-scored gate written there would have read a zero and called the cell \
             inert. That zero is also what an UNREACHABLE cell reads, which is why side 1 counts \
             and side 2 compares the returned value.");
}

// =============================================================================================
// 5 — P6: THE UNTAGGED MESSAGE
// =============================================================================================

/// **P6 — *at least one of the three ungated shipped messages is reachable by a port defect that
/// every ported gate passes*, with rung 76's untagged one named.**
///
/// § 5.31 (v) censused nine `assert` messages across the two classes. Three carry no suite
/// `match=` needle, and **exactly one names no rung at all**: `cap_bill`'s *"the two cap laws
/// marched different grids"* (`engine.py:19523`). A port that raised a PARENT's message there
/// passes any `rung-76:` prefix check, which is the whole of P6's claim.
///
/// This gate does two things and they are different. It DRIVES the message — through an
/// injection that makes one of the two laws march a shorter grid — and it then checks that
/// `rung76.rs`'s own needle for that site would not have caught a parent's message in its place.
#[test]
fn the_untagged_message_is_reachable_and_a_prefix_check_cannot_see_it() {
    let f = flight();
    // THE CONTROL, FIRST. The shipped reader must RETURN, or every needle below is being read off
    // a fixture that was already broken.
    let m = build(Rung::R76, Inj::None, PHI_BOTH, false);
    let ok = caught(|| {
        let _ = cap_bill(&m, &f, LO, HI, TT4_MAX, PHI_BOTH, MARGIN, TAUS, TAU_T, REF_LAW_DEFAULT,
                         WINDUP_LAW_NONE, false, R, SETTLE, MX_DS, V_MAX, CAP_BILL_TAIL);
    });
    assert!(ok.is_none(), "the baseline `cap_bill` must return: {ok:?}");

    let bill = |inj: Inj| -> Option<String> {
        let bad = build(Rung::R76, inj, PHI_BOTH, false);
        caught(|| {
            let _ = cap_bill(&bad, &f, LO, HI, TT4_MAX, PHI_BOTH, MARGIN, TAUS, TAU_T,
                             REF_LAW_DEFAULT, WINDUP_LAW_NONE, false, R, SETTLE, MX_DS, V_MAX,
                             CAP_BILL_TAIL);
        })
    };
    let lens = |inj: Inj| -> (usize, usize) {
        let one = |law: &'static str| -> usize {
            let b = build(Rung::R76, inj, PHI_BOTH, false);
            let mut n = 0usize;
            let _ = caught(|| { n = march_at(&b, PHI_BOTH, law, false).len(); });
            n
        };
        (one(CAP_LAW_SOLVE), one(CAP_LAW_SENSED))
    };

    // THE TWO PLAUSIBLE DEFECTS — *does a realistic slip reach the message?*
    for (label, inj) in [("pt4 without pi_b", Inj::NoPiB), ("cap's two args swapped", Inj::Swapped)]
    {
        let (la, lb) = lens(inj);
        println!("P6 [{label}]: solve marched {la}, sensed marched {lb}, cap_bill said {:?}",
                 bill(inj));
    }

    // THE REACHABILITY SEARCH — *is the site reachable at all?* The march is
    // `for _ in 0..=n_steps` with `let Ok(k1) = der(...) else { break }` at the top, so the two
    // laws produce trajectories of DIFFERENT LENGTH exactly when one aborts a derivative
    // PART-WAY and the other does not. A defect that aborts at `s = 0` gives an empty march that
    // panics before the assert; one that never aborts gives equal lengths. The window is
    // narrow and it is SWEPT rather than guessed at.
    let mut found: Option<(f64, usize, usize, String)> = None;
    for scale in [0.999, 0.99, 0.95, 0.9, 0.8, 0.6, 0.4, 0.2, 1.05, 1.2, 1.5] {
        SCALE.with(|x| x.set(scale));
        let (la, lb) = lens(Inj::Scaled);
        let msg = bill(Inj::Scaled).unwrap_or_default();
        println!("P6 scale {scale}: solve {la}, sensed {lb}, cap_bill {msg:?}");
        if msg.contains("the two cap laws marched different grids") && found.is_none() {
            found = Some((scale, la, lb, msg));
        }
    }
    SCALE.with(|x| x.set(1.0));

    let (scale, la, lb, msg) = found.expect(
        "P6's named message was NOT REACHED by either plausible defect or by any of eleven          scalings of the shipped cap. That is a stronger finding than P6 predicted — a shipped          assert with no reachable input — and it is what this gate would have to be rewritten to          say. It is deliberately spelled as a failure rather than as a silent `if`, because a          search that quietly finds nothing is the vacuity this whole file is about.");
    println!("P6 REACHED at scale {scale}: solve {la}, sensed {lb}");
    assert_ne!(la, lb, "the message is about the two lengths, so they must actually differ");

    // **AND THE PROPERTY P6 IS ACTUALLY ABOUT.** The message names no rung, so a needle checking
    // for a `rung-76:` prefix would pass a port that raised a PARENT's message here — while every
    // other message in the class opens with one. Both halves are asserted, because the second is
    // what makes the first worth having.
    assert!(!msg.contains("rung-76:") && !msg.contains("rung-75:") && !msg.contains("rung-74:"),
            "the message under test is the UNTAGGED one: {msg:?}");
    let tagged = caught(|| {
        let mm = build(Rung::R76, Inj::None, PHI_BOTH, false);
        let _ = march_at(&mm, PHI_BOTH, "nonsense", false);
    }).unwrap_or_default();
    assert!(tagged.contains("rung-76:"),
            "THE CONTROL — a message from the same class that DOES name its rung, so *this one              does not* is a measurement and not a property of how the needle was written:              {tagged:?}");
}
