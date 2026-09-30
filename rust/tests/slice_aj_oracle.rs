//! SLICE AJ step 6 — **THE ORACLE for rungs 81–84**, against PyPy *and* CPython.
//!
//! `slice_aj_clock.rs`, `slice_aj_threshold.rs`, `slice_aj_corrector.rs` and
//! `slice_aj_staircase.rs` pin steps 2–4's readings against PyPy-only probes. This file is the
//! slice's VALUE seat across both interpreters and both stator arms: every public reader of the
//! four rungs, the plant they reduce, compared line for line against
//! `oracle/dump_slice_aj.py`'s goldens (`oracle/slice_aj_pypy.tsv`, `oracle/slice_aj_cpython.tsv`).
//!
//! # THE FORMAT IS STEPS 2–4's, AND THE CONVERTERS ARE THEIRS
//!
//! One `(path, token)` per value in PYTHON's order. Python writes every key it returns, so a field
//! the port forgot, renamed or reordered fails at its path — [`the_paths_are_equal_in_order`] runs
//! before any token is compared. The converters are `tests/slice_aj_flat/mod.rs`, lifted out of
//! steps 2–4's four binaries at this step: those binaries VERIFY them against their own goldens,
//! so no struct is described twice. Only the march point is new here ([`point`], [`plant`]).
//!
//! # WHAT THIS FILE SCORES THAT NOTHING ELSE CAN
//!
//! * **CPython.** See [`CPYTHON_DIFFS`]: the prediction, the REVERSE run that explains it, and the
//!   frozen list.
//! * **The incidence stator arm** (`i_*`). Every rung-81–84 call anywhere else passes
//!   `inc = false` (plan § 5.34 (iii) C), so a port that DROPPED the flag passed every gate. Here
//!   it is driven where its window is LIVE (`r = 0.25`; measured on PyPy before the dumper was
//!   written — non-monotone `n_fuel` 4, 10, 4, 3 at `tau_f` 0.004 / 0.02 / 0.05 / 0.30), and
//!   [`the_incidence_arm_is_live`] keeps it from going vacuous unnoticed.
//! * **The plant** (`p_plant`, `i_plant`): `scan_cells` — the march, `riding4_idx` against
//!   Python's `id(p)` indices, every scored cell — walked every fifth point plus column folds.
//! * **A ladder with `n - 1 = 3`** (`p_shape_n4`) — aimed at step 4's coverage gap C6 and
//!   MISSING it: both ladder spellings are bit-identical at its points, so C6 stays open (§ 5.34.6 (e)).
//!
//! # THE BUDGET (§ 5.34 (ix) P2)
//!
//! One ramp and one `ds` pair per reader; rung 81 at its fixtures' defaults. Readings, their
//! arguments and why each is here: the dumper's `READINGS` table, mirrored by [`drive_flat`].
//!
//! # WHAT IS COMPARED, AND WHAT IS ONLY COUNTED
//!
//! Every line is COMPUTED by the port and compared; nothing is read out of a golden as an input.

use std::sync::OnceLock;

use turbojet::authority_clock::{
    authority_clock, authority_mask, CLOCK_COORDS, CLOCK_TAU_FS, CLOCK_TAU_GOVS, CLOCK_TAU_Q,
    CLOCK_TAU_S, MASK_CLOCKS, MASK_EVERY,
};
use turbojet::bleed_transient::LeverArm;
use turbojet::corrector_law::{self, CAP, FLAT};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{FuelPoint, PointExtra};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{BleedLimiter, Regime};
use turbojet::map::ComponentMap;
use turbojet::reference_split::StatorIncidenceLimiter;
use turbojet::split_wall::build_split_wall_cascade;
use turbojet::staircase_law::{self, EPS};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::threshold_law::{
    self, ScanKw, BRACKET, DS_FINE, N_BISECT, REF_TAU_REFS, TERMS_TAU_GOVS,
};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

mod slice_aj_flat;
use slice_aj_flat::*;

const ORACLE_PYPY: &str = include_str!("../oracle/slice_aj_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/slice_aj_cpython.tsv");

/// The one path family every comparison excludes by NAME — the interpreter sentinel.
const INTERP: &str = "_interp.";

