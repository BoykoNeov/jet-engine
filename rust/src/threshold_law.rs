//! RUNG 82 — **THE THRESHOLD'S OWN LAW.** `ThresholdLawTransient` (`engine.py:22148`), slice AJ.
//!
//! Rung 81 scored its criterion as a LABEL predictor (99%) and never asked whether it predicts
//! the THRESHOLD — the `tau_f` at which the fuel region opens. Headline: **a threshold is a fixed
//! point, not a formula.** The criterion's inputs are read off a trajectory that `tau_f` itself
//! moves, so read FORWARD off a fixed reference march it is wrong by a multiple, and read as the
//! ROOT of its own residual it lands. And the swept knob is not the binding clock: every binding
//! point is in RELEASE, where rung 52's lag runs at `3·tau_f`.
//!
//! # NO TABLE, NO BUILDER — A RUNG-80 CORE PLUS THE NINE FREE FUNCTIONS BELOW
//!
//! Reader-only, as [`authority_clock`](crate::authority_clock) says for all four rungs (plan
//! § 5.34 (ii)): no `R82` table, no builder. The nine methods are [`scan_cells`]
//! (`_scan_cells`), [`hats`] (`_hats`), [`threshold_scan`] (`_threshold_scan`), [`scan`]
//! (`_scan`), [`bisect`] (`_bisect`), [`threshold_law`], [`threshold_row`] (`_threshold_row`),
//! [`threshold_reference`] and [`threshold_terms`]. Gated bit for bit, every returned value in
//! Python's own key order, in `tests/slice_aj_threshold.rs` against
//! `oracle/probe_slice_aj_step3.py`'s output.
//!
//! # WHAT THE PORT HONOURS
//!
//! * **Python's `**kw` is [`ScanKw`]** — every `_threshold_scan` argument except `tau_f`, which
//!   is the one [`bisect`] moves. Python's `key` lambdas are `Fn(&ThresholdScan) -> bool`; the
//!   two the readers pass are [`key_fuel`] (`n_fuel > 0`) and [`key_residual`] (`h < 0`).
//! * **`_scan` is named in THIS module and called fully qualified.** `stator.rs` already has a
//!   `scan` (rung 53's `VariableStatorMatcher._scan`, an unrelated hierarchy — a reused name, not
//!   a cell). No bare `use` of either.
//! * **`id(p)` → indices** in `_scan_cells` (`engine.py:22218`, `engine.py:22220`), through
//!   [`riding4_idx`](crate::shared_actuator::riding4_idx) — the same port rung 81's
//!   `authority_clock` uses.
//! * **Four dict SHAPES, not one, wherever Python returns different keys**: `_bisect` has four
//!   ([`Bisect`]), `_threshold_row` two ([`ThresholdRow`]), `threshold_reference` two
//!   ([`ThresholdReference`]), `threshold_terms`'s `at` two ([`TermsAt`]). Their key orders
//!   differ — `void` is the FIRST key of a void dict and the FIFTH of `_bisect`'s ok one — and
//!   the gate's flattening pins each.
//! * **`min(hats, key=…)`** (`engine.py:22255`) is the FIRST minimum in iteration order — a
//!   strict-`<` loop. **`tau_hat_min` is `min(hat)`, not the argmin's `hat`**: the two coincide
//!   only where `tau_eff` is the same at every hat.
//! * **`round(c.tau_f/tau_f, 6)`** (`engine.py:22256`) is [`round6`](crate::two_spool::round6),
//!   over EVERY scored cell (not only the hats), guarded by `tau_f > 0.0` as Python guards it.
//! * **The `*_bind` lookups match `c.s == arg.s` by FLOAT EQUALITY over every cell** — here
//!   Python itself compares `s`, so the port does too (unlike the `id()` sites).
//! * **`_bisect` scans BOTH ends before testing either**, returns the CURRENT bracket on a
//!   mid-bisection V1, and its ok branch runs a FRESH scan at `b` (`engine.py:22316`), kept: the
//!   march is deterministic, so no value can see it, but it is Python's cost and Python's shape.
//! * **The V3 message is `%g`** (`engine.py:22305`) → [`py_g`](crate::demand_coordinate::py_g).
//! * **`sorted(…, key=dist)`** (`engine.py:22495`) and `sorted(phi_lims)` are STABLE →
//!   `sort_by`, never `sort_unstable_by`.
//! * **`float('nan')` is a returned VALUE** in `_threshold_row` (`engine.py:22414`), on an ok row
//!   whose reference march is kappa-IMPURE.
//! * **Truthiness is ported as truthiness.** `if kap`, `if b["ratio_bind"] and b["kappa"]`,
//!   `if a["gap_bind"]` treat `0.0` as absent; `pred not in (None, 0.0)` likewise; but
//!   `gap_falls`/`ratio_rises` test `is not None`. Each is kept as written ([`truthy`]).
//! * **Every refusal is Python's exception**: `TypeError` for a `None` compared or subtracted on
//!   the `b` side of `threshold_terms`' P5, `ZeroDivisionError` for a zero denominator
//!   ([`py_div`]). (`kappa[0]` cannot raise: [`kappa0`] reads it only when `kappa_pure`, i.e.
//!   exactly one element.)

use crate::authority_clock::{authority_of, b_max_of, criterion_at, Criterion};
use crate::demand_coordinate::py_g;
use crate::engine::FlightCondition;
use crate::fuel_transient::{Authority, FuelPoint};
use crate::shared_actuator::riding4_idx;
use crate::split_wall::split_march;
use crate::stator_transient::ScheduledStatorCore;
// this module's own path, so every `_scan` call site reads `threshold_law::scan` (header)
use crate::threshold_law;
use crate::two_lag::{py_max_of, py_min_of};
use crate::two_spool::round6;

// ---------------------------------------------------------------------------------------------
// THE READERS' DEFAULTS, named once so the ported suite and the oracle cannot drift from them
// ---------------------------------------------------------------------------------------------

