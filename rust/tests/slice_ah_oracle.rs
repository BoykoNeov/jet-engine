//! SLICE AH step 6 (b) — **THE ORACLE for rungs 77 AND 78**, against PyPy *and* CPython.
//!
//! `rung77.rs` and `rung78.rs` port the two suites' 29 gates and add two. Those gates are
//! RELATIONS — *the order is φ-top*, *`w*` does not move with `k`*, *the second root collides at
//! `k·c = 1`*. This file is the VALUE seat: every number the nine readers publish, plus the two
//! plants underneath them, compared bit for bit against a Python golden.
//!
//! # WHAT THIS FILE SCORES THAT NOTHING ELSE CAN — **P3**
//!
//! Plan § 5.32 (vii) P3: the port's frozen-state guards restore the PREVIOUS value where Python's
//! `finally` blocks restore `None`, and the two semantics DO leave different values behind at two
//! of the six nest sites (`gauge_vs_device` → `_phi_at`, handed `q ± dq`). The claim is that
//! nothing ever reads what is left, so **every key here is bit-identical to PyPy** — and a gate
//! cannot assert the choice itself, only that the choice moves nothing (§ 5.32 (ix)). **That is
//! only a measurement if the drive visits the nest sites**, so the per-site visit counts over
//! THIS drive are recorded beside the result in § 5.32.6 (i), taken by the pre-flight's own
//! runtime probe re-run over `dump_slice_ah.py`.
//!
//! # THE KEY SET IS **WALKED** ON THE PYTHON SIDE AND **HAND-LISTED** HERE, ON PURPOSE
//!
//! Slice AG's design, kept: a key both emitters forgot is invisible to every check either side
//! runs, so the Python side names no keys and [`the_two_key_sets_are_equal`] fails by name before
//! a value is compared. A generic Rust walker would reintroduce the shared blindness.
//!
//! # FOUR SHAPE HAZARDS THIS SLICE SUPPLIES, EACH OF WHICH IS A KEY-SET FAILURE IF MISSED
//!
//! 1. **Float-keyed dicts.** `gauge_scan`'s `ks` and `root_census`' `cells` key on the `mult`
//!    FLOAT, which Python spells with `repr` — `0.0`, `2.0`, `-0.5`. Rust's `{}` prints `0` and
//!    `2`; its `{:?}` prints Python's spelling for every value on these grids. [`py_key`] is the
//!    one place that choice is made.
//! 2. **Python's field casing.** `Gw`, `Gq`, `Gw_pred`, `Gw_span`, `Gw1`, `G_at_w0`, `Tt4_max` —
//!    the port's struct fields are snake-case; the KEYS are Python's.
//! 3. **A ledger cell with `n = 0` has FIVE keys**, not sixteen, so [`v_ledger_cell`] branches.
//!    None does on the shipped grid (24 of 24 live); the branch is there because the SHAPE is
//!    Python's, not because it fires.
//! 4. **The plant is CLIP.** `_ledger_march` points carry 30 keys (`PointExtra::Shared`), the
//!    gauged march's 35 (`Demand`). Section P's column folds range over whatever float fields
//!    the POINT carries, derived from [`v_point`]'s own entries — the Python side does the same
//!    from `traj[0]` — so the set is derived on both sides rather than typed on either.
//!
//! # `drive()` RUNS ONCE, BEHIND A `OnceLock`, AND THAT IS NOT AN OPTIMISATION
//!
//! `gauge_march` resets and reads the process-global `GAUGE_HITS` / `GAUGE_BINDS`, and cargo runs
//! a binary's tests on parallel threads. Three gates each calling `drive()` would race on the
//! counters and publish a `hits` that belongs to another thread's march. One drive, shared.
//!
//! # THE INTERPRETER SENTINEL
//!
//! Each golden carries `_interp/sum_probe`, the bits of `sum([1e16, 1.0, -1e16])` in that
//! interpreter: `1.0` under CPython 3.12+'s compensated `sum`, `0.0` under PyPy's naive fold.
//! Every comparison excludes `_interp/*` by name, and
//! [`the_two_goldens_carry_their_own_interpreters_sum`] asserts each arm's value — so the gate
//! that says *these are two different runs* reads the one key built to say so, and does not
//! infer it from whatever else happened to differ. AG's version asserted `differ > 0`, which a
//! CORRECT zero-exemption slice cannot satisfy.
//!
//! # WHAT IS COMPARED, AND WHAT IS ONLY COUNTED
//!
//! Every key here is COMPUTED by the port and compared. Nothing is read out of the golden as an
//! input: the declared-read term is exactly zero, stated rather than omitted.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{FuelPoint, PointExtra};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{BleedLimiter, Regime};
use turbojet::map::ComponentMap;
use turbojet::reference_split::StatorIncidenceLimiter;
use turbojet::residual_gauge::{
    build_residual_gauge_cascade, gauge_counters, gauge_march, gauge_scan, gauge_vs_device,
    reset_gauge_counters, root_census,
    CensusCell, CensusRow, DeviceRow, GaugeCell, GaugeMarch, GaugeRestored, GaugeRow, GaugeScan,
    GaugeVsDevice, MarchCell, RootCensus, DEVICE_DQ, DEVICE_EVERY, DEVICE_SPREAD,
    GAUGE_MARCH_MULTS, GAUGE_SCAN_DQ, GAUGE_SCAN_MULTS, ROOT_CENSUS_MULTS, ROOT_CENSUS_N,
    ROOT_COUNT_HI, ROOT_COUNT_LO,
};
use turbojet::sensed_cap::{accel_for, cap_march};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::stiffness_ledger::{
    build_stiffness_ledger_cascade, ledger_march, leg_slopes, set_point_gains, singular_limit,
    stiffness_ledger, GainLeg, GainRow, LedgerCell, Leg, LegSlopes, SetPointGains, SingularLimit,
    SingularRow, SlopeLeg, SlopeRow, StiffnessLedger,
};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE_PYPY: &str = include_str!("../oracle/slice_ah_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/slice_ah_cpython.tsv");