// ============================================================================ the grid
//
// The four suites' module constants, and the dumper's copy of them.

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
const SETTLE: f64 = 1.2;
const DS: f64 = 0.005;
/// `threshold_law`'s default `tau_ref`.
const TAU_REF: f64 = 0.05;
/// `threshold_reference`'s and `threshold_terms`' default ramp.
const R_REF: f64 = 0.35;
/// Rung 81's own ramp.
const R81: f64 = 0.5;
/// Rung 83 § 3.2's jump pair (`r = 0.25`) — steps 4/5's literals.
const JUMP_LO: f64 = 0.0197750;
const JUMP_HI: f64 = 0.0197875;
/// `test_p5`'s spacing formula at `ds = 0.005`.
const SPACING: f64 = 0.005 * (0.024 - 0.016) / 0.005938;
/// The incidence arm's ramp — the one where its window is live.
const R_INC: f64 = 0.25;
/// The plant walk's `tau_f`, inside both arms' live windows; its other clocks are 0.05.
const PLANT_TAU_F: f64 = 0.02;
const P_STRIDE: usize = 5;

/// Python's `(BRACKET[0]*BRACKET[1]) ** 0.5` — step 4's spelling, pinned there to PyPy's bits.
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

/// The dumper's `rig(cls, inc)`: the four suites' `_rig` with the stator arm made explicit. One
/// rung-80 core serves every rung — rungs 81–84 add no cell (plan § 5.34 (ii)).
fn rig(inc: bool) -> ScheduledStatorCore {
    let arm = LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, SM, Some(TAU))),
        stator_lim: if inc { None }
                    else { Some(StatorLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU_S))) },
        stator_inc: if inc {
            Some(StatorIncidenceLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU_S)))
        } else { None },
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

/// The dumper's `kw(r, inc, ds, phi_lim)` — rungs 83/84's `**kw`.
fn kw(f: &FlightCondition, r: f64, inc: bool, ds: f64) -> ScanKw<'_> {
    ScanKw {
        flight: f, tt4_lo: LO, tt4_hi: HI, tt4_max: TT4_MAX, phi_lim: PHI_FUEL, phi_air: PHI_AIR,
        tau_gov: 0.05, tau_q: 0.05, tau_s: 0.05, r, s_settle: SETTLE, ds, v_max: V_MAX, inc,
    }
}

// ============================================================================ the march point
//
// The one shape this step adds to the shared flatteners.

/// Python stores the valve regime as a string; the port has an enum (AG's spelling, kept).
fn regime_str(r: Regime) -> &'static str {
    match r {
        Regime::Dormant => "dormant",
        Regime::Riding => "riding",
        Regime::Saturated => "saturated",
    }
}

/// A DEMAND march point, WHOLE, in the dict's own insertion order (35 keys) — read off PyPy
/// (`W:/temp/claude/slice-aj-step6/probe_keys.py`), not AI's sorted walk, which could not see it.
fn point_fields(p: &FuelPoint) -> Vec<(&'static str, Tok)> {
    let PointExtra::Demand {
        g, required, b, b_cmd, v, v_cmd, v_regime, ic_iters, ic_res, ic_order, g_fuel, g_gov,
        required_fuel, required_gov, authority, share_law, w_fuel, w_gov, cap_fuel, cap_gov,
        lag_coord,
    } = &p.extra else {
        panic!("`scan_cells` marches the DEMAND coordinate only: {:?}", p.extra)
    };
    vec![
        ("s", Tok::F(p.s)), ("nu_lp", Tok::F(p.nu_lp)), ("nu_hp", Tok::F(p.nu_hp)),
        ("Tt4", Tok::F(p.tt4)), ("f", Tok::F(p.f)), ("pi_lpc", Tok::F(p.pi_lpc)),
        ("pi_hpc", Tok::F(p.pi_hpc)), ("phi_lp", Tok::F(p.phi_lp)), ("phi_hp", Tok::F(p.phi_hp)),
        ("mdot_air", Tok::F(p.mdot_air)), ("sp_thrust", Tok::F(p.sp_thrust)),
        ("branch", Tok::S(p.branch.label().to_string())), ("mf", Tok::F(p.mf)),
        ("mf_sched", Tok::F(p.mf_sched)), ("g", Tok::F(*g)), ("required", Tok::F(*required)),
        ("g_fuel", Tok::F(*g_fuel)), ("g_gov", Tok::F(*g_gov)),
        ("required_fuel", Tok::F(*required_fuel)), ("required_gov", Tok::F(*required_gov)),
        ("w_fuel", Tok::F(*w_fuel)), ("w_gov", Tok::F(*w_gov)), ("cap_fuel", Tok::F(*cap_fuel)),
        ("cap_gov", Tok::F(*cap_gov)), ("authority", Tok::S(authority.as_str().to_string())),
        ("b", Tok::F(*b)), ("b_cmd", Tok::F(*b_cmd)), ("v", Tok::F(*v)),
        ("v_cmd", Tok::F(*v_cmd)),
        ("v_regime", v_regime.map_or(Tok::N, |r| Tok::S(regime_str(r).to_string()))),
        ("ic_iters", Tok::I(*ic_iters)), ("ic_res", Tok::F(*ic_res)),
        ("ic_order", Tok::S(ic_order.to_string())), ("share_law", Tok::S(share_law.to_string())),
        ("lag_coord", Tok::S(lag_coord.to_string())),
    ]
}