/// `threshold_law`'s default `rs`.
pub const LAW_RS: [f64; 5] = [0.20, 0.25, 0.35, 0.50, 0.70];
/// `threshold_reference`'s default `tau_refs`.
pub const REF_TAU_REFS: [f64; 5] = [0.02, 0.03, 0.05, 0.08, 0.12];
/// `threshold_terms`' default `phi_lims`.
pub const TERMS_PHI_LIMS: [f64; 5] = [0.745, 0.7475, 0.750, 0.7525, 0.755];
/// `threshold_terms`' default `tau_govs`.
pub const TERMS_TAU_GOVS: [f64; 3] = [0.02, 0.05, 0.20];
/// Every reader's default `bracket`.
pub const BRACKET: (f64, f64) = (0.004, 0.30);
/// Every reader's default `n_bisect`.
pub const N_BISECT: usize = 10;
/// `threshold_law`'s default `ds_fine`.
pub const DS_FINE: f64 = 0.0025;

// ---------------------------------------------------------------------------------------------
// SMALL PYTHON SHAPES
// ---------------------------------------------------------------------------------------------

/// Python's `bool(x)` for an optional float: `None` and `0.0` are false, and so is NOTHING else —
/// `nan` is true, as in Python.
pub(crate) fn truthy(x: Option<f64>) -> bool {
    matches!(x, Some(v) if v != 0.0)
}

/// Python's `a / b` on floats — `ZeroDivisionError` where Rust would return an infinity.
pub(crate) fn py_div(a: f64, b: f64) -> f64 {
    assert!(b != 0.0, "rung-82: float division by zero -- Python raises ZeroDivisionError here");
    a / b
}

/// A `None` that Python would subtract or compare — its `TypeError`.
pub(crate) fn need(x: Option<f64>, what: &str) -> f64 {
    x.unwrap_or_else(|| panic!("rung-82: `{what}` is None where Python does arithmetic on it -- \
                                Python raises TypeError here"))
}

/// Python's `kappa[0] if kappa_pure else …` — the one read of a pure scan's clock map.
pub(crate) fn kappa0(s: &ThresholdScan) -> Option<f64> {
    if s.kappa_pure { Some(s.kappa[0]) } else { None }
}

/// Python's `sorted(xs)` on floats — stable, and it refuses a NaN rather than guess its order.
pub(crate) fn sorted_f(xs: &[f64]) -> Vec<f64> {
    let mut v = xs.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).expect("rung-82: sorted() over a NaN"));
    v
}

// ---------------------------------------------------------------------------------------------
// THE RESIDUAL, AND THE TWO CURRENCIES IT LIVES IN
// ---------------------------------------------------------------------------------------------

/// Python's `**kw` for one scan — every [`threshold_scan`] argument except `tau_f`.
#[derive(Clone, Copy, Debug)]
pub struct ScanKw<'a> {
    pub flight: &'a FlightCondition,
    pub tt4_lo: f64,
    pub tt4_hi: f64,
    pub tt4_max: f64,
    pub phi_lim: f64,
    pub phi_air: Option<f64>,
    pub tau_gov: f64,
    pub tau_q: f64,
    pub tau_s: f64,
    pub r: f64,
    pub s_settle: f64,
    pub ds: f64,
    pub v_max: f64,
    pub inc: bool,
}

/// RUNG 82's `_scan_cells` (`engine.py:22201`) — **the march and the points it scores.**
///
/// Returns the trajectory, the riding-four INDICES (Python's `ride`, as `id(p)` would find them,
/// trajectory ends included) and the interior cells, in trajectory order. Always the `demand`
/// coordinate: `_threshold_scan` asks for no other.
pub fn scan_cells(
    core: &ScheduledStatorCore, tau_f: f64, kw: &ScanKw,
) -> (Vec<FuelPoint>, Vec<usize>, Vec<Criterion>) {
    let (m, _surge, lag, traj) = split_march(
        core, kw.flight, kw.tt4_lo, kw.tt4_hi, kw.tt4_max, kw.phi_lim, kw.phi_air, "demand",
        (tau_f, kw.tau_gov, kw.tau_q, kw.tau_s), kw.r, kw.s_settle, kw.ds, kw.v_max, kw.inc);
    let ride = riding4_idx(&traj, b_max_of(&m));
    let cells = ride.iter()
        .filter(|&&i| 0 < i && i < traj.len() - 1)
        .map(|&i| criterion_at(&traj, i, "demand", lag.as_ref(), kw.tau_gov))
        .collect();
    (traj, ride, cells)
}

/// One implied threshold — Python's `(hat, c["tau_f"], c["s"])`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hat {
    /// `(gap + tau_gov·slope_r) / slope_f` — the POINT-WISE threshold, in the EFFECTIVE clock.
    pub hat: f64,
    /// The cell's ACTIVE fuel clock.
    pub tau_eff: f64,
    pub s: f64,
}

/// RUNG 82's `_hats` (`engine.py:22224`) — the point-wise implied threshold, and the count of
/// cells excluded BY NAME because `slope_f <= 0` (a sign the derivation does not cover).
pub fn hats(cells: &[Criterion], tau_gov: f64) -> (Vec<Hat>, usize) {
    let (mut out, mut n_bad) = (Vec::new(), 0usize);
    for c in cells {
        if c.slope_f <= 0.0 {
            n_bad += 1;
            continue;
        }
        out.push(Hat {
            hat: (c.setpoint_gap + tau_gov * c.slope_r) / c.slope_f,
            tau_eff: c.tau_f,
            s: c.s,
        });
    }
    (out, n_bad)
}

