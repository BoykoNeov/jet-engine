//! RUNG 94 — A BURNER THAT CARRIES THE LOAD: each throttle holds thrust or HP speed, and the burner temperature
//! floats to pay for rung 92's generator.
//!
//! Anchor (the model and every pre-registered bar, committed before any of this code existed):
//! `docs/plans/rung94-anchor-burner-hold.md`. Spec: `docs/rung94-spec.md`.
//!
//! # The target
//!
//! A throttle set to `T_set` asks for what THIS engine gives at `T_set` with the generator OFF — its absolute thrust
//! or its HP shaft speed — and the burner temperature `T` then solves `Q(T; load) = Q(T_set; no load)` on the loaded
//! engine. Absolute thrust, not thrust per kg of air: the airflow moves with `T`.
//!
//! # The search ([`try_hold_root`])
//!
//! Bisection in `T`, the held quantity rising with `T`. Its LOW end is the lowest `T` the loaded engine solves at —
//! a 50 K grid from the bottom of the range, then BISECTED between the last failing and first solving point (an HP
//! 0.10 solves from 974 K, where thrust is 80 N, but not at 950 K: a grid alone would start above an 89 N target).
//! Refused, NAMING the bound, when the target lies below what the load allows at that floor (an LP 0.05 cannot run
//! below 143 N) or above the range's top.
//!
//! # Reduce — by DISPATCH
//!
//! Holding the burner temperature IS rung 92; a hold with no active load meets its target at `T_set` itself and
//! never searches. Nothing else in the model moves. The perfect gas only (rung 92's).
//!
//! # The metal (rung 93)
//!
//! Read on the loaded point from its temperatures and shares ([`crate::metal_temperature::MetalInputs::of_shares`]);
//! REPORTED by default, and on the user's switch it BINDS ([`try_metal_cap`]): the burner stops at the metal limit,
//! and the shortfall is the result.

use crate::engine::FlightCondition;
use crate::gas::{Abort, Gas};
use crate::metal_temperature::{self as mt, MetalInputs, MetalLimit, MetalReading};
use crate::shaft_offtake::{OfftakeResult, ShaftOfftakeMatcher};
use crate::turbine_cooling::TurbineCooling;

/// What each throttle holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hold {
    /// The burner temperature itself — rung 92's throttle (the default).
    BurnerTemp,
    /// The absolute thrust the unloaded engine gives at the set temperature.
    Thrust,
    /// The HP shaft speed the unloaded engine runs at the set temperature.
    HpSpeed,
}

impl Hold {
    pub const ALL: [Hold; 3] = [Hold::BurnerTemp, Hold::Thrust, Hold::HpSpeed];
    pub fn key(self) -> &'static str {
        match self { Hold::BurnerTemp => "tt4", Hold::Thrust => "thrust", Hold::HpSpeed => "speed" }
    }
}

/// The grid the search's low end steps up on, K.
pub const GRID: f64 = 50.0;
/// The search's tolerance on `T`, K.
pub const T_TOL: f64 = 1e-9;
/// How many halvings a bisection may take.
pub const MAX_HALVINGS: usize = 200;

/// Where a hold's search could not land.
#[derive(Clone, Debug, PartialEq)]
pub enum HoldRefusal {
    /// The engine solves nowhere in the range with this load.
    NoSolve,
    /// The target lies below the least the load allows: at the lowest `T` that solves (`t_floor`) the held
    /// quantity is already `q_floor`.
    BelowFloor { t_floor: f64, q_floor: f64 },
    /// The target lies above what the range's top `t_top` gives (`q_top`).
    AboveRange { t_top: f64, q_top: f64 },
    /// A solve failed between two that worked — the held quantity is not reachable continuously.
    Gap { t: f64, message: String },
}

/// A hold's landing point: the burner temperature and what it holds there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HoldRoot {
    pub t: f64,
    pub q: f64,
    /// The lowest `T` the loaded engine solves at (the search's floor).
    pub t_floor: f64,
}

