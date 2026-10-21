//! RUNG 106 — THE GOVERNOR BESIDE THE SLOW TEMPERATURE LIMITER: rung 95's governor as the demand rung 47's lagged
//! redline clip is cut from.
//!
//! Anchor (the device and every pre-registered bar, committed before any of this code existed):
//! `docs/plans/rung106-anchor-governor-lag.md`. Spec: `docs/rung106-spec.md`.
//!
//! # The clip is measured off the governor's demand
//!
//! Rung 47 carries the redline's cut as a THIRD state `g` lagging the cut the instant limiter would make, `r`, off the
//! open ramp; rung 95 replaced the ramp by a governor and refused the lag. Here the governor's fuel `G(ν, s)` (rung
//! 95/96's laws, unchanged and stateless) is the demand:
//!
//! ```text
//! r     = G − T(ν; Tt4_max)   if Tt4 at G is above the redline, else 0     (rung 46/47's guard, off the DEMAND)
//! dg/ds = (r − g)/τ
//! u     = min(G − g, the other armed legs solved off G)                     (rung 47's pattern)
//! ```
//!
//! The governor holds no state, but `g` does — so rung 95's "nothing winds up" holds only for the governor itself:
//! the cut stored while the demand rose is paid back when it falls (the clip's ledger, `∫(r − g) ds = τ·Δg`).
//!
//! # The minimum-fuel stop (the user's choice)
//!
//! An applied fuel below the leanest the engine runs on is HELD at that wall and marked ([`Leg::MinFuel`]) — the
//! leftover cut of a governor that has dived. On the page only a start above the redline reached it (anchor G4), and
//! that start is refused there; the hold stays as the march's safety net.
//!
//! # Reduce
//!
//! The demand is a [`Demand`]: the open ramp handed in instead of the governor gives rung 47's march, bit for bit
//! where the ramp's instant runs (`tests/rung106.rs`). Before the clip first engages the run IS rung 95's.

use crate::engine::FlightCondition;
use crate::fuel_transient::{point, FuelInstant, FuelLimiters, FuelPoint, FuelTransientCore, PointExtra};
use crate::gas::Abort;
use crate::governor::{self as gv, Clamp, GovCall, GovFailure, GovMark, Governor, Leg};
use crate::two_spool_transient::Instant2;

/// Rung 47's lagged clip: the redline and the lag, in HP spool time constants.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClipLag {
    pub tt4_max: f64,
    pub tau: f64,
}

/// What the clip is cut from.
#[derive(Clone, Copy)]
pub enum Demand<'a> {
    /// Rung 95/96's governor — the rung.
    Governor(&'a Governor),
    /// The open ramp's fuel at `s` — rung 47's own march, the gate's second code path.
    Ramp(&'a dyn Fn(f64) -> f64),
}

impl Demand<'_> {
    /// The demanded fuel at a state and where it sits (the ramp is always on its "root").
    fn try_fuel(&self, ft: &FuelTransientCore, flight: &FlightCondition, a: f64, b: f64, s: f64)
                -> Result<(f64, Clamp), Abort> {
        match self {
            Demand::Governor(g) => gv::try_governor_fuel(ft, flight, g, a, b, s),
            Demand::Ramp(f) => Ok((f(s), Clamp::Root)),
        }
    }
}

/// One derivative evaluation.
#[derive(Clone, Debug)]
pub struct LagDer {
    pub da: f64,
    pub db: f64,
    pub dg: f64,
    /// The demand the clip and the legs cut, and where it sits.
    pub mf_dem: f64,
    pub clamp: Clamp,
    /// The cut the instant limiter would make here (`r`).
    pub required: f64,
    /// The applied fuel and what set it.
    pub mf: f64,
    pub leg: Leg,
    pub inst: FuelInstant,
}