/// One scalar of a point, typed as Python's `isinstance` sees it.
enum Tok {
    F(f64),
    I(usize),
    S(String),
    N,
}

fn point(o: &mut Flat, p: &str, x: &FuelPoint) {
    let kv = point_fields(x);
    o.keys(p, kv.len());
    for (k, t) in kv {
        let q = format!("{p}.{k}");
        match t {
            Tok::F(v) => o.f(&q, v),
            Tok::I(v) => o.i(&q, v),
            Tok::S(v) => o.s(&q, &v),
            Tok::N => o.none(&q),
        }
    }
}

/// Python's `min(col)` / `max(col)`: the FIRST extremum, replaced only when STRICTLY beaten.
fn py_min(xs: &[f64]) -> f64 {
    xs[1..].iter().fold(xs[0], |a, &x| if x < a { x } else { a })
}
fn py_max(xs: &[f64]) -> f64 {
    xs[1..].iter().fold(xs[0], |a, &x| if x > a { x } else { a })
}

/// The dumper's `plant()` — `scan_cells` reduced for a walk: `n`, every fifth point as
/// `(i, point)`, per float column `min`/`max`/`last`/`neg`, the riding points as INDICES, and
/// every scored cell. The column set is DERIVED from the point, as Python derives it.
fn plant(o: &mut Flat, p: &str, m: &ScheduledStatorCore, kw: &ScanKw) {
    let (traj, ride, cells) = threshold_law::scan_cells(m, PLANT_TAU_F, kw);
    assert!(!traj.is_empty(), "an empty march is not a measurement: {p}");
    o.keys(p, 5);
    o.i(&format!("{p}.n"), traj.len());
    let idx: Vec<usize> = (0..traj.len()).step_by(P_STRIDE).collect();
    o.len(&format!("{p}.points"), idx.len());
    for (k, &i) in idx.iter().enumerate() {
        let q = format!("{p}.points.{k}");
        o.len(&q, 2);
        o.i(&format!("{q}.0"), i);
        point(o, &format!("{q}.1"), &traj[i]);
    }
    let cols: Vec<&'static str> = point_fields(&traj[0]).into_iter()
        .filter_map(|(k, t)| matches!(t, Tok::F(_)).then_some(k))
        .collect();
    o.keys(&format!("{p}.cols"), cols.len());
    for (ci, name) in cols.iter().enumerate() {
        let col: Vec<f64> = traj.iter().map(|x| {
            let fl: Vec<(&str, f64)> = point_fields(x).into_iter()
                .filter_map(|(k, t)| if let Tok::F(v) = t { Some((k, v)) } else { None })
                .collect();
            assert_eq!(fl[ci].0, *name, "a column moved mid-march");
            fl[ci].1
        }).collect();
        let q = format!("{p}.cols.{name}");
        o.keys(&q, 4);
        o.f(&format!("{q}.min"), py_min(&col));
        o.f(&format!("{q}.max"), py_max(&col));
        o.f(&format!("{q}.last"), *col.last().expect("non-empty"));
        o.i(&format!("{q}.neg"), col.iter().filter(|&&x| x < 0.0).count());
    }
    o.is(&format!("{p}.ride"), &ride);
    o.len(&format!("{p}.cells"), cells.len());
    for (k, c) in cells.iter().enumerate() {
        criterion(o, &format!("{p}.cells.{k}"), c);
    }
}

// ============================================================================ the drive
//
// The dumper's `READINGS`, in its order: each reading, then its rig readback; then the two
// derived readings (`classify`, `corrector_step`) on values already in hand.