/// The lowest `T` in `[lo, hi]` at which `q_at` answers: a `GRID` scan, then bisection to `T_TOL` between the last
/// failing and first answering points. `lo` itself when it answers.
pub fn floor_of(q_at: &dyn Fn(f64) -> Result<f64, Abort>, lo: f64, hi: f64) -> Option<(f64, f64)> {
    if let Ok(q) = q_at(lo) {
        return Some((lo, q));
    }
    let mut t = lo;
    let first = loop {
        t += GRID;
        if t > hi {
            return None;
        }
        if q_at(t).is_ok() {
            break t;
        }
    };
    let (mut a, mut b) = (first - GRID, first);
    for _ in 0..MAX_HALVINGS {
        if b - a <= T_TOL {
            break;
        }
        let mid = 0.5 * (a + b);
        if q_at(mid).is_ok() { b = mid } else { a = mid }
    }
    q_at(b).ok().map(|q| (b, q))
}

/// The `T` in `[lo, hi]` at which `q_at(T) = target`, the held quantity rising with `T`.
pub fn try_hold_root(
    q_at: &dyn Fn(f64) -> Result<f64, Abort>, target: f64, lo: f64, hi: f64,
) -> Result<HoldRoot, HoldRefusal> {
    let (t_floor, q_floor) = floor_of(q_at, lo, hi).ok_or(HoldRefusal::NoSolve)?;
    if q_floor > target {
        return Err(HoldRefusal::BelowFloor { t_floor, q_floor });
    }
    // Step up from the floor until the target is passed.
    let (mut a, mut qa) = (t_floor, q_floor);
    let mut b = a;
    loop {
        if qa == target {
            return Ok(HoldRoot { t: a, q: qa, t_floor });
        }
        b = (b + GRID).min(hi);
        let qb = q_at(b).map_err(|e| HoldRefusal::Gap { t: b, message: e.0 })?;
        if qb >= target {
            break;
        }
        if b >= hi {
            return Err(HoldRefusal::AboveRange { t_top: hi, q_top: qb });
        }
        a = b;
        qa = qb;
    }
    for _ in 0..MAX_HALVINGS {
        if b - a <= T_TOL {
            break;
        }
        let mid = 0.5 * (a + b);
        let qm = q_at(mid).map_err(|e| HoldRefusal::Gap { t: mid, message: e.0 })?;
        if qm < target { a = mid } else { b = mid }
    }
    // Land on the end nearer the target (both within T_TOL).
    let (qa, qb) = (q_at(a).map_err(|e| HoldRefusal::Gap { t: a, message: e.0 })?,
                    q_at(b).map_err(|e| HoldRefusal::Gap { t: b, message: e.0 })?);
    Ok(if (qa - target).abs() <= (qb - target).abs() { HoldRoot { t: a, q: qa, t_floor } }
       else { HoldRoot { t: b, q: qb, t_floor } })
}

/// Plain words for a refusal, in the hold's own units.
pub fn refusal_words(r: &HoldRefusal, hold: Hold, target: f64) -> String {
    let (what, unit, scale) = match hold {
        Hold::Thrust => ("thrust", "N per kg/s of design airflow", 1.0),
        Hold::HpSpeed => ("HP shaft speed", "% of design", 100.0),
        Hold::BurnerTemp => ("burner temperature", "K", 1.0),
    };
    match r {
        HoldRefusal::NoSolve =>
            "With this load the engine has no steady point anywhere in the range searched.".to_string(),
        HoldRefusal::BelowFloor { t_floor, q_floor } => format!(
            "The {what} asked for ({:.4} {unit}) is below the least this load allows: the engine needs at least \
             {t_floor:.0} K to carry it at all, and gives {:.4} {unit} there.", target * scale, q_floor * scale),
        HoldRefusal::AboveRange { t_top, q_top } => format!(
            "The {what} asked for ({:.4} {unit}) is more than this loaded engine gives even at {t_top:.0} K ({:.4} {unit}).",
            target * scale, q_top * scale),
        HoldRefusal::Gap { t, message } => format!(
            "The search for the {what} met a burner temperature ({t:.0} K) where the engine has no steady point. The \
             model says: {message}"),
    }
}

