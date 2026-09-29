//! SLICE AJ step 3 — rung 82's nine readers (`_scan_cells`, `_hats`, `_threshold_scan`, `_scan`,
//! `_bisect`, `threshold_law`, `_threshold_row`, `threshold_reference`, `threshold_terms`),
//! **every returned value bit for bit, in Python's own key order.**
//!
//! # WHAT IS PINNED
//!
//! Twelve readings on `tests/test_rung82.py`'s rig, each against
//! `oracle/probe_slice_aj_step3.py`'s output (`oracle/slice_aj_step3_pypy.tsv`, PyPy, the repo
//! venv): the three module fixtures `law`, `ref` and `terms` verbatim; the suite's two direct
//! `_threshold_scan` calls and its two V3 bisections verbatim; and five calls for branches the
//! suite never takes —
//!
//! * `bisect_v3_g` — V3 with `lo = 0.0123456789`. On the default bracket Python's `%g` and Rust's
//!   `{}` print the SAME string (`[0.004, 0.3]`), so only an argument like this one shows the
//!   message is built through `py_g`.
//! * `bisect_v1` — V1 with BOTH ends empty. At `r = 1.0` the default bracket's `0.30` end is NOT
//!   empty (14 riding points, all fuel), so there V1 fires off the low end alone and a bisection
//!   testing the straddle first would return the same void: K16 of the step's sweep SURVIVED
//!   every other reading.
//! * `law_fine` — the `ds_fine` leg (the `law` fixture passes `None`), and a VOID row (`r = 1.0`).
//! * `ref_void` — `threshold_reference`'s void shape.
//! * `terms_void` — unsorted `phi_lims` with one wall censored: the SORTED iteration beside the
//!   AS-PASSED field, the void `at` shape, `n_void > 0`, `p4 = p5 = None`.
//!
//! Each reading is flattened to `path <TAB> token` lines — a dict writes `keys:N` then its items in
//! order, a list `len:N` then its items, a float its IEEE bits (`f:nan` for any NaN) — and
//! compared one for one. A field missing, extra, renamed, or out of Python's order fails at its
//! path, not only a value. After each reading the caller's rig is read back (`lag_coord`,
//! `_sm_air`): `split_march`'s scope must have restored the knob.
//!
//! # UNREACHED ON THIS RIG — recorded, not implied by a green run (plan § 5.34.3)
//!
//! The `nan` of `tau_star_eff` (it needs a kappa-IMPURE reference march; kappa is `[3.0]` on all
//! 45 marches of a search over `r`, `tau_f`, both walls, both governor clocks and a halved `ds`
//! — rung 82's own E5, every riding point in RELEASE), `n_slope_excluded > 0` (0 on the same
//! 45), `_bisect`'s mid-bisection V1, a row where the measured bisection is ok but the fixed
//! point's voids, `ds_stable = False`, and `tau_f = 0.0`'s empty kappa.

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::split_wall::build_split_wall_cascade;
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::threshold_law::{
    self, key_fuel, AtOk, Bisect, RefFull, RefRow, RowOk, RowVoid, ScanKw, TermsAt,
    ThresholdLaw, ThresholdReference, ThresholdRow, ThresholdScan, ThresholdTerms, BRACKET,
    N_BISECT, P4, P5, REF_TAU_REFS, TERMS_TAU_GOVS,
};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE: &str = include_str!("../oracle/slice_aj_step3_pypy.tsv");

// ---------------------------------------------------------------------------- the rig
//
// `tests/test_rung82.py`'s module constants.

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
/// `test_rung82.py`'s trimmed ramp set.
const RS: [f64; 3] = [0.25, 0.35, 0.50];
/// Rung 81's own ramp.
const R81: f64 = 0.5;
// the readers' own defaults for the arguments no call passes
const SETTLE: f64 = 1.2;
const DS: f64 = 0.005;
const TAU_GOV: f64 = 0.05;
const TAU_REF: f64 = 0.05;
const TAU_Q: f64 = 0.05;
const R_REF: f64 = 0.35;

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