/// ONE march, reduced — Python's `_threshold_scan` dict, fields in its key order.
#[derive(Clone, Debug, PartialEq)]
pub struct ThresholdScan {
    pub tau_f: f64,
    pub tau_gov: f64,
    pub r: f64,
    pub ds: f64,
    pub phi_lim: f64,
    pub phi_air: Option<f64>,
    pub npts: usize,
    pub n_riding4: usize,
    pub max_tt4: f64,
    pub min_phi: f64,
    /// V2: the plant accelerated at all.
    pub riding4_valid: bool,
    /// V1: a four-loop window exists.
    pub window_open: bool,
    pub n_scored: usize,
    pub n_slope_excluded: usize,
    /// The MEASURED detector.
    pub n_fuel: usize,
    pub n_gov: usize,
    pub n_pred_fuel: usize,
    pub n_agree: usize,
    /// The criterion's own residual, `min(hat - tau_eff)`.
    pub h: Option<f64>,
    pub tau_hat_min: Option<f64>,
    pub s_bind: Option<f64>,
    pub tau_eff_bind: Option<f64>,
    /// `sorted({round(tau_eff/tau_f, 6)})` — READ, never imposed.
    pub kappa: Vec<f64>,
    pub kappa_pure: bool,
    pub gap_bind: Option<f64>,
    pub ratio_bind: Option<f64>,
    pub slope_f_bind: Option<f64>,
    pub slope_r_bind: Option<f64>,
}

/// RUNG 82's `_threshold_scan` (`engine.py:22239`) — **one march, reduced to the three numbers a
/// threshold needs, and to the flags that say whether they may be quoted at all.**
pub fn threshold_scan(core: &ScheduledStatorCore, tau_f: f64, kw: &ScanKw) -> ThresholdScan {
    let (traj, ride, cells) = scan_cells(core, tau_f, kw);
    let tt4s: Vec<f64> = traj.iter().map(|p| p.tt4).collect();
    let phis: Vec<f64> = traj.iter().map(|p| p.phi_lp).collect();
    let max_t = py_max_of(&tt4s);
    let (hs, n_bad) = hats(&cells, kw.tau_gov);
    let resid: Vec<f64> = hs.iter().map(|x| x.hat - x.tau_eff).collect();
    let h = if resid.is_empty() { None } else { Some(py_min_of(&resid)) };
    // `min(hats, key=lambda t: t[0] - t[1])` — the FIRST minimum, as Python's `min` keeps it
    let mut arg: Option<&Hat> = None;
    for (x, &k) in hs.iter().zip(&resid) {
        if arg.map_or(true, |a| k < a.hat - a.tau_eff) {
            arg = Some(x);
        }
    }
    let kap = if tau_f > 0.0 {
        let mut v = sorted_f(&cells.iter().map(|c| round6(c.tau_f / tau_f)).collect::<Vec<_>>());
        v.dedup();
        v
    } else {
        Vec::new()
    };
    let at_bind = |c: &&Criterion| arg.is_some_and(|a| c.s == a.s);
    let hat_vals: Vec<f64> = hs.iter().map(|x| x.hat).collect();
    let count = |a: Authority| ride.iter().filter(|&&i| authority_of(&traj[i]) == a).count();
    ThresholdScan {
        tau_f,
        tau_gov: kw.tau_gov,
        r: kw.r,
        ds: kw.ds,
        phi_lim: kw.phi_lim,
        phi_air: kw.phi_air,
        npts: traj.len(),
        n_riding4: ride.len(),
        max_tt4: max_t,
        min_phi: py_min_of(&phis),
        riding4_valid: max_t > kw.tt4_lo * (1.0 + 1e-9),
        window_open: !ride.is_empty(),
        n_scored: cells.len(),
        n_slope_excluded: n_bad,
        n_fuel: count(Authority::Fuel),
        n_gov: count(Authority::Gov),
        n_pred_fuel: cells.iter().filter(|c| c.predicted == Authority::Fuel).count(),
        n_agree: cells.iter().filter(|c| c.predicted == c.measured).count(),
        h,
        tau_hat_min: if hat_vals.is_empty() { None } else { Some(py_min_of(&hat_vals)) },
        s_bind: arg.map(|a| a.s),
        tau_eff_bind: arg.map(|a| a.tau_eff),
        kappa_pure: kap.len() == 1,
        kappa: kap,
        gap_bind: cells.iter().find(at_bind).map(|c| c.setpoint_gap),
        ratio_bind: cells.iter()
            .find(|c| at_bind(c) && c.slope_r != 0.0)
            .map(|c| c.slope_f / c.slope_r),
        slope_f_bind: cells.iter().find(at_bind).map(|c| c.slope_f),
        slope_r_bind: cells.iter().find(at_bind).map(|c| c.slope_r),
    }
}

/// RUNG 82's `_scan` (`engine.py:22286`) — [`threshold_scan`] at one `tau_f`. Call it
/// `threshold_law::scan`, never through a bare `use` (see the module header).
pub fn scan(core: &ScheduledStatorCore, tau_f: f64, kw: &ScanKw) -> ThresholdScan {
    threshold_scan(core, tau_f, kw)
}

/// The MEASURED detector — Python's `lambda s: s["n_fuel"] > 0`.
pub fn key_fuel(s: &ThresholdScan) -> bool {
    s.n_fuel > 0
}

/// The criterion's OWN detector — Python's `lambda s: s["h"] is not None and s["h"] < 0.0`.
pub fn key_residual(s: &ThresholdScan) -> bool {
    s.h.is_some_and(|h| h < 0.0)
}

/// A bisection that converged — Python's ok dict: `lo, hi, mid, width, void=None, at_hi`.
#[derive(Clone, Debug, PartialEq)]
pub struct BisectOk {
    pub lo: f64,
    pub hi: f64,
    pub mid: f64,
    pub width: f64,
    /// A FRESH scan at the final `hi`.
    pub at_hi: ThresholdScan,
}

/// `_bisect`'s four return shapes, each in its own key order.
#[derive(Clone, Debug, PartialEq)]
pub enum Bisect {
    /// `void, lo, hi, at_lo, at_hi` — V1 at a bracket end.
    VoidEnd { void: String, lo: f64, hi: f64, at_lo: ThresholdScan, at_hi: ThresholdScan },
    /// `void, lo, hi, at_lo, at_hi, below, above` — V3, the ends do not straddle.
    VoidStraddle {
        void: String, lo: f64, hi: f64, at_lo: ThresholdScan, at_hi: ThresholdScan,
        below: bool, above: bool,
    },
    /// `void, lo, hi` — V1 mid-bisection, with the CURRENT bracket.
    VoidMid { void: String, lo: f64, hi: f64 },
    Ok(BisectOk),
}

