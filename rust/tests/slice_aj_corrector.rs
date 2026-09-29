//! SLICE AJ step 4, rung 83 — `corrector_read`, `corrector_step`, `residual_shape` and
//! `corrector_secant`, **every returned value bit for bit, in Python's own key order.**
//!
//! # WHAT IS PINNED
//!
//! Fourteen readings on `tests/test_rung83.py`'s rig, each against
//! `oracle/probe_slice_aj_step4.py`'s output (`oracle/slice_aj_step4_pypy.tsv`, PyPy, the repo
//! venv; rung 84's readings are in the same file and gated in `slice_aj_staircase.rs`). The
//! suite's own calls where it has one — `read_ctl` and its two steps, the two shape windows, the
//! two secants — and, for branches the suite never takes:
//!
//! * `step_v4` — a step off the EMPTY-window read (`r = 1.0`), and Python's misnamed
//!   `V4: kappa impure`.
//! * `shape_jump` at `n = 3` — a sign change AND a same-sign pair (the `continue`); its two ends are
//!   bit-equal to the suite's `n = 2` window.
//! * `shape_v1` — one ladder end with no `g` (`n_kappa_impure = 1`).
//! * `secant_iter_v4` — `V4: kappa impure at an iterate`, which the pre-flight could not drive: a
//!   secant from `(0.30, 0.29)` on `r = 1.0` steps into an empty window.
//! * `secant_start_v4` — the first start pushed, the second aborting.
//! * `secant_s2` — the `%g` of `flat = 1e-12`, which `{}` would print `0.000000000001`.
//! * `secant_clamp` — a bracket that excludes the root, so the iterates CLAMP (S1).
//!
//! Flattened as step 3's gate: `path <TAB> token`, a dict `keys:N` then its items in order, a list
//! `len:N`, a float its IEEE bits. A reading that ran a march is followed by its rig readback
//! (`lag_coord`, `_sm_air`); a derived one (`step_*`) ran none and has no readback.

use turbojet::bleed_transient::LeverArm;
use turbojet::corrector_law::{
    self, Change, CorrectorRead, CorrectorSecant, CorrectorStep, ResidualShape, SecantStep, CAP,
    FLAT,
};
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::split_wall::build_split_wall_cascade;
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::threshold_law::{ScanKw, ThresholdScan, BRACKET};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE: &str = include_str!("../oracle/slice_aj_step4_pypy.tsv");

// ---------------------------------------------------------------------------- the rig
//
// `tests/test_rung83.py`'s module constants (identical to `test_rung84.py`'s).

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
/// Rung 81's own ramp — the identity control.
const R81: f64 = 0.5;
const DS: f64 = 0.005;
/// § 3.2's jump window, `r = 0.25`.
const JUMP_LO: f64 = 0.0197750;
const JUMP_HI: f64 = 0.0197875;
/// § 3.4's crossing window, `r = 0.35`.
const CROSS_LO: f64 = 0.037000;
const CROSS_HI: f64 = 0.037333;
/// Rung 82's 13-march bisected mid at `r = 0.25`.
const ROOT: f64 = 0.019753906249999998;

/// § 4's start rule, `sqrt(lo·hi)` over rung 82's bracket — Python's `** 0.5`, so `powf`, not
/// `sqrt` (the oracle pins `t0`, so a spelling that rounded differently would fail at its path).
fn t_start() -> f64 {
    (BRACKET.0 * BRACKET.1).powf(0.5)
}

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

/// Python's `_rig`, with its four knob assignments. The class is `CorrectorLawTransient` there;
/// here it is a rung-80 core, because rung 83 adds no cell (plan § 5.34 (ii)).
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

