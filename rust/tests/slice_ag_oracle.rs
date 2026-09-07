//! SLICE AG step 6 (b) — **THE ORACLE for rungs 75 AND 76**, against PyPy *and* CPython.
//!
//! `rung75.rs` and `rung76.rs` port the two suites' 33 gates and add three. Those gates are
//! RELATIONS — *the pole leaves the origin*, *`det J` scales by roughly `1 - c`*, *the sensed leg
//! cuts harder*. This file is the VALUE seat: every number every reader publishes, plus the
//! plants underneath them, compared bit for bit against a Python golden.
//!
//! # THE THREE THINGS THE 36 GATES CANNOT DO
//!
//! 1. **Say how much moved.** A gate answers caught / not caught. Steps 3 and 5 scored their
//!    mutation sweeps on throwaway harnesses that were DELETED with the step; this file is that
//!    seat made shipped, so the ledger survives the session that measured it.
//! 2. **Reach the plant per point.** A–G are the readers' AGGREGATES over marches, and the two
//!    reduce spines compare NINE or ELEVEN of a marched point's THIRTY-FIVE recorded fields. AD
//!    step 5 found six drifting keys at 2 points of 1 302 only because the equivalent of section
//!    H existed.
//! 3. **See a SIGN.** Rungs 74/75/76's `g_fuel`/`required_*` are unfloored projections and go
//!    NEGATIVE. No aggregate in A–G records that; `H/*/neg/*` counts it per column per arm.
//!
//! # THE KEY SET IS **WALKED** ON THE PYTHON SIDE AND **HAND-LISTED** HERE, ON PURPOSE
//!
//! `dump_slice_ag.py` names no keys: it descends the returned dict and emits whatever is in it.
//! This file has structs and no reflection, so it hand-lists. **The asymmetry is the point** — a
//! key present in the Python reading and forgotten in BOTH emitters is invisible to every check
//! either side runs, which is [[rust-port-documented-gate-that-doesnt-exist]] in its
//! oracle-shaped form. Here a forgotten key is a MISSING key, and
//! [`the_two_key_sets_are_equal`] fails by name before a single value is compared.
//!
//! It only works in this direction. A generic Rust walker would reintroduce the shared blindness,
//! which is why one was not written even though it would have been shorter.
//!
//! # THE CELL KEYS CARRY A FLOAT FORMATTED AS PYTHON FORMATS IT
//!
//! `windup_gains` and `device_control` key their cells `f"{ref}|{tau_t}"` — a `f64` rendered by
//! Python's shortest-repr. Rust's `{}` for `f64` is also shortest-round-trip, so the ten clocks
//! on the shipped grids (`0.00625` … `0.4`) render identically. **That is an agreement between
//! two implementations, not a guarantee**, and it is exactly the kind of thing the key-set
//! equality check exists to catch: a divergence would appear as two missing and two extra keys
//! rather than as a wrong number. `sensed_cap.rs`'s own doc comment records why `CapGains` keys
//! its cells with three STRINGS instead — the same hazard, avoided at the source where it could
//! be.
//!
//! # WHAT IS COMPARED, AND WHAT IS ONLY COUNTED
//!
//! Every key here is COMPUTED by the port and compared. **Nothing is read out of the golden as an
//! input**: this slice replays no captured argument, so the success line's number is a count of
//! comparisons with a declared-read term of exactly zero. AE step 4's success line had been
//! calling 5 726 golden READS "values compared"; the distinction is kept alive by stating that
//! the term is zero rather than by omitting it.

use std::collections::{BTreeMap, BTreeSet};

use turbojet::anti_windup::{
    build_anti_windup_cascade, contraction_law, device_control, windup_bill, windup_gains,
    windup_march, BillAccident, BillRow, ContractionLaw, ContractionRow, DeviceCell,
    DeviceControl, WindupBill, WindupCell, WindupCellRead, WindupGains, WindupRatio, WindupRow,
    WINDUP_LAW_NONE, WINDUP_LAW_TRACK,
};
use turbojet::applied_reference::REF_LAW_APPLIED;
use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{FuelPoint, PointExtra};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::reference_split::StatorIncidenceLimiter;
use turbojet::sensed_cap::{
    accel_for, build_sensed_cap_cascade, cap_bill, cap_gains, cap_march, solve_gain, CapBill,
    CapCell, CapCellRead, CapGains, SolveGain, SolveGainRow, CAP_BILL_TAIL, CAP_BILL_TAU_T,
    CAP_GAINS_LAWS, CAP_GAINS_REFS, CAP_LAW_SENSED, CAP_LAW_SOLVE, SOLVE_GAIN_DQ,
    SOLVE_GAIN_EVERY, SOLVE_GAIN_REF,
};
use turbojet::shared_actuator::REF_LAW_DEFAULT;
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE_PYPY: &str = include_str!("../oracle/slice_ag_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/slice_ag_cpython.tsv");

// ============================================================================ the grid
//
// The two suites' shared module constants, verbatim.

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
const PHI_BOTH: f64 = 0.76;
const TAU_T: f64 = 0.05;
const TAU_T_FAST: f64 = 0.0125;
const MARGIN: f64 = 0.10;
const MARGIN_HI: f64 = 0.20;

// -------------------------------------------------- THE READERS' OWN DEFAULTS, NAMED
//
// The dumper passes NONE of these — it relies on Python's `def` line, so on this side they are
// typed at a call site with no signature to check them against. § 5.31.5 (l)'s rule, and the
// same reason it named seven constants in `sensed_cap.rs`: a bare literal at a call site is a
// number nobody can check. Every line is verified by `tests/test_rust_line_citations.py`.

