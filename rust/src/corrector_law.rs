//! RUNG 83 — **THE CORRECTOR'S OWN BAR.** `CorrectorLawTransient` (`engine.py:22645`), slice AJ.
//!
//! Rung 82 asked whether ONE Newton step off its residual `h` reaches what thirteen bisection
//! marches reach. The answer is no, and the reason is not accuracy. Headline: **a bracketing
//! solve locates a SIGN CHANGE; a corrector needs a ROOT, and on a residual built as a `min` those
//! are different objects** — `h` jumps at every argmin handover, bisection reads only `sign(h)`,
//! and at `r = 0.25` on the shipped grid there is no root at all. Rung 78 found a residual's
//! slope is a gauge and its root's UNIQUENESS is not; this rung finds its root's EXISTENCE is not
//! either.
//!
//! # NO TABLE, NO BUILDER — A RUNG-80 CORE PLUS THE FOUR FREE FUNCTIONS BELOW
//!
//! Reader-only, as [`authority_clock`](crate::authority_clock) says for all four rungs: no `R83`
//! table, no builder. The four methods are [`corrector_read`] (`engine.py:22672`),
//! [`corrector_step`] (`engine.py:22704`), [`residual_shape`] (`engine.py:22727`) and
//! [`corrector_secant`] (`engine.py:22761`). Every march is rung 82's `_scan`, called as
//! `threshold_law::scan`. Gated bit for bit, every returned value in Python's own key order, in
//! `tests/slice_aj_corrector.rs` against `oracle/probe_slice_aj_step4.py`'s output.
//!
//! # WHAT THE PORT HONOURS
//!
//! * **`F` and `g` are RUNG 83's arithmetic, not rung 84's.** Here `F = tau_hat_min/kappa` and
//!   `g = F - tau`; [`staircase_law::edge_read`](crate::staircase_law::edge_read) computes
//!   `g = h/kappa` and `F = h/kappa + tau`. The two agree in algebra and NOT in the last bit —
//!   measured on PyPy off this step's own oracle, where both rungs read the same `tau`: `g`
//!   differs at both of rung 83 § 3.2's jump `tau`s, `F` at § 3.4's `0.037333`, and both at
//!   `read_ctl` (`r = 0.5`, `tau = 0.08`) — so they share no code.
//! * **`exact` is a BIT comparison** — `tau_hat_min - kappa*tau_f == h`, spelled as Python spells
//!   it. Its whole point is that it holds at zero tolerance (rung 83 § 1.1).
//! * **Truthiness is ported as truthiness** — `if kap` in [`corrector_read`], `if a["F"]` for
//!   `jump_in_F` — through [`truthy`](crate::threshold_law) (a `0.0` is absent).
//! * **Two dict SHAPES wherever Python returns two**: [`CorrectorStep`] (the void one's key order
//!   is `tau_hat, c, forward, correction, void`; the ok one's `tau_hat, c, void, forward,
//!   correction`).
//! * **`residual_shape`'s ladder is `lo + (hi - lo)·i/(n - 1.0)`**, left to right as Python
//!   evaluates it; `min(abs(g_lo), abs(g_hi))` is Python's two-argument `min`
//!   ([`py_min2`](crate::state_coordinate)).
//! * **`corrector_secant`'s S2 message** (`engine.py:22784`) is built with `%g` →
//!   [`py_g`](crate::demand_coordinate::py_g). The default `flat = 1e-12` already prints
//!   differently (`1e-12` vs Rust's `0.000000000001`).
//! * **Every void is a RETURNED string, byte for byte** (§ 5.34 (vi)) — including
//!   `V4: kappa impure` and `V4: kappa impure at a start point`, which at `r = 1.0` MISNAME their
//!   cause: there is no four-loop window, so κ is EMPTY, not impure. Reproduced word for word; the
//!   misnaming is recorded in the plan, not repaired here. **`V4: kappa impure at an iterate`**,
//!   which the pre-flight never drove, IS reachable (measured on PyPy): a secant started at
//!   `(0.30, 0.29)` on `r = 1.0` — where the `0.30` window is open (rung 82's step-3 finding) —
//!   steps its first iterate into an empty window.
//! * **Every refusal is Python's exception** — a zero `step` (`n = 1`, or `lo == hi` with a sign
//!   change) is `ZeroDivisionError` ([`py_div`](crate::threshold_law)).

