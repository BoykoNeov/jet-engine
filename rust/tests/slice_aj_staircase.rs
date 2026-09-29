//! SLICE AJ step 4, rung 84 — `edge_read`, `classify`, `staircase_scan`, `lattice_count`,
//! `staircase_number` and `root_class`, **every returned value bit for bit, in Python's own key
//! order.**
//!
//! # WHAT IS PINNED
//!
//! Twenty-two readings on `tests/test_rung84.py`'s rig, each against
//! `oracle/probe_slice_aj_step4.py`'s output (`oracle/slice_aj_step4_pypy.tsv`, PyPy, the repo
//! venv; rung 83's readings share the file and are gated in `slice_aj_corrector.rs`). The suite's
//! own calls where it has one — rung 83 § 3.2's and § 3.4's read pairs and their `classify`,
//! `test_p4`'s ladder, `test_p3`'s count at two steps, `test_p7`'s coarse `root_class` — and, for
//! branches the suite never takes:
//!
//! * `classify_rev` — the jump pair REVERSED: `left` non-empty, the sign change mirrored.
//! * `edge_v1` / `classify_v1` — the EMPTY window: no summands, every Option `None`, `kind = None`.
//! * `lattice_v3` — Python's misnamed `V3: an edge off the march grid` at `r = 1.0`.
//! * `sn_ok` / `sn_bare` / `sn_zero` — one edge move with a spacing, with none, and with `0.0`
//!   (Python's `if spacing` is TRUTHINESS: no tread, but `spacing = 0.0` reported).
//! * `sn_v5` at `0.0198123456` — `%g` prints `0.0198123`, `{}` would not; `sn_v2` at `r = 1.0`.
//! * `root_cross` — a CROSSING (`root_exists = True`, `d_membership = 0.0`) at the shipped step;
//!   `root_v1` — the bisection's V1 carried; `root_v6` — `eps = 0.123456789` (`%g`: `0.123457`).
//!
//! `summands`, Python's float-keyed dict, is flattened as a LIST of `[key, value]` pairs: the key
//! bits pinned, insertion order kept, and no float spelled into a path.
//!
//! # UNREACHED ON THIS RIG — recorded, not implied by a green run (plan § 5.34.4)
//!
//! `n_slope_excluded > 0` (so `n_scored` = the cell count here, and a port counting cells would
//! pass), a riding point at a trajectory END (so `edge` = the first scored cell here), a merged
//! `round(s, 9)` key within one march, an edge off the grid on an OPEN window, a non-monotone
//! edge, and a summand exactly `0.0`.

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::split_wall::build_split_wall_cascade;
use turbojet::staircase_law::{
    self, Classified, EdgeRead, LatticeCount, RootClass, StaircaseNumber, StaircaseScan, EPS,
    N_BISECT,
};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::threshold_law::{ScanKw, BRACKET};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE: &str = include_str!("../oracle/slice_aj_step4_pypy.tsv");

// ---------------------------------------------------------------------------- the rig
//
// `tests/test_rung84.py`'s module constants (identical to `test_rung83.py`'s).

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const SM: f64 = 0.80 / FLOOR - 1.0;
const B: f64 = 0.10;
const V_MAX: f64 = 0.20;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const TT4_MAX: f64 = 1200.0;
const PHI_FUEL: f64 = 0.75;
const PHI_AIR: Option<f64> = Some(0.77);
const DS: f64 = 0.005;
/// § 3.2's jump window, `r = 0.25`.
const JUMP_LO: f64 = 0.0197750;
const JUMP_HI: f64 = 0.0197875;
/// § 3.4's crossing window, `r = 0.35`.
const CROSS_LO: f64 = 0.037000;
const CROSS_HI: f64 = 0.037333;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

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

/// Python's `_rig`, with its four knob assignments. The class is `StaircaseLawTransient` there;
/// here it is a rung-80 core, because rung 84 adds no cell (plan § 5.34 (ii)).
fn rig() -> ScheduledStatorCore {
    let arm = LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, SM, Some(TAU))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU_S))),
        ..Default::default()
    };
    let m = match build_split_wall_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    };
    let t = &m.fuel.inner;
    t.lag_coord.set("demand");
    t.ref_law.set("sched");
    t.windup_law.set("none");
    t.cap_law.set("solve");
    m
}

