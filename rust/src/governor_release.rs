//! RUNG 107 — THE GOVERNOR BESIDE THE REALISTIC RELEASE: rung 95's governor as the demand rung 52's fast-attack /
//! slow-release lag is cut from.
//!
//! Anchor (the device and every pre-registered bar, committed before any of this code existed):
//! `docs/plans/rung107-anchor-governor-release.md`. Spec: `docs/rung107-spec.md`.
//!
//! # The lag is measured off the governor's demand
//!
//! Rung 52 carries the acceleration schedule's and the stall floor's cut as a THIRD state `g`, lagging the cut the
//! instant legs would make, `req`, with two constants — fast to cut, slow to give back — off the open ramp; rung 95
//! replaced the ramp by a governor and refused the lag. Here the governor's fuel `G(ν, s)` (rungs 95/96's laws,
//! unchanged and stateless) is the demand, in `integrate_fuel_asym`'s order:
//!
//! ```text
//! u₀    = G − g                                                   (the lagged cut)
//! u     = min(u₀, the redline and the metal leg solved off u₀)    (instant, on top — rung 52's "reads the CLIPPED fuel")
//! req   = max(0, G − min(schedule cap, floor cap) solved off G)   (rung 52's target, off the DEMAND)
//! dg/ds = (req − g)/τ_att   if req > g     (fast attack)
//!         (req − g)/τ_rel   otherwise      (slow release)
//! ```
//!
//! The instant legs are solved off `u₀` together and the smallest taken (rung 95's min-select), so at `g = 0` they are
//! rung 95's. `τ_rel` is never read while `req > g`, so everything before the first crossing is `τ_rel`-free (rung 52's
//! pinned trigger), governed or not.
//!
//! # The minimum-fuel stop (rung 106's, on the governed path)
//!
//! An applied fuel below the leanest the engine runs on is HELD at that wall and marked ([`Leg::MinFuel`]). The open
//! path keeps rung 52's `1e−9` floor instead, so it stays rung 52's march.
//!
//! # Reduce
//!
//! The open ramp handed in as the [`Demand`] gives rung 52's march, bit for bit (`tests/rung107.rs`). Before the lagged
//! legs first ask for a cut the run IS rung 95's.

use crate::engine::FlightCondition;
use crate::fuel_transient::{point, AsymmetricLag, FuelInstant, FuelLimiters, FuelPoint, FuelTransientCore, PointExtra};
use crate::gas::Abort;
use crate::governor::{self as gv, Clamp, GovCall, GovFailure, GovMark, Leg};
use crate::governor_lag::Demand;
use crate::two_spool_transient::Instant2;

/// One derivative evaluation.
#[derive(Clone, Debug)]
pub struct ReleaseDer {
    pub da: f64,
    pub db: f64,
    pub dg: f64,
    /// The demand the lag and the legs cut, and where it sits.
    pub mf_dem: f64,
    pub clamp: Clamp,
    /// The cut the instant schedule / floor would make here (`req`).
    pub required: f64,
    /// The applied fuel and what set it.
    pub mf: f64,
    pub leg: Leg,
    pub inst: FuelInstant,
}