/// `test_rung83.py`'s `_kw(r)`.
fn kw(f: &FlightCondition, r: f64) -> ScanKw<'_> {
    ScanKw {
        flight: f, tt4_lo: LO, tt4_hi: HI, tt4_max: TT4_MAX, phi_lim: PHI_FUEL, phi_air: PHI_AIR,
        tau_gov: 0.05, tau_q: 0.05, tau_s: 0.05, r, s_settle: 1.2, ds: DS, v_max: 0.20,
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
    fn ob(&mut self, p: &str, x: Option<bool>) {
        match x {
            Some(x) => self.b(p, x),
            None => self.none(p),
        }
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

fn scan(o: &mut Flat, p: &str, s: &ThresholdScan) {
    o.keys(p, 28);
    o.f(&format!("{p}.tau_f"), s.tau_f);
    o.f(&format!("{p}.tau_gov"), s.tau_gov);
    o.f(&format!("{p}.r"), s.r);
    o.f(&format!("{p}.ds"), s.ds);
    o.f(&format!("{p}.phi_lim"), s.phi_lim);
    o.of(&format!("{p}.phi_air"), s.phi_air);
    o.i(&format!("{p}.npts"), s.npts);
    o.i(&format!("{p}.n_riding4"), s.n_riding4);
    o.f(&format!("{p}.max_Tt4"), s.max_tt4);
    o.f(&format!("{p}.min_phi"), s.min_phi);
    o.b(&format!("{p}.riding4_valid"), s.riding4_valid);
    o.b(&format!("{p}.window_open"), s.window_open);
    o.i(&format!("{p}.n_scored"), s.n_scored);
    o.i(&format!("{p}.n_slope_excluded"), s.n_slope_excluded);
    o.i(&format!("{p}.n_fuel"), s.n_fuel);
    o.i(&format!("{p}.n_gov"), s.n_gov);
    o.i(&format!("{p}.n_pred_fuel"), s.n_pred_fuel);
    o.i(&format!("{p}.n_agree"), s.n_agree);
    o.of(&format!("{p}.h"), s.h);
    o.of(&format!("{p}.tau_hat_min"), s.tau_hat_min);
    o.of(&format!("{p}.s_bind"), s.s_bind);
    o.of(&format!("{p}.tau_eff_bind"), s.tau_eff_bind);
    o.fs(&format!("{p}.kappa"), &s.kappa);
    o.b(&format!("{p}.kappa_pure"), s.kappa_pure);
    o.of(&format!("{p}.gap_bind"), s.gap_bind);
    o.of(&format!("{p}.ratio_bind"), s.ratio_bind);
    o.of(&format!("{p}.slope_f_bind"), s.slope_f_bind);
    o.of(&format!("{p}.slope_r_bind"), s.slope_r_bind);
}

fn read(o: &mut Flat, p: &str, r: &CorrectorRead) {
    o.keys(p, 16);
    o.f(&format!("{p}.tau_f"), r.tau_f);
    o.of(&format!("{p}.F"), r.f);
    o.of(&format!("{p}.g"), r.g);
    o.of(&format!("{p}.h"), r.h);
    o.of(&format!("{p}.kappa"), r.kappa);
    o.b(&format!("{p}.kappa_pure"), r.kappa_pure);
    o.b(&format!("{p}.exact"), r.exact);
    o.of(&format!("{p}.identity_pred"), r.identity_pred);
    o.ob(&format!("{p}.below_root"), r.below_root);
    o.of(&format!("{p}.s_bind"), r.s_bind);
    o.i(&format!("{p}.n_fuel"), r.n_fuel);
    o.i(&format!("{p}.n_scored"), r.n_scored);
    o.b(&format!("{p}.window_open"), r.window_open);
    o.b(&format!("{p}.riding4_valid"), r.riding4_valid);
    o.of(&format!("{p}.tau_hat_min"), r.tau_hat_min);
    scan(o, &format!("{p}.scan"), &r.scan);
}

fn step(o: &mut Flat, p: &str, s: &CorrectorStep) {
    o.keys(p, 5);
    match s {
        CorrectorStep::Void { c, forward, void } => {
            o.none(&format!("{p}.tau_hat"));
            o.f(&format!("{p}.c"), *c);
            o.of(&format!("{p}.forward"), *forward);
            o.none(&format!("{p}.correction"));
            o.s(&format!("{p}.void"), void);
        }
        CorrectorStep::Ok { tau_hat, c, forward, correction } => {
            o.f(&format!("{p}.tau_hat"), *tau_hat);
            o.f(&format!("{p}.c"), *c);
            o.none(&format!("{p}.void"));
            o.f(&format!("{p}.forward"), *forward);
            o.f(&format!("{p}.correction"), *correction);
        }
    }
}

fn change(o: &mut Flat, p: &str, c: &Change) {
    o.keys(p, 11);
    o.f(&format!("{p}.tau_lo"), c.tau_lo);
    o.f(&format!("{p}.tau_hi"), c.tau_hi);
    o.f(&format!("{p}.g_lo"), c.g_lo);
    o.f(&format!("{p}.g_hi"), c.g_hi);
    o.of(&format!("{p}.s_lo"), c.s_lo);
    o.of(&format!("{p}.s_hi"), c.s_hi);
    o.b(&format!("{p}.argmin_moved"), c.argmin_moved);
    o.f(&format!("{p}.smallest_g"), c.smallest_g);
    o.f(&format!("{p}.step"), c.step);
    o.f(&format!("{p}.ratio"), c.ratio);
    o.of(&format!("{p}.jump_in_F"), c.jump_in_f);
}

fn shape(o: &mut Flat, p: &str, s: &ResidualShape) {
    o.keys(p, 9);
    o.f(&format!("{p}.lo"), s.lo);
    o.f(&format!("{p}.hi"), s.hi);
    o.i(&format!("{p}.n"), s.n);
    o.f(&format!("{p}.step"), s.step);
    o.len(&format!("{p}.points"), s.points.len());
    for (k, x) in s.points.iter().enumerate() {
        read(o, &format!("{p}.points.{k}"), x);
    }
    o.len(&format!("{p}.changes"), s.changes.len());
    for (k, x) in s.changes.iter().enumerate() {
        change(o, &format!("{p}.changes.{k}"), x);
    }
    o.i(&format!("{p}.n_changes"), s.n_changes);
    o.i(&format!("{p}.n_kappa_impure"), s.n_kappa_impure);
    o.i(&format!("{p}.n_argmin_switches"), s.n_argmin_switches);
}

fn trace_step(o: &mut Flat, p: &str, x: &SecantStep) {
    o.keys(p, 5);
    o.f(&format!("{p}.tau"), x.tau);
    o.f(&format!("{p}.F"), x.f);
    o.f(&format!("{p}.g"), x.g);
    o.of(&format!("{p}.s_bind"), x.s_bind);
    o.b(&format!("{p}.clamped"), x.clamped);
}

fn secant(o: &mut Flat, p: &str, s: &CorrectorSecant) {
    o.keys(p, 11);
    o.f(&format!("{p}.t0"), s.t0);
    o.f(&format!("{p}.t1"), s.t1);
    o.i(&format!("{p}.cap"), s.cap);
    o.len(&format!("{p}.trace"), s.trace.len());
    for (k, x) in s.trace.iter().enumerate() {
        trace_step(o, &format!("{p}.trace.{k}"), x);
    }
    o.i(&format!("{p}.clamps"), s.clamps);
    o.os(&format!("{p}.abort"), s.abort.as_deref());
    o.i(&format!("{p}.marches"), s.marches);
    o.of(&format!("{p}.tau"), s.tau);
    o.of(&format!("{p}.final_g"), s.final_g);
    o.len(&format!("{p}.marches_vs_bisect"), 2);
    o.i(&format!("{p}.marches_vs_bisect.0"), s.marches_vs_bisect.0);
    o.i(&format!("{p}.marches_vs_bisect.1"), s.marches_vs_bisect.1);
    o.b(&format!("{p}.converged"), s.converged);
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

/// `test_a_degenerate_slope_voids_the_step_rather_than_clipping_it`'s read and its two steps.
#[test]
fn control_read_and_its_steps_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let rd = corrector_law::corrector_read(&m, 0.08, &kw(&f, R81));
    one("read_ctl", Some(&m), &rd, read);
    one("step_ok", None, &corrector_law::corrector_step(&rd, 0.044), step);
    one("step_v5", None, &corrector_law::corrector_step(&rd, 1.0), step);
}

/// The EMPTY window (`r = 1.0`) and a step off it — Python's misnamed `V4: kappa impure`.
#[test]
fn empty_window_read_and_the_v4_step_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let rd = corrector_law::corrector_read(&m, 0.05, &kw(&f, 1.0));
    one("read_v1", Some(&m), &rd, read);
    one("step_v4", None, &corrector_law::corrector_step(&rd, 0.044), step);
}

/// § 3.2's jump window at `n = 3` — a sign change and a same-sign pair.
#[test]
fn jump_shape_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let s = corrector_law::residual_shape(&m, JUMP_LO, JUMP_HI, 3, &kw(&f, 0.25));
    one("shape_jump", Some(&m), &s, shape);
}

