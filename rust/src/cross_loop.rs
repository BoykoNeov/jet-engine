//! RUNG 67 — CASCADE A: two loops on TWO variables, and the ONE SCALAR that sets both faces.
//!
//! Slice Z, second of two. Python's `CrossLoopCascadeTransient` (`turbojet/engine.py`, **843
//! lines**, 15 methods) — rung 47's lagged `Tt4` topping GOVERNOR beside rung 65's lagged
//! `phi_lp` bleed VALVE. Four states, two clocks, and — unlike [`crate::two_lag`] — **two
//! different protected variables**.
//!
//! **IT IS RUNG 66's CONSTRUCTION WITH ONE SUBSTITUTION** — the fuel leg's SENSOR moves from
//! `phi_lp` to `Tt4` — and that single change inverts the algebra. With `P = R_q · C_g`, rung 66
//! has `P ≡ +1` identically, so `det J ≡ 0` and the discriminant is `tr²`: degenerate, provably
//! no oscillation at any clock ratio. Here nothing pins `P`, `det J = (1 − P)/(t_g t_v) ≠ 0`, and
//! **one scalar decides both faces** — it ends the degeneracy (so the pair buys AUTHORITY) and it
//! opens a ringing window, then damps it: **admissible, unobservable**.
//!
//! # `_eig`'s COMPLEX ARM IS DEAD ON THE RUNG THAT DEFINES IT AND LIVE ONLY HERE
//!
//! § 5.24 (vi). Rung 66 defines `_eig`; rung 67's `cross_identity` calls it too, and a census that
//! does not split by CALLING FUNCTION conflates them (the first one did, at 134 real / 57
//! complex). Split, over `test_rung66.py` alone: **80 of 80 real, and the complex arm never runs
//! on rung 66 at all** — which is not an accident but rung 66's own headline, since `det J ≡ 0`
//! makes the discriminant `tr² − 4·0 = tr² ≥ 0` identically. The arm is kept alive one rung up.
//! **A port that drops it passes every rung-66 gate and breaks here.**
//!
//! # THE TWO JOINT INITIAL CONDITIONS ARE A DELIBERATE DUPLICATION AND MUST NOT BE FACTORED
//!
//! § 5.24 (iv) / P6. Both marches need the two laws' simultaneous equilibrium at `s = 0`, and the
//! two rungs solve it **differently on purpose**:
//!
//! * **rung 66** iterates INLINE and UNDAMPED, capped at 60, and asserts on failure — its own
//!   identity pins the contraction factor `|R_q C_g|` at 1, so a stall there genuinely IS the
//!   degeneracy the rung is about, and the assert says so.
//! * **rung 67** calls `_joint_fixed_point`, which sweeps `w ∈ (1.0, 0.5, 0.25)` — here `|P|` is
//!   pinned by nothing and `det J ≠ 0` for every `P ≠ 1`, so a stall would be a SOLVER failure and
//!   reporting it as a marginal mode would be a FALSE FINDING.
//!
//! Giving rung 66 a call to this rung's damped solver is bit-exact on the shipped grid (probe 3:
//! `w = 1.0` on 36 of 39 calls) and destroys the distinction the two asserts are FOR. That is the
//! standing *copy vs re-derivation* rule and this is the reflex it exists to stop.
//!
//! # The dispatch, the four refusals and [`RINGS`] shipped a step BEFORE the march
//!
//! Splitting them is what made the three reduce arms gate-able before a line of the march
//! existed. Step 2 filled the march in and no reduce gate moved.

use crate::bleed_transient::{r62_try_close_fuel, LeverArm, LeverArming, LeverHooks};
use crate::engine::FlightCondition;
use crate::fuel_transient::{asym_extra, point, FuelInstant, FuelLimiters, FuelPoint,
                            FuelTransientCore, FuelTransientHooks, PointExtra};
use crate::gas::Abort;
use crate::lagged_bleed::{lagged, valve_of};
use crate::limited_bleed::BleedLimiter;
use crate::map::ComponentMap;
use crate::stator_transient::{MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient,
                              StatorLeg, StatorTransientHooks};
use crate::two_lag::{eig, py_max_default, py_max_of, py_min_of, violation, GAINS_DG, GAINS_DQ};
use crate::two_spool::{Spool, TwoSpoolEngine};
use crate::two_spool_transient::{ForcedBleed, LaggedGovernor, MarchedBleed,
                                 TwoSpoolTransientHooks};

/// RUNG 67's `_RINGS` — **how many sign changes in a sampled eigenvalue-imaginary-part trace
/// count as a RING**, and the one class attribute in either rung of this slice that a gate reads
/// directly (`tests/test_rung67.py:285`).
///
/// **A PLAIN `const`, AND THAT IS A MEASUREMENT RATHER THAN A DEFAULT.** Python reads it through
/// `cls._RINGS` and `self._RINGS` — the dynamic-lookup spelling, which is what an OVERRIDABLE
/// constant looks like, so the port owes the question *does any rung above 67 rebind it?* before
/// choosing between an associated const and a table cell. Grepped over the whole 23 066-line
/// `engine.py` and over all 27 phase-7 suites: **one definition, two reads, both inside rung 67's
/// own readers, and no rebinding at any rung through 84.** So a module const is faithful, and if
/// a later rung ever rebinds it this comment is where the port is wrong.
///
/// The two reads are `detector_sensitivity` (`n >= cls._RINGS`) and `oscillation_window`
/// (`max(nq, ng) >= self._RINGS`) — both arrive at step 2.
pub const RINGS: usize = 2;

// ---------------------------------------------------------------------------------------------
// THE CELLS — the SAME three swaps as rung 66, and again zero additions
// ---------------------------------------------------------------------------------------------

/// RUNG 67's `_stator_march` — rung 66's march with ONE addition, `tau_gov`.
///
/// The governor's clock rides on the carrier for rung 65/66's reason verbatim: a dozen
/// rung-57-to-66 readers call this cell knowing nothing about it, and every one must keep
/// reaching the IDENTICAL march. `tau_gov = None` leaves them all bit-for-bit.
///
/// **THE REDLINE NEEDS NO PLUMBING OF ITS OWN.** `Tt4_max` has been a rung-58 [`StatorLeg`] field
/// since slice V, so only the CLOCK is new here — which is why this rung adds one scope field and
/// not two.
///
/// The guard is [`LaggedGovernor`], restore-PREVIOUS, and the two questions (*does it restore the
/// previous value*, *does it ever nest*) were asked separately from rung 66's rather than
/// inherited from it — a carrier claim on ONE hook says nothing about the next. Both answers came
/// back the same: restore-previous, max depth **1**, **0** nested events.
pub fn r67_stator_march(
    ft: &FuelTransientCore, flight: &FlightCondition, ramp: &Ramp, nu0: Option<(f64, f64)>,
    leg: &StatorLeg<'_>, scope: &MarchScope,
) -> (Vec<FuelPoint>, (f64, f64)) {
    let _g = LaggedGovernor::set(&ft.inner, scope.tau_gov);
    // Python forwards `lag=lag` and drops `tau_gov` — this rung consumes exactly one field and
    // passes the other two on, as rung 66 passes `b0` on.
    crate::two_lag::r66_stator_march(
        ft, flight, ramp, nu0, leg,
        &MarchScope { b0: scope.b0, lag: scope.lag, ..MarchScope::DEFAULT })
}

/// RUNG 67's `at_lever` — **the SIXTH instance of the sibling-constructor trap** rungs
/// 61/62/63/64/65/66 each hit: the inherited constructor hardcodes its own class name, so a
/// rung-67 machine would silently hand back a rung-66 one.
///
/// The governor clock is a per-MARCH argument, not a machine keyword, so there is nothing here
/// for it to drop.
pub fn r67_at_lever(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    match build_cross_loop_cascade(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(), arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("at_lever never disables LP"),
    }
}

/// RUNG 67's `integrate_fuel` — the dispatch and **four** refusals (rung 66 has three).
///
/// | reduce arm | condition | lands on |
/// |---|---|---|
/// | rung 66 | `tau_gov is None`, `lag` set | rung 66's merged march |
/// | rung 65 | `tau_gov is None`, `lag is None` | `r65_integrate_fuel_valve_lag` |
/// | rung 47 | `_lagged()` false | `integrate_fuel_lagged`, untouched |
///
/// Probe 3 measured all four `(armed, arg)` arms of this dispatch LIVE (5 / 6 / 8 / 38).
///
/// **THE `Tt4_max` REFUSAL IS THE ONE THAT MAKES A PLACEMENT CHOICE TESTABLE.** Rung 66 recorded
/// an ambiguity and dodged it — rung 52 min-selects the redline UNLAGGED on top of already-clipped
/// fuel, rung 65 puts it inside the caps at `mf_sched`, and cascade B never armed it. Here the
/// redline IS the lagged leg, so it is carried BY the state exactly as rung 47 carries it, and a
/// wrong pick shows up as a diff against rung 47 itself.
pub fn r67_integrate_fuel(
    ft: &FuelTransientCore, flight: &FlightCondition, fuel_schedule: &dyn Fn(f64) -> f64,
    nu0: (f64, f64), s_end: f64, ds: f64, lim: &FuelLimiters<'_>,
) -> Vec<FuelPoint> {
    // Python's `tau_gov = tau_gov if tau_gov is not None else self._tau_gov` — the ARGUMENT wins,
    // and the RESOLVED value is what `super()` receives. `lag` is forwarded RAW: rung 66's cell
    // resolves it against its OWN carrier, which is why this rung must not resolve it here.
    let tau_gov = lim.tau_gov.or_else(|| ft.inner.tau_gov.get());
    if !(lagged(&ft.inner) && tau_gov.is_some()) {
        return crate::two_lag::r66_integrate_fuel(
            ft, flight, fuel_schedule, nu0, s_end, ds,
            &FuelLimiters { tau_gov, ..lim.clone() });
    }
    assert!(lim.tt4_max.is_some(),
            "rung-67: `tau_gov` is the GOVERNOR's clock and a governor needs a redline to \
             lag (rung 47's own assert, one cascade up). Without `Tt4_max` the fuel state has \
             nothing to run on and the cascade would silently reduce to rung 65 while \
             claiming four states.");
    assert!(lim.lag.is_none(),
            "rung-67 is CASCADE A: rung 47's Tt4 governor beside rung 65's phi valve -- two \
             loops on TWO variables. Rung 52's AsymmetricLag over rung 49's phi floor is \
             CASCADE B, which is rung 66 and reached by leaving `tau_gov` None. Running both \
             fuel legs at once is THREE loops on two variables -- a separate rung (rung 67's \
             own next seam), asserted against rather than run.");
    assert!(lim.accel.is_none() && lim.floor().is_none(),
            "rung-67 arms the GOVERNOR as its fuel leg. A second fuel-side leg (rung 48's \
             accel schedule, rung 49's phi floor) makes it three loops and, for `surge`, puts \
             a SECOND loop back on `phi_lp` -- which would superpose rung 66's identity onto \
             this rung's window and measure neither cleanly. One rung, one headline.");
    assert!(lim.s_off.is_none() && lim.tau_rel.is_none(),
            "rung-67: rungs 50/51's FORCED release edges are an isolation instrument for a leg \
             that could not pin its own trigger. Both legs here pin their own (rung 47's \
             governor rides its own signal, rung 65's valve its own), so forcing one would \
             measure the forcing.");
    r67_integrate_fuel_cross(
        ft, flight, fuel_schedule, nu0, s_end, ds, lim,
        lim.tt4_max.expect("the assert above proves this is Some"),
        tau_gov.expect("the dispatch above proves this is Some"))
}