/// RUNG 107. One derivative evaluation at the lag `g`: the demand, `G − g`, the instant legs (redline, metal) solved off
/// it, held at the lean wall if it does not run (governed only), the instant there, the target `req` off the DEMAND,
/// and `dg/ds` on the active constant. The first of equal instant cuts wins, in the order redline, metal.
#[allow(clippy::too_many_arguments)]
pub fn try_der(
    ft: &FuelTransientCore, flight: &FlightCondition, dem: Demand<'_>, lim: &FuelLimiters<'_>,
    metal: Option<&dyn Fn(&Instant2) -> f64>, a: f64, b: f64, g: f64, s: f64,
) -> Result<ReleaseDer, GovFailure> {
    ft.inner.inner.base.light_at(s);   // RUNG 102: the afterburner as lit at this march time
    assert!(lim.freeze.is_none() && lim.tau_gov.is_none() && lim.s_off.is_none() && lim.tau_rel.is_none(),
            "rung-107: rung 52's lag on rung 43's plain legs — no freeze, no lagged redline, no forced release");
    let lag: AsymmetricLag = lim.lag.expect("rung-107: the release's lag");
    let open = matches!(dem, Demand::Ramp(_));
    let fail = |call: GovCall, mf: f64| move |e: Abort| GovFailure { message: e.0, call, mf };
    let (mf_dem, clamp) = dem.try_fuel(ft, flight, a, b, s).map_err(fail(GovCall::Governor, f64::NAN))?;
    let mut u0 = mf_dem - g;
    let mut leg = if g != 0.0 { Leg::Lag } else { Leg::None };
    let i0 = if open {
        // Rung 52's floor on the clipped fuel.
        if 1e-9 > u0 {
            u0 = 1e-9;
        }
        ft.try_instant_fuel(flight, a, b, u0).map_err(fail(GovCall::AtApplied, u0))?
    } else {
        match (u0 > 0.0).then(|| ft.try_instant_fuel(flight, a, b, u0)) {
            Some(Ok(x)) => x,
            // Below the leanest fuel that runs (only a cut can put it there): the minimum-fuel stop holds it at the wall.
            _ if u0 < mf_dem => {
                ft.try_instant_fuel(flight, a, b, mf_dem).map_err(fail(GovCall::AtDemand, mf_dem))?;
                u0 = gv::wall(ft, flight, a, b, mf_dem, u0.max(0.0));
                leg = Leg::MinFuel;
                ft.try_instant_fuel(flight, a, b, u0).map_err(fail(GovCall::AtApplied, u0))?
            }
            Some(Err(e)) => return Err(fail(GovCall::AtDemand, u0)(e)),
            None => return Err(GovFailure { message: "rung-107: the governor's demand is at or below zero fuel".into(),
                                            call: GovCall::Governor, mf: mf_dem }),
        }
    };
    let mut caps: Vec<(f64, Leg)> = Vec::new();
    if let Some(t4) = lim.tt4_max {
        if i0.base.tt4 > t4 {
            caps.push((ft.try_topping_fuel(flight, a, b, t4, u0).map_err(fail(GovCall::Redline, u0))?, Leg::Redline));
        }
    }
    if let Some(over) = metal {
        if over(&i0.base) > 0.0 {
            caps.push((gv::try_metal_fuel(ft, flight, a, b, u0, over).map_err(fail(GovCall::Metal, u0))?, Leg::Metal));
        }
    }
    let mut mf = u0;
    for &(c, l) in &caps {
        if c < mf {
            mf = c;
            leg = l;
        }
    }
    let inst = if mf == u0 { i0 } else { ft.try_instant_fuel(flight, a, b, mf).map_err(fail(GovCall::AtApplied, mf))? };
    // The target, off the DEMAND — each leg off the demand, or off the solvable ceiling where it has no operating point
    // (rung 52's `required`, 2026-10-10's helper).
    let floor = lim.floor();
    let top = std::cell::OnceCell::new();
    let mut undecided = false;
    let mut lagged: Vec<f64> = Vec::new();
    if let Some(acc) = lim.accel {
        let solve = |x: f64| ft.try_sched_fuel(flight, a, b, x, acc);
        match ft.try_leg_gated(flight, a, b, mf_dem, &top, &solve, lim.below_ceiling).map_err(fail(GovCall::Accel, mf_dem))? {
            Some(c) => lagged.push(c),
            None => undecided = true,
        }
    }
    if let Some(f) = floor.as_ref() {
        let solve = |x: f64| ft.try_surge_fuel(flight, a, b, x, f);
        match ft.try_leg_gated(flight, a, b, mf_dem, &top, &solve, lim.below_ceiling).map_err(fail(GovCall::Floor, mf_dem))? {
            Some(c) => lagged.push(c),
            None => undecided = true,
        }
    }
    if undecided && lagged.is_empty() {
        let e = ft.try_instant_fuel(flight, a, b, mf_dem).err().expect("an undecided leg means the demand's instant failed");
        return Err(fail(GovCall::AtDemand, mf_dem)(e));
    }
    let required = match lagged.split_first() {
        None => 0.0,
        Some((&first, rest)) => {
            let mut m = first;
            for &c in rest {
                if c < m {
                    m = c;
                }
            }
            0.0f64.max(mf_dem - m)
        }
    };
    let dg = (required - g) / lag.tau(required, g);
    Ok(ReleaseDer { da: inst.base.phi_lp_dot / ft.rho(), db: inst.base.phi_hp_dot, dg, mf_dem, clamp, required, mf, leg,
                    inst })
}