/// What the liveness gate reads off the drive, besides the lines.
struct Drive {
    lines: Vec<(String, String)>,
    i_scan_n_fuel: usize,
    i_plant_ride: usize,
    i_clock_riding: usize,
}

fn drive_flat() -> Drive {
    let f = flight();
    let mut o = Flat::default();
    let (p, i) = (false, true);
    macro_rules! reading {
        ($name:expr, $inc:expr, |$m:ident| $conv:ident($x:expr)) => {{
            let $m = rig($inc);
            let r = $x;
            $conv(&mut o, $name, &r);
            rig_after(&mut o, $name, &$m);
            r
        }};
    }

    // ------------------------------------------------------------ the `phi` stator arm
    reading!("p_clock", p, |m| clock(authority_clock(
        &m, &f, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &CLOCK_TAU_FS, &CLOCK_TAU_GOVS, CLOCK_TAU_Q,
        CLOCK_TAU_S, &CLOCK_COORDS, R81, SETTLE, DS, V_MAX, p)));
    reading!("p_mask", p, |m| mask(authority_mask(
        &m, &f, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &MASK_CLOCKS, "demand", R81, SETTLE, DS,
        V_MAX, p, MASK_EVERY).expect("the mask's gains do not abort on this rig")));
    reading!("p_law", p, |m| law(threshold_law::threshold_law(
        &m, &f, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &[0.25], 0.05, TAU_REF, 0.05, 0.05, BRACKET,
        N_BISECT, SETTLE, DS, Some(DS_FINE), V_MAX, p)));
    reading!("p_ref", p, |m| reference(threshold_law::threshold_reference(
        &m, &f, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, R_REF, &REF_TAU_REFS, 0.05, 0.05, 0.05,
        BRACKET, N_BISECT, SETTLE, DS, V_MAX, p)));
    reading!("p_terms", p, |m| terms(threshold_law::threshold_terms(
        &m, &f, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &[0.745, 0.750, 0.755], R_REF,
        &TERMS_TAU_GOVS, 0.05, 0.05, BRACKET, N_BISECT, SETTLE, DS, V_MAX, p)));
    reading!("p_scan", p, |m| scan(threshold_law::threshold_scan(&m, 0.05, &kw(&f, R81, p, DS))));
    let p_read = reading!("p_read", p, |m| read(
        corrector_law::corrector_read(&m, 0.08, &kw(&f, R81, p, DS))));
    reading!("p_shape_n4", p, |m| shape(
        corrector_law::residual_shape(&m, 0.016, 0.024, 4, &kw(&f, 0.25, p, DS))));
    let t = t_start();
    reading!("p_secant", p, |m| secant(corrector_law::corrector_secant(
        &m, t, 1.25 * t, CAP, BRACKET, FLAT, &kw(&f, R_REF, p, DS))));
    reading!("p_secant_iter_v4", p, |m| secant(corrector_law::corrector_secant(
        &m, 0.30, 0.29, 3, BRACKET, FLAT, &kw(&f, 1.0, p, DS))));
    let p_lo = reading!("p_edge_lo", p, |m| edge(
        staircase_law::edge_read(&m, JUMP_LO, &kw(&f, 0.25, p, DS))));
    let p_hi = reading!("p_edge_hi", p, |m| edge(
        staircase_law::edge_read(&m, JUMP_HI, &kw(&f, 0.25, p, DS))));
    reading!("p_scan_p4", p, |m| ladder(
        staircase_law::staircase_scan(&m, 0.0190, 0.0206, 9, &kw(&f, 0.25, p, DS))));
    reading!("p_lattice", p, |m| lattice(
        staircase_law::lattice_count(&m, 0.016, 0.024, &kw(&f, 0.25, p, DS))));
    reading!("p_lattice_fine", p, |m| lattice(
        staircase_law::lattice_count(&m, 0.016, 0.024, &kw(&f, 0.25, p, DS_FINE))));
    reading!("p_number", p, |m| number(staircase_law::staircase_number(
        &m, JUMP_LO, JUMP_HI, Some(SPACING), &kw(&f, 0.25, p, DS))));
    reading!("p_root", p, |m| root(
        staircase_law::root_class(&m, BRACKET, staircase_law::N_BISECT, EPS,
                                  &kw(&f, R_REF, p, DS))));
    {
        let m = rig(p);
        plant(&mut o, "p_plant", &m, &kw(&f, R_INC, p, DS));
        rig_after(&mut o, "p_plant", &m);
    }

    // ------------------------------------------------------- the INCIDENCE stator arm
    let i_clock = reading!("i_clock", i, |m| clock(authority_clock(
        &m, &f, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &[0.004, 0.02, 0.05], &[0.02, 0.05],
        CLOCK_TAU_Q, CLOCK_TAU_S, &CLOCK_COORDS, R_INC, SETTLE, DS, V_MAX, i)));
    reading!("i_mask", i, |m| mask(authority_mask(
        &m, &f, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &MASK_CLOCKS, "demand", R_INC, SETTLE, DS,
        V_MAX, i, MASK_EVERY).expect("the mask's gains do not abort on this rig")));
    reading!("i_law", i, |m| law(threshold_law::threshold_law(
        &m, &f, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &[R_INC], 0.05, TAU_REF, 0.05, 0.05, BRACKET,
        N_BISECT, SETTLE, DS, None, V_MAX, i)));
    let i_scan = reading!("i_scan", i, |m| scan(
        threshold_law::threshold_scan(&m, 0.02, &kw(&f, R_INC, i, DS))));
    reading!("i_read", i, |m| read(corrector_law::corrector_read(&m, 0.02, &kw(&f, R_INC, i, DS))));
    reading!("i_shape", i, |m| shape(
        corrector_law::residual_shape(&m, 0.004, 0.05, 4, &kw(&f, R_INC, i, DS))));
    reading!("i_secant", i, |m| secant(corrector_law::corrector_secant(
        &m, 0.01, 0.0125, 4, (0.004, 0.05), FLAT, &kw(&f, R_INC, i, DS))));
    let i_lo = reading!("i_edge_lo", i, |m| edge(
        staircase_law::edge_read(&m, 0.004, &kw(&f, R_INC, i, DS))));
    let i_hi = reading!("i_edge_hi", i, |m| edge(
        staircase_law::edge_read(&m, 0.02, &kw(&f, R_INC, i, DS))));
    reading!("i_ladder", i, |m| ladder(
        staircase_law::staircase_scan(&m, 0.004, 0.05, 5, &kw(&f, R_INC, i, DS))));
    reading!("i_lattice", i, |m| lattice(
        staircase_law::lattice_count(&m, 0.004, 0.05, &kw(&f, R_INC, i, DS))));
    let i_ride = {
        let m = rig(i);
        let k = kw(&f, R_INC, i, DS);
        plant(&mut o, "i_plant", &m, &k);
        rig_after(&mut o, "i_plant", &m);
        threshold_law::scan_cells(&m, PLANT_TAU_F, &k).1.len()
    };

    // ------------------------------------------------------------ the derived readings
    classified(&mut o, "p_classify", &staircase_law::classify(&p_lo, &p_hi));
    classified(&mut o, "i_classify", &staircase_law::classify(&i_lo, &i_hi));
    step(&mut o, "p_step", &corrector_law::corrector_step(&p_read, 0.044));

    Drive {
        lines: o.0,
        i_scan_n_fuel: i_scan.n_fuel,
        i_plant_ride: i_ride,
        i_clock_riding: i_clock.rows.iter().map(|r| r.n_riding4).sum(),
    }
}

