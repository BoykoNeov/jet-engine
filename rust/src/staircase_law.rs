//! RUNG 84 — **THE MARCHED MINIMUM'S STAIRCASE.** `StaircaseLawTransient` (`engine.py:22809`),
//! slice AJ.
//!
//! Rung 83 asked which ramps have roots and at which `ds`. Headline: **a minimum over a MARCHED
//! set is a reading on a MOVING GRID BOUNDARY, so the residual carries the march's own
//! sawtooth.** A `min` over a fixed set of continuous functions can kink but not jump; rung 83's
//! jump is the four-loop window opening one march step earlier, and the entering point binds at
//! once. So `h(tau; ds) = h_true(tau) + hat'·delta(tau, ds)` with `delta` in `[0, ds)`, and a
//! missing root is a crossing that lands on a step. Rise and tread share the factor `ds`, so the
//! staircase number `Lambda = rise/tread` has no `ds` in it.
//!
//! # NO TABLE, NO BUILDER — A RUNG-80 CORE PLUS THE SIX FREE FUNCTIONS BELOW
//!
//! Reader-only, as [`authority_clock`](crate::authority_clock) says for all four rungs: no `R84`
//! table, no builder. The six methods are [`edge_read`] (`engine.py:22861`), [`classify`]
//! (`engine.py:22903`), [`staircase_scan`] (`engine.py:22939`), [`lattice_count`]
//! (`engine.py:22978`), [`staircase_number`] (`engine.py:23005`) and [`root_class`]
//! (`engine.py:23044`). Every march is rung 82's, through
//! [`scan_cells`](crate::threshold_law::scan_cells) + [`hats`](crate::threshold_law::hats) — NOT
//! through `threshold_scan`, which reduces the point identities this rung exists to read to
//! counts. Gated bit for bit, every returned value in Python's own key order, in
//! `tests/slice_aj_staircase.rs` against `oracle/probe_slice_aj_step4.py`'s output.
//!
//! # WHAT THE PORT HONOURS
//!
//! * **`classify` is named in THIS module and called fully qualified** — `fuel_transient` already
//!   has a `classify`. No bare `use` of either.
//! * **`F` and `g` are RUNG 84's arithmetic** — `g = h/kappa`, `F = h/kappa + tau` — not rung
//!   83's `F = tau_hat_min/kappa` (see [`corrector_law`](crate::corrector_law)); they differ in
//!   the last bit at points measured on PyPy (`g` at both of rung 83 § 3.2's jump `tau`s).
//! * **`summands` is Python's dict comprehension** `{round(s, 9): hat - te …}`
//!   (`engine.py:22880`): an insertion-ordered `Vec` of pairs, a repeated key keeping its FIRST
//!   position and its LAST value ([`dict_put`](crate::authority_clock)). Keys compare with
//!   Python's `==` — as its dicts and sets do — never by bits; the two differ only at `±0.0` and
//!   NaN, as [`round10`](crate::full_split::round10)'s doc records.
//! * **`round(s, 9)` merges nothing within a march** — `engine.py:22879`'s claim, *"`ds >=
//!   6.25e-4` here"*; the smallest `ds` any reading here passes is `0.0025`. **The pre-flight's cross-`ds` merge (339 → 195) is NOT a reader's**: every shipped
//!   `classify` call — `staircase_scan`, `staircase_number`, `root_class`, and both of
//!   `test_rung84.py`'s — compares two reads taken with the SAME `kw`, so the SAME `ds`. The count
//!   was a property of the pre-flight's own census; nothing here reads keys across steps.
//! * **`n_scored` is `len(summands)`, NOT the cell count**, and **`edge` is the FIRST RIDING
//!   point, trajectory ends included** (`engine.py:22881`), not the first scored cell. On this rig the pairs agree
//!   (`n_slope_excluded = 0`, no riding point at an end), so no reading can see a port that
//!   confuses them: recorded as coverage gaps, not identities (plan § 5.34.4).
//! * **`min(summ, key=summ.get)`** (`engine.py:22883`) is the FIRST minimum in insertion order — a
//!   strict-`<` loop.
//! * **`int(round(edge / ds))`** (`engine.py:22898`) is 1-argument `round`: ties-to-even,
//!   returning an int → `round_ties_even` and a cast. No exact `.5` was measured; the nearest
//!   value was `309.99999999999994`. `edge_index`, `n_jumps` are SIGNED.
//! * **`round(c.tau_f/tau_f, 6)`** (`engine.py:22876`) as at rung 82, over every scored cell.
//! * **Two `%g` messages** (`engine.py:23030`, `engine.py:23061`) →
//!   [`py_g`](crate::demand_coordinate::py_g), including `root_class`'s default `eps = 1e-7`,
//!   which Python prints `1e-07`.
//! * **Truthiness is ported as truthiness** — `if spacing`, `if tread`, `bi.get("void")`: a
//!   `spacing = 0.0` returns NO tread, exactly as `None` does, but reports `spacing = 0.0`.
//! * **`lattice_count`'s `V3: an edge off the march grid`** MISNAMES its cause at `r = 1.0` (no
//!   window, so no edge exists to be off the grid). Reproduced word for word, as rung 83's V4.
//! * **Four functions, several SHAPES each** — [`StaircaseNumber`] (V2 5 keys, V5 6, ok 15),
//!   [`RootClass`] (void 5, ok 21); `lattice_count`'s `void` is its LAST key.
//! * **`classify`'s `min` over the common set** iterates a Python SET, whose order is not
//!   insertion order; the minimum VALUE is order-free except between `0.0` and `-0.0`, i.e. a
//!   summand exactly zero, which no reading reaches.

