//! RUNG 66 — the TWO-LAG CASCADE: what a SECOND limiter buys, and what it provably cannot.
//!
//! Slice Z, first of two. Python's `TwoLagCascadeTransient` (`turbojet/engine.py`, **653 lines**,
//! 11 methods), ported onto [`crate::lagged_bleed`]'s shape: a lagged bleed VALVE beside a lagged
//! FUEL leg, **both watching `phi_lp`**. Four states, two clocks, and no new control law — rung
//! 52's `AsymmetricLag` over rung 49's floor, merged with rung 65's valve.
//!
//! **HEADLINE (the rung's): two loops on one variable are ONE loop with the RATES ADDED.** Both
//! laws are implicit functions of the same constraint `phi(w, b) = phi_lim`, so their cross-gains
//! are reciprocals *by construction* — `R_q · C_g ≡ 1` is an IDENTITY, not a locus, hence
//! `det J ≡ 0`. The pair has ONE effective actuator direction, so the credits cannot add: 60.46 %
//! and 92.51 % alone deliver **94.09 %** together. A second limiter — its own sensor, law,
//! actuator and clock — buys BANDWIDTH, not AUTHORITY.
//!
//! # What slice Z adds to the table
//!
//! **ZERO new cells** — probe 1's emitted census, and the THIRD row of § 5.19 (x)'s cell column
//! an emitter confirms (after X's 1 and Y's 0). Rungs 66 and 67 swap the **same three**
//! already-open cells: `stator_march` (opened at slice V), `at_lever` (slice W) and
//! `integrate_fuel` (slice Y). Nothing here re-opens a signature.
//!
//! # The dispatch and its three refusals ship SEPARATELY from the march, and that was the point
//!
//! [`r66_integrate_fuel`]'s dispatch and its three asserts are complete and live — which means
//! the reduce arms are gate-able **before a single line of the march exists**, and that is the
//! point of splitting the step: rung 66 reduces to rung 64 (`_lagged()` false), to rung 65
//! (`lag is None`) and to rung 52, all three BY DISPATCH, and all three route through code that
//! already ships. Step 1 shipped this file with [`r66_integrate_fuel_cascade`] stubbed for
//! exactly that reason; step 2 filled it in and the reduce gates did not move.
//!
//! # The precedence no value key can see
//!
//! Python opens with `lag = lag if lag is not None else self._lag` — **the ARGUMENT wins over the
//! carrier**, and the RESOLVED value is what gets forwarded to `super()`. On every shipped grid
//! at most one of the two is ever set, so a port that reads the carrier first, or that forwards
//! the raw argument instead of the resolved one, agrees everywhere. Both are spelled explicitly
//! below for that reason.

use crate::bleed_transient::{r62_try_close_fuel, LeverArm, LeverArming, LeverHooks};
use crate::engine::FlightCondition;
use crate::fuel_transient::{asym_extra, point, AccelSchedule, AsymmetricLag, Floor, FuelInstant,
                            FuelLimiters, FuelPoint, FuelTransientCore, FuelTransientHooks,
                            PointExtra, SurgeLimiter};
use crate::gas::Abort;
use crate::lagged_bleed::{lagged, valve_of};
use crate::limited_bleed::BleedLimiter;
use crate::map::ComponentMap;
use crate::stator_transient::{MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient,
                              StatorLeg, StatorTransientHooks};
use crate::two_spool::{Spool, TwoSpoolEngine};
use crate::two_spool_transient::{ForcedBleed, LaggedFuel, MarchedBleed, TwoSpoolTransientHooks};

// ---------------------------------------------------------------------------------------------
// THE CELLS — three swaps, zero additions
// ---------------------------------------------------------------------------------------------

/// RUNG 66's `_stator_march` — rung 65's march with ONE addition, `lag`.
///
/// **THE FUEL LAG IS A PER-MARCH ARGUMENT AND NOT A MACHINE KEYWORD**, which is rung 65's `b0`
/// discipline verbatim and the reason [`r66_at_lever`] has nothing to drop: *a sibling
/// constructor cannot drop what it never carries.*
///
/// **THE GUARD RESTORES THE PREVIOUS VALUE** ([`LaggedFuel`], not a restore-to-`None`) — Python is
/// `prev, self._lag = self._lag, lag` … `finally: self._lag = prev`. Probe 3 measured max nesting
/// depth **1** with **0** nested events over rungs 62–67, so the difference is invisible to every
/// value key and step 5 manufactures the nest (§ 5.24 P7).
///
/// `lag = None` is a real assignment: a rung-66 march called WITHOUT it CLEARS an outer one for
/// the duration.
pub fn r66_stator_march(
    ft: &FuelTransientCore, flight: &FlightCondition, ramp: &Ramp, nu0: Option<(f64, f64)>,
    leg: &StatorLeg<'_>, scope: &MarchScope,
) -> (Vec<FuelPoint>, (f64, f64)) {
    let _g = LaggedFuel::set(&ft.inner, scope.lag);
    // Python forwards to `super()._stator_march(...)` WITHOUT `lag` — the parameter is CONSUMED
    // here and `b0` is passed on. Each rung consumes exactly one field and forwards the rest,
    // which is why the scope is rebuilt rather than passed through.
    crate::lagged_bleed::r65_stator_march(
        ft, flight, ramp, nu0, leg, &MarchScope { b0: scope.b0, ..MarchScope::DEFAULT })
}

/// RUNG 66's `at_lever` — rung 65's sibling constructor returning THIS class.
///
/// **THE FIFTH INSTANCE OF ONE TRAP, AND THE OVERRIDE IS ONE WORD.** Rungs 61/62/63/64/65 each
/// hit it: the inherited constructor hardcodes its own class name, so a rung-66 machine calling
/// rung 65's `at_lever` would silently hand back a rung-65 one. In Rust that word is which
/// BUILDER is called, so deleting this override is caught by the tables the sibling carries
/// rather than by any value key on the machine it returns.
///
/// The fuel lag is a per-MARCH argument, so — exactly as at rung 65 — there is no lag keyword
/// here for a sibling constructor to drop.
pub fn r66_at_lever(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    match build_two_lag_cascade(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(), arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("at_lever never disables LP"),
    }
}