/// RUNG 107. The rest of one RK4 step from `(a, b, g)` at `s`, given its opening slope `k1` — the march's own stages and
/// arithmetic (rung 52's).
#[allow(clippy::too_many_arguments)]
pub fn try_rk_step(
    ft: &FuelTransientCore, flight: &FlightCondition, dem: Demand<'_>, lim: &FuelLimiters<'_>,
    metal: Option<&dyn Fn(&Instant2) -> f64>, a: f64, b: f64, g: f64, s: f64, ds: f64, k1: (f64, f64, f64),
) -> Result<(f64, f64, f64), GovFailure> {
    let d = |a: f64, b: f64, g: f64, s: f64| try_der(ft, flight, dem, lim, metal, a, b, g, s).map(|d| (d.da, d.db, d.dg));
    let (k1a, k1b, k1g) = k1;
    let (k2a, k2b, k2g) = d(a + ds / 2.0 * k1a, b + ds / 2.0 * k1b, g + ds / 2.0 * k1g, s + ds / 2.0)?;
    let (k3a, k3b, k3g) = d(a + ds / 2.0 * k2a, b + ds / 2.0 * k2b, g + ds / 2.0 * k2g, s + ds / 2.0)?;
    let (k4a, k4b, k4g) = d(a + ds * k3a, b + ds * k3b, g + ds * k3g, s + ds)?;
    Ok((a + ds / 6.0 * (k1a + 2.0 * k2a + 2.0 * k3a + k4a), b + ds / 6.0 * (k1b + 2.0 * k2b + 2.0 * k3b + k4b),
        g + ds / 6.0 * (k1g + 2.0 * k2g + 2.0 * k3g + k4g)))
}

/// RUNG 107. The march on three states `(ν_L, ν_H, g)`, the lag starting at 0 — rung 95's loop (`s` accumulated, the
/// step count `round_ties_even(s_end/ds)`, a failed evaluation ends the run), each point recording the demand as
/// `mf_sched`, the applied fuel as `mf`, and the lag and its target ([`PointExtra::Asym`], rung 52's two keys).
#[allow(clippy::too_many_arguments)]
pub fn march(
    ft: &FuelTransientCore, flight: &FlightCondition, dem: Demand<'_>, lim: &FuelLimiters<'_>,
    metal: Option<&dyn Fn(&Instant2) -> f64>, nu0: (f64, f64), s_end: f64, ds: f64,
) -> (Vec<FuelPoint>, Vec<GovMark>) {
    let (mut pts, mut marks) = (Vec::new(), Vec::new());
    let ((mut a, mut b), mut g) = (nu0, 0.0f64);
    let mut s = 0.0f64;
    let n_steps = (s_end / ds).round_ties_even() as i64;
    for _ in 0..=n_steps {
        let Ok(d) = try_der(ft, flight, dem, lim, metal, a, b, g, s) else { break };
        pts.push(point(s, a, b, &d.inst, d.mf, d.mf_dem, PointExtra::Asym { g, required: d.required }));
        marks.push(GovMark { clamp: d.clamp, leg: d.leg });
        let Ok((a1, b1, g1)) = try_rk_step(ft, flight, dem, lim, metal, a, b, g, s, ds, (d.da, d.db, d.dg)) else { break };
        (a, b, g) = (a1, b1, g1);
        s += ds;
    }
    (pts, marks)
}
