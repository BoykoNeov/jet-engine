//! RUNG 87 — **the CONTROLLED valve beside the stator schedule: what should it watch?**
//!
//! `docs/rung87-spec.md`; the anchor, with every derivation and pre-registered bar, is
//! `docs/plans/rung87-anchor-valve-reference.md` (§ refs below are to it).
//!
//! # What it adds
//!
//! The DEVICE is in [`crate::limited_bleed`]: rung 64's valve with a [`ValveReference`] — it holds
//! either the LP flow coefficient (rungs 64–80) or the LP rotor INCIDENCE, `T_c − 1/φ + v ≥ m_lim`,
//! which is the same solve against `φ_lim/(1 + v·φ_lim)` (`T_c` cancels; `v = 0` is the `φ` valve
//! bit for bit). This module is the READING: one run of either valve, with or without the LP stator
//! schedule and with or without one fuel floor ([`run`]), and the `φ` valve MATCHED to a given
//! incidence protection ([`matched_phi`]) — anchor § 3's equal-protection comparison.
//!
//! # Disciplines (anchor § 0–§ 2)
//!
//! * Every machine is a FRESH sibling of a bare machine built by rung 64's
//!   [`build_limited_bleed`](crate::limited_bleed::build_limited_bleed), its arming ASSERTED: rung
//!   62's builder drops a floor SILENTLY (G1).
//! * Every bleed number is RE-SOLVED at the point ([`ScheduledStatorCore::b_at_point`]), never read
//!   off `b_of` (rung 64 § 5.22 (ii): that returns exactly 0 on a floored march).
//! * The currency is the LP incidence margin `M_i = T_c − 1/φ_LP + v`, read as the GRID minimum over
//!   the march points: on a pinned plateau a parabola refinement measures nothing.
//! * The bill is against the SAME stator face's valve-shut run: the stators move the shaft speed and
//!   the thrust by themselves, and that is not the valve's to pay.

use crate::bleed_transient::LeverArm;
use crate::engine::FlightCondition;
use crate::fuel_transient::FuelPoint;
use crate::limited_bleed::{BleedLimiter, ValveReference};
use crate::spool::{try_illinois, ILLINOIS_MAXIT};
use crate::stator_transient::{Ramp, ScheduledStatorCore};
use crate::three_levers::{march, ArmedLimiter};
use crate::two_spool::Spool;

/// One march read in rung 87's currencies.
#[derive(Clone, Debug)]
pub struct ValveRun {
    /// Points marched; 0 = the run did not start (a fuel floor above the start) — a RESULT.
    pub npts: usize,
    /// Grid-minimum `φ_LP` and grid-minimum `M_i` over the run, with the `s` and `v` at the latter.
    pub min_phi: f64,
    pub m_i: f64,
    pub s_at_m_i: f64,
    pub v_at_m_i: f64,
    /// Bleed, RE-SOLVED at every point: its time integral, peak and last value.
    pub b_int: f64,
    pub b_peak: f64,
    pub b_end: f64,
    /// Thrust (specific thrust × air flow): its time integral and last value.
    pub thrust_int: f64,
    pub thrust_end: f64,
    pub nu_lp_end: f64,
    pub nu_hp_end: f64,
    /// Fuel the fuel floor withheld (scheduled − applied), its time integral and largest value.
    pub fuel_withheld: f64,
    pub fuel_cut_max: f64,
    /// Points with the valve shut / riding / fully open.
    pub shut: usize,
    pub riding: usize,
    pub saturated: usize,
    /// Over the riding points, the largest `|φ − target(v)|` — the valve's own pin (D1), in ITS
    /// coordinate.
    pub pin_dev: f64,
}

/// A fresh sibling of `bare` with the LP stator schedule `stator` (or none) and `valve` (or none),
/// its arming ASSERTED (G1).
pub fn sibling(bare: &ScheduledStatorCore, stator: Option<&LeverArm>, valve: Option<BleedLimiter>)
    -> ScheduledStatorCore {
    assert!(!bare.arming().is_armed() && !bare.armed_bleed(),
            "rung-87 builds every machine from the BARE one: pass the unarmed machine.");
    let empty = LeverArm::default();
    let s = stator.unwrap_or(&empty);
    assert!(!s.arms_valve_floored(), "rung-87: `stator` must carry no valve");
    let arm = match valve {
        Some(l) => LeverArm::merged(s, &LeverArm::floored(l)),
        None => *s,
    };
    let m = bare.at_lever(&arm);
    assert!(m.fuel.inner.lever.lim == valve && m.arming().is_scheduled() == stator.is_some(),
            "rung-87 sibling: asked stator={} valve={valve:?}, the machine carries stator={} \
             valve={:?} -- built through a builder that drops the floor? (G1: use rung 64's \
             build_limited_bleed)", stator.is_some(), m.arming().is_scheduled(), m.fuel.inner.lever.lim);
    m
}

/// March `m` (one fuel floor, or none) and read it.
pub fn run(m: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, lim: Option<&ArmedLimiter>)
    -> ValveRun {
    let (traj, _) = march(m, flight, ramp, lim);
    read(m, flight, &traj)
}