/// RUNG 67's MARCH — rung 47's `_integrate_fuel_lagged` and rung 65's
/// `_integrate_fuel_valve_lag`, MERGED: four states, the two actuators coupled ONLY through the
/// plant, and the two laws watching DIFFERENT variables.
///
/// `g`/`required` (rung 47/52's keys) and `b`/`b_cmd` (rung 65's) are ALL recorded per point, so
/// both tracking errors read straight off one trajectory and every rung-47, rung-52 and rung-65
/// reader works unchanged on it.
///
/// **THE `_b_state` BOUNDARY IS LOAD-BEARING HERE IN A WAY IT WAS NOT ON CASCADE B.** `R_q ≠ 0`
/// only because the governor senses `Tt4` on the machine AS THE VALVE ACTUALLY IS. Drop the state
/// around `required` and `R_q ≡ 0`, the rung silently becomes two INDEPENDENT loops with
/// `det J = 1/(t_g t_v)`, no complex branch anywhere — **and nothing fails.** `cross_identity`
/// measures `R_q ≠ 0` as a gate for exactly that reason.
///
/// # THE DERIVATIVE HAS NO REDLINE MIN-SELECT, AND THAT IS THE PLACEMENT DECISION EXECUTING
///
/// Rung 66's `der` min-selects an UNLAGGED `Tt4_max` on top of the already-clipped fuel (rung
/// 52's placement) and its own docstring records the ambiguity, dodging it by never arming a
/// redline on cascade B. **Here the redline IS the lagged leg**, so it is carried BY the state —
/// `mf = mf_sched - g`, exactly as rung 47 carries it — and there is nothing left to min-select.
/// A port that copies rung 66's `der` and keeps its `Tt4_max` branch still compiles, still
/// marches, and breaks the rung-47 reduce, **which is the detector**: with the valve disarmed
/// this class must reproduce `integrate_fuel_lagged` bit-for-bit by dispatch, so a wrong
/// placement shows up as a diff against rung 47 itself.
#[allow(clippy::too_many_arguments)]
pub fn r67_integrate_fuel_cross(
    ft: &FuelTransientCore, flight: &FlightCondition, fuel_schedule: &dyn Fn(f64) -> f64,
    nu0: (f64, f64), s_end: f64, ds: f64, lim: &FuelLimiters<'_>,
    tt4_max: f64, tau_gov: f64,
) -> Vec<FuelPoint> {
    let bl = ft.inner.lever.lim.expect("rung-67's march on an unfloored machine");
    let tau = bl.tau.expect("rung-67's march on an unlagged machine");
    let freeze = lim.freeze;
    // THE MODELLING FLOOR -- rung 66's, INHERITED AND STILL SAFE, but no longer the radius. Rung
    // 66 derived `ds*(1/t_g + 1/t_v) <= 2` from its own identity: `det J == 0` makes the non-zero
    // eigenvalue exactly `-(1/t_g + 1/t_v)`, so the rates ADD. Here `det J != 0` and on the
    // complex branch the radius is `sqrt(det) = sqrt((1+|P|)/(t_g t_v))`, which at matched clocks
    // is `1.01/t` against the sum's `2/t` -- CONSERVATIVE by ~2x. A floor derived from an
    // identity is conservative wherever the identity does not hold, and the sum stops bounding
    // the radius only once `|P| > 3` (measured: ~0.02). It is kept as the a-priori assert because
    // it is what can be computed BEFORE the march; `cross_identity` reports the measured radius
    // beside it.
    let rate = 1.0 / tau + 1.0 / tau_gov;
    assert!(ds * rate <= 2.0,
            "rung-67: ds*(1/tau_v + 1/tau_gov) = {:.3} is outside the explicit RK4 stability \
             region for the two actuator states (ds = {ds}, tau_v = {tau}, tau_gov = {tau_gov}). \
             Rung 65 published a RETRACTION for exactly this failure mode at one state -- an \
             instability that looked like a physical finding. The sum is rung 66's bound and is \
             CONSERVATIVE here (the radius is sqrt(det)), so a violation is not borderline. \
             Refine the grid or slow a clock.", ds * rate);
    let (tt2, pt2, _) = ft.inner.inlet(flight);

    // Rung 64's instantaneous root at THIS state and fuel, WITHOUT the march state.
    let command = |a: f64, h: f64, mf: f64| -> Result<f64, Abort> {
        Ok(crate::limited_bleed::r64_solve_b(&bl, |b| {
            let _g = ForcedBleed::set(&ft.inner, b);
            r62_try_close_fuel(ft, a, h, mf, tt2, pt2)
        })?.1)
    };

    // Rung 47's governor requirement, ON THE PLANT AS THE VALVE ACTUALLY IS. Solved from the
    // SCHEDULED fuel (rung 47's own discipline: `required` is what the clip WOULD have to be, not
    // what the current clip makes it). **`R_q != 0` ONLY because of the state below** — drop it
    // and the rung silently becomes two INDEPENDENT loops with no complex branch anywhere, and
    // nothing fails.
    let required = |a: f64, h: f64, q: f64, mf_sched: f64| -> Result<f64, Abort> {
        let _st = MarchedBleed::set(&ft.inner, q);
        let i = ft.try_instant_fuel(flight, a, h, mf_sched)?;
        if i.base.tt4 <= tt4_max {
            return Ok(0.0);
        }
        Ok(0.0f64.max(mf_sched - ft.try_topping_fuel(flight, a, h, tt4_max, mf_sched)?))
    };

    type Der = (f64, f64, f64, f64, f64, FuelInstant, f64, f64);
    let der = |a: f64, h: f64, g: f64, q: f64, s: f64| -> Result<Der, Abort> {
        let mf_sched = fuel_schedule(s);
        let req = required(a, h, q, mf_sched)?;
        let mf = 1e-9f64.max(mf_sched - g);
        let inst = {
            let _st = MarchedBleed::set(&ft.inner, q);
            ft.try_instant_fuel(flight, a, h, mf)?
        };
        let cmd = command(a, h, mf)?;
        let da = if freeze == Some(Spool::Lp) { 0.0 } else { inst.base.phi_lp_dot / ft.rho() };
        let dh = if freeze == Some(Spool::Hp) { 0.0 } else { inst.base.phi_hp_dot };
        Ok((da, dh, (req - g) / tau_gov, (cmd - q) / tau, mf, inst, req, cmd))
    };

    // --- THE JOINT INITIAL CONDITION, AND IT CANNOT INHERIT RUNG 66's MESSAGE ------------------
    // The iteration contracts at `|P|`, and THAT is where the two cascades diverge. On B the
    // identity pins `|P| = 1` wherever both laws ride, so the solve converges only because the
    // march opens dormant and rung 66 can honestly report a stall as THE DEGENERACY. Here `|P|`
    // is pinned by nothing: a stall would mean `|P| >= 1` with the equilibrium still UNIQUE
    // (`det J != 0`) -- a SOLVER failure, and reporting it as a marginal mode would be a FALSE
    // FINDING. So the fallback is a DAMPED sweep, and only ITS failure asserts.
    let (mut a, mut h) = nu0;
    let mf0 = fuel_schedule(0.0);
    let b0 = ft.inner.b0.get();
    if let Some(x) = b0 {
        assert!((0.0..=bl.b_max).contains(&x),
                "rung-67 b0 is a valve POSITION: {x} is outside [0, {}]", bl.b_max);
    }
    // Python raises out of the whole method here — the initial solves sit BEFORE the loop's
    // `try`. `joint_ic_corners` is the one caller that catches it.
    let raise = |e: Abort| -> f64 { panic!("{}", e.0) };
    let ic = joint_fixed_point(
        &|qq: f64| required(a, h, qq, mf0).unwrap_or_else(raise),
        &|gg: f64| command(a, h, 1e-9f64.max(mf0 - gg)).unwrap_or_else(raise),
        match b0 { Some(x) => x, None => command(a, h, mf0).unwrap_or_else(raise) },
        b0.is_some(), 1e-12, 60);
    let (mut g, mut q, res, its, w_used) = (ic.g, ic.q, ic.res, ic.its, ic.w);
    assert!(res <= 1e-9,
            "rung-67: the joint initial condition did not converge (residual {res:.3e} after \
             {its} iterations, down to damping {w_used}). The iteration contracts at \
             |P| = |R_q C_g|, which on THIS cascade is pinned by NO identity -- so unlike rung 66 \
             this is a SOLVER failure and NOT a marginal mode: det J = (1-P)/(t_g t_v) is \
             non-zero for every P != 1, so the equilibrium exists and is unique. Report the \
             measured |P| (cross_identity) and solve the 2x2 by Newton; do not report a \
             degeneracy.");

    let mut pts: Vec<FuelPoint> = Vec::new();
    let mut s = 0.0f64;
    let n_steps = (s_end / ds).round_ties_even() as i64;
    for _ in 0..=n_steps {
        let Ok((k1a, k1h, k1g, k1q, mf_app, inst, req, cmd)) = der(a, h, g, q, s) else { break };
        pts.push(point(s, a, h, &inst, mf_app, fuel_schedule(s),
                       PointExtra::CrossCascade { g, required: req, b: q, b_cmd: cmd,
                                                  ic_iters: its, ic_res: res, ic_damp: w_used }));
        let stages = (|| -> Result<[f64; 12], Abort> {
            let (k2a, k2h, k2g, k2q, ..) = der(a + ds / 2.0 * k1a, h + ds / 2.0 * k1h,
                                               g + ds / 2.0 * k1g, q + ds / 2.0 * k1q,
                                               s + ds / 2.0)?;
            let (k3a, k3h, k3g, k3q, ..) = der(a + ds / 2.0 * k2a, h + ds / 2.0 * k2h,
                                               g + ds / 2.0 * k2g, q + ds / 2.0 * k2q,
                                               s + ds / 2.0)?;
            let (k4a, k4h, k4g, k4q, ..) = der(a + ds * k3a, h + ds * k3h, g + ds * k3g,
                                               q + ds * k3q, s + ds)?;
            Ok([k2a, k2h, k2g, k2q, k3a, k3h, k3g, k3q, k4a, k4h, k4g, k4q])
        })();
        let Ok([k2a, k2h, k2g, k2q, k3a, k3h, k3g, k3q, k4a, k4h, k4g, k4q]) = stages
        else { break };
        a += ds / 6.0 * (k1a + 2.0 * k2a + 2.0 * k3a + k4a);
        h += ds / 6.0 * (k1h + 2.0 * k2h + 2.0 * k3h + k4h);
        g += ds / 6.0 * (k1g + 2.0 * k2g + 2.0 * k3g + k4g);
        q += ds / 6.0 * (k1q + 2.0 * k2q + 2.0 * k3q + k4q);
        // Both hardware stops, verbatim from rungs 65/66: applied to the STATE and never to the
        // command; the clip floored at zero because a limiter cannot hand back more fuel than it
        // took.
        q = bl.b_max.min(0.0f64.max(q));
        g = 0.0f64.max(g);
        s += ds;
    }
    pts
}