use crate::authority_clock::dict_put;
use crate::demand_coordinate::py_g;
use crate::stator_transient::ScheduledStatorCore;
use crate::threshold_law::{self, key_residual, py_div, sorted_f, truthy, Bisect, ScanKw};
use crate::two_lag::{py_max_of, py_min_of};
use crate::two_spool::round6;

/// `root_class`'s default `n_bisect`.
pub const N_BISECT: usize = 10;
/// `root_class`'s default `eps`.
pub const EPS: f64 = 1e-7;
/// `staircase_scan`'s default `n`.
pub const SCAN_N: usize = 17;

/// Python's `round(x, 9)` — format-and-parse, the crate's spelling
/// ([`round6`](crate::two_spool::round6), [`round10`](crate::full_split::round10)).
pub fn round9(x: f64) -> f64 {
    format!("{x:.9}").parse::<f64>().expect("a formatted finite double parses back")
}

/// Python's `x in d` / `set` membership over float keys — `==`, not bits.
fn has(keys: &[(f64, f64)], k: f64) -> bool {
    keys.iter().any(|&(x, _)| x == k)
}

/// Python's `sorted(set)` of float keys — stable, refusing a NaN.
fn sorted_keys(ks: Vec<f64>) -> Vec<f64> {
    sorted_f(&ks)
}

// ---------------------------------------------------------------------------------------------
// § 1: ONE MARCH, AND THE POINT IDENTITIES RUNG 82 REDUCED TO COUNTS
// ---------------------------------------------------------------------------------------------

/// ONE march, with the SET the minimum is taken over — Python's `edge_read` dict, fields in its
/// key order.
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeRead {
    pub tau_f: f64,
    pub r: f64,
    pub ds: f64,
    pub h: Option<f64>,
    pub kappa: Option<f64>,
    pub kappa_pure: bool,
    /// `h/kappa + tau_f` — rung 84's spelling.
    pub f: Option<f64>,
    /// `h/kappa`.
    pub g: Option<f64>,
    /// `round(s, 9) -> hat - tau_eff`, insertion-ordered.
    pub summands: Vec<(f64, f64)>,
    pub s_bind: Option<f64>,
    /// The window's FIRST riding point, `round(s, 9)`.
    pub edge: Option<f64>,
    /// § 1.1: the minimum is attained AT the window's leading point.
    pub at_edge: bool,
    pub n_ride: usize,
    /// `len(summands)`.
    pub n_scored: usize,
    pub n_slope_excluded: usize,
    pub window_open: bool,
    pub riding4_valid: bool,
    /// V3: the edge sits on the march grid.
    pub edge_on_grid: bool,
    pub edge_index: Option<i64>,
}