impl Bisect {
    /// Python's `b.get("void")`.
    pub fn void(&self) -> Option<&str> {
        match self {
            Bisect::VoidEnd { void, .. }
            | Bisect::VoidStraddle { void, .. }
            | Bisect::VoidMid { void, .. } => Some(void),
            Bisect::Ok(_) => None,
        }
    }

    /// The ok dict — Python reads `mid`/`width`/`at_hi` only after testing `void`, so a void
    /// here is its `KeyError`.
    pub fn ok(&self) -> &BisectOk {
        match self {
            Bisect::Ok(b) => b,
            _ => panic!("rung-82: a VOID bisection has no mid/width/at_hi -- Python raises \
                         KeyError here"),
        }
    }
}

/// RUNG 82's `_bisect` (`engine.py:22289`) — **bisect `tau_f` on a MONOTONE detector, returning
/// the BRACKET and never a point.** The endpoints are scanned first and the row voided if they
/// do not straddle (V3): an endpoint is a CENSORED observation, not a value.
///
/// **The WINDOW is tested before the straddle, and only a both-ends-empty bracket can see the
/// order.** At `r = 1.0` the default bracket's `0.30` end is NOT empty — 14 riding points, all
/// fuel, on a VALID march (`riding4_valid`, `max_Tt4` 1198.6 K), measured — so V1 fires off the
/// low end alone, and the reverse order returns the same void there. Rung 82's docstring
/// *"at `r >= 1.0` there is no four-loop point at all"* (`engine.py:22329`) drops the scope its
/// anchor's E1 states — measured *"at rung 80's clocks"*, i.e. `tau_f = 0.05` — and is false at
/// `0.30`. The void verdict at `r = 1.0` stands (V1 fires on EITHER empty end); only its stated
/// reason is too wide (plan § 5.34.3 and its addendum).
pub fn bisect(
    core: &ScheduledStatorCore, key: impl Fn(&ThresholdScan) -> bool, lo: f64, hi: f64, n: usize,
    kw: &ScanKw,
) -> Bisect {
    let (s_lo, s_hi) = (threshold_law::scan(core, lo, kw), threshold_law::scan(core, hi, kw));
    if !(s_lo.window_open && s_hi.window_open) {
        return Bisect::VoidEnd {
            void: "V1: four-loop window empty at a bracket end".into(),
            lo, hi, at_lo: s_lo, at_hi: s_hi,
        };
    }
    if key(&s_lo) || !key(&s_hi) {
        return Bisect::VoidStraddle {
            void: format!("V3: threshold not strictly inside [{}, {}]", py_g(lo), py_g(hi)),
            lo, hi, below: key(&s_lo), above: !key(&s_hi), at_lo: s_lo, at_hi: s_hi,
        };
    }
    let (mut a, mut b) = (lo, hi);
    for _ in 0..n {
        let mid = 0.5 * (a + b);
        let s = threshold_law::scan(core, mid, kw);
        if !s.window_open {
            return Bisect::VoidMid { void: "V1: window closed mid-bisection".into(), lo: a, hi: b };
        }
        if key(&s) { b = mid } else { a = mid }
    }
    Bisect::Ok(BisectOk {
        lo: a, hi: b, mid: 0.5 * (a + b), width: b - a, at_hi: threshold_law::scan(core, b, kw),
    })
}

// ---------------------------------------------------------------------------------------------
// §§ 1–3: THE RAMP SWEEP, AND THE THREE READINGS
// ---------------------------------------------------------------------------------------------

/// A void ramp — Python's void row, keys `r, ok=False, void, kappa, kappa_pure, n_riding4,
/// window_open, riding4_valid` (the last five off the REFERENCE scan).
#[derive(Clone, Debug, PartialEq)]
pub struct RowVoid {
    pub r: f64,
    pub void: String,
    pub kappa: Vec<f64>,
    pub kappa_pure: bool,
    pub n_riding4: usize,
    pub window_open: bool,
    pub riding4_valid: bool,
}

/// A live ramp — Python's ok row, fields in its key order (`ok=True`, `void=None` implied).
#[derive(Clone, Debug, PartialEq)]
pub struct RowOk {
    pub r: f64,
    /// The MEASURED threshold, the bracket's mid.
    pub tau_star: f64,
    pub lo: f64,
    pub hi: f64,
    pub width: f64,
    /// `tau_star · kappa`, or `nan` on a kappa-IMPURE reference — a returned VALUE.
    pub tau_star_eff: f64,
    /// The FIXED POINT, the `h < 0` bisection's mid.
    pub fixed: f64,
    /// The FORWARD reading.
    pub fwd: Option<f64>,
    pub tau_ref: f64,
    pub tau_hat_min_ref: Option<f64>,
    pub fixed_in_bracket: bool,
    pub fwd_in_bracket: bool,
    pub fixed_early: bool,
    pub fwd_early: bool,
    pub err_fixed: f64,
    pub err_fwd: Option<f64>,
    pub dist_ref: f64,
    pub s_bind: Option<f64>,
    pub gap_bind: Option<f64>,
    pub ratio_bind: Option<f64>,
    pub tau_eff_bind: Option<f64>,
    pub n_riding4: usize,
    pub n_fuel: usize,
    pub agreement: Option<f64>,
    pub kappa: Vec<f64>,
    pub kappa_pure: bool,
    pub riding4_valid: bool,
    /// Always `True` on an ok row.
    pub window_open: bool,
    /// V5: `None` where the control was NOT RUN — never the same as passing it.
    pub ds_stable: Option<bool>,
    pub tau_star_fine: Option<f64>,
}

/// `_threshold_row`'s two shapes.
#[derive(Clone, Debug, PartialEq)]
pub enum ThresholdRow {
    Void(RowVoid),
    Ok(RowOk),
}