// ---------------------------------------------------------------------------------------------
// THE LEAF STATICS — Python's four `@staticmethod`s and one `@classmethod`
// ---------------------------------------------------------------------------------------------

/// The `(ic_iters, ic_res, ic_damp)` a rung-67 point carries — a PANIC on every other route,
/// because Python raises `KeyError` there.
///
/// The arms are spelled out rather than left to a wildcard so that the NEXT `PointExtra` variant
/// breaks the build here and gets the same question asked of it — see rung 65's `valve_of`, whose
/// wildcard is what slice Z's audit had to unpick by hand.
pub fn cross_extra(p: &FuelPoint) -> (usize, f64, f64) {
    match p.extra {
        PointExtra::CrossCascade { ic_iters, ic_res, ic_damp, .. } => (ic_iters, ic_res, ic_damp),
        PointExtra::None | PointExtra::Asym { .. } | PointExtra::Valve { .. }
        | PointExtra::Cascade { .. } => panic!(
            "rung-67 reader on a trajectory with no joint-IC record: this march did not \
             dispatch to r67_integrate_fuel_cross. Rung 66's cascade carries `ic_iters` and \
             `ic_res` but NOT `ic_damp` -- it iterates undamped -- so it is refused here rather \
             than folded in."),
    }
}

/// Python's `_window` return. `rho_lo` / `rho_hi` / `reciprocal` are `None` on the `P >= 0`
/// branch, where the interval does not open.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Window {
    pub p: f64,
    pub k: f64,
    pub zeta: f64,
    pub t_over_tau: f64,
    pub rho_lo: Option<f64>,
    pub rho_hi: Option<f64>,
    pub opens: bool,
    /// `|rho_lo * rho_hi − 1|` — the log-symmetry, measured rather than asserted. Absent (not
    /// zero) on the closed branch, which is Python's dict missing the key.
    pub reciprocal: Option<f64>,
}

/// The complex branch in `rho = t_v/t_g`, **in closed form**:
///
/// ```text
/// disc < 0   <=>   rho + 1/rho < 2 + 4|P|
/// ```
///
/// so the edges are the two roots of `rho² − k rho + 1 = 0` with `k = 2 + 4|P|` — RECIPROCALS,
/// hence an interval log-symmetric about matched clocks. `P >= 0` (cascade B's regime) returns no
/// window at all: **rung 66's result, recovered as the `P → +1` limit of this formula rather than
/// asserted separately.**
///
/// `zeta` and `T_over_tau` are quoted AT `rho = 1`, the window's centre, and **NEITHER contains a
/// time constant** — which is the whole reason a faster valve cannot make the mode visible.
///
/// # Two spellings that are not free choices
///
/// Python is `1.0 / (1.0 + abs(P)) ** 0.5`, and `**` binds tighter than `/`, so it is
/// `1/sqrt(1+|P|)` — **not** `sqrt(1/(1+|P|))`. Likewise `2*pi / abs(P) ** 0.5` is
/// `2π/sqrt|P|`, not `sqrt(4π²/|P|)`. Both are algebraically equal and neither is
/// bit-equal.
///
/// The `P == 0` guard is REAL and not defensive: Python's float division by zero RAISES, so the
/// `inf` is written out. **Zero of 31 shipped calls take it** (§ 5.24 (v)) and step 5 owes it a
/// manufactured gate.
pub fn window(p: f64) -> Window {
    let k = 2.0 + 4.0 * p.abs();
    let zeta = 1.0 / (1.0 + p.abs()).sqrt();
    let t_over_tau = if p != 0.0 {
        2.0 * std::f64::consts::PI / p.abs().sqrt()
    } else {
        f64::INFINITY
    };
    if p >= 0.0 {
        return Window { p, k, zeta, t_over_tau, rho_lo: None, rho_hi: None, opens: false,
                        reciprocal: None };
    }
    let disc = (k * k - 4.0).sqrt();
    let (lo, hi) = (0.5 * (k - disc), 0.5 * (k + disc));
    Window { p, k, zeta, t_over_tau, rho_lo: Some(lo), rho_hi: Some(hi), opens: true,
             reciprocal: Some((lo * hi - 1.0).abs()) }
}

/// Sign changes in a sequence, ignoring exact zeros and values below a floor set by the
/// sequence's OWN scale — a decaying free response eventually reaches roundoff, where sign flips
/// are noise and not a mode.
///
/// **THE `peak <= 0` EARLY RETURN IS DEAD ON THE GRID** (0 of 10 calls, § 5.24 (v)) and ships
/// live, exhibited directly by `sign_changes(&[0.0, 0.0, 0.0]) == 0`.
pub fn sign_changes(xs: &[f64]) -> usize {
    let abs: Vec<f64> = xs.iter().map(|v| v.abs()).collect();
    let peak = py_max_default(&abs, 0.0);
    if peak <= 0.0 {
        return 0;
    }
    let floor = 1e-6 * peak;
    let (mut n, mut prev) = (0usize, 0.0f64);
    for &x in xs {
        if x.abs() < floor {
            continue;
        }
        if prev != 0.0 && (x > 0.0) != (prev > 0.0) {
            n += 1;
        }
        prev = x;
    }
    n
}

/// [`joint_fixed_point`]'s return — Python's five-tuple `(g, q, residual, iterations, damping)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JointFixedPoint {
    pub g: f64,
    pub q: f64,
    /// **THE DAMPED STEP, not an equation residual** — `max(|Δg|, |Δq|)` after the relaxation.
    pub res: f64,
    /// The LAST attempt's iteration count, which is Python's leaked loop variable.
    pub its: usize,
    /// The damping the successful attempt used.
    pub w: f64,
}