/// RUNG 66's `integrate_fuel` — **the dispatch, and the three refusals that keep the rung from
/// silently becoming a different one.**
///
/// # The three reduce arms all leave through the SAME `if`
///
/// The merged integrator is entered only when BOTH clocks are actually armed, which is what makes
/// every arm bit-for-bit **by dispatch** rather than by numerical agreement:
///
/// | arm | condition | lands on |
/// |---|---|---|
/// | rung 64 | `_lagged()` false, no `lag` | rung 43's march, through rung 65's cell |
/// | rung 65 | `lag is None` | `r65_integrate_fuel_valve_lag` |
/// | rung 52 | `_lagged()` false, `lag` set | `integrate_fuel_asym` |
///
/// Probe 3 measured **all four** `(armed, arg)` arms of this dispatch LIVE (18 / 4 / 12 / 32), so
/// unlike slice Y's mirror-zero pair no arm here needs a manufactured gate — a non-recurrence
/// registered by MEASUREMENT rather than assumed away, because the previous slice's headline is
/// exactly what a port is tempted to carry forward on a family resemblance.
pub fn r66_integrate_fuel(
    ft: &FuelTransientCore, flight: &FlightCondition, fuel_schedule: &dyn Fn(f64) -> f64,
    nu0: (f64, f64), s_end: f64, ds: f64, lim: &FuelLimiters<'_>,
) -> Vec<FuelPoint> {
    // Python's `lag = lag if lag is not None else self._lag` — THE ARGUMENT WINS, and the
    // RESOLVED value is what `super()` receives. Both halves matter and neither is visible in a
    // float on any shipped grid (module doc).
    let lag = lim.lag.or_else(|| ft.inner.lag.get());
    if !(lagged(&ft.inner) && lag.is_some()) {
        return crate::lagged_bleed::r65_integrate_fuel(
            ft, flight, fuel_schedule, nu0, s_end, ds,
            &FuelLimiters { lag, ..lim.clone() });
    }
    assert!(lim.tau_gov.is_none(),
            "rung-66 takes CASCADE B: rung 52's phi-referenced fuel lag beside rung 65's phi \
             valve -- two loops on ONE variable, which is what rung 65 s 3's marginal mode is \
             about. Rung 47's tau_gov watches Tt4, a DIFFERENT variable, so that pairing \
             (cascade A) tests rung 52 s 3's non-additivity instead. Its cross-gains have \
             OPPOSITE signs and it therefore admits an oscillatory mode this one provably \
             cannot -- a separate rung, asserted against rather than run.");
    assert!(lim.s_off.is_none() && lim.tau_rel.is_none(),
            "rung-66: rungs 50/51's FORCED release edges are an isolation instrument for a leg \
             that could not pin its own trigger. BOTH legs here pin their own (rung 52's \
             argument on the fuel side, rung 65's on the valve), so forcing one would measure \
             the forcing. `lag.tau_rel` -- the RATE the fuel leg hands its clip back at -- is \
             a different object and is exactly what this rung sweeps.");
    assert!(lim.accel.is_some() || lim.floor().is_some(),
            "rung-66's fuel lag lags a min-select LEG's clip -- arm one (accel/surge). With \
             neither armed `required == 0` identically and the fuel clock has nothing to run \
             on, which would silently reduce the cascade to rung 65 while claiming four \
             states.");
    r66_integrate_fuel_cascade(ft, flight, fuel_schedule, nu0, s_end, ds, lim,
                               lag.expect("the dispatch above proves this is Some"))
}