/// Python's `_rig`, with its four knob assignments. The class is `ThresholdLawTransient` there;
/// here it is a rung-80 core, because rung 82 adds no cell (plan § 5.34 (ii)).
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

/// A `ScanKw` at the suite's defaults, `r` and `phi_lim` given.
fn kw(f: &FlightCondition, r: f64, phi_lim: f64) -> ScanKw<'_> {
    ScanKw {
        flight: f, tt4_lo: LO, tt4_hi: HI, tt4_max: TT4_MAX, phi_lim, phi_air: PHI_AIR,
        tau_gov: TAU_GOV, tau_q: TAU_Q, tau_s: TAU_S, r, s_settle: SETTLE, ds: DS, v_max: V_MAX,
        inc: false,
    }
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
        if x.is_nan() {
            self.put(p, "f:nan".into());
        } else {
            self.put(p, format!("f:{:016x}", x.to_bits()));
        }
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
    fn ob(&mut self, p: &str, x: Option<bool>) {
        match x {
            Some(x) => self.b(p, x),
            None => self.put(p, "n".into()),
        }
    }
    fn s(&mut self, p: &str, x: &str) {
        self.put(p, format!("s:{x}"));
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
    fn ofs(&mut self, p: &str, xs: &[Option<f64>]) {
        self.len(p, xs.len());
        for (k, &x) in xs.iter().enumerate() {
            self.of(&format!("{p}.{k}"), x);
        }
    }
    fn pair(&mut self, p: &str, a: f64, b: f64) {
        self.len(p, 2);
        self.f(&format!("{p}.0"), a);
        self.f(&format!("{p}.1"), b);
    }
    fn pairs(&mut self, p: &str, xs: &[(f64, f64)]) {
        self.len(p, xs.len());
        for (k, &(a, b)) in xs.iter().enumerate() {
            self.pair(&format!("{p}.{k}"), a, b);
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

fn bisect(o: &mut Flat, p: &str, b: &Bisect) {
    match b {
        Bisect::VoidEnd { void, lo, hi, at_lo, at_hi } => {
            o.keys(p, 5);
            o.s(&format!("{p}.void"), void);
            o.f(&format!("{p}.lo"), *lo);
            o.f(&format!("{p}.hi"), *hi);
            scan(o, &format!("{p}.at_lo"), at_lo);
            scan(o, &format!("{p}.at_hi"), at_hi);
        }
        Bisect::VoidStraddle { void, lo, hi, at_lo, at_hi, below, above } => {
            o.keys(p, 7);
            o.s(&format!("{p}.void"), void);
            o.f(&format!("{p}.lo"), *lo);
            o.f(&format!("{p}.hi"), *hi);
            scan(o, &format!("{p}.at_lo"), at_lo);
            scan(o, &format!("{p}.at_hi"), at_hi);
            o.b(&format!("{p}.below"), *below);
            o.b(&format!("{p}.above"), *above);
        }
        Bisect::VoidMid { void, lo, hi } => {
            o.keys(p, 3);
            o.s(&format!("{p}.void"), void);
            o.f(&format!("{p}.lo"), *lo);
            o.f(&format!("{p}.hi"), *hi);
        }
        Bisect::Ok(b) => {
            o.keys(p, 6);
            o.f(&format!("{p}.lo"), b.lo);
            o.f(&format!("{p}.hi"), b.hi);
            o.f(&format!("{p}.mid"), b.mid);
            o.f(&format!("{p}.width"), b.width);
            o.none(&format!("{p}.void"));
            scan(o, &format!("{p}.at_hi"), &b.at_hi);
        }
    }
}

fn row_void(o: &mut Flat, p: &str, x: &RowVoid) {
    o.keys(p, 8);
    o.f(&format!("{p}.r"), x.r);
    o.b(&format!("{p}.ok"), false);
    o.s(&format!("{p}.void"), &x.void);
    o.fs(&format!("{p}.kappa"), &x.kappa);
    o.b(&format!("{p}.kappa_pure"), x.kappa_pure);
    o.i(&format!("{p}.n_riding4"), x.n_riding4);
    o.b(&format!("{p}.window_open"), x.window_open);
    o.b(&format!("{p}.riding4_valid"), x.riding4_valid);
}

fn row_ok(o: &mut Flat, p: &str, x: &RowOk) {
    o.keys(p, 32);
    o.f(&format!("{p}.r"), x.r);
    o.b(&format!("{p}.ok"), true);
    o.none(&format!("{p}.void"));
    o.f(&format!("{p}.tau_star"), x.tau_star);
    o.f(&format!("{p}.lo"), x.lo);
    o.f(&format!("{p}.hi"), x.hi);
    o.f(&format!("{p}.width"), x.width);
    o.f(&format!("{p}.tau_star_eff"), x.tau_star_eff);
    o.f(&format!("{p}.fixed"), x.fixed);
    o.of(&format!("{p}.fwd"), x.fwd);
    o.f(&format!("{p}.tau_ref"), x.tau_ref);
    o.of(&format!("{p}.tau_hat_min_ref"), x.tau_hat_min_ref);
    o.b(&format!("{p}.fixed_in_bracket"), x.fixed_in_bracket);
    o.b(&format!("{p}.fwd_in_bracket"), x.fwd_in_bracket);
    o.b(&format!("{p}.fixed_early"), x.fixed_early);
    o.b(&format!("{p}.fwd_early"), x.fwd_early);
    o.f(&format!("{p}.err_fixed"), x.err_fixed);
    o.of(&format!("{p}.err_fwd"), x.err_fwd);
    o.f(&format!("{p}.dist_ref"), x.dist_ref);
    o.of(&format!("{p}.s_bind"), x.s_bind);
    o.of(&format!("{p}.gap_bind"), x.gap_bind);
    o.of(&format!("{p}.ratio_bind"), x.ratio_bind);
    o.of(&format!("{p}.tau_eff_bind"), x.tau_eff_bind);
    o.i(&format!("{p}.n_riding4"), x.n_riding4);
    o.i(&format!("{p}.n_fuel"), x.n_fuel);
    o.of(&format!("{p}.agreement"), x.agreement);
    o.fs(&format!("{p}.kappa"), &x.kappa);
    o.b(&format!("{p}.kappa_pure"), x.kappa_pure);
    o.b(&format!("{p}.riding4_valid"), x.riding4_valid);
    o.b(&format!("{p}.window_open"), x.window_open);
    o.ob(&format!("{p}.ds_stable"), x.ds_stable);
    o.of(&format!("{p}.tau_star_fine"), x.tau_star_fine);
}

fn law(o: &mut Flat, p: &str, r: &ThresholdLaw) {
    o.keys(p, 25);
    o.fs(&format!("{p}.rs"), &r.rs);
    o.f(&format!("{p}.tau_gov"), r.tau_gov);
    o.f(&format!("{p}.tau_ref"), r.tau_ref);
    o.f(&format!("{p}.phi_lim"), r.phi_lim);
    o.of(&format!("{p}.phi_air"), r.phi_air);
    o.pair(&format!("{p}.bracket"), r.bracket.0, r.bracket.1);
    o.f(&format!("{p}.ds"), r.ds);
    o.of(&format!("{p}.ds_fine"), r.ds_fine);
    o.len(&format!("{p}.rows"), r.rows.len());
    for (k, x) in r.rows.iter().enumerate() {
        match x {
            ThresholdRow::Void(v) => row_void(o, &format!("{p}.rows.{k}"), v),
            ThresholdRow::Ok(v) => row_ok(o, &format!("{p}.rows.{k}"), v),
        }
    }
    o.i(&format!("{p}.n_void"), r.n_void);
    o.fs(&format!("{p}.p1_fixed_in"), &r.p1_fixed_in);
    o.fs(&format!("{p}.p1_fixed_out"), &r.p1_fixed_out);
    o.fs(&format!("{p}.p1_fwd_in"), &r.p1_fwd_in);
    o.fs(&format!("{p}.p1_fwd_out"), &r.p1_fwd_out);
    o.fs(&format!("{p}.p2_fixed_early"), &r.p2_fixed_early);
    o.fs(&format!("{p}.p2_fwd_early"), &r.p2_fwd_early);
    o.b(&format!("{p}.p2_all_early"), r.p2_all_early);
    o.b(&format!("{p}.p3_fwd_never_better"), r.p3_fwd_never_better);
    o.fs(&format!("{p}.kappa_seen"), &r.kappa_seen);
    o.b(&format!("{p}.all_kappa_pure"), r.all_kappa_pure);
    o.fs(&format!("{p}.ds_stable"), &r.ds_stable);
    o.fs(&format!("{p}.ds_unstable"), &r.ds_unstable);
    o.fs(&format!("{p}.ds_unrun"), &r.ds_unrun);
    o.pairs(&format!("{p}.thresholds"), &r.thresholds);
    o.b(&format!("{p}.monotone_in_r"), r.monotone_in_r);
}

fn ref_row(o: &mut Flat, p: &str, x: &RefRow) {
    o.keys(p, 12);
    o.f(&format!("{p}.tau_ref"), x.tau_ref);
    o.of(&format!("{p}.fwd"), x.fwd);
    o.b(&format!("{p}.ref_above"), x.ref_above);
    o.b(&format!("{p}.early"), x.early);
    o.of(&format!("{p}.err"), x.err);
    o.f(&format!("{p}.dist"), x.dist);
    o.of(&format!("{p}.s_bind"), x.s_bind);
    o.i(&format!("{p}.n_riding4"), x.n_riding4);
    o.i(&format!("{p}.n_fuel"), x.n_fuel);
    o.b(&format!("{p}.window_open"), x.window_open);
    o.b(&format!("{p}.riding4_valid"), x.riding4_valid);
    o.b(&format!("{p}.kappa_pure"), x.kappa_pure);
}

fn ref_full(o: &mut Flat, p: &str, r: &RefFull) {
    o.keys(p, 14);
    o.f(&format!("{p}.r"), r.r);
    o.f(&format!("{p}.tau_star"), r.tau_star);
    o.pair(&format!("{p}.bracket"), r.bracket[0], r.bracket[1]);
    o.f(&format!("{p}.width"), r.width);
    o.fs(&format!("{p}.tau_refs"), &r.tau_refs);
    o.len(&format!("{p}.rows"), r.rows.len());
    for (k, x) in r.rows.iter().enumerate() {
        ref_row(o, &format!("{p}.rows.{k}"), x);
    }
    o.none(&format!("{p}.void"));
    o.i(&format!("{p}.n_live"), r.n_live);
    o.b(&format!("{p}.sign_follows_reference"), r.sign_follows_reference);
    o.i(&format!("{p}.n_agree"), r.n_agree);
    o.len(&format!("{p}.crossing"), r.crossing.len());
    for (k, (t, e)) in r.crossing.iter().enumerate() {
        o.len(&format!("{p}.crossing.{k}"), 2);
        o.f(&format!("{p}.crossing.{k}.0"), *t);
        o.b(&format!("{p}.crossing.{k}.1"), *e);
    }
    o.ob(&format!("{p}.grows_above"), r.grows_above);
    o.ob(&format!("{p}.grows_below"), r.grows_below);
    o.len(&format!("{p}.s_binds"), r.s_binds.len());
    for (k, (t, s)) in r.s_binds.iter().enumerate() {
        o.len(&format!("{p}.s_binds.{k}"), 2);
        o.f(&format!("{p}.s_binds.{k}.0"), *t);
        o.of(&format!("{p}.s_binds.{k}.1"), *s);
    }
}

fn reference(o: &mut Flat, p: &str, r: &ThresholdReference) {
    match r {
        ThresholdReference::Void { r, void } => {
            o.keys(p, 3);
            o.f(&format!("{p}.r"), *r);
            o.s(&format!("{p}.void"), void);
            o.len(&format!("{p}.rows"), 0);
        }
        ThresholdReference::Full(f) => ref_full(o, p, f),
    }
}

fn at_ok(o: &mut Flat, p: &str, x: &AtOk) {
    o.keys(p, 15);
    o.f(&format!("{p}.phi_lim"), x.phi_lim);
    o.f(&format!("{p}.tau_gov"), x.tau_gov);
    o.b(&format!("{p}.ok"), true);
    o.none(&format!("{p}.void"));
    o.f(&format!("{p}.tau_star"), x.tau_star);
    o.f(&format!("{p}.width"), x.width);
    o.of(&format!("{p}.gap_bind"), x.gap_bind);
    o.of(&format!("{p}.ratio_bind"), x.ratio_bind);
    o.of(&format!("{p}.s_bind"), x.s_bind);
    o.i(&format!("{p}.n_riding4"), x.n_riding4);
    o.i(&format!("{p}.n_fuel"), x.n_fuel);
    o.b(&format!("{p}.riding4_valid"), x.riding4_valid);
    o.of(&format!("{p}.slope_f"), x.slope_f);
    o.of(&format!("{p}.slope_r"), x.slope_r);
    o.of(&format!("{p}.kappa"), x.kappa);
}

fn ats(o: &mut Flat, p: &str, xs: &[TermsAt]) {
    o.len(p, xs.len());
    for (k, x) in xs.iter().enumerate() {
        let q = format!("{p}.{k}");
        match x {
            TermsAt::Void { phi_lim, tau_gov, void } => {
                o.keys(&q, 4);
                o.f(&format!("{q}.phi_lim"), *phi_lim);
                o.f(&format!("{q}.tau_gov"), *tau_gov);
                o.b(&format!("{q}.ok"), false);
                o.s(&format!("{q}.void"), void);
            }
            TermsAt::Ok(a) => at_ok(o, &q, a),
        }
    }
}

fn p4(o: &mut Flat, p: &str, x: &Option<P4>) {
    let Some(x) = x else {
        o.none(p);
        return;
    };
    o.keys(p, 9);
    o.f(&format!("{p}.measured"), x.measured);
    o.of(&format!("{p}.predicted"), x.predicted);
    o.b(&format!("{p}.rises"), x.rises);
    o.of(&format!("{p}.rel_err"), x.rel_err);
    o.of(&format!("{p}.transfer"), x.transfer);
    o.len(&format!("{p}.secants"), x.secants.len());
    for (k, s) in x.secants.iter().enumerate() {
        let q = format!("{p}.secants.{k}");
        o.keys(&q, 4);
        o.f(&format!("{q}.lo"), s.lo);
        o.f(&format!("{q}.hi"), s.hi);
        o.f(&format!("{q}.measured"), s.measured);
        o.of(&format!("{q}.predicted"), s.predicted);
    }
    o.ofs(&format!("{p}.kappa"), &x.kappa);
    o.ofs(&format!("{p}.ratio_bind"), &x.ratio_bind);
    o.pairs(&format!("{p}.taus"), &x.taus);
}

fn p5(o: &mut Flat, p: &str, x: &Option<P5>) {
    let Some(x) = x else {
        o.none(p);
        return;
    };
    o.keys(p, 15);
    o.of(&format!("{p}.d_gap"), x.d_gap);
    o.of(&format!("{p}.d_lag"), x.d_lag);
    o.of(&format!("{p}.d_slope_f"), x.d_slope_f);
    o.of(&format!("{p}.d_ratio"), x.d_ratio);
    o.b(&format!("{p}.rises"), x.rises);
    o.b(&format!("{p}.separates"), x.separates);
    o.b(&format!("{p}.v7_withdrawn"), x.v7_withdrawn);
    o.b(&format!("{p}.gap_falls"), x.gap_falls);
    o.b(&format!("{p}.ratio_rises"), x.ratio_rises);
    o.b(&format!("{p}.monotone"), x.monotone);
    o.pairs(&format!("{p}.taus"), &x.taus);
    o.ofs(&format!("{p}.gaps"), &x.gaps);
    o.ofs(&format!("{p}.ratios"), &x.ratios);
    o.ofs(&format!("{p}.slope_f"), &x.slope_f);
    o.ofs(&format!("{p}.slope_r"), &x.slope_r);
}

fn terms(o: &mut Flat, p: &str, r: &ThresholdTerms) {
    o.keys(p, 12);
    o.f(&format!("{p}.r"), r.r);
    o.f(&format!("{p}.phi_lim"), r.phi_lim);
    o.fs(&format!("{p}.phi_lims"), &r.phi_lims);
    o.of(&format!("{p}.phi_air"), r.phi_air);
    o.f(&format!("{p}.ds"), r.ds);
    o.fs(&format!("{p}.tau_govs"), &r.tau_govs);
    o.pair(&format!("{p}.bracket"), r.bracket.0, r.bracket.1);
    ats(o, &format!("{p}.govs"), &r.govs);
    ats(o, &format!("{p}.walls"), &r.walls);
    o.i(&format!("{p}.n_void"), r.n_void);
    p4(o, &format!("{p}.p4"), &r.p4);
    p5(o, &format!("{p}.p5"), &r.p5);
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

fn check(name: &str, m: &ScheduledStatorCore, mut got: Flat) {
    rig_after(&mut got, name, m);
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

/// `test_rung82.py`'s `law` fixture: three ramps, the step control SKIPPED (`ds_fine = None`).
#[test]
fn law_fixture_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let r = threshold_law::threshold_law(
        &m, &f, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &RS, TAU_GOV, TAU_REF, TAU_Q, TAU_S, BRACKET,
        N_BISECT, SETTLE, DS, None, V_MAX, false);
    let mut o = Flat::default();
    law(&mut o, "law", &r);
    check("law", &m, o);
}

/// `test_rung82.py`'s `ref` fixture: § 3a at `r = 0.35`, five references.
#[test]
fn ref_fixture_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let r = threshold_law::threshold_reference(
        &m, &f, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, R_REF, &REF_TAU_REFS, TAU_GOV, TAU_Q, TAU_S,
        BRACKET, N_BISECT, SETTLE, DS, V_MAX, false);
    let mut o = Flat::default();
    reference(&mut o, "ref", &r);
    check("ref", &m, o);
}

/// `test_rung82.py`'s `terms` fixture: three governor clocks, three walls.
#[test]
fn terms_fixture_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let r = threshold_law::threshold_terms(
        &m, &f, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &[0.745, 0.750, 0.755], R_REF,
        &TERMS_TAU_GOVS, TAU_Q, TAU_S, BRACKET, N_BISECT, SETTLE, DS, V_MAX, false);
    let mut o = Flat::default();
    terms(&mut o, "terms", &r);
    check("terms", &m, o);
}

/// `test_the_identity_control_reproduces_rung81s_own_cell` — rung 81's matched cell.
#[test]
fn identity_scan_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let s = threshold_law::threshold_scan(&m, 0.05, &kw(&f, R81, PHI_FUEL));
    let mut o = Flat::default();
    scan(&mut o, "scan_ctl", &s);
    check("scan_ctl", &m, o);
}

/// `test_V1_no_four_loop_window_above_the_admissible_ramp` — the EMPTY window: every Option
/// `None`, `kappa = []`.
#[test]
fn empty_window_scan_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let s = threshold_law::threshold_scan(&m, 0.05, &kw(&f, 1.0, PHI_FUEL));
    let mut o = Flat::default();
    scan(&mut o, "scan_v1", &s);
    check("scan_v1", &m, o);
}