/// `test_rung84.py`'s `_kw(r, ds)`.
fn kw(f: &FlightCondition, r: f64, ds: f64) -> ScanKw<'_> {
    ScanKw {
        flight: f, tt4_lo: LO, tt4_hi: HI, tt4_max: TT4_MAX, phi_lim: PHI_FUEL, phi_air: PHI_AIR,
        tau_gov: 0.05, tau_q: 0.05, tau_s: 0.05, r, s_settle: 1.2, ds, v_max: 0.20,
        inc: false,
    }
}

// ---------------------------------------------------------------------------- the flattening

#[derive(Default)]
struct Flat(Vec<(String, String)>);

impl Flat {
    fn put(&mut self, p: &str, t: String) {
        self.0.push((p.to_string(), t));
    }
    fn f(&mut self, p: &str, x: f64) {
        if x.is_nan() {
            self.put(p, "f:nan".into());
        } else {
            self.put(p, format!("f:{:016x}", x.to_bits()));
        }
    }
    fn of(&mut self, p: &str, x: Option<f64>) {
        match x {
            Some(x) => self.f(p, x),
            None => self.none(p),
        }
    }
    fn i(&mut self, p: &str, x: usize) {
        self.put(p, format!("i:{x}"));
    }
    fn b(&mut self, p: &str, x: bool) {
        self.put(p, format!("b:{}", x as u8));
    }
    fn s(&mut self, p: &str, x: &str) {
        self.put(p, format!("s:{x}"));
    }
    fn os(&mut self, p: &str, x: Option<&str>) {
        match x {
            Some(x) => self.s(p, x),
            None => self.none(p),
        }
    }
    fn none(&mut self, p: &str) {
        self.put(p, "n".into());
    }
    fn len(&mut self, p: &str, n: usize) {
        self.put(p, format!("len:{n}"));
    }
    fn keys(&mut self, p: &str, n: usize) {
        self.put(p, format!("keys:{n}"));
    }
    fn fs(&mut self, p: &str, xs: &[f64]) {
        self.len(p, xs.len());
        for (k, &x) in xs.iter().enumerate() {
            self.f(&format!("{p}.{k}"), x);
        }
    }
}

/// `test_p5`'s spacing formula at `ds = 0.005`: `ds·(hi - lo)/ds_star`.
const SPACING: f64 = 0.005 * (0.024 - 0.016) / 0.005938;

impl Flat {
    fn oi(&mut self, p: &str, x: Option<i64>) {
        match x {
            Some(x) => self.put(p, format!("i:{x}")),
            None => self.none(p),
        }
    }
    fn kind(&mut self, p: &str, k: Option<staircase_law::Kind>) {
        self.os(p, k.map(|k| k.as_str()));
    }
}

fn edge(o: &mut Flat, p: &str, e: &EdgeRead) {
    o.keys(p, 19);
    o.f(&format!("{p}.tau_f"), e.tau_f);
    o.f(&format!("{p}.r"), e.r);
    o.f(&format!("{p}.ds"), e.ds);
    o.of(&format!("{p}.h"), e.h);
    o.of(&format!("{p}.kappa"), e.kappa);
    o.b(&format!("{p}.kappa_pure"), e.kappa_pure);
    o.of(&format!("{p}.F"), e.f);
    o.of(&format!("{p}.g"), e.g);
    o.len(&format!("{p}.summands"), e.summands.len());
    for (k, &(s, v)) in e.summands.iter().enumerate() {
        o.len(&format!("{p}.summands.{k}"), 2);
        o.f(&format!("{p}.summands.{k}.0"), s);
        o.f(&format!("{p}.summands.{k}.1"), v);
    }
    o.of(&format!("{p}.s_bind"), e.s_bind);
    o.of(&format!("{p}.edge"), e.edge);
    o.b(&format!("{p}.at_edge"), e.at_edge);
    o.i(&format!("{p}.n_ride"), e.n_ride);
    o.i(&format!("{p}.n_scored"), e.n_scored);
    o.i(&format!("{p}.n_slope_excluded"), e.n_slope_excluded);
    o.b(&format!("{p}.window_open"), e.window_open);
    o.b(&format!("{p}.riding4_valid"), e.riding4_valid);
    o.b(&format!("{p}.edge_on_grid"), e.edge_on_grid);
    o.oi(&format!("{p}.edge_index"), e.edge_index);
}