/// RUNG 84's `edge_read` (`engine.py:22861`) — **one march, returning the set the minimum is
/// taken over, keyed by `s`,** so two reads can be differenced point by point.
pub fn edge_read(core: &ScheduledStatorCore, tau_f: f64, kw: &ScanKw) -> EdgeRead {
    let (traj, ride, cells) = threshold_law::scan_cells(core, tau_f, kw);
    let (hs, n_bad) = threshold_law::hats(&cells, kw.tau_gov);
    let kap = if tau_f > 0.0 {
        let mut v = sorted_f(&cells.iter().map(|c| round6(c.tau_f / tau_f)).collect::<Vec<_>>());
        v.dedup();
        v
    } else {
        Vec::new()
    };
    let pure = kap.len() == 1;
    let mut summ: Vec<(f64, f64)> = Vec::new();
    for x in &hs {
        dict_put(&mut summ, round9(x.s), x.hat - x.tau_eff);
    }
    let edge = ride.first().map(|&i| round9(traj[i].s));
    let vals: Vec<f64> = summ.iter().map(|&(_, v)| v).collect();
    let h = if vals.is_empty() { None } else { Some(py_min_of(&vals)) };
    // `min(summ, key=summ.get)` — the FIRST minimum in insertion order
    let mut sb: Option<(f64, f64)> = None;
    for &(k, v) in &summ {
        if sb.map_or(true, |(_, bv)| v < bv) {
            sb = Some((k, v));
        }
    }
    let sb = sb.map(|(k, _)| k);
    let g = match (pure, h) {
        (true, Some(h)) => Some(py_div(h, kap[0])),
        _ => None,
    };
    let tt4s: Vec<f64> = traj.iter().map(|p| p.tt4).collect();
    let on_grid = |e: f64| {
        let q = py_div(e, kw.ds);
        (q - q.round_ties_even()).abs() < 1e-9
    };
    EdgeRead {
        tau_f,
        r: kw.r,
        ds: kw.ds,
        h,
        kappa: if pure { Some(kap[0]) } else { None },
        kappa_pure: pure,
        // `(h / kap[0] + tau_f)`
        f: g.map(|g| g + tau_f),
        g,
        s_bind: sb,
        edge,
        at_edge: matches!((sb, edge), (Some(a), Some(b)) if a == b),
        n_ride: ride.len(),
        n_scored: summ.len(),
        n_slope_excluded: n_bad,
        window_open: !ride.is_empty(),
        riding4_valid: py_max_of(&tt4s) > kw.tt4_lo * (1.0 + 1e-9),
        edge_on_grid: edge.is_some_and(on_grid),
        edge_index: edge.map(|e| py_div(e, kw.ds).round_ties_even() as i64),
        summands: summ,
    }
}

// ---------------------------------------------------------------------------------------------
// § 2: THE CLASSIFIER, AND IT IS EXACT
// ---------------------------------------------------------------------------------------------

/// A sign change's verdict — Python's `"crossing"` / `"jump"`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// The sign change SURVIVES restriction to the common points — a root.
    Crossing,
    /// It does not — the set changed across zero.
    Jump,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Crossing => "crossing",
            Kind::Jump => "jump",
        }
    }
}

