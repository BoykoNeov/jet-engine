//! SLICE AJ step 2 — rung 81's five readers (`_central`, `_criterion_at`, `authority_clock`,
//! `_tau_f_inert`, `authority_mask`), **every returned value bit for bit, in Python's own key
//! order.**
//!
//! # WHAT IS PINNED
//!
//! Five readings on `tests/test_rung81.py`'s rig, each against
//! `oracle/probe_slice_aj_step2.py`'s output (`oracle/slice_aj_step2_pypy.tsv`, PyPy, the repo
//! venv): the two module fixtures `clock` and `mask` verbatim, plus three calls for branches the
//! fixtures never take —
//!
//! * `clock_latched` — a `demand-latched` grid: DEMAND-shaped points through `_criterion_at`'s
//!   ELSE branch (it tests the coordinate's NAME). Its predicted and measured labels disagree on
//!   most points, so the branch choice is loud here.
//! * `clock_single` — one `tau_f` in `clip`: `_tau_f_inert`'s `None` column and an EMPTY
//!   `control_clip_shared` (the control never meets `tau_f == 0.05`).
//! * `mask_clip` — the mask in `clip` at stride 2, where it is VACUOUS.
//!
//! Each reading is flattened to `path <TAB> token` lines — a dict writes `keys:N` then its items in
//! order, a list `len:N` then its items, a float its IEEE bits — and compared one for one. A field
//! missing, extra, renamed, or out of Python's order fails at its path, not only a value.
//! After each reading the caller's rig is read back (`lag_coord`, `_sm_air`): `split_march`'s
//! scope must have restored the knob.
//!
//! # UNREACHED ON THIS RIG — recorded, not implied by a green run (plan § 5.34.2)
//!
//! A riding point at a trajectory END (`n_edge > 0`), an ARRESTED row (`riding4_valid = False`,
//! `n_invalid > 0`), a skipped gains point (`switch` / `regime`), a `tie`/`dormant` label in a
//! cell, a quartic with other than ONE zero, `ever_two_authorities`, a row with no interior cell
//! (`agreement = None`), and `all_fuel`.

use turbojet::authority_clock::{
    authority_clock, authority_mask, AuthStats, AuthorityClock, AuthorityMask, ClipControlRow,
    ClockRow, Criterion, MaskArm, MaskCell, TauFInert, CLOCK_COORDS, CLOCK_TAU_FS,
    CLOCK_TAU_GOVS, CLOCK_TAU_Q, CLOCK_TAU_S, MASK_CLOCKS, MASK_EVERY,
};
use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::Authority;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::split_wall::build_split_wall_cascade;
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE: &str = include_str!("../oracle/slice_aj_step2_pypy.tsv");

// ---------------------------------------------------------------------------- the rig
//
// `tests/test_rung81.py`'s module constants.

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
// the readers' own defaults for the arguments no call passes
const R: f64 = 0.5;
const SETTLE: f64 = 1.2;
const DS: f64 = 0.005;

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

/// Python's `_rig`, with its four knob assignments. The class is `AuthorityClockTransient`
/// there; here it is a rung-80 core, because rung 81 adds no cell (plan § 5.34 (ii)).
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

// ---------------------------------------------------------------------------- the flattening
//
// The probe's `flat`, mirrored: one `(path, token)` per value, in Python's dict order.

#[derive(Default)]
struct Flat(Vec<(String, String)>);