/// Read a march already made on `m`.
pub fn read(m: &ScheduledStatorCore, flight: &FlightCondition, traj: &[FuelPoint]) -> ValveRun {
    let nan = f64::NAN;
    let mut out = ValveRun {
        npts: traj.len(), min_phi: nan, m_i: nan, s_at_m_i: nan, v_at_m_i: nan, b_int: nan,
        b_peak: nan, b_end: nan, thrust_int: nan, thrust_end: nan, nu_lp_end: nan, nu_hp_end: nan,
        fuel_withheld: nan, fuel_cut_max: nan, shut: 0, riding: 0, saturated: 0, pin_dev: nan,
    };
    if traj.is_empty() {
        return out;
    }
    let (tt2, _, _) = m.fuel.inner.inlet(flight);
    let t_c = m.design_map(Spool::Lp).tan_beta1_crit();
    let lim = m.fuel.inner.lever.lim;
    let b: Vec<f64> = traj.iter().map(|p| m.b_at_point(flight, p)).collect();
    let v: Vec<f64> = traj.iter().map(|p| m.v_of(Spool::Lp, p.nu_lp, p.nu_hp, Some(tt2))).collect();
    let (mut min_phi, mut m_i, mut j) = (f64::INFINITY, f64::INFINITY, 0usize);
    let (mut b_int, mut th_int, mut withheld) = (0.0, 0.0, 0.0);
    let (mut b_peak, mut cut_max, mut pin_dev) = (0.0f64, 0.0f64, 0.0f64);
    for (i, p) in traj.iter().enumerate() {
        min_phi = min_phi.min(p.phi_lp);
        let mi = t_c - 1.0 / p.phi_lp + v[i];
        if mi < m_i {
            m_i = mi;
            j = i;
        }
        b_peak = b_peak.max(b[i]);
        cut_max = cut_max.max(p.mf_sched - p.mf);
        if let Some(l) = lim {
            if b[i] <= 0.0 {
                out.shut += 1;
            } else if b[i] >= l.b_max {
                out.saturated += 1;
            } else {
                out.riding += 1;
                pin_dev = pin_dev.max((p.phi_lp - l.target(v[i])).abs());
            }
        }
        if i > 0 {
            let q = &traj[i - 1];
            let h = p.s - q.s;
            b_int += 0.5 * h * (b[i] + b[i - 1]);
            th_int += 0.5 * h * (p.sp_thrust * p.mdot_air + q.sp_thrust * q.mdot_air);
            withheld += 0.5 * h * ((p.mf_sched - p.mf) + (q.mf_sched - q.mf));
        }
    }
    let last = &traj[traj.len() - 1];
    out.min_phi = min_phi;
    out.m_i = m_i;
    out.s_at_m_i = traj[j].s;
    out.v_at_m_i = v[j];
    out.b_int = b_int;
    out.b_peak = b_peak;
    out.b_end = b[b.len() - 1];
    out.thrust_int = th_int;
    out.thrust_end = last.sp_thrust * last.mdot_air;
    out.nu_lp_end = last.nu_lp;
    out.nu_hp_end = last.nu_hp;
    out.fuel_withheld = withheld;
    out.fuel_cut_max = cut_max;
    out.pin_dev = if out.riding > 0 { pin_dev } else { 0.0 };
    out
}

/// The `φ` valve beside `stator`, MATCHED so its run minimum `M_i` equals `m_target` (anchor § 3):
/// Illinois over `φ_lim` between the shut run's `min φ` (the valve never opens: `M_i` = the shut
/// run's, below a riding target) and 1.5 (open to `b_max` throughout: the full-open run's `M_i`).
/// Returns `None` where those two do not bracket `m_target` — reported, never scored.
#[allow(clippy::too_many_arguments)]
pub fn matched_phi(
    bare: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, stator: Option<&LeverArm>,
    b_max: f64, m_target: f64, tol_phi: f64,
) -> Option<(f64, ValveRun)> {
    let at = |phi_lim: f64| run(&sibling(bare, stator, Some(BleedLimiter::new(phi_lim, b_max))), flight, ramp, None);
    let lo = run(&sibling(bare, stator, None), flight, ramp, None).min_phi;
    let hi = 1.5;
    let (flo, fhi) = (at(lo).m_i - m_target, at(hi).m_i - m_target);
    if !(flo < 0.0 && fhi > 0.0) {
        return None;
    }
    let phi = try_illinois(|x| Ok(at(x).m_i - m_target), lo, hi, flo, fhi, tol_phi, ILLINOIS_MAXIT)
        .expect("infallible residual");
    Some((phi, at(phi)))
}

/// The incidence valve at `φ_lim` (its set point at `v = 0`).
pub fn incidence_valve(phi_lim: f64, b_max: f64) -> BleedLimiter {
    BleedLimiter::new(phi_lim, b_max).on_incidence()
}

/// `m_lim` for a valve: `T_c − 1/φ_lim` (the incidence set point a `φ_lim` names at `v = 0`).
pub fn m_lim(t_c: f64, phi_lim: f64) -> f64 {
    t_c - 1.0 / phi_lim
}

/// Which wall a valve watches, as the panel and the sandbox name it.
pub fn reference_key(r: ValveReference) -> &'static str {
    match r {
        ValveReference::Phi => "phi",
        ValveReference::Incidence => "incidence",
    }
}