/// RUNG 66's MARCH — rung 52's `_integrate_fuel_asym` and rung 65's
/// `_integrate_fuel_valve_lag`, MERGED: four states, and the two actuators coupled ONLY through
/// the plant.
///
/// `g`/`required` (rung 52's keys) and `b`/`b_cmd` (rung 65's) are ALL recorded per point, so
/// both tracking errors read straight off one trajectory and **every rung-52 and rung-65 reader
/// works unchanged on it** — a claim about the READERS, and the reason `asym_extra` and
/// `valve_of` were widened rather than left refusing this route.
///
/// # The `_b_state` boundary, which is the one thing here that can go wrong silently
///
/// Every closure call that represents THE PLANT ([`FuelTransientCore::try_instant_fuel`],
/// `try_topping_fuel`, `try_sched_fuel`, `try_surge_fuel`) runs with [`MarchedBleed`] holding the
/// LIVE position; only `command`, which roots rung 64's valve over TRIAL positions, runs without
/// it. Get it backwards and a solver converges on a residual the plant never uses, **with no test
/// failing** — rung 62's `_powers` trap, reloaded. In Rust the boundary is a SCOPE rather than a
/// `finally`, and `command` sits outside it exactly where Python's call sits after the `finally`.
///
/// # `Tt4_max` takes RUNG 52's placement, not rung 65's
///
/// The redline is min-selected UNLAGGED on top of the ALREADY-CLIPPED fuel (`mf_sched - g`),
/// because this extends the LAGGED leg's integrator. Rung 65 puts it inside the caps at
/// `mf_sched` instead; the two disagree, nothing would catch a wrong pick, and cascade B arms
/// `surge` alone — so every rung-66 diagnostic passes `Tt4_max = None` and the ambiguity never
/// runs. **Rung 67 is where the choice finally executes**, and there it is rung 47's placement.
#[allow(clippy::too_many_arguments)]
pub fn r66_integrate_fuel_cascade(
    ft: &FuelTransientCore, flight: &FlightCondition, fuel_schedule: &dyn Fn(f64) -> f64,
    nu0: (f64, f64), s_end: f64, ds: f64, lim: &FuelLimiters<'_>,
    lag: AsymmetricLag,
) -> Vec<FuelPoint> {
    let bl = ft.inner.lever.lim.expect("rung-66's march on an unfloored machine");
    let tau = bl.tau.expect("rung-66's march on an unlagged machine");
    let (freeze, tt4_max, accel) = (lim.freeze, lim.tt4_max, lim.accel);
    let surge = lim.floor();
    // THE MODELLING FLOOR -- rung 65's, and THE RATES ADD. Rung 65 published a RETRACTION: an RK4
    // instability at z = ds/tau = 5 returned an `int b ds` 4.4x the converged value and looked
    // exactly like a physical finding. A cascade has TWO clocks, and the naive transfer -- bound
    // the FASTEST one, `ds/min(tau) <= 2` -- IS WRONG, in the unsafe direction, by up to 2x.
    //
    // WHY, AND IT IS THIS RUNG'S OWN IDENTITY: two loops holding ONE variable to ONE set point
    // have `R_q C_g == 1` identically, so `det J == 0` and the eigenvalues are exactly
    // {0, tr J} = {0, -(1/t_g + 1/t_v)}. THE TWO RATES ADD. At MATCHED clocks this is
    // `ds/tau <= 1.0`, HALF of rung 65's single-state bound: a sweep that inherited rung 65's
    // constant would run at twice the step this rung admits.
    let rate = 1.0 / tau + 1.0 / lag.tau_att.min(lag.tau_rel);
    assert!(ds * rate <= 2.0,
            "rung-66: ds*(1/tau_v + 1/tau_g) = {:.3} is outside the explicit RK4 stability \
             region for the two actuator states (ds = {ds}, tau_v = {tau}, lag = {}/{}). THE \
             RATES ADD -- det J == 0 makes the non-zero eigenvalue exactly -(1/t_g + 1/t_v) -- \
             so bounding the fastest clock alone is optimistic by up to 2x. Refine the grid or \
             slow a clock; BOTH tau -> 0 limits are APPROACHED on this integrator and never \
             reached.", ds * rate, lag.tau_att, lag.tau_rel);
    let (tt2, pt2, _) = ft.inner.inlet(flight);

    // Rung 64's instantaneous root at THIS state and fuel, WITHOUT the march state: it roots over
    // TRIAL positions, so it must not see the live one. It does not read `q`, which is what keeps
    // `dq/ds` affine in `q` (rung 65's RK4-legality argument).
    let command = |a: f64, h: f64, mf: f64| -> Result<f64, Abort> {
        Ok(crate::limited_bleed::r64_solve_b(&bl, |b| {
            let _g = ForcedBleed::set(&ft.inner, b);
            r62_try_close_fuel(ft, a, h, mf, tt2, pt2)
        })?.1)
    };

    // Rung 52's clip requirement, on the plant AS THE VALVE ACTUALLY IS. Solved from the
    // SCHEDULED fuel (rung 52's discipline verbatim) so arming one leg cannot perturb the other's
    // bracket. **`min(caps)` is RAW here and the FLOOR is on the RESULT** — rung 65's cell
    // `retain`s the caps below `mf_sched` and falls back to `mf_sched` itself, which is a
    // different branch structure at the `cap == mf_sched` boundary. Rung 52's shape is the one
    // this rung merges, so it is rung 52's that is copied.
    let required = |a: f64, h: f64, q: f64, mf_sched: f64| -> Result<f64, Abort> {
        let _st = MarchedBleed::set(&ft.inner, q);
        let mut caps: Vec<f64> = Vec::new();
        if let Some(sch) = accel {
            caps.push(ft.try_sched_fuel(flight, a, h, mf_sched, sch)?);
        }
        if let Some(fl) = &surge {
            caps.push(ft.try_surge_fuel(flight, a, h, mf_sched, fl)?);
        }
        if caps.is_empty() {
            return Ok(0.0);
        }
        let mut m = caps[0];
        for &c in &caps[1..] {
            if c < m { m = c; }
        }
        Ok(0.0f64.max(mf_sched - m))
    };

    type Der = (f64, f64, f64, f64, f64, FuelInstant, f64, f64);
    let der = |a: f64, h: f64, g: f64, q: f64, s: f64| -> Result<Der, Abort> {
        let mf_sched = fuel_schedule(s);
        let req = required(a, h, q, mf_sched)?;
        // Python's `max(1e-9, x)` — the LITERAL first, which is what decides the NaN case:
        // Python holds the first operand and replaces it only on a strict `>`, and `f64::max`
        // with the literal as the receiver agrees. `x.max(1e-9)` would NOT.
        let mut mf = 1e-9f64.max(mf_sched - g);
        let inst = {
            let _st = MarchedBleed::set(&ft.inner, q);
            if let Some(t4) = tt4_max {
                // The UNLAGGED redline, rung 52's placement — and it reads the CLIPPED `mf`.
                if ft.try_instant_fuel(flight, a, h, mf)?.base.tt4 > t4 {
                    let c = ft.try_topping_fuel(flight, a, h, t4, mf)?;
                    if c < mf { mf = c; }
                }
            }
            // UNCONDITIONAL, unlike rung 65's cell, which re-solves only when a cap bound. With
            // the redline armed that is always TWO instant solves per derivative.
            ft.try_instant_fuel(flight, a, h, mf)?
        };
        // OUTSIDE the state's scope, exactly where Python's call sits after the `finally`.
        let cmd = command(a, h, mf)?;
        let da = if freeze == Some(Spool::Lp) { 0.0 } else { inst.base.phi_lp_dot / ft.rho() };
        let dh = if freeze == Some(Spool::Hp) { 0.0 } else { inst.base.phi_hp_dot };
        Ok((da, dh, (req - g) / lag.tau(req, g), (cmd - q) / tau, mf, inst, req, cmd))
    };

    // --- THE JOINT INITIAL CONDITION ----------------------------------------------------------
    // `g` and `q` are each other's arguments, so neither rung 52's `g = 0` nor rung 65's
    // `q = b_cmd(0)` is by itself the equilibrium of the pair. **THE ITERATION IS THE
    // DIAGNOSTIC**: its contraction factor is `|R_q C_g|`, which this rung's identity pins at 1
    // wherever BOTH laws ride, so it converges only because a march OPENS DORMANT. Divergence
    // here is the marginal mode announcing itself at `s = 0`, not a numerical nuisance.
    //
    // **IT IS INLINE AND UNDAMPED, AND THAT IS THE POINT (§ 5.24 P6).** Rung 67 solves the same
    // object through a DAMPED sweep, because on cascade A `|P|` is pinned by nothing and a stall
    // there would be a SOLVER failure rather than a finding. Routing this through that solver is
    // bit-exact on the shipped grid (`w = 1.0` on 36 of 39 calls) and destroys the distinction
    // the two asserts exist for — the standing *copy vs re-derivation* rule, and the reflex it
    // exists to stop.
    let (mut a, mut h) = nu0;
    let mf0 = fuel_schedule(0.0);
    let b0 = ft.inner.b0.get();
    if let Some(x) = b0 {
        assert!((0.0..=bl.b_max).contains(&x),
                "rung-66 b0 is a valve POSITION: {x} is outside [0, {}]", bl.b_max);
    }
    // Python raises out of the whole method here — the initial solves sit BEFORE the loop's
    // `try`, so neither is one of the two `break` sites. `joint_ic_corners` (rung 67) is the only
    // caller that catches it, and it catches an AssertionError, which is what this panic is.
    let ic = |r: Result<f64, Abort>| -> f64 { r.unwrap_or_else(|e| panic!("{}", e.0)) };
    let mut g = 0.0f64;
    let mut q = match b0 { Some(x) => x, None => ic(command(a, h, mf0)) };
    let mut res = f64::INFINITY;
    let mut its = 0usize;
    for k in 1..=60usize {
        its = k;
        let gn = ic(required(a, h, q, mf0));
        let qn = match b0 { Some(_) => q, None => ic(command(a, h, 1e-9f64.max(mf0 - gn))) };
        res = (gn - g).abs().max((qn - q).abs());
        g = gn;
        q = qn;
        if res <= 1e-12 { break; }
    }
    assert!(res <= 1e-9,
            "rung-66: the joint initial condition did not converge (residual {res:.3e} after \
             {its} iterations). The iteration contracts at |R_q C_g|, so this is the DEGENERACY \
             LOCUS `R_q C_g = 1` -- det J = 0, the marginal mode -- present already at s = 0. It \
             is a finding, not a solver failure: report the state, do not raise the iteration \
             cap.");

    let mut pts: Vec<FuelPoint> = Vec::new();
    let mut s = 0.0f64;
    // `s` ACCUMULATED and `round_ties_even`, both INHERITED from rung 43's marcher rather than
    // re-decided — see `FuelTransientCore::integrate_fuel`'s note on why `k as f64 * ds` flips a
    // published boolean.
    let n_steps = (s_end / ds).round_ties_even() as i64;
    for _ in 0..=n_steps {
        let Ok((k1a, k1h, k1g, k1q, mf_app, inst, req, cmd)) = der(a, h, g, q, s) else { break };
        pts.push(point(s, a, h, &inst, mf_app, fuel_schedule(s),
                       PointExtra::Cascade { g, required: req, b: q, b_cmd: cmd,
                                             ic_iters: its, ic_res: res }));
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
        // THE POSITION IS PHYSICAL (rung 65, verbatim): the actuator's own hardware stops, applied
        // to the STATE and never to the command. The CLIP is floored at zero for the same reason
        // — a min-select leg cannot hand back more fuel than it took.
        q = bl.b_max.min(0.0f64.max(q));
        g = 0.0f64.max(g);
        s += ds;
    }
    pts
}

// ---------------------------------------------------------------------------------------------
// THE LEAF STATICS — Python's two `@staticmethod`s, and the two folds that are NOT `f64::min`
// ---------------------------------------------------------------------------------------------