/// The two laws' simultaneous equilibrium, by **DAMPED Gauss–Seidel**.
///
/// **IT IS EXTRACTED FROM THE MARCH SO IT CAN BE TESTED, and that is not tidiness**: on the
/// anchored plant `|P| ~ 0.02` and the undamped sweep converges in one or two iterations, so the
/// damped retries are code that NEVER RUNS THERE. Fed synthetic laws with a chosen `P` it is
/// exercised directly — the composite map's multiplier is `(1−w) + wP`, so `w = 1` handles
/// `|P| < 1`, `w = 1/2` up to `|P| < 3`, `w = 1/4` up to `|P| < 7`.
///
/// **WHY THE LADDER EXISTS AT ALL, AND IT IS RUNG 66's MESSAGE THAT MUST NOT BE INHERITED**:
/// rung 66's iteration contracts at `|P|`, which ITS identity pins at 1, so a stall there
/// genuinely IS the degeneracy. Here `|P|` is pinned by nothing and the equilibrium is unique
/// regardless — so a stall would be a SOLVER failure, and reporting it as a marginal mode would
/// be a false finding. **Damping first, assert second**, and [`crate::two_lag`]'s inline
/// 60-iteration loop is NOT re-routed through this (§ 5.24 P6).
///
/// # Three details a tidy re-derivation would lose
///
/// * `command_of` reads the **UNDAMPED** `gn`; the relaxation is applied to BOTH raw values
///   afterwards.
/// * `res` is the **damped step**, so shrinking `w` shrinks `res` for free — which is why the
///   outer acceptance bar (`1e-9`) is looser than the inner break (`tol = 1e-12`) and why the two
///   must not be conflated. Probe 3 found exactly ONE shipped call that exhausts the inner bar
///   without breaking yet passes the outer one; folding them loses its `its == 60`.
/// * `its` and `w` are Python's LEAKED loop variables — the last attempt's, not the successful
///   one's, whenever they differ.
pub fn joint_fixed_point(
    required_of: &dyn Fn(f64) -> f64, command_of: &dyn Fn(f64) -> f64, q0: f64, fix_q: bool,
    tol: f64, cap: usize,
) -> JointFixedPoint {
    let (mut g, mut q, mut res) = (0.0f64, 0.0f64, 0.0f64);
    let (mut its, mut w_used) = (0usize, 1.0f64);
    for w in [1.0f64, 0.5, 0.25] {
        g = 0.0;
        q = q0;
        res = f64::INFINITY;
        w_used = w;
        for k in 1..=cap {
            its = k;
            let gn_raw = required_of(q);
            let qn_raw = if fix_q { q } else { command_of(gn_raw) };
            let (gn, qn) = (g + w * (gn_raw - g), q + w * (qn_raw - q));
            res = (gn - g).abs().max((qn - q).abs());
            g = gn;
            q = qn;
            if res <= tol {
                break;
            }
        }
        if res <= 1e-9 {
            break;
        }
    }
    JointFixedPoint { g, q, res, its, w: w_used }
}

/// `∫ max(0, Tt4 − Tt4_max) ds` over the ramp — **the TEMPERATURE currency**, built the same way
/// as rung 66's [`violation`] and for the same reason: an AREA cannot be clamped by its own
/// initial condition.
///
/// **IT DOES NOT COPY RUNG 66's UPPER LIMIT, AND THE DIFFERENCE IS MEASURED.** `violation` breaks
/// on `traj[i].s > s_hi`, which DROPS the whole final cell whenever the marched `s` lands a
/// float's width past `r`. On rung 66's currency that is immaterial — the phi violation is an
/// EARLY-ramp object and its integrand is ~0 by `s = r`. On this one the integrand is at its
/// MAXIMUM there (`Tt4` peaks at the end of the ramp), so a dropped cell is worth ~`ds * 490` and
/// the raw integral drifts 2.8 % over an 8x `ds` range, monotone, with the increments refusing to
/// halve — **a grid artefact that reads exactly like slow convergence.** Here the straddling cell
/// is INTERPOLATED at `s_hi` instead. The credit RATIO was stable either way, which is why the
/// fix changes no published number; the raw integral becomes quotable, which is why it is made.
/// **Rung 66's is deliberately NOT touched — its numbers are gated** — so the two are two
/// functions and never one with a flag.
pub fn exceed(traj: &[FuelPoint], tt4_max: f64, s_hi: f64) -> f64 {
    let mut out = 0.0;
    for i in 1..traj.len() {
        let (s0, mut s1) = (traj[i - 1].s, traj[i].s);
        if s0 >= s_hi {
            break;
        }
        let f0 = 0.0f64.max(traj[i - 1].tt4 - tt4_max);
        let mut f1 = 0.0f64.max(traj[i].tt4 - tt4_max);
        if s1 > s_hi {
            // the straddling cell: CLIP, do not drop
            let w = (s_hi - s0) / (s1 - s0);
            f1 = f0 + w * (f1 - f0);
            s1 = s_hi;
        }
        out += 0.5 * (s1 - s0) * (f0 + f1);
    }
    out
}

/// One `P` of [`DetectorSensitivity`] — Python's `out[i]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DetectorRow {
    pub p: f64,
    pub zeta: f64,
    pub t_over_tau: f64,
    pub t: f64,
    pub periods: f64,
    pub decay_per_period: f64,
    pub sign_changes: usize,
    pub rings: bool,
}

/// Python's `detector_sensitivity` return.
#[derive(Clone, Debug)]
pub struct DetectorSensitivity {
    pub tau: f64,
    pub ds: f64,
    pub s_end: f64,
    pub rows: Vec<DetectorRow>,
    pub fires: bool,
    /// `None` on an EMPTY `Ps` — Python's `if out else None`.
    pub quiet_at_weak: Option<bool>,
}

/// **WHAT THE RINGING DETECTOR CAN SEE — measured, not assumed.**
///
/// [`ScheduledStatorCore::oscillation_window`] reports ZERO sign changes in the free response at
/// every clock pair, and **a null result is worth nothing until the instrument is shown to
/// fire.** So the same RK4 and the same [`sign_changes`] are run on the LINEAR block itself for a
/// range of `P`, at matched clocks, from a unit offset in `g`:
///
/// ```text
/// d/ds [g q] = [[-1, R_q], [C_g, -1]]/tau [g q],   R_q C_g = P
/// ```
///
/// With `R_q = 1` and `C_g = P` the block has the right spectrum for any `P`. **THE POINT**: at
/// `|P| ~ 0.02` the detector reads 0 because `T = 45 tau` and the amplitude is `e^-45` by then —
/// NOT because the detector is blind.
///
/// A free function because Python's is a `@classmethod` reading only `cls._window`,
/// `cls._sign_changes` and [`RINGS`] — no instance state at all.
pub fn detector_sensitivity(ps: &[f64], tau: f64, ds: f64, s_end: f64) -> DetectorSensitivity {
    let mut out: Vec<DetectorRow> = Vec::new();
    for &p in ps {
        let (r_q, c_g) = (1.0f64, p);
        let (mut g, mut q) = (1.0f64, 0.0f64);
        let mut xs: Vec<f64> = Vec::new();
        let der = |gg: f64, qq: f64| -> (f64, f64) {
            ((-gg + r_q * qq) / tau, (c_g * gg - qq) / tau)
        };
        let n_steps = (s_end / ds).round_ties_even() as i64;
        for _ in 0..=n_steps {
            xs.push(g);
            let k1 = der(g, q);
            let k2 = der(g + ds / 2.0 * k1.0, q + ds / 2.0 * k1.1);
            let k3 = der(g + ds / 2.0 * k2.0, q + ds / 2.0 * k2.1);
            let k4 = der(g + ds * k3.0, q + ds * k3.1);
            g += ds / 6.0 * (k1.0 + 2.0 * k2.0 + 2.0 * k3.0 + k4.0);
            q += ds / 6.0 * (k1.1 + 2.0 * k2.1 + 2.0 * k3.1 + k4.1);
        }
        let w = window(p);
        let n = sign_changes(&xs);
        out.push(DetectorRow {
            p,
            zeta: w.zeta,
            t_over_tau: w.t_over_tau,
            t: w.t_over_tau * tau,
            periods: s_end / (w.t_over_tau * tau),
            decay_per_period: (-w.t_over_tau).exp(),
            sign_changes: n,
            rings: n >= RINGS,
        });
    }
    DetectorSensitivity {
        tau,
        ds,
        s_end,
        fires: out.iter().any(|x| x.rings),
        quiet_at_weak: out.first().map(|x| !x.rings),
        rows: out,
    }
}

// ---------------------------------------------------------------------------------------------
// THE READING INSTRUMENTS
// ---------------------------------------------------------------------------------------------

/// One clock's row of [`CrossIdentity`] — Python's `rows[i]`.
///
/// The six window fields are `Option` because Python spreads them in with `win.get(k)` off a dict
/// that is **EMPTY** when the row had no riding points — and `reciprocal` is additionally absent
/// on the `P >= 0` branch of a NON-empty one. Two different routes to the same `None`, and both
/// are real.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CrossIdentityRow {
    pub tau_gov: f64,
    pub tau_v: f64,
    pub rho_clock: f64,
    pub n_ride: usize,
    /// The DELIVERED sample count — the stride's, not the request's (§ 5.24 (i)).
    pub n_sample: usize,
    pub n_complex: usize,
    pub n_saturated: usize,
    pub prod_lo: f64,
    pub prod_hi: f64,
    /// **THE ONE FLOAT `sum()` IN EITHER RUNG OF THIS SLICE**, and the port's only declared
    /// cross-interpreter exemption — see [`ScheduledStatorCore::cross_identity`].
    pub p_mid: f64,
    /// The gate rung 66 did not need: `R_q != 0` is what makes this a cascade at all (the
    /// `_b_state` trap), so it is REPORTED, never assumed.
    pub r_q_lo: f64,
    pub r_q_hi: f64,
    pub c_g_lo: f64,
    pub c_g_hi: f64,
    pub gain_span_r: f64,
    pub gain_span_c: f64,
    pub rho_max: f64,
    pub sum_bound: f64,
    pub sum_conservative: f64,
    pub rho_lo: Option<f64>,
    pub rho_hi: Option<f64>,
    pub zeta: Option<f64>,
    pub t_over_tau: Option<f64>,
    pub opens: Option<bool>,
    pub reciprocal: Option<f64>,
}

/// Python's `cross_identity` return — **RUNG 67's CORE INSTRUMENT**: the scalar `P`, and
/// everything that follows from it.
#[derive(Clone, Debug)]
pub struct CrossIdentity {
    pub tt4_max: f64,
    pub tau: f64,
    pub tau_govs: Vec<f64>,
    pub ds: f64,
    pub r: f64,
    pub phi_lim: f64,
    pub b_max: f64,
    pub rows: Vec<CrossIdentityRow>,
    pub all_negative: bool,
    pub prod_lo: f64,
    pub prod_hi: f64,
    /// The gate against the `_b_state` trap: **a zero `R_q` is not a small coupling, it is a
    /// MISSING one.**
    pub r_q_min_abs: f64,
    pub sum_always_safe: bool,
}