fn drive() -> &'static Drive {
    static D: OnceLock<Drive> = OnceLock::new();
    D.get_or_init(drive_flat)
}

// ============================================================================ the goldens

/// A golden's lines, in order, the `_interp.*` sentinel split off.
fn golden(text: &str) -> (Vec<(String, String)>, Vec<(String, String)>) {
    let (mut lines, mut interp) = (Vec::new(), Vec::new());
    for l in text.lines() {
        if l.is_empty() { continue; }
        let (p, t) = l.split_once('\t').expect("path<TAB>token");
        let pair = (p.to_string(), t.to_string());
        if p.starts_with(INTERP) { interp.push(pair) } else { lines.push(pair) }
    }
    (lines, interp)
}

fn pypy() -> &'static Vec<(String, String)> {
    static G: OnceLock<Vec<(String, String)>> = OnceLock::new();
    G.get_or_init(|| golden(ORACLE_PYPY).0)
}

fn cpython() -> &'static Vec<(String, String)> {
    static G: OnceLock<Vec<(String, String)>> = OnceLock::new();
    G.get_or_init(|| golden(ORACLE_CPYTHON).0)
}

/// The reading a path belongs to — its name up to the first `.` or `@`.
fn reading_of(path: &str) -> &str {
    path.split(['.', '@']).next().unwrap_or(path)
}