impl ThresholdRow {
    /// Python's `x["ok"]`, as the live filter reads it.
    pub fn live(&self) -> Option<&RowOk> {
        match self {
            ThresholdRow::Ok(x) => Some(x),
            ThresholdRow::Void(_) => None,
        }
    }

    /// Python's `x["kappa"]` — both shapes carry it.
    pub fn kappa(&self) -> &[f64] {
        match self {
            ThresholdRow::Ok(x) => &x.kappa,
            ThresholdRow::Void(x) => &x.kappa,
        }
    }

    /// Python's `x["kappa_pure"]` — both shapes carry it.
    pub fn kappa_pure(&self) -> bool {
        match self {
            ThresholdRow::Ok(x) => x.kappa_pure,
            ThresholdRow::Void(x) => x.kappa_pure,
        }
    }
}

/// RUNG 82's `_threshold_row` (`engine.py:22395`) — one ramp's row, **the void reason carried
/// rather than the row dropped**: `meas`'s reason first, then `fix`'s.
pub fn threshold_row(
    r: f64, meas: &Bisect, fix: &Bisect, fwd: Option<f64>, rf: &ThresholdScan,
    fine: Option<&Bisect>, tau_ref: f64,
) -> ThresholdRow {
    if let Some(v) = meas.void().or(fix.void()) {
        return ThresholdRow::Void(RowVoid {
            r,
            void: v.to_string(),
            kappa: rf.kappa.clone(),
            kappa_pure: rf.kappa_pure,
            n_riding4: rf.n_riding4,
            window_open: rf.window_open,
            riding4_valid: rf.riding4_valid,
        });
    }
    let (m, f) = (meas.ok(), fix.ok());
    let (t, fx) = (m.mid, f.mid);
    // `(not fine.get("void")) and abs(fine["mid"] - t) <= meas["width"]`
    let ds_ok = fine.map(|x| x.void().is_none() && (x.ok().mid - t).abs() <= m.width);
    let a = &m.at_hi;
    ThresholdRow::Ok(RowOk {
        r,
        tau_star: t,
        lo: m.lo,
        hi: m.hi,
        width: m.width,
        tau_star_eff: t * kappa0(rf).unwrap_or(f64::NAN),
        fixed: fx,
        fwd,
        tau_ref,
        tau_hat_min_ref: rf.tau_hat_min,
        fixed_in_bracket: m.lo <= fx && fx <= m.hi,
        fwd_in_bracket: fwd.is_some_and(|w| m.lo <= w && w <= m.hi),
        fixed_early: fx < t,
        fwd_early: fwd.is_some_and(|w| w < t),
        err_fixed: py_div((fx - t).abs(), t),
        err_fwd: fwd.map(|w| py_div((w - t).abs(), t)),
        dist_ref: py_div((tau_ref - t).abs(), t),
        s_bind: a.s_bind,
        gap_bind: a.gap_bind,
        ratio_bind: a.ratio_bind,
        tau_eff_bind: a.tau_eff_bind,
        n_riding4: a.n_riding4,
        n_fuel: a.n_fuel,
        agreement: if a.n_scored > 0 {
            Some(a.n_agree as f64 / a.n_scored as f64)
        } else {
            None
        },
        kappa: rf.kappa.clone(),
        kappa_pure: rf.kappa_pure,
        riding4_valid: a.riding4_valid,
        window_open: true,
        ds_stable: ds_ok,
        tau_star_fine: fine.and_then(|x| if x.void().is_some() { None } else { Some(x.ok().mid) }),
    })
}

/// The forward reading off one reference scan: `tau_hat_min / kappa`, where both exist —
/// Python's `if kap and s["tau_hat_min"] is not None` (a `0.0` kappa is falsy).
fn forward(s: &ThresholdScan) -> Option<f64> {
    let kap = kappa0(s);
    match (truthy(kap), s.tau_hat_min) {
        (true, Some(t)) => Some(t / kap.expect("truthy")),
        _ => None,
    }
}

/// §§ 1–3's reading — Python's `threshold_law` return dict, fields in its key order.
#[derive(Clone, Debug, PartialEq)]
pub struct ThresholdLaw {
    pub rs: Vec<f64>,
    pub tau_gov: f64,
    pub tau_ref: f64,
    pub phi_lim: f64,
    pub phi_air: Option<f64>,
    pub bracket: (f64, f64),
    pub ds: f64,
    pub ds_fine: Option<f64>,
    pub rows: Vec<ThresholdRow>,
    pub n_void: usize,
    pub p1_fixed_in: Vec<f64>,
    pub p1_fixed_out: Vec<f64>,
    pub p1_fwd_in: Vec<f64>,
    pub p1_fwd_out: Vec<f64>,
    pub p2_fixed_early: Vec<f64>,
    pub p2_fwd_early: Vec<f64>,
    pub p2_all_early: bool,
    pub p3_fwd_never_better: bool,
    /// Over EVERY row, void ones included.
    pub kappa_seen: Vec<f64>,
    /// Over EVERY row, void ones included.
    pub all_kappa_pure: bool,
    pub ds_stable: Vec<f64>,
    pub ds_unstable: Vec<f64>,
    pub ds_unrun: Vec<f64>,
    pub thresholds: Vec<(f64, f64)>,
    pub monotone_in_r: bool,
}