/// One clock ratio of [`OscillationWindow`] — a SKIPPED row and a LIVE one carry DIFFERENT keys
/// in Python, so they are two variants rather than one struct with `Option`s.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OscRow {
    /// Python's `dict(rho=…, tau_gov=…, skipped="ds floor")` — the inherited rung-66 floor,
    /// never violated on the shipped grid.
    Skipped { rho: f64, tau_gov: f64 },
    Live(OscLive),
}

/// A LIVE row of [`OscillationWindow`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OscLive {
    pub rho: f64,
    pub tau_gov: f64,
    pub npts: usize,
    pub complex_predicted: bool,
    pub sign_changes_q: usize,
    pub sign_changes_g: usize,
    pub rings: bool,
    pub d0: f64,
    pub d_end: f64,
    /// `NaN` when the offset trajectory opens at exactly the natural one — Python's
    /// `if dq[0] else float("nan")`, a TRUTHINESS test on a float, so `-0.0` takes the NaN arm
    /// too.
    pub survives: f64,
    pub d_peak: f64,
}

/// Python's `oscillation_window` return — **RUNG 67's SECOND INSTRUMENT**: the window swept in
/// `rho = t_v/t_g`, against the FREE response of the real plant.
///
/// For each `rho` two marches are run, natural and with the valve's initial position offset by
/// `d_b0`, and the DIFFERENCE trajectory is taken. That difference is the homogeneous solution —
/// the forcing is common to both and cancels to first order — so sign changes in it are THE MODE,
/// not the ramp. **THE PREDICTION BEING TESTED IS A NULL**: complex INSIDE the window and ZERO
/// sign changes EVERYWHERE, because `zeta` has no time constant in it, and
/// [`detector_sensitivity`] is what makes the null falsifiable.
#[derive(Clone, Debug)]
pub struct OscillationWindow {
    pub tt4_max: f64,
    pub tau: f64,
    pub ds: f64,
    pub r: f64,
    pub d_b0: f64,
    pub p: f64,
    pub window: Window,
    pub rhos: Vec<f64>,
    pub rows: Vec<OscRow>,
    pub n_complex: usize,
    pub n_real: usize,
    /// Reported RAW so a reader can see it sit AT the one crossing a real pair is allowed
    /// (see [`RINGS`]), not below it.
    pub max_sign_changes: usize,
    pub rings_anywhere: bool,
    pub survives_max: f64,
}

/// One cell of [`CrossBill`]'s 2x2 — **scored on BOTH protected variables**, which is the object
/// cascade B could not build because it had only one currency.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CrossBillCell {
    /// `∫ max(0, Tt4 − Tt4_max) ds` — the governor's currency.
    pub i_t: f64,
    /// `∫ max(0, phi_lim − phi_lp) ds` — the valve's, rung 66's verbatim.
    pub i_phi: f64,
    pub npts: usize,
    pub s_last: f64,
    pub truncated: bool,
    pub max_tt4: f64,
    pub min_phi: f64,
    pub removed: f64,
    pub nu_lp_end: f64,
    pub nu_hp_end: f64,
    pub thrust_end: f64,
}

/// Python's `cross_bill` return — **RUNG 67's PROTECTION LEDGER**, the 2x2 rung 66 could not
/// build.
///
/// **WHAT THE OFF-DIAGONAL MEASURES**: the valve should DEBIT the temperature (`R_q > 0` — bleed
/// makes it hotter) while the governor CREDITS the surge margin (`C_g < 0` — clipping fuel raises
/// `phi_lp`). One loop helps the other, the other hurts it, and the asymmetry is derivable from
/// the two signs before any march.
///
/// **WHAT THE DIAGONAL MEASURES**: rung 66's 38x erosion came from `det J == 0` — one effective
/// actuator direction. Here `det J != 0` with `|P| ~ 0.02`, so each loop should keep nearly all
/// of its standalone credit ON ITS OWN currency. Same instrument, same `phi_lim`, opposite
/// verdict.
#[derive(Clone, Debug)]
pub struct CrossBill {
    pub tt4_max: f64,
    pub tau: f64,
    pub tau_gov: f64,
    pub ds: f64,
    pub r: f64,
    pub phi_lim: f64,
    pub bare: CrossBillCell,
    pub gov: CrossBillCell,
    pub valve: CrossBillCell,
    pub both: CrossBillCell,
    pub credit_t_gov: f64,
    pub credit_t_valve: f64,
    pub credit_t_both: f64,
    pub credit_phi_gov: f64,
    pub credit_phi_valve: f64,
    pub credit_phi_both: f64,
    pub erosion_gov: f64,
    pub erosion_valve: f64,
    pub marginal_gov_t: f64,
    pub marginal_valve_phi: f64,
    pub valve_on_t: f64,
    pub gov_on_phi: f64,
    pub valve_debits_t: bool,
    pub gov_credits_phi: bool,
    pub sum_alone_t: f64,
    pub sum_alone_phi: f64,
}

/// One member of the sweep [`MarginalModeCross`] runs — Python's `run(...)` return.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MarginalCrossCell {
    pub b0: f64,
    pub b_end: f64,
    pub g_end: f64,
    pub drift: f64,
    pub removed: f64,
    pub i_phi: f64,
    pub i_t: f64,
    pub min_phi_lp: f64,
    pub track_b: f64,
    pub track_g: f64,
    pub n_on: usize,
    pub npts: usize,
    pub ic_iters: usize,
}

/// Python's `marginal_mode_cross` return — **RUNG 65/66's `b0` INSTRUMENT, VERBATIM, ON A
/// NON-DEGENERATE PAIR**, which is exactly what rung 66 § 8 said it lacked.
///
/// Cascade A IS that pair: `det J = (1 + |P|)/(t_g t_v) > 0` strictly, both eigenvalues strictly
/// negative, so an initial offset has a restoring force along EVERY direction and must be
/// forgotten in ~3 t. BOTH outcomes were pre-registered — a COLLAPSE attributes rung 66's spread
/// to its zero eigenvalue and discharges the concession; a SURVIVING spread says rung 66's 84 %
/// was ordinary transient sensitivity and INVERTS it. The comparison is legitimate only because
/// the instrument, the offset, the grid and `phi_lim` are all unchanged.
#[derive(Clone, Debug)]
pub struct MarginalModeCross {
    pub tt4_max: f64,
    pub tau: f64,
    pub tau_gov: f64,
    pub d_b0: f64,
    pub r: f64,
    pub ds: f64,
    pub phi_lim: f64,
    pub natural: MarginalCrossCell,
    pub moved_lo: MarginalCrossCell,
    pub moved_hi: MarginalCrossCell,
    pub b_natural: f64,
    pub db_db0: f64,
    pub dremoved: f64,
    pub dremoved_rel: f64,
    pub d_i_phi: f64,
    pub d_i_phi_rel: f64,
    pub drift: f64,
    pub track_b: f64,
    pub track_g: f64,
}

/// One corner of [`JointIcCorners`] — a march that RAN and one that RAISED carry different keys.
#[derive(Clone, Debug, PartialEq)]
pub enum IcCorner {
    /// Python's `dict(Tt4_lo=…, Tt4_max=…, failed=str(e)[:120])`.
    Failed { tt4_lo: f64, tt4_max: f64, failed: String },
    Ok(IcOk),
}

/// A corner whose march ran.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IcOk {
    pub tt4_lo: f64,
    pub tt4_max: f64,
    /// Is the fuel leg LIVE at `s = 0`? Rung 66 could not exhibit one.
    pub live: bool,
    pub required0: f64,
    pub b0: f64,
    pub g0: f64,
    pub ic_iters: usize,
    pub ic_res: f64,
    pub ic_damp: f64,
    pub npts: usize,
}

/// Python's `joint_ic_corners` return — **RUNG 66's INITIAL-CONDITION DIAGNOSTIC, ON A
/// CONTRACTION THAT IS NOT PINNED AT 1.**
///
/// Rung 66's joint solve converged at every corner it tried — but only because every one opened
/// DORMANT (`required(0) == 0`, `ic_iters == 1`, residual exactly 0), and its own docstring says
/// the contraction is `|R_q C_g|`, which its identity pins at 1 wherever both laws ride. **It
/// could not exhibit a LIVE start.** Cascade A can: the contraction is `|P| ~ 0.02`, and the
/// overlap table shows starts where the governor is already engaged at `s = 0`.
#[derive(Clone, Debug)]
pub struct JointIcCorners {
    pub tt4_los: Vec<f64>,
    pub tt4_maxes: Vec<f64>,
    pub tau: f64,
    pub tau_gov: f64,
    pub ds: f64,
    pub rows: Vec<IcCorner>,
    pub n_live: usize,
    pub all_converged: bool,
    pub max_iters: usize,
    pub ever_damped: bool,
}

