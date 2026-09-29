//! SLICE AI step 6 — **THE ORACLE for rungs 79 AND 80**, against PyPy *and* CPython.
//!
//! `rung79.rs`, `rung80.rs` and the four `slice_ai_*.rs` files gate RELATIONS and counters. This
//! file is the VALUE seat: every number the eight readers publish, rung 74's `demand_gains` on
//! the machines that re-aim its setter, and the two plants underneath, compared bit for bit
//! against a Python golden written by `oracle/dump_slice_ai.py`.
//!
//! # THE KEY SET IS **WALKED** ON THE PYTHON SIDE AND **HAND-LISTED** HERE, ON PURPOSE
//!
//! Slice AG's design, kept: a key both emitters forgot is invisible to every check either side
//! runs, so the Python side names no keys and [`the_two_key_sets_are_equal`] fails by name before
//! a value is compared. A generic Rust walker would reintroduce the shared blindness.
//!
//! # SIX SHAPE HAZARDS THIS SLICE SUPPLIES, EACH A KEY-SET FAILURE IF MISSED
//!
//! 1. **`skipped` is a DICT keyed by name in both readers that carry it, and the two Rust tuples
//!    hold it in OPPOSITE orders** — [`SplitGains`]' `(switch, regime)`, [`DemandGains`]'
//!    `(regime, switch)`. The converters name each half.
//! 2. **`split_gains`' `authority` dict is keyed by the authority's `%s`** — a label, or `None`.
//! 3. **`coord_march`'s `br_phi` / `br_inc` are SIGNED** (`calls − fb`). Python's walker masks an
//!    int to 64 bits, so a negative arrives as its two's complement; [`vi64`] is that cast.
//! 4. **`where` is a `(key, i)` TUPLE or `None`, and `vacuous` has THREE states** — `True`,
//!    `False`, `None`.
//! 5. **`split_arrest`'s rows are `dict(wall=w, **row)`** — a split row with one more key.
//! 6. **Python's casing** — `max_Tt4`, `pair_RC`, `pair_CV`, `Tt4`.
//!
//! # THE STATOR ARM IS SWEPT ON EVERY READER, THOUGH NEITHER SUITE SWEEPS IT
//!
//! See the dumper's header. It is the first rung-80 machine built with an incidence stator in
//! either language. **It does NOT drive `walls_of`'s incidence round trip**: every rig arms the
//! valve, whose wall `phi_air` reads first. Injected as `999.0`, that branch moved 0 keys here.
//!
//! # WHAT THIS FILE SCORES THAT NOTHING ELSE CAN
//!
//! * **P2a** — rung 74's `demand_gains` on an R79/R80 machine equals the Python on every value key
//!   AND on rung 79's six counter DELTAS (section I).
//! * **P2b** — the re-aim is value-INVISIBLE: every section-I value key on R79 and R80 equals R78's,
//!   while the counters move. A RELATION between two drives each pinned to the golden
//!   ([`p2b_the_re_aim_moves_counters_and_no_value`]). The field half — `("clip", "demand")` inside
//!   the scope — is not reachable from a returned value and is gated by `slice_ai_cells.rs`.
//! * **P3** — thread-local counters are value-identical to Python under PARALLEL execution. A single
//!   drive cannot test that (one thread marching is the same for a thread-local and a global), so
//!   [`p3_two_threads_marching_at_once_both_match_the_golden`] runs every counter-bearing section on
//!   TWO spawned threads at once.
//!
//! # WHAT IS COMPARED, AND WHAT IS ONLY COUNTED
//!
//! Every key here is COMPUTED by the port and compared. Nothing is read out of the golden as an
//! input: the declared-read term is exactly zero, stated rather than omitted.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use turbojet::bleed_transient::LeverArm;
use turbojet::demand_coordinate::{demand_gains, CoordScope, DemandGains};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{Authority, FuelPoint, PointExtra};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{BleedLimiter, Regime};
use turbojet::map::ComponentMap;
use turbojet::reference_split::StatorIncidenceLimiter;
use turbojet::residual_gauge::build_residual_gauge_cascade;
use turbojet::sensed_cap::{accel_for, cap_march};
use turbojet::split_wall::{
    build_split_wall_cascade, split_arrest, split_gains, split_liveness, split_march,
    split_saturation, ArrestArm, GainsArm, SplitArrest, SplitCell, SplitGains, SplitLiveness,
    SplitRow, SplitSaturation, ARREST_WALLS, LIVENESS_COORDS, LIVENESS_PHI_AIRS,
    SATURATION_PHI_AIRS,
};
use turbojet::state_coordinate::{
    build_state_coordinate_cascade, coord_census, coord_counters, coord_forced, coord_march,
    coord_scan, reset_coord_counters, CoordCensus, CoordCensusRow, CoordCounters, CoordForced,
    CoordForcedRow, CoordMarch, CoordRow, CoordScan, COORD_AT_DQ, COORD_CENSUS_WALK,
    PHI_REF_INCIDENCE, PHI_REF_PHI,
};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE_PYPY: &str = include_str!("../oracle/slice_ai_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/slice_ai_cpython.tsv");

/// The one key family every comparison excludes by NAME — see the dumper's header.
const INTERP: &str = "_interp/";