/// `windup_gains(..., tau_ts=(0.05, 0.0125), refs=("applied", "sched"))` —
/// `engine.py:18902` / `18903`.
const WG_TAU_TS: [f64; 2] = [TAU_T, TAU_T_FAST];
const WG_REFS: [&str; 2] = [REF_LAW_APPLIED, REF_LAW_DEFAULT];
/// `contraction_law(..., tau_ts=(0.4, 0.2, 0.1, 0.05, 0.025, 0.0125), res0=2.898e-3,
/// tol=1e-12, ic_cap=400)` — `engine.py:18994` / `18995`. **WIDER than the suite's four**, and
/// the two fastest are where `measured` legitimately comes back `None`.
const CL_TAU_TS: [f64; 6] = [0.4, 0.2, 0.1, 0.05, 0.025, 0.0125];
const CL_RES0: f64 = 2.898e-3;
const CL_TOL: f64 = 1e-12;
const CL_IC_CAP: usize = 400;
/// `device_control(..., tau_ts=(0.05, 0.0125), refs=("sched", "applied"))` — `engine.py:19045` /
/// `19046`. **The reference order is the OPPOSITE of `windup_gains`'**, which changes no key here
/// (cells are looked up by name) and would change every ratio in a reader that paired them.
const DC_TAU_TS: [f64; 2] = [TAU_T, TAU_T_FAST];
const DC_REFS: [&str; 2] = [REF_LAW_DEFAULT, REF_LAW_APPLIED];
/// `windup_bill(..., tau_ts=(0.00625, …, 0.1), ref="applied")` — `engine.py:19091` / `19092`.
/// **`tau_ts[0]` IS the RK4 admissibility floor** at this grid, so this grid runs ON its own
/// boundary and whether `_rk4_floor_shared` admits equality is a value, not a detail.
const WB_TAU_TS: [f64; 8] = [0.00625, 0.0125, 0.025, 0.05, 0.0625, 0.075, 0.0875, 0.1];
const WB_REF: &str = REF_LAW_APPLIED;
/// The stride every gains reader defaults to.
const EVERY: usize = 8;

// ============================================================================ the value tree
//
// Python's `walk()`, mirrored. See the module header for why only ONE side is generic.