impl Flat {
    fn put(&mut self, p: &str, t: String) {
        self.0.push((p.to_string(), t));
    }
    fn f(&mut self, p: &str, x: f64) {
        self.put(p, format!("f:{:016x}", x.to_bits()));
    }
    fn of(&mut self, p: &str, x: Option<f64>) {
        match x {
            Some(x) => self.f(p, x),
            None => self.put(p, "n".into()),
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
    fn lab(&mut self, p: &str, x: Option<Authority>) {
        match x {
            Some(a) => self.s(p, a.as_str()),
            None => self.put(p, "n".into()),
        }
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
    fn is(&mut self, p: &str, xs: &[usize]) {
        self.len(p, xs.len());
        for (k, &x) in xs.iter().enumerate() {
            self.i(&format!("{p}.{k}"), x);
        }
    }
    fn census(&mut self, p: &str, c: &[(Authority, usize)]) {
        self.keys(p, c.len());
        for (a, n) in c {
            self.i(&format!("{p}.{}", a.as_str()), *n);
        }
    }
}

fn criterion(o: &mut Flat, p: &str, c: &Criterion) {
    o.keys(p, 10);
    o.f(&format!("{p}.s"), c.s);
    o.f(&format!("{p}.setpoint_gap"), c.setpoint_gap);
    o.f(&format!("{p}.lag_gap"), c.lag_gap);
    o.f(&format!("{p}.tau_f"), c.tau_f);
    o.f(&format!("{p}.tau_gov"), c.tau_gov);
    o.f(&format!("{p}.slope_f"), c.slope_f);
    o.f(&format!("{p}.slope_r"), c.slope_r);
    o.s(&format!("{p}.predicted"), c.predicted.as_str());
    o.s(&format!("{p}.measured"), c.measured.as_str());
    o.f(&format!("{p}.margin"), c.margin);
}

fn clock_row(o: &mut Flat, p: &str, x: &ClockRow) {
    o.keys(p, 20);
    o.s(&format!("{p}.coord"), x.coord);
    o.f(&format!("{p}.tau_f"), x.tau_f);
    o.f(&format!("{p}.tau_gov"), x.tau_gov);
    o.f(&format!("{p}.tau_q"), x.tau_q);
    o.f(&format!("{p}.tau_s"), x.tau_s);
    o.f(&format!("{p}.phi_lim"), x.phi_lim);
    o.of(&format!("{p}.phi_air"), x.phi_air);
    o.f(&format!("{p}.max_Tt4"), x.max_tt4);
    o.b(&format!("{p}.riding4_valid"), x.riding4_valid);
    o.i(&format!("{p}.n_riding4"), x.n_riding4);
    o.census(&format!("{p}.census"), &x.census);
    o.i(&format!("{p}.n_fuel"), x.n_fuel);
    o.i(&format!("{p}.n_gov"), x.n_gov);
    o.i(&format!("{p}.n_scored"), x.n_scored);
    o.i(&format!("{p}.n_edge"), x.n_edge);
    o.i(&format!("{p}.n_agree"), x.n_agree);
    o.of(&format!("{p}.agreement"), x.agreement);
    o.of(&format!("{p}.worst_miss"), x.worst_miss);
    o.of(&format!("{p}.min_margin"), x.min_margin);
    o.len(&format!("{p}.cells"), x.cells.len());
    for (k, c) in x.cells.iter().enumerate() {
        criterion(o, &format!("{p}.cells.{k}"), c);
    }
}

fn ctl_row(o: &mut Flat, p: &str, x: &ClipControlRow) {
    o.keys(p, 5);
    o.f(&format!("{p}.tau_f"), x.tau_f);
    o.i(&format!("{p}.n_riding4"), x.n_riding4);
    o.census(&format!("{p}.census"), &x.census);
    o.i(&format!("{p}.n_fuel"), x.n_fuel);
    o.f(&format!("{p}.max_Tt4"), x.max_tt4);
}

fn inert(o: &mut Flat, p: &str, v: &Option<TauFInert>) {
    let Some(v) = v else {
        o.put(p, "n".into());
        return;
    };
    o.keys(p, 8);
    o.i(&format!("{p}.n_tau_f"), v.n_tau_f);
    o.i(&format!("{p}.n_floats"), v.n_floats);
    o.i(&format!("{p}.n_differing"), v.n_differing);
    o.b(&format!("{p}.march_identical"), v.march_identical);
    o.b(&format!("{p}.Tt4_identical"), v.tt4_identical);
    o.b(&format!("{p}.riding4_identical"), v.riding4_identical);
    o.is(&format!("{p}.n_riding4"), &v.n_riding4);
    o.b(&format!("{p}.all_valid"), v.all_valid);
}

fn clock(o: &mut Flat, p: &str, r: &AuthorityClock) {
    o.keys(p, 23);
    o.f(&format!("{p}.phi_lim"), r.phi_lim);
    o.of(&format!("{p}.phi_air"), r.phi_air);
    o.fs(&format!("{p}.tau_fs"), &r.tau_fs);
    o.fs(&format!("{p}.tau_govs"), &r.tau_govs);
    o.f(&format!("{p}.tau_q"), r.tau_q);
    o.f(&format!("{p}.tau_s"), r.tau_s);
    o.len(&format!("{p}.coords"), r.coords.len());
    for (k, c) in r.coords.iter().enumerate() {
        o.s(&format!("{p}.coords.{k}"), c);
    }
    o.f(&format!("{p}.ds"), r.ds);
    o.len(&format!("{p}.rows"), r.rows.len());
    for (k, x) in r.rows.iter().enumerate() {
        clock_row(o, &format!("{p}.rows.{k}"), x);
    }
    o.b(&format!("{p}.control_all_gov"), r.control_all_gov);
    o.is(&format!("{p}.control_n_riding4"), &r.control_n_riding4);
    o.census(&format!("{p}.control_clip_shared"), &r.control_clip_shared);
    o.i(&format!("{p}.control_clip_fuel"), r.control_clip_fuel);
    o.len(&format!("{p}.control_clip_rows"), r.control_clip_rows.len());
    for (k, x) in r.control_clip_rows.iter().enumerate() {
        ctl_row(o, &format!("{p}.control_clip_rows.{k}"), x);
    }
    o.b(&format!("{p}.control_clip_tau_f_live"), r.control_clip_tau_f_live);
    o.b(&format!("{p}.all_fuel"), r.all_fuel);
    o.of(&format!("{p}.agreement"), r.agreement);
    o.i(&format!("{p}.n_scored"), r.n_scored);
    o.of(&format!("{p}.worst_miss"), r.worst_miss);
    o.keys(&format!("{p}.fuel_cells"), r.fuel_cells.len());
    for (c, cells) in &r.fuel_cells {
        let q = format!("{p}.fuel_cells.{c}");
        o.len(&q, cells.len());
        for (k, (tf, tg, n)) in cells.iter().enumerate() {
            o.len(&format!("{q}.{k}"), 3);
            o.f(&format!("{q}.{k}.0"), *tf);
            o.f(&format!("{q}.{k}.1"), *tg);
            o.i(&format!("{q}.{k}.2"), *n);
        }
    }
    o.keys(&format!("{p}.fuel_side"), r.fuel_side.len());
    for (c, side) in &r.fuel_side {
        let q = format!("{p}.fuel_side.{c}");
        o.len(&q, side.len());
        for (k, s) in side.iter().enumerate() {
            o.s(&format!("{q}.{k}"), s);
        }
    }
    o.keys(&format!("{p}.tau_f_inert"), r.tau_f_inert.len());
    for (key, v) in &r.tau_f_inert {
        inert(o, &format!("{p}.tau_f_inert.{key}"), v);
    }
    o.i(&format!("{p}.n_invalid"), r.n_invalid);
}

fn mask_cell(o: &mut Flat, p: &str, c: &MaskCell) {
    o.keys(p, 9);
    o.f(&format!("{p}.s"), c.s);
    o.f(&format!("{p}.phi"), c.phi);
    o.lab(&format!("{p}.authority"), c.authority);
    o.lab(&format!("{p}.masked"), c.masked);
    o.of(&format!("{p}.mask_leak"), c.mask_leak);
    o.f(&format!("{p}.c1"), c.c1);
    o.f(&format!("{p}.c0"), c.c0);
    o.i(&format!("{p}.zeros"), c.zeros);
    o.f(&format!("{p}.cyc"), c.cyc);
}

fn stats(o: &mut Flat, p: &str, d: &AuthStats) {
    o.keys(p, 4);
    o.i(&format!("{p}.n"), d.n);
    o.f(&format!("{p}.max_cyc"), d.max_cyc);
    o.f(&format!("{p}.max_leak"), d.max_leak);
    o.is(&format!("{p}.zeros"), &d.zeros);
}

fn taus4(o: &mut Flat, p: &str, t: (f64, f64, f64, f64)) {
    o.fs(p, &[t.0, t.1, t.2, t.3]);
}

fn mask_arm(o: &mut Flat, p: &str, a: &MaskArm) {
    o.keys(p, 9);
    taus4(o, &format!("{p}.taus"), a.taus);
    o.b(&format!("{p}.riding4_valid"), a.riding4_valid);
    o.f(&format!("{p}.max_Tt4"), a.max_tt4);
    o.i(&format!("{p}.n_riding"), a.n_riding);
    o.i(&format!("{p}.n_sampled"), a.n_sampled);
    o.i(&format!("{p}.n_interior"), a.n_interior);
    o.keys(&format!("{p}.skipped"), 2);
    o.i(&format!("{p}.skipped.switch"), a.skipped.0);
    o.i(&format!("{p}.skipped.regime"), a.skipped.1);
    o.keys(&format!("{p}.by_authority"), a.by_authority.len());
    for (k, d) in &a.by_authority {
        stats(o, &format!("{p}.by_authority.{}", k.map_or("None", |a| a.as_str())), d);
    }
    o.len(&format!("{p}.cells"), a.cells.len());
    for (k, c) in a.cells.iter().enumerate() {
        mask_cell(o, &format!("{p}.cells.{k}"), c);
    }
}

fn mask(o: &mut Flat, p: &str, r: &AuthorityMask) {
    o.keys(p, 16);
    o.s(&format!("{p}.coord"), r.coord);
    o.len(&format!("{p}.clocks"), r.clocks.len());
    for (k, t) in r.clocks.iter().enumerate() {
        taus4(o, &format!("{p}.clocks.{k}"), *t);
    }
    o.f(&format!("{p}.phi_lim"), r.phi_lim);
    o.of(&format!("{p}.phi_air"), r.phi_air);
    o.f(&format!("{p}.ds"), r.ds);
    o.len(&format!("{p}.arms"), r.arms.len());
    for (k, a) in r.arms.iter().enumerate() {
        mask_arm(o, &format!("{p}.arms.{k}"), a);
    }
    o.b(&format!("{p}.vacuous"), r.vacuous);
    o.i(&format!("{p}.n_fuel_interior"), r.n_fuel_interior);
    o.i(&format!("{p}.n_gov_interior"), r.n_gov_interior);
    o.b(&format!("{p}.all_differenced"), r.all_differenced);
    o.b(&format!("{p}.ever_two_authorities"), r.ever_two_authorities);
    o.of(&format!("{p}.max_mask_leak"), r.max_mask_leak);
    o.of(&format!("{p}.cyc_fuel_auth"), r.cyc_fuel_auth);
    o.of(&format!("{p}.cyc_gov_auth"), r.cyc_gov_auth);
    o.is(&format!("{p}.zeros_fuel_auth"), &r.zeros_fuel_auth);
    o.is(&format!("{p}.zeros_gov_auth"), &r.zeros_gov_auth);
}

/// The rig after the reader — the probe's `name@rig` dict.
fn rig_after(o: &mut Flat, name: &str, m: &ScheduledStatorCore) {
    let p = format!("{name}@rig");
    o.keys(&p, 2);
    o.s(&format!("{p}.lag_coord"), m.fuel.inner.lag_coord.get());
    o.of(&format!("{p}.sm_air"), m.fuel.inner.sm_air.get());
}

// ---------------------------------------------------------------------------- the comparison

/// The oracle's lines for one reading: its root, its subtree, and its `@rig` readback.
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

fn check(name: &str, got: Flat) {
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

// ---------------------------------------------------------------------------- the gates

/// `test_rung81.py`'s `clock` fixture: the whole default grid, both coordinates, and the
/// shared-wall control.
#[test]
fn clock_fixture_bit_for_bit() {
    let m = rig();
    let r = authority_clock(&m, &flight(), LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &CLOCK_TAU_FS,
                            &CLOCK_TAU_GOVS, CLOCK_TAU_Q, CLOCK_TAU_S, &CLOCK_COORDS, R, SETTLE,
                            DS, V_MAX, false);
    let mut o = Flat::default();
    clock(&mut o, "clock", &r);
    rig_after(&mut o, "clock", &m);
    check("clock", o);
}

/// `test_rung81.py`'s `mask` fixture: the mirror cell and rung 80's matched clocks, stride 1.
#[test]
fn mask_fixture_bit_for_bit() {
    let m = rig();
    let r = authority_mask(&m, &flight(), LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &MASK_CLOCKS,
                           "demand", R, SETTLE, DS, V_MAX, false, MASK_EVERY)
        .expect("the mask's gains do not abort on this rig");
    let mut o = Flat::default();
    mask(&mut o, "mask", &r);
    rig_after(&mut o, "mask", &m);
    check("mask", o);
}

/// A `demand-latched` grid: DEMAND-shaped points through `_criterion_at`'s CLIP-form branch.
#[test]
fn latched_grid_takes_the_else_branch_bit_for_bit() {
    let m = rig();
    let r = authority_clock(&m, &flight(), LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &[0.05, 0.20],
                            &[0.05], CLOCK_TAU_Q, CLOCK_TAU_S, &["demand-latched"], R, SETTLE,
                            DS, V_MAX, false);
    let mut o = Flat::default();
    clock(&mut o, "clock_latched", &r);
    rig_after(&mut o, "clock_latched", &m);
    check("clock_latched", o);
}

/// One `tau_f`, `clip` only: the `None` inertness column and the EMPTY control census.
#[test]
fn single_column_is_none_bit_for_bit() {
    let m = rig();
    let r = authority_clock(&m, &flight(), LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &[0.20], &[0.05],
                            CLOCK_TAU_Q, CLOCK_TAU_S, &["clip"], R, SETTLE, DS, V_MAX, false);
    let mut o = Flat::default();
    clock(&mut o, "clock_single", &r);
    rig_after(&mut o, "clock_single", &m);
    check("clock_single", o);
}

/// The mask in `clip`, stride 2 — every cell governor-held, so VACUOUS.
#[test]
fn clip_mask_at_stride_two_bit_for_bit() {
    let m = rig();
    let r = authority_mask(&m, &flight(), LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &MASK_CLOCKS,
                           "clip", R, SETTLE, DS, V_MAX, false, 2)
        .expect("the mask's gains do not abort on this rig");
    let mut o = Flat::default();
    mask(&mut o, "mask_clip", &r);
    rig_after(&mut o, "mask_clip", &m);
    check("mask_clip", o);
}