fn classified(o: &mut Flat, p: &str, c: &Classified) {
    o.keys(p, 17);
    o.f(&format!("{p}.tau_lo"), c.tau_lo);
    o.f(&format!("{p}.tau_hi"), c.tau_hi);
    o.fs(&format!("{p}.entered"), &c.entered);
    o.fs(&format!("{p}.left"), &c.left);
    o.b(&format!("{p}.set_changed"), c.set_changed);
    o.b(&format!("{p}.argmin_moved"), c.argmin_moved);
    o.b(&format!("{p}.edge_moved"), c.edge_moved);
    o.of(&format!("{p}.h_lo"), c.h_lo);
    o.of(&format!("{p}.h_hi"), c.h_hi);
    o.of(&format!("{p}.h_common_lo"), c.h_common_lo);
    o.of(&format!("{p}.h_common_hi"), c.h_common_hi);
    o.of(&format!("{p}.d_full"), c.d_full);
    o.of(&format!("{p}.d_smooth"), c.d_smooth);
    o.of(&format!("{p}.d_membership"), c.d_membership);
    o.b(&format!("{p}.sign_change"), c.sign_change);
    o.b(&format!("{p}.sign_change_common"), c.sign_change_common);
    o.kind(&format!("{p}.kind"), c.kind);
}

fn ladder(o: &mut Flat, p: &str, s: &StaircaseScan) {
    o.keys(p, 22);
    o.f(&format!("{p}.lo"), s.lo);
    o.f(&format!("{p}.hi"), s.hi);
    o.i(&format!("{p}.n"), s.n);
    o.len(&format!("{p}.points"), s.points.len());
    for (k, x) in s.points.iter().enumerate() {
        edge(o, &format!("{p}.points.{k}"), x);
    }
    o.len(&format!("{p}.pairs"), s.pairs.len());
    for (k, x) in s.pairs.iter().enumerate() {
        classified(o, &format!("{p}.pairs.{k}"), x);
    }
    o.len(&format!("{p}.changes"), s.changes.len());
    for (k, x) in s.changes.iter().enumerate() {
        classified(o, &format!("{p}.changes.{k}"), x);
    }
    o.i(&format!("{p}.n_at_edge"), s.n_at_edge);
    o.i(&format!("{p}.n_points"), s.n_points);
    o.b(&format!("{p}.all_at_edge"), s.all_at_edge);
    o.i(&format!("{p}.n_sign_changes"), s.n_sign_changes);
    o.i(&format!("{p}.n_crossings"), s.n_crossings);
    o.i(&format!("{p}.n_jumps"), s.n_jumps);
    o.b(&format!("{p}.exact_zero_when_set_equal"), s.exact_zero_when_set_equal);
    o.b(&format!("{p}.nonzero_when_set_differs"), s.nonzero_when_set_differs);
    o.i(&format!("{p}.n_argmin_only"), s.n_argmin_only);
    o.i(&format!("{p}.n_set_only"), s.n_set_only);
    o.i(&format!("{p}.n_edge_moves"), s.n_edge_moves);
    o.b(&format!("{p}.all_on_grid"), s.all_on_grid);
    o.b(&format!("{p}.edge_monotone"), s.edge_monotone);
    o.len(&format!("{p}.edge_indices"), s.edge_indices.len());
    for (k, &x) in s.edge_indices.iter().enumerate() {
        o.oi(&format!("{p}.edge_indices.{k}"), x);
    }
    o.b(&format!("{p}.all_open"), s.all_open);
    o.b(&format!("{p}.all_kappa_pure"), s.all_kappa_pure);
}