#[derive(Clone, Debug)]
enum V {
    Null,
    B(bool),
    I(u64),
    F(f64),
    S(String),
    L(Vec<V>),
    M(Vec<(&'static str, V)>),
}

fn vf(x: f64) -> V { V::F(x) }
fn vi(n: usize) -> V { V::I(n as u64) }
fn vb(x: bool) -> V { V::B(x) }
fn vs(x: &str) -> V { V::S(x.to_string()) }
fn vopt_f(x: Option<f64>) -> V { x.map(V::F).unwrap_or(V::Null) }
fn vopt_i(x: Option<usize>) -> V { x.map(vi).unwrap_or(V::Null) }
/// Python's `(a, b)` — a TUPLE, which `walk` treats as a list.
fn vpair(a: f64, b: f64) -> V { V::L(vec![V::F(a), V::F(b)]) }
fn vopt_pair(x: Option<(f64, f64)>) -> V {
    x.map(|(a, b)| vpair(a, b)).unwrap_or(V::Null)
}
fn vtaus(t: (f64, f64, f64, f64)) -> V {
    V::L(vec![V::F(t.0), V::F(t.1), V::F(t.2), V::F(t.3)])
}
fn vfloats(xs: &[f64]) -> V { V::L(xs.iter().copied().map(V::F).collect()) }

fn fnv(text: &str) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for byte in text.as_bytes() {
        h ^= *byte as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

/// The emitted map, and **A SET OF WHICH KEYS CARRY A FLOAT, RECORDED AS THEY ARE EMITTED.**
///
/// The first spelling of this decided floatness AFTER THE FACT, from the key NAME, against a
/// hand-typed list of integer leaves. It over-counted by 173 and `Z/n_pos_zero` caught it —
/// which is the defence working, and also the point: a census that classifies its own
/// population by pattern-matching the names is [[rust-port-guessed-census-bars]] again, and the
/// list would have needed a new entry every time a reader grew an integer field. Python does not
/// guess; it counts inside `f()`, at the one place a float is written. So does this now: `raw_f`
/// and `V::F` are the only two float paths and both record the key. The two sides then count the
/// SAME POPULATION by the same rule rather than agreeing about a list.
struct Emit {
    map: BTreeMap<String, u64>,
    floats: BTreeSet<String>,
}

impl Emit {
    fn put(&mut self, key: String, v: u64) {
        assert!(self.map.insert(key.clone(), v).is_none(), "DUPLICATE KEY {key}");
    }

    fn put_f(&mut self, key: String, x: f64) {
        self.floats.insert(key.clone());
        self.put(key, x.to_bits());
    }

    fn raw_f(&mut self, key: &str, x: f64) { self.put_f(key.to_string(), x); }
    fn raw_d(&mut self, key: &str, n: usize) { self.put(key.to_string(), n as u64); }
    fn raw_b(&mut self, key: &str, x: bool) { self.put(key.to_string(), u64::from(x)); }

    /// Python's `walk()`, key for key.
    fn walk(&mut self, key: &str, v: &V) {
        self.put(format!("{key}?"), u64::from(!matches!(v, V::Null)));
        match v {
            V::Null => {}
            V::B(x) => self.put(key.to_string(), u64::from(*x)),
            V::I(n) => self.put(key.to_string(), *n),
            V::F(x) => self.put_f(key.to_string(), *x),
            V::S(s) => self.put(key.to_string(), fnv(s)),
            V::L(xs) => {
                self.put(format!("{key}/#len"), xs.len() as u64);
                for (i, x) in xs.iter().enumerate() {
                    self.walk(&format!("{key}/{i}"), x);
                }
            }
            V::M(kv) => {
                self.put(format!("{key}/#len"), kv.len() as u64);
                for (k, x) in kv {
                    self.walk(&format!("{key}/{k}"), x);
                }
            }
        }
    }

    /// A Python dict whose KEYS are computed (`cells`), which `V::M`'s `&'static str` cannot
    /// hold. Walked directly so the two shapes stay identical.
    fn walk_cells(&mut self, key: &str, cells: &[(String, V)]) {
        self.put(format!("{key}?"), 1);
        self.put(format!("{key}/#len"), cells.len() as u64);
        for (k, v) in cells {
            self.walk(&format!("{key}/{k}"), v);
        }
    }
}

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

/// The dumper's `rig(cls, phi_lim, inc)` — the RECEIVER, with the four knobs Python's fixture
/// sets by plain assignment.
fn windup_rig(phi_lim: f64, inc: bool) -> ScheduledStatorCore {
    let sm = phi_lim / FLOOR - 1.0;
    let m = full_of(build_anti_windup_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &suite_arm(sm, inc)));
    m.fuel.inner.lag_coord.set("demand");
    m.fuel.inner.ref_law.set(REF_LAW_DEFAULT);
    m.fuel.inner.windup_law.set(WINDUP_LAW_NONE);
    m.fuel.inner.tau_t.set(None);
    m
}

fn cap_rig(phi_lim: f64, inc: bool) -> ScheduledStatorCore {
    let sm = phi_lim / FLOOR - 1.0;
    let m = full_of(build_sensed_cap_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &suite_arm(sm, inc)));
    m.fuel.inner.lag_coord.set("demand");
    m.fuel.inner.ref_law.set(REF_LAW_DEFAULT);
    m.fuel.inner.windup_law.set(WINDUP_LAW_NONE);
    m.fuel.inner.tau_t.set(None);
    m.fuel.inner.cap_law.set(CAP_LAW_SOLVE);
    m
}

/// Python's `f"{ref}|{tau_t}"`. See the module header on the float rendering.
fn cell_key(ref_law: &str, tau_t: f64) -> String { format!("{ref_law}|{tau_t}") }

// ============================================================================ the converters

fn v_windup_row(r: &WindupRow) -> V {
    V::M(vec![
        ("s", vf(r.s)),
        ("auth", vs(r.auth.as_str())),
        ("masked", vs(r.masked.as_str())),
        ("tau_masked", vf(r.tau_masked)),
        ("masked_diag", vf(r.masked_diag)),
        ("masked_diag0", vf(r.masked_diag0)),
        ("auth_diag", vf(r.auth_diag)),
        ("auth_diag0", vf(r.auth_diag0)),
        ("row_auth", vf(r.row_auth)),
        ("row_auth0", vf(r.row_auth0)),
        ("mask_leak", vf(r.mask_leak)),
        ("mask_leak0", vf(r.mask_leak0)),
        ("track_leak", vf(r.track_leak)),
        ("det", vf(r.det)),
        ("det0", vf(r.det0)),
        ("zeros", vi(r.zeros)),
        ("zeros0", vi(r.zeros0)),
    ])
}

fn v_windup_cell_read(c: &WindupCellRead) -> V {
    V::M(vec![
        ("n", vi(c.n)),
        ("n_riding", vi(c.n_riding)),
        ("tau_t", vf(c.tau_t)),
        ("ref", vs(c.ref_law)),
        ("masked_diag", vpair(c.masked_diag.0, c.masked_diag.1)),
        ("masked_diag0", vpair(c.masked_diag0.0, c.masked_diag0.1)),
        ("diag_err", vf(c.diag_err)),
        ("auth_diag_moved", vf(c.auth_diag_moved)),
        ("track_leak", vf(c.track_leak)),
        ("mask_leak", vf(c.mask_leak)),
        ("mask_leak0", vf(c.mask_leak0)),
        ("det", vpair(c.det.0, c.det.1)),
        ("det0", vpair(c.det0.0, c.det0.1)),
        ("det_alive", vf(c.det_alive)),
        ("det0_alive", vf(c.det0_alive)),
        ("zeros", V::L(c.zeros.iter().map(|z| vi(*z)).collect())),
        ("zeros0", V::L(c.zeros0.iter().map(|z| vi(*z)).collect())),
        ("row_err", vf(c.row_err)),
        ("row_auth0", vpair(c.row_auth0.0, c.row_auth0.1)),
        ("rows", V::L(c.rows.iter().map(v_windup_row).collect())),
    ])
}

fn v_windup_cell(c: &WindupCell) -> V {
    match c {
        // Python's EMPTY cell is `dict(n=0, n_riding=n)` — TWO keys, not the full twenty.
        WindupCell::Empty { n_riding } =>
            V::M(vec![("n", vi(0)), ("n_riding", vi(*n_riding))]),
        WindupCell::Read(b) => v_windup_cell_read(b),
    }
}

fn v_ratio(r: &WindupRatio) -> V {
    V::M(vec![
        ("n", vi(r.n)),
        ("diag", vpair(r.diag.0, r.diag.1)),
        ("det", vpair(r.det.0, r.det.1)),
    ])
}

fn emit_windup_gains(e: &mut Emit, key: &str, g: &WindupGains) {
    e.put(format!("{key}?"), 1);
    e.put(format!("{key}/#len"), 7);
    e.walk(&format!("{key}/phi_lim"), &vf(g.phi_lim));
    e.walk(&format!("{key}/taus"), &vtaus(g.taus));
    e.walk(&format!("{key}/tau_ts"), &vfloats(&g.tau_ts));
    e.walk(&format!("{key}/inc"), &vb(g.inc));
    e.walk(&format!("{key}/ds"), &vf(g.ds));
    let cells: Vec<(String, V)> = g.cells.iter()
        .map(|((r, t), c)| (cell_key(r, *t), v_windup_cell(c))).collect();
    e.walk_cells(&format!("{key}/cells"), &cells);
    let ratios: Vec<(String, V)> = g.ratios.iter()
        .map(|r| (r.ref_law.to_string(), v_ratio(r))).collect();
    e.walk_cells(&format!("{key}/ratios"), &ratios);
}

fn v_contraction_row(r: &ContractionRow) -> V {
    V::M(vec![
        ("tau_t", vf(r.tau_t)),
        ("sigma", vf(r.sigma)),
        // Python's `math.ceil` returns an `int`, and it can be NEGATIVE where `sigma` is small
        // enough that the log ratio flips — emitted as a signed value re-read as bits, which is
        // what `d()` does on the Python side for a negative too.
        ("predicted", V::I(r.predicted as u64)),
        ("measured", vopt_i(r.measured)),
        ("res", vopt_f(r.res)),
        ("within_inherited_cap", vb(r.within_inherited_cap)),
    ])
}

fn emit_contraction(e: &mut Emit, key: &str, c: &ContractionLaw) {
    e.put(format!("{key}?"), 1);
    e.put(format!("{key}/#len"), 9);
    e.walk(&format!("{key}/phi_lim"), &vf(c.phi_lim));
    e.walk(&format!("{key}/taus"), &vtaus(c.taus));
    e.walk(&format!("{key}/res0"), &vf(c.res0));
    e.walk(&format!("{key}/tol"), &vf(c.tol));
    e.walk(&format!("{key}/ic_cap"), &vi(c.ic_cap));
    e.walk(&format!("{key}/rows"),
           &V::L(c.rows.iter().map(v_contraction_row).collect()));
    e.walk(&format!("{key}/n"), &vi(c.n));
    e.walk(&format!("{key}/n_exact"), &vi(c.n_exact));
    e.walk(&format!("{key}/all_exact"), &vb(c.all_exact));
}

fn v_device_cell(c: &DeviceCell) -> V {
    V::M(vec![
        ("ref", vs(c.ref_law)),
        ("tau_t", vf(c.tau_t)),
        ("n_dormant", vi(c.n_dormant)),
        ("n_cutting", vi(c.n_cutting)),
        ("dormant_output", vopt_f(c.dormant_output)),
        ("dormant_state", vopt_f(c.dormant_state)),
        ("cutting_output", vopt_f(c.cutting_output)),
    ])
}

fn emit_device_control(e: &mut Emit, key: &str, d: &DeviceControl) {
    e.put(format!("{key}?"), 1);
    e.put(format!("{key}/#len"), 4);
    e.walk(&format!("{key}/phi_lim"), &vf(d.phi_lim));
    e.walk(&format!("{key}/taus"), &vtaus(d.taus));
    e.walk(&format!("{key}/inc"), &vb(d.inc));
    let cells: Vec<(String, V)> = d.cells.iter()
        .map(|c| (cell_key(c.ref_law, c.tau_t), v_device_cell(c))).collect();
    e.walk_cells(&format!("{key}/cells"), &cells);
}

fn v_bill_row(r: &BillRow) -> V {
    V::M(vec![
        ("tau_t", vf(r.tau_t)),
        ("ratio", vf(r.ratio)),
        ("max_Tt4", vf(r.max_tt4)),
        ("over", vf(r.over)),
        ("holds", vb(r.holds)),
        ("min_phi", vf(r.min_phi)),
        ("handover", vopt_f(r.handover)),
    ])
}

fn v_accident(a: &BillAccident) -> V {
    V::M(vec![
        ("max_Tt4", vf(a.max_tt4)),
        ("over", vf(a.over)),
        ("min_phi", vf(a.min_phi)),
        ("handover", vopt_f(a.handover)),
    ])
}

fn emit_windup_bill(e: &mut Emit, key: &str, b: &WindupBill) {
    e.put(format!("{key}?"), 1);
    e.put(format!("{key}/#len"), 15);
    e.walk(&format!("{key}/phi_lim"), &vf(b.phi_lim));
    e.walk(&format!("{key}/taus"), &vtaus(b.taus));
    e.walk(&format!("{key}/ref"), &vs(b.ref_law));
    e.walk(&format!("{key}/inc"), &vb(b.inc));
    e.walk(&format!("{key}/ds"), &vf(b.ds));
    e.walk(&format!("{key}/rows"), &V::L(b.rows.iter().map(v_bill_row).collect()));
    e.walk(&format!("{key}/accident"), &v_accident(&b.accident));
    e.walk(&format!("{key}/tau_t_holds"), &vopt_f(b.tau_t_holds));
    e.walk(&format!("{key}/tau_t_breaks"), &vopt_f(b.tau_t_breaks));
    e.walk(&format!("{key}/ratio_holds"), &vopt_f(b.ratio_holds));
    e.walk(&format!("{key}/ratio_breaks"), &vopt_f(b.ratio_breaks));
    e.walk(&format!("{key}/handover_monotone"), &vb(b.handover_monotone));
    e.walk(&format!("{key}/handover_span"), &vopt_pair(b.handover_span));
    // Python's `((min, max), hand(acc))` — a PAIR whose second element can itself be `None`.
    e.walk(&format!("{key}/handover_vs_accident"),
           &match b.handover_vs_accident {
               None => V::Null,
               Some((span, acc)) => V::L(vec![vpair(span.0, span.1), vopt_f(acc)]),
           });
    e.walk(&format!("{key}/Tt4_monotone"), &vb(b.tt4_monotone));
}

fn v_cap_cell(c: &CapCell) -> V {
    match c {
        CapCell::Empty { n_riding, ref_law, law, auth, n_inert } => V::M(vec![
            ("n", vi(0)),
            ("n_riding", vi(*n_riding)),
            ("ref", vs(ref_law)),
            ("law", vs(law)),
            ("auth", vs(auth.as_str())),
            ("n_inert", vi(*n_inert)),
        ]),
        CapCell::Read(b) => v_cap_cell_read(b),
    }
}

fn v_cap_cell_read(c: &CapCellRead) -> V {
    V::M(vec![
        ("n", vi(c.n)),
        ("n_riding", vi(c.n_riding)),
        ("ref", vs(c.ref_law)),
        ("law", vs(c.law)),
        ("auth", vs(c.auth.as_str())),
        ("n_inert", vi(c.n_inert)),
        ("c", vpair(c.c.0, c.c.1)),
        ("auth_diag", vpair(c.auth_diag.0, c.auth_diag.1)),
        ("auth_diag0", vpair(c.auth_diag0.0, c.auth_diag0.1)),
        ("auth_moved", vf(c.auth_moved)),
        ("auth_err", vf(c.auth_err)),
        ("masked_diag", vpair(c.masked_diag.0, c.masked_diag.1)),
        ("masked_moved", vf(c.masked_moved)),
        ("row_auth", vpair(c.row_auth.0, c.row_auth.1)),
        ("row_auth0", vpair(c.row_auth0.0, c.row_auth0.1)),
        ("row_err", vopt_f(c.row_err)),
        ("mask_leak", vf(c.mask_leak)),
        ("mask_leak0", vf(c.mask_leak0)),
        ("det", vpair(c.det.0, c.det.1)),
        ("det0", vpair(c.det0.0, c.det0.1)),
        ("det_ratio", vopt_pair(c.det_ratio)),
        ("det_err", vopt_f(c.det_err)),
        ("zeros", V::L(vec![vi(c.zeros.0), vi(c.zeros.1)])),
        ("zeros0", V::L(vec![vi(c.zeros0.0), vi(c.zeros0.1)])),
        ("zeros_moved", vi(c.zeros_moved)),
        ("gov_row", vf(c.gov_row)),
    ])
}

fn emit_cap_gains(e: &mut Emit, key: &str, g: &CapGains) {
    e.put(format!("{key}?"), 1);
    e.put(format!("{key}/#len"), 7);
    e.walk(&format!("{key}/phi_lim"), &vf(g.phi_lim));
    e.walk(&format!("{key}/margin"), &vf(g.margin));
    e.walk(&format!("{key}/taus"), &vtaus(g.taus));
    e.walk(&format!("{key}/tau_t"), &vf(g.tau_t));
    e.walk(&format!("{key}/inc"), &vb(g.inc));
    e.walk(&format!("{key}/ds"), &vf(g.ds));
    let cells: Vec<(String, V)> = g.cells.iter()
        .map(|(k, c)| (k.clone(), v_cap_cell(c))).collect();
    e.walk_cells(&format!("{key}/cells"), &cells);
}

/// `cap_bill` MINUS its `traj` key, whose two marches are section H's. Their LENGTHS are emitted,
/// because the reader asserts the two grids match and a pair that both went empty would satisfy
/// every fold above it.
fn emit_cap_bill(e: &mut Emit, key: &str, b: &CapBill) {
    e.raw_d(&format!("{key}/traj/solve/n"), b.traj_solve.len());
    e.raw_d(&format!("{key}/traj/sensed/n"), b.traj_sensed.len());
    e.put(format!("{key}?"), 1);
    e.put(format!("{key}/#len"), 14);
    e.walk(&format!("{key}/phi_lim"), &vf(b.phi_lim));
    e.walk(&format!("{key}/margin"), &vf(b.margin));
    e.walk(&format!("{key}/ref"), &vs(b.ref_law));
    e.walk(&format!("{key}/law"), &vs(b.law));
    e.walk(&format!("{key}/inc"), &vb(b.inc));
    e.walk(&format!("{key}/ds"), &vf(b.ds));
    e.walk(&format!("{key}/n"), &vi(b.n));
    e.walk(&format!("{key}/s_tail"), &vf(b.s_tail));
    e.walk(&format!("{key}/max_Tt4"), &vpair(b.max_tt4.0, b.max_tt4.1));
    e.walk(&format!("{key}/min_phi"), &vpair(b.min_phi.0, b.min_phi.1));
    e.walk(&format!("{key}/fuel_int"), &vpair(b.fuel_int.0, b.fuel_int.1));
    e.walk(&format!("{key}/wf_tail"), &vopt_f(b.wf_tail));
    e.walk(&format!("{key}/wf_ramp"), &vopt_f(b.wf_ramp));
    e.walk(&format!("{key}/cuts_harder"), &vb(b.cuts_harder));
}

fn v_solve_row(r: &SolveGainRow) -> V {
    V::M(vec![
        ("s", vf(r.s)),
        ("cap_solve", vf(r.cap_solve)),
        ("c", vf(r.c)),
        ("fixed_point", vf(r.fixed_point)),
        // PYTHON's SPELLING, not the port's: the struct calls these `d_s`/`d_d`, the dict calls
        // them `dS`/`dD`, and the KEY SET is what the two sides are compared on.
        ("dS", vf(r.d_s)),
        ("dD", vf(r.d_d)),
        ("gain", vf(r.gain)),
        ("predicted", vf(r.predicted)),
    ])
}

fn emit_solve_gain(e: &mut Emit, key: &str, g: &SolveGain) {
    e.put(format!("{key}?"), 1);
    e.put(format!("{key}/#len"), 9);
    e.walk(&format!("{key}/phi_lim"), &vf(g.phi_lim));
    e.walk(&format!("{key}/margin"), &vf(g.margin));
    e.walk(&format!("{key}/ref"), &vs(g.ref_law));
    e.walk(&format!("{key}/inc"), &vb(g.inc));
    e.walk(&format!("{key}/n"), &vi(g.n));
    e.walk(&format!("{key}/rows"), &V::L(g.rows.iter().map(v_solve_row).collect()));
    e.walk(&format!("{key}/fixed_point"), &vopt_f(g.fixed_point));
    e.walk(&format!("{key}/gain"), &vopt_pair(g.gain));
    e.walk(&format!("{key}/gain_err"), &vopt_f(g.gain_err));
}

// ---------------------------------------------------------------- section H's per-point fields

/// The dumper's `H_FLOATS`, in its order. Every one is present on a rung-75/76 marched point, so
/// none of them is `None` here — but the presence FLAG is emitted anyway, because Python's
/// `p.get(k)` would return `None` on a trajectory that stopped recording one and the two sides
/// have to disagree about that rather than both stay silent.
fn h_floats(p: &FuelPoint) -> Vec<(&'static str, f64)> {
    let (g, required, b, b_cmd, v, v_cmd, ic_res, g_fuel, g_gov, required_fuel, required_gov,
         w_fuel, w_gov, cap_fuel, cap_gov) = match p.extra {
        PointExtra::Demand { g, required, b, b_cmd, v, v_cmd, ic_res, g_fuel, g_gov,
                             required_fuel, required_gov, w_fuel, w_gov, cap_fuel, cap_gov, .. } =>
            (g, required, b, b_cmd, v, v_cmd, ic_res, g_fuel, g_gov, required_fuel, required_gov,
             w_fuel, w_gov, cap_fuel, cap_gov),
        _ => panic!("section H marches the DEMAND coordinate at both rungs."),
    };
    vec![
        ("s", p.s), ("nu_lp", p.nu_lp), ("nu_hp", p.nu_hp), ("Tt4", p.tt4), ("f", p.f),
        ("pi_lpc", p.pi_lpc), ("pi_hpc", p.pi_hpc), ("phi_lp", p.phi_lp), ("phi_hp", p.phi_hp),
        ("mdot_air", p.mdot_air), ("sp_thrust", p.sp_thrust), ("mf", p.mf),
        ("mf_sched", p.mf_sched), ("g", g), ("required", required), ("b", b), ("b_cmd", b_cmd),
        ("v", v), ("v_cmd", v_cmd), ("ic_res", ic_res), ("g_fuel", g_fuel), ("g_gov", g_gov),
        ("required_fuel", required_fuel), ("required_gov", required_gov),
        ("w_fuel", w_fuel), ("w_gov", w_gov), ("cap_fuel", cap_fuel), ("cap_gov", cap_gov),
    ]
}

fn h_strs(p: &FuelPoint) -> Vec<(&'static str, Option<String>)> {
    let (authority, share_law, ic_order, v_regime) = match p.extra {
        PointExtra::Demand { authority, share_law, ic_order, v_regime, .. } =>
            (authority, share_law, ic_order, v_regime),
        _ => unreachable!(),
    };
    let lag_coord = match p.extra {
        PointExtra::Demand { lag_coord, .. } => lag_coord,
        _ => unreachable!(),
    };
    vec![
        ("branch", Some(p.branch.label().to_string())),
        ("authority", Some(authority.as_str().to_string())),
        ("share_law", Some(share_law.to_string())),
        ("ic_order", Some(ic_order.to_string())),
        ("lag_coord", Some(lag_coord.to_string())),
        ("v_regime", v_regime.map(|r| regime_str(r).to_string())),
    ]
}

/// Python stores the valve regime as its own string; the port has an enum with no `as_str`, so
/// the mapping is spelled HERE rather than added to the library for the oracle's convenience.
fn regime_str(r: turbojet::limited_bleed::Regime) -> &'static str {
    use turbojet::limited_bleed::Regime;
    match r {
        Regime::Dormant => "dormant",
        Regime::Riding => "riding",
        Regime::Saturated => "saturated",
    }
}