/// The one key family every comparison excludes by NAME — see the module header.
const INTERP: &str = "_interp/";

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
const MARGIN: f64 = 0.10;

// -------------------------------------------------- THE READERS' OWN DEFAULTS
//
// Rung 78's are the module's own exports (above). Rung 77's are typed here, as `rung77.rs` types
// them — § 5.32.6 (f): two test files already pass them, so a module constant would be a third
// home for the numbers rather than a single one.

/// `leg_slopes`, `set_point_gains`, `singular_limit`, `gauge_scan` and `root_census` all default
/// `every = 8`.
const EVERY: usize = 8;
/// `set_point_gains`' `dq`.
const DQ: f64 = 1e-5;
/// `singular_limit`'s `spread`.
const SPREAD: f64 = 0.10;
/// `stiffness_ledger`'s four sweep axes and its own `every = 16`.
const PHI_LIMS: [f64; 2] = [0.76, 0.80];
const MARGINS: [f64; 3] = [0.05, 0.10, 0.40];
const TT4_MAXES: [f64; 2] = [1180.0, 1200.0];
const ARMS: [bool; 2] = [false, true];
const LEDGER_EVERY: usize = 16;

/// Section P's stride and gauged multiple — the dumper's `P_STRIDE` / `P_GAUGE_MULT`.
const P_STRIDE: usize = 5;
const P_GAUGE_MULT: f64 = 2.0;

// ============================================================================ the value tree
//
// Python's `walk()`, mirrored — slice AG's, unchanged.

#[derive(Clone, Debug)]
enum V {
    Null,
    B(bool),
    I(u64),
    F(f64),
    S(String),
    L(Vec<V>),
    M(Vec<(String, V)>),
}

fn vf(x: f64) -> V { V::F(x) }
fn vi(n: usize) -> V { V::I(n as u64) }
fn vb(x: bool) -> V { V::B(x) }
fn vs(x: &str) -> V { V::S(x.to_string()) }
fn vopt_f(x: Option<f64>) -> V { x.map(V::F).unwrap_or(V::Null) }
fn vopt_b(x: Option<bool>) -> V { x.map(V::B).unwrap_or(V::Null) }
/// Python's `(a, b)` — a TUPLE, which `walk` treats as a list.
fn vpair(a: f64, b: f64) -> V { V::L(vec![V::F(a), V::F(b)]) }
fn vopt_pair(x: Option<(f64, f64)>) -> V { x.map(|(a, b)| vpair(a, b)).unwrap_or(V::Null) }
fn vfloats(xs: &[f64]) -> V { V::L(xs.iter().copied().map(V::F).collect()) }
/// A tuple of leg NAMES — `("accel", "gov", "phi")`.
fn vlegs(xs: &[Leg]) -> V { V::L(xs.iter().map(|l| vs(l.name())).collect()) }
/// Python's `{k: span(k, …) for k in ("accel", "gov", "phi")}` — a dict keyed by leg NAME, off
/// the port's `[(lo, hi); 3]` in `Leg::ORDER`.
fn vspans(x: &[(f64, f64); 3]) -> V {
    V::M(Leg::ORDER.iter().zip(x.iter()).map(|(l, (a, b))| (l.name().to_string(), vpair(*a, *b)))
         .collect())
}
fn vopt_spans(x: &Option<[(f64, f64); 3]>) -> V { x.as_ref().map(vspans).unwrap_or(V::Null) }
/// Python's `dict(a=…, b=…)`, spelled with `&str` keys at the call site.
fn m(kv: Vec<(&str, V)>) -> V { V::M(kv.into_iter().map(|(k, v)| (k.to_string(), v)).collect()) }

/// **Hazard 1: a float used as a dict KEY, spelled as Python's `repr` spells it.** `{:?}` prints
/// `0.0` / `2.0` / `-0.5` / `1.05` for every multiple on these grids; `{}` would print `0` / `2`.
fn py_key(x: f64) -> String { format!("{x:?}") }