/// §§ 1–3 (`engine.py:22320`) — **the threshold against the ramp rate, read three ways**:
/// MEASURED (`n_fuel > 0`, bisected), FIXED POINT (the SAME bisection on `h < 0`), FORWARD (one
/// reference march, no search). `ds_fine = None` SKIPS the step control and the row carries
/// `ds_stable = None`. Every leg runs on every ramp, a void one included — Python does not
/// short-circuit.
#[allow(clippy::too_many_arguments)]
pub fn threshold_law(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, phi_air: Option<f64>, rs: &[f64], tau_gov: f64, tau_ref: f64, tau_q: f64,
    tau_s: f64, bracket: (f64, f64), n_bisect: usize, s_settle: f64, ds: f64,
    ds_fine: Option<f64>, v_max: f64, inc: bool,
) -> ThresholdLaw {
    let (lo, hi) = bracket;
    let mut rows: Vec<ThresholdRow> = Vec::new();
    for &r in rs {
        let kw = ScanKw {
            flight, tt4_lo, tt4_hi, tt4_max, phi_lim, phi_air, tau_gov, tau_q, tau_s, r,
            s_settle, ds, v_max, inc,
        };
        let meas = bisect(core, key_fuel, lo, hi, n_bisect, &kw);
        let fix = bisect(core, key_residual, lo, hi, n_bisect, &kw);
        let rf = threshold_law::scan(core, tau_ref, &kw);
        let fwd = forward(&rf);
        let fine = ds_fine.map(|d| bisect(core, key_fuel, lo, hi, n_bisect, &ScanKw { ds: d, ..kw }));
        rows.push(threshold_row(r, &meas, &fix, fwd, &rf, fine.as_ref(), tau_ref));
    }
    let live: Vec<&RowOk> = rows.iter().filter_map(ThresholdRow::live).collect();
    let pairs: Vec<(f64, f64)> = live.iter().map(|x| (x.r, x.tau_star)).collect();
    let rs_where = |f: &dyn Fn(&RowOk) -> bool| -> Vec<f64> {
        live.iter().filter(|x| f(x)).map(|x| x.r).collect()
    };
    let mut kappa_seen = sorted_f(&rows.iter().flat_map(|x| x.kappa().to_vec()).collect::<Vec<_>>());
    kappa_seen.dedup();
    ThresholdLaw {
        rs: rs.to_vec(),
        tau_gov,
        tau_ref,
        phi_lim,
        phi_air,
        bracket,
        ds,
        ds_fine,
        n_void: rows.len() - live.len(),
        p1_fixed_in: rs_where(&|x| x.fixed_in_bracket),
        p1_fixed_out: rs_where(&|x| !x.fixed_in_bracket),
        p1_fwd_in: rs_where(&|x| x.fwd_in_bracket),
        p1_fwd_out: rs_where(&|x| !x.fwd_in_bracket),
        p2_fixed_early: rs_where(&|x| x.fixed_early),
        p2_fwd_early: rs_where(&|x| x.fwd_early),
        p2_all_early: !live.is_empty() && live.iter().all(|x| x.fixed_early && x.fwd_early),
        p3_fwd_never_better: !live.is_empty() && live.iter().all(|x| match x.err_fwd {
            Some(e) => e >= x.err_fixed,
            None => true, // filtered out by Python's `if … is not None`
        }),
        kappa_seen,
        all_kappa_pure: rows.iter().all(ThresholdRow::kappa_pure),
        ds_stable: rs_where(&|x| x.ds_stable == Some(true)),
        ds_unstable: rs_where(&|x| x.ds_stable == Some(false)),
        ds_unrun: rs_where(&|x| x.ds_stable.is_none()),
        monotone_in_r: pairs.windows(2).all(|w| w[0].1 < w[1].1),
        thresholds: pairs,
        rows,
    }
}

// ---------------------------------------------------------------------------------------------
// § 3a: THE DISCRIMINATOR — IS THE FORWARD READING's SIGN THE REFERENCE's OR THE RAMP's?
// ---------------------------------------------------------------------------------------------

/// One reference of § 3a — Python's row dict, fields in its key order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RefRow {
    pub tau_ref: f64,
    pub fwd: Option<f64>,
    pub ref_above: bool,
    /// The SIGNED error: the forward reading sits BELOW the measured threshold.
    pub early: bool,
    pub err: Option<f64>,
    pub dist: f64,
    pub s_bind: Option<f64>,
    pub n_riding4: usize,
    pub n_fuel: usize,
    pub window_open: bool,
    pub riding4_valid: bool,
    pub kappa_pure: bool,
}

/// § 3a's full reading, fields in Python's key order (`void=None` implied after `rows`).
#[derive(Clone, Debug, PartialEq)]
pub struct RefFull {
    pub r: f64,
    pub tau_star: f64,
    /// Python's `[b["lo"], b["hi"]]` — a LIST.
    pub bracket: [f64; 2],
    pub width: f64,
    pub tau_refs: Vec<f64>,
    pub rows: Vec<RefRow>,
    pub n_live: usize,
    /// THE DISCRIMINATOR — an ALL, never a count.
    pub sign_follows_reference: bool,
    pub n_agree: usize,
    pub crossing: Vec<(f64, bool)>,
    pub grows_above: Option<bool>,
    pub grows_below: Option<bool>,
    pub s_binds: Vec<(f64, Option<f64>)>,
}

/// `threshold_reference`'s two shapes. The void one is `r, void, rows=[]`.
#[derive(Clone, Debug, PartialEq)]
pub enum ThresholdReference {
    Void { r: f64, void: String },
    Full(RefFull),
}

