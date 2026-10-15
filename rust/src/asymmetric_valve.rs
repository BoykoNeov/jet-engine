//! RUNG 89 — **the ASYMMETRIC valve: which edge decides who is cheaper?**
//!
//! `docs/rung89-spec.md`; the anchor, with every derivation and pre-registered bar, is
//! `docs/plans/rung89-anchor-asymmetric-valve.md` (§ refs below are to it).
//!
//! # What it adds
//!
//! No new law and no new state. Rung 65's first-order lag given a SECOND time constant: the valve
//! chases its command at `τ_open` while the command is at or above the position and at `τ_close`
//! below it. The DEVICE is [`BleedLimiter::closing`] and [`BleedLimiter::lag_rate`] (in
//! [`crate::limited_bleed`]), read by rung 65's march; `tau_close = None` is rung 65's valve, bit for
//! bit. This module is the READING: a valve family with both constants ([`asym_valve`]), the
//! equal-requirement re-tune on it ([`matched_asym`]), and the `φ` valve's FLOOR ([`phi_floor`]) —
//! the lowest set point any closing speed can buy it (anchor D5).
//!
//! # The two facts every reading here rests on
//!
//! * **D2 — the closing constant is not read before the first stage that closes.** The march is bit
//!   for bit independent of `τ_close` up to the first RK4 stage that finds the command below the
//!   position ([`first_closing_read`]). A run-minimum recorded before that mark is out of its reach.
//! * **D1′ — the bill, edge by edge.** `∫(b − b_cmd) ds = τ_close·D − τ_open·U` (`U`, `D` the total
//!   opening and closing travel, [`LagRead::rise`](crate::lagged_incidence::LagRead) /
//!   `fall`): a slow closing edge ADDS `τ_close·D` of air, a slow opening edge TAKES `τ_open·U` away.
//!
//! # Disciplines
//!
//! Rung 88's: bleed is the RECORDED position, integrals are compared over the stretch both runs
//! reach, and the clamp count is read around each march (D1′ holds only where it is zero).

use crate::bleed_transient::LeverArm;
use crate::engine::FlightCondition;
use crate::fuel_transient::FuelPoint;
use crate::lagged_bleed::{first_closing_read, Census65};
use crate::lagged_incidence::matched_with;
use crate::limited_bleed::{BleedLimiter, ValveReference};
use crate::spool::{try_illinois, ILLINOIS_MAXIT};
use crate::stator_transient::{Ramp, ScheduledStatorCore};
use crate::three_levers::march;
use crate::two_spool::Spool;
use crate::valve_reference::{read, sibling, ValveRun};

/// The valve watching `reference` at set point `phi_lim`, opening at `tau_open` and closing at
/// `tau_close`. Equal constants build the SYMMETRIC valve (`tau_close = None`, rung 65's object), not
/// `closing(τ)` — so the grid's diagonal IS rung 88's, through rung 88's code.
pub fn asym_valve(phi_lim: f64, b_max: f64, tau_open: f64, tau_close: f64, reference: ValveReference)
    -> BleedLimiter {
    let v = BleedLimiter::with_tau(phi_lim, b_max, Some(tau_open));
    let v = if tau_close == tau_open { v } else { v.closing(tau_close) };
    match reference { ValveReference::Phi => v, ValveReference::Incidence => v.on_incidence() }
}

/// The asymmetric valve re-tuned so its run-minimum incidence margin `M_i` equals `m_target` (rung 88
/// § 4's currency): Illinois over `φ_lim` between the shut run's `min φ` and 1.5. `None` where those
/// do not bracket.
#[allow(clippy::too_many_arguments)]
pub fn matched_asym(
    bare: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, stator: Option<&LeverArm>,
    b_max: f64, tau_open: f64, tau_close: f64, reference: ValveReference, m_target: f64, tol_phi: f64,
) -> Option<(f64, Vec<FuelPoint>, ValveRun)> {
    matched_with(bare, flight, ramp, stator, |x| asym_valve(x, b_max, tau_open, tau_close, reference),
                 m_target, tol_phi)
}

/// One march of `m` with the census reset around it: the trajectory, the march coordinate of the
/// first closing read ([`first_closing_read`], `+∞` if none) and the clamp count.
pub fn march_marked(m: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp)
    -> (Vec<FuelPoint>, f64, u64) {
    Census65::reset();
    let (traj, _) = march(m, flight, ramp, None);
    let fc = first_closing_read();
    let clamps = Census65::take().clamp_hits;
    (traj, fc, clamps)
}

/// The incidence margin `M_i = T_c − 1/φ_LP + v` at each point of a march of `m`.
pub fn margins(m: &ScheduledStatorCore, flight: &FlightCondition, traj: &[FuelPoint]) -> Vec<f64> {
    let (tt2, _, _) = m.fuel.inner.inlet(flight);
    let t_c = m.design_map(Spool::Lp).tan_beta1_crit();
    traj.iter().map(|p| t_c - 1.0 / p.phi_lp + m.v_of(Spool::Lp, p.nu_lp, p.nu_hp, Some(tt2))).collect()
}

/// The minimum of `M_i` over the points recorded BEFORE `mark` (D2's untouched stretch).
pub fn min_before(margins: &[f64], traj: &[FuelPoint], mark: f64) -> f64 {
    traj.iter().zip(margins).filter(|(p, _)| p.s < mark).map(|(_, &m)| m).fold(f64::INFINITY, f64::min)
}

/// RUNG 89's floor (anchor G4/D5): the SYMMETRIC `φ` valve at lag `tau` re-tuned until the minimum of
/// `M_i` over the points before its first closing read is `m_target` — the lowest set point any
/// closing speed can buy it, since that stretch is the same at every `τ_close` (D2) and a slower close
/// only adds margin after it. Returns `(φ_lim, residual)`: the residual is read back at the root
/// because the mark moves in grid steps with the set point, so a bracketing solve can land on a jump
/// rather than a root (rung 83). `None` where the bracket fails.
#[allow(clippy::too_many_arguments)]
pub fn phi_floor(
    bare: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, stator: Option<&LeverArm>,
    b_max: f64, tau: f64, m_target: f64, tol_phi: f64,
) -> Option<(f64, f64)> {
    let f = |x: f64| {
        let m = sibling(bare, stator, Some(BleedLimiter::with_tau(x, b_max, Some(tau))));
        let (t, fc, _) = march_marked(&m, flight, ramp);
        min_before(&margins(&m, flight, &t), &t, fc) - m_target
    };
    let shut = sibling(bare, stator, None);
    let lo = read(&shut, flight, &march(&shut, flight, ramp, None).0).min_phi;
    let hi = 1.5;
    let (flo, fhi) = (f(lo), f(hi));
    if !(flo < 0.0 && fhi > 0.0) {
        return None;
    }
    let x = try_illinois(|x| Ok(f(x)), lo, hi, flo, fhi, tol_phi, ILLINOIS_MAXIT).expect("infallible residual");
    Some((x, f(x)))
}