fn fnv(text: &str) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for byte in text.as_bytes() {
        h ^= *byte as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

/// The emitted map, and the set of keys that carry a float, recorded AS THEY ARE EMITTED (AG
/// § 5.31.6 (c) 2: deciding floatness afterwards from the key name over-counted by 173).
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

#[derive(Clone, Copy)]
enum Cls {
    /// `StiffnessLedgerTransient`.
    Ledger,
    /// `ResidualGaugeTransient`.
    Gauge,
}

/// The dumper's `rig(cls, inc)` — the RECEIVER at `PHI_JAC`, with the five knobs the suites set
/// by plain assignment.
fn rig(cls: Cls, inc: bool) -> ScheduledStatorCore {
    let arm = suite_arm(PHI_JAC / FLOOR - 1.0, inc);
    let m = full_of(match cls {
        Cls::Ledger => build_stiffness_ledger_cascade(
            design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm),
        Cls::Gauge => build_residual_gauge_cascade(
            design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm),
    });
    m.fuel.inner.lag_coord.set("demand");
    m.fuel.inner.ref_law.set("sched");
    m.fuel.inner.windup_law.set("none");
    m.fuel.inner.tau_t.set(None);
    m.fuel.inner.cap_law.set("solve");
    m
}

// ============================================================================ the converters
//
// Each is Python's returned dict, key for key, in Python's casing (hazard 2).

fn v_slope_leg(l: &SlopeLeg) -> V {
    m(vec![("w", vf(l.w)), ("Gw", vf(l.gw)), ("norm", vf(l.norm)), ("stiff", vf(l.stiff))])
}

fn v_slope_row(r: &SlopeRow) -> V {
    let mut kv = vec![("s", vf(r.s))];
    for (l, leg) in Leg::ORDER.iter().zip(r.legs.iter()) {
        kv.push((l.name(), v_slope_leg(leg)));
    }
    kv.push(("c", vf(r.c)));
    kv.push(("c_err", vf(r.c_err)));
    m(kv)
}

fn v_leg_slopes(x: &LegSlopes) -> V {
    m(vec![
        ("phi_lim", vf(x.phi_lim)), ("margin", vf(x.margin)), ("inc", vb(x.inc)), ("n", vi(x.n)),
        ("rows", V::L(x.rows.iter().map(v_slope_row).collect())),
        ("c_err", vopt_f(x.c_err)), ("c", vopt_pair(x.c)), ("Gw", vopt_spans(&x.gw)),
        ("norm", vopt_spans(&x.norm)), ("stiff", vopt_spans(&x.stiff)), ("sep", vopt_f(x.sep)),
    ])
}

fn v_gain_leg(l: &GainLeg) -> V {
    m(vec![("w", vf(l.w)), ("direct", vf(l.direct)), ("ift", vf(l.ift)), ("Gq", vf(l.gq)),
           ("Gw", vf(l.gw)), ("live", vb(l.live)), ("err", vf(l.err))])
}

fn v_gain_row(r: &GainRow) -> V {
    let mut kv = vec![("s", vf(r.s))];
    for (l, leg) in Leg::ORDER.iter().zip(r.legs.iter()) {
        kv.push((l.name(), v_gain_leg(leg)));
    }
    kv.push(("live", vlegs(&r.live)));
    m(kv)
}

fn v_set_point_gains(x: &SetPointGains) -> V {
    m(vec![
        ("phi_lim", vf(x.phi_lim)), ("margin", vf(x.margin)), ("inc", vb(x.inc)), ("n", vi(x.n)),
        ("rows", V::L(x.rows.iter().map(v_gain_row).collect())),
        ("ift_err", vopt_f(x.ift_err)), ("gain", vopt_spans(&x.gain)),
        ("order", x.order.as_ref().map(|o| vlegs(o)).unwrap_or(V::Null)),
        ("order_stable", vopt_b(x.order_stable)), ("n_guarded", vi(x.n_guarded)),
        ("guarded", x.guarded.as_ref().map(|g| vlegs(g)).unwrap_or(V::Null)),
        ("guarded_stable", vopt_b(x.guarded_stable)),
        ("guarded_orders", V::L(x.guarded_orders.iter().map(|g| vlegs(g)).collect())),
        ("phi_top", vb(x.phi_top)),
    ])
}

fn v_singular_row(r: &SingularRow) -> V {
    m(vec![
        ("s", vf(r.s)), ("w_phi", vf(r.w_phi)), ("w_gov", vf(r.w_gov)),
        ("phi_open", vf(r.phi_open)), ("phi_closed", vf(r.phi_closed)),
        ("gov_open", vf(r.gov_open)), ("gov_closed", vf(r.gov_closed)),
        ("gov_rel", vf(r.gov_rel)), ("phi_spread", vf(r.phi_spread)), ("phi_off", vf(r.phi_off)),
    ])
}

fn v_singular_limit(x: &SingularLimit) -> V {
    m(vec![
        ("phi_lim", vf(x.phi_lim)), ("margin", vf(x.margin)), ("inc", vb(x.inc)),
        ("spread", vf(x.spread)), ("n", vi(x.n)),
        ("rows", V::L(x.rows.iter().map(v_singular_row).collect())),
        ("phi_open", vopt_f(x.phi_open)), ("phi_closed", vopt_f(x.phi_closed)),
        ("phi_off", vopt_f(x.phi_off)), ("phi_spread", vopt_f(x.phi_spread)),
        ("gov_rel", vopt_f(x.gov_rel)), ("gov_open", vopt_f(x.gov_open)),
    ])
}

/// **Hazard 3: a dead cell is FIVE keys, a live one sixteen** — Python appends two different dict
/// literals, and the port's `Option`s cannot say which one was meant.
fn v_ledger_cell(c: &LedgerCell) -> V {
    let mut kv = vec![("inc", vb(c.inc)), ("phi_lim", vf(c.phi_lim)), ("margin", vf(c.margin)),
                      ("Tt4_max", vf(c.tt4_max)), ("n", vi(c.n))];
    if c.n == 0 {
        return m(kv);
    }
    kv.extend(vec![
        ("order", c.order.as_ref().map(|o| vlegs(o)).unwrap_or(V::Null)),
        ("stable", vopt_b(c.stable)), ("ift_err", vopt_f(c.ift_err)), ("c", vopt_pair(c.c)),
        ("norm", vopt_spans(&c.norm)), ("sep", vopt_f(c.sep)),
        ("gov_norm", vopt_pair(c.gov_norm)),
        ("guarded", c.guarded.as_ref().map(|g| vlegs(g)).unwrap_or(V::Null)),
        ("n_guarded", vi(c.n_guarded)), ("guarded_stable", vopt_b(c.guarded_stable)),
        ("phi_top", vopt_b(c.phi_top)),
    ]);
    m(kv)
}

fn v_stiffness_ledger(x: &StiffnessLedger) -> V {
    m(vec![
        ("n_cells", vi(x.n_cells)), ("n_live", vi(x.n_live)),
        ("cells", V::L(x.cells.iter().map(v_ledger_cell).collect())),
        ("orders", V::L(x.orders.iter().map(|o| vlegs(o)).collect())),
        ("order_invariant", vb(x.order_invariant)), ("order_stable", vb(x.order_stable)),
        ("guarded_orders", V::L(x.guarded_orders.iter().map(|g| vlegs(g)).collect())),
        ("guarded_invariant", vb(x.guarded_invariant)),
        ("guarded_stable", vb(x.guarded_stable)), ("phi_top", vb(x.phi_top)),
        ("ift_err", vopt_f(x.ift_err)), ("sep", vopt_f(x.sep)), ("gov_norm", vopt_f(x.gov_norm)),
        ("c_max", vf(x.c_max)),
    ])
}

fn v_gauge_cell(c: &GaugeCell) -> V {
    m(vec![
        ("mult", vf(c.mult)), ("k", vf(c.k)), ("w", vf(c.w)), ("ok", vb(c.ok)), ("Gw", vf(c.gw)),
        ("Gw_pred", vf(c.gw_pred)), ("anchor", vf(c.anchor)), ("n_roots", vi(c.n_roots)),
        ("w_move", vf(c.w_move)), ("Gw_err", vf(c.gw_err)), ("direct", vf(c.direct)),
        ("excluded", vb(c.excluded)), ("gain_move", vf(c.gain_move)),
    ])
}

fn v_gauge_row(r: &GaugeRow) -> V {
    let ks = V::M(r.ks.iter().map(|c| (py_key(c.mult), v_gauge_cell(c))).collect());
    m(vec![
        ("s", vf(r.s)), ("c", vf(r.c)), ("k_crit", vf(r.k_crit)), ("w1", vf(r.w1)),
        ("Gw1", vf(r.gw1)), ("c_err", vf(r.c_err)), ("ks", ks), ("base", vf(r.base)),
    ])
}

fn v_gauge_scan(x: &GaugeScan) -> V {
    m(vec![
        ("phi_lim", vf(x.phi_lim)), ("margin", vf(x.margin)), ("inc", vb(x.inc)), ("n", vi(x.n)),
        ("rows", V::L(x.rows.iter().map(v_gauge_row).collect())),
        ("n_excluded", vi(x.n_excluded)), ("n_kept", vi(x.n_kept)),
        ("excluded_mults", vfloats(&x.excluded_mults)),
        ("excluded_worst", vopt_f(x.excluded_worst)), ("c_err", vopt_f(x.c_err)),
        ("w_move", vopt_f(x.w_move)), ("Gw_err", vopt_f(x.gw_err)),
        ("Gw_span", vopt_pair(x.gw_span)), ("sign_change", vopt_b(x.sign_change)),
        ("gain_move", vopt_f(x.gain_move)), ("n_bad", vi(x.n_bad)), ("c", vopt_pair(x.c)),
    ])
}

fn v_census_cell(c: &CensusCell) -> V {
    m(vec![
        ("mult", vf(c.mult)), ("k", vf(c.k)), ("G_at_w0", vf(c.g_at_w0)),
        ("n_roots", vi(c.n_roots)), ("roots", vfloats(&c.roots)),
        ("spurious", vfloats(&c.spurious)),
    ])
}

fn v_census_row(r: &CensusRow) -> V {
    let cells = V::M(r.cells.iter().map(|c| (py_key(c.mult), v_census_cell(c))).collect());
    m(vec![("s", vf(r.s)), ("w0", vf(r.w0)), ("c", vf(r.c)), ("k_crit", vf(r.k_crit)),
           ("cells", cells)])
}

fn v_root_census(x: &RootCensus) -> V {
    m(vec![
        ("phi_lim", vf(x.phi_lim)), ("margin", vf(x.margin)), ("inc", vb(x.inc)), ("n", vi(x.n)),
        ("rows", V::L(x.rows.iter().map(v_census_row).collect())),
        ("G_at_w0", vopt_f(x.g_at_w0)), ("true_found", vb(x.true_found)),
        ("n_roots", V::L(x.n_roots.iter().map(|&n| vi(n)).collect())),
        ("multi_mults", vfloats(&x.multi_mults)), ("band", vopt_pair(x.band)),
        ("brackets", vopt_b(x.brackets)), ("approach", vopt_f(x.approach)),
    ])
}

fn v_device_row(r: &DeviceRow) -> V {
    m(vec![
        ("s", vf(r.s)), ("w_solve", vf(r.w_solve)), ("w_sensed", vf(r.w_sensed)),
        ("device", vf(r.device)), ("w_phi", vf(r.w_phi)), ("phi_open_w", vf(r.phi_open_w)),
        ("phi_open_q", vf(r.phi_open_q)), ("phi_closed_w", vf(r.phi_closed_w)),
        ("kill_w", vf(r.kill_w)), ("phi_spread", vf(r.phi_spread)),
    ])
}

fn v_gauge_vs_device(x: &GaugeVsDevice) -> V {
    m(vec![
        ("phi_lim", vf(x.phi_lim)), ("margin", vf(x.margin)), ("inc", vb(x.inc)), ("n", vi(x.n)),
        ("rows", V::L(x.rows.iter().map(v_device_row).collect())),
        ("device", vopt_f(x.device)), ("device_max", vopt_f(x.device_max)),
        ("kill_w", vopt_f(x.kill_w)), ("phi_open_w", vopt_f(x.phi_open_w)),
        ("phi_closed_w", vopt_f(x.phi_closed_w)), ("phi_open_q", vopt_f(x.phi_open_q)),
        ("phi_spread", vopt_f(x.phi_spread)),
    ])
}

fn v_march_cell(c: &MarchCell) -> V {
    m(vec![
        ("mult", vf(c.mult)), ("k", vf(c.k)), ("n", vi(c.n)), ("same_len", vb(c.same_len)),
        ("worst", vf(c.worst)),
        // Python's `where`: `(key, i)` or `None`.
        ("where", c.where_.map(|(k, i)| V::L(vec![vs(k), vi(i)])).unwrap_or(V::Null)),
        ("kc", vpair(c.kc.0, c.kc.1)), ("hits", V::I(c.hits)), ("binds", V::I(c.binds)),
        ("clear", vb(c.clear)),
    ])
}

fn v_gauge_march(x: &GaugeMarch) -> V {
    m(vec![
        ("phi_lim", vf(x.phi_lim)), ("margin", vf(x.margin)), ("inc", vb(x.inc)),
        ("c0", vf(x.c0)), ("n", vi(x.n)),
        ("cells", V::L(x.cells.iter().map(v_march_cell).collect())),
        ("worst", vopt_f(x.worst)), ("same_len", vb(x.same_len)), ("hits", V::I(x.hits)),
        ("binds", V::I(x.binds)), ("sched_moved", vopt_f(x.sched_moved)),
        ("clear", vb(x.clear)),
        ("kc", V::L(x.kc.iter().map(|(mult, (a, b))| V::L(vec![vf(*mult), vpair(*a, *b)]))
                   .collect())),
    ])
}

// ---------------------------------------------------------------- section P's points

/// Python stores the valve regime as a string; the port has an enum. Spelled HERE rather than
/// added to the library for the oracle's convenience (AG's choice, kept).
fn regime_str(r: Regime) -> &'static str {
    match r {
        Regime::Dormant => "dormant",
        Regime::Riding => "riding",
        Regime::Saturated => "saturated",
    }
}