/// Every reading whose lines (paths OR tokens) differ between the two lists, in drive order —
/// what a failing gate reports, so an injection's reach is read by NAME, not inferred.
fn moved_readings(a: &[(String, String)], b: &[(String, String)]) -> Vec<String> {
    let group = |xs: &[(String, String)]| {
        let mut g: Vec<(String, Vec<(String, String)>)> = Vec::new();
        for x in xs {
            let r = reading_of(&x.0).to_string();
            match g.last_mut() {
                Some((n, v)) if *n == r => v.push(x.clone()),
                _ => g.push((r, vec![x.clone()])),
            }
        }
        g
    };
    let (ga, gb) = (group(a), group(b));
    let mut names: Vec<String> = ga.iter().map(|x| x.0.clone()).collect();
    for (n, _) in &gb {
        if !names.contains(n) { names.push(n.clone()); }
    }
    names.into_iter()
        .filter(|n| ga.iter().find(|x| &x.0 == n).map(|x| &x.1)
                    != gb.iter().find(|x| &x.0 == n).map(|x| &x.1))
        .collect()
}

/// Indices where two EQUAL-PATH line lists differ in token.
fn token_diffs(a: &[(String, String)], b: &[(String, String)]) -> Vec<usize> {
    assert_eq!(a.len(), b.len(), "compare tokens only after the paths are gated");
    (0..a.len()).filter(|&k| a[k].1 != b[k].1).collect()
}

// ============================================================================ the gates

/// **THE PATHS ARE COMPARED BEFORE ANY TOKEN IS** — a field missing, extra, renamed or out of
/// Python's order fails here, by name, at the first line where the two lists part.
#[test]
fn the_paths_are_equal_in_order() {
    let (mine, theirs) = (&drive().lines, pypy());
    assert!(!mine.is_empty(), "an empty drive agrees with an empty golden");
    let moved = moved_readings(mine, theirs);
    if !moved.is_empty() {
        eprintln!("readings that differ from PyPy: {moved:?}");
    }
    if let Some(k) = (0..mine.len().min(theirs.len())).find(|&k| mine[k].0 != theirs[k].0) {
        panic!("paths part at line {k}: rust {:?}, python {:?}", mine[k].0, theirs[k].0);
    }
    assert_eq!(mine.len(), theirs.len(), "one list is a prefix of the other");
    // the CPython golden walks the same paths — else its token comparison below means nothing
    let cp = cpython();
    assert_eq!(cp.len(), theirs.len(), "the CPython golden has a different line count");
    assert!(cp.iter().zip(theirs).all(|(a, b)| a.0 == b.0), "the two goldens' paths differ");
}

/// **Every token bit-identical to PyPy** (plan predictions P-A).
#[test]
fn every_token_equals_the_pypy_golden() {
    let (mine, theirs) = (&drive().lines, pypy());
    assert!(moved_readings(mine, theirs).is_empty(),
            "readings differ from PyPy: {:?}", moved_readings(mine, theirs));
    let bad = token_diffs(mine, theirs);
    for &k in bad.iter().take(20) {
        eprintln!("line {k} {}: rust {} vs pypy {}", mine[k].0, mine[k].1, theirs[k].1);
    }
    assert!(bad.is_empty(), "{} of {} lines differ from PyPy", bad.len(), theirs.len());
    println!("PyPy arm: {} lines compared, 0 read as input", theirs.len());
}

/// **THE TWO GOLDENS ARE TWO RUNS, SAID BY THE ONE LINE BUILT TO SAY IT** — AH's sentinel.
#[test]
fn the_two_goldens_carry_their_own_interpreters_sum() {
    let tok = |text: &str| golden(text).1;
    let want = |x: f64| vec![("_interp.sum_probe".to_string(), format!("f:{:016x}", x.to_bits()))];
    assert_eq!(tok(ORACLE_PYPY), want(0.0), "the PyPy golden must carry PyPy's naive `sum`");
    assert_eq!(tok(ORACLE_CPYTHON), want(1.0),
               "the CPython golden must carry CPython's compensated `sum`");
}