/// § 3a (`engine.py:22438`) — **at ONE ramp, does the forward reading's SIGN follow its own
/// reference's side of the threshold?** The ramp is held and the reference swept across the
/// measured `tau*`; P3's growth clause is scored per SIDE, sorted stably by distance.
#[allow(clippy::too_many_arguments)]
pub fn threshold_reference(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, phi_air: Option<f64>, r: f64, tau_refs: &[f64], tau_gov: f64, tau_q: f64,
    tau_s: f64, bracket: (f64, f64), n_bisect: usize, s_settle: f64, ds: f64, v_max: f64,
    inc: bool,
) -> ThresholdReference {
    let base = ScanKw {
        flight, tt4_lo, tt4_hi, tt4_max, phi_lim, phi_air, tau_gov, tau_q, tau_s, r, s_settle,
        ds, v_max, inc,
    };
    let b = bisect(core, key_fuel, bracket.0, bracket.1, n_bisect, &base);
    if let Some(v) = b.void() {
        return ThresholdReference::Void { r, void: v.to_string() };
    }
    let b = b.ok();
    let tstar = b.mid;
    let rows: Vec<RefRow> = tau_refs.iter().map(|&tr| {
        let s = threshold_law::scan(core, tr, &base);
        let fwd = forward(&s);
        RefRow {
            tau_ref: tr,
            fwd,
            ref_above: tr > tstar,
            early: fwd.is_some_and(|w| w < tstar),
            err: fwd.map(|w| py_div((w - tstar).abs(), tstar)),
            dist: py_div((tr - tstar).abs(), tstar),
            s_bind: s.s_bind,
            n_riding4: s.n_riding4,
            n_fuel: s.n_fuel,
            window_open: s.window_open,
            riding4_valid: s.riding4_valid,
            kappa_pure: s.kappa_pure,
        }
    }).collect();
    let live: Vec<&RefRow> = rows.iter().filter(|x| x.fwd.is_some() && x.window_open).collect();
    let n_agree = live.iter().filter(|x| x.early == x.ref_above).count();
    let grows = |side: bool| -> Option<bool> {
        let mut xs: Vec<&&RefRow> = live.iter().filter(|x| x.ref_above == side).collect();
        xs.sort_by(|a, b| a.dist.partial_cmp(&b.dist).expect("rung-82: a NaN distance"));
        if xs.len() < 2 {
            return None;
        }
        let err = |x: &RefRow| x.err.expect("a live row has a forward reading, so an error");
        Some(xs.windows(2).all(|w| err(w[0]) <= err(w[1])))
    };
    ThresholdReference::Full(RefFull {
        r,
        tau_star: tstar,
        bracket: [b.lo, b.hi],
        width: b.width,
        tau_refs: tau_refs.to_vec(),
        n_live: live.len(),
        sign_follows_reference: !live.is_empty() && n_agree == live.len(),
        n_agree,
        crossing: live.iter().map(|x| (x.tau_ref, x.early)).collect(),
        grows_above: grows(true),
        grows_below: grows(false),
        s_binds: live.iter().map(|x| (x.tau_ref, x.s_bind)).collect(),
        rows,
    })
}

// ---------------------------------------------------------------------------------------------
// §§ 4–5: THE TWO TERMS, SEPARATELY
// ---------------------------------------------------------------------------------------------

/// One bisected `(phi_lim, tau_gov)` of § 4–5 — Python's ok `at` dict, fields in its key order
/// (`ok=True`, `void=None` implied after `tau_gov`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtOk {
    pub phi_lim: f64,
    pub tau_gov: f64,
    pub tau_star: f64,
    pub width: f64,
    pub gap_bind: Option<f64>,
    pub ratio_bind: Option<f64>,
    pub s_bind: Option<f64>,
    pub n_riding4: usize,
    pub n_fuel: usize,
    pub riding4_valid: bool,
    pub slope_f: Option<f64>,
    pub slope_r: Option<f64>,
    pub kappa: Option<f64>,
}

/// `threshold_terms`' `at` shapes. The void one is `phi_lim, tau_gov, ok=False, void`.
#[derive(Clone, Debug, PartialEq)]
pub enum TermsAt {
    Void { phi_lim: f64, tau_gov: f64, void: String },
    Ok(AtOk),
}

impl TermsAt {
    fn live(&self) -> Option<&AtOk> {
        match self {
            TermsAt::Ok(x) => Some(x),
            TermsAt::Void { .. } => None,
        }
    }
}

/// One sub-interval secant of P4.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Secant {
    pub lo: f64,
    pub hi: f64,
    pub measured: f64,
    pub predicted: Option<f64>,
}

/// P4, the LAG term — Python's dict, fields in its key order.
#[derive(Clone, Debug, PartialEq)]
pub struct P4 {
    pub measured: f64,
    pub predicted: Option<f64>,
    pub rises: bool,
    pub rel_err: Option<f64>,
    pub transfer: Option<f64>,
    pub secants: Vec<Secant>,
    pub kappa: Vec<Option<f64>>,
    pub ratio_bind: Vec<Option<f64>>,
    pub taus: Vec<(f64, f64)>,
}

/// P5 / V7, the SET-POINT term — Python's dict, fields in its key order.
#[derive(Clone, Debug, PartialEq)]
pub struct P5 {
    pub d_gap: Option<f64>,
    pub d_lag: Option<f64>,
    pub d_slope_f: Option<f64>,
    pub d_ratio: Option<f64>,
    pub rises: bool,
    pub separates: bool,
    pub v7_withdrawn: bool,
    pub gap_falls: bool,
    pub ratio_rises: bool,
    pub monotone: bool,
    pub taus: Vec<(f64, f64)>,
    pub gaps: Vec<Option<f64>>,
    pub ratios: Vec<Option<f64>>,
    pub slope_f: Vec<Option<f64>>,
    pub slope_r: Vec<Option<f64>>,
}

/// §§ 4–5's reading — Python's `threshold_terms` return dict, fields in its key order.
#[derive(Clone, Debug, PartialEq)]
pub struct ThresholdTerms {
    pub r: f64,
    pub phi_lim: f64,
    /// As the CALLER passed them — the walls are iterated SORTED, the field is not.
    pub phi_lims: Vec<f64>,
    pub phi_air: Option<f64>,
    pub ds: f64,
    pub tau_govs: Vec<f64>,
    pub bracket: (f64, f64),
    pub govs: Vec<TermsAt>,
    pub walls: Vec<TermsAt>,
    pub n_void: usize,
    pub p4: Option<P4>,
    pub p5: Option<P5>,
}

/// `abs(b - a) / abs(a) if a else None` — P5's fractional move, TRUTHINESS on `a` (a `0.0` is
/// absent), and a `None` on `b` is Python's `TypeError`.
fn frac_move(a: Option<f64>, b: Option<f64>, what: &str) -> Option<f64> {
    if truthy(a) {
        let a = a.expect("truthy");
        Some((need(b, what) - a).abs() / a.abs())
    } else {
        None
    }
}