fn vregime(r: Option<Regime>) -> V { r.map(|r| vs(regime_str(r))).unwrap_or(V::Null) }

/// **Hazard 4: a marched point, WHOLE, as the dict Python records** — 30 keys on the CLIP march
/// (`Shared`), 35 on the gauged DEMAND one. Any other variant is a march this file did not drive.
fn v_point(p: &FuelPoint) -> V {
    let mut kv: Vec<(&str, V)> = vec![
        ("s", vf(p.s)), ("nu_lp", vf(p.nu_lp)), ("nu_hp", vf(p.nu_hp)), ("Tt4", vf(p.tt4)),
        ("f", vf(p.f)), ("pi_lpc", vf(p.pi_lpc)), ("pi_hpc", vf(p.pi_hpc)),
        ("phi_lp", vf(p.phi_lp)), ("phi_hp", vf(p.phi_hp)), ("mdot_air", vf(p.mdot_air)),
        ("sp_thrust", vf(p.sp_thrust)), ("branch", vs(p.branch.label())), ("mf", vf(p.mf)),
        ("mf_sched", vf(p.mf_sched)),
    ];
    match p.extra {
        PointExtra::Shared { g, required, b, b_cmd, v, v_cmd, v_regime, ic_iters, ic_res,
                             ic_order, g_fuel, g_gov, required_fuel, required_gov, authority,
                             share_law } => {
            kv.extend(vec![
                ("g", vf(g)), ("required", vf(required)), ("b", vf(b)), ("b_cmd", vf(b_cmd)),
                ("v", vf(v)), ("v_cmd", vf(v_cmd)), ("v_regime", vregime(v_regime)),
                ("ic_iters", vi(ic_iters)), ("ic_res", vf(ic_res)), ("ic_order", vs(ic_order)),
                ("g_fuel", vf(g_fuel)), ("g_gov", vf(g_gov)),
                ("required_fuel", vf(required_fuel)), ("required_gov", vf(required_gov)),
                ("authority", vs(authority.as_str())), ("share_law", vs(share_law)),
            ]);
        }
        PointExtra::Demand { g, required, b, b_cmd, v, v_cmd, v_regime, ic_iters, ic_res,
                             ic_order, g_fuel, g_gov, required_fuel, required_gov, authority,
                             share_law, w_fuel, w_gov, cap_fuel, cap_gov, lag_coord } => {
            kv.extend(vec![
                ("g", vf(g)), ("required", vf(required)), ("b", vf(b)), ("b_cmd", vf(b_cmd)),
                ("v", vf(v)), ("v_cmd", vf(v_cmd)), ("v_regime", vregime(v_regime)),
                ("ic_iters", vi(ic_iters)), ("ic_res", vf(ic_res)), ("ic_order", vs(ic_order)),
                ("g_fuel", vf(g_fuel)), ("g_gov", vf(g_gov)),
                ("required_fuel", vf(required_fuel)), ("required_gov", vf(required_gov)),
                ("authority", vs(authority.as_str())), ("share_law", vs(share_law)),
                ("w_fuel", vf(w_fuel)), ("w_gov", vf(w_gov)), ("cap_fuel", vf(cap_fuel)),
                ("cap_gov", vf(cap_gov)), ("lag_coord", vs(lag_coord)),
            ]);
        }
        other => panic!("section P drives CLIP (`Shared`) and DEMAND marches only: {other:?}"),
    }
    m(kv)
}