// ============================================================================ the grid
//
// The two suites' shared module constants, verbatim — and the dumper's copy of them.

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
/// Both suites' receiver wall — rung 79's `PHI_JAC`, rung 80's literal `0.80`.
const PHI_RIG: f64 = 0.80;
/// Rung 80's `PHI_FUEL`.
const PHI_FUEL: f64 = 0.75;
/// `tests/test_rung80.py`'s `gains` / `gains_demand` walls — WIDER than the reader's default.
const GAINS_AIRS: [Option<f64>; 3] = [None, Some(0.77), Some(0.80)];

// -------------------------------------------------- THE READERS' OWN DEFAULTS
//
// Rung 80's are the module's own exports (above). Rung 79's scalar defaults are typed here, as
// `rung79.rs` types them.

/// `coord_scan`'s, `coord_census`' and `coord_forced`'s `phi_lim` and `margin`, and `coord_march`'s.
const C_PHI: f64 = 0.80;
const C_MARGIN: f64 = 0.10;
/// Their `every`.
const C_EVERY: usize = 8;
/// `split_arrest`'s `phi_air_hi` and default `coord`.
const ARREST_AIR_HI: f64 = 0.80;
const ARREST_COORD: &str = "demand";
/// `split_saturation`'s default `coord`.
const SAT_COORD: &str = "demand";
/// `split_gains`' `every`.
const GAINS_EVERY: usize = 5;

/// Section I — the pre-flight's settings at its two walls; `ds` is the grid's `0.005`, not the
/// reader's own `0.002` default (the fingerprint passes `_S3_DS`).
const I_WALLS: [f64; 2] = [0.80, 0.76];
const I_EVERY: usize = 4;
/// Section P's stride and split wall — the dumper's `P_STRIDE` / `P_SPLIT_AIR`.
const P_STRIDE: usize = 5;
const P_SPLIT_AIR: f64 = 0.77;

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
/// **Hazard 3**: Python's `int(n) & 0xFFFFFFFFFFFFFFFF` — a negative arrives as two's complement.
fn vi64(n: i64) -> V { V::I(n as u64) }
fn vb(x: bool) -> V { V::B(x) }
fn vs(x: &str) -> V { V::S(x.to_string()) }
fn vopt_f(x: Option<f64>) -> V { x.map(V::F).unwrap_or(V::Null) }
fn vopt_b(x: Option<bool>) -> V { x.map(V::B).unwrap_or(V::Null) }
fn vopt_i(x: Option<usize>) -> V { x.map(vi).unwrap_or(V::Null) }
fn vfloats(xs: &[f64]) -> V { V::L(xs.iter().copied().map(V::F).collect()) }
fn vints(xs: &[usize]) -> V { V::L(xs.iter().copied().map(vi).collect()) }
fn vopt_floats(xs: &[Option<f64>]) -> V { V::L(xs.iter().copied().map(vopt_f).collect()) }
fn vstrs(xs: &[&str]) -> V { V::L(xs.iter().map(|x| vs(x)).collect()) }
fn vtaus(t: (f64, f64, f64, f64)) -> V { vfloats(&[t.0, t.1, t.2, t.3]) }
fn vauth(a: Option<Authority>) -> V { a.map(|a| vs(a.as_str())).unwrap_or(V::Null) }
/// Python's `dict(a=…, b=…)`, spelled with `&str` keys at the call site.
fn m(kv: Vec<(&str, V)>) -> V { V::M(kv.into_iter().map(|(k, v)| (k.to_string(), v)).collect()) }