/// `h(b) - h(a)` split into a SMOOTH term and a MEMBERSHIP term — Python's `classify` dict,
/// fields in its key order.
#[derive(Clone, Debug, PartialEq)]
pub struct Classified {
    pub tau_lo: f64,
    pub tau_hi: f64,
    pub entered: Vec<f64>,
    pub left: Vec<f64>,
    pub set_changed: bool,
    pub argmin_moved: bool,
    pub edge_moved: bool,
    pub h_lo: Option<f64>,
    pub h_hi: Option<f64>,
    pub h_common_lo: Option<f64>,
    pub h_common_hi: Option<f64>,
    pub d_full: Option<f64>,
    pub d_smooth: Option<f64>,
    /// EXACTLY zero when the sets agree — an identity, not a bound.
    pub d_membership: Option<f64>,
    pub sign_change: bool,
    pub sign_change_common: bool,
    /// `None` where there is no sign change to classify.
    pub kind: Option<Kind>,
}

/// RUNG 84's `classify` (`engine.py:22903`) — **split `h(b) - h(a)` into a smooth term and a
/// membership term, exactly**; a sign change is a CROSSING iff it survives restriction to the
/// points both marches share. Call it `staircase_law::classify` (module header).
pub fn classify(a: &EdgeRead, b: &EdgeRead) -> Classified {
    let common = |x: &EdgeRead, y: &EdgeRead| -> Option<f64> {
        let v: Vec<f64> =
            x.summands.iter().filter(|&&(k, _)| has(&y.summands, k)).map(|&(_, v)| v).collect();
        if v.is_empty() { None } else { Some(py_min_of(&v)) }
    };
    let (ha, hb) = (common(a, b), common(b, a));
    let only = |x: &EdgeRead, y: &EdgeRead| -> Vec<f64> {
        sorted_keys(x.summands.iter().map(|&(k, _)| k).filter(|&k| !has(&y.summands, k)).collect())
    };
    let (entered, left) = (only(b, a), only(a, b));
    let sign_full = matches!((a.h, b.h), (Some(x), Some(y)) if (x > 0.0) != (y > 0.0));
    let sign_common = matches!((ha, hb), (Some(x), Some(y)) if (x > 0.0) != (y > 0.0));
    let d_full = match (a.h, b.h) {
        (Some(x), Some(y)) => Some(y - x),
        _ => None,
    };
    let d_common = match (ha, hb) {
        (Some(x), Some(y)) => Some(y - x),
        _ => None,
    };
    Classified {
        tau_lo: a.tau_f,
        tau_hi: b.tau_f,
        // `sa != sb` — two sets of keys are equal iff neither has a key the other lacks
        set_changed: !entered.is_empty() || !left.is_empty(),
        entered,
        left,
        argmin_moved: a.s_bind != b.s_bind,
        edge_moved: a.edge != b.edge,
        h_lo: a.h,
        h_hi: b.h,
        h_common_lo: ha,
        h_common_hi: hb,
        d_full,
        d_smooth: d_common,
        d_membership: match (d_full, d_common) {
            (Some(f), Some(c)) => Some(f - c),
            _ => None,
        },
        sign_change: sign_full,
        sign_change_common: sign_common,
        kind: if !sign_full {
            None
        } else if sign_common {
            Some(Kind::Crossing)
        } else {
            Some(Kind::Jump)
        },
    }
}

/// The ladder's census — Python's `staircase_scan` dict, fields in its key order.
#[derive(Clone, Debug, PartialEq)]
pub struct StaircaseScan {
    pub lo: f64,
    pub hi: f64,
    pub n: usize,
    pub points: Vec<EdgeRead>,
    pub pairs: Vec<Classified>,
    pub changes: Vec<Classified>,
    pub n_at_edge: usize,
    pub n_points: usize,
    pub all_at_edge: bool,
    pub n_sign_changes: usize,
    pub n_crossings: usize,
    pub n_jumps: usize,
    pub exact_zero_when_set_equal: bool,
    pub nonzero_when_set_differs: bool,
    pub n_argmin_only: usize,
    pub n_set_only: usize,
    pub n_edge_moves: usize,
    pub all_on_grid: bool,
    /// V4: the licence for [`lattice_count`]'s two-march number.
    pub edge_monotone: bool,
    pub edge_indices: Vec<Option<i64>>,
    pub all_open: bool,
    pub all_kappa_pure: bool,
}