use crate::demand_coordinate::py_g;
use crate::state_coordinate::py_min2;
use crate::stator_transient::ScheduledStatorCore;
use crate::threshold_law::{self, kappa0, py_div, truthy, ScanKw, ThresholdScan};

/// `corrector_secant`'s default `bracket`, rung 82's own.
pub const BRACKET: (f64, f64) = (0.004, 0.30);
/// `corrector_secant`'s default `cap`.
pub const CAP: usize = 6;
/// `corrector_secant`'s default `flat`.
pub const FLAT: f64 = 1e-12;
/// `residual_shape`'s default `n`.
pub const SHAPE_N: usize = 21;

// ---------------------------------------------------------------------------------------------
// § 1: THE IDENTITY, AND THE FORWARD READING OFF ONE MARCH
// ---------------------------------------------------------------------------------------------

/// ONE march, reduced to everything a corrector could read from it — Python's `corrector_read`
/// dict, fields in its key order.
#[derive(Clone, Debug, PartialEq)]
pub struct CorrectorRead {
    pub tau_f: f64,
    /// Rung 82's FORWARD reading, `tau_hat_min / kappa`.
    pub f: Option<f64>,
    /// `F - tau_f`.
    pub g: Option<f64>,
    pub h: Option<f64>,
    pub kappa: Option<f64>,
    pub kappa_pure: bool,
    /// The identity `h == tau_hat_min - kappa*tau_f`, at ZERO tolerance.
    pub exact: bool,
    pub identity_pred: Option<f64>,
    /// § 2: THE SIDE, from this single march — `h > 0` ⇔ BELOW the root.
    pub below_root: Option<bool>,
    pub s_bind: Option<f64>,
    pub n_fuel: usize,
    pub n_scored: usize,
    pub window_open: bool,
    pub riding4_valid: bool,
    pub tau_hat_min: Option<f64>,
    pub scan: ThresholdScan,
}

/// RUNG 83's `corrector_read` (`engine.py:22672`) — **one `_scan`, and the identity checked on
/// the shipped dict at zero tolerance.**
pub fn corrector_read(core: &ScheduledStatorCore, tau_f: f64, kw: &ScanKw) -> CorrectorRead {
    let s = threshold_law::scan(core, tau_f, kw);
    // `s["kappa"][0] if s["kappa_pure"] and s["kappa"] else None` — pure means one element
    let kap = kappa0(&s);
    let (f, pred) = match (truthy(kap), s.tau_hat_min) {
        (true, Some(t)) => {
            let k = kap.expect("truthy");
            (Some(t / k), Some(t - k * tau_f))
        }
        _ => (None, None),
    };
    CorrectorRead {
        tau_f,
        f,
        g: f.map(|f| f - tau_f),
        h: s.h,
        kappa: kap,
        kappa_pure: s.kappa_pure,
        exact: matches!((pred, s.h), (Some(p), Some(h)) if p == h),
        identity_pred: pred,
        below_root: s.h.map(|h| h > 0.0),
        s_bind: s.s_bind,
        n_fuel: s.n_fuel,
        n_scored: s.n_scored,
        window_open: s.window_open,
        riding4_valid: s.riding4_valid,
        tau_hat_min: s.tau_hat_min,
        scan: s,
    }
}

/// `corrector_step`'s two shapes, each in its own key order.
#[derive(Clone, Debug, PartialEq)]
pub enum CorrectorStep {
    /// `tau_hat=None, c, forward, correction=None, void` — V4 (no forward reading) or V5.
    Void { c: f64, forward: Option<f64>, void: String },
    /// `tau_hat, c, void=None, forward, correction`.
    Ok { tau_hat: f64, c: f64, forward: f64, correction: f64 },
}