/// **PYTHON'S `min(seq)` / `max(seq)`, WHICH ARE NOT `fold(f64::min)` — the difference is `NaN`.**
///
/// `f64::min` and `f64::max` DISCARD a `NaN` operand; Python's builtins hold the first element and
/// replace it only on a strict comparison, so a `NaN` **in position 0 survives to the end** while
/// one in any later position is overwritten. [`py_max3`] is rung 65's two-argument version of the
/// same fact, found there by asking a reader for its degenerate case.
///
/// **WHY IT IS USED AT EVERY SEQUENCE FOLD IN THIS SLICE RATHER THAN AT A CHOSEN FEW.** Both
/// rungs' core instruments publish per-row values that are `float("nan")` **exactly when that row
/// had no riding points** (`prod_lo`, `prod_hi`, `rho_err`, `survives`), and the aggregates fold
/// those rows. On the shipped grid every row rides, so no value key can separate the two
/// spellings — which is the shape this project has now been caught by four times. Copying Python's
/// fold is not a factoring: Python has ONE builtin here, and this is it.
///
/// [`py_max3`]: crate::lagged_bleed::py_max3
pub fn py_min_of(xs: &[f64]) -> f64 {
    let mut best = xs[0];
    for &x in &xs[1..] {
        if x < best { best = x; }
    }
    best
}

/// [`py_min_of`]'s twin — Python's `max(seq)`.
pub fn py_max_of(xs: &[f64]) -> f64 {
    let mut best = xs[0];
    for &x in &xs[1..] {
        if x > best { best = x; }
    }
    best
}

/// Python's `max(seq, default=d)` — the `default=` form, which returns `d` on an EMPTY sequence
/// and is otherwise [`py_max_of`].
///
/// Probe 9 measured that **no shipped grid empties any of the three sequences** this guards, so
/// the branch ships live and unexercised and step 5 owes it a manufactured gate (§ 5.24 P4).
pub fn py_max_default(xs: &[f64], default: f64) -> f64 {
    if xs.is_empty() { default } else { py_max_of(xs) }
}

/// The 2x2 actuator block's spectrum — Python's `_eig`, **RUNG 66's `@staticmethod` and RUNG 67's
/// too**, which is why it is a free function here rather than a method on either.
///
/// Reported, never asserted: § 3's stability floor is the A-PRIORI sum, precisely because this
/// needs a march to evaluate.
///
/// # THE COMPLEX ARM IS DEAD ON THE RUNG THAT DEFINES IT (§ 5.24 (vi) / P5)
///
/// A census that does not split by CALLING FUNCTION conflates the two rungs and reads 134 real /
/// 57 complex. Split, over `test_rung66.py` alone: **80 of 80 real, and the complex arm never
/// runs on rung 66 at all** — not an accident but rung 66's own headline, since `det J ≡ 0` makes
/// the discriminant `tr² − 4·0 = tr² ≥ 0` identically. It is kept alive one rung up, where
/// `det J ≠ 0`. **A port that drops it passes every rung-66 gate and breaks at rung 67.**
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Eig {
    pub tr: f64,
    pub det: f64,
    pub disc: f64,
    pub real: bool,
    /// The two real roots, or `None` on the complex branch — Python's `lam`.
    pub lam: Option<(f64, f64)>,
    pub rho: f64,
}

/// [`Eig`]'s constructor — `** 0.5` is `.sqrt()` in BOTH places, never `powf(0.5)` (`gas.rs` § 2).
pub fn eig(r_q: f64, c_g: f64, t_g: f64, t_v: f64) -> Eig {
    let tr = -(1.0 / t_g + 1.0 / t_v);
    let det = (1.0 - r_q * c_g) / (t_g * t_v);
    let disc = tr * tr - 4.0 * det;
    if disc >= 0.0 {
        let root = disc.sqrt();
        let (lo, hi) = (0.5 * (tr - root), 0.5 * (tr + root));
        return Eig { tr, det, disc, real: true, lam: Some((lo, hi)),
                     rho: lo.abs().max(hi.abs()) };
    }
    Eig { tr, det, disc, real: false, lam: None, rho: det.abs().sqrt() }
}

/// `∫ max(0, phi_lim − phi_lp) ds` over the ramp — **AN AREA**, and rung 66's primary currency.
///
/// It replaces `min phi`, and the reason is a measurement: on the fuel-leg-alone control the
/// argmin sits at `s = 0`, so `min phi` there is the RUNNING LINE the march starts on and not a
/// protected minimum at all. A credit table built on a clamped extremum is not quotable; an
/// integral cannot be clamped by its own initial condition.
///
/// **IT DROPS THE STRADDLING CELL, AND RUNG 67's [`exceed`] INTERPOLATES ONE — THE TWO MUST NOT
/// BE FOLDED INTO ONE HELPER WITH A FLAG.** The `break` below discards the whole final cell
/// whenever the marched `s` lands a float's width past `s_hi`. On THIS currency that is
/// immaterial (the phi violation is an EARLY-ramp object, its integrand ~0 by `s = r`); on rung
/// 67's it is at its MAXIMUM there. Rung 67 therefore fixed its own and left this one alone, in
/// so many words, *because its numbers are gated*.
///
/// [`exceed`]: crate::cross_loop::exceed
pub fn violation(traj: &[FuelPoint], phi_lim: f64, s_hi: f64) -> f64 {
    let mut out = 0.0;
    for i in 1..traj.len() {
        if traj[i].s > s_hi { break; }
        let h = traj[i].s - traj[i - 1].s;
        out += 0.5 * h * (0.0f64.max(phi_lim - traj[i - 1].phi_lp)
                          + 0.0f64.max(phi_lim - traj[i].phi_lp));
    }
    out
}

/// Python's `_gains` default `dq` — a valve POSITION step, on `[0, b_max ~ 0.1]`.
///
/// The two defaults differ by two orders **because the two arguments do**, and they are named
/// rather than inlined so rung 67's [`gains_cross`](ScheduledStatorCore::gains_cross) can state
/// that it takes the SAME pair — which is Python's own "rung 66, verbatim".
pub const GAINS_DQ: f64 = 1e-5;

/// Python's `_gains` default `dg` — a fuel CLIP step, of order `1e-3` kg/s.
pub const GAINS_DG: f64 = 1e-7;

// ---------------------------------------------------------------------------------------------
// THE READING INSTRUMENTS
// ---------------------------------------------------------------------------------------------

/// One clock's row of [`CascadeIdentity`] — Python's `rows[i]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CascadeIdentityRow {
    pub tau_att: f64,
    pub tau_v: f64,
    /// How many marched points RIDE — `required > 0` AND the valve strictly inside its stops.
    pub n_ride: usize,
    /// How many were SAMPLED. **NOT the caller's `n_sample`** — the sub-sample is a STRIDE,
    /// `ride[::max(1, n_ride / n_sample)]`, so the delivered count is
    /// `n_ride / (n_ride / n_sample)` and lands ABOVE the request. Reading the request instead of
    /// this number is what § 5.24 (i)'s leading finding is about.
    pub n_sample: usize,
    pub n_real: usize,
    pub prod_lo: f64,
    pub prod_hi: f64,
    pub rho_max: f64,
    pub rate_closed_form: f64,
    pub rho_err: f64,
    /// THE CONTROL on the identity: the gains themselves must MOVE, or a constant product is
    /// measuring a constant plant instead of a reciprocal pair. Taken on MAGNITUDES — both gains
    /// are strictly negative, so a raw max/min would invert the ratio and report a 1.7x swing
    /// as 0.57.
    pub gain_span_r: f64,
    pub gain_span_c: f64,
    pub r_q_lo: f64,
    pub r_q_hi: f64,
    pub c_g_lo: f64,
    pub c_g_hi: f64,
    pub ds_rho: f64,
}