/// RUNG 84's `staircase_scan` (`engine.py:22939`) — **a `tau` ladder, every adjacent pair
/// classified and the edge tracked**; `n_edge_moves` counted the expensive way so the two-march
/// count has something to be checked against.
pub fn staircase_scan(
    core: &ScheduledStatorCore, lo: f64, hi: f64, n: usize, kw: &ScanKw,
) -> StaircaseScan {
    let nm1 = n as f64 - 1.0;
    let pts: Vec<EdgeRead> = (0..n)
        .map(|i| edge_read(core, lo + py_div((hi - lo) * i as f64, nm1), kw))
        .collect();
    let pairs: Vec<Classified> = pts.windows(2).map(|w| classify(&w[0], &w[1])).collect();
    let changes: Vec<Classified> = pairs.iter().filter(|p| p.sign_change).cloned().collect();
    let count = |f: &dyn Fn(&Classified) -> bool| pairs.iter().filter(|p| f(p)).count();
    StaircaseScan {
        lo,
        hi,
        n,
        n_at_edge: pts.iter().filter(|p| p.at_edge).count(),
        n_points: pts.len(),
        all_at_edge: pts.iter().all(|p| p.at_edge),
        n_sign_changes: changes.len(),
        n_crossings: changes.iter().filter(|c| c.kind == Some(Kind::Crossing)).count(),
        n_jumps: changes.iter().filter(|c| c.kind == Some(Kind::Jump)).count(),
        exact_zero_when_set_equal: pairs.iter()
            .filter(|p| !p.set_changed)
            .filter_map(|p| p.d_membership)
            .all(|d| d == 0.0),
        nonzero_when_set_differs: pairs.iter()
            .filter(|p| p.set_changed)
            .filter_map(|p| p.d_membership)
            .all(|d| d != 0.0),
        n_argmin_only: count(&|p| p.argmin_moved && !p.set_changed),
        n_set_only: count(&|p| p.set_changed && !p.argmin_moved),
        n_edge_moves: count(&|p| p.edge_moved),
        all_on_grid: pts.iter().all(|p| p.edge_on_grid),
        edge_monotone: pts.windows(2).all(|w| match (w[0].edge_index, w[1].edge_index) {
            (Some(a), Some(b)) => a >= b,
            _ => true, // filtered out by Python's `if … is not None`
        }),
        edge_indices: pts.iter().map(|p| p.edge_index).collect(),
        all_open: pts.iter().all(|p| p.window_open),
        all_kappa_pure: pts.iter().all(|p| p.kappa_pure),
        points: pts,
        changes,
        pairs,
    }
}

// ---------------------------------------------------------------------------------------------
// § 3: THE COUNT, FROM TWO MARCHES
// ---------------------------------------------------------------------------------------------

/// § 1.2's COUNT — Python's `lattice_count` dict, fields in its key order (`void` LAST).
#[derive(Clone, Debug, PartialEq)]
pub struct LatticeCount {
    pub lo: f64,
    pub hi: f64,
    pub ds: f64,
    pub r: f64,
    pub edge_lo: Option<f64>,
    pub edge_hi: Option<f64>,
    pub index_lo: Option<i64>,
    pub index_hi: Option<i64>,
    pub n_jumps: Option<i64>,
    /// `n_jumps · ds` — the PLANT property the count divides by `ds` to get.
    pub ds_star: Option<f64>,
    pub slope_s_star: Option<f64>,
    pub on_grid: bool,
    pub at_edge: (bool, bool),
    pub void: Option<String>,
}