fn lattice(o: &mut Flat, p: &str, c: &LatticeCount) {
    o.keys(p, 14);
    o.f(&format!("{p}.lo"), c.lo);
    o.f(&format!("{p}.hi"), c.hi);
    o.f(&format!("{p}.ds"), c.ds);
    o.f(&format!("{p}.r"), c.r);
    o.of(&format!("{p}.edge_lo"), c.edge_lo);
    o.of(&format!("{p}.edge_hi"), c.edge_hi);
    o.oi(&format!("{p}.index_lo"), c.index_lo);
    o.oi(&format!("{p}.index_hi"), c.index_hi);
    o.oi(&format!("{p}.n_jumps"), c.n_jumps);
    o.of(&format!("{p}.ds_star"), c.ds_star);
    o.of(&format!("{p}.slope_s_star"), c.slope_s_star);
    o.b(&format!("{p}.on_grid"), c.on_grid);
    o.len(&format!("{p}.at_edge"), 2);
    o.b(&format!("{p}.at_edge.0"), c.at_edge.0);
    o.b(&format!("{p}.at_edge.1"), c.at_edge.1);
    o.os(&format!("{p}.void"), c.void.as_deref());
}

fn number(o: &mut Flat, p: &str, n: &StaircaseNumber) {
    match n {
        StaircaseNumber::V2 { void, tau_lo, tau_hi } => {
            o.keys(p, 5);
            o.s(&format!("{p}.void"), void);
            o.f(&format!("{p}.tau_lo"), *tau_lo);
            o.f(&format!("{p}.tau_hi"), *tau_hi);
            o.none(&format!("{p}.rise"));
            o.none(&format!("{p}.dg_dtau"));
        }
        StaircaseNumber::V5 { void, tau_lo, tau_hi, edge_moved } => {
            o.keys(p, 6);
            o.s(&format!("{p}.void"), void);
            o.f(&format!("{p}.tau_lo"), *tau_lo);
            o.f(&format!("{p}.tau_hi"), *tau_hi);
            o.b(&format!("{p}.edge_moved"), *edge_moved);
            o.none(&format!("{p}.rise"));
            o.none(&format!("{p}.dg_dtau"));
        }
        StaircaseNumber::Ok(x) => {
            o.keys(p, 15);
            o.f(&format!("{p}.tau_lo"), x.tau_lo);
            o.f(&format!("{p}.tau_hi"), x.tau_hi);
            o.f(&format!("{p}.ds"), x.ds);
            o.f(&format!("{p}.r"), x.r);
            o.none(&format!("{p}.void"));
            o.f(&format!("{p}.rise"), x.rise);
            o.f(&format!("{p}.dg_dtau"), x.dg_dtau);
            o.f(&format!("{p}.dtau"), x.dtau);
            o.of(&format!("{p}.spacing"), x.spacing);
            o.of(&format!("{p}.tread"), x.tread);
            o.of(&format!("{p}.lam"), x.lam);
            o.fs(&format!("{p}.entered"), &x.entered);
            o.fs(&format!("{p}.left"), &x.left);
            o.f(&format!("{p}.d_membership"), x.d_membership);
            o.f(&format!("{p}.d_smooth"), x.d_smooth);
        }
    }
}

fn root(o: &mut Flat, p: &str, r: &RootClass) {
    match r {
        RootClass::Void { void, r, ds } => {
            o.keys(p, 5);
            o.s(&format!("{p}.void"), void);
            o.f(&format!("{p}.r"), *r);
            o.f(&format!("{p}.ds"), *ds);
            o.none(&format!("{p}.kind"));
            o.none(&format!("{p}.root_exists"));
        }
        RootClass::Ok(x) => {
            o.keys(p, 21);
            o.f(&format!("{p}.r"), x.r);
            o.f(&format!("{p}.ds"), x.ds);
            o.none(&format!("{p}.void"));
            o.f(&format!("{p}.lo"), x.lo);
            o.f(&format!("{p}.hi"), x.hi);
            o.f(&format!("{p}.mid"), x.mid);
            o.f(&format!("{p}.width"), x.width);
            o.kind(&format!("{p}.kind"), x.kind);
            o.b(&format!("{p}.root_exists"), x.root_exists);
            o.b(&format!("{p}.set_changed"), x.set_changed);
            o.b(&format!("{p}.argmin_moved"), x.argmin_moved);
            o.b(&format!("{p}.edge_moved"), x.edge_moved);
            o.fs(&format!("{p}.entered"), &x.entered);
            o.fs(&format!("{p}.left"), &x.left);
            o.of(&format!("{p}.d_full"), x.d_full);
            o.of(&format!("{p}.d_smooth"), x.d_smooth);
            o.of(&format!("{p}.d_membership"), x.d_membership);
            o.of(&format!("{p}.h_lo"), x.h_lo);
            o.of(&format!("{p}.h_hi"), x.h_hi);
            o.of(&format!("{p}.h_common_lo"), x.h_common_lo);
            o.of(&format!("{p}.h_common_hi"), x.h_common_hi);
        }
    }
}

