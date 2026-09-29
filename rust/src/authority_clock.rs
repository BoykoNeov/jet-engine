//! RUNG 81 — **THE AUTHORITY CLOCK.** `AuthorityClockTransient` (`engine.py:21795`), slice AJ.
//!
//! Rungs 72–80 read `n_live <= 3` six times off ONE side of a switch: under `min` exactly one
//! fuel-side leg reaches the actuator, and in every cell measured it was the `Tt4` governor. This
//! rung throws the switch. Headline: **authority is decided by the LAG, not by the set point** —
//! in `demand` each leg's state sits below its cap by the ramp-tracking error `tau·dc/ds`, so
//! `min` hands the actuator to the SLOWER leg, which need not be the one demanding the deeper
//! cut; and **a leg that never holds the actuator has no clock** (a 10× sweep of a masked leg's
//! `tau` moves the march by nothing, bit for bit).
//!
//! # NO TABLE, NO BUILDER — A RUNG-80 CORE PLUS THE FIVE FREE FUNCTIONS BELOW
//!
//! Plan § 5.34 (ii) measured rungs 81–84 as **reader-only**: 24 methods over the four classes,
//! every one single-definer in the MRO, and no class after rung 80 defines `at_lever`,
//! `_shared_rig`, `_cap_fuel`, `_with_coord` or any other cell. So there is **no `R81` table** and
//! no builder: a Python `AuthorityClockTransient` is a rung-80 core — built by
//! [`build_split_wall_cascade`](crate::split_wall::build_split_wall_cascade) — plus this module's
//! free functions. A `build_authority_clock_cascade` would name a machine that does not exist,
//! and the only check it could invite (*is it `R81`'s?*) is vacuous by construction.
//!
//! # WHAT STEP 2 ADDS — THE FIVE READERS
//!
//! [`central`] (`_central`), [`criterion_at`] (`_criterion_at`), [`authority_clock`],
//! [`tau_f_inert`] (`_tau_f_inert`) and [`authority_mask`]. Gated bit for bit, every returned
//! value in Python's own key order, in `tests/slice_aj_clock.rs` against
//! `oracle/probe_slice_aj_step2.py`'s output.
//!
//! * **`id(p)` → indices.** `authority_clock` maps `_riding4`'s points back to trajectory
//!   positions through `id(p)` (`engine.py:21941`, `engine.py:21944`), because `_criterion_at`
//!   differences `traj[i-1]` and `traj[i+1]`. Ported through
//!   [`riding4_idx`](crate::shared_actuator::riding4_idx), whose doc says why the two sets are
//!   equal without a no-aliasing premise.
//! * **`_criterion_at` BRANCHES ON THE COORDINATE's NAME, NOT ON THE POINT's SHAPE.** A
//!   `demand-latched` march records [`Demand`](PointExtra::Demand) points but takes the ELSE
//!   (clip-form) branch, because Python tests `coord == "demand"`. A port matching on the
//!   variant would be silently wrong there; the probe runs one latched grid for that reason.
//! * **TWO FUEL-CLOCK LOOKUPS, NOT ONE.** `_criterion_at`'s demand branch reads rung 74's
//!   argument-swapped [`demand_tau`] off `(cap_fuel, w_fuel)` (`engine.py:21874`); its clip branch
//!   AND `authority_mask` in EITHER coordinate read `lag.tau(required_fuel, g_fuel)`
//!   (`engine.py:21882`, `engine.py:22100`). Kept as Python spells them — **but no value can tell
//!   them apart, and that is measured, not assumed** (plan § 5.34.2, J2a/J2b): on a `demand`
//!   point `required_fuel = mf_sched - cap_fuel` and `g_fuel = mf_sched - w_fuel`, so
//!   `required > g` ⇔ `w > cap`, which is exactly the swap `demand_tau` makes. Either substitution
//!   survived all five gates. The distinction is one of FORM; a gate for it would be vacuous.
//! * **`authority_mask` dispatches ON THE RIG.** Its gains calls (`engine.py:22096`–`22103` —
//!   `_with_share`, `_quad_gains_at`, `_jac4`, `_charpoly4`, `_quartic_roots_c`) are made on the
//!   machine `_split_march` RETURNS, not on `self`: the one cell among them, `quad_gains_at`, is
//!   read off `m.triple_hooks()`. With no swaps the two carry the same pointers, so no value gate
//!   can tell a port that reads the caller's table from one that reads the rig's; step 7 owes the
//!   injection that does (P4).
//! * **One float sum** — `rate = sum(1.0 / t for t in tt)` (`engine.py:22104`), four terms, used
//!   only as `zeros`' bar: a naive left fold, as the four precedents at rungs 72/73/75/76 port it.
//!   The pre-registered CPython exemption is `_charpoly4`'s `c0`/`c1`, not this sum (§ 5.34 (v)).
//! * **Every two-argument `max(a, b)` is Python's** ([`py_max2`]), every sequence fold
//!   [`py_max_of`]/[`py_min_of`], every dict insertion-ordered, every `sorted` stable.
//! * **No `%g`, no carriers, no `assert`, no class state** (§ 5.34 (iv)); every refusal below is
//!   the `KeyError`/`TypeError`/`AttributeError` Python would raise on the same input.