fn ic_iters(p: &FuelPoint) -> usize {
    match p.extra {
        PointExtra::Demand { ic_iters, .. } => ic_iters,
        _ => unreachable!(),
    }
}

const H_STRIDE: usize = 5;

fn emit_march(e: &mut Emit, tag: &str, traj: &[FuelPoint]) {
    e.raw_d(&format!("{tag}/n"), traj.len());
    assert!(!traj.is_empty(), "an empty march is not a measurement: {tag}");
    e.raw_d(&format!("{tag}/keys"), traj[0].key_count());
    for (i, p) in traj.iter().enumerate().step_by(H_STRIDE) {
        let q = format!("{tag}/p{i}");
        for (k, x) in h_floats(p) {
            e.walk(&format!("{q}/{k}"), &vf(x));
        }
        e.walk(&format!("{q}/ic_iters"), &vi(ic_iters(p)));
        for (k, x) in h_strs(p) {
            e.walk(&format!("{q}/{k}"), &x.map(|s| V::S(s)).unwrap_or(V::Null));
        }
    }
    let names: Vec<&'static str> = h_floats(&traj[0]).iter().map(|(k, _)| *k).collect();
    for (ci, name) in names.iter().enumerate() {
        let col: Vec<f64> = traj.iter().map(|p| h_floats(p)[ci].1).collect();
        e.raw_d(&format!("{tag}/col/{name}/n"), col.len());
        e.raw_f(&format!("{tag}/col/{name}/min"),
                col.iter().copied().fold(f64::INFINITY, f64::min));
        e.raw_f(&format!("{tag}/col/{name}/max"),
                col.iter().copied().fold(f64::NEG_INFINITY, f64::max));
        e.raw_f(&format!("{tag}/col/{name}/last"), *col.last().expect("non-empty"));
    }
    for name in ["g_fuel", "g_gov", "required_fuel", "required_gov"] {
        let ci = names.iter().position(|k| *k == name).expect("a named column");
        e.raw_d(&format!("{tag}/neg/{name}"),
                traj.iter().filter(|p| h_floats(p)[ci].1 < 0.0).count());
    }
}