/// `test_V3_censors_the_wall_on_both_sides`, the high wall: censored BELOW.
#[test]
fn v3_high_wall_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let b = threshold_law::bisect(&m, key_fuel, 0.004, 0.30, 4, &kw(&f, 0.35, 0.760));
    let mut o = Flat::default();
    bisect(&mut o, "bisect_v3_hi", &b);
    check("bisect_v3_hi", &m, o);
}

/// `test_V3_censors_the_wall_on_both_sides`, the low wall: censored ABOVE.
#[test]
fn v3_low_wall_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let b = threshold_law::bisect(&m, key_fuel, 0.004, 0.30, 4, &kw(&f, 0.35, 0.740));
    let mut o = Flat::default();
    bisect(&mut o, "bisect_v3_lo", &b);
    check("bisect_v3_lo", &m, o);
}

/// V3 on a bracket end whose `%g` (`0.0123457`) is NOT Rust's `{}` (`0.0123456789`).
#[test]
fn v3_message_is_percent_g_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let b = threshold_law::bisect(&m, key_fuel, 0.0123456789, 0.30, 4, &kw(&f, 0.35, 0.760));
    let mut o = Flat::default();
    bisect(&mut o, "bisect_v3_g", &b);
    check("bisect_v3_g", &m, o);
}