impl ScheduledStatorCore {
    /// `R_q = dR/dq`, `C_g = dC/dg` and `C(g)` itself, by CENTRAL DIFFERENCE on the SHIPPED
    /// closures — `try_topping_fuel` for the governor's law, `r64_solve_b` for the valve's.
    ///
    /// **THE BASE POINT IS THE APPLIED FUEL `mf_sched − g`, NOT THE SCHEDULED ONE, and getting
    /// that wrong is the one way this returns a plausible lie.** Evaluated at `g = 0` the valve
    /// command sits hard on `b_max` (the unclipped schedule drives `Tt4 ~ 1900 K`), both sides of
    /// the difference return the STOP, and `C_g` reads EXACTLY 0 — which looks like proof that
    /// the loops are independent. **Any `C_g == 0` from this method is a SATURATED valve, never a
    /// decoupled one**, and `b_cmd` is returned beside it so a reader can tell.
    ///
    /// The two step sizes are rung 66's, verbatim — [`GAINS_DQ`] and [`GAINS_DG`].
    #[allow(clippy::too_many_arguments)]
    pub fn gains_cross(
        &self, flight: &FlightCondition, a: f64, h: f64, g: f64, q: f64, mf_sched: f64,
        tt4_max: f64, dq: f64, dg: f64,
    ) -> (f64, f64, f64) {
        let ft = &self.fuel;
        let bl = ft.inner.lever.lim.expect("rung-67's gains on an unfloored machine");
        let (tt2, pt2, _) = ft.inner.inlet(flight);
        let raise = |e: Abort| -> f64 { panic!("{}", e.0) };
        // The PLANT side: the valve AS IT IS.
        let big_r = |qq: f64| -> f64 {
            let _st = MarchedBleed::set(&ft.inner, qq);
            let i = ft.try_instant_fuel(flight, a, h, mf_sched).unwrap_or_else(|e| panic!("{}",
                                                                                         e.0));
            if i.base.tt4 <= tt4_max {
                return 0.0;
            }
            0.0f64.max(mf_sched
                       - ft.try_topping_fuel(flight, a, h, tt4_max, mf_sched)
                            .unwrap_or_else(raise))
        };
        // The COMMAND side: a root over TRIALS.
        let big_c = |gg: f64| -> f64 {
            crate::limited_bleed::r64_solve_b(&bl, |b| {
                let _g = ForcedBleed::set(&ft.inner, b);
                r62_try_close_fuel(ft, a, h, 1e-9f64.max(mf_sched - gg), tt2, pt2)
            }).unwrap_or_else(|e| panic!("{}", e.0)).1
        };
        ((big_r(q + dq) - big_r(q - dq)) / (2.0 * dq),
         (big_c(g + dg) - big_c(g - dg)) / (2.0 * dg),
         big_c(g))
    }

    /// **RUNG 67's CORE INSTRUMENT** — the scalar `P`, and everything that follows from it.
    ///
    /// **RIDING IS `required > 0`, NOT `mf < mf_sched`** (rung 66's lesson, verbatim): a lagged
    /// clip decays but never reaches zero, so the second test is true forever after first
    /// engagement and would sample the gains where the governor's law is dormant and `R_q == 0` —
    /// exactly where the algebra does not apply.
    ///
    /// # THE ONE DECLARED CROSS-INTERPRETER EXEMPTION IN THIS SLICE
    ///
    /// `P_mid` is `sum(prods) / len(prods)`, the **only float `sum()` in either rung**. CPython
    /// 3.12+'s `sum()` is Neumaier-COMPENSATED and PyPy's is naive, so a Rust left fold agrees
    /// with one interpreter and not the other — slice W's exemption shape verbatim. **Measured at
    /// the DELIVERED chunk width and not the requested one** (§ 5.24 (i)): 1 of `cross_identity`'s
    /// 3 rows differs by ONE ULP on CPython, and pushed through [`window`] that reaches exactly
    /// **2 of 8 keys** (`P`, `T_over_tau`), also one ulp, with the other five absorbing it
    /// exactly. The fold below is NAIVE and LEFT, which is PyPy's — the reference interpreter —
    /// and step 4's CPython arm names the two keys rather than opening a tolerance tier.
    #[allow(clippy::too_many_arguments)]
    pub fn cross_identity(
        &self, flight: &FlightCondition, ramp: &Ramp, tt4_max: f64, tau: f64, tau_govs: &[f64],
        n_sample: usize,
    ) -> CrossIdentity {
        let own = self.fuel.inner.lever.lim
                      .expect("rung-67's cross_identity needs the machine's own valve");
        let b_cap = own.b_max;
        let leg = StatorLeg { accel: None, surge: None, tt4_max: Some(tt4_max) };
        let mut rows: Vec<CrossIdentityRow> = Vec::new();
        for &tg in tau_govs {
            let m = self.at_lever(&LeverArm::floored(
                BleedLimiter::with_tau(own.phi_lim, own.b_max, Some(tau))));
            let (traj, _) = m.stator_march_scoped(
                flight, ramp, None, &leg,
                &MarchScope { tau_gov: Some(tg), ..MarchScope::DEFAULT });
            let ride: Vec<&FuelPoint> = traj.iter().filter(|p| {
                let (_, req) = asym_extra(p);
                let (_, cmd) = valve_of(p);
                req > 0.0 && 0.0 < cmd && cmd < b_cap
            }).collect();
            let stride = 1usize.max(ride.len() / n_sample);
            let sub: Vec<&FuelPoint> = ride.iter().copied().step_by(stride).collect();
            let (mut prods, mut rhos, mut rqs, mut cgs) =
                (Vec::new(), Vec::new(), Vec::new(), Vec::new());
            let (mut cplx, mut sat) = (0usize, 0usize);
            for p in &sub {
                let (g, _) = asym_extra(p);
                let (b, _) = valve_of(p);
                let (r_q, c_g, cmd) = m.gains_cross(flight, p.nu_lp, p.nu_hp, g, b, p.mf_sched,
                                                    tt4_max, GAINS_DQ, GAINS_DG);
                let e = eig(r_q, c_g, tg, tau);
                prods.push(r_q * c_g);
                rhos.push(e.rho);
                cplx += usize::from(!e.real);
                rqs.push(r_q);
                cgs.push(c_g);
                sat += usize::from(cmd <= 0.0 || cmd >= b_cap);
            }
            // The exemption, in one line: a NAIVE LEFT FOLD, which is PyPy's `sum()`.
            let p_mid = if prods.is_empty() {
                f64::NAN
            } else {
                let mut acc = 0.0f64;
                for &x in &prods { acc += x; }
                acc / prods.len() as f64
            };
            let win = if prods.is_empty() { None } else { Some(window(p_mid)) };
            let rate = 1.0 / tg + 1.0 / tau;
            let abs_r: Vec<f64> = rqs.iter().map(|v| v.abs()).collect();
            let abs_c: Vec<f64> = cgs.iter().map(|v| v.abs()).collect();
            rows.push(CrossIdentityRow {
                tau_gov: tg,
                tau_v: tau,
                rho_clock: tau / tg,
                n_ride: ride.len(),
                n_sample: sub.len(),
                n_complex: cplx,
                n_saturated: sat,
                prod_lo: if prods.is_empty() { f64::NAN } else { py_min_of(&prods) },
                prod_hi: if prods.is_empty() { f64::NAN } else { py_max_of(&prods) },
                p_mid,
                r_q_lo: if rqs.is_empty() { f64::NAN } else { py_min_of(&rqs) },
                r_q_hi: if rqs.is_empty() { f64::NAN } else { py_max_of(&rqs) },
                c_g_lo: if cgs.is_empty() { f64::NAN } else { py_min_of(&cgs) },
                c_g_hi: if cgs.is_empty() { f64::NAN } else { py_max_of(&cgs) },
                // the CONTROL on a constant product (rung 66's, and it matters MORE here — a
                // small `P` could be a small plant rather than a weak coupling)
                gain_span_r: if rqs.is_empty() { f64::NAN }
                             else { py_max_of(&abs_r) / py_min_of(&abs_r) },
                gain_span_c: if cgs.is_empty() { f64::NAN }
                             else { py_max_of(&abs_c) / py_min_of(&abs_c) },
                rho_max: if rhos.is_empty() { f64::NAN } else { py_max_of(&rhos) },
                sum_bound: rate,
                sum_conservative: if rhos.is_empty() { f64::NAN }
                                  else { rate / py_max_of(&rhos) },
                rho_lo: win.and_then(|w| w.rho_lo),
                rho_hi: win.and_then(|w| w.rho_hi),
                zeta: win.map(|w| w.zeta),
                t_over_tau: win.map(|w| w.t_over_tau),
                opens: win.map(|w| w.opens),
                reciprocal: win.and_then(|w| w.reciprocal),
            });
        }
        let allp: Vec<f64> = rows.iter().flat_map(|x| [x.prod_lo, x.prod_hi]).collect();
        let rq_abs: Vec<f64> = rows.iter().map(|x| x.r_q_lo.abs()).collect();
        CrossIdentity {
            tt4_max,
            tau,
            tau_govs: tau_govs.to_vec(),
            ds: ramp.ds,
            r: ramp.r,
            phi_lim: own.phi_lim,
            b_max: own.b_max,
            all_negative: allp.iter().all(|&x| x < 0.0),
            prod_lo: py_min_of(&allp),
            prod_hi: py_max_of(&allp),
            r_q_min_abs: py_min_of(&rq_abs),
            sum_always_safe: rows.iter().all(|x| x.rho_max <= x.sum_bound),
            rows,
        }
    }