/// RUNG 106. One derivative evaluation at the clip `g`: the demand, the instant there, the clip's target off it,
/// `G − g` against each armed instant leg (schedule, floor, metal — solved off the DEMAND, as rung 47's), the minimum,
/// held at the lean wall if it does not run. The first of equal cuts wins, in the order clip, schedule, floor, metal.
#[allow(clippy::too_many_arguments)]
pub fn try_der(
    ft: &FuelTransientCore, flight: &FlightCondition, dem: Demand<'_>, clip: ClipLag, lim: &FuelLimiters<'_>,
    metal: Option<&dyn Fn(&Instant2) -> f64>, a: f64, b: f64, g: f64, s: f64,
) -> Result<LagDer, GovFailure> {
    ft.inner.inner.base.light_at(s);   // RUNG 102: the afterburner as lit at this march time
    assert!(lim.freeze.is_none() && lim.tt4_max.is_none() && lim.tau_gov.is_none() && lim.lag.is_none()
                && lim.s_off.is_none() && lim.tau_rel.is_none(),
            "rung-106: the redline rides in the clip; the other legs are rung 43's plain ones");
    let fail = |call: GovCall, mf: f64| move |e: Abort| GovFailure { message: e.0, call, mf };
    let (mf_dem, clamp) = dem.try_fuel(ft, flight, a, b, s).map_err(fail(GovCall::Governor, f64::NAN))?;
    let i = ft.try_instant_fuel(flight, a, b, mf_dem).map_err(fail(GovCall::AtDemand, mf_dem))?;
    let required = if i.base.tt4 > clip.tt4_max {
        mf_dem - ft.try_topping_fuel(flight, a, b, clip.tt4_max, mf_dem).map_err(fail(GovCall::Redline, mf_dem))?
    } else {
        0.0
    };
    let (mut mf, mut leg) = (mf_dem - g, if g != 0.0 { Leg::Lag } else { Leg::None });
    // The lowest fuel KNOWN to run — where the lean hold's bisection starts.
    let mut lowest = mf_dem;
    let floor = lim.floor();
    let mut caps: Vec<(f64, Leg)> = Vec::new();
    if let Some(acc) = lim.accel {
        caps.push((ft.try_sched_fuel(flight, a, b, mf_dem, acc).map_err(fail(GovCall::Accel, mf_dem))?, Leg::Accel));
    }
    if let Some(f) = floor.as_ref() {
        caps.push((ft.try_surge_fuel(flight, a, b, mf_dem, f).map_err(fail(GovCall::Floor, mf_dem))?, Leg::Floor));
    }
    if let Some(over) = metal {
        if over(&i.base) > 0.0 {
            caps.push((gv::try_metal_fuel(ft, flight, a, b, mf_dem, over).map_err(fail(GovCall::Metal, mf_dem))?,
                       Leg::Metal));
        }
    }
    for &(c, l) in &caps {
        lowest = lowest.min(c);
        if c < mf {
            mf = c;
            leg = l;
        }
    }
    let inst = match leg {
        Leg::None => i,
        _ => match (mf > 0.0).then(|| ft.try_instant_fuel(flight, a, b, mf)) {
            Some(Ok(x)) => x,
            // Below the leanest fuel that runs: the minimum-fuel stop holds it at the wall.
            _ => {
                mf = gv::wall(ft, flight, a, b, lowest, mf.max(0.0));
                leg = Leg::MinFuel;
                ft.try_instant_fuel(flight, a, b, mf).map_err(fail(GovCall::AtApplied, mf))?
            }
        },
    };
    Ok(LagDer { da: inst.base.phi_lp_dot / ft.rho(), db: inst.base.phi_hp_dot, dg: (required - g) / clip.tau,
                mf_dem, clamp, required, mf, leg, inst })
}

/// RUNG 106. The rest of one RK4 step from `(a, b, g)` at `s`, given its opening slope `k1` — the march's own stages
/// and arithmetic (rung 47's).
#[allow(clippy::too_many_arguments)]
pub fn try_rk_step(
    ft: &FuelTransientCore, flight: &FlightCondition, dem: Demand<'_>, clip: ClipLag, lim: &FuelLimiters<'_>,
    metal: Option<&dyn Fn(&Instant2) -> f64>, a: f64, b: f64, g: f64, s: f64, ds: f64, k1: (f64, f64, f64),
) -> Result<(f64, f64, f64), GovFailure> {
    let d = |a: f64, b: f64, g: f64, s: f64| try_der(ft, flight, dem, clip, lim, metal, a, b, g, s).map(|d| (d.da, d.db, d.dg));
    let (k1a, k1b, k1g) = k1;
    let (k2a, k2b, k2g) = d(a + ds / 2.0 * k1a, b + ds / 2.0 * k1b, g + ds / 2.0 * k1g, s + ds / 2.0)?;
    let (k3a, k3b, k3g) = d(a + ds / 2.0 * k2a, b + ds / 2.0 * k2b, g + ds / 2.0 * k2g, s + ds / 2.0)?;
    let (k4a, k4b, k4g) = d(a + ds * k3a, b + ds * k3b, g + ds * k3g, s + ds)?;
    Ok((a + ds / 6.0 * (k1a + 2.0 * k2a + 2.0 * k3a + k4a), b + ds / 6.0 * (k1b + 2.0 * k2b + 2.0 * k3b + k4b),
        g + ds / 6.0 * (k1g + 2.0 * k2g + 2.0 * k3g + k4g)))
}

/// RUNG 106. The march on three states `(ν_L, ν_H, g)`, the clip starting at 0 — rung 95's loop (`s` accumulated, the
/// step count `round_ties_even(s_end/ds)`, a failed evaluation ends the run), each point recording the demand as
/// `mf_sched`, the applied fuel as `mf`, and the clip and its target ([`PointExtra::Asym`], rung 52's two keys).
#[allow(clippy::too_many_arguments)]
pub fn march(
    ft: &FuelTransientCore, flight: &FlightCondition, dem: Demand<'_>, clip: ClipLag, lim: &FuelLimiters<'_>,
    metal: Option<&dyn Fn(&Instant2) -> f64>, nu0: (f64, f64), s_end: f64, ds: f64,
) -> (Vec<FuelPoint>, Vec<GovMark>) {
    let (mut pts, mut marks) = (Vec::new(), Vec::new());
    let ((mut a, mut b), mut g) = (nu0, 0.0f64);
    let mut s = 0.0f64;
    let n_steps = (s_end / ds).round_ties_even() as i64;
    for _ in 0..=n_steps {
        let Ok(d) = try_der(ft, flight, dem, clip, lim, metal, a, b, g, s) else { break };
        pts.push(point(s, a, b, &d.inst, d.mf, d.mf_dem, PointExtra::Asym { g, required: d.required }));
        marks.push(GovMark { clamp: d.clamp, leg: d.leg });
        let Ok((a1, b1, g1)) = try_rk_step(ft, flight, dem, clip, lim, metal, a, b, g, s, ds, (d.da, d.db, d.dg))
        else { break };
        (a, b, g) = (a1, b1, g1);
        s += ds;
    }
    (pts, marks)
}