/// V1 with BOTH ends empty (`r = 1.0`, `[0.004, 0.05]`) — the `VoidEnd` shape pinned directly, and
/// the only reading that sees `_bisect` test the WINDOW before the straddle. On the default bracket
/// the `0.30` end has 14 riding points (all fuel), so V1 fires off the low end alone and the
/// reverse order returns the same void there (plan § 5.34.3, K16).
#[test]
fn v1_both_ends_empty_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let b = threshold_law::bisect(&m, key_fuel, 0.004, 0.05, 4, &kw(&f, 1.0, PHI_FUEL));
    let mut o = Flat::default();
    bisect(&mut o, "bisect_v1", &b);
    check("bisect_v1", &m, o);
}

/// The `ds_fine` leg and a VOID row (`r = 1.0`).
#[test]
fn law_with_step_control_and_a_void_row_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let r = threshold_law::threshold_law(
        &m, &f, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &[0.25, 1.0], TAU_GOV, TAU_REF, TAU_Q, TAU_S,
        BRACKET, N_BISECT, SETTLE, DS, Some(threshold_law::DS_FINE), V_MAX, false);
    let mut o = Flat::default();
    law(&mut o, "law_fine", &r);
    check("law_fine", &m, o);
}

/// § 3a's VOID shape (`r = 1.0`).
#[test]
fn ref_void_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let r = threshold_law::threshold_reference(
        &m, &f, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, 1.0, &REF_TAU_REFS, TAU_GOV, TAU_Q, TAU_S,
        BRACKET, N_BISECT, SETTLE, DS, V_MAX, false);
    let mut o = Flat::default();
    reference(&mut o, "ref_void", &r);
    check("ref_void", &m, o);
}

/// Unsorted walls with one censored: sorted iteration, as-passed field, void `at`, no P4/P5.
#[test]
fn terms_void_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let r = threshold_law::threshold_terms(
        &m, &f, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &[0.760, 0.750], R_REF, &[0.05], TAU_Q,
        TAU_S, BRACKET, 4, SETTLE, DS, V_MAX, false);
    let mut o = Flat::default();
    terms(&mut o, "terms_void", &r);
    check("terms_void", &m, o);
}