/// § 3.4's crossing window, verbatim.
#[test]
fn crossing_shape_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let s = corrector_law::residual_shape(&m, CROSS_LO, CROSS_HI, 2, &kw(&f, 0.35));
    one("shape_cross", Some(&m), &s, shape);
}

/// A ladder with one end in the empty window: `n_kappa_impure = 1`, no change.
#[test]
fn half_empty_shape_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let s = corrector_law::residual_shape(&m, 0.05, 0.30, 2, &kw(&f, 1.0));
    one("shape_v1", Some(&m), &s, shape);
}

/// § 4's converging secant, from the registered start rule.
#[test]
fn converging_secant_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let t = t_start();
    let s = corrector_law::corrector_secant(&m, t, 1.25 * t, CAP, BRACKET, FLAT, &kw(&f, 0.35));
    one("secant_ok", Some(&m), &s, secant);
}

/// § 4's control — started at rung 82's own answer on the jump, the secant oscillates.
#[test]
fn jump_secant_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let s = corrector_law::corrector_secant(
        &m, ROOT, 1.25 * ROOT, CAP, BRACKET, FLAT, &kw(&f, 0.25));
    one("secant_jump", Some(&m), &s, secant);
}

/// `V4: kappa impure at an iterate` — the void the pre-flight could not drive.
#[test]
fn secant_aborting_at_an_iterate_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let s = corrector_law::corrector_secant(&m, 0.30, 0.29, 3, BRACKET, FLAT, &kw(&f, 1.0));
    one("secant_iter_v4", Some(&m), &s, secant);
}

/// The first start pushed, the second aborting — a one-entry trace.
#[test]
fn secant_aborting_at_the_second_start_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let s = corrector_law::corrector_secant(&m, 0.30, 0.05, CAP, BRACKET, FLAT, &kw(&f, 1.0));
    one("secant_start_v4", Some(&m), &s, secant);
}

/// S2 from identical starts, its message through `%g`.
#[test]
fn flat_pair_secant_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let s = corrector_law::corrector_secant(&m, 0.03, 0.03, CAP, BRACKET, FLAT, &kw(&f, 0.35));
    one("secant_s2", Some(&m), &s, secant);
}

/// A bracket excluding the root — the iterates CLAMP (S1), counted.
#[test]
fn clamping_secant_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let s = corrector_law::corrector_secant(&m, 0.025, 0.03, 2, (0.02, 0.035), FLAT, &kw(&f, 0.35));
    one("secant_clamp", Some(&m), &s, secant);
}