/// Python's `cascade_identity` return — **RUNG 66's CORE INSTRUMENT**, § 2's identity MEASURED
/// rather than asserted.
#[derive(Clone, Debug)]
pub struct CascadeIdentity {
    pub sm: f64,
    pub b_cap: f64,
    pub tau: f64,
    pub tau_atts: Vec<f64>,
    pub ds: f64,
    pub r: f64,
    pub phi_lim: f64,
    pub rows: Vec<CascadeIdentityRow>,
    pub all_real: bool,
    pub prod_lo: f64,
    pub prod_hi: f64,
    pub rho_err_max: f64,
}

/// One cell of [`CascadeBill`]'s 2x2 — Python's `cells[name]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CascadeBillCell {
    /// Python's `"I"` — the violation integral [`violation`] returns.
    pub i: f64,
    pub npts: usize,
    pub min_phi: f64,
    pub s_at_min: f64,
    pub s_last: f64,
    pub truncated: bool,
    pub removed: f64,
    pub min_phi_hp: f64,
    pub nu_lp_end: f64,
    pub nu_hp_end: f64,
    pub thrust_end: f64,
}

/// Python's `cascade_bill` return — **RUNG 66's PROTECTION LEDGER**, the 2x2: each lagged loop
/// alone, both, and neither.
///
/// **THE CONTROLS ARE BOTH LAGGED ON PURPOSE.** A pairing of one lagged loop against one
/// INSTANTANEOUS one is not a control, it is a different plant — rung 65 already called the
/// instantaneous limit singular, so any such comparison collapses to *"the instantaneous loop
/// holds the set point"* and measures nothing about redundancy. The comparison with content is
/// ONE FINITE-BANDWIDTH LOOP AGAINST TWO.
#[derive(Clone, Debug)]
pub struct CascadeBill {
    pub sm: f64,
    pub b_cap: f64,
    pub tau: f64,
    pub tau_att: f64,
    pub ds: f64,
    pub r: f64,
    pub phi_lim: f64,
    pub bare: CascadeBillCell,
    pub fuel: CascadeBillCell,
    pub valve: CascadeBillCell,
    pub both: CascadeBillCell,
    pub credit_fuel: f64,
    pub credit_valve: f64,
    pub credit_both: f64,
    pub sum_alone: f64,
    pub delivered: f64,
    pub subadditive: bool,
    pub beats_both: bool,
    pub marginal_fuel: f64,
    pub marginal_valve: f64,
    pub erosion_fuel: f64,
    pub erosion_valve: f64,
}

/// One member of the family [`MarginalModeCascade`] sweeps — Python's `run(...)` return.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MarginalCascadeCell {
    pub b0: f64,
    pub b_end: f64,
    pub g_end: f64,
    pub drift: f64,
    pub removed: f64,
    /// Python's `"I"`.
    pub i: f64,
    pub min_phi_lp: f64,
    /// The two laws' TRACKING ERRORS — rung 65 had both machine-zero wherever the pair rode;
    /// OFF-manifold neither is.
    pub track_b: f64,
    pub track_g: f64,
    /// `NaN` when nothing rides — and this is one of the per-row `NaN`s [`py_max_of`] exists for.
    pub laws_held: f64,
    pub n_on: usize,
    pub npts: usize,
}

/// Python's `marginal_mode_cascade` return — **RUNG 65's `marginal_mode`, VERBATIM, on a plant
/// whose second loop also has a clock.**
///
/// A zero eigenvalue is NO RESTORING FORCE ALONG a direction, not a state that sits still. Rung
/// 65's instantaneous fuel leg pinned the state to the manifold `phi_lp = phi_lim`, where the
/// marginal direction has nothing to drive it; give the fuel leg a clock and the state runs
/// OFF-manifold and drifts ALONG that direction. Same degeneracy, different observable:
/// **the freeze was the MANIFOLD, not the mode.**
#[derive(Clone, Debug)]
pub struct MarginalModeCascade {
    pub sm: f64,
    pub tau: f64,
    pub tau_att: f64,
    pub b_cap: f64,
    pub d_b0: f64,
    pub r: f64,
    pub ds: f64,
    pub phi_lim: f64,
    pub natural: MarginalCascadeCell,
    pub moved_lo: MarginalCascadeCell,
    pub moved_hi: MarginalCascadeCell,
    pub b_natural: f64,
    /// (i) is the STATE frozen? rung 65: exactly. here: REPORTED.
    pub frozen: f64,
    /// (ii) does a `b0` offset SURVIVE? rung 65: one-for-one to the end.
    pub db_db0: f64,
    /// (iii) does the WITHHELD FUEL still move with it? rung 65: yes.
    pub dremoved: f64,
    pub dremoved_rel: f64,
    pub washed_out: bool,
    pub track_b: f64,
    pub track_g: f64,
    /// **A SINGLE VALUE, `natural`'s** — where rung 65's `MarginalMode::laws_held` aggregates
    /// three through `py_max3`. The two return dicts were diffed field-by-field rather than the
    /// rung-65 struct being edited, because a reader that LOOKS like its parent is exactly where
    /// an aggregation silently appears.
    pub laws_held: f64,
}

/// One release rate's row of [`MergeIdentity`] — Python's `rows[i]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MergeRow {
    pub tau_rel: f64,
    pub npts: usize,
    /// The index where this run first departs from the reference — `None` when identical.
    pub first_diff: Option<usize>,
    pub s_first: Option<f64>,
    pub identical: bool,
}

/// Python's `merge_identity` return — **RUNG 52's STRUCTURAL FACT, re-measured after the merge,
/// and it is a BUG DETECTOR rather than a finding.**
///
/// `tau_rel` is never READ while `required > g`, so the entire march up to the first crossing
/// must be BIT-IDENTICAL across a release-rate sweep. If it is not, either the merged integrator
/// started reading the release constant or § 1's `_b_state` boundary leaked — both silent
/// failures that no protection number would expose.
#[derive(Clone, Debug)]
pub struct MergeIdentity {
    pub sm: f64,
    pub tau: f64,
    pub tau_att: f64,
    pub tau_rels: Vec<f64>,
    pub ds: f64,
    /// Where `required` first falls below `g`. `first_diff` and this must coincide.
    pub crossing: Option<usize>,
    pub s_crossing: Option<f64>,
    pub rows: Vec<MergeRow>,
    pub ok: bool,
}