/// RUNG 84's `lattice_count` (`engine.py:22978`) — **the number of jumps in `[lo, hi]` as an
/// edge-INDEX difference, from TWO marches.** Valid only on a monotone edge (V4), which the
/// caller certifies with [`staircase_scan`].
pub fn lattice_count(core: &ScheduledStatorCore, lo: f64, hi: f64, kw: &ScanKw) -> LatticeCount {
    let (a, b) = (edge_read(core, lo, kw), edge_read(core, hi, kw));
    let ok = a.edge_index.is_some() && b.edge_index.is_some() && a.edge_on_grid && b.edge_on_grid;
    let nj = if ok { Some(a.edge_index.expect("ok") - b.edge_index.expect("ok")) } else { None };
    let ds_star = nj.map(|n| n as f64 * kw.ds);
    LatticeCount {
        lo,
        hi,
        ds: kw.ds,
        r: kw.r,
        edge_lo: a.edge,
        edge_hi: b.edge,
        index_lo: a.edge_index,
        index_hi: b.edge_index,
        n_jumps: nj,
        ds_star,
        slope_s_star: ds_star.map(|d| py_div(d, hi - lo)),
        on_grid: ok,
        at_edge: (a.at_edge, b.at_edge),
        void: if ok { None } else { Some("V3: an edge off the march grid".into()) },
    }
}

// ---------------------------------------------------------------------------------------------
// § 4: THE STAIRCASE NUMBER, AND THE MAP
// ---------------------------------------------------------------------------------------------

/// A live staircase number — Python's ok dict, fields in its key order (`void=None` implied
/// after `r`).
#[derive(Clone, Debug, PartialEq)]
pub struct StaircaseOk {
    pub tau_lo: f64,
    pub tau_hi: f64,
    pub ds: f64,
    pub r: f64,
    /// The JUMP in `g`, membership term alone.
    pub rise: f64,
    /// `|dg/dtau|` on the branch, from this pair.
    pub dg_dtau: f64,
    pub dtau: f64,
    /// The CALLER's lattice spacing — an argument, never derived (rung 84 § 0.2).
    pub spacing: Option<f64>,
    pub tread: Option<f64>,
    pub lam: Option<f64>,
    pub entered: Vec<f64>,
    pub left: Vec<f64>,
    pub d_membership: f64,
    pub d_smooth: f64,
}

/// `staircase_number`'s three shapes, each in its own key order.
#[derive(Clone, Debug, PartialEq)]
pub enum StaircaseNumber {
    /// `void, tau_lo, tau_hi, rise=None, dg_dtau=None`.
    V2 { void: String, tau_lo: f64, tau_hi: f64 },
    /// `void, tau_lo, tau_hi, edge_moved, rise=None, dg_dtau=None`.
    V5 { void: String, tau_lo: f64, tau_hi: f64, edge_moved: bool },
    Ok(StaircaseOk),
}

/// RUNG 84's `staircase_number` (`engine.py:23005`) — **`Lambda = RISE / TREAD` at ONE lattice
/// event.** `tau_lo`/`tau_hi` must bracket exactly one edge move; `spacing = None` (or `0.0` —
/// Python's truthiness) returns the FACTORS and no `lam`.
pub fn staircase_number(
    core: &ScheduledStatorCore, tau_lo: f64, tau_hi: f64, spacing: Option<f64>, kw: &ScanKw,
) -> StaircaseNumber {
    let (a, b) = (edge_read(core, tau_lo, kw), edge_read(core, tau_hi, kw));
    let c = classify(&a, &b);
    if !(a.kappa_pure && b.kappa_pure) {
        return StaircaseNumber::V2 {
            void: "V2: kappa impure at an end of the bracket".into(), tau_lo, tau_hi,
        };
    }
    let (Some(dm), true) = (c.d_membership, c.edge_moved) else {
        return StaircaseNumber::V5 {
            void: format!("V5: no edge move in [{}, {}]", py_g(tau_lo), py_g(tau_hi)),
            tau_lo, tau_hi, edge_moved: c.edge_moved,
        };
    };
    let kap = a.kappa.expect("kappa_pure");
    let dt = tau_hi - tau_lo;
    let d_smooth = c.d_smooth.expect("a membership term implies a smooth one");
    let rise = py_div(dm.abs(), kap);
    let slope = py_div(py_div(d_smooth.abs(), kap), dt);
    let tread = if truthy(spacing) { Some(slope * spacing.expect("truthy")) } else { None };
    StaircaseNumber::Ok(StaircaseOk {
        tau_lo,
        tau_hi,
        ds: kw.ds,
        r: kw.r,
        rise,
        dg_dtau: slope,
        dtau: dt,
        spacing,
        tread,
        lam: if truthy(tread) { Some(rise / tread.expect("truthy")) } else { None },
        entered: c.entered,
        left: c.left,
        d_membership: dm,
        d_smooth,
    })
}

