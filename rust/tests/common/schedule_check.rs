//! **WHY A FLOORED RUNG-57 MARCH STOPPED (2026-10-14)** — rung 63's floor table re-derived with
//! `FuelLimiters::below_ceiling` on (`docs/rung63-spec.md` § 3).
//!
//! The plain rung-43 march (`r43_integrate_fuel`) ends at the first step with no answer and says
//! nothing about which call failed. This re-runs that step — the last recorded point's RK4 step,
//! then the next step's opening evaluation — through the public pieces the march calls, in its
//! order and arithmetic, for the ONE leg the floor table arms (a `phi` floor, no release):
//!
//! * `try_instant_fuel` at the SCHEDULED fuel — failing there with the switch OFF is rung 43's
//!   every-step schedule check ([`Stop::ScheduleCheck`]);
//! * with the switch ON, `r43_cut_below_ceiling` instead — called, not copied;
//! * the floor's own set-point solve `try_surge_fuel`, then the instant at the applied fuel.
//!
//! **The replica is held to the march before it is believed**: its opening evaluation at every
//! recorded point must give that point's applied fuel bit for bit. Classification is by WHICH CALL
//! failed — never by a cause guessed from the trajectory (sandbox plan § 12.10).

use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{r43_cut_below_ceiling, FuelLimiters, FuelPoint, SurgeLimiter};
use turbojet::stator_transient::{Ramp, ScheduledStatorCore};

#[derive(Debug, Clone, PartialEq)]
pub enum Stop {
    /// The march recorded every point.
    Ran,
    /// The instant at the SCHEDULED fuel had no answer and the switch was off.
    ScheduleCheck(String),
    /// Any other call failed — its message, unchanged.
    Other(String),
}

/// `r57_stator_march`'s schedule, line for line.
fn schedule(m: &ScheduledStatorCore, fl: &FlightCondition, ramp: &Ramp) -> impl Fn(f64) -> f64 {
    let mf_lo = m.fuel.fuel_for_tt4(fl, ramp.tt4_lo);
    let mf_hi = m.fuel.fuel_for_tt4(fl, ramp.tt4_hi);
    let r = ramp.r;
    move |s: f64| {
        if s <= 0.0 {
            mf_lo
        } else if s >= r {
            mf_hi
        } else {
            mf_lo + (mf_hi - mf_lo) * (s / r)
        }
    }
}

/// One evaluation of the plain march's `der` with a single `phi` floor: `(dnu_L, dnu_H, mf)`.
fn der(
    m: &ScheduledStatorCore, fl: &FlightCondition, floor: SurgeLimiter, below_ceiling: bool,
    a: f64, b: f64, mf_in: f64, s: f64,
) -> Result<(f64, f64, f64), Stop> {
    let ft = &m.fuel;
    let lim = FuelLimiters { surge: Some(floor), below_ceiling, ..Default::default() };
    let fl_floor = lim.floor();
    let mut mf = mf_in;
    let mut i = match ft.try_instant_fuel(fl, a, b, mf) {
        Ok(i) => i,
        Err(e) if !below_ceiling => return Err(Stop::ScheduleCheck(e.0)),
        Err(e) => {
            let (mf, i) = r43_cut_below_ceiling(ft, fl, a, b, mf, s, &lim, fl_floor.as_ref(), e)
                .map_err(|e| Stop::Other(e.0))?;
            return Ok((i.base.phi_lp_dot / ft.rho(), i.base.phi_hp_dot, mf));
        }
    };
    let cap = ft.try_surge_fuel(fl, a, b, mf, fl_floor.as_ref().expect("armed"))
        .map_err(|e| Stop::Other(e.0))?;
    if cap < mf {
        mf = cap;
        i = ft.try_instant_fuel(fl, a, b, mf).map_err(|e| Stop::Other(e.0))?;
    }
    Ok((i.base.phi_lp_dot / ft.rho(), i.base.phi_hp_dot, mf))
}

/// Why `traj` — a floored `stator_march` on `m` — ended. `npts_full` is what a complete run
/// records. Panics if the replica disagrees with the march, or if the step it re-runs succeeds.
pub fn stop_of(
    m: &ScheduledStatorCore, fl: &FlightCondition, ramp: &Ramp, floor: SurgeLimiter,
    below_ceiling: bool, traj: &[FuelPoint], npts_full: usize,
) -> Stop {
    let sched = schedule(m, fl, ramp);
    assert!(!traj.is_empty(), "a march that records no point is out of this helper's scope");
    for p in traj {
        assert_eq!(sched(p.s).to_bits(), p.mf_sched.to_bits(), "the schedule replica at s={}", p.s);
        let (_, _, mf) = der(m, fl, floor, below_ceiling, p.nu_lp, p.nu_hp, p.mf_sched, p.s)
            .unwrap_or_else(|e| panic!("a recorded point does not evaluate at s={}: {e:?}", p.s));
        assert_eq!(mf.to_bits(), p.mf.to_bits(), "the replica's applied fuel at s={}", p.s);
    }
    if traj.len() == npts_full {
        return Stop::Ran;
    }
    let p = traj.last().unwrap();
    let (a, b, s, ds) = (p.nu_lp, p.nu_hp, p.s, ramp.ds);
    let d = |a: f64, b: f64, mf: f64, s: f64| der(m, fl, floor, below_ceiling, a, b, mf, s);
    let step = || -> Result<(), Stop> {
        let (k1a, k1b, _) = d(a, b, sched(s), s)?;
        let mfm = sched(s + ds / 2.0);
        let (k2a, k2b, _) = d(a + ds / 2.0 * k1a, b + ds / 2.0 * k1b, mfm, s + ds / 2.0)?;
        let (k3a, k3b, _) = d(a + ds / 2.0 * k2a, b + ds / 2.0 * k2b, mfm, s + ds / 2.0)?;
        let (k4a, k4b, _) = d(a + ds * k3a, b + ds * k3b, sched(s + ds), s + ds)?;
        let a2 = a + ds / 6.0 * (k1a + 2.0 * k2a + 2.0 * k3a + k4a);
        let b2 = b + ds / 6.0 * (k1b + 2.0 * k2b + 2.0 * k3b + k4b);
        // The march accumulates `s += ds`; the next opening evaluation is at that `s`.
        let s2 = s + ds;
        d(a2, b2, sched(s2), s2)?;
        Ok(())
    };
    match step() {
        Err(stop) => stop,
        Ok(()) => panic!("the march stopped after s={s} but its next step re-runs cleanly"),
    }
}