/// The point's float fields, BY NAME, derived from [`v_point`]'s own entries — the Python side
/// derives the same set from `traj[0]`.
fn float_fields(p: &FuelPoint) -> Vec<(String, f64)> {
    match v_point(p) {
        V::M(kv) => kv.into_iter()
            .filter_map(|(k, v)| if let V::F(x) = v { Some((k, x)) } else { None })
            .collect(),
        _ => unreachable!(),
    }
}

/// Python's `min(col)` / `max(col)`: the FIRST extremum, replaced only when STRICTLY beaten — the
/// rule that decides which of `-0.0` / `+0.0` a tie returns, where `f64::min` does not promise.
fn py_min(xs: &[f64]) -> f64 {
    xs[1..].iter().fold(xs[0], |a, &x| if x < a { x } else { a })
}
fn py_max(xs: &[f64]) -> f64 {
    xs[1..].iter().fold(xs[0], |a, &x| if x > a { x } else { a })
}

fn emit_march(e: &mut Emit, tag: &str, traj: &[FuelPoint]) {
    e.raw_d(&format!("{tag}/n"), traj.len());
    assert!(!traj.is_empty(), "an empty march is not a measurement: {tag}");
    for (i, p) in traj.iter().enumerate().step_by(P_STRIDE) {
        e.walk(&format!("{tag}/p{i}"), &v_point(p));
    }
    let names: Vec<String> = float_fields(&traj[0]).into_iter().map(|(k, _)| k).collect();
    e.raw_d(&format!("{tag}/#cols"), names.len());
    let rows: Vec<Vec<(String, f64)>> = traj.iter().map(float_fields).collect();
    for (ci, name) in names.iter().enumerate() {
        let col: Vec<f64> = rows.iter().map(|r| {
            assert_eq!(&r[ci].0, name, "column {name} changes shape mid-march");
            r[ci].1
        }).collect();
        e.raw_f(&format!("{tag}/col/{name}/min"), py_min(&col));
        e.raw_f(&format!("{tag}/col/{name}/max"), py_max(&col));
        e.raw_f(&format!("{tag}/col/{name}/last"), *col.last().expect("non-empty"));
        e.raw_d(&format!("{tag}/col/{name}/neg"), col.iter().filter(|&&x| x < 0.0).count());
    }
}

