//! RUNG 108 — THE TWO-SPEED TEMPERATURE LIMITER: rung 52's fast-cut / slow-give-back lag on rung 47's redline clip,
//! off the governor's demand (rung 106's route) or the open ramp's (rung 47's).
//!
//! Anchor (the device and every pre-registered bar, committed before any of this code existed):
//! `docs/plans/rung108-anchor-two-speed-limiter.md`. Spec: `docs/rung108-spec.md`.
//!
//! # One constant becomes two
//!
//! Rung 106 carries the redline's cut as a third state `g` lagging the cut the instant limiter would make, `r`, with
//! one time constant. Here the lag reads rung 52's pair — fast to cut, slow to give back:
//!
//! ```text
//! r     = G − T(ν; Tt4_max)   if Tt4 at G is above the redline, else 0     (rung 106's, unchanged)
//! dg/ds = (r − g)/τ_att       if r > g        (fast to cut)
//!         (r − g)/τ_rel       otherwise       (slow to give back)
//! u     = min(G − g, the other armed legs solved off G)                     (rung 106's, unchanged)
//! ```
//!
//! Everything but `dg/ds` is rung 106's own evaluation ([`gl::try_der`]), called as it is; only the rate is re-taken
//! with [`AsymmetricLag::tau`] (the tie `r = g` takes the release constant — continuous there, the error being 0).
//!
//! # Reduce
//!
//! * Equal constants ⇒ rung 106's march (governed) and rung 47's (the open ramp as the [`Demand`]), bit for bit: `tau`
//!   only SELECTS, so the rate is the same division (`tests/rung108.rs`).
//! * The pinned trigger (rung 52 § 1): `τ_rel` is never read while `r > g`, so every point before the first crossing is
//!   rung 106's at `τ = τ_att`, for any give-back time.
//! * The ledger splits by phase (anchor § 1, D1): `∫_attack (r − g) ds = τ_att·g_c`, `∫_release (g − r) ds =
//!   τ_rel·(g_c − g_end)`.

use crate::engine::FlightCondition;
use crate::fuel_transient::{point, AsymmetricLag, FuelLimiters, FuelPoint, FuelTransientCore, PointExtra};
use crate::governor::{GovFailure, GovMark};
use crate::governor_lag::{self as gl, ClipLag, Demand, LagDer};
use crate::two_spool_transient::Instant2;

/// The two-speed clip: the redline and rung 52's pair of time constants, in HP spool time constants.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TwoSpeedClip {
    pub tt4_max: f64,
    pub lag: AsymmetricLag,
}

impl TwoSpeedClip {
    /// Rung 106's clip at the cutting constant — everything its evaluation reads but the rate.
    fn at_attack(self) -> ClipLag { ClipLag { tt4_max: self.tt4_max, tau: self.lag.tau_att } }
}

/// RUNG 108. One derivative evaluation at the clip `g`: rung 106's ([`gl::try_der`]), the clip's rate on the active
/// constant.
#[allow(clippy::too_many_arguments)]
pub fn try_der(
    ft: &FuelTransientCore, flight: &FlightCondition, dem: Demand<'_>, clip: TwoSpeedClip, lim: &FuelLimiters<'_>,
    metal: Option<&dyn Fn(&Instant2) -> f64>, a: f64, b: f64, g: f64, s: f64,
) -> Result<LagDer, GovFailure> {
    let mut d = gl::try_der(ft, flight, dem, clip.at_attack(), lim, metal, a, b, g, s)?;
    d.dg = (d.required - g) / clip.lag.tau(d.required, g);
    Ok(d)
}

/// RUNG 108. The rest of one RK4 step from `(a, b, g)` at `s`, given its opening slope `k1` — rung 106's stages and
/// arithmetic.
#[allow(clippy::too_many_arguments)]
pub fn try_rk_step(
    ft: &FuelTransientCore, flight: &FlightCondition, dem: Demand<'_>, clip: TwoSpeedClip, lim: &FuelLimiters<'_>,
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

/// RUNG 108. The march on three states `(ν_L, ν_H, g)`, the clip starting at 0 — rung 106's loop, each point recording
/// the demand as `mf_sched`, the applied fuel as `mf`, and the clip and its target ([`PointExtra::Asym`]).
#[allow(clippy::too_many_arguments)]
pub fn march(
    ft: &FuelTransientCore, flight: &FlightCondition, dem: Demand<'_>, clip: TwoSpeedClip, lim: &FuelLimiters<'_>,
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