use crate::demand_coordinate::demand_tau;
use crate::engine::FlightCondition;
use crate::fuel_transient::{AsymmetricLag, Authority, FuelPoint, PointExtra};
use crate::gas::Abort;
use crate::shared_actuator::{charpoly4, jac4, quartic_roots_c, riding4, riding4_idx, ShareScope};
use crate::split_wall::{py_max2, split_march};
use crate::stator_transient::ScheduledStatorCore;
use crate::two_lag::{py_max_of, py_min_of};

// ---------------------------------------------------------------------------------------------
// THE READERS' DEFAULTS, named once so the ported suite and the oracle cannot drift from them
// ---------------------------------------------------------------------------------------------

/// `authority_clock`'s default `tau_fs`.
pub const CLOCK_TAU_FS: [f64; 6] = [0.02, 0.05, 0.08, 0.10, 0.12, 0.20];
/// `authority_clock`'s default `tau_govs`.
pub const CLOCK_TAU_GOVS: [f64; 3] = [0.02, 0.05, 0.20];
/// `authority_clock`'s default `coords`.
pub const CLOCK_COORDS: [&str; 2] = ["demand", "clip"];
/// `authority_clock`'s default `tau_q` and `tau_s` — rung 80's, PINNED across the grid (anchor
/// control 3).
pub const CLOCK_TAU_Q: f64 = 0.05;
/// See [`CLOCK_TAU_Q`].
pub const CLOCK_TAU_S: f64 = 0.05;
/// `authority_mask`'s default `clocks`: the slow-fuel mirror cell, then rung 80's matched one.
pub const MASK_CLOCKS: [(f64, f64, f64, f64); 2] =
    [(0.20, 0.05, 0.05, 0.05), (0.05, 0.05, 0.05, 0.05)];
/// `authority_mask`'s default stride — **1, not rung 80's 5.**
pub const MASK_EVERY: usize = 1;

// ---------------------------------------------------------------------------------------------
// SMALL PYTHON SHAPES
// ---------------------------------------------------------------------------------------------

/// Python's `d[k] = v` on an insertion-ordered dict: a repeated key keeps its FIRST position and
/// takes the LAST value.
fn dict_put<K: PartialEq, V>(d: &mut Vec<(K, V)>, k: K, v: V) {
    match d.iter_mut().find(|(key, _)| *key == k) {
        Some(e) => e.1 = v,
        None => d.push((k, v)),
    }
}

/// Python's `census[a] = census.get(a, 0) + 1`, insertion-ordered.
fn tally<K: PartialEq + Copy>(d: &mut Vec<(K, usize)>, k: K) {
    match d.iter_mut().find(|(key, _)| *key == k) {
        Some(e) => e.1 += 1,
        None => d.push((k, 1)),
    }
}

/// Python's `census.get(a, 0)`.
fn get0<K: PartialEq>(d: &[(K, usize)], k: K) -> usize {
    d.iter().find(|(key, _)| *key == k).map_or(0, |e| e.1)
}

/// `len({xs})` for floats — distinct by `==`, as Python's set is on these values.
fn n_distinct(xs: impl Iterator<Item = f64>) -> usize {
    let mut seen: Vec<f64> = Vec::new();
    for x in xs {
        if !seen.contains(&x) {
            seen.push(x);
        }
    }
    seen.len()
}