    /// **RUNG 67's SECOND INSTRUMENT** — see [`OscillationWindow`].
    ///
    /// `cross_identity` is re-run here at the NATURAL clocks only (`tau_govs = (tau,)`,
    /// `n_sample = 8`) to take `P` once, and it is called on `self` rather than on the sibling
    /// `m` — Python's spelling, kept because the two machines' limiters differ in `tau` and the
    /// difference is not obviously inert.
    #[allow(clippy::too_many_arguments)]
    pub fn oscillation_window(
        &self, flight: &FlightCondition, ramp: &Ramp, tt4_max: f64, tau: f64, rhos: &[f64],
        d_b0: f64,
    ) -> OscillationWindow {
        let own = self.fuel.inner.lever.lim
                      .expect("rung-67's oscillation_window needs the machine's own valve");
        let m = self.at_lever(&LeverArm::floored(
            BleedLimiter::with_tau(own.phi_lim, own.b_max, Some(tau))));
        let leg = StatorLeg { accel: None, surge: None, tt4_max: Some(tt4_max) };
        // the window edges from the measured P at THIS anchor, taken once at the natural clocks
        let ident = self.cross_identity(flight, ramp, tt4_max, tau, &[tau], 8);
        let p = ident.rows[0].p_mid;
        let win = window(p);
        let mut rows: Vec<OscRow> = Vec::new();
        for &rho in rhos {
            let tg = tau / rho;
            if ramp.ds * (1.0 / tau + 1.0 / tg) > 2.0 {
                // the inherited rung-66 floor, never violated on the shipped grid
                rows.push(OscRow::Skipped { rho, tau_gov: tg });
                continue;
            }
            let (nat, _) = m.stator_march_scoped(
                flight, ramp, None, &leg,
                &MarchScope { tau_gov: Some(tg), ..MarchScope::DEFAULT });
            let b_nat = valve_of(&nat[0]).0;
            let (off, _) = m.stator_march_scoped(
                flight, ramp, None, &leg,
                &MarchScope { tau_gov: Some(tg), b0: Some(b_nat + d_b0),
                              ..MarchScope::DEFAULT });
            let n = nat.len().min(off.len());
            let dq: Vec<f64> = (0..n).map(|i| valve_of(&off[i]).0 - valve_of(&nat[i]).0).collect();
            let dg: Vec<f64> =
                (0..n).map(|i| asym_extra(&off[i]).0 - asym_extra(&nat[i]).0).collect();
            let complex_here = win.opens
                && win.rho_lo.is_some_and(|lo| lo < rho)
                && win.rho_hi.is_some_and(|hi| rho < hi);
            let (nq, ng) = (sign_changes(&dq), sign_changes(&dg));
            let peak: Vec<f64> = dq.iter().map(|v| v.abs()).collect();
            rows.push(OscRow::Live(OscLive {
                rho,
                tau_gov: tg,
                npts: n,
                complex_predicted: complex_here,
                sign_changes_q: nq,
                sign_changes_g: ng,
                rings: nq.max(ng) >= RINGS,
                d0: dq[0],
                d_end: dq[n - 1],
                // Python's `if dq[0] else float("nan")` — a TRUTHINESS test, so `-0.0` takes the
                // NaN arm as well as `+0.0`. Spelled as `!= 0.0`, which agrees on both.
                survives: if dq[0] != 0.0 { dq[n - 1].abs() / dq[0].abs() } else { f64::NAN },
                d_peak: py_max_of(&peak),
            }));
        }
        let live: Vec<&OscLive> = rows.iter().filter_map(|r| match r {
            OscRow::Live(x) => Some(x),
            OscRow::Skipped { .. } => None,
        }).collect();
        let survives: Vec<f64> = live.iter().map(|x| x.survives).collect();
        OscillationWindow {
            tt4_max,
            tau,
            ds: ramp.ds,
            r: ramp.r,
            d_b0,
            p,
            window: win,
            rhos: rhos.to_vec(),
            n_complex: live.iter().filter(|x| x.complex_predicted).count(),
            n_real: live.iter().filter(|x| !x.complex_predicted).count(),
            max_sign_changes: live.iter().map(|x| x.sign_changes_q.max(x.sign_changes_g))
                                  .max().unwrap_or(0),
            rings_anywhere: live.iter().any(|x| x.rings),
            survives_max: py_max_default(&survives, f64::NAN),
            rows,
        }
    }

    /// **RUNG 67's PROTECTION LEDGER** — see [`CrossBill`].
    ///
    /// Both loops are LAGGED in every cell, rung 66's discipline verbatim: a lagged loop against
    /// an INSTANTANEOUS one is not a control but a different plant.
    #[allow(clippy::too_many_arguments)]
    pub fn cross_bill(
        &self, flight: &FlightCondition, ramp: &Ramp, tt4_max: f64, tau: f64, tau_gov: f64,
    ) -> CrossBill {
        let own = self.fuel.inner.lever.lim
                      .expect("rung-67's cross_bill needs the machine's own valve");
        let valve = BleedLimiter::with_tau(own.phi_lim, own.b_max, Some(tau));
        let cell = |blim: Option<BleedLimiter>, tg: Option<f64>| -> CrossBillCell {
            let arm = match blim { Some(v) => LeverArm::floored(v), None => LeverArm::default() };
            let m = self.at_lever(&arm);
            // The redline is armed EXACTLY WHEN the clock is: a `Tt4_max` without a `tau_gov`
            // would be rung 46's UNLAGGED governor, which is a third plant and not a control.
            let leg = StatorLeg { accel: None, surge: None,
                                  tt4_max: if tg.is_some() { Some(tt4_max) } else { None } };
            let (traj, _) = m.stator_march_scoped(
                flight, ramp, None, &leg,
                &MarchScope { tau_gov: tg, ..MarchScope::DEFAULT });
            let last = &traj[traj.len() - 1];
            let t4: Vec<f64> = traj.iter().map(|p| p.tt4).collect();
            let pos: Vec<f64> =
                traj.iter().filter(|p| p.s > 0.0).map(|p| p.phi_lp).collect();
            CrossBillCell {
                i_t: exceed(&traj, tt4_max, ramp.r),
                i_phi: violation(&traj, own.phi_lim, ramp.r),
                npts: traj.len(),
                s_last: last.s,
                truncated: last.s < (ramp.r + ramp.s_settle) - 0.5 * ramp.ds,
                max_tt4: py_max_of(&t4),
                min_phi: py_min_of(&pos),
                removed: self.removed_over(&traj),
                nu_lp_end: last.nu_lp,
                nu_hp_end: last.nu_hp,
                thrust_end: last.sp_thrust * last.mdot_air,
            }
        };
        let bare = cell(None, None);
        let gov = cell(None, Some(tau_gov));
        let valve_cell = cell(Some(valve), None);
        let both = cell(Some(valve), Some(tau_gov));
        let (t0, f0) = (bare.i_t, bare.i_phi);
        // Python's `cred` — `NaN` when the bare cell had no violation to remove at all.
        let cred = |x: f64, base: f64| -> f64 {
            if base > 0.0 { 1.0 - x / base } else { f64::NAN }
        };
        let (ct_gov, ct_valve, ct_both) =
            (cred(gov.i_t, t0), cred(valve_cell.i_t, t0), cred(both.i_t, t0));
        let (cf_gov, cf_valve, cf_both) =
            (cred(gov.i_phi, f0), cred(valve_cell.i_phi, f0), cred(both.i_phi, f0));
        CrossBill {
            tt4_max,
            tau,
            tau_gov,
            ds: ramp.ds,
            r: ramp.r,
            phi_lim: own.phi_lim,
            bare,
            gov,
            valve: valve_cell,
            both,
            credit_t_gov: ct_gov,
            credit_t_valve: ct_valve,
            credit_t_both: ct_both,
            credit_phi_gov: cf_gov,
            credit_phi_valve: cf_valve,
            credit_phi_both: cf_both,
            erosion_gov: if (ct_both - ct_valve) > 0.0 { ct_gov / (ct_both - ct_valve) }
                         else { f64::INFINITY },
            erosion_valve: if (cf_both - cf_gov) > 0.0 { cf_valve / (cf_both - cf_gov) }
                           else { f64::INFINITY },
            marginal_gov_t: ct_both - ct_valve,
            marginal_valve_phi: cf_both - cf_gov,
            valve_on_t: ct_valve,
            gov_on_phi: cf_gov,
            valve_debits_t: ct_valve < 0.0,
            gov_credits_phi: cf_gov > 0.0,
            sum_alone_t: ct_gov + ct_valve,
            sum_alone_phi: cf_gov + cf_valve,
        }
    }

    /// **RUNG 66 § 8's CONCESSION, DISCHARGED** — see [`MarginalModeCross`].
    #[allow(clippy::too_many_arguments)]
    pub fn marginal_mode_cross(
        &self, flight: &FlightCondition, ramp: &Ramp, tt4_max: f64, tau: f64, tau_gov: f64,
        d_b0: f64,
    ) -> MarginalModeCross {
        let own = self.fuel.inner.lever.lim
                      .expect("rung-67's marginal_mode_cross needs the machine's own valve");
        let m = self.at_lever(&LeverArm::floored(
            BleedLimiter::with_tau(own.phi_lim, own.b_max, Some(tau))));
        let leg = StatorLeg { accel: None, surge: None, tt4_max: Some(tt4_max) };

        let run = |b0: Option<f64>| -> MarginalCrossCell {
            let (traj, _) = m.stator_march_scoped(
                flight, ramp, None, &leg,
                &MarchScope { tau_gov: Some(tau_gov), b0, ..MarchScope::DEFAULT });
            let on = traj.iter().filter(|p| asym_extra(p).1 > 0.0).count();
            let b_first = valve_of(&traj[0]).0;
            let last = &traj[traj.len() - 1];
            let drift: Vec<f64> = traj.iter().map(|p| (valve_of(p).0 - b_first).abs()).collect();
            let pos: Vec<f64> = traj.iter().filter(|p| p.s > 0.0).map(|p| p.phi_lp).collect();
            let tb: Vec<f64> =
                traj.iter().map(|p| { let (b, c) = valve_of(p); (b - c).abs() }).collect();
            let tg: Vec<f64> =
                traj.iter().map(|p| { let (g, r) = asym_extra(p); (g - r).abs() }).collect();
            MarginalCrossCell {
                b0: b_first,
                b_end: valve_of(last).0,
                g_end: asym_extra(last).0,
                drift: py_max_of(&drift),
                removed: self.removed_over(&traj),
                i_phi: violation(&traj, own.phi_lim, ramp.r),
                i_t: exceed(&traj, tt4_max, ramp.r),
                min_phi_lp: py_min_of(&pos),
                track_b: py_max_of(&tb),
                track_g: py_max_of(&tg),
                n_on: on,
                npts: traj.len(),
                ic_iters: cross_extra(&traj[0]).0,
            }
        };

        let nat = run(None);
        let b_nat = nat.b0;
        let mut moved = Vec::new();
        for (lbl, x) in [("lo", b_nat - d_b0), ("hi", b_nat + d_b0)] {
            assert!(x > 0.0 && x < own.b_max,
                    "rung-67 b0 sweep leaves the valve's stops at {lbl}: {x:.6} not in \
                     (0, {}).", own.b_max);
            moved.push(run(Some(x)));
        }
        let (lo, hi) = (moved[0], moved[1]);
        let span = (hi.removed - lo.removed).abs();
        let span_f = (hi.i_phi - lo.i_phi).abs();
        MarginalModeCross {
            tt4_max,
            tau,
            tau_gov,
            d_b0,
            r: ramp.r,
            ds: ramp.ds,
            phi_lim: own.phi_lim,
            // (i) does a `b0` offset survive to the END? rung 66: -8e-10 (it did not, because the
            // valve hit its stop). Here the mechanism is the SPECTRUM.
            db_db0: (hi.b_end - lo.b_end) / (2.0 * d_b0),
            // (ii) does the PATH remember it? THIS is the number the concession is about.
            dremoved: span,
            dremoved_rel: span / nat.removed.abs(),
            d_i_phi: span_f,
            d_i_phi_rel: if nat.i_phi > 0.0 { span_f / nat.i_phi } else { f64::NAN },
            drift: nat.drift,
            track_b: nat.track_b,
            track_g: nat.track_g,
            natural: nat,
            moved_lo: lo,
            moved_hi: hi,
            b_natural: b_nat,
        }
    }