impl ScheduledStatorCore {
    /// `R_q = dR/dq` and `C_g = dC/dg` by CENTRAL DIFFERENCE on the SHIPPED closures —
    /// `try_sched_fuel`/`try_surge_fuel` for the fuel law, `r64_solve_b` for the valve's.
    ///
    /// **NEITHER KNOWS THE OTHER EXISTS**, which is what makes their product a MEASUREMENT of
    /// § 2's identity rather than a restatement of it.
    ///
    /// `dq` and `dg` are parameters here where Python gives them defaults — the crate's
    /// convention — and [`GAINS_DQ`] / [`GAINS_DG`] carry those defaults for the one shipped
    /// caller.
    #[allow(clippy::too_many_arguments)]
    pub fn gains(
        &self, flight: &FlightCondition, a: f64, h: f64, g: f64, q: f64, mf_sched: f64,
        accel: Option<&AccelSchedule>, surge: Option<&Floor>, dq: f64, dg: f64,
    ) -> (f64, f64) {
        let ft = &self.fuel;
        let bl = ft.inner.lever.lim.expect("rung-66's gains on an unfloored machine");
        let (tt2, pt2, _) = ft.inner.inlet(flight);
        let raise = |e: Abort| -> f64 { panic!("{}", e.0) };
        // The PLANT side: the valve AS IT IS.
        let big_r = |qq: f64| -> f64 {
            let _st = MarchedBleed::set(&ft.inner, qq);
            let mut caps: Vec<f64> = Vec::new();
            if let Some(sch) = accel {
                caps.push(ft.try_sched_fuel(flight, a, h, mf_sched, sch).unwrap_or_else(raise));
            }
            if let Some(fl) = surge {
                caps.push(ft.try_surge_fuel(flight, a, h, mf_sched, fl).unwrap_or_else(raise));
            }
            if caps.is_empty() {
                return 0.0;
            }
            let mut m = caps[0];
            for &c in &caps[1..] {
                if c < m { m = c; }
            }
            0.0f64.max(mf_sched - m)
        };
        // The COMMAND side: a root over TRIALS, and it must NOT see the live position.
        let big_c = |gg: f64| -> f64 {
            crate::limited_bleed::r64_solve_b(&bl, |b| {
                let _g = ForcedBleed::set(&ft.inner, b);
                r62_try_close_fuel(ft, a, h, 1e-9f64.max(mf_sched - gg), tt2, pt2)
            }).unwrap_or_else(|e| panic!("{}", e.0)).1
        };
        ((big_r(q + dq) - big_r(q - dq)) / (2.0 * dq),
         (big_c(g + dg) - big_c(g - dg)) / (2.0 * dg))
    }

    /// **RUNG 66's CORE INSTRUMENT** — § 2's identity, measured rather than asserted.
    ///
    /// **RIDING IS `required > 0`, NOT `mf < mf_sched`.** A lagged clip DECAYS but never reaches
    /// zero, so the second test is true forever after first engagement and would sample the gains
    /// at points where the fuel law is dormant and `R_q == 0` — which is exactly where the
    /// identity does not apply.
    #[allow(clippy::too_many_arguments)]
    pub fn cascade_identity(
        &self, flight: &FlightCondition, ramp: &Ramp, sm: f64, b_cap: f64, tau: f64,
        tau_atts: &[f64], rel_mult: f64, n_sample: usize,
    ) -> CascadeIdentity {
        let cmap = self.arming().map_lp_design;
        let fuel = SurgeLimiter::from_margin(&cmap, Spool::Lp, sm);
        let floor = Floor::Phi(fuel);
        let leg = StatorLeg { accel: None, surge: Some(floor), tt4_max: None };
        let mut rows: Vec<CascadeIdentityRow> = Vec::new();
        for &ta in tau_atts {
            let lag = AsymmetricLag::new(ta, rel_mult * ta);
            let m = self.at_lever(&LeverArm::floored(
                BleedLimiter::from_margin_tau(&cmap, b_cap, sm, Some(tau))));
            let (traj, _) = m.stator_march_scoped(
                flight, ramp, None, &leg, &MarchScope { lag: Some(lag), ..MarchScope::DEFAULT });
            let ride: Vec<&FuelPoint> = traj.iter().filter(|p| {
                let (_, req) = asym_extra(p);
                let (_, cmd) = valve_of(p);
                req > 0.0 && 0.0 < cmd && cmd < b_cap
            }).collect();
            // THE STRIDE, not the request — § 5.24 (i). `n_sample` below is `sub.len()`.
            let stride = 1usize.max(ride.len() / n_sample);
            let sub: Vec<&FuelPoint> = ride.iter().copied().step_by(stride).collect();
            let (mut prods, mut rhos, mut rqs, mut cgs) =
                (Vec::new(), Vec::new(), Vec::new(), Vec::new());
            let mut reals = 0usize;
            for p in &sub {
                let (g, req) = asym_extra(p);
                let (b, _) = valve_of(p);
                let (r_q, c_g) = m.gains(flight, p.nu_lp, p.nu_hp, g, b, p.mf_sched,
                                         None, Some(&floor), GAINS_DQ, GAINS_DG);
                let e = eig(r_q, c_g, lag.tau(req, g), tau);
                prods.push(r_q * c_g);
                rhos.push(e.rho);
                reals += usize::from(e.real);
                rqs.push(r_q);
                cgs.push(c_g);
            }
            let rate = 1.0 / ta + 1.0 / tau;
            let abs_r: Vec<f64> = rqs.iter().map(|v| v.abs()).collect();
            let abs_c: Vec<f64> = cgs.iter().map(|v| v.abs()).collect();
            rows.push(CascadeIdentityRow {
                tau_att: ta,
                tau_v: tau,
                n_ride: ride.len(),
                n_sample: sub.len(),
                n_real: reals,
                prod_lo: if prods.is_empty() { f64::NAN } else { py_min_of(&prods) },
                prod_hi: if prods.is_empty() { f64::NAN } else { py_max_of(&prods) },
                rho_max: if rhos.is_empty() { f64::NAN } else { py_max_of(&rhos) },
                rate_closed_form: rate,
                rho_err: if rhos.is_empty() { f64::NAN }
                         else { (py_max_of(&rhos) - rate).abs() / rate },
                gain_span_r: if rqs.is_empty() { f64::NAN }
                             else { py_max_of(&abs_r) / py_min_of(&abs_r) },
                gain_span_c: if cgs.is_empty() { f64::NAN }
                             else { py_max_of(&abs_c) / py_min_of(&abs_c) },
                r_q_lo: if rqs.is_empty() { f64::NAN } else { py_min_of(&rqs) },
                r_q_hi: if rqs.is_empty() { f64::NAN } else { py_max_of(&rqs) },
                c_g_lo: if cgs.is_empty() { f64::NAN } else { py_min_of(&cgs) },
                c_g_hi: if cgs.is_empty() { f64::NAN } else { py_max_of(&cgs) },
                ds_rho: ramp.ds * if rhos.is_empty() { 0.0 } else { py_max_of(&rhos) },
            });
        }
        let los: Vec<f64> = rows.iter().map(|x| x.prod_lo).collect();
        let his: Vec<f64> = rows.iter().map(|x| x.prod_hi).collect();
        let errs: Vec<f64> = rows.iter().map(|x| x.rho_err).collect();
        CascadeIdentity {
            sm,
            b_cap,
            tau,
            tau_atts: tau_atts.to_vec(),
            ds: ramp.ds,
            r: ramp.r,
            phi_lim: fuel.phi_lim,
            all_real: rows.iter().all(|x| x.n_real == x.n_sample),
            prod_lo: py_min_of(&los),
            prod_hi: py_max_of(&his),
            rho_err_max: py_max_of(&errs),
            rows,
        }
    }