// ---------------------------------------------------------------------------- the comparison

/// The oracle's lines for one reading: its root, its subtree, and its `@rig` readback if any.
fn expected(name: &str) -> Vec<(String, String)> {
    let dot = format!("{name}.");
    let at = format!("{name}@");
    let v: Vec<(String, String)> = ORACLE.lines()
        .map(|l| {
            let (p, t) = l.split_once('\t').expect("path<TAB>token");
            (p.to_string(), t.to_string())
        })
        .filter(|(p, _)| p == name || p.starts_with(&dot) || p.starts_with(&at))
        .collect();
    assert!(!v.is_empty(), "the oracle has no reading named {name:?}");
    v
}

/// `rig` is the core the reading marched on — `None` for a derived reading, which has no
/// readback in the oracle either.
fn check(name: &str, rig: Option<&ScheduledStatorCore>, mut got: Flat) {
    if let Some(m) = rig {
        let p = format!("{name}@rig");
        got.keys(&p, 2);
        got.s(&format!("{p}.lag_coord"), m.fuel.inner.lag_coord.get());
        got.of(&format!("{p}.sm_air"), m.fuel.inner.sm_air.get());
    }
    let want = expected(name);
    let got = got.0;
    let bad: Vec<usize> = (0..want.len().min(got.len())).filter(|&k| got[k] != want[k]).collect();
    for &k in bad.iter().take(12) {
        eprintln!("{name}: line {k}: got {:?}, want {:?}", got[k], want[k]);
    }
    assert!(bad.is_empty() && got.len() == want.len(),
            "{name}: {} of {} lines differ (lengths got {}, want {})",
            bad.len(), want.len(), got.len(), want.len());
}

fn one<T>(name: &str, m: Option<&ScheduledStatorCore>, x: &T, w: fn(&mut Flat, &str, &T)) {
    let mut o = Flat::default();
    w(&mut o, name, x);
    check(name, m, o);
}

// ---------------------------------------------------------------------------- the gates

/// Rung 83 § 3.2's jump pair (`r = 0.25`) — `test_p2`'s second half — and its `classify`,
/// forward and REVERSED.
#[test]
fn jump_pair_and_its_classification_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let a = staircase_law::edge_read(&m, JUMP_LO, &kw(&f, 0.25, DS));
    one("edge_jump_lo", Some(&m), &a, edge);
    let b = staircase_law::edge_read(&m, JUMP_HI, &kw(&f, 0.25, DS));
    one("edge_jump_hi", Some(&m), &b, edge);
    one("classify_jump", None, &staircase_law::classify(&a, &b), classified);
    one("classify_rev", None, &staircase_law::classify(&b, &a), classified);
}

/// Rung 83 § 3.4's crossing pair (`r = 0.35`) — `test_p2`'s first half.
#[test]
fn crossing_pair_and_its_classification_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let a = staircase_law::edge_read(&m, CROSS_LO, &kw(&f, 0.35, DS));
    one("edge_cross_lo", Some(&m), &a, edge);
    let b = staircase_law::edge_read(&m, CROSS_HI, &kw(&f, 0.35, DS));
    one("edge_cross_hi", Some(&m), &b, edge);
    one("classify_cross", None, &staircase_law::classify(&a, &b), classified);
}