/// A live root classification — Python's ok dict, fields in its key order (`void=None` implied
/// after `ds`).
#[derive(Clone, Debug, PartialEq)]
pub struct RootOk {
    pub r: f64,
    pub ds: f64,
    pub lo: f64,
    pub hi: f64,
    pub mid: f64,
    pub width: f64,
    pub kind: Option<Kind>,
    /// `kind == "crossing"` — a plain bool on this shape.
    pub root_exists: bool,
    pub set_changed: bool,
    pub argmin_moved: bool,
    pub edge_moved: bool,
    pub entered: Vec<f64>,
    pub left: Vec<f64>,
    pub d_full: Option<f64>,
    pub d_smooth: Option<f64>,
    pub d_membership: Option<f64>,
    pub h_lo: Option<f64>,
    pub h_hi: Option<f64>,
    pub h_common_lo: Option<f64>,
    pub h_common_hi: Option<f64>,
}

/// `root_class`'s two shapes. The void one is `void, r, ds, kind=None, root_exists=None`.
#[derive(Clone, Debug, PartialEq)]
pub enum RootClass {
    Void { void: String, r: f64, ds: f64 },
    Ok(RootOk),
}

/// RUNG 84's `root_class` (`engine.py:23044`) — **bisect the residual with rung 82's own
/// `_bisect`, then say EXACTLY whether the sign change it found is a root.** `eps` is no
/// classifier tolerance; it guards only a bracket so narrow the two marches are one float (V6).
pub fn root_class(
    core: &ScheduledStatorCore, bracket: (f64, f64), n_bisect: usize, eps: f64, kw: &ScanKw,
) -> RootClass {
    let bi = threshold_law::bisect(core, key_residual, bracket.0, bracket.1, n_bisect, kw);
    // `if bi.get("void")` — truthiness on the string; every void `_bisect` returns is a
    // non-empty literal, so `Some` is exactly Python's truthy
    if let Some(v) = bi.void() {
        return RootClass::Void { void: v.to_string(), r: kw.r, ds: kw.ds };
    }
    let (a, b) = match &bi {
        Bisect::Ok(x) => (x.lo, x.hi),
        _ => unreachable!("a non-void bisection is the ok shape"),
    };
    if b - a < eps {
        return RootClass::Void {
            void: format!("V6: bracket narrower than {}", py_g(eps)), r: kw.r, ds: kw.ds,
        };
    }
    let c = classify(&edge_read(core, a, kw), &edge_read(core, b, kw));
    RootClass::Ok(RootOk {
        r: kw.r,
        ds: kw.ds,
        lo: a,
        hi: b,
        mid: 0.5 * (a + b),
        width: b - a,
        kind: c.kind,
        root_exists: c.kind == Some(Kind::Crossing),
        set_changed: c.set_changed,
        argmin_moved: c.argmin_moved,
        edge_moved: c.edge_moved,
        entered: c.entered,
        left: c.left,
        d_full: c.d_full,
        d_smooth: c.d_smooth,
        d_membership: c.d_membership,
        h_lo: c.h_lo,
        h_hi: c.h_hi,
        h_common_lo: c.h_common_lo,
        h_common_hi: c.h_common_hi,
    })
}