// ============================================================================ the drive

fn arms(e: &mut Emit, tag: &str, inc: bool) -> String {
    let p = format!("{tag}/i{}", u8::from(inc));
    e.raw_b(&format!("{p}/arm_inc"), inc);
    p
}

fn drive() -> BTreeMap<String, u64> {
    let mut e = Emit { map: BTreeMap::new(), floats: BTreeSet::new() };
    let fl = flight();

    // --- A: `windup_gains`, the reader's own defaults plus the suite's REVERSED tuple
    for inc in [false, true] {
        let p = arms(&mut e, "A", inc);
        let m = windup_rig(PHI_JAC, inc);
        emit_windup_gains(&mut e, &format!("{p}/fwd"), &windup_gains(
            &m, &fl, LO, HI, TT4_MAX, PHI_JAC, TAUS, &WG_TAU_TS, &WG_REFS, inc, R, SETTLE, DS,
            V_MAX, EVERY));
        emit_windup_gains(&mut e, &format!("{p}/rev"), &windup_gains(
            &m, &fl, LO, HI, TT4_MAX, PHI_JAC, TAUS, &[TAU_T_FAST, TAU_T], &WG_REFS, inc, R,
            SETTLE, DS, V_MAX, EVERY));
    }

    // --- B: `contraction_law`, the six-clock default
    for inc in [false, true] {
        let p = arms(&mut e, "B", inc);
        emit_contraction(&mut e, &p, &contraction_law(
            &windup_rig(PHI_BOTH, inc), &fl, LO, HI, TT4_MAX, PHI_BOTH, TAUS, &CL_TAU_TS,
            CL_RES0, CL_TOL, CL_IC_CAP, inc, R, SETTLE, DS, V_MAX));
    }

    // --- C: `device_control`
    for inc in [false, true] {
        let p = arms(&mut e, "C", inc);
        emit_device_control(&mut e, &p, &device_control(
            &windup_rig(PHI_BOTH, inc), &fl, LO, HI, TT4_MAX, PHI_BOTH, TAUS, &DC_TAU_TS,
            &DC_REFS, inc, R, SETTLE, DS, V_MAX));
    }

    // --- D: `windup_bill`, the eight-clock default that STARTS on the RK4 floor
    for inc in [false, true] {
        let p = arms(&mut e, "D", inc);
        emit_windup_bill(&mut e, &p, &windup_bill(
            &windup_rig(PHI_BOTH, inc), &fl, LO, HI, TT4_MAX, PHI_BOTH, TAUS, &WB_TAU_TS,
            WB_REF, inc, R, SETTLE, DS, V_MAX));
    }

    // --- E: `cap_gains`, at BOTH margins
    for inc in [false, true] {
        let a = arms(&mut e, "E", inc);
        for (mi, margin) in [MARGIN, MARGIN_HI].into_iter().enumerate() {
            let p = format!("{a}/m{mi}");
            e.raw_f(&format!("{p}/grid_margin"), margin);
            emit_cap_gains(&mut e, &p, &cap_gains(
                &cap_rig(PHI_JAC, inc), &fl, LO, HI, TT4_MAX, PHI_JAC, margin, TAUS, TAU_T,
                &CAP_GAINS_REFS, &CAP_GAINS_LAWS, inc, R, SETTLE, DS, V_MAX, EVERY));
        }
    }

    // --- F: `cap_bill`, both references. P2's two keys live here.
    //
    // **THE TWO ARMS CARRY DIFFERENT WINDUP LAWS, AND THIS IS NOT A CHOICE.** `cap_bill`
    // marches the DEMAND coordinate directly -- unlike `cap_rows` and `solve_gain`, which
    // march the CLIP plant -- so `("applied", "none")` is rung 74 s 4's cell with NO
    // INTERIOR EQUILIBRIUM and the march REFUSES. Both sides found that out the same way,
    // at the same residual: `2.898e-03 after 60 iterations`, rung 74's own reported number.
    // The applied arm is reachable ONLY with rung 75's device armed, hence `track`.
    // Widening a grid along one axis can require ARMING A DIFFERENT ONE.
    for inc in [false, true] {
        let a = arms(&mut e, "F", inc);
        for (ref_law, law) in [(REF_LAW_DEFAULT, WINDUP_LAW_NONE),
                               (REF_LAW_APPLIED, WINDUP_LAW_TRACK)] {
            emit_cap_bill(&mut e, &format!("{a}/{ref_law}"), &cap_bill(
                &cap_rig(PHI_BOTH, inc), &fl, LO, HI, TT4_MAX, PHI_BOTH, MARGIN, TAUS,
                CAP_BILL_TAU_T, ref_law, law, inc, R, SETTLE, DS, V_MAX,
                CAP_BILL_TAIL));
        }
    }

    // --- G: `solve_gain`, the suite's three margins x both arms
    for inc in [false, true] {
        let a = arms(&mut e, "G", inc);
        for (gi, margin) in [0.05, MARGIN, 0.40].into_iter().enumerate() {
            let p = format!("{a}/m{gi}");
            e.raw_f(&format!("{p}/grid_margin"), margin);
            emit_solve_gain(&mut e, &p, &solve_gain(
                &cap_rig(PHI_JAC, inc), &fl, LO, HI, TT4_MAX, PHI_JAC, margin, TAUS,
                SOLVE_GAIN_REF, inc, R, SETTLE, DS, V_MAX, SOLVE_GAIN_DQ, SOLVE_GAIN_EVERY));
        }
    }

    // --- H: the PLANTS themselves
    for inc in [false, true] {
        let p = arms(&mut e, "H", inc);
        let sm = PHI_BOTH / FLOOR - 1.0;
        let traj = windup_march(&windup_rig(PHI_BOTH, inc), &fl, LO, HI, TT4_MAX, sm, TAUS, R,
                                SETTLE, DS, V_MAX, inc, "demand", REF_LAW_APPLIED,
                                WINDUP_LAW_TRACK, Some(TAU_T), None).3;
        emit_march(&mut e, &format!("{p}/windup"), &traj);
        let mc = cap_rig(PHI_BOTH, inc);
        let acc = accel_for(&mc, &fl, LO, HI, sm, TT4_MAX, TAUS, V_MAX, inc, MARGIN);
        for law in [CAP_LAW_SOLVE, CAP_LAW_SENSED] {
            let t = cap_march(&mc, &fl, LO, HI, TT4_MAX, sm, TAUS, R, SETTLE, DS, V_MAX, inc,
                              "demand", REF_LAW_DEFAULT, WINDUP_LAW_NONE, None, law, &acc,
                              None).3;
            emit_march(&mut e, &format!("{p}/cap_{law}"), &t);
        }
    }

    // --- Z: the censuses. Recomputed from the emitted map rather than accumulated, which is a
    // DIFFERENT computation from the dumper's running counters and therefore a real comparison.
    let mut n_none = 0usize;
    let (mut n_neg_zero, mut n_pos_zero, mut n_nan) = (0usize, 0usize, 0usize);
    for (k, v) in &e.map {
        if k.ends_with('?') && *v == 0 {
            n_none += 1;
        }
    }
    // Only the keys that carry a FLOAT are eligible, and WHICH THOSE ARE WAS RECORDED WHEN THEY
    // WERE WRITTEN — see `Emit`. Deciding it here, from the key name, is what over-counted by 173.
    let floats: Vec<String> = e.floats.iter().cloned().collect();
    assert!(!floats.is_empty(), "a float census over an empty population agrees with anything");
    for k in &floats {
        let bits = e.map[k];
        let x = f64::from_bits(bits);
        if x == 0.0 {
            if bits == 0 { n_pos_zero += 1 } else { n_neg_zero += 1 }
        } else if x.is_nan() {
            n_nan += 1;
        }
    }
    e.raw_d("Z/n_none", n_none);
    e.raw_d("Z/n_neg_zero", n_neg_zero);
    e.raw_d("Z/n_pos_zero", n_pos_zero);
    e.raw_d("Z/n_nan", n_nan);
    e.map
}