    /// **THE JOINT IC WHERE RUNG 66's WOULD HAVE STALLED** — see [`JointIcCorners`].
    ///
    /// # The one place this port catches a panic, and what it costs
    ///
    /// Python wraps the march in `except AssertionError` and records `str(e)[:120]`. The port's
    /// equivalent is [`std::panic::catch_unwind`], and `AssertUnwindSafe` is legitimate here for
    /// the reason the guards exist: every dynamically-scoped field on this core is restored by
    /// `Drop`, which runs on the unwind, so the machine on the far side is the machine that went
    /// in.
    ///
    /// **THE PANIC HOOK IS NOT TOUCHED, AND THAT IS A DELIBERATE DIVERGENCE.** Python prints
    /// nothing; Rust's default hook writes one line to stderr per caught panic, and four of the
    /// eight shipped corners raise. Suppressing it means installing a process-global hook, which
    /// races with the `set_hook`/`take_hook` pairs the test files already use — a real hazard
    /// traded for cosmetic quiet. **No VALUE differs**; only stderr does, and it is recorded here
    /// rather than fixed silently.
    ///
    /// The truncation is by CHARACTERS (`str[:120]` is Python's), which for these ASCII messages
    /// is the same as bytes — stated rather than assumed, because it would not be for a message
    /// carrying an em dash. Both halves are gated in `slice_z_smoke.rs`, because **no shipped
    /// grid raises here**: 0 of 8 corners at `ds = 0.01`, and the suite's own call gives four
    /// converged rows.
    ///
    /// # Python's `Tt4_lo` parameter is DEAD and this signature drops it
    ///
    /// `joint_ic_corners(self, flight, Tt4_lo, Tt4_hi, Tt4_maxes=…, Tt4_los=…)` never reads
    /// `Tt4_lo` — every march takes its start from `Tt4_los`. The [`Ramp`] below therefore
    /// supplies `tt4_hi`, `r`, `s_settle` and `ds`, and its own `tt4_lo` is OVERWRITTEN per
    /// corner, which is what Python does with the value it ignores. Named rather than silently
    /// dropped: a reader comparing the two signatures should be told the omission was measured.
    ///
    /// Python's return also keys the tuple of starts as `"Tt4_lo"` (singular) while holding
    /// `Tt4_los`; the port names it [`JointIcCorners::tt4_los`] and step 4's dump maps it.
    #[allow(clippy::too_many_arguments)]
    pub fn joint_ic_corners(
        &self, flight: &FlightCondition, ramp: &Ramp, tt4_maxes: &[f64], tt4_los: &[f64],
        tau: f64, tau_gov: f64,
    ) -> JointIcCorners {
        let own = self.fuel.inner.lever.lim
                      .expect("rung-67's joint_ic_corners needs the machine's own valve");
        let mut rows: Vec<IcCorner> = Vec::new();
        for &lo in tt4_los {
            for &tm in tt4_maxes {
                let m = self.at_lever(&LeverArm::floored(
                    BleedLimiter::with_tau(own.phi_lim, own.b_max, Some(tau))));
                let corner = Ramp { tt4_lo: lo, ..*ramp };
                let leg = StatorLeg { accel: None, surge: None, tt4_max: Some(tm) };
                let ran = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    m.stator_march_scoped(
                        flight, &corner, None, &leg,
                        &MarchScope { tau_gov: Some(tau_gov), ..MarchScope::DEFAULT })
                }));
                let traj = match ran {
                    Ok((t, _)) => t,
                    Err(e) => {
                        let msg = match e.downcast_ref::<String>() {
                            Some(s) => s.clone(),
                            None => e.downcast_ref::<&str>().map(|s| (*s).to_string())
                                     .unwrap_or_else(|| "<non-string panic>".into()),
                        };
                        rows.push(IcCorner::Failed {
                            tt4_lo: lo, tt4_max: tm,
                            failed: msg.chars().take(120).collect(),
                        });
                        continue;
                    }
                };
                let p0 = &traj[0];
                let (iters, res, damp) = cross_extra(p0);
                rows.push(IcCorner::Ok(IcOk {
                    tt4_lo: lo,
                    tt4_max: tm,
                    live: asym_extra(p0).1 > 0.0,
                    required0: asym_extra(p0).1,
                    b0: valve_of(p0).0,
                    g0: asym_extra(p0).0,
                    ic_iters: iters,
                    ic_res: res,
                    ic_damp: damp,
                    npts: traj.len(),
                }));
            }
        }
        let ok: Vec<&IcOk> = rows.iter().filter_map(|r| match r {
            IcCorner::Ok(x) => Some(x),
            IcCorner::Failed { .. } => None,
        }).collect();
        JointIcCorners {
            tt4_los: tt4_los.to_vec(),
            tt4_maxes: tt4_maxes.to_vec(),
            tau,
            tau_gov,
            ds: ramp.ds,
            n_live: ok.iter().filter(|x| x.live).count(),
            all_converged: ok.iter().all(|x| x.ic_res <= 1e-9),
            max_iters: ok.iter().map(|x| x.ic_iters).max().unwrap_or(0),
            ever_damped: ok.iter().any(|x| x.ic_damp < 1.0),
            rows,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// THE CONSTRUCTOR
// ---------------------------------------------------------------------------------------------

/// RUNG 67's object — like [`build_two_lag_cascade`], **rung 65's constructor with the TABLES
/// swapped and nothing else**, and for the same measured reason: `CrossLoopCascadeTransient`
/// defines no `__init__` and rebinds no class attribute except `_tau_gov` (the carrier) and
/// [`RINGS`] (a reader's threshold). None of rung 65's ten construction asserts moves.
///
/// [`build_two_lag_cascade`]: crate::two_lag::build_two_lag_cascade
pub fn build_cross_loop_cascade(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    let built = ScheduledStatorTransient::with_tables(
        design_engine, flight_design, mdot_design, map_lp, map_hp, rho, arm.stator,
        &R67_TWO, &R67_STATOR, &R67_FUEL, &R67,
        LeverArming { bleed: arm.bleed, sched: arm.bleed_sched, lim: arm.bleed_lim });
    // Rung 62's two, in Python's order.
    assert!(!(arm.bleed != 0.0 && arm.bleed_sched.is_some()),
            "rung-62: the valve gets a CONSTANT position or a SCHEDULE, not both -- they are the \
             two legs the rung differences (rung 57's discipline).");
    assert!((0.0..0.5).contains(&arm.bleed),
            "rung-42 bleed fraction must be in [0, 0.5): b>=0.5 starves the core and the choked \
             branch is long gone by then");
    // Rung 64's three-way arming exclusion.
    assert!(!(arm.bleed_lim.is_some() && (arm.bleed != 0.0 || arm.bleed_sched.is_some())),
            "rung-64: the valve gets a CONSTANT position (42), a SCHEDULE (62) or a FLOOR (64) -- \
             exactly one. They are the three legs this rung differences, and rung 62's two-way \
             assert is extended rather than replaced.");
    built
}

// ---------------------------------------------------------------------------------------------
// THE TABLES — the same three cells, one rung further up
// ---------------------------------------------------------------------------------------------

/// RUNG 67's lever table — ONE cell, `at_lever`.
pub const R67: LeverHooks = LeverHooks {
    at_lever: r67_at_lever,
    ..crate::two_lag::R66
};

/// RUNG 67's `TwoSpoolTransientHooks` — **ZERO cells swapped**, named for [`R66_TWO`]'s reason.
///
/// [`R66_TWO`]: crate::two_lag::R66_TWO
pub const R67_TWO: TwoSpoolTransientHooks = crate::two_lag::R66_TWO;

/// RUNG 67's fuel table — ONE cell, `integrate_fuel`.
pub const R67_FUEL: FuelTransientHooks = FuelTransientHooks {
    integrate_fuel: r67_integrate_fuel,
    ..crate::two_lag::R66_FUEL
};

/// RUNG 67's stator table — ONE cell, the march that carries `tau_gov`.
pub const R67_STATOR: StatorTransientHooks = StatorTransientHooks {
    stator_march: r67_stator_march,
    ..crate::two_lag::R66_STATOR
};