/// RUNG 83's `corrector_step` (`engine.py:22704`) — **the single Newton step in closed form**,
/// `tau_1 = tau_0 + (F - tau_0)/(1 - c)`; `kappa` cancels. `c` is SUPPLIED; `|1 - c| < 1e-12` is
/// V5 and returns no number, never a clipped one.
pub fn corrector_step(read: &CorrectorRead, c: f64) -> CorrectorStep {
    // `F is None or abs(1.0 - c) < 1e-12` voids — written as Python's `<`, negated, so a NaN `c`
    // takes the ok branch as it does there (a `>=` would void it)
    match read.f {
        Some(f) if !((1.0 - c).abs() < 1e-12) => {
            let t0 = read.tau_f;
            CorrectorStep::Ok {
                tau_hat: t0 + (f - t0) / (1.0 - c),
                c,
                forward: f,
                correction: (f - t0) * c / (1.0 - c),
            }
        }
        _ => CorrectorStep::Void {
            c,
            forward: read.f,
            void: if read.f.is_none() { "V4: kappa impure" } else { "V5: |1-c| below 1e-12" }
                .to_string(),
        },
    }
}

// ---------------------------------------------------------------------------------------------
// § 3: THE SHAPE — IS THE SIGN CHANGE A ROOT, OR A JUMP?
// ---------------------------------------------------------------------------------------------

/// One sign change of `g` on the ladder — Python's dict, fields in its key order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Change {
    pub tau_lo: f64,
    pub tau_hi: f64,
    pub g_lo: f64,
    pub g_hi: f64,
    pub s_lo: Option<f64>,
    pub s_hi: Option<f64>,
    pub argmin_moved: bool,
    pub smallest_g: f64,
    pub step: f64,
    /// `smallest_g / step` — REPORTED, never thresholded into a verdict.
    pub ratio: f64,
    /// `(F_hi - F_lo)/F_lo`, or `None` where `F_lo` is falsy.
    pub jump_in_f: Option<f64>,
}

/// § 3's reading — Python's `residual_shape` dict, fields in its key order.
#[derive(Clone, Debug, PartialEq)]
pub struct ResidualShape {
    pub lo: f64,
    pub hi: f64,
    pub n: usize,
    pub step: f64,
    pub points: Vec<CorrectorRead>,
    pub changes: Vec<Change>,
    pub n_changes: usize,
    pub n_kappa_impure: usize,
    /// A handover ANYWHERE on the ladder, sign change or not.
    pub n_argmin_switches: usize,
}

/// RUNG 83's `residual_shape` (`engine.py:22727`) — **resolve `g = F - tau` on a ladder and
/// classify EVERY sign change it contains** by the ratio `min(|g|)/step`, reported and never
/// thresholded.
pub fn residual_shape(
    core: &ScheduledStatorCore, lo: f64, hi: f64, n: usize, kw: &ScanKw,
) -> ResidualShape {
    let nm1 = n as f64 - 1.0;
    let step = py_div(hi - lo, nm1);
    let pts: Vec<CorrectorRead> = (0..n)
        .map(|i| corrector_read(core, lo + py_div((hi - lo) * i as f64, nm1), kw))
        .collect();
    let mut changes = Vec::new();
    for w in pts.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        let (Some(ga), Some(gb)) = (a.g, b.g) else { continue };
        if (ga > 0.0) == (gb > 0.0) {
            continue;
        }
        let small = py_min2(ga.abs(), gb.abs());
        changes.push(Change {
            tau_lo: a.tau_f,
            tau_hi: b.tau_f,
            g_lo: ga,
            g_hi: gb,
            s_lo: a.s_bind,
            s_hi: b.s_bind,
            argmin_moved: a.s_bind != b.s_bind,
            smallest_g: small,
            step,
            ratio: py_div(small, step),
            jump_in_f: if truthy(a.f) {
                let (fa, fb) = (a.f.expect("truthy"), need_f(b.f));
                Some((fb - fa) / fa)
            } else {
                None
            },
        });
    }
    ResidualShape {
        lo,
        hi,
        n,
        step,
        n_changes: changes.len(),
        n_kappa_impure: pts.iter().filter(|p| !p.kappa_pure).count(),
        n_argmin_switches: pts.windows(2).filter(|w| w[0].s_bind != w[1].s_bind).count(),
        points: pts,
        changes,
    }
}