/// `sorted(set(xs))` for integers.
fn sorted_set(xs: impl Iterator<Item = usize>) -> Vec<usize> {
    let mut v: Vec<usize> = xs.collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// Python's `str(x)` for a float — **on a DECLARED domain only**, and it refuses outside it.
///
/// `_tau_f_inert` keys its dict `f"{c}@{tg}"`. Python's float `repr` and Rust's `Display` both
/// print the SHORTEST round-trip digits; they differ only in that Python appends `.0` to an
/// integral value and switches to exponent form below `1e-4` or from `1e16` up, which `Display`
/// never does. Inside `[1e-4, 1e16)` (and at zero) the `.0` is the whole difference. Outside it
/// this would print a different string, so it panics rather than guess — `py_repr`'s precedent of
/// a formatter correct only on a closed set, said so.
fn py_float_str(x: f64) -> String {
    let a = x.abs();
    assert!(x.is_finite() && (x == 0.0 || ((1e-4..1e16).contains(&a))),
            "rung-81: `py_float_str` is Python's `str(float)` only on [1e-4, 1e16) and zero; \
             got {x:e}, which Python prints in exponent form");
    let s = format!("{x}");
    if s.contains('.') { s } else { s + ".0" }
}

/// Python's `p[key]` for the four keys `_central` is asked for, plus the two `_criterion_at`
/// reads beside them. A key the point does not carry is Python's `KeyError`: `cap_*`/`w_*` exist
/// only on a rung-74 [`Demand`](PointExtra::Demand) point, `required_*`/`g_fuel` on it and on
/// rung 72's [`Shared`](PointExtra::Shared).
fn key_of(p: &FuelPoint, key: &str) -> f64 {
    match (key, &p.extra) {
        ("s", _) => p.s,
        ("cap_fuel", PointExtra::Demand { cap_fuel, .. }) => *cap_fuel,
        ("cap_gov", PointExtra::Demand { cap_gov, .. }) => *cap_gov,
        ("w_fuel", PointExtra::Demand { w_fuel, .. }) => *w_fuel,
        ("required_fuel", PointExtra::Shared { required_fuel, .. })
        | ("required_fuel", PointExtra::Demand { required_fuel, .. }) => *required_fuel,
        ("required_gov", PointExtra::Shared { required_gov, .. })
        | ("required_gov", PointExtra::Demand { required_gov, .. }) => *required_gov,
        ("g_fuel", PointExtra::Shared { g_fuel, .. })
        | ("g_fuel", PointExtra::Demand { g_fuel, .. }) => *g_fuel,
        _ => panic!("rung-81: this point carries no {key:?} -- Python raises KeyError here"),
    }
}

/// Python's `p["authority"]` — rung 72's label, on the two six-state routes only.
fn authority_of(p: &FuelPoint) -> Authority {
    match p.extra {
        PointExtra::Shared { authority, .. } | PointExtra::Demand { authority, .. } => authority,
        _ => panic!("rung-81: this point carries no `authority` -- Python raises KeyError here"),
    }
}

/// Python's `p["b"], p["v"]` on the two six-state routes. Anything else is `KeyError`.
fn b_v(p: &FuelPoint) -> (f64, f64) {
    match p.extra {
        PointExtra::Shared { b, v, .. } | PointExtra::Demand { b, v, .. } => (b, v),
        _ => panic!("rung-81: this point carries no `b`/`v` -- Python raises KeyError here"),
    }
}

/// Python's `m.bleed_lim.b_max` — `AttributeError` on a valve-less rig.
fn b_max_of(m: &ScheduledStatorCore) -> f64 {
    m.fuel.inner.lever.lim
        .unwrap_or_else(|| panic!("rung-81: `m.bleed_lim.b_max` on a rig with no valve -- \
                                   Python raises AttributeError here"))
        .b_max
}

/// The rig's fuel lag — Python's `lag.tau(...)` on `None` is `AttributeError`.
fn lag_of(lag: Option<&AsymmetricLag>) -> &AsymmetricLag {
    lag.unwrap_or_else(|| panic!("rung-81: `lag.tau` on a rig with no fuel lag -- Python raises \
                                  AttributeError here"))
}

// ---------------------------------------------------------------------------------------------
// THE CRITERION, READ OFF THE SHIPPED TRAJECTORY
// ---------------------------------------------------------------------------------------------

/// RUNG 81's `_central` (`engine.py:21854`) — `d(key)/ds` by central difference, the spacing
/// taken from `s` itself.
///
/// **`i = 0` reads `traj[-1]`**, the LAST point, because that is what Python's negative index
/// does. No caller passes it (`authority_clock` counts the ends as `n_edge` and skips them), so
/// this is Python's shape kept, not a reached path; `i = len - 1` is `IndexError` in both.
pub fn central(traj: &[FuelPoint], i: usize, key: &str) -> f64 {
    let prev = if i == 0 { traj.len() - 1 } else { i - 1 };
    let (a, b) = (&traj[i + 1], &traj[prev]);
    (key_of(a, key) - key_of(b, key)) / (a.s - b.s)
}

/// One scored point of the § 1 race — Python's `_criterion_at` dict, fields in its key order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Criterion {
    pub s: f64,
    /// `required_gov - required_fuel` — the SET-POINT gap.
    pub setpoint_gap: f64,
    /// The LAG-ERROR gap, in whichever coordinate's sign.
    pub lag_gap: f64,
    /// The fuel leg's ACTIVE clock, read through the accessor the march integrates with.
    pub tau_f: f64,
    pub tau_gov: f64,
    pub slope_f: f64,
    pub slope_r: f64,
    /// Python's `"fuel" if gap < lag_gap else "gov"` — only ever those two.
    pub predicted: Authority,
    /// The label the march recorded.
    pub measured: Authority,
    /// `(lag_gap - gap) / scale`, or `0.0` where both gaps are zero.
    pub margin: f64,
}

/// RUNG 81's `_criterion_at` (`engine.py:21860`) — **the anchor § 1 race at one interior index.**
///
/// Branches on `coord == "demand"`, never on the point's variant (see the module header). The
/// demand branch reads the fuel clock through rung 74's [`demand_tau`] — the argument swap is
/// that function's, so it is called `(lag, cap_fuel, w_fuel)` exactly as Python calls it; the
/// else branch reads `lag.tau(required_fuel, g_fuel)` and EXCHANGES the two `tau` terms.
///
/// The slopes are each computed once and used in both `lag_gap` and the reported `slope_*` —
/// Python calls `_central` twice per key, a pure function of the same three points, so the two
/// calls return one float.
pub fn criterion_at(
    traj: &[FuelPoint], i: usize, coord: &str, lag: Option<&AsymmetricLag>, tau_gov: f64,
) -> Criterion {
    let p = &traj[i];
    let gap = key_of(p, "required_gov") - key_of(p, "required_fuel");
    let (tau_f, lag_gap, slope_f, slope_r) = if coord == "demand" {
        let tau_f = demand_tau(lag_of(lag), key_of(p, "cap_fuel"), key_of(p, "w_fuel"));
        let (sf, sr) = (central(traj, i, "cap_fuel"), central(traj, i, "cap_gov"));
        (tau_f, tau_f * sf - tau_gov * sr, sf, sr)
    } else {
        // THE COORDINATE's OWN SIGN: the clip state lags `required` from BELOW, so the two tau
        // terms EXCHANGE — one expression with a swapped pair, as Python writes it.
        let tau_f = lag_of(lag).tau(key_of(p, "required_fuel"), key_of(p, "g_fuel"));
        let (sf, sr) = (central(traj, i, "required_fuel"), central(traj, i, "required_gov"));
        (tau_f, tau_gov * sr - tau_f * sf, sf, sr)
    };
    let scale = py_max2(gap.abs(), lag_gap.abs());
    Criterion {
        s: p.s,
        setpoint_gap: gap,
        lag_gap,
        tau_f,
        tau_gov,
        slope_f,
        slope_r,
        predicted: if gap < lag_gap { Authority::Fuel } else { Authority::Gov },
        measured: authority_of(p),
        margin: if scale > 0.0 { (lag_gap - gap) / scale } else { 0.0 },
    }
}