/// `1.0 / (kappa * ratio_bind) if ratio_bind and kappa else None` — the frozen-trajectory
/// coefficient in the SWEPT knob.
fn frozen_coef(x: &AtOk) -> Option<f64> {
    if truthy(x.ratio_bind) && truthy(x.kappa) {
        Some(py_div(1.0, x.kappa.expect("truthy") * x.ratio_bind.expect("truthy")))
    } else {
        None
    }
}

/// §§ 4–5 (`engine.py:22511`) — **do `tau_gov` and the WALL PAIR reach the threshold through the
/// two DIFFERENT terms the criterion says they do?** The governor clocks run at the caller's
/// `phi_lim`; the walls run SORTED at a hard-coded `tau_gov = 0.05`. P4 is read off the first and
/// last LIVE governor rows, P5 off the first and last live walls.
#[allow(clippy::too_many_arguments)]
pub fn threshold_terms(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, phi_air: Option<f64>, phi_lims: &[f64], r: f64, tau_govs: &[f64], tau_q: f64,
    tau_s: f64, bracket: (f64, f64), n_bisect: usize, s_settle: f64, ds: f64, v_max: f64,
    inc: bool,
) -> ThresholdTerms {
    let (lo, hi) = bracket;
    let at = |pl: f64, tg: f64| -> TermsAt {
        let kw = ScanKw {
            flight, tt4_lo, tt4_hi, tt4_max, phi_lim: pl, phi_air, tau_gov: tg, tau_q, tau_s, r,
            s_settle, ds, v_max, inc,
        };
        let b = bisect(core, key_fuel, lo, hi, n_bisect, &kw);
        if let Some(v) = b.void() {
            return TermsAt::Void { phi_lim: pl, tau_gov: tg, void: v.to_string() };
        }
        let b = b.ok();
        let s = &b.at_hi;
        TermsAt::Ok(AtOk {
            phi_lim: pl,
            tau_gov: tg,
            tau_star: b.mid,
            width: b.width,
            gap_bind: s.gap_bind,
            ratio_bind: s.ratio_bind,
            s_bind: s.s_bind,
            n_riding4: s.n_riding4,
            n_fuel: s.n_fuel,
            riding4_valid: s.riding4_valid,
            slope_f: s.slope_f_bind,
            slope_r: s.slope_r_bind,
            kappa: kappa0(s),
        })
    };
    let govs: Vec<TermsAt> = tau_govs.iter().map(|&tg| at(phi_lim, tg)).collect();
    let walls: Vec<TermsAt> = sorted_f(phi_lims).iter().map(|&pl| at(pl, 0.05)).collect();
    let gl: Vec<&AtOk> = govs.iter().filter_map(TermsAt::live).collect();
    let wl: Vec<&AtOk> = walls.iter().filter_map(TermsAt::live).collect();
    let secant = |u: &AtOk, v: &AtOk| py_div(v.tau_star - u.tau_star, v.tau_gov - u.tau_gov);

    let p4 = if gl.len() >= 2 {
        let (a, b) = (gl[0], gl[gl.len() - 1]);
        let slope = secant(a, b);
        let pred = frozen_coef(b);
        Some(P4 {
            measured: slope,
            predicted: pred,
            rises: slope > 0.0,
            rel_err: pred.filter(|&p| p != 0.0).map(|p| (slope - p).abs() / p.abs()),
            transfer: pred.filter(|&p| p != 0.0).map(|p| slope / p),
            secants: gl.windows(2).map(|w| Secant {
                lo: w[0].tau_gov,
                hi: w[1].tau_gov,
                measured: secant(w[0], w[1]),
                predicted: frozen_coef(w[1]),
            }).collect(),
            kappa: gl.iter().map(|x| x.kappa).collect(),
            ratio_bind: gl.iter().map(|x| x.ratio_bind).collect(),
            taus: gl.iter().map(|x| (x.tau_gov, x.tau_star)).collect(),
        })
    } else {
        None
    };

    let p5 = if wl.len() >= 2 {
        let (a, b) = (wl[0], wl[wl.len() - 1]);
        let dg = frac_move(a.gap_bind, b.gap_bind, "gap_bind");
        let dr = frac_move(a.slope_r, b.slope_r, "slope_r");
        let both = |f: fn(f64, f64) -> bool| match (dg, dr) {
            (Some(g), Some(r)) => f(g, r),
            _ => false,
        };
        Some(P5 {
            d_gap: dg,
            d_lag: dr,
            d_slope_f: frac_move(a.slope_f, b.slope_f, "slope_f"),
            d_ratio: frac_move(a.ratio_bind, b.ratio_bind, "ratio_bind"),
            rises: b.tau_star > a.tau_star,
            separates: both(|g, r| r < g / 3.0),
            v7_withdrawn: both(|g, r| r >= g / 3.0),
            // `is not None` here, NOT truthiness — and a `None` on `b` is `TypeError`
            gap_falls: a.gap_bind.is_some_and(|ga| need(b.gap_bind, "gap_bind") < ga),
            ratio_rises: a.ratio_bind.is_some_and(|ra| need(b.ratio_bind, "ratio_bind") > ra),
            monotone: wl.windows(2).all(|w| w[0].tau_star > w[1].tau_star),
            taus: wl.iter().map(|x| (x.phi_lim, x.tau_star)).collect(),
            gaps: wl.iter().map(|x| x.gap_bind).collect(),
            ratios: wl.iter().map(|x| x.ratio_bind).collect(),
            slope_f: wl.iter().map(|x| x.slope_f).collect(),
            slope_r: wl.iter().map(|x| x.slope_r).collect(),
        })
    } else {
        None
    };

    ThresholdTerms {
        r,
        phi_lim,
        phi_lims: phi_lims.to_vec(),
        phi_air,
        ds,
        tau_govs: tau_govs.to_vec(),
        bracket,
        n_void: govs.iter().chain(&walls).filter(|x| x.live().is_none()).count(),
        govs,
        walls,
        p4,
        p5,
    }
}