// ============================================================================ the gates

fn golden(text: &str) -> BTreeMap<String, u64> {
    let mut out = BTreeMap::new();
    for line in text.lines() {
        if line.is_empty() { continue; }
        let (k, v) = line.split_once('\t').expect("key<TAB>u64");
        out.insert(k.to_string(), v.parse::<u64>().expect("a u64"));
    }
    out
}

/// **THE KEY SETS ARE COMPARED BEFORE ANY VALUE IS.** The Python side walks and this side
/// hand-lists, so a key the port forgot to emit is a NAMED FAILURE here rather than a silence in
/// both emitters — the module header's whole reason for the asymmetry.
#[test]
fn the_two_key_sets_are_equal() {
    let mine: BTreeSet<String> = drive().into_keys().collect();
    let theirs: BTreeSet<String> = golden(ORACLE_PYPY).into_keys().collect();
    let missing: Vec<&String> = theirs.difference(&mine).take(20).collect();
    let extra: Vec<&String> = mine.difference(&theirs).take(20).collect();
    assert!(missing.is_empty() && extra.is_empty(),
            "key sets differ: {} missing (first 20 {missing:?}), {} extra (first 20 {extra:?})",
            theirs.difference(&mine).count(), mine.difference(&theirs).count());
    assert!(!mine.is_empty(), "an empty key set agrees with an empty golden");
}