// =========================================================================================
// STEADY — rung 92's loaded matcher
// =========================================================================================

/// The held quantity on a steady point: absolute thrust, N, or the HP shaft speed (design 1).
pub fn held(r: &OfftakeResult, hold: Hold) -> f64 {
    match hold {
        Hold::Thrust => r.base.base.thrust,
        Hold::HpSpeed => r.base.n_hp_ratio,
        Hold::BurnerTemp => r.base.base.tt4,
    }
}

/// A steady hold.
#[derive(Clone, Debug)]
pub struct HoldPoint {
    /// The burner temperature it lands on (`T_set` itself when holding the burner temperature or with no load).
    pub tt4: f64,
    pub target: f64,
    pub point: OfftakeResult,
    /// The search's floor (`T_set` when it did not search).
    pub t_floor: f64,
}

/// Hold `hold` at the throttle `t_set` on the loaded engine `loaded`; `bare` is the same engine without the load
/// (the target's engine). `[lo, hi]` is the search range.
#[allow(clippy::too_many_arguments)]
pub fn try_hold_point(
    loaded: &ShaftOfftakeMatcher, bare: &ShaftOfftakeMatcher, flight: &FlightCondition, t_set: f64, hold: Hold,
    lo: f64, hi: f64,
) -> Result<HoldPoint, String> {
    let active = loaded.core.base.offtake.is_some_and(|o| o.is_active());
    if hold == Hold::BurnerTemp || !active {
        let point = loaded.try_match_point(flight, t_set).map_err(|e| e.0)?;
        return Ok(HoldPoint { tt4: t_set, target: held(&point, hold), point, t_floor: t_set });
    }
    let target = held(&bare.try_match_point(flight, t_set).map_err(|e| e.0)?, hold);
    let q_at = |t: f64| loaded.try_match_point(flight, t).map(|r| held(&r, hold));
    let root = try_hold_root(&q_at, target, lo, hi).map_err(|r| refusal_words(&r, hold, target))?;
    let point = loaded.try_match_point(flight, root.t).map_err(|e| e.0)?;
    Ok(HoldPoint { tt4: root.t, target, point, t_floor: root.t_floor })
}

/// Rung 93's metal on a loaded steady point, from its temperatures and shares.
pub fn metal_of(gas: &Gas, r: &OfftakeResult, lim: &MetalLimit) -> MetalReading {
    let s = &r.base.base;
    let (customer, cool) = match r.booking {
        Some(k) => (k.customer, TurbineCooling { vane: k.vane, blade: k.blade }),
        None => (0.0, TurbineCooling { vane: 0.0, blade: 0.0 }),
    };
    mt::read(&MetalInputs::of_shares(gas, s.tt4, s.station("3").tt, s.station("4").far, customer, cool), lim)
}

/// The burner temperature at which the hotter metal (relative to its limit) reaches it, in `[lo, hi]`, as a
/// function `excess(T)` (> 0 over the limit): the metal cap's landing point. `None` when the metal holds at `hi`
/// (no cap needed) — `Some(lo)` is never returned for an excess at `lo`: that is the caller's refusal.
pub fn try_metal_cap(excess: &dyn Fn(f64) -> Result<f64, Abort>, lo: f64, hi: f64) -> Result<Option<f64>, String> {
    let eh = excess(hi).map_err(|e| e.0)?;
    if eh <= 0.0 {
        return Ok(None);
    }
    let el = excess(lo).map_err(|e| e.0)?;
    if el > 0.0 {
        return Err(format!("Even at {lo:.0} K the turbine metal is {el:.1} K over its limit: the metal cap leaves this \
                            throttle nothing to run at. Cool the turbine more, or pick a newer material."));
    }
    let (mut a, mut b) = (lo, hi);
    for _ in 0..MAX_HALVINGS {
        if b - a <= T_TOL * b {
            break;
        }
        let mid = 0.5 * (a + b);
        if excess(mid).map_err(|e| e.0)? <= 0.0 { a = mid } else { b = mid }
    }
    Ok(Some(a))
}