// ---------------------------------------------------------------------------------------------
// §§ 1–3: THE GRID, BOTH COORDINATES
// ---------------------------------------------------------------------------------------------

/// One `(coord, tau_f, tau_gov)` cell of the grid — Python's row dict, fields in its key order.
#[derive(Clone, Debug, PartialEq)]
pub struct ClockRow {
    pub coord: &'static str,
    pub tau_f: f64,
    pub tau_gov: f64,
    pub tau_q: f64,
    pub tau_s: f64,
    pub phi_lim: f64,
    pub phi_air: Option<f64>,
    pub max_tt4: f64,
    /// Never read `n_riding4` without it — rung 80 § 8's frozen plant.
    pub riding4_valid: bool,
    /// Every riding point, trajectory ENDS INCLUDED.
    pub n_riding4: usize,
    /// Who held the actuator at each riding point — insertion-ordered.
    pub census: Vec<(Authority, usize)>,
    pub n_fuel: usize,
    pub n_gov: usize,
    /// Interior riding points only.
    pub n_scored: usize,
    /// Riding points at a trajectory end, where no central difference exists.
    pub n_edge: usize,
    pub n_agree: usize,
    pub agreement: Option<f64>,
    pub worst_miss: Option<f64>,
    pub min_margin: Option<f64>,
    pub cells: Vec<Criterion>,
}

/// One row of the SHARED-wall `clip` control sweep.
#[derive(Clone, Debug, PartialEq)]
pub struct ClipControlRow {
    pub tau_f: f64,
    pub n_riding4: usize,
    pub census: Vec<(Authority, usize)>,
    pub n_fuel: usize,
    pub max_tt4: f64,
}

/// One `(coord, tau_gov)` column of [`tau_f_inert`].
#[derive(Clone, Debug, PartialEq)]
pub struct TauFInert {
    pub n_tau_f: usize,
    /// `len(sig)` of the column's FIRST row — 341 points × 4 states on the shipped grid.
    pub n_floats: usize,
    /// The most floats any later row moves against the first.
    pub n_differing: usize,
    pub march_identical: bool,
    pub tt4_identical: bool,
    pub riding4_identical: bool,
    /// `sorted(set(...))`.
    pub n_riding4: Vec<usize>,
    pub all_valid: bool,
}

/// §§ 1–3's reading — Python's `authority_clock` return dict, fields in its key order.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthorityClock {
    pub phi_lim: f64,
    pub phi_air: Option<f64>,
    pub tau_fs: Vec<f64>,
    pub tau_govs: Vec<f64>,
    pub tau_q: f64,
    pub tau_s: f64,
    pub coords: Vec<&'static str>,
    pub ds: f64,
    pub rows: Vec<ClockRow>,
    /// THE CONTROL: rung 80's own cell (`demand`, `tau_f == tau_gov == 0.05`) is all-governor.
    pub control_all_gov: bool,
    pub control_n_riding4: Vec<usize>,
    /// The census of the LAST control row with `tau_f == 0.05`; EMPTY if there is none.
    pub control_clip_shared: Vec<(Authority, usize)>,
    pub control_clip_fuel: usize,
    pub control_clip_rows: Vec<ClipControlRow>,
    /// Where the fuel leg holds, its clock is NOT inert: `max_Tt4` takes more than one value.
    pub control_clip_tau_f_live: bool,
    /// V4: fuel authority on EVERY valid row.
    pub all_fuel: bool,
    /// P1: the worst cell's agreement, over the valid rows that scored.
    pub agreement: Option<f64>,
    pub n_scored: usize,
    pub worst_miss: Option<f64>,
    /// Per coordinate, in `coords` order: `sorted((tau_f, tau_gov, n_fuel))` of the fuel region.
    pub fuel_cells: Vec<(&'static str, Vec<(f64, f64, usize)>)>,
    /// Per coordinate: `sorted({slow_fuel | fast_fuel | matched})` — string order.
    pub fuel_side: Vec<(&'static str, Vec<&'static str>)>,
    /// Keyed `f"{coord}@{tau_gov}"`, in Python's loop order; `None` where the column cannot compare.
    pub tau_f_inert: Vec<(String, Option<TauFInert>)>,
    pub n_invalid: usize,
}