#[test]
fn every_value_is_bit_identical_to_the_pypy_golden() {
    let mine = drive();
    let theirs = golden(ORACLE_PYPY);
    let mut bad: Vec<String> = Vec::new();
    for (k, v) in &theirs {
        match mine.get(k) {
            None => bad.push(format!("{k}: MISSING")),
            Some(got) if got != v =>
                bad.push(format!("{k}: rust {got} ({}) vs pypy {v} ({})",
                                 f64::from_bits(*got), f64::from_bits(*v))),
            _ => {}
        }
    }
    assert!(bad.is_empty(), "{} of {} keys differ:\n  {}", bad.len(), theirs.len(),
            bad.iter().take(25).cloned().collect::<Vec<_>>().join("\n  "));
    println!("PyPy arm: {} keys compared, 0 read as input", theirs.len());
}

/// **P2, PRE-REGISTERED IN THE DUMPER BEFORE THE CPYTHON ARM RAN — AND FALSIFIED, 398 KEYS WIDE.**
///
/// § 5.31 (iv) measured the CPython arm needing an exemption for exactly `cap_bill`'s two
/// `fuel_int` values, 2 of 83 273 keys: they are the two `sum()` calls that add a 341-long
/// trajectory rather than a literal `1`, and CPython 3.12+'s `sum` is Neumaier-compensated where
/// PyPy's and Rust's are naive left folds. P2 predicted **the list stays exactly those**.
///
/// **MEASURED: 406 of 38 100 differ.** The eight `fuel_int` keys P2 named ALL differ, so the
/// prediction's CAUSE is confirmed. Its SCOPE is wrong by 398, and the reason is one sentence:
///
/// **P2 REASONED ABOUT THE LENGTH OF THE SUM.** It exempted the sums that add 341 things and
/// implicitly held that a short one is safe. `_charpoly4` (`engine.py:16235` / `16237`,
/// Faddeev–LeVerrier) is built on two `sum()` calls of **FOUR TERMS** — a matrix product and a
/// trace — and every one of them diverges too. Compensated summation parts company with a naive
/// fold as soon as the addends have mixed magnitudes; `n` is not the criterion, and a four-term
/// sum in a Jacobian's characteristic polynomial has mixed magnitudes by construction.
///
/// **AND THE BLAST RADIUS IS SET BY SHARING, NOT BY SIZE.** The suspected 341-term sum lives in
/// ONE reader and touched 8 keys. The unsuspected 4-term one lives in a SHARED static helper, so
/// it reached `det`, `det0`, `det_alive`, `det0_alive`, `det_err` and `det_ratio` across THREE
/// sections and six readers — 398 keys. **How many keys a divergence touches is a property of
/// how shared the CODE is, not of how big the SUM is**, and P2 sized it by the sum.
///
/// The exemption below is therefore written as the CAUSAL PATH — "this value passed through
/// `_charpoly4`, or through a trajectory `sum()`" — and NOT as the list of keys that came back
/// different. The difference matters: a key that stops differing must fail this gate, which is
/// why the count is asserted as well.
#[test]
fn every_value_is_bit_identical_to_the_cpython_golden_outside_the_predicted_exemption() {
    let mine = drive();
    let theirs = golden(ORACLE_CPYTHON);
    let (mut det_seen, mut int_seen) = (0usize, 0usize);
    let (mut det_total, mut int_total) = (0usize, 0usize);
    let mut bad: Vec<String> = Vec::new();
    for (k, v) in &theirs {
        let got = mine.get(k).unwrap_or_else(|| panic!("{k}: MISSING"));
        // The two causal paths, by the field the value came out of — NOT by whether it differed.
        let via_charpoly = k.split('/').any(|seg| seg.starts_with("det"));
        let via_traj_sum = k.contains("/fuel_int/");
        // `key?` and `key/#len` are the walker's SHAPE keys — an integer 0/1 and a length. They
        // can never differ between two runs of the same shape, so counting them into a
        // population whose members are asserted to differ makes the assertion unsatisfiable.
        let numeric = !k.ends_with('?') && !k.ends_with("#len");
        if via_charpoly && numeric { det_total += 1 }
        if via_traj_sum && numeric { int_total += 1 }
        if got == v { continue; }
        if via_charpoly { det_seen += 1; continue }
        if via_traj_sum { int_seen += 1; continue }
        bad.push(format!("{k}: rust {got} vs cpython {v}"));
    }
    assert!(bad.is_empty(),
            "a CPython difference on a path neither `_charpoly4` nor a trajectory `sum()` \
             reaches — {} keys, the first 25:\n  {}", bad.len(),
            bad.iter().take(25).cloned().collect::<Vec<_>>().join("\n  "));

    // AN EXEMPTION THAT COVERS KEYS THAT NEVER DIFFER IS AN EXEMPTION FOR NOTHING, and it would
    // hide a real Rust defect on every key it names. Both halves are pinned to what was measured.
    assert_eq!((det_seen, int_seen), (398, 8),
               "the exemption stopped matching what it exempts: {det_seen} `det*` and \
                {int_seen} `fuel_int` differ, measured 398 and 8. A DROP means the two \
                interpreters stopped disagreeing where `sum()` says they must — which is a \
                fact about the runtimes, not a licence to retype the numbers.");
    assert_eq!(int_seen, int_total,
               "EVERY `fuel_int` value must differ — {int_seen} of {int_total} do. One that \
                agrees is a cell where the march produced no trajectory to sum.");
    assert!(det_seen < det_total,
            "every single `det*` key differs ({det_seen} of {det_total}); the exemption has \
             stopped discriminating and would pass a Rust defect anywhere in the family");
    println!("CPython arm: {} keys compared, {det_seen}/{det_total} `det*` and \
              {int_seen}/{int_total} `fuel_int` exempt", theirs.len());
}

/// **THE TWO GOLDENS ARE THEMSELVES DIFFERENT FILES**, and a test suite that read the same bytes
/// twice would report perfect agreement having compared one thing with itself — AE step 5's
/// `ptr::eq` finding, in the shape an oracle can take it.
#[test]
fn the_two_goldens_are_not_the_same_file() {
    let a = golden(ORACLE_PYPY);
    let b = golden(ORACLE_CPYTHON);
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>(),
               "the two arms must publish the same key set");
    let differ = a.iter().filter(|(k, v)| b.get(*k) != Some(v)).count();
    assert!(differ > 0,
            "the two goldens agree on every one of {} keys — either the CPython arm was never \
             run and the file is a copy, or `sum()`'s compensation stopped being observable, \
             and those are different facts", a.len());
}