/// **THE INCIDENCE ARM IS LIVE** — its whole value is catching a port that drops `inc`, which it
/// cannot do on an empty window. Read off the Rust drive, which the PyPy gate pins line for line.
#[test]
fn the_incidence_arm_is_live() {
    let d = drive();
    assert!(d.i_scan_n_fuel > 0, "i_scan has no fuel-held riding point");
    assert!(d.i_plant_ride > 0, "i_plant has no riding point");
    assert!(d.i_clock_riding > 0, "i_clock's rows have no riding point");
    println!("incidence arm: i_scan n_fuel {}, i_plant riding {}, i_clock riding {}",
             d.i_scan_n_fuel, d.i_plant_ride, d.i_clock_riding);
}

/// **THE CPYTHON ARM — PREDICTED, EXPLAINED BY A REVERSE RUN, THEN FROZEN.**
///
/// Predictions written before either CPython file was opened
/// (`W:/temp/claude/slice-aj-step6/predictions.md`); outcomes, plan § 5.34.6:
///
/// * **P2's CPython half** (*"bit-exact except `authority_mask`'s `c0`/`c1`"*) was already
///   REFUTED by step 4 — by `eta_c_at`'s `(n - 1.0) ** 2`, which CPython's `pow` misrounds where
///   PyPy multiplies (26 of 1 291 982 calls in one march), amplified by the Illinois solve. That
///   rate is why no per-reading census could predict WHICH lines move: every march has flips.
/// * **P-B, THE REVERSE RUN — CONFIRMED.** CPython with every literal `** 2` in `turbojet/`
///   rewritten `x * x` at import (27 sites, 1 116 742 416 calls) and `sum` a naive fold is
///   BYTE-IDENTICAL to the PyPy golden, all 24 324 lines, sentinel included. So the whole
///   CPython difference is those two mechanisms and nothing else.
/// * **P-C1 — CONFIRMED**: step 4's two readings, verbatim here, move at exactly its 27 values
///   (`p_scan_p4` 21 + 3 `summands` at ladder points 4 and 5; `p_secant_iter_v4` step 2's `F`,
///   `g`, and `final_g`).
/// * **P-C2 — HALF**: `p_mask` moves 155 lines, every one a `c0`/`c1` (the pre-flight's r81m
///   count); `i_mask` moves NOTHING — it is vacuous, with no cell to move.
/// * **P-C3 — not predicted, recorded**: `p_plant` 11, `i_clock` 63, `i_edge_hi` 6, `i_plant` 159.
///   The incidence plant drifts most (valve / stator states `v`, `b`). Every bisecting reader
///   (`p_law`, `p_ref`, `p_terms`, `p_root`) is CPython-exact.
/// * **P-C4 — CONFIRMED**: every differing line is a FLOAT; no count, label, void or shape moves.
///
/// Frozen as the 421 paths in `oracle/slice_aj_cpython_diffs.txt` and this tally. Since Rust ≡
/// PyPy line for line, Rust-vs-CPython IS PyPy-vs-CPython.
const CPYTHON_TALLY: &[(&str, usize)] = &[
    ("p_mask", 155), ("p_secant_iter_v4", 3), ("p_scan_p4", 24), ("p_plant", 11),
    ("i_clock", 63), ("i_edge_hi", 6), ("i_plant", 159),
];
const CPYTHON_DIFFS: &str = include_str!("../oracle/slice_aj_cpython_diffs.txt");

#[test]
fn the_cpython_golden_differs_exactly_at_the_frozen_lines() {
    let (mine, theirs) = (&drive().lines, cpython());
    let bad = token_diffs(mine, theirs);
    for &k in &bad {
        assert!(mine[k].1.starts_with("f:") && theirs[k].1.starts_with("f:"),
                "P-C4: a NON-float line differs on CPython: {} ({} vs {})",
                mine[k].0, mine[k].1, theirs[k].1);
    }
    let got: Vec<&str> = bad.iter().map(|&k| mine[k].0.as_str()).collect();
    let want: Vec<&str> = CPYTHON_DIFFS.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(got, want, "the CPython differences are not the frozen list");
    let mut tally: Vec<(&str, usize)> = Vec::new();
    for p in &got {
        let r = reading_of(p);
        match tally.last_mut() {
            Some((n, c)) if *n == r => *c += 1,
            _ => tally.push((r, 1)),
        }
    }
    assert_eq!(tally, CPYTHON_TALLY, "the per-reading CPython tally moved");
    println!("CPython arm: {} lines, {} differ, all floats, all on the frozen list",
             theirs.len(), got.len());
}