    /// **RUNG 66's PROTECTION LEDGER** — the 2x2: each lagged loop alone, both, and neither.
    ///
    /// § 2 predicts strong SUB-ADDITIVITY: `det J == 0` means the pair has ONE effective
    /// actuator, so the second loop buys the RATE (they add) and not the AUTHORITY.
    #[allow(clippy::too_many_arguments)]
    pub fn cascade_bill(
        &self, flight: &FlightCondition, ramp: &Ramp, sm: f64, b_cap: f64, tau: f64,
        tau_att: f64, rel_mult: f64,
    ) -> CascadeBill {
        let cmap = self.arming().map_lp_design;
        let fuel = SurgeLimiter::from_margin(&cmap, Spool::Lp, sm);
        let lag = AsymmetricLag::new(tau_att, rel_mult * tau_att);
        let valve = BleedLimiter::from_margin_tau(&cmap, b_cap, sm, Some(tau));
        let cell = |blim: Option<BleedLimiter>, sg: bool, lg: bool| -> CascadeBillCell {
            let arm = match blim { Some(v) => LeverArm::floored(v), None => LeverArm::default() };
            let m = self.at_lever(&arm);
            let leg = StatorLeg { accel: None,
                                  surge: if sg { Some(Floor::Phi(fuel)) } else { None },
                                  tt4_max: None };
            let (traj, _) = m.stator_march_scoped(
                flight, ramp, None, &leg,
                &MarchScope { lag: if lg { Some(lag) } else { None }, ..MarchScope::DEFAULT });
            // Python's `min(pos, key=…)` — the FIRST-STRICT argmin, because `s_at_min` publishes
            // its LOCATION and a `<=` would report the last of a plateau instead of the first.
            let mut am: Option<&FuelPoint> = None;
            for p in traj.iter().filter(|p| p.s > 0.0) {
                if am.is_none_or(|q| p.phi_lp < q.phi_lp) { am = Some(p); }
            }
            let am = am.expect("rung-66's bill needs a marched point past s = 0");
            let last = &traj[traj.len() - 1];
            let hp: Vec<f64> = traj.iter().map(|p| p.phi_hp).collect();
            CascadeBillCell {
                i: violation(&traj, fuel.phi_lim, ramp.r),
                npts: traj.len(),
                min_phi: am.phi_lp,
                s_at_min: am.s,
                s_last: last.s,
                truncated: last.s < (ramp.r + ramp.s_settle) - 0.5 * ramp.ds,
                removed: self.removed_over(&traj),
                min_phi_hp: py_min_of(&hp),
                nu_lp_end: last.nu_lp,
                nu_hp_end: last.nu_hp,
                thrust_end: last.sp_thrust * last.mdot_air,
            }
        };
        let bare = cell(None, false, false);
        let fuel_cell = cell(None, true, true);
        let valve_cell = cell(Some(valve), false, false);
        let both = cell(Some(valve), true, true);
        let i0 = bare.i;
        let (c_f, c_v, c_b) =
            (1.0 - fuel_cell.i / i0, 1.0 - valve_cell.i / i0, 1.0 - both.i / i0);
        // what the FUEL leg adds on top of the valve, and the valve on top of the fuel leg
        let (m_f, m_v) = (c_b - c_v, c_b - c_f);
        CascadeBill {
            sm,
            b_cap,
            tau,
            tau_att,
            ds: ramp.ds,
            r: ramp.r,
            phi_lim: fuel.phi_lim,
            bare,
            fuel: fuel_cell,
            valve: valve_cell,
            both,
            credit_fuel: c_f,
            credit_valve: c_v,
            credit_both: c_b,
            sum_alone: c_f + c_v,
            delivered: c_b,
            subadditive: c_b < c_f + c_v,
            beats_both: c_b > c_f && c_b > c_v,
            marginal_fuel: m_f,
            marginal_valve: m_v,
            erosion_fuel: if m_f > 0.0 { c_f / m_f } else { f64::INFINITY },
            erosion_valve: if m_v > 0.0 { c_v / m_v } else { f64::INFINITY },
        }
    }

    /// **RUNG 65's `marginal_mode`, VERBATIM, on a plant whose second loop also has a clock** —
    /// and what it returns is the correction. See [`MarginalModeCascade`].
    #[allow(clippy::too_many_arguments)]
    pub fn marginal_mode_cascade(
        &self, flight: &FlightCondition, ramp: &Ramp, sm: f64, b_cap: f64, tau: f64,
        tau_att: f64, rel_mult: f64, d_b0: f64,
    ) -> MarginalModeCascade {
        let cmap = self.arming().map_lp_design;
        let fuel = SurgeLimiter::from_margin(&cmap, Spool::Lp, sm);
        let valve = BleedLimiter::from_margin_tau(&cmap, b_cap, sm, Some(tau));
        let lag = AsymmetricLag::new(tau_att, rel_mult * tau_att);
        let m = self.at_lever(&LeverArm::floored(valve));
        let leg = StatorLeg { accel: None, surge: Some(Floor::Phi(fuel)), tt4_max: None };

        let run = |b0: Option<f64>| -> MarginalCascadeCell {
            let (traj, _) = m.stator_march_scoped(
                flight, ramp, None, &leg,
                &MarchScope { lag: Some(lag), b0, ..MarchScope::DEFAULT });
            let on: Vec<&FuelPoint> =
                traj.iter().filter(|p| asym_extra(p).1 > 0.0).collect();
            let b_first = valve_of(&traj[0]).0;
            let last = &traj[traj.len() - 1];
            let drift: Vec<f64> =
                traj.iter().map(|p| (valve_of(p).0 - b_first).abs()).collect();
            let pos: Vec<f64> =
                traj.iter().filter(|p| p.s > 0.0).map(|p| p.phi_lp).collect();
            let tb: Vec<f64> =
                traj.iter().map(|p| { let (b, c) = valve_of(p); (b - c).abs() }).collect();
            let tg: Vec<f64> =
                traj.iter().map(|p| { let (g, r) = asym_extra(p); (g - r).abs() }).collect();
            let held: Vec<f64> =
                on.iter().map(|p| (p.phi_lp - fuel.phi_lim).abs()).collect();
            MarginalCascadeCell {
                b0: b_first,
                b_end: valve_of(last).0,
                g_end: asym_extra(last).0,
                drift: py_max_of(&drift),
                removed: self.removed_over(&traj),
                i: violation(&traj, fuel.phi_lim, ramp.r),
                min_phi_lp: py_min_of(&pos),
                track_b: py_max_of(&tb),
                track_g: py_max_of(&tg),
                laws_held: if held.is_empty() { f64::NAN } else { py_max_of(&held) },
                n_on: on.len(),
                npts: traj.len(),
            }
        };

        let nat = run(None);
        let b_nat = nat.b0;
        let mut moved = Vec::new();
        for (lbl, x) in [("lo", b_nat - d_b0), ("hi", b_nat + d_b0)] {
            assert!(x > 0.0 && x < b_cap,
                    "rung-66 b0 sweep leaves the valve's stops at {lbl}: {x:.6} not in \
                     (0, {b_cap}).");
            moved.push(run(Some(x)));
        }
        let (lo, hi) = (moved[0], moved[1]);
        let span = (hi.removed - lo.removed).abs();
        MarginalModeCascade {
            sm,
            tau,
            tau_att,
            b_cap,
            d_b0,
            r: ramp.r,
            ds: ramp.ds,
            phi_lim: fuel.phi_lim,
            // **`f64::max` AND NOT `py_max3`, AND THAT IS A MEASUREMENT** (§ 5.24 (vii)). `drift`
            // is itself a `max` over a NON-EMPTY trajectory of finite values, so it cannot be
            // `NaN` — the one place the two spellings part company. Rung 65's `laws_held` needed
            // the helper because its per-cell value IS `NaN` when nothing rides; this rung's
            // `laws_held` (below) is a SINGLE value and never a reduction. The `py_max3` defect
            // does NOT recur here, and the reason is stated rather than the helper reached for.
            frozen: nat.drift.max(lo.drift).max(hi.drift),
            db_db0: (hi.b_end - lo.b_end) / (2.0 * d_b0),
            dremoved: span,
            dremoved_rel: span / nat.removed.abs(),
            washed_out: ((hi.b_end - lo.b_end) / (2.0 * d_b0)).abs() < 1e-3,
            track_b: nat.track_b,
            track_g: nat.track_g,
            laws_held: nat.laws_held,
            natural: nat,
            moved_lo: lo,
            moved_hi: hi,
            b_natural: b_nat,
        }
    }