/// A march's comparison signature — `phi_lp`, `Tt4`, `b`, `v` at every point, in that order.
type Sig = Vec<f64>;

/// §§ 1–3 (`engine.py:21904`) — **who holds the actuator, over a clean `(tau_f, tau_gov)` grid.**
///
/// Rows in Python's loop order: coordinate outer, `tau_f`, then `tau_gov`. Then the SECOND
/// control, which hard-codes `clip`, the SHARED wall (`phi_air = None`) and `tau_gov = 0.05`, but
/// passes the CALLER's `tau_q`/`tau_s` (`engine.py:21980`).
///
/// **Each aggregate has its own filter, and they are not interchangeable:** `census` and
/// `n_riding4` count every riding point, ends included; `cells` only the interior ones;
/// `n_scored` and `worst_miss` fold the VALID rows; `agreement` the valid rows that SCORED; and
/// [`tau_f_inert`] gets ALL rows, invalid ones included (which is why it reports `all_valid`).
///
/// The per-march signatures are kept local, as Python keeps them: the booleans are returned, the
/// floats are not.
#[allow(clippy::too_many_arguments)]
pub fn authority_clock(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, phi_air: Option<f64>, tau_fs: &[f64], tau_govs: &[f64], tau_q: f64,
    tau_s: f64, coords: &[&'static str], r: f64, s_settle: f64, ds: f64, v_max: f64, inc: bool,
) -> AuthorityClock {
    let mut rows: Vec<ClockRow> = Vec::new();
    let mut sigs: Vec<((&'static str, f64), Vec<Sig>)> = Vec::new();
    for &coord in coords {
        for &tf in tau_fs {
            for &tg in tau_govs {
                let (m, _surge, lag, traj) = split_march(
                    core, flight, tt4_lo, tt4_hi, tt4_max, phi_lim, phi_air, coord,
                    (tf, tg, tau_q, tau_s), r, s_settle, ds, v_max, inc);
                let b_max = b_max_of(&m);
                // `sigs.setdefault((coord, tg), []).append(...)`
                let sig: Sig = traj.iter()
                    .flat_map(|p| {
                        let (b, v) = b_v(p);
                        [p.phi_lp, p.tt4, b, v]
                    })
                    .collect();
                match sigs.iter_mut().find(|(k, _)| k.0 == coord && k.1 == tg) {
                    Some(e) => e.1.push(sig),
                    None => sigs.push(((coord, tg), vec![sig])),
                }
                let tt4s: Vec<f64> = traj.iter().map(|p| p.tt4).collect();
                let max_t = py_max_of(&tt4s);
                // `seen = {id(p) for p in ride}` + `enumerate(traj)`: the riding INDICES
                let ride = riding4_idx(&traj, b_max);
                let mut cells: Vec<Criterion> = Vec::new();
                let mut edge = 0usize;
                for &i in &ride {
                    if i == 0 || i == traj.len() - 1 {
                        edge += 1; // no central difference exists -- COUNTED, V5
                        continue;
                    }
                    cells.push(criterion_at(&traj, i, coord, lag.as_ref(), tg));
                }
                let mut census: Vec<(Authority, usize)> = Vec::new();
                for &i in &ride {
                    tally(&mut census, authority_of(&traj[i]));
                }
                let agree = cells.iter().filter(|c| c.predicted == c.measured).count();
                let bad: Vec<f64> = cells.iter()
                    .filter(|c| c.predicted != c.measured)
                    .map(|c| c.margin.abs())
                    .collect();
                let margins: Vec<f64> = cells.iter().map(|c| c.margin.abs()).collect();
                rows.push(ClockRow {
                    coord,
                    tau_f: tf,
                    tau_gov: tg,
                    tau_q,
                    tau_s,
                    phi_lim,
                    phi_air,
                    max_tt4: max_t,
                    riding4_valid: max_t > tt4_lo * (1.0 + 1e-9),
                    n_riding4: ride.len(),
                    n_fuel: get0(&census, Authority::Fuel),
                    n_gov: get0(&census, Authority::Gov),
                    census,
                    n_scored: cells.len(),
                    n_edge: edge,
                    n_agree: agree,
                    agreement: if cells.is_empty() {
                        None
                    } else {
                        Some(agree as f64 / cells.len() as f64)
                    },
                    worst_miss: if bad.is_empty() { None } else { Some(py_max_of(&bad)) },
                    min_margin: if margins.is_empty() { None } else { Some(py_min_of(&margins)) },
                    cells,
                });
            }
        }
    }
    // THE SECOND CONTROL: `clip` at the SHARED wall, swept in `tau_f`.
    let mut ctl_rows: Vec<ClipControlRow> = Vec::new();
    let mut ccensus: Vec<(Authority, usize)> = Vec::new();
    for &tf in tau_fs {
        let (cm, _, _, ctraj) = split_march(
            core, flight, tt4_lo, tt4_hi, tt4_max, phi_lim, None, "clip",
            (tf, 0.05, tau_q, tau_s), r, s_settle, ds, v_max, inc);
        let cride = riding4(&ctraj, b_max_of(&cm));
        let mut cc: Vec<(Authority, usize)> = Vec::new();
        for p in &cride {
            tally(&mut cc, authority_of(p));
        }
        let tt4s: Vec<f64> = ctraj.iter().map(|p| p.tt4).collect();
        ctl_rows.push(ClipControlRow {
            tau_f: tf,
            n_riding4: cride.len(),
            n_fuel: get0(&cc, Authority::Fuel),
            census: cc.clone(),
            max_tt4: py_max_of(&tt4s),
        });
        if tf == 0.05 {
            ccensus = cc;
        }
    }

    let val: Vec<&ClockRow> = rows.iter().filter(|x| x.riding4_valid).collect();
    let dem: Vec<&ClockRow> = val.iter().copied().filter(|x| x.coord == "demand").collect();
    // `x["tau_f"] == x["tau_gov"] == 0.05` — Python's CHAINED comparison
    let ctl: Vec<&ClockRow> = dem.iter().copied()
        .filter(|x| x.tau_f == x.tau_gov && x.tau_gov == 0.05)
        .collect();
    let scored: Vec<f64> = val.iter()
        .filter(|x| x.n_scored > 0)
        .map(|x| x.agreement.expect("a row that scored has an agreement"))
        .collect();
    let misses: Vec<f64> = val.iter().filter_map(|x| x.worst_miss).collect();
    let mut fuel_cells: Vec<(&'static str, Vec<(f64, f64, usize)>)> = Vec::new();
    let mut fuel_side: Vec<(&'static str, Vec<&'static str>)> = Vec::new();
    for &c in coords {
        let region: Vec<&&ClockRow> = val.iter().filter(|x| x.coord == c && x.n_fuel > 0).collect();
        let mut cells: Vec<(f64, f64, usize)> =
            region.iter().map(|x| (x.tau_f, x.tau_gov, x.n_fuel)).collect();
        // Python's tuple order, stable; the taus are never NaN, so `partial_cmp` is total here
        cells.sort_by(|a, b| {
            a.0.partial_cmp(&b.0).expect("tau_f is not NaN")
                .then(a.1.partial_cmp(&b.1).expect("tau_gov is not NaN"))
                .then(a.2.cmp(&b.2))
        });
        dict_put(&mut fuel_cells, c, cells);
        let mut side: Vec<&'static str> = Vec::new();
        for x in &region {
            let lab = if x.tau_f > x.tau_gov {
                "slow_fuel"
            } else if x.tau_f < x.tau_gov {
                "fast_fuel"
            } else {
                "matched"
            };
            if !side.contains(&lab) {
                side.push(lab);
            }
        }
        side.sort_unstable();
        dict_put(&mut fuel_side, c, side);
    }
    let control_all_gov = !ctl.is_empty() && ctl.iter().all(|x| x.n_fuel == 0 && x.n_gov > 0);
    let control_n_riding4 = ctl.iter().map(|x| x.n_riding4).collect();
    let all_fuel = !val.is_empty() && val.iter().all(|x| x.n_gov == 0);
    let agreement = if scored.is_empty() { None } else { Some(py_min_of(&scored)) };
    let n_scored = val.iter().map(|x| x.n_scored).sum();
    let worst_miss = if misses.is_empty() { None } else { Some(py_max_of(&misses)) };
    let n_invalid = rows.len() - val.len();
    let inert = tau_f_inert(&rows, &sigs, coords, tau_govs);
    AuthorityClock {
        phi_lim,
        phi_air,
        tau_fs: tau_fs.to_vec(),
        tau_govs: tau_govs.to_vec(),
        tau_q,
        tau_s,
        coords: coords.to_vec(),
        ds,
        control_all_gov,
        control_n_riding4,
        control_clip_fuel: get0(&ccensus, Authority::Fuel),
        control_clip_shared: ccensus,
        control_clip_tau_f_live: n_distinct(ctl_rows.iter().map(|x| x.max_tt4)) > 1,
        control_clip_rows: ctl_rows,
        all_fuel,
        agreement,
        n_scored,
        worst_miss,
        fuel_cells,
        fuel_side,
        tau_f_inert: inert,
        n_invalid,
        rows,
    }
}

/// RUNG 81's `_tau_f_inert` (`engine.py:22039`) — **per `(coord, tau_gov)` column, does every
/// `tau_f` give the SAME MARCH**, bit for bit over every signature float, no tolerance.
///
/// `sigs` is `authority_clock`'s local dict, in insertion order. `None` where the column has fewer
/// than two rows, or where its signature count disagrees with its row count. The float count is
/// `zip`'s — truncated to the SHORTER of the two signatures, as Python's is, so a short (refused)
/// march compares on its own length.
pub fn tau_f_inert(
    rows: &[ClockRow], sigs: &[((&'static str, f64), Vec<Sig>)], coords: &[&'static str],
    tau_govs: &[f64],
) -> Vec<(String, Option<TauFInert>)> {
    let mut out: Vec<(String, Option<TauFInert>)> = Vec::new();
    for &c in coords {
        for &tg in tau_govs {
            let key = format!("{c}@{}", py_float_str(tg));
            let col: Vec<&ClockRow> =
                rows.iter().filter(|x| x.coord == c && x.tau_gov == tg).collect();
            let empty: Vec<Sig> = Vec::new();
            let sg = sigs.iter().find(|(k, _)| k.0 == c && k.1 == tg).map_or(&empty, |e| &e.1);
            if col.len() < 2 || sg.len() != col.len() {
                dict_put(&mut out, key, None);
                continue;
            }
            let nd = sg[1..].iter()
                .map(|s| sg[0].iter().zip(s).filter(|(a, b)| a != b).count())
                .max()
                .expect("at least one later row");
            dict_put(&mut out, key, Some(TauFInert {
                n_tau_f: col.len(),
                n_floats: sg[0].len(),
                n_differing: nd,
                march_identical: nd == 0,
                tt4_identical: n_distinct(col.iter().map(|x| x.max_tt4)) == 1,
                riding4_identical: sorted_set(col.iter().map(|x| x.n_riding4)).len() == 1,
                n_riding4: sorted_set(col.iter().map(|x| x.n_riding4)),
                all_valid: col.iter().all(|x| x.riding4_valid),
            }));
        }
    }
    out
}

// ---------------------------------------------------------------------------------------------
// §§ 4–5: THE MIRROR MASK
// ---------------------------------------------------------------------------------------------

/// One interior cell of the mask — Python's cell dict, fields in its key order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaskCell {
    pub s: f64,
    pub phi: f64,
    pub authority: Option<Authority>,
    pub masked: Option<Authority>,
    pub mask_leak: Option<f64>,
    pub c1: f64,
    pub c0: f64,
    /// Quartic roots with `|z| < 1e-4·rate`.
    pub zeros: usize,
    /// `max(|(F_q·C_v)·V_f|, |(F_v·V_q)·C_f|)` — ONE field, where rung 80 reports the two.
    pub cyc: f64,
}

/// One authority's entry in an arm's `by_authority`.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthStats {
    pub n: usize,
    /// Folded from `0.0` with Python's two-argument `max`.
    pub max_cyc: f64,
    /// Folded from `0.0` over `abs(mask_leak)`.
    pub max_leak: f64,
    /// `sorted(set(...))`.
    pub zeros: Vec<usize>,
}

/// One clock setting of the mask.
#[derive(Clone, Debug, PartialEq)]
pub struct MaskArm {
    pub taus: (f64, f64, f64, f64),
    pub riding4_valid: bool,
    pub max_tt4: f64,
    pub n_riding: usize,
    pub n_sampled: usize,
    pub n_interior: usize,
    /// Python's `skipped` dict: `(switch, regime)`.
    pub skipped: (usize, usize),
    /// Insertion-ordered, keyed by the gains' authority label.
    pub by_authority: Vec<(Option<Authority>, AuthStats)>,
    pub cells: Vec<MaskCell>,
}

/// §§ 4–5's reading — Python's `authority_mask` return dict, fields in its key order.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthorityMask {
    pub coord: &'static str,
    pub clocks: Vec<(f64, f64, f64, f64)>,
    pub phi_lim: f64,
    pub phi_air: Option<f64>,
    pub ds: f64,
    pub arms: Vec<MaskArm>,
    /// THE VACUITY FLAG, first and unconditional: one authority regime only, over the ALIVE arms.
    pub vacuous: bool,
    pub n_fuel_interior: usize,
    pub n_gov_interior: usize,
    pub all_differenced: bool,
    pub ever_two_authorities: bool,
    pub max_mask_leak: Option<f64>,
    pub cyc_fuel_auth: Option<f64>,
    pub cyc_gov_auth: Option<f64>,
    pub zeros_fuel_auth: Vec<usize>,
    pub zeros_gov_auth: Vec<usize>,
}

/// Python's `abs(c["mask_leak"])` — `TypeError` on `None`.
fn leak_abs(c: &MaskCell) -> f64 {
    c.mask_leak
        .unwrap_or_else(|| panic!("rung-81: abs(None) on a cell's mask_leak -- Python raises \
                                   TypeError here"))
        .abs()
}

/// §§ 4–5 (`engine.py:22069`) — **rung 72's block, read on the OTHER side of the switch.**
///
/// # IT IS NOT RUNG 80's `split_gains`, AND FIVE DIFFERENCES ARE THE PORT
///
/// * the stride defaults to **1** ([`MASK_EVERY`]), not 5;
/// * `cyc` is ONE field, a two-argument `max` of the two left-to-right products;
/// * the per-authority folds start at `0.0` and use Python's two-argument `max`;
/// * `abs(mask_leak)` REFUSES a `None` (Python's `TypeError`) rather than folding it;
/// * the aggregates run over the ALIVE (`riding4_valid`) arms only.
///
/// # THE GAINS GO THROUGH THE RIG's TABLE, UNDER `ShareScope("max")`
///
/// `m._with_share("max", m._quad_gains_at, flight, p, None, surge, Tt4_max)` (`engine.py:22096`)
/// with `_quad_gains_at`'s defaults `(1e-7, 1e-5, 1e-4, True, 4.0)` — read off `m`, the machine
/// `split_march` built. The fuel clock is `lag.tau(required_fuel, g_fuel)` in EITHER coordinate
/// (`engine.py:22100`). An `Abort` from the gains chain propagates: Python does not catch it.
#[allow(clippy::too_many_arguments)]
pub fn authority_mask(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, phi_air: Option<f64>, clocks: &[(f64, f64, f64, f64)], coord: &'static str,
    r: f64, s_settle: f64, ds: f64, v_max: f64, inc: bool, every: usize,
) -> Result<AuthorityMask, Abort> {
    let mut out: Vec<MaskArm> = Vec::new();
    for &taus in clocks {
        let (m, surge, lag, traj) = split_march(
            core, flight, tt4_lo, tt4_hi, tt4_max, phi_lim, phi_air, coord, taus, r, s_settle,
            ds, v_max, inc);
        let tt4s: Vec<f64> = traj.iter().map(|p| p.tt4).collect();
        let max_t = py_max_of(&tt4s);
        let pts = riding4(&traj, b_max_of(&m));
        let sampled: Vec<&FuelPoint> = pts.iter().step_by(every).collect();
        let mut cells: Vec<MaskCell> = Vec::new();
        let (mut sk_switch, mut sk_regime) = (0usize, 0usize);
        for p in sampled.iter() {
            let gg = {
                let _sh = ShareScope::set(&m, "max");
                (m.triple_hooks().quad_gains_at)(&m, flight, p, None, surge.as_ref(), tt4_max,
                                                 1e-7, 1e-5, 1e-4, true, 4.0)?
            };
            if !gg.interior {
                if gg.near_switch {
                    sk_switch += 1;
                } else {
                    sk_regime += 1;
                }
                continue;
            }
            let tau_f = lag_of(lag.as_ref()).tau(key_of(p, "required_fuel"), key_of(p, "g_fuel"));
            let tt = (tau_f, taus.1, taus.2, taus.3);
            let coef = charpoly4(&jac4(&gg, tt));
            let roots = quartic_roots_c(&coef);
            let rate = 1.0 / tt.0 + 1.0 / tt.1 + 1.0 / tt.2 + 1.0 / tt.3;
            cells.push(MaskCell {
                s: p.s,
                phi: p.phi_lp,
                authority: gg.authority,
                masked: gg.masked,
                mask_leak: gg.mask_leak,
                c1: coef[3],
                c0: coef[4],
                zeros: roots.iter().filter(|z| z.abs() < 1e-4 * rate).count(),
                cyc: py_max2((gg.f_q * gg.c_v * gg.v_f).abs(), (gg.f_v * gg.v_q * gg.c_f).abs()),
            });
        }
        let mut by: Vec<(Option<Authority>, AuthStats)> = Vec::new();
        for c in &cells {
            let i = match by.iter().position(|(a, _)| *a == c.authority) {
                Some(i) => i,
                None => {
                    by.push((c.authority,
                             AuthStats { n: 0, max_cyc: 0.0, max_leak: 0.0, zeros: Vec::new() }));
                    by.len() - 1
                }
            };
            let d = &mut by[i].1;
            d.n += 1;
            d.max_cyc = py_max2(d.max_cyc, c.cyc);
            d.max_leak = py_max2(d.max_leak, leak_abs(c));
            if !d.zeros.contains(&c.zeros) {
                d.zeros.push(c.zeros);
            }
        }
        for (_, d) in by.iter_mut() {
            d.zeros.sort_unstable();
        }
        out.push(MaskArm {
            taus,
            riding4_valid: max_t > tt4_lo * (1.0 + 1e-9),
            max_tt4: max_t,
            n_riding: pts.len(),
            n_sampled: sampled.len(),
            n_interior: cells.len(),
            skipped: (sk_switch, sk_regime),
            by_authority: by,
            cells,
        });
    }
    let alive: Vec<&MaskArm> = out.iter().filter(|a| a.riding4_valid).collect();
    let cells_all: Vec<&MaskCell> = alive.iter().flat_map(|a| a.cells.iter()).collect();
    let fuel: Vec<&MaskCell> =
        cells_all.iter().copied().filter(|c| c.authority == Some(Authority::Fuel)).collect();
    let gov: Vec<&MaskCell> =
        cells_all.iter().copied().filter(|c| c.authority == Some(Authority::Gov)).collect();
    let leaks: Vec<f64> = cells_all.iter().map(|c| leak_abs(c)).collect();
    let cyc_f: Vec<f64> = fuel.iter().map(|c| c.cyc).collect();
    let cyc_g: Vec<f64> = gov.iter().map(|c| c.cyc).collect();
    let fold_max = |xs: &[f64]| if xs.is_empty() { None } else { Some(py_max_of(xs)) };
    Ok(AuthorityMask {
        coord,
        clocks: clocks.to_vec(),
        phi_lim,
        phi_air,
        ds,
        vacuous: fuel.is_empty() || gov.is_empty(),
        n_fuel_interior: fuel.len(),
        n_gov_interior: gov.len(),
        all_differenced: alive.iter().all(|a| a.skipped == (0, 0)),
        ever_two_authorities: cells_all.iter()
            .any(|c| !matches!(c.authority, Some(Authority::Fuel) | Some(Authority::Gov))),
        max_mask_leak: fold_max(&leaks),
        cyc_fuel_auth: fold_max(&cyc_f),
        cyc_gov_auth: fold_max(&cyc_g),
        zeros_fuel_auth: sorted_set(fuel.iter().map(|c| c.zeros)),
        zeros_gov_auth: sorted_set(gov.iter().map(|c| c.zeros)),
        arms: out,
    })
}