/// The EMPTY window, and `classify` of it with itself — every Option `None`.
#[test]
fn empty_window_read_and_classification_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let a = staircase_law::edge_read(&m, 0.05, &kw(&f, 1.0, DS));
    one("edge_v1", Some(&m), &a, edge);
    one("classify_v1", None, &staircase_law::classify(&a, &a), classified);
}

/// `test_p4`'s ladder, verbatim.
#[test]
fn p4_ladder_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let s = staircase_law::staircase_scan(&m, 0.0190, 0.0206, 9, &kw(&f, 0.25, DS));
    one("scan_p4", Some(&m), &s, ladder);
}

/// `test_p3`'s count at the shipped step.
#[test]
fn p3_count_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let c = staircase_law::lattice_count(&m, 0.016, 0.024, &kw(&f, 0.25, 0.005));
    one("lattice_p3", Some(&m), &c, lattice);
}

/// `test_p3`'s count at the halved step.
#[test]
fn p3_count_at_the_halved_step_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let c = staircase_law::lattice_count(&m, 0.016, 0.024, &kw(&f, 0.25, 0.0025));
    one("lattice_fine", Some(&m), &c, lattice);
}

/// Python's misnamed `V3: an edge off the march grid`, at `r = 1.0`.
#[test]
fn count_void_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let c = staircase_law::lattice_count(&m, 0.004, 0.05, &kw(&f, 1.0, DS));
    one("lattice_v3", Some(&m), &c, lattice);
}

/// ONE edge move with a spacing — a `lam`.
#[test]
fn staircase_number_with_a_spacing_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let n = staircase_law::staircase_number(&m, JUMP_LO, JUMP_HI, Some(SPACING), &kw(&f, 0.25, DS));
    one("sn_ok", Some(&m), &n, number);
}

/// No spacing — the FACTORS and no `lam`.
#[test]
fn staircase_number_bare_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let n = staircase_law::staircase_number(&m, JUMP_LO, JUMP_HI, None, &kw(&f, 0.25, DS));
    one("sn_bare", Some(&m), &n, number);
}

/// `spacing = 0.0` — Python's truthiness: no tread, but the spacing reported.
#[test]
fn staircase_number_zero_spacing_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let n = staircase_law::staircase_number(&m, JUMP_LO, JUMP_HI, Some(0.0), &kw(&f, 0.25, DS));
    one("sn_zero", Some(&m), &n, number);
}

/// V5 on a degenerate bracket, its message through `%g`.
#[test]
fn staircase_number_v5_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let n = staircase_law::staircase_number(
        &m, 0.0198123456, 0.0198123456, None, &kw(&f, 0.25, DS));
    one("sn_v5", Some(&m), &n, number);
}

/// V2 — an end of the bracket in the empty window.
#[test]
fn staircase_number_v2_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let n = staircase_law::staircase_number(&m, 0.05, 0.30, None, &kw(&f, 1.0, DS));
    one("sn_v2", Some(&m), &n, number);
}

/// `test_p7`'s coarse call — a JUMP, no root.
#[test]
fn root_class_jump_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let r = staircase_law::root_class(&m, BRACKET, N_BISECT, EPS, &kw(&f, 0.25, 0.005));
    one("root_jump", Some(&m), &r, root);
}

/// A CROSSING at the shipped step (`r = 0.35`).
#[test]
fn root_class_crossing_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let r = staircase_law::root_class(&m, BRACKET, N_BISECT, EPS, &kw(&f, 0.35, 0.005));
    one("root_cross", Some(&m), &r, root);
}

/// The bisection's V1, carried.
#[test]
fn root_class_v1_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let r = staircase_law::root_class(&m, BRACKET, N_BISECT, EPS, &kw(&f, 1.0, DS));
    one("root_v1", Some(&m), &r, root);
}

/// V6 with an `eps` whose `%g` (`0.123457`) is not Rust's `{}`.
#[test]
fn root_class_v6_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let r = staircase_law::root_class(&m, BRACKET, 2, 0.123456789, &kw(&f, 0.35, DS));
    one("root_v6", Some(&m), &r, root);
}