    /// **RUNG 52's STRUCTURAL FACT, RE-MEASURED AFTER THE MERGE** — see [`MergeIdentity`].
    ///
    /// `first_diff` is the index where a run first departs from the reference; `crossing` is where
    /// `required` first falls below `g`. **They must coincide.**
    #[allow(clippy::too_many_arguments)]
    pub fn merge_identity(
        &self, flight: &FlightCondition, ramp: &Ramp, sm: f64, b_cap: f64, tau: f64,
        tau_att: f64, tau_rels: &[f64],
    ) -> MergeIdentity {
        let cmap = self.arming().map_lp_design;
        let fuel = SurgeLimiter::from_margin(&cmap, Spool::Lp, sm);
        let m = self.at_lever(&LeverArm::floored(
            BleedLimiter::from_margin_tau(&cmap, b_cap, sm, Some(tau))));
        let leg = StatorLeg { accel: None, surge: Some(Floor::Phi(fuel)), tt4_max: None };
        // Python's `keys` tuple, in its order: s, nu_lp, nu_hp, phi_lp, phi_hp, Tt4, mf, b, g.
        let key9 = |p: &FuelPoint| -> [f64; 9] {
            [p.s, p.nu_lp, p.nu_hp, p.phi_lp, p.phi_hp, p.tt4, p.mf, valve_of(p).0,
             asym_extra(p).0]
        };
        let run = |tr: f64| -> (Vec<FuelPoint>, Vec<[f64; 9]>) {
            let (traj, _) = m.stator_march_scoped(
                flight, ramp, None, &leg,
                &MarchScope { lag: Some(AsymmetricLag::new(tau_att, tr)),
                              ..MarchScope::DEFAULT });
            let ks = traj.iter().map(key9).collect();
            (traj, ks)
        };
        let (base_traj, base) = run(tau_rels[0]);
        let crossing = base_traj.iter().position(|p| {
            let (g, req) = asym_extra(p);
            req < g
        });
        let mut rows: Vec<MergeRow> = Vec::new();
        for &tr in tau_rels {
            let (traj, ks) = run(tr);
            let n = base.len().min(ks.len());
            let first = (0..n).find(|&i| base[i] != ks[i]);
            rows.push(MergeRow {
                tau_rel: tr,
                npts: traj.len(),
                first_diff: first,
                s_first: first.map(|i| traj[i].s),
                identical: first.is_none(),
            });
        }
        // The reference against ITSELF must be identical; every OTHER rate must first differ AT
        // the crossing (one cell of slack for the kink's own step).
        let ok = rows.iter().all(|x| if x.tau_rel == tau_rels[0] {
            x.first_diff.is_none()
        } else {
            match (x.first_diff, crossing) {
                (Some(f), Some(c)) => f.abs_diff(c) <= 1,
                _ => false,
            }
        });
        MergeIdentity {
            sm,
            tau,
            tau_att,
            tau_rels: tau_rels.to_vec(),
            ds: ramp.ds,
            crossing,
            s_crossing: crossing.map(|i| base_traj[i].s),
            rows,
            ok,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// THE CONSTRUCTOR
// ---------------------------------------------------------------------------------------------

/// RUNG 66's object — **rung 65's constructor with the TABLES swapped and NOTHING else, and that
/// is a measurement.**
///
/// `TwoLagCascadeTransient` defines no `__init__` and rebinds no class attribute except `_lag`
/// (the carrier, not a constructor knob), so **none of rung 65's ten construction asserts is
/// added to, relaxed or re-ordered here** — `_LAG_OK` in particular stays `True` by inheritance.
/// The difference between this builder and `build_lagged_bleed` is the four table constants it
/// passes, and saying so explicitly is the point: a reader who finds two builders differing by
/// four words should be told that the SAMENESS was measured and is the finding.
pub fn build_two_lag_cascade(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    let built = ScheduledStatorTransient::with_tables(
        design_engine, flight_design, mdot_design, map_lp, map_hp, rho, arm.stator,
        &R66_TWO, &R66_STATOR, &R66_FUEL, &R66,
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
// THE TABLES — three cells swapped across three tables, and ZERO across the fourth
// ---------------------------------------------------------------------------------------------

/// RUNG 66's lever table — ONE cell, `at_lever`. `b_of` / `b_at_point` / `armed_bleed` /
/// `isolating` / `legs` are rung 64's and rung 65's, inherited.
pub const R66: LeverHooks = LeverHooks {
    at_lever: r66_at_lever,
    ..crate::lagged_bleed::R65
};

/// RUNG 66's `TwoSpoolTransientHooks` — **ZERO cells swapped.**
///
/// **AN ALIAS, NOT A RE-ENUMERATION, AND THAT IS THE FAITHFUL SPELLING.** Rung 66 subclasses rung
/// 65, so a change to any rung-65 cell propagates to rung 66 BY INHERITANCE; an alias reproduces
/// exactly that, where a hand-enumerated literal would freeze rung 65's current bodies into rung
/// 66 and silently stop tracking it.
///
/// It is NAMED rather than left implicit at the builder so the cell census reads off the table
/// list instead of off an absence. Note what that does and does not buy: it makes the ZERO
/// legible, and it does **not** make a future addition to `R65_TWO` loud here — nothing should,
/// because propagating is what Python does.
pub const R66_TWO: TwoSpoolTransientHooks = crate::lagged_bleed::R65_TWO;

/// RUNG 66's fuel table — ONE cell, `integrate_fuel`. `try_close_fuel` is rung 65's.
pub const R66_FUEL: FuelTransientHooks = FuelTransientHooks {
    integrate_fuel: r66_integrate_fuel,
    ..crate::lagged_bleed::R65_FUEL
};

/// RUNG 66's stator table — ONE cell, the march that carries `lag`.
pub const R66_STATOR: StatorTransientHooks = StatorTransientHooks {
    stator_march: r66_stator_march,
    ..crate::lagged_bleed::R65_STATOR
};
