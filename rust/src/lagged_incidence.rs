//! RUNG 88 — **the SLOW valve on a moving target: does a lag's cost depend on the wall it watches?**
//!
//! `docs/rung88-spec.md`; the anchor, with every derivation and pre-registered bar, is
//! `docs/plans/rung88-anchor-lagged-incidence.md` (§ refs below are to it).
//!
//! # What it adds
//!
//! No new law. Rung 87's incidence valve ([`ValveReference::Incidence`](crate::limited_bleed::ValveReference))
//! given rung 65's first-order lag: the DEVICE edits are in [`crate::limited_bleed`] (`lagged` keeps
//! the reference) and [`crate::lagged_bleed`] (the march's command is solved against the valve's
//! own target). This module is the READING: one lagged run with D1's ledger ([`lag_read`]), and the
//! `φ` valve MATCHED, at the same `τ`, to a given incidence protection ([`matched_phi_lagged`]).
//!
//! # D1, the ledger every reading here carries
//!
//! Integrating `db/ds = (b_cmd − b)/τ` over a run: `∫(b − b_cmd) ds = τ·(b(0) − b(end))`, exact in
//! RK4's stage quadrature, to the trapezoid's O(ds²) on recorded points, and broken only by the
//! position clamp. So against the instantaneous valve the lag's bleed bill is `τ·(b0 − b_end)` plus
//! the DRIFT — the command integral re-read along the lagged trajectory, which is where a MOVING
//! target (the incidence valve's, through `v`) can enter.
//!
//! # Disciplines
//!
//! * Bleed on a lagged march is the RECORDED position (rung 65: a lagged position carries history);
//!   on an instantaneous one the RE-SOLVED position (rung 64 § 5.22 (ii)) — both through the machine's
//!   own `b_at_point`.
//! * Integrals compared across two runs are taken over the stretch BOTH reach ([`trapz`] with `n`):
//!   a lagged run may stop earlier, and rung 65 § 3 compared totals of unequal runs.
//! * The clamp count is read around each march ([`Census65`]); D1 holds only where it is zero.

use crate::bleed_transient::LeverArm;
use crate::engine::FlightCondition;
use crate::fuel_transient::{FuelPoint, PointExtra};
use crate::lagged_bleed::Census65;
use crate::limited_bleed::{BleedLimiter, ValveReference};
use crate::spool::{try_illinois, ILLINOIS_MAXIT};
use crate::stator_transient::{Ramp, ScheduledStatorCore};
use crate::three_levers::march;
use crate::valve_reference::{read, sibling, ValveRun};

/// The trapezoid of `f(p)` over the first `n` points of `traj` (all of them when `n` ≥ its length).
pub fn trapz(traj: &[FuelPoint], n: usize, f: impl Fn(&FuelPoint) -> f64) -> f64 {
    let n = n.min(traj.len());
    let mut acc = 0.0;
    for i in 1..n {
        acc += 0.5 * (traj[i].s - traj[i - 1].s) * (f(&traj[i]) + f(&traj[i - 1]));
    }
    acc
}

/// The recorded `(b, b_cmd)` of a lagged march point. Panics on a point from another integrator —
/// rung 65's refusal, for the same reason.
pub fn valve_at(p: &FuelPoint) -> (f64, f64) {
    match p.extra {
        PointExtra::Valve { b, b_cmd } => (b, b_cmd),
        _ => panic!("rung-88 reads a LAGGED march: this point did not record the valve"),
    }
}

/// One lagged march, in rung 87's currencies plus D1's ledger.
#[derive(Clone, Debug)]
pub struct LagRead {
    pub run: ValveRun,
    /// `b(0)` — rung 65's convention puts it at `b_cmd(0)`, and D1 bills the lag `τ` times it.
    pub b0: f64,
    pub b_end: f64,
    /// `∫b ds` and `∫b_cmd ds` over the whole run (trapezoid on recorded points).
    pub int_b: f64,
    pub int_b_cmd: f64,
    /// `(∫b − ∫b_cmd) − τ·(b0 − b_end)`: D1's residual — the trapezoid's error where `clamp_hits` = 0.
    pub identity_residual: f64,
    /// Times the position clamp bit during the march (D1's precondition: zero).
    pub clamp_hits: u64,
    /// Whether the command was RISING across the grid-min `M_i` point (H4): `b_cmd` at its later
    /// neighbour above its earlier one. `None` at an end point.
    pub cmd_rising_at_m_i: Option<bool>,
}