/// `b["F"]` where Python subtracts it — a `g` implies an `F`, so `None` here is Python's
/// `TypeError`, unreachable on a sign change (both `g`s exist).
fn need_f(x: Option<f64>) -> f64 {
    x.expect("rung-83: `F` is None where Python subtracts it -- Python raises TypeError here")
}

// ---------------------------------------------------------------------------------------------
// § 4: THE ITERATED SECANT, UNDER A START RULE THE CALLER MUST NAME
// ---------------------------------------------------------------------------------------------

/// One secant iterate — Python's trace dict, fields in its key order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SecantStep {
    pub tau: f64,
    pub f: f64,
    pub g: f64,
    pub s_bind: Option<f64>,
    /// S1: clamped to the bracket — COUNTED, never silent.
    pub clamped: bool,
}

/// § 4's reading — Python's `corrector_secant` dict, fields in its key order.
#[derive(Clone, Debug, PartialEq)]
pub struct CorrectorSecant {
    pub t0: f64,
    pub t1: f64,
    pub cap: usize,
    pub trace: Vec<SecantStep>,
    pub clamps: usize,
    pub abort: Option<String>,
    pub marches: usize,
    pub tau: Option<f64>,
    pub final_g: Option<f64>,
    /// `(len(trace), 13)` — against the 13 marches a `_bisect(n=10)` costs.
    pub marches_vs_bisect: (usize, usize),
    pub converged: bool,
}

/// RUNG 83's `corrector_secant` (`engine.py:22761`) — **the secant on `g = F - tau`, the two
/// most recent iterates, and the COST reported.** The starts are the caller's; clamps to the
/// bracket are COUNTED (S1); a flat pair ABORTS (S2) rather than being nudged.
pub fn corrector_secant(
    core: &ScheduledStatorCore, t0: f64, t1: f64, cap: usize, bracket: (f64, f64), flat: f64,
    kw: &ScanKw,
) -> CorrectorSecant {
    let (blo, bhi) = bracket;
    let (mut trace, mut pts, mut clamps, mut abort) =
        (Vec::<SecantStep>::new(), Vec::<(f64, f64)>::new(), 0usize, None::<String>);
    for t in [t0, t1] {
        let rd = corrector_read(core, t, kw);
        let Some(g) = rd.g else {
            abort = Some("V4: kappa impure at a start point".into());
            break;
        };
        pts.push((t, g));
        trace.push(SecantStep {
            tau: t, f: need_f(rd.f), g, s_bind: rd.s_bind, clamped: false,
        });
    }
    if abort.is_none() {
        for _ in 0..cap {
            let ((ta, ga), (tb, gb)) = (pts[pts.len() - 2], pts[pts.len() - 1]);
            if (gb - ga).abs() < flat {
                // `%g`, never `{}`: the default `1e-12` prints `0.000000000001` through Display
                abort = Some(format!("S2: flat pair, |dg| < {}", py_g(flat)));
                break;
            }
            let mut tn = tb - gb * (tb - ta) / (gb - ga);
            let mut cl = false;
            if tn < blo {
                (tn, cl) = (blo, true);
            } else if tn > bhi {
                (tn, cl) = (bhi, true);
            }
            clamps += cl as usize;
            let rd = corrector_read(core, tn, kw);
            let Some(g) = rd.g else {
                abort = Some("V4: kappa impure at an iterate".into());
                break;
            };
            pts.push((tn, g));
            trace.push(SecantStep { tau: tn, f: need_f(rd.f), g, s_bind: rd.s_bind, clamped: cl });
        }
    }
    let last = trace.last();
    CorrectorSecant {
        t0,
        t1,
        cap,
        clamps,
        abort,
        marches: trace.len(),
        tau: last.map(|x| x.tau),
        final_g: last.map(|x| x.g.abs()),
        marches_vs_bisect: (trace.len(), 13),
        // `bool(trace) and abs(g) < 1e-12 and not clamps`
        converged: last.is_some_and(|x| x.g.abs() < 1e-12) && clamps == 0,
        trace,
    }
}