fn fnv(text: &str) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for byte in text.as_bytes() {
        h ^= *byte as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

/// The emitted map, and the set of keys that carry a float, recorded AS THEY ARE EMITTED (AG
/// § 5.31.6 (c) 2).
struct Emit {
    map: BTreeMap<String, u64>,
    floats: BTreeSet<String>,
}

impl Emit {
    fn new() -> Self { Emit { map: BTreeMap::new(), floats: BTreeSet::new() } }

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

    /// Rung 79's six counter DELTAS, under the dumper's names.
    fn ctr(&mut self, tag: &str, c: CoordCounters) {
        for (k, v) in [("hits", c.hits), ("binds", c.binds), ("fb_phi", c.fb_phi),
                       ("fb_inc", c.fb_inc), ("calls_phi", c.calls_phi),
                       ("calls_inc", c.calls_inc)] {
            self.put(format!("{tag}/{k}"), v);
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
    /// `ResidualGaugeTransient` — section I's baseline, where the re-aim is introduced against.
    R78,
    /// `StateCoordinateTransient`.
    R79,
    /// `SplitWallTransient`.
    R80,
}

/// The dumper's `rig(cls, inc)` — the RECEIVER at `PHI_RIG`, with the knobs both suites set by
/// plain assignment (`tau_t` is the class default `None`, set as AH sets it).
fn rig(cls: Cls, inc: bool) -> ScheduledStatorCore {
    let arm = suite_arm(PHI_RIG / FLOOR - 1.0, inc);
    let built = match cls {
        Cls::R78 => build_residual_gauge_cascade(
            design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm),
        Cls::R79 => build_state_coordinate_cascade(
            design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm),
        Cls::R80 => build_split_wall_cascade(
            design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm),
    };
    let m = match built {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    };
    let t = &m.fuel.inner;
    t.lag_coord.set("demand");
    t.ref_law.set("sched");
    t.windup_law.set("none");
    t.tau_t.set(None);
    t.cap_law.set("solve");
    m
}

// ============================================================================ the converters
//
// Each is Python's returned dict, key for key, in Python's casing (hazard 6).

fn v_coord_row(r: &CoordRow) -> V {
    m(vec![
        ("s", vf(r.s)), ("w_phi", vf(r.w_phi)), ("w_inc", vf(r.w_inc)), ("d_set", vf(r.d_set)),
        ("same_float", vb(r.same_float)), ("slope_phi", vf(r.slope_phi)),
        ("slope_inc", vf(r.slope_inc)), ("ratio", vf(r.ratio)), ("dwdq_phi", vf(r.dwdq_phi)),
        ("dwdq_inc", vf(r.dwdq_inc)),
    ])
}

fn v_coord_scan(x: &CoordScan) -> V {
    m(vec![
        ("phi_lim", vf(x.phi_lim)), ("margin", vf(x.margin)), ("inc", vb(x.inc)), ("n", vi(x.n)),
        ("rows", V::L(x.rows.iter().map(v_coord_row).collect())),
        ("predicted_ratio", vf(x.predicted_ratio)), ("ratio_err", vopt_f(x.ratio_err)),
        ("dwdq_err", vopt_f(x.dwdq_err)), ("d_set", vopt_f(x.d_set)),
        ("d_set_min", vopt_f(x.d_set_min)), ("n_same_float", vi(x.n_same_float)),
    ])
}

fn v_census_row(r: &CoordCensusRow) -> V {
    m(vec![
        ("s", vf(r.s)), ("w0", vf(r.w0)), ("n_phi", vi(r.n_phi)), ("n_inc", vi(r.n_inc)),
        ("roots_phi", vfloats(&r.roots_phi)), ("roots_inc", vfloats(&r.roots_inc)),
        ("worst", vf(r.worst)),
    ])
}

fn v_coord_census(x: &CoordCensus) -> V {
    m(vec![
        ("phi_lim", vf(x.phi_lim)), ("margin", vf(x.margin)), ("inc", vb(x.inc)), ("n", vi(x.n)),
        ("rows", V::L(x.rows.iter().map(v_census_row).collect())),
        ("counts_equal", vb(x.counts_equal)), ("n_roots", vints(&x.n_roots)),
        ("worst", vopt_f(x.worst)),
    ])
}

fn v_coord_march(x: &CoordMarch) -> V {
    m(vec![
        ("phi_lim", vf(x.phi_lim)), ("margin", vf(x.margin)), ("inc", vb(x.inc)), ("n", vi(x.n)),
        ("same_len", vb(x.same_len)), ("worst", vf(x.worst)),
        // Hazard 4: Python's `where` — `(key, i)` or `None`.
        ("where", x.where_.map(|(k, i)| V::L(vec![vs(k), vi(i)])).unwrap_or(V::Null)),
        ("hits", V::I(x.hits)), ("binds", V::I(x.binds)), ("n_armed", vi(x.n_armed)),
        ("n_log", vi(x.n_log)), ("calls_phi", V::I(x.calls_phi)),
        ("calls_inc", V::I(x.calls_inc)), ("fb_phi", V::I(x.fb_phi)),
        ("fb_inc", V::I(x.fb_inc)), ("br_phi", vi64(x.br_phi)), ("br_inc", vi64(x.br_inc)),
        ("n_distinct", vi(x.n_distinct)), ("n_distinct_accel", vi(x.n_distinct_accel)),
        ("n_live", vi(x.n_live)), ("n_reach", vi(x.n_reach)), ("n_both", vi(x.n_both)),
        ("flips", vi(x.flips)), ("gap_min", vopt_f(x.gap_min)), ("gap_med", vopt_f(x.gap_med)),
        ("gap_max", vopt_f(x.gap_max)), ("n_distinct_gap", vi(x.n_distinct_gap)),
        ("d_max", vopt_f(x.d_max)), ("d_med", vopt_f(x.d_med)),
        ("vacuous", vopt_b(x.vacuous)), ("sched_moved", vf(x.sched_moved)),
    ])
}

fn v_forced_row(r: &CoordForcedRow) -> V {
    m(vec![
        ("s", vf(r.s)), ("binding", vb(r.binding)), ("w_phi", vf(r.w_phi)),
        ("w_inc", vf(r.w_inc)), ("w_shipped", vf(r.w_shipped)), ("d_forced", vf(r.d_forced)),
        ("d_shipped", vf(r.d_shipped)), ("same_float", vb(r.same_float)),
    ])
}

fn v_coord_forced(x: &CoordForced) -> V {
    m(vec![
        ("phi_lim", vf(x.phi_lim)), ("margin", vf(x.margin)), ("inc", vb(x.inc)), ("n", vi(x.n)),
        ("rows", V::L(x.rows.iter().map(v_forced_row).collect())),
        ("n_binding", vi(x.n_binding)), ("d_forced", vopt_f(x.d_forced)),
        ("d_forced_med", vopt_f(x.d_forced_med)), ("n_same_float", vi(x.n_same_float)),
        ("d_shipped", vopt_f(x.d_shipped)),
    ])
}

/// `_split_row`'s dict — 22 keys, with an optional leading `wall` for `split_arrest` (hazard 5).
fn v_split_row(r: &SplitRow, wall: Option<f64>) -> V {
    let mut kv: Vec<(&str, V)> = Vec::new();
    if let Some(w) = wall {
        kv.push(("wall", vf(w)));
    }
    kv.extend(vec![
        ("coord", vs(r.coord)), ("phi_lim", vf(r.phi_lim)), ("phi_air", vopt_f(r.phi_air)),
        ("npts", vi(r.npts)), ("phi_lim_built", vopt_f(r.phi_lim_built)),
        ("phi_air_built", vopt_f(r.phi_air_built)), ("phi0", vf(r.phi0)), ("b0", vf(r.b0)),
        ("v0", vf(r.v0)), ("b0_frac", vf(r.b0_frac)), ("min_phi", vf(r.min_phi)),
        ("max_Tt4", vf(r.max_tt4)), ("arrested", vb(r.arrested)),
        ("riding4_valid", vb(r.riding4_valid)), ("n_cut_fuel", vi(r.n_cut_fuel)),
        ("n_cut_gov", vi(r.n_cut_gov)), ("max_req_fuel", vf(r.max_req_fuel)),
        ("max_req_gov", vf(r.max_req_gov)), ("valve_moved", vi(r.valve_moved)),
        ("stator_moved", vi(r.stator_moved)), ("n_riding4", vi(r.n_riding4)),
        ("b_max_hit", vb(r.b_max_hit)),
    ]);
    m(kv)
}

fn v_split_liveness(x: &SplitLiveness) -> V {
    m(vec![
        ("phi_lim", vf(x.phi_lim)), ("phi_airs", vopt_floats(&x.phi_airs)),
        ("coords", vstrs(&x.coords)), ("taus", vtaus(x.taus)), ("ds", vf(x.ds)),
        ("rows", V::L(x.rows.iter().map(|r| v_split_row(r, None)).collect())),
        ("control_ok", vb(x.control_ok)), ("levers_woke", vb(x.levers_woke)),
        ("fuel_off", vb(x.fuel_off)), ("four_live", vi(x.four_live)), ("n_split", vi(x.n_split)),
    ])
}

fn v_arrest_arm(a: &ArrestArm) -> V {
    m(vec![
        ("rows", V::L(a.rows.iter().map(|(w, r)| v_split_row(r, Some(*w))).collect())),
        ("marched", vfloats(&a.marched)), ("arrested", vfloats(&a.arrested)),
        ("last_march", vopt_f(a.last_march)), ("first_arrest", vopt_f(a.first_arrest)),
        ("monotone", vb(a.monotone)),
    ])
}

fn v_split_arrest(x: &SplitArrest) -> V {
    m(vec![
        ("walls", vfloats(&x.walls)), ("phi_lim_lo", vf(x.phi_lim_lo)),
        ("phi_air_hi", vf(x.phi_air_hi)), ("coord", vs(x.coord)), ("taus", vtaus(x.taus)),
        ("ds", vf(x.ds)),
        ("arms", V::M(x.arms.iter().map(|(k, a)| (k.to_string(), v_arrest_arm(a))).collect())),
        ("control_bracket",
         V::L(vec![vopt_f(x.control_bracket.0), vopt_f(x.control_bracket.1)])),
        ("owner", vstrs(&x.owner)),
    ])
}

fn v_split_saturation(x: &SplitSaturation) -> V {
    m(vec![
        ("phi_lim", vf(x.phi_lim)), ("coord", vs(x.coord)),
        ("rows", V::L(x.rows.iter().map(|r| v_split_row(r, None)).collect())),
        ("ds", vf(x.ds)), ("first_sat", vopt_f(x.first_sat)),
        ("last_march", vopt_f(x.last_march)), ("cell", vfloats(&x.cell)),
        ("impossible", vb(x.impossible)),
    ])
}

fn v_split_cell(c: &SplitCell) -> V {
    m(vec![
        ("s", vf(c.s)), ("phi", vf(c.phi)), ("authority", vauth(c.authority)),
        ("masked", vauth(c.masked)), ("mask_leak", vopt_f(c.mask_leak)),
        ("zeros", vi(c.zeros)), ("c1", vf(c.c1)), ("c0", vf(c.c0)),
        ("cyc_fwd", vf(c.cyc_fwd)), ("cyc_rev", vf(c.cyc_rev)), ("pair_RC", vf(c.pair_rc)),
        ("pair_CV", vf(c.pair_cv)),
    ])
}

/// **Hazard 2: the authority dict's key is the label's `%s`, `None` included.**
fn auth_key(a: Option<Authority>) -> String {
    a.map(|a| a.as_str().to_string()).unwrap_or_else(|| "None".to_string())
}

fn v_gains_arm(a: &GainsArm) -> V {
    m(vec![
        ("phi_air", vopt_f(a.phi_air)), ("n_riding", vi(a.n_riding)),
        ("n_sampled", vi(a.n_sampled)), ("n_interior", vi(a.n_interior)),
        // Hazard 1: SplitGains holds `(switch, regime)`.
        ("skipped", m(vec![("switch", vi(a.skipped.0)), ("regime", vi(a.skipped.1))])),
        ("cells", V::L(a.cells.iter().map(v_split_cell).collect())),
        ("authority", V::M(a.authority.iter().map(|(k, n)| (auth_key(*k), vi(*n))).collect())),
        ("masked", V::L(a.masked.iter().map(|x| vauth(*x)).collect())),
        ("max_mask_leak", vopt_f(a.max_mask_leak)), ("zeros", vints(&a.zeros)),
        ("max_cyc", vopt_f(a.max_cyc)),
    ])
}

fn v_split_gains(x: &SplitGains) -> V {
    m(vec![
        ("phi_lim", vf(x.phi_lim)), ("coord", vs(x.coord)), ("taus", vtaus(x.taus)),
        ("ds", vf(x.ds)), ("arms", V::L(x.arms.iter().map(v_gains_arm).collect())),
        ("vacuous", vb(x.vacuous)), ("n_interior", vints(&x.n_interior)),
        ("all_differenced", vb(x.all_differenced)),
        ("ever_two_authorities", vb(x.ever_two_authorities)),
        ("control_nonzero", vopt_f(x.control_nonzero)),
        ("max_mask_leak", vopt_f(x.max_mask_leak)),
    ])
}

fn v_demand_row(r: &turbojet::demand_coordinate::GainRow) -> V {
    m(vec![
        ("s", vf(r.s)), ("authority", vs(r.authority.as_str())), ("poly_gap", vf(r.poly_gap)),
        ("poly_scale", vf(r.poly_scale)), ("worst_flip", vf(r.worst_flip)),
        ("worst_keep", vf(r.worst_keep)), ("n_sign_changed", vi(r.n_sign_changed)),
        ("biggest_moved", vf(r.biggest_moved)), ("mask_leak_w", vopt_f(r.mask_leak_w)),
        ("mask_leak_g", vopt_f(r.mask_leak_g)), ("pairs_gap", vf(r.pairs_gap)),
    ])
}

fn v_demand_gains(x: &DemandGains) -> V {
    m(vec![
        ("inc", vb(x.inc)), ("phi_lim", vf(x.phi_lim)), ("taus", vtaus(x.taus)),
        ("ds", vf(x.ds)), ("n", vi(x.n)),
        ("rows", V::L(x.rows.iter().map(v_demand_row).collect())),
        // Hazard 1: DemandGains holds `(regime, switch)`.
        ("skipped", m(vec![("regime", vi(x.skipped.0)), ("switch", vi(x.skipped.1))])),
        ("worst_poly_gap", vopt_f(x.worst_poly_gap)),
        ("worst_poly_rel", vopt_f(x.worst_poly_rel)), ("worst_flip", vopt_f(x.worst_flip)),
        ("worst_keep", vopt_f(x.worst_keep)), ("worst_pairs_gap", vopt_f(x.worst_pairs_gap)),
        ("worst_mask_leak", vopt_f(x.worst_mask_leak)),
        ("min_sign_changed", vopt_i(x.min_sign_changed)),
        ("biggest_moved", vopt_f(x.biggest_moved)),
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

/// A marched point, WHOLE, as the dict Python records — 30 keys on a CLIP march (`Shared`), 35 on
/// a DEMAND one. AH's converter, unchanged.
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

/// Python's `min(col)` / `max(col)`: the FIRST extremum, replaced only when STRICTLY beaten.
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

/// Python's `"%s" % wall` for the section-I prefix — `repr`, which `{:?}` reproduces here.
fn wall_key(w: f64) -> String { format!("{w:?}") }

/// Sections A-D — rung 79's four readers.
fn drive_79(e: &mut Emit, fl: &FlightCondition, with_march: bool) {
    for inc in [false, true] {
        let p = arms(e, "A", inc);
        e.walk(&p, &v_coord_scan(&coord_scan(
            &rig(Cls::R79, inc), fl, LO, HI, TT4_MAX, C_PHI, C_MARGIN, TAUS, inc, R, SETTLE, DS,
            V_MAX, COORD_AT_DQ, C_EVERY)));
    }
    for inc in [false, true] {
        let p = arms(e, "B", inc);
        let (lo, hi, n) = COORD_CENSUS_WALK;
        e.walk(&p, &v_coord_census(&coord_census(
            &rig(Cls::R79, inc), fl, LO, HI, TT4_MAX, C_PHI, C_MARGIN, TAUS, inc, R, SETTLE, DS,
            V_MAX, C_EVERY, lo, hi, n)));
    }
    if with_march {
        drive_c(e, fl);
    }
    for inc in [false, true] {
        let p = arms(e, "D", inc);
        e.walk(&p, &v_coord_forced(&coord_forced(
            &rig(Cls::R79, inc), fl, LO, HI, TT4_MAX, C_PHI, C_MARGIN, TAUS, inc, R, SETTLE, DS,
            V_MAX, C_EVERY)));
    }
}

/// Section C — `coord_march`, which resets and reads rung 79's counters itself.
fn drive_c(e: &mut Emit, fl: &FlightCondition) {
    for inc in [false, true] {
        let p = arms(e, "C", inc);
        e.walk(&p, &v_coord_march(&coord_march(
            &rig(Cls::R79, inc), fl, LO, HI, TT4_MAX, C_PHI, C_MARGIN, TAUS, inc, R, SETTLE, DS,
            V_MAX)));
    }
}

/// Sections E-J — rung 80's four readers at the suite's arguments.
fn drive_80(e: &mut Emit, fl: &FlightCondition) {
    for inc in [false, true] {
        let p = arms(e, "E", inc);
        e.walk(&p, &v_split_liveness(&split_liveness(
            &rig(Cls::R80, inc), fl, LO, HI, TT4_MAX, PHI_FUEL, &LIVENESS_PHI_AIRS,
            &LIVENESS_COORDS, TAUS, inc, R, SETTLE, DS, V_MAX)));
    }
    for inc in [false, true] {
        let p = arms(e, "F", inc);
        e.walk(&p, &v_split_arrest(&split_arrest(
            &rig(Cls::R80, inc), fl, LO, HI, TT4_MAX, &ARREST_WALLS, PHI_FUEL, ARREST_AIR_HI,
            ARREST_COORD, TAUS, inc, R, SETTLE, DS, V_MAX)));
    }
    for inc in [false, true] {
        let p = arms(e, "G", inc);
        e.walk(&p, &v_split_saturation(&split_saturation(
            &rig(Cls::R80, inc), fl, LO, HI, TT4_MAX, PHI_FUEL, &SATURATION_PHI_AIRS, SAT_COORD,
            TAUS, inc, R, SETTLE, DS, V_MAX)));
    }
    for (tag, coord) in [("H", "clip"), ("J", "demand")] {
        for inc in [false, true] {
            let p = arms(e, tag, inc);
            let g = split_gains(&rig(Cls::R80, inc), fl, LO, HI, TT4_MAX, PHI_FUEL, &GAINS_AIRS,
                                coord, TAUS, inc, R, SETTLE, DS, V_MAX, GAINS_EVERY)
                .unwrap_or_else(|e| panic!("{}", e.0));
            e.walk(&p, &v_split_gains(&g));
        }
    }
}

/// Section I — § 5.33 (i)'s arm: rung 74's `demand_gains`, counter DELTAS beside it.
fn drive_i(e: &mut Emit, fl: &FlightCondition) {
    for (tag, cls) in [("r78", Cls::R78), ("r79", Cls::R79), ("r80", Cls::R80)] {
        for wall in I_WALLS {
            for inc in [false, true] {
                let p = arms(e, &format!("I/{tag}/w{}", wall_key(wall)), inc);
                let core = rig(cls, inc);
                reset_coord_counters();
                let g = demand_gains(&core, fl, LO, HI, TT4_MAX, wall, TAUS, inc, R, SETTLE, DS,
                                     V_MAX, I_EVERY);
                let c = coord_counters();
                e.walk(&format!("{p}/g"), &v_demand_gains(&g));
                e.ctr(&format!("{p}/ctr"), c);
            }
        }
    }
}

/// Section P, rung 79's half — the INCIDENCE plant, `coord_march`'s `traj1` without the probe.
fn drive_p79(e: &mut Emit, fl: &FlightCondition, inc: bool, p: &str) {
    let core = rig(Cls::R79, inc);
    let t = &core.fuel.inner;
    let sm = PHI_RIG / core.arming().map_lp_design.phi_surge - 1.0;
    let acc = {
        let _cs = CoordScope::set(t, PHI_REF_PHI);
        accel_for(&core, fl, LO, HI, sm, TT4_MAX, TAUS, V_MAX, inc, C_MARGIN)
    };
    reset_coord_counters();
    let traj = {
        let _cs = CoordScope::set(t, PHI_REF_INCIDENCE);
        cap_march(&core, fl, LO, HI, TT4_MAX, sm, TAUS, R, SETTLE, DS, V_MAX, inc, "demand",
                  "sched", "none", None, "solve", &acc, None).3
    };
    let c = coord_counters();
    e.ctr(&format!("{p}/coord/ctr"), c);
    emit_march(e, &format!("{p}/coord"), &traj);
}

/// Section P, rung 80's half — `_split_march` at one split wall, both coordinates.
fn drive_p80(e: &mut Emit, fl: &FlightCondition, inc: bool, p: &str) {
    for coord in ["demand", "clip"] {
        let core = rig(Cls::R80, inc);
        reset_coord_counters();
        let traj = split_march(&core, fl, LO, HI, TT4_MAX, PHI_FUEL, Some(P_SPLIT_AIR), coord,
                               TAUS, R, SETTLE, DS, V_MAX, inc).3;
        let c = coord_counters();
        e.ctr(&format!("{p}/split_{coord}/ctr"), c);
        emit_march(e, &format!("{p}/split_{coord}"), &traj);
    }
}

fn run_drive() -> BTreeMap<String, u64> {
    let mut e = Emit::new();
    let fl = flight();
    drive_79(&mut e, &fl, true);
    drive_80(&mut e, &fl);
    drive_i(&mut e, &fl);
    for inc in [false, true] {
        let p = arms(&mut e, "P", inc);
        drive_p79(&mut e, &fl, inc, &p);
        drive_p80(&mut e, &fl, inc, &p);
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

/// The COUNTER-BEARING sections only — C, I and P's rung-79 plant — for P3's two-thread gate.
/// Every key it emits is a key of the full drive, so each is compared to the golden directly.
fn counter_drive() -> BTreeMap<String, u64> {
    let mut e = Emit::new();
    let fl = flight();
    drive_c(&mut e, &fl);
    drive_i(&mut e, &fl);
    for inc in [false, true] {
        let p = arms(&mut e, "P", inc);
        drive_p79(&mut e, &fl, inc, &p);
    }
    e.map
}

/// ONE full drive per binary. Rung 79's counters are thread-local, so this is a COST choice here
/// and not AH's race guard — P3's gate is the one that runs the counters on several threads.
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

fn pypy() -> &'static BTreeMap<String, u64> {
    static G: OnceLock<BTreeMap<String, u64>> = OnceLock::new();
    G.get_or_init(|| golden(ORACLE_PYPY).0)
}

/// `(key, rust, python)` for every golden key in `theirs` whose value differs from `mine`.
fn diffs<'a>(mine: &BTreeMap<String, u64>, theirs: &'a BTreeMap<String, u64>)
    -> Vec<(&'a str, Option<u64>, u64)> {
    theirs.iter()
        .filter_map(|(k, v)| match mine.get(k) {
            Some(got) if got == v => None,
            got => Some((k.as_str(), got.copied(), *v)),
        })
        .collect()
}

fn show(d: &[(&str, Option<u64>, u64)]) -> String {
    d.iter().take(25)
        .map(|(k, got, v)| match got {
            None => format!("{k}: MISSING"),
            Some(g) => format!("{k}: rust {g} ({}) vs python {v} ({})",
                               f64::from_bits(*g), f64::from_bits(*v)),
        })
        .collect::<Vec<_>>().join("\n  ")
}

/// **THE KEY SETS ARE COMPARED BEFORE ANY VALUE IS** — the module header's reason for the
/// asymmetry.
#[test]
fn the_two_key_sets_are_equal() {
    let mine: BTreeSet<&String> = drive().keys().collect();
    let theirs: BTreeSet<&String> = pypy().keys().collect();
    let missing: Vec<&&String> = theirs.difference(&mine).take(20).collect();
    let extra: Vec<&&String> = mine.difference(&theirs).take(20).collect();
    assert!(missing.is_empty() && extra.is_empty(),
            "key sets differ: {} missing (first 20 {missing:?}), {} extra (first 20 {extra:?})",
            theirs.difference(&mine).count(), mine.difference(&theirs).count());
    assert!(!mine.is_empty(), "an empty key set agrees with an empty golden");
}

/// **Every key bit-identical to PyPy** — P2a's value half and its counter half together, since
/// section I's counter deltas are keys like any other.
#[test]
fn every_value_is_bit_identical_to_the_pypy_golden() {
    let bad = diffs(drive(), pypy());
    assert!(bad.is_empty(), "{} of {} keys differ:\n  {}", bad.len(), pypy().len(), show(&bad));
    println!("PyPy arm: {} keys compared, 0 read as input", pypy().len());
}

/// **THE TWO GOLDENS ARE TWO RUNS, SAID BY THE ONE KEY BUILT TO SAY IT** — AH's sentinel.
#[test]
fn the_two_goldens_carry_their_own_interpreters_sum() {
    let key = format!("{INTERP}sum_probe");
    let (_, pypy) = golden(ORACLE_PYPY);
    let (_, cpython) = golden(ORACLE_CPYTHON);
    assert_eq!(pypy.get(&key).copied(), Some(0.0_f64.to_bits()),
               "the PyPy golden must carry PyPy's naive `sum`");
    assert_eq!(cpython.get(&key).copied(), Some(1.0_f64.to_bits()),
               "the CPython golden must carry CPython's compensated `sum`");
    let naive = [1e16_f64, 1.0, -1e16].iter().fold(0.0, |a, x| a + x);
    assert_eq!(naive.to_bits(), 0.0_f64.to_bits());
}

/// **THE CPYTHON PREDICTION, WRITTEN AS A CAUSAL PATH BEFORE THE CPYTHON GOLDEN WAS COMPARED.**
///
/// Read off a RUNTIME `builtins.sum` census over this exact drive
/// (`W:/temp/claude/slice-ai-step6/probe_drive.py`, wrapper installed BEFORE `turbojet` was
/// imported), not off the pre-flight's class-scoped § (v). The drive reaches exactly two float
/// summations in the model:
///
/// * **`_charpoly4`** (`engine.py:16235` / `:16237`), 1 004 calls — rung 72's INHERITED helper, AG's
///   398 keys and AF's 49. It feeds `split_gains`' `c1` / `c0` and, through the quartic's roots,
///   `zeros`; and `demand_gains`' `poly_gap` / `poly_scale` and their two worsts.
/// * **`split_gains`' `rate`** (`engine.py:21750`), 23 calls, four terms — it enters only the
///   `zeros` threshold.
///
/// Everything else the census saw is an integer count, this file's sentinel, or `gas.py`'s
/// import-time constants — which AH's CPython arm already showed bit-identical on this gas.
///
/// **THE PREDICTION: a key may differ ONLY if it is one of those readings** — [`via_sum`]'s path
/// predicate — and at least one does (AF's `poly_gap`s did, on the same reader). **Every rung-79
/// key, every other rung-80 key, both plants and all counters are bit-identical.**
/// **THE FALSIFIER:** any differing key outside the predicate. The gate is not relaxed to fit one.
fn via_sum(k: &str) -> bool {
    let seg: Vec<&str> = k.split('/').collect();
    let last = seg.last().copied().unwrap_or("");
    match seg.first().copied() {
        // split_gains: per-cell `c1`, `c0`, `zeros`, and the arm's sorted `zeros` set
        Some("H") | Some("J") => {
            matches!(last, "c1" | "c0" | "zeros")
                || seg.windows(2).any(|w| w[0] == "zeros" && (w[1] == "#len"
                                                             || w[1].parse::<usize>().is_ok()))
        }
        // demand_gains: the charpoly readings
        Some("I") => matches!(last, "poly_gap" | "poly_scale" | "worst_poly_gap"
                                    | "worst_poly_rel"),
        _ => false,
    }
}

#[test]
fn every_value_is_bit_identical_to_the_cpython_golden_off_the_two_sum_paths() {
    let (theirs, _) = golden(ORACLE_CPYTHON);
    let mine = drive();
    let bad = diffs(mine, &theirs);
    let outside: Vec<_> = bad.iter().filter(|(k, _, _)| !via_sum(k)).cloned().collect();
    assert!(outside.is_empty(),
            "{} CPython differences OFF the two predicted sum paths -- the prediction is \
             FALSIFIED:\n  {}", outside.len(), show(&outside));
    let covered = theirs.keys().filter(|k| via_sum(k)).count();
    assert!(!bad.is_empty(), "no key differs at all -- `_charpoly4`'s drift did not reach this \
                              grid, or the CPython file is the PyPy one");
    for (k, got, v) in &bad {
        let (g, p) = (f64::from_bits(got.expect("key sets are gated separately")),
                      f64::from_bits(*v));
        eprintln!("  CPython drift {k}: rust {g:e} vs cpython {p:e}");
    }
    println!("CPython arm: {} keys, {} differ, all on the {} keys of the two sum paths",
             theirs.len(), bad.len(), covered);
}

/// **P2b — THE RE-AIM MOVES THE COUNTERS AND NO VALUE.** Section I on R79 and R80 against R78,
/// key for key under `/g`, at both walls and both arms: every value equal. And the counters are
/// NOT equal — R78 must read zero hits and R79/R80 a positive count with `fb_inc = calls_inc`, so
/// the equality above is a masked branch and not an unreached one (§ 5.33 (i)'s two masks).
///
/// Read off the Rust drive, which [`every_value_is_bit_identical_to_the_pypy_golden`] pins to the
/// golden key for key — so this is the Python fact, re-asserted in the port.
#[test]
fn p2b_the_re_aim_moves_counters_and_no_value() {
    let d = drive();
    let mut compared = 0usize;
    for wall in I_WALLS {
        for inc in [0, 1] {
            let base = format!("I/r78/w{}/i{inc}", wall_key(wall));
            let r78: BTreeMap<&str, u64> = d.range(format!("{base}/g")..)
                .take_while(|(k, _)| k.starts_with(&format!("{base}/g")))
                .map(|(k, v)| (&k[base.len()..], *v)).collect();
            assert!(r78.len() > 20, "section I's rung-78 reading is empty at {base}");
            let hits78 = d[&format!("{base}/ctr/hits")];
            assert_eq!(hits78, 0, "rung 78 has no rung-79 branch to enter: {base}");
            for tag in ["r79", "r80"] {
                let here = format!("I/{tag}/w{}/i{inc}", wall_key(wall));
                let mine: BTreeMap<&str, u64> = d.range(format!("{here}/g")..)
                    .take_while(|(k, _)| k.starts_with(&format!("{here}/g")))
                    .map(|(k, v)| (&k[here.len()..], *v)).collect();
                assert_eq!(mine, r78, "{here}: the re-aim MOVED a value against rung 78");
                compared += mine.len();
                let (hits, fbi, ci) = (d[&format!("{here}/ctr/hits")],
                                       d[&format!("{here}/ctr/fb_inc")],
                                       d[&format!("{here}/ctr/calls_inc")]);
                assert!(hits > 0, "{here}: the rung-79 branch never ran -- the equality above \
                                   would then be an UNREACHED branch, not a masked one");
                assert_eq!(fbi, ci, "{here}: an incidence solve BRACKETED -- mask 2 broke");
            }
        }
    }
    println!("P2b: {compared} value keys equal to rung 78's across 2 walls x 2 arms x 2 rungs");
}

/// **P3 — THREAD-LOCAL COUNTERS UNDER PARALLEL EXECUTION.** Two threads run every counter-bearing
/// section AT ONCE, and each must match the golden on every key it emits. A process-global counter
/// here would sum both threads' marches into one reading; a single drive could not tell the two
/// apart, which is why this gate spawns and the full drive does not.
#[test]
fn p3_two_threads_marching_at_once_both_match_the_golden() {
    let (a, b) = std::thread::scope(|s| {
        let ha = s.spawn(counter_drive);
        let hb = s.spawn(counter_drive);
        (ha.join().expect("thread A"), hb.join().expect("thread B"))
    });
    for (name, got) in [("A", &a), ("B", &b)] {
        let theirs: BTreeMap<String, u64> = got.keys()
            .map(|k| (k.clone(), *pypy().get(k).unwrap_or_else(|| panic!("{k}: not in the golden"))))
            .collect();
        let bad = diffs(got, &theirs);
        assert!(bad.is_empty(), "thread {name}: {} of {} keys differ:\n  {}", bad.len(),
                theirs.len(), show(&bad));
        let n_ctr = got.keys().filter(|k| k.contains("/ctr/") || k.ends_with("/hits")).count();
        assert!(n_ctr > 0, "thread {name} emitted no counter key");
    }
    assert_eq!(a, b, "the two threads disagree with each other");
    println!("P3: {} keys per thread, both threads equal to the golden", a.len());
}