/// March a LAGGED machine and read it. Panics if the machine's valve has no lag.
pub fn run_lagged(m: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp)
    -> (Vec<FuelPoint>, LagRead) {
    let tau = m.fuel.inner.lever.lim.and_then(|l| l.tau)
        .expect("rung-88 run_lagged needs a machine carrying a LAGGED valve");
    Census65::reset();
    let (traj, _) = march(m, flight, ramp, None);
    let clamp_hits = Census65::take().clamp_hits;
    let r = lag_read(m, flight, &traj, tau, clamp_hits);
    (traj, r)
}

/// Read a lagged march already made (`clamp_hits` as counted around it).
pub fn lag_read(m: &ScheduledStatorCore, flight: &FlightCondition, traj: &[FuelPoint], tau: f64,
                clamp_hits: u64) -> LagRead {
    let run = read(m, flight, traj);
    let nan = f64::NAN;
    if traj.is_empty() {
        return LagRead { run, b0: nan, b_end: nan, int_b: nan, int_b_cmd: nan,
                         identity_residual: nan, clamp_hits, cmd_rising_at_m_i: None };
    }
    let n = traj.len();
    let (b0, _) = valve_at(&traj[0]);
    let (b_end, _) = valve_at(&traj[n - 1]);
    let int_b = trapz(traj, n, |p| valve_at(p).0);
    let int_b_cmd = trapz(traj, n, |p| valve_at(p).1);
    let j = traj.iter().position(|p| p.s == run.s_at_m_i).expect("the M_i point is a march point");
    let cmd_rising_at_m_i = if j > 0 && j + 1 < n {
        Some(valve_at(&traj[j + 1]).1 > valve_at(&traj[j - 1]).1)
    } else {
        None
    };
    LagRead { run, b0, b_end, int_b, int_b_cmd,
              identity_residual: (int_b - int_b_cmd) - tau * (b0 - b_end), clamp_hits,
              cmd_rising_at_m_i }
}

/// The `φ` valve beside `stator`, with lag `tau` (`None` = rung 87's instantaneous valve), MATCHED so
/// its run-minimum `M_i` equals `m_target` (anchor § 3, H5): Illinois over `φ_lim` between the shut
/// run's `min φ` and 1.5. Returns `None` where those two do not bracket — reported, never scored. The
/// lagged valve's `M_i(φ_lim)` is non-decreasing with a flat top (anchor G5), so a bracketed target
/// below the top has one root.
#[allow(clippy::too_many_arguments)]
pub fn matched_phi_lagged(
    bare: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, stator: Option<&LeverArm>,
    b_max: f64, tau: Option<f64>, m_target: f64, tol_phi: f64,
) -> Option<(f64, Vec<FuelPoint>, ValveRun)> {
    matched_lagged(bare, flight, ramp, stator, b_max, tau, ValveReference::Phi, m_target, tol_phi)
}

/// [`matched_phi_lagged`] for either reference: the valve watching `reference`, its set point `φ_lim`
/// (for the incidence valve, its target at `v = 0`) solved so its run-minimum `M_i` equals `m_target`.
/// The spec's equal-REQUIREMENT table re-tunes BOTH slow valves to rung 87's protection this way.
#[allow(clippy::too_many_arguments)]
pub fn matched_lagged(
    bare: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, stator: Option<&LeverArm>,
    b_max: f64, tau: Option<f64>, reference: ValveReference, m_target: f64, tol_phi: f64,
) -> Option<(f64, Vec<FuelPoint>, ValveRun)> {
    let valve = |phi_lim: f64| {
        let v = BleedLimiter::with_tau(phi_lim, b_max, tau);
        match reference { ValveReference::Phi => v, ValveReference::Incidence => v.on_incidence() }
    };
    let mk = |phi_lim: f64| sibling(bare, stator, Some(valve(phi_lim)));
    let at = |phi_lim: f64| -> (Vec<FuelPoint>, ValveRun) {
        let m = mk(phi_lim);
        let (traj, _) = march(&m, flight, ramp, None);
        let r = read(&m, flight, &traj);
        (traj, r)
    };
    let shut = sibling(bare, stator, None);
    let lo = read(&shut, flight, &march(&shut, flight, ramp, None).0).min_phi;
    let hi = 1.5;
    let (flo, fhi) = (at(lo).1.m_i - m_target, at(hi).1.m_i - m_target);
    if !(flo < 0.0 && fhi > 0.0) {
        return None;
    }
    let phi = try_illinois(|x| Ok(at(x).1.m_i - m_target), lo, hi, flo, fhi, tol_phi, ILLINOIS_MAXIT)
        .expect("infallible residual");
    let (traj, r) = at(phi);
    Some((phi, traj, r))
}