// ============================================================================ the drive

fn arms(e: &mut Emit, tag: &str, inc: bool) -> String {
    let p = format!("{tag}/i{}", u8::from(inc));
    e.raw_b(&format!("{p}/arm_inc"), inc);
    p
}

fn run_drive() -> BTreeMap<String, u64> {
    let mut e = Emit { map: BTreeMap::new(), floats: BTreeSet::new() };
    let fl = flight();

    // --- A-C: rung 77's per-arm readers, on a rung-77 receiver
    for inc in [false, true] {
        let p = arms(&mut e, "A", inc);
        e.walk(&p, &v_leg_slopes(&leg_slopes(
            &rig(Cls::Ledger, inc), &fl, LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, inc, R, SETTLE,
            DS, V_MAX, EVERY)));
    }
    for inc in [false, true] {
        let p = arms(&mut e, "B", inc);
        e.walk(&p, &v_set_point_gains(&set_point_gains(
            &rig(Cls::Ledger, inc), &fl, LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, inc, R, SETTLE,
            DS, V_MAX, DQ, EVERY)));
    }
    for inc in [false, true] {
        let p = arms(&mut e, "C", inc);
        e.walk(&p, &v_singular_limit(&singular_limit(
            &rig(Cls::Ledger, inc), &fl, LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, inc, R, SETTLE,
            DS, V_MAX, SPREAD, EVERY)));
    }

    // --- D: `stiffness_ledger`, once, at its own 24-cell default
    e.walk("D", &v_stiffness_ledger(&stiffness_ledger(
        &rig(Cls::Ledger, false), &fl, LO, HI, &PHI_LIMS, &MARGINS, &TT4_MAXES, &ARMS, TAUS, R,
        SETTLE, DS, V_MAX, LEDGER_EVERY)));

    // --- E-H: rung 78's readers, on a rung-78 receiver
    for inc in [false, true] {
        let p = arms(&mut e, "E", inc);
        e.walk(&p, &v_gauge_scan(&gauge_scan(
            &rig(Cls::Gauge, inc), &fl, LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, inc, R, SETTLE,
            DS, V_MAX, GAUGE_SCAN_DQ, EVERY, &GAUGE_SCAN_MULTS)));
    }
    for inc in [false, true] {
        let p = arms(&mut e, "F", inc);
        e.walk(&p, &v_root_census(&root_census(
            &rig(Cls::Gauge, inc), &fl, LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, inc, R, SETTLE,
            DS, V_MAX, EVERY, &ROOT_CENSUS_MULTS, ROOT_COUNT_LO, ROOT_COUNT_HI, ROOT_CENSUS_N)));
    }
    for inc in [false, true] {
        let p = arms(&mut e, "G", inc);
        e.walk(&p, &v_gauge_vs_device(&gauge_vs_device(
            &rig(Cls::Gauge, inc), &fl, LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, inc, R, SETTLE,
            DS, V_MAX, DEVICE_EVERY, DEVICE_DQ, DEVICE_SPREAD)));
    }
    let mut c0 = [0.0_f64; 2];
    for inc in [false, true] {
        let p = arms(&mut e, "H", inc);
        let g = gauge_march(&rig(Cls::Gauge, inc), &fl, LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS,
                            inc, R, SETTLE, DS, V_MAX, &GAUGE_MARCH_MULTS);
        c0[usize::from(inc)] = g.c0;
        e.walk(&p, &v_gauge_march(&g));
    }

    // --- P: the plants themselves
    for inc in [false, true] {
        let p = arms(&mut e, "P", inc);
        let sm = PHI_JAC / FLOOR - 1.0;
        let traj = ledger_march(&rig(Cls::Ledger, inc), &fl, LO, HI, TT4_MAX, sm, TAUS, R,
                                SETTLE, DS, V_MAX, inc, MARGIN).3;
        emit_march(&mut e, &format!("{p}/ledger"), &traj);
        let mg = rig(Cls::Gauge, inc);
        let acc = accel_for(&mg, &fl, LO, HI, sm, TT4_MAX, TAUS, V_MAX, inc, MARGIN);
        let k = P_GAUGE_MULT / c0[usize::from(inc)];
        e.raw_f(&format!("{p}/gauged_k"), k);
        // THE TRAJECTORY CANNOT SEE THE GAUGE, SO ITS COUNTER RIDES BESIDE IT — see the dumper.
        // Dropping `GaugeRestored` from this block moved 0 of 29 284 keys until these two existed.
        reset_gauge_counters();
        let gauged = {
            let _gk = GaugeRestored::set(&mg.fuel.inner, k);
            cap_march(&mg, &fl, LO, HI, TT4_MAX, sm, TAUS, R, SETTLE, DS, V_MAX, inc, "demand",
                      "sched", "none", None, "solve", &acc, None).3
        };
        let (hits, binds) = gauge_counters();
        e.put(format!("{p}/gauged_hits"), hits);
        e.put(format!("{p}/gauged_binds"), binds);
        emit_march(&mut e, &format!("{p}/gauged"), &gauged);
    }

    // --- Z: the censuses, recomputed from the emitted map — a DIFFERENT computation from the
    // dumper's running counters, and therefore a real comparison.
    let n_none = e.map.iter().filter(|(k, v)| k.ends_with('?') && **v == 0).count();
    let (mut n_neg_zero, mut n_pos_zero, mut n_nan) = (0usize, 0usize, 0usize);
    assert!(!e.floats.is_empty(), "a float census over an empty population agrees with anything");
    for k in &e.floats {
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

/// ONE drive per binary — see the module header on the process-global gauge counters.
fn drive() -> &'static BTreeMap<String, u64> {
    static D: OnceLock<BTreeMap<String, u64>> = OnceLock::new();
    D.get_or_init(run_drive)
}

// ============================================================================ the gates

/// A golden, split into the compared keys and the `_interp/*` keys every comparison excludes.
fn golden(text: &str) -> (BTreeMap<String, u64>, BTreeMap<String, u64>) {
    let (mut keys, mut interp) = (BTreeMap::new(), BTreeMap::new());
    for line in text.lines() {
        if line.is_empty() { continue; }
        let (k, v) = line.split_once('\t').expect("key<TAB>u64");
        let v = v.parse::<u64>().expect("a u64");
        if k.starts_with(INTERP) { interp.insert(k.to_string(), v) } else { keys.insert(k.to_string(), v) };
    }
    (keys, interp)
}

/// **THE KEY SETS ARE COMPARED BEFORE ANY VALUE IS** — the module header's reason for the
/// asymmetry.
#[test]
fn the_two_key_sets_are_equal() {
    let mine: BTreeSet<&String> = drive().keys().collect();
    let (theirs, _) = golden(ORACLE_PYPY);
    let theirs: BTreeSet<&String> = theirs.keys().collect();
    let missing: Vec<&&String> = theirs.difference(&mine).take(20).collect();
    let extra: Vec<&&String> = mine.difference(&theirs).take(20).collect();
    assert!(missing.is_empty() && extra.is_empty(),
            "key sets differ: {} missing (first 20 {missing:?}), {} extra (first 20 {extra:?})",
            theirs.difference(&mine).count(), mine.difference(&theirs).count());
    assert!(!mine.is_empty(), "an empty key set agrees with an empty golden");
}

/// **P3 — every key bit-identical to PyPy.** See the module header for what this certifies and
/// for the visit counts that make it a measurement rather than a zero.
#[test]
fn every_value_is_bit_identical_to_the_pypy_golden() {
    let mine = drive();
    let (theirs, _) = golden(ORACLE_PYPY);
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

/// **THE CPYTHON ARM, WITH NO EXEMPTION AT ALL** — pre-registered in `dump_slice_ah.py`'s header
/// before either golden was compared, and written from a DERIVED set rather than a named one.
///
/// AG's P2 sized its exemption from the sums it had reasoned about and missed `_charpoly4`, a
/// four-term sum in a SHARED helper, by 398 keys. So this prediction is not read off § 5.32 (iv)
/// (which scoped itself to the two classes' own bodies). It is read off a RUNTIME census of
/// `builtins.sum` over this exact drive, and that census found ONE model site, `engine.py`'s
/// `n_bad` count over integer literals, and no float summation anywhere the readers or plants reach.
/// Compensated-vs-naive `sum` therefore has nowhere to land, and every key must agree.
///
/// **THE FALSIFIER:** any differing key. The gate is not relaxed to fit one. A difference would
/// mean either a divergence that is not summation or a `sum` path the census could not see, and
/// both are findings.
#[test]
fn every_value_is_bit_identical_to_the_cpython_golden() {
    let mine = drive();
    let (theirs, _) = golden(ORACLE_CPYTHON);
    let mut bad: Vec<String> = Vec::new();
    for (k, v) in &theirs {
        let got = mine.get(k).unwrap_or_else(|| panic!("{k}: MISSING"));
        if got != v {
            bad.push(format!("{k}: rust {got} ({}) vs cpython {v} ({})",
                             f64::from_bits(*got), f64::from_bits(*v)));
        }
    }
    assert!(bad.is_empty(), "{} of {} keys differ:\n  {}", bad.len(), theirs.len(),
            bad.iter().take(25).cloned().collect::<Vec<_>>().join("\n  "));
    println!("CPython arm: {} keys compared, 0 exempt", theirs.len());
}

/// **THE TWO GOLDENS ARE TWO RUNS, SAID BY THE ONE KEY BUILT TO SAY IT.** `sum([1e16, 1.0,
/// -1e16])` is `1.0` under CPython 3.12+'s compensated `sum` and `0.0` under PyPy's naive fold. A
/// CPython file that is a copy of the PyPy one fails here — which AG's `differ > 0` form could
/// only say while some OTHER key happened to differ.
#[test]
fn the_two_goldens_carry_their_own_interpreters_sum() {
    let key = format!("{INTERP}sum_probe");
    let (_, pypy) = golden(ORACLE_PYPY);
    let (_, cpython) = golden(ORACLE_CPYTHON);
    assert_eq!(pypy.get(&key).copied(), Some(0.0_f64.to_bits()),
               "the PyPy golden must carry PyPy's naive `sum`");
    assert_eq!(cpython.get(&key).copied(), Some(1.0_f64.to_bits()),
               "the CPython golden must carry CPython's compensated `sum`");
    // and Rust's fold is PyPy's, which is why PyPy is the bit-exact arm
    let naive = [1e16_f64, 1.0, -1e16].iter().fold(0.0, |a, x| a + x);
    assert_eq!(naive.to_bits(), 0.0_f64.to_bits());
}
