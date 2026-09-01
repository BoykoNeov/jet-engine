//! RUNG 65 — the LAGGED BLEED VALVE: what a finite bandwidth costs, and what it gives back.
//!
//! Slice Y. Python's `LaggedBleedTransient` (`turbojet/engine.py`, 479 lines), ported onto
//! [`crate::limited_bleed`]'s shape. `BleedLimiter` — `tau` included — was ported whole at slice
//! X, so this module carries no device of its own.
//!
//! **HEADLINE (the rung's): a lag repairs the SOLVE without removing the DEGENERACY.** Two loops
//! on one variable are redundant, and the redundancy is CONSERVED — rung 64's instantaneous valve
//! hid it in a solver where it was a roundoff coin flip; a finite bandwidth moves it into the
//! STATE, where it is a MARGINAL MODE: exactly frozen, `tau`-invariant, a one-parameter family
//! selected by the initial condition alone.
//!
//! # What slice Y adds to the table
//!
//! **ZERO new cells** — probe 1's emitted census, and the second row of § 5.19 (x)'s cell column
//! an emitter confirms. Six SWAPS (`try_close`, `try_close_fuel`, `stator_march`, `at_lever`,
//! `b_at_point`, `integrate_fuel`), of which `integrate_fuel` is the **third and last** of the
//! code-resident `⚠` notes slices S/T left in `fuel_transient.rs`.
//!
//! # The two MIRROR ZEROS, measured — and they are why two gates here are manufactured
//!
//! § 5.23 (i). Rung 65 gives `_close` and `_close_fuel` the SAME two-way test, and counted over
//! rungs 62 + 63 + 64 + 65 neither is ever exercised both ways — the zeros are on OPPOSITE arms:
//!
//! | cell | `lagged=F` | `lagged=T, state=F` | `lagged=T, state=T` |
//! |---|---|---|---|
//! | [`r65_try_close`] | 864 | **2 064** | **0** |
//! | [`r65_try_close_fuel`] | 20 477 | **0** | **200 960** |
//!
//! `_b_state` is set only inside [`r65_integrate_fuel_valve_lag`]'s derivative, which imposes
//! FUEL — so every close under a live state is a `try_close_fuel`; and `try_close` is reached
//! only from the STEADY solves, which never run inside a derivative evaluation. Consequently
//! **[`r65_try_close`] is a no-op on the whole shipped grid** (deleting Python's override moved
//! all 13 witness keys by zero) and **[`r65_try_close_fuel`]'s state test is untested**: a port
//! spelling it *"if lagged, dispatch to rung 63"* agrees everywhere. Both are gated by
//! manufactured bugs in `tests/slice_y_dispatch.rs`, never by a value key, because no value key
//! can reach them.
//!
//! # And the branch slice X declared dead is now the second-busiest
//!
//! `b_of`'s `b_state` override was **0 of 1 705** at rung 64 and shipped anyway because *"a port
//! that drops it passes every slice-X gate and breaks at slice Y"*. Measured here: **200 960 of
//! 908 001**. `slice_x_dispatch.rs`'s `b_of_state == 0` stays green, correctly — its census
//! marches all build rung-64 machines.

use std::cell::Cell;

use crate::bleed_transient::{r62_try_close, r62_try_close_fuel, LeverArm, LeverArming,
                             LeverHooks};
use crate::engine::FlightCondition;
use crate::fuel_transient::{point, AccelSchedule, Floor, FuelCloseState, FuelLimiters, FuelPoint,
                            FuelTransientCore, FuelTransientHooks, PointExtra, SurgeLimiter};
use crate::gas::Abort;
use crate::limited_bleed::{BillCell, BleedLimiter};
use crate::map::ComponentMap;
use crate::stator_transient::{MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient,
                              StatorLeg, StatorTransientHooks};
use crate::two_spool::{Spool, TwoSpoolEngine};
use crate::two_spool_transient::{CloseState, ForcedBleed, InitialBleed, MarchedBleed,
                                 TwoSpoolTransientCore, TwoSpoolTransientHooks};

// ---------------------------------------------------------------------------------------------
// COUNTERS — the dispatch is invisible to every value key, in BOTH directions
// ---------------------------------------------------------------------------------------------

// § 5.23 (i): the two closure cells are one-armed on the shipped grid, and rung 65's `_close`
// override is a no-op there. Neither fact is visible in a float, so both are counted.
thread_local! {
    static CLOSE_UNLAGGED: Cell<u64> = const { Cell::new(0) };
    static CLOSE_STEADY: Cell<u64> = const { Cell::new(0) };
    static CLOSE_MARCHED: Cell<u64> = const { Cell::new(0) };
    static CLOSE_FUEL_UNLAGGED: Cell<u64> = const { Cell::new(0) };
    static CLOSE_FUEL_STEADY: Cell<u64> = const { Cell::new(0) };
    static CLOSE_FUEL_MARCHED: Cell<u64> = const { Cell::new(0) };
    static BAP_SUPER: Cell<u64> = const { Cell::new(0) };
    static BAP_RECORDED: Cell<u64> = const { Cell::new(0) };
    static MARCH_SUPER: Cell<u64> = const { Cell::new(0) };
    static MARCH_VALVE_LAG: Cell<u64> = const { Cell::new(0) };
    static STATOR_B0_NONE: Cell<u64> = const { Cell::new(0) };
    static STATOR_B0_SET: Cell<u64> = const { Cell::new(0) };
    static CMD_CALLS: Cell<u64> = const { Cell::new(0) };
    static CLAMP_HITS: Cell<u64> = const { Cell::new(0) };
}

fn bump(c: &'static std::thread::LocalKey<Cell<u64>>) {
    c.with(|x| x.set(x.get() + 1));
}

/// Rung 65's dispatch census — the instrument § 5.23 P3 registers its manufactured gates against.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Census65 {
    pub close_unlagged: u64,
    /// `lagged = true, b_state = None` — the STEADY-solve arm. **2 064 on the shipped grid.**
    pub close_steady: u64,
    /// `lagged = true, b_state = Some` — **ZERO on the shipped grid** (§ 5.23 (i)).
    pub close_marched: u64,
    pub close_fuel_unlagged: u64,
    /// **ZERO on the shipped grid** — the mirror of `close_marched`, and the reason
    /// [`r65_try_close_fuel`]'s state test needs a manufactured gate.
    pub close_fuel_steady: u64,
    pub close_fuel_marched: u64,
    pub bap_super: u64,
    pub bap_recorded: u64,
    pub march_super: u64,
    pub march_valve_lag: u64,
    pub stator_b0_none: u64,
    pub stator_b0_set: u64,
    pub cmd_calls: u64,
    /// How often the physical stop actually bit. Python's comment says the clamp is INERT while
    /// the command is interior; this is that claim as a number rather than as a sentence.
    pub clamp_hits: u64,
}

impl Census65 {
    pub fn take() -> Census65 {
        Census65 {
            close_unlagged: CLOSE_UNLAGGED.with(Cell::get),
            close_steady: CLOSE_STEADY.with(Cell::get),
            close_marched: CLOSE_MARCHED.with(Cell::get),
            close_fuel_unlagged: CLOSE_FUEL_UNLAGGED.with(Cell::get),
            close_fuel_steady: CLOSE_FUEL_STEADY.with(Cell::get),
            close_fuel_marched: CLOSE_FUEL_MARCHED.with(Cell::get),
            bap_super: BAP_SUPER.with(Cell::get),
            bap_recorded: BAP_RECORDED.with(Cell::get),
            march_super: MARCH_SUPER.with(Cell::get),
            march_valve_lag: MARCH_VALVE_LAG.with(Cell::get),
            stator_b0_none: STATOR_B0_NONE.with(Cell::get),
            stator_b0_set: STATOR_B0_SET.with(Cell::get),
            cmd_calls: CMD_CALLS.with(Cell::get),
            clamp_hits: CLAMP_HITS.with(Cell::get),
        }
    }

    pub fn reset() {
        for c in [&CLOSE_UNLAGGED, &CLOSE_STEADY, &CLOSE_MARCHED, &CLOSE_FUEL_UNLAGGED,
                  &CLOSE_FUEL_STEADY, &CLOSE_FUEL_MARCHED, &BAP_SUPER, &BAP_RECORDED,
                  &MARCH_SUPER, &MARCH_VALVE_LAG, &STATOR_B0_NONE, &STATOR_B0_SET, &CMD_CALLS,
                  &CLAMP_HITS] {
            c.with(|x| x.set(0));
        }
    }
}

// ---------------------------------------------------------------------------------------------
// THE PREDICATE
// ---------------------------------------------------------------------------------------------

/// Python's `_lagged()` — **the lag rides on the LIMITER, not on the machine.**
///
/// That is deliberate and it is what stops rung 65 from becoming the fifth instance of the
/// sibling-constructor trap rungs 61/62/63/64 each hit: there is no separate lag keyword for
/// [`r65_at_lever`] to drop.
pub fn lagged(t: &TwoSpoolTransientCore) -> bool {
    t.lever.lim.is_some_and(|l| l.tau.is_some())
}

// ---------------------------------------------------------------------------------------------
// THE PLANT — the closure runs at the STATE, never at the command
// ---------------------------------------------------------------------------------------------

/// RUNG 65's `_close`. **A NO-OP ON THE SHIPPED GRID, AND THAT IS MEASURED, NOT ASSUMED.**
///
/// Its `b_state`-live arm ran **0** times in 2 928 calls over rungs 62–65 (§ 5.23 (i)), because
/// `_b_state` is only ever set around a FUEL-imposed close. Deleting Python's override moves
/// every witness key by zero. It ships because Python ships it and because rungs 66–68 put more
/// states in the same place; `tests/slice_y_dispatch.rs` reaches the dead arm by hand.
pub fn r65_try_close(
    t: &TwoSpoolTransientCore, nu_lp: f64, nu_hp: f64, tt4: f64, tt2: f64, pt2: f64,
) -> Result<CloseState, Abort> {
    if lagged(t) && t.b_state.get().is_some() {
        bump(&CLOSE_MARCHED);
        return r62_try_close(t, nu_lp, nu_hp, tt4, tt2, pt2);
    }
    bump(if lagged(t) { &CLOSE_STEADY } else { &CLOSE_UNLAGGED });
    crate::limited_bleed::r64_try_close(t, nu_lp, nu_hp, tt4, tt2, pt2)
}

/// RUNG 65's `_close_fuel` — **the live one, and the mirror image of its sibling.**
///
/// Python: *"Inside the march (`_b_state` set) the valve IS the state, so this dispatches to rung
/// 63's closure and `b_of` hands back the state. Outside it — every STEADY solve — the lag is
/// meaningless and rung 64's instantaneous root runs, which is what makes the initial running
/// line identical to the machine this rung is compared against."*
///
/// **THE SECOND HALF OF THAT SENTENCE IS TRUE AND THIS METHOD NEVER DELIVERS IT.** The steady arm
/// ran **0** times in 221 437 calls; the mechanism the docstring describes is real but arrives
/// entirely through [`r65_try_close`], the sibling. So dropping `&& b_state.is_some()` here
/// agrees on every shipped path — § 5.23 (i), and the gate for it is manufactured.
pub fn r65_try_close_fuel(
    ft: &FuelTransientCore, nu_lp: f64, nu_hp: f64, mdot_fuel: f64, tt2: f64, pt2: f64,
) -> Result<FuelCloseState, Abort> {
    if lagged(&ft.inner) && ft.inner.b_state.get().is_some() {
        bump(&CLOSE_FUEL_MARCHED);
        return r62_try_close_fuel(ft, nu_lp, nu_hp, mdot_fuel, tt2, pt2);
    }
    bump(if lagged(&ft.inner) { &CLOSE_FUEL_STEADY } else { &CLOSE_FUEL_UNLAGGED });
    crate::limited_bleed::r64_try_close_fuel(ft, nu_lp, nu_hp, mdot_fuel, tt2, pt2)
}

// ---------------------------------------------------------------------------------------------
// THE CELLS
// ---------------------------------------------------------------------------------------------

/// RUNG 65's `b_at_point` — **CORRECTS RUNG 64's COMMENT.**
///
/// There the valve is a pure function of the state, so the position is RE-SOLVED at a recorded
/// point. A LAGGED position is not a function of the state — it carries history — so it must be
/// RECORDED, and re-solving it would silently hand back the COMMAND instead: the one number that
/// is not the valve.
///
/// The panic ports Python's assert verbatim, and it is reachable: a trajectory from any other
/// integrator carries [`PointExtra::None`].
pub fn r65_b_at_point(core: &ScheduledStatorCore, flight: &FlightCondition, p: &FuelPoint) -> f64 {
    if !lagged(&core.fuel.inner) {
        bump(&BAP_SUPER);
        return crate::limited_bleed::r64_b_at_point(core, flight, p);
    }
    bump(&BAP_RECORDED);
    match p.extra {
        PointExtra::Valve { b, .. } => b,
        // SLICE Z. Rungs 66/67 record `b` too, and Python reads the DICT KEY — so a wildcard
        // panic here would refuse a point Python answers. See [`valve_of`] for the audit.
        PointExtra::Cascade { b, .. } | PointExtra::CrossCascade { b, .. }
        // SLICE AA: rung 68's five-state march records `b` for the same reason.
        | PointExtra::Triple { b, .. }
        // SLICE AD: rung 72 carries the valve POSITION unchanged.
        | PointExtra::Shared { b, .. } => b,
        PointExtra::None | PointExtra::Asym { .. } => panic!(
            "rung-65: a lagged valve's position is a march STATE and cannot be recovered from a \
             trajectory point that did not record it. This point came from a different \
             integrator."),
    }
}

/// RUNG 65's `at_lever` — rung 64's sibling constructor returning THIS class.
///
/// **THE LAG RIDES ON `bleed_lim` PRECISELY SO THIS CANNOT BECOME THE FIFTH INSTANCE OF ONE
/// TRAP** (rung 61's `at_setting`, 62's `at_stator`, 63's `_isolating`, 64's `at_lever`): there is
/// no separate lag keyword for a sibling constructor to drop. The only thing that changes from
/// rung 64's body is which builder it calls — and that one word is the whole override, which is
/// why deleting it is caught by the CONSTRUCTOR rather than by a value (rung 64's builder refuses
/// a lagged limiter).
pub fn r65_at_lever(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    match build_lagged_bleed(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(), arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("at_lever never disables LP"),
    }
}

/// RUNG 65's `_stator_march` — rung 57's march with ONE addition, `b0`.
///
/// **`b0` IS AN ISOLATION DIAGNOSTIC AND NOT A CONTROL SETTING**, which is why Python makes it a
/// per-march argument rather than a machine keyword: *a sibling constructor cannot drop what it
/// never carries.* It exists because rung 65 § 3's finding is that `b` is a CONSTANT OF THE
/// MOTION, and a constant of the motion is only demonstrable by moving its value.
///
/// **THE GUARD RESTORES THE PREVIOUS VALUE** ([`InitialBleed`], not [`ForcedBleed`]) — Python is
/// `prev, self._b0 = self._b0, b0` … `finally: self._b0 = prev`. Probe 3 measured max nesting
/// depth **1**, so the difference is invisible to every value key; `slice_y_dispatch.rs`
/// manufactures the nest.
///
/// `b0 = None` is a real assignment: a rung-65 march called WITHOUT it CLEARS an outer one for
/// the duration, and `_bill_cell` reaches this cell exactly that way.
pub fn r65_stator_march(
    ft: &FuelTransientCore, flight: &FlightCondition, ramp: &Ramp, nu0: Option<(f64, f64)>,
    leg: &StatorLeg<'_>, scope: &MarchScope,
) -> (Vec<FuelPoint>, (f64, f64)) {
    bump(if scope.b0.is_some() { &STATOR_B0_SET } else { &STATOR_B0_NONE });
    let _g = InitialBleed::set(&ft.inner, scope.b0);
    // Python forwards to `super()._stator_march(...)` WITHOUT `b0` — the parameter is consumed
    // here. Rungs 66/67/68 will each consume one more and forward the rest, which is why the
    // scope is rebuilt rather than passed through.
    crate::stator_transient::r57_stator_march(ft, flight, ramp, nu0, leg, &MarchScope::DEFAULT)
}

/// RUNG 65's `integrate_fuel` — **the cell slice Y OPENS**, and the two refusals that keep the
/// rung from silently becoming its own next seam.
///
/// A lagged VALVE beside a lagged FUEL leg is the TWO-LAG CASCADE (rung 66); rungs 50/51's forced
/// release edges are an isolation instrument for a leg that cannot pin its own trigger, which
/// this one can. Both are refused rather than run.
pub fn r65_integrate_fuel(
    ft: &FuelTransientCore, flight: &FlightCondition, fuel_schedule: &dyn Fn(f64) -> f64,
    nu0: (f64, f64), s_end: f64, ds: f64, lim: &FuelLimiters<'_>,
) -> Vec<FuelPoint> {
    if !lagged(&ft.inner) {
        bump(&MARCH_SUPER);
        return crate::fuel_transient::r43_integrate_fuel(
            ft, flight, fuel_schedule, nu0, s_end, ds, lim);
    }
    assert!(lim.tau_gov.is_none() && lim.lag.is_none(),
            "rung-65: a lagged VALVE beside a lagged FUEL leg (rung 47's tau_gov, rung 52's \
             AsymmetricLag) is the TWO-LAG CASCADE -- rung 52's own standing seam, on two levers \
             instead of one, and rung 65's next seam. It is four states and a second clock; \
             nothing here has measured it, so it is refused rather than run.");
    assert!(lim.s_off.is_none() && lim.tau_rel.is_none(),
            "rung-65: rungs 50/51's FORCED release edges are an isolation instrument for a leg \
             that could not pin its own trigger. This valve pins its own (rung 52's argument, one \
             lever over), so forcing one would measure the forcing.");
    bump(&MARCH_VALVE_LAG);
    r65_integrate_fuel_valve_lag(ft, flight, fuel_schedule, nu0, s_end, ds, lim.freeze,
                                 lim.tt4_max, lim.accel, lim.floor().as_ref())
}

/// RUNG 65's MARCH — rung 47/52's third-state pattern moved from a fuel CLIP onto a valve
/// POSITION, **and the position is the first state in the ladder whose derivative is driven by
/// the closure's own root rather than by the state vector.**
///
/// `b` and `b_cmd` are recorded per point ([`PointExtra::Valve`]; every rung-64 key is unchanged)
/// so the TRACKING ERROR is readable straight off a trajectory, exactly as rung 52 made
/// `g`/`required` readable.
///
/// **`b(0) = b_cmd(0)`** — the EQUILIBRIUM position at the running line the march starts on.
/// Starting at 0 would inject a startup transient into the early-ramp LP minimum, which is the
/// binding one (rungs 41/44), and every number the rung reports would be measuring that instead
/// of the lag.
///
/// **THE COMMAND DOES NOT READ THE LIVE POSITION**, which is what makes `db/ds` AFFINE in `b` —
/// Lipschitz with constant `1/tau`, no latch, so rung 47's hazard cannot recur. It is also why
/// [`ForcedBleed`] never nests here: the state is cleared BEFORE the command solve runs, and the
/// solve binds rung 63's closure rather than rung 64's.
#[allow(clippy::too_many_arguments)]
pub fn r65_integrate_fuel_valve_lag(
    ft: &FuelTransientCore, flight: &FlightCondition, fuel_schedule: &dyn Fn(f64) -> f64,
    nu0: (f64, f64), s_end: f64, ds: f64, freeze: Option<Spool>, tt4_max: Option<f64>,
    accel: Option<&AccelSchedule>, surge: Option<&Floor>,
) -> Vec<FuelPoint> {
    let lim = ft.inner.lever.lim.expect("r65 march on an unfloored machine");
    let tau = lim.tau.expect("r65 march on an unlagged machine");
    // THE MODELLING FLOOR, found rather than assumed. `db/ds = (b_cmd - b)/tau` under an EXPLICIT
    // RK4 needs `z = ds/tau` inside the stability region (|z| <~ 2.78 on the negative real axis).
    // A first pre-check of this rung ran z = 5 and returned an `int b ds` 4.4x the grid-converged
    // value -- an instability that looks exactly like a physical finding ("a fast valve bleeds
    // more") and was published as a RETRACTION. The bar is Python's **2.0**, not the 2.78 the
    // prose derives, and it is a COPY.
    assert!(ds / tau <= 2.0,
            "rung-65: ds/tau = {:.3} is outside the explicit RK4 stability region for the valve \
             state (ds = {ds}, tau = {tau}). Refine the grid or raise tau -- the tau -> 0 limit \
             is APPROACHED on this integrator and never reached.", ds / tau);
    let (tt2, pt2, _) = ft.inner.inlet(flight);

    // Rung 64's instantaneous root at THIS state and fuel. `r62_try_close_fuel` is Python's
    // `super(LimitedBleedTransient, self)._close_fuel` — rung 63's closure, NOT this table's.
    let command = |a: f64, h: f64, mf: f64| -> Result<f64, Abort> {
        bump(&CMD_CALLS);
        Ok(crate::limited_bleed::r64_solve_b(&lim, |b| {
            let _g = ForcedBleed::set(&ft.inner, b);
            r62_try_close_fuel(ft, a, h, mf, tt2, pt2)
        })?.1)
    };

    type Der = (f64, f64, f64, f64, crate::fuel_transient::FuelInstant, f64);
    let der = |a: f64, h: f64, q: f64, s: f64| -> Result<Der, Abort> {
        let mf_sched = fuel_schedule(s);
        // THE MIN-SELECT, rung 48/49's discipline verbatim: every cap is solved from the
        // SCHEDULED fuel so arming one leg cannot perturb another's bracket.
        let (mf, inst) = {
            let _st = MarchedBleed::set(&ft.inner, q);
            let mut i = ft.try_instant_fuel(flight, a, h, mf_sched)?;
            let mut caps: Vec<f64> = Vec::new();
            if let Some(t4) = tt4_max {
                if i.base.tt4 > t4 {
                    caps.push(ft.try_topping_fuel(flight, a, h, t4, mf_sched)?);
                }
            }
            if let Some(sch) = accel {
                caps.push(ft.try_sched_fuel(flight, a, h, mf_sched, sch)?);
            }
            if let Some(fl) = surge {
                caps.push(ft.try_surge_fuel(flight, a, h, mf_sched, fl)?);
            }
            caps.retain(|&c| c < mf_sched);
            let mf = if caps.is_empty() {
                mf_sched
            } else {
                let mut m = caps[0];
                for &c in &caps[1..] {
                    if c < m { m = c; }
                }
                m
            };
            if !caps.is_empty() {
                i = ft.try_instant_fuel(flight, a, h, mf)?;
            }
            (mf, i)
        };
        // OUTSIDE the state's scope, exactly as Python's `command(...)` sits after the `finally`.
        let cmd = command(a, h, mf)?;
        let da = if freeze == Some(Spool::Lp) { 0.0 } else { inst.base.phi_lp_dot / ft.rho() };
        let dh = if freeze == Some(Spool::Hp) { 0.0 } else { inst.base.phi_hp_dot };
        Ok((da, dh, (cmd - q) / tau, mf, inst, cmd))
    };

    let (mut a, mut h) = nu0;
    let mut q = match ft.inner.b0.get() {
        Some(b0) => {
            assert!((0.0..=lim.b_max).contains(&b0),
                    "rung-65 b0 is a valve POSITION: {b0} is outside [0, {}]", lim.b_max);
            b0
        }
        None => match command(a, h, fuel_schedule(0.0)) {
            Ok(c) => c,
            // Python lets the initial command's AssertionError escape `_integrate_fuel_valve_lag`
            // entirely — it is raised BEFORE the `for` loop's `try`, so it is not one of the two
            // `break` sites. An `Abort` here would do the same; there is nothing to catch it.
            Err(_) => panic!("rung-65's initial command solve aborted: b(0) is taken before the \
                              march loop, so Python has no `except` here either"),
        },
    };
    let mut pts: Vec<FuelPoint> = Vec::new();
    let mut s = 0.0f64;
    // `s` ACCUMULATED and `round_ties_even`, both INHERITED from rung 43's marcher rather than
    // re-decided — see `FuelTransientCore::integrate_fuel`'s note on why a `k as f64 * ds` would
    // flip a published boolean.
    let n_steps = (s_end / ds).round_ties_even() as i64;
    for _ in 0..=n_steps {
        let Ok((k1a, k1h, k1q, mf_app, inst, cmd)) = der(a, h, q, s) else { break };
        pts.push(point(s, a, h, &inst, mf_app, fuel_schedule(s),
                       PointExtra::Valve { b: q, b_cmd: cmd }));
        let stages = (|| -> Result<[f64; 9], Abort> {
            // Python computes `mfm` here and never uses it — `der` re-reads the schedule off `s`.
            // Kept so the schedule is called the same number of times on both sides.
            let _mfm = fuel_schedule(s + ds / 2.0);
            let (k2a, k2h, k2q, ..) =
                der(a + ds / 2.0 * k1a, h + ds / 2.0 * k1h, q + ds / 2.0 * k1q, s + ds / 2.0)?;
            let (k3a, k3h, k3q, ..) =
                der(a + ds / 2.0 * k2a, h + ds / 2.0 * k2h, q + ds / 2.0 * k2q, s + ds / 2.0)?;
            let (k4a, k4h, k4q, ..) =
                der(a + ds * k3a, h + ds * k3h, q + ds * k3q, s + ds)?;
            Ok([k2a, k2h, k2q, k3a, k3h, k3q, k4a, k4h, k4q])
        })();
        let Ok([k2a, k2h, k2q, k3a, k3h, k3q, k4a, k4h, k4q]) = stages else { break };
        a += ds / 6.0 * (k1a + 2.0 * k2a + 2.0 * k3a + k4a);
        h += ds / 6.0 * (k1h + 2.0 * k2h + 2.0 * k3h + k4h);
        q += ds / 6.0 * (k1q + 2.0 * k2q + 2.0 * k3q + k4q);
        // THE POSITION IS PHYSICAL: a valve cannot open past its stop or shut past closed. The
        // clamp is INERT while the command is interior (a bounded state chasing a bounded command
        // from a bounded start) and it is the actuator's own HARDWARE, not a solver tolerance —
        // so it is applied to the STATE and never to the command. Python's spelling is
        // `min(b_max, max(0.0, q))`, and the order is copied rather than normalised.
        let clamped = lim.b_max.min(0.0f64.max(q));
        if clamped != q {
            bump(&CLAMP_HITS);
        }
        q = clamped;
        s += ds;
    }
    pts
}

// ---------------------------------------------------------------------------------------------
// THE CONSTRUCTOR — ten asserts re-spelled to relax ONE
// ---------------------------------------------------------------------------------------------

/// RUNG 65's constructor. Python has none: `LaggedBleedTransient` inherits rung 64's `__init__`
/// and flips **one class constant**, `_LAG_OK`, which rung 64's last assert reads.
///
/// **SO THIS RE-SPELLS THE CHAIN, AND THAT DUPLICATION IS DELIBERATE**
/// ([[rust-port-copy-vs-rederivation]]). `build_limited_bleed` hard-codes `_LAG_OK` false and its
/// own note forbids adding a bool parameter — that would put rung 65's state in rung 64's
/// signature. Probe 5 emitted the chain in execution order; it is **ten** asserts, of which the
/// first six belong to [`ScheduledStatorTransient::with_tables`] and its base and are inherited
/// here for free, and four are re-spelled below:
///
/// | # | owner | assert |
/// |---|---|---|
/// | 1–2 | `TwoSpoolMatcher` | no polytropic knobs; `nozzle_convergent` |
/// | 3–6 | rung 57 | the four capture-discipline asserts |
/// | 7–8 | rung 62 | `bleed` xor `bleed_sched`; `0 <= bleed < 0.5` |
/// | 9 | rung 64 | the THREE-way arming exclusion |
/// | **10** | rung 64 | **`tau is None or _LAG_OK` — RELAXED, and nothing else is** |
///
/// **THE FAILURE MODE IS SILENT.** Re-spelling can drop one of the other nine and every value key
/// stays green, so `tests/rung65.rs` gates each surviving assert individually and asserts the
/// COUNT, rather than gating "the constructor accepts a lagged limiter" — § 5.23 P7.
pub fn build_lagged_bleed(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    let built = ScheduledStatorTransient::with_tables(
        design_engine, flight_design, mdot_design, map_lp, map_hp, rho, arm.stator,
        &R65_TWO, &R65_STATOR, &R65_FUEL, &R65,
        LeverArming { bleed: arm.bleed, sched: arm.bleed_sched, lim: arm.bleed_lim });
    // 7–8, rung 62's, in Python's order.
    assert!(!(arm.bleed != 0.0 && arm.bleed_sched.is_some()),
            "rung-62: the valve gets a CONSTANT position or a SCHEDULE, not both -- they are the \
             two legs the rung differences (rung 57's discipline).");
    assert!((0.0..0.5).contains(&arm.bleed),
            "rung-42 bleed fraction must be in [0, 0.5): b>=0.5 starves the core and the choked \
             branch is long gone by then");
    // 9, rung 64's — EXTENDED to three, never replaced.
    assert!(!(arm.bleed_lim.is_some() && (arm.bleed != 0.0 || arm.bleed_sched.is_some())),
            "rung-64: the valve gets a CONSTANT position (42), a SCHEDULE (62) or a FLOOR (64) -- \
             exactly one. They are the three legs this rung differences, and rung 62's two-way \
             assert is extended rather than replaced.");
    // 10 is Python's `... or self._LAG_OK`, and THIS is the class that flips it to True. The
    // assert is not weakened, it is SATISFIED — spelled as a comment rather than as a dropped
    // line so a reader can see which of the ten is the one rung 65 exists to pass.
    built
}

// ---------------------------------------------------------------------------------------------
// THE TABLES
// ---------------------------------------------------------------------------------------------

/// RUNG 65's lever table — TWO cells swapped, `b_of` / `armed_bleed` / `isolating` / `legs`
/// inherited from rung 64.
pub const R65: LeverHooks = LeverHooks {
    at_lever: r65_at_lever,
    b_at_point: r65_b_at_point,
    ..crate::limited_bleed::R64
};

/// RUNG 65's swap into rung 64's table — ONE cell, and it is the one measured INERT (§ 5.23 (i)).
pub const R65_TWO: TwoSpoolTransientHooks = TwoSpoolTransientHooks {
    try_close: r65_try_close,
    ..crate::limited_bleed::R64_TWO
};

/// RUNG 65's swap into rung 64's fuel table — TWO cells, one of which slice Y OPENED.
pub const R65_FUEL: FuelTransientHooks = FuelTransientHooks {
    try_close_fuel: r65_try_close_fuel,
    integrate_fuel: r65_integrate_fuel,
    ..crate::limited_bleed::R64_FUEL
};

/// RUNG 65's swap into rung 64's stator table — ONE cell, the march that carries `b0`.
pub const R65_STATOR: StatorTransientHooks = StatorTransientHooks {
    stator_march: r65_stator_march,
    ..crate::limited_bleed::R64_STATOR
};

// ---------------------------------------------------------------------------------------------
// THE READING INSTRUMENTS
// ---------------------------------------------------------------------------------------------

/// One bandwidth's row in [`BandwidthCeiling`] — Python's `rows[i]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BandwidthRow {
    pub tau: f64,
    pub min_phi_lp: f64,
    /// `min phi_lp - phi_lim` (`<= 0`): the protection the bandwidth costs.
    pub undershoot: f64,
    /// The bleed actually committed — **NOT monotone with `undershoot` in the direction a "lag is
    /// pure loss" reading expects**, which is the point.
    pub b_int: f64,
    pub b_peak: f64,
    pub b_end: f64,
    /// Rung 64 § 4's destroyed argmin, and whether a lag restores it.
    pub plateau_pts: usize,
    pub plateau_span: f64,
    pub s_at_min_lp: f64,
    pub b_at_min_lp: f64,
    /// **THE SATURATED CASE IS NOT THE RIDING CASE and the two must not be read together** — a
    /// floor above the fully-open march's own minimum commands `b_max` throughout, so under a lag
    /// it is a bare exponential approach with no feedback content at all.
    pub saturated: bool,
    /// `max |phi_lp(tau) - phi_lp(instantaneous)|` on the SAME grid: the `tau -> 0` arm of the
    /// reduce, MEASURED rather than asserted.
    pub dev: f64,
    pub d_nu_lp_end: f64,
    pub thrust_end_pct: f64,
    pub thrust_int_pct: f64,
    pub d_min_phi_hp: f64,
    pub max_track: f64,
}

/// Python's `bandwidth_ceiling` return — **RUNG 65, HALF ONE.**
#[derive(Clone, Debug)]
pub struct BandwidthCeiling {
    pub phi_lim: f64,
    pub b_cap: f64,
    pub r: f64,
    pub ds: f64,
    pub taus: Vec<f64>,
    pub rows: Vec<BandwidthRow>,
    pub shut: BillCell,
    pub inst: BillCell,
    /// One cell per `tau`, in the order the caller passed.
    pub cells: Vec<(f64, BillCell)>,
    pub inst_min_phi: f64,
    pub inst_b_int: f64,
    pub inst_plateau_pts: usize,
    pub inst_d_min_phi_hp: f64,
    /// Monotone in the SWEEP ORDER the caller passed (`taus` descending), not in `tau`.
    pub under_monotone: bool,
    pub bint_monotone: bool,
    pub dev_shrinks: bool,
}

/// One member of rung 65 § 3's one-parameter family — Python's `run(...)` return.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MarginalCell {
    pub b0: f64,
    pub b_end: f64,
    pub drift: f64,
    pub dbds: f64,
    pub removed: f64,
    pub min_phi_lp: f64,
    /// BOTH laws, wherever the fuel leg rides: the floor held EXACTLY. `NaN` when nothing rides.
    pub laws_held: f64,
    /// …and the valve strictly inside its stops, so neither law is merely clamped.
    pub interior: bool,
    pub n_ride: usize,
    pub npts: usize,
}

/// Python's `marginal_mode` return — **RUNG 65, HALF TWO, THE RUNG.**
#[derive(Clone, Debug)]
pub struct MarginalMode {
    pub sm: f64,
    pub tau: f64,
    pub taus: Vec<f64>,
    pub b_cap: f64,
    pub d_b0: f64,
    pub r: f64,
    pub ds: f64,
    pub phi_lim: f64,
    pub natural: MarginalCell,
    pub moved_lo: MarginalCell,
    pub moved_hi: MarginalCell,
    pub taucells: Vec<(f64, MarginalCell)>,
    pub b_natural: f64,
    /// (i) the mode is MARGINAL: `b` does not move over the whole march.
    pub frozen: f64,
    /// (ii) it is a CONTINUUM: the frozen value tracks `b0` one-for-one…
    pub db_db0: f64,
    /// …and the withheld fuel moves with it, both laws still exactly satisfied.
    pub dremoved: f64,
    pub laws_held: f64,
    pub interior: bool,
    /// (iii) `tau` multiplies a machine zero, so it is powerless over the mode.
    pub tau_span: f64,
    pub tau_span_rel: f64,
}

/// One plant's side of the discriminator — Python's `out[name]`.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthoritySide {
    pub phis: Vec<f64>,
    pub g: Vec<f64>,
    /// The authority the fuel has over the protected variable — THE currency.
    pub span: f64,
    pub monotone: bool,
    pub sign_change: bool,
    pub max_abs_g: f64,
}

/// Where the bracket was swept — Python's `at` sub-dict.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AuthorityAt {
    pub s: f64,
    pub nu_lp: f64,
    pub nu_hp: f64,
    pub mf: f64,
    pub b: f64,
    pub phi_lp: f64,
}

/// Python's `fuel_authority` return — **RUNG 65's DISCRIMINATOR**, the one thing rung 64 § 3
/// could not measure.
#[derive(Clone, Debug)]
pub struct FuelAuthority {
    pub sm: f64,
    pub tau: f64,
    pub b_cap: f64,
    pub phi_lim: f64,
    pub fracs: Vec<f64>,
    pub at: AuthorityAt,
    pub inst: AuthoritySide,
    pub lagged: AuthoritySide,
    pub ratio: f64,
    pub deleted: bool,
    pub restored: bool,
}

/// **PYTHON'S `max(a, b, c)`, WHICH IS NOT `a.max(b).max(c)` — the difference is `NaN`.**
///
/// `f64::max` DISCARDS a `NaN` operand: `f64::NAN.max(1.0) == 1.0`. Python's builtin holds the
/// first element and replaces it only on a strict `>`, and every comparison against `NaN` is
/// false — so a `NaN` in the FIRST position survives to the end while one in any later position is
/// overwritten. Measured, because "it propagates" was the wrong guess and the two halves differ:
///
/// ```text
/// python: max(nan, 1.0, 2.0) -> nan      rust: NAN.max(1.0).max(2.0) -> 2.0     DIFFERENT
/// python: max(1.0, nan, 2.0) -> 2.0      rust: (1.0).max(NAN).max(2.0) -> 2.0   same
/// ```
///
/// It matters at exactly one call site: [`MarginalMode::laws_held`], whose per-cell value is
/// `float("nan")` when a cell has NO riding points — and `natural` is the FIRST argument, which is
/// the position where the two spellings part company.
///
/// **AND NO INPUT ON THE SHIPPED GRID REACHES IT.** `n_ride` is 340 / 251 / 214 on
/// natural / lo / hi and 340 on both taucells (slice Y step 4), so all three are finite and the
/// aggregate is the same number either way. That is why this was found by asking the reader for
/// its degenerate case rather than by a value key, and why `slice_y_dispatch.rs` gates the
/// FUNCTION rather than the call site: no reachable march distinguishes them.
pub fn py_max3(a: f64, b: f64, c: f64) -> f64 {
    let mut best = a;
    if b > best {
        best = b;
    }
    if c > best {
        best = c;
    }
    best
}

/// The valve position and command a point carries — **a PANIC on the routes whose dict has
/// NEITHER key**, which is [`r65_b_at_point`]'s assert in reader form.
///
/// **SLICE Z's `PointExtra` AUDIT, RECORDED HERE BECAUSE THE COMPILER DID NOT ASK.** Adding a
/// variant breaks the exhaustive matches loudly and leaves the `_ => panic!()` ones silent, and a
/// silent one is a NARROWING: rungs 66/67 record `b` and `b_cmd`, Python reads the dict key, so
/// refusing them would be stricter than the source with every suite green. Four arms were asked
/// the question by hand and the answers split two-two:
///
/// | reader | rung 66/67 dict | verdict |
/// |---|---|---|
/// | this, and [`r65_b_at_point`] | has `b`, `b_cmd` | **WIDENED** |
/// | [`asym_extra`] | has `g`, `required` | **WIDENED** |
/// | `asym_extra` on [`PointExtra::Valve`] | no `g` — Python raises | still refuses |
/// | this on [`PointExtra::Asym`] | no `b` — Python raises | still refuses |
///
/// The wildcard is spelled out as named arms so the NEXT variant breaks the build here too.
///
/// **`pub` RATHER THAN `pub(crate)`, AND THE REASON IS THIS TABLE.** Four test files
/// (`rung65.rs`, `slice_y_smoke.rs`, `slice_y_oracle.rs`, `slice_z_smoke.rs`) each
/// re-match the two keys by hand, which duplicates a decision that is documented HERE and
/// nowhere else — so a widening or a refusal recorded above would not reach them. Exported
/// so slice Z's own suites, and later ones, can call the audited reader instead.
///
/// [`asym_extra`]: crate::fuel_transient::asym_extra
pub fn valve_of(p: &FuelPoint) -> (f64, f64) {
    match p.extra {
        PointExtra::Valve { b, b_cmd } => (b, b_cmd),
        PointExtra::Cascade { b, b_cmd, .. } | PointExtra::CrossCascade { b, b_cmd, .. }
        // SLICE AA: rung 68's five-state march records both.
        | PointExtra::Triple { b, b_cmd, .. }
        // SLICE AD: and the command beside it.
        | PointExtra::Shared { b, b_cmd, .. } => (b, b_cmd),
        PointExtra::None | PointExtra::Asym { .. } =>
            panic!("rung-65 reader on a trajectory with no valve state: this march did not \
                    dispatch to r65_integrate_fuel_valve_lag"),
    }
}

impl ScheduledStatorCore {
    /// Python's `_removed` — the fuel a min-select leg withheld over a march.
    ///
    /// Rung 57's `_cell` formula, **recomputed here** because this rung needs it off a march
    /// carrying `b0`. A deliberate duplication, in Python and therefore here.
    pub fn removed_over(&self, traj: &[FuelPoint]) -> f64 {
        let mut out = 0.0;
        for i in 1..traj.len() {
            let h = traj[i].s - traj[i - 1].s;
            out += 0.5 * h * ((traj[i - 1].mf_sched - traj[i - 1].mf)
                              + (traj[i].mf_sched - traj[i].mf));
        }
        out
    }

    /// **RUNG 65, HALF ONE** — the SAME control law at a sweep of bandwidths, against rung 64's
    /// instantaneous valve on identical hardware.
    ///
    /// Rung 64: the ceiling on the protected coordinate is a property of `b_max`, the lever's
    /// AUTHORITY, which is hardware. This adds the SECOND hardware axis: a valve that cannot
    /// reach its command in time does not deliver its set point either, and it fails for a reason
    /// no control law can touch.
    pub fn bandwidth_ceiling(
        &self, flight: &FlightCondition, ramp: &Ramp, phi_lim: f64, b_cap: f64, taus: &[f64],
    ) -> BandwidthCeiling {
        assert!(phi_lim > 0.0 && b_cap > 0.0 && b_cap < 0.5);
        let shut = self.at_lever(&LeverArm::default()).bill_cell(flight, ramp, false);
        let inst_m = self.at_lever(&LeverArm::floored(BleedLimiter::new(phi_lim, b_cap)));
        let mut inst = inst_m.bill_cell(flight, ramp, true);
        let base: Vec<f64> =
            inst.traj.as_ref().expect("keep_traj").iter().map(|p| p.phi_lp).collect();
        let mut cells: Vec<(f64, BillCell)> = Vec::new();
        let mut rows: Vec<BandwidthRow> = Vec::new();
        for &tau in taus {
            let m = self.at_lever(&LeverArm::floored(
                BleedLimiter::with_tau(phi_lim, b_cap, Some(tau))));
            let mut c = m.bill_cell(flight, ramp, true);
            let traj = c.traj.as_ref().expect("keep_traj");
            let phis: Vec<f64> = traj.iter().map(|p| p.phi_lp).collect();
            let n = base.len().min(phis.len());
            let dev = (0..n).map(|i| (base[i] - phis[i]).abs()).fold(f64::NEG_INFINITY, f64::max);
            let max_track = traj.iter().map(|p| { let (b, c) = valve_of(p); (b - c).abs() })
                                .fold(f64::NEG_INFINITY, f64::max);
            rows.push(BandwidthRow {
                tau,
                min_phi_lp: c.min_phi_lp,
                undershoot: c.min_phi_lp - phi_lim,
                b_int: c.b_int,
                b_peak: c.b_peak,
                b_end: c.b_end,
                plateau_pts: c.plateau_pts,
                plateau_span: c.plateau_span,
                s_at_min_lp: c.s_at_min_lp,
                b_at_min_lp: c.b_at_min_lp,
                saturated: c.b_peak >= b_cap * (1.0 - 1e-12),
                dev,
                // the BILL, in rung 61's currency, against the valve-SHUT reference
                d_nu_lp_end: c.nu_lp_end - shut.nu_lp_end,
                thrust_end_pct: (c.thrust_end / shut.thrust_end - 1.0) * 100.0,
                thrust_int_pct: (c.thrust_int / shut.thrust_int - 1.0) * 100.0,
                d_min_phi_hp: c.min_phi_hp - shut.min_phi_hp,
                max_track,
            });
            c.traj = None;
            cells.push((tau, c));
        }
        let inst_min_phi = inst.min_phi_lp;
        let inst_b_int = inst.b_int;
        let inst_plateau_pts = inst.plateau_pts;
        let inst_d_min_phi_hp = inst.min_phi_hp - shut.min_phi_hp;
        inst.traj = None;
        let under: Vec<f64> = rows.iter().map(|x| x.undershoot).collect();
        let bint: Vec<f64> = rows.iter().map(|x| x.b_int).collect();
        BandwidthCeiling {
            phi_lim,
            b_cap,
            r: ramp.r,
            ds: ramp.ds,
            taus: taus.to_vec(),
            under_monotone: (0..under.len().saturating_sub(1)).all(|i| under[i] <= under[i + 1]),
            bint_monotone: (0..bint.len().saturating_sub(1)).all(|i| bint[i] <= bint[i + 1]),
            dev_shrinks: (0..rows.len().saturating_sub(1))
                .all(|i| rows[i].dev >= rows[i + 1].dev),
            rows,
            shut,
            inst,
            cells,
            inst_min_phi,
            inst_b_int,
            inst_plateau_pts,
            inst_d_min_phi_hp,
        }
    }

    /// **RUNG 65, HALF TWO — THE RUNG.** Rung 64 § 3 on a valve with finite bandwidth.
    ///
    /// **RUNG 64 FOUND** that an instantaneous valve re-pins `phi_lp` to `phi_lim` at ANY fuel, so
    /// rung 49's fuel leg solves `G == 0` across its whole bracket and returns an ARBITRARY POINT
    /// OF A CONTINUUM. **A LAG REPAIRS THE SOLVE AND DOES NOT REMOVE THE CONTINUUM**: inside any
    /// one derivative evaluation the valve is a CONSTANT, so the leg sees rung 42's imposed-valve
    /// plant and returns a definite clip — but the pair still regulates ONE variable with TWO
    /// actuators, so wherever both ride, `db/ds == 0` for every `tau`. `b` is a CONSTANT OF THE
    /// MOTION: a marginal mode selected by the initial condition and nothing else.
    ///
    /// **THE PROOF IS THE `b0` SWEEP, NOT THE FREEZE.** A frozen state could be one initial
    /// condition's coincidence; a CONTINUUM means the frozen value MOVES one-for-one with `b0`
    /// while both control laws stay exactly satisfied and the withheld fuel changes with it.
    #[allow(clippy::too_many_arguments)]
    pub fn marginal_mode(
        &self, flight: &FlightCondition, ramp: &Ramp, sm: f64, b_cap: f64, tau: f64, taus: &[f64],
        d_b0: f64,
    ) -> MarginalMode {
        let cmap = self.arming().map_lp_design;
        let fuel = SurgeLimiter::from_margin(&cmap, Spool::Lp, sm);
        let valve = BleedLimiter::from_margin_tau(&cmap, b_cap, sm, Some(tau));
        let m = self.at_lever(&LeverArm::floored(valve));
        let leg = StatorLeg { accel: None, surge: Some(Floor::Phi(fuel)), tt4_max: None };

        let run = |mach: &ScheduledStatorCore, b0: Option<f64>| -> MarginalCell {
            let (traj, _) = mach.stator_march_scoped(
                flight, ramp, None, &leg, &MarchScope { b0, ..MarchScope::DEFAULT });
            let rides: Vec<&FuelPoint> = traj.iter().filter(|p| p.mf < p.mf_sched).collect();
            let tau_m = mach.fuel.inner.lever.lim.and_then(|l| l.tau).expect("lagged");
            let b_first = valve_of(&traj[0]).0;
            MarginalCell {
                b0: b_first,
                b_end: valve_of(&traj[traj.len() - 1]).0,
                drift: traj.iter().map(|p| (valve_of(p).0 - b_first).abs())
                           .fold(f64::NEG_INFINITY, f64::max),
                dbds: traj.iter().map(|p| { let (b, c) = valve_of(p); (c - b).abs() })
                          .fold(f64::NEG_INFINITY, f64::max) / tau_m,
                removed: self.removed_over(&traj),
                min_phi_lp: traj.iter().map(|p| p.phi_lp).fold(f64::INFINITY, f64::min),
                laws_held: if rides.is_empty() {
                    f64::NAN
                } else {
                    rides.iter().map(|p| (p.phi_lp - fuel.phi_lim).abs())
                         .fold(f64::NEG_INFINITY, f64::max)
                },
                interior: !rides.is_empty()
                    && rides.iter().map(|p| valve_of(p).0).fold(f64::INFINITY, f64::min) > 0.0
                    && rides.iter().map(|p| valve_of(p).0).fold(f64::NEG_INFINITY, f64::max)
                       < b_cap,
                n_ride: rides.len(),
                npts: traj.len(),
            }
        };

        let nat = run(&m, None);
        let b_nat = nat.b0;
        let mut moved = Vec::new();
        for (lbl, x) in [("lo", b_nat - d_b0), ("hi", b_nat + d_b0)] {
            assert!(x > 0.0 && x < b_cap,
                    "rung-65 b0 sweep leaves the valve's stops at {lbl}: {x:.6} not in \
                     (0, {b_cap}). A clamped member is not a member of the continuum.");
            moved.push(run(&m, Some(x)));
        }
        let (lo, hi) = (moved[0], moved[1]);
        // tau-INVARIANCE: the same initial condition at two bandwidths.
        let taucells: Vec<(f64, MarginalCell)> = taus.iter()
            .map(|&t| (t, run(&self.at_lever(&LeverArm::floored(valve.lagged(t))), None)))
            .collect();
        let span = (taucells[0].1.removed - taucells[taucells.len() - 1].1.removed).abs();
        MarginalMode {
            sm,
            tau,
            taus: taus.to_vec(),
            b_cap,
            d_b0,
            r: ramp.r,
            ds: ramp.ds,
            phi_lim: fuel.phi_lim,
            frozen: nat.drift.max(lo.drift).max(hi.drift),
            db_db0: (hi.b0 - lo.b0) / (2.0 * d_b0),
            dremoved: hi.removed - lo.removed,
            // `py_max3`, NOT `f64::max` — see that function. `laws_held` is the ONE reduction in
            // this rung that can carry a `NaN`, and the two spellings disagree there. Its
            // neighbours (`frozen` above, `span` below) fold quantities that cannot be `NaN`, so
            // they keep `f64::max` rather than being swept into a helper they do not need
            // ([[rust-port-copy-vs-rederivation]] — do not factor away a difference that is real).
            laws_held: py_max3(nat.laws_held, lo.laws_held, hi.laws_held),
            interior: nat.interior && lo.interior && hi.interior,
            tau_span: span,
            tau_span_rel: span / taucells[0].1.removed.abs(),
            natural: nat,
            moved_lo: lo,
            moved_hi: hi,
            taucells,
            b_natural: b_nat,
        }
    }

    /// **RUNG 65's DISCRIMINATOR** — the one thing rung 64 § 3 could not measure.
    ///
    /// Rung 49's leg solves `G(w) = phi_lim - phi_lp(w) = 0` in the fuel. Rung 64 DERIVED that an
    /// instantaneous valve makes `G == 0` across the whole bracket; what it could not do is
    /// exhibit the repair, because on its own plant there is nothing to exhibit. Here the same
    /// bracket is swept on BOTH plants at ONE state taken off an armed march.
    ///
    /// **NO WALL-CLOCK NUMBER IS REPORTED.** Rung 64 § 3 measured that a deleted plant makes the
    /// leg GRIND and was explicit that no number about the tangent residual is a result; cost is
    /// machine- and load-dependent, the sign structure of `G` is not.
    #[allow(clippy::too_many_arguments)]
    pub fn fuel_authority(
        &self, flight: &FlightCondition, ramp: &Ramp, sm: f64, b_cap: f64, tau: f64, fracs: &[f64],
    ) -> FuelAuthority {
        let cmap = self.arming().map_lp_design;
        let fuel = SurgeLimiter::from_margin(&cmap, Spool::Lp, sm);
        let valve = BleedLimiter::from_margin_tau(&cmap, b_cap, sm, Some(tau));
        let lag_m = self.at_lever(&LeverArm::floored(valve));
        let (traj, _) = lag_m.stator_march(flight, ramp, None, &StatorLeg::default());
        // where a fuel leg would bite hardest — FIRST-STRICT argmin, as Python's `min(key=…)` is
        let mut at = &traj[0];
        for p in &traj[1..] {
            if p.phi_lp < at.phi_lp {
                at = p;
            }
        }
        let (b_at, _) = valve_of(at);
        assert!(b_at > 0.0 && b_at < b_cap,
                "rung-65's discriminator needs the valve RIDING at the probe state -- at a stop \
                 it is not a control law; got b = {b_at:.6} against [0, {b_cap}].");
        let inst_m = self.at_lever(&LeverArm::floored(BleedLimiter::new(valve.phi_lim, b_cap)));
        let mf = at.mf;
        let side = |mach: &ScheduledStatorCore, state: Option<f64>| -> AuthoritySide {
            let phis: Vec<f64> = fracs.iter().map(|&x| {
                let _g = MarchedBleed::set_opt(&mach.fuel.inner, state);
                mach.fuel.try_instant_fuel(flight, at.nu_lp, at.nu_hp, mf * x)
                    .expect("rung-65 discriminator: the bracket must close on both plants")
                    .base.close.phi_lp
            }).collect();
            let g: Vec<f64> = phis.iter().map(|&v| fuel.phi_lim - v).collect();
            AuthoritySide {
                span: phis.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
                    - phis.iter().cloned().fold(f64::INFINITY, f64::min),
                monotone: (0..phis.len() - 1).all(|i| phis[i] <= phis[i + 1]),
                sign_change: g.iter().cloned().fold(f64::INFINITY, f64::min) < 0.0
                    && g.iter().cloned().fold(f64::NEG_INFINITY, f64::max) > 0.0,
                max_abs_g: g.iter().map(|v| v.abs()).fold(f64::NEG_INFINITY, f64::max),
                phis,
                g,
            }
        };
        let inst = side(&inst_m, None);
        let lagged_side = side(&lag_m, Some(b_at));
        FuelAuthority {
            sm,
            tau,
            b_cap,
            phi_lim: fuel.phi_lim,
            fracs: fracs.to_vec(),
            at: AuthorityAt { s: at.s, nu_lp: at.nu_lp, nu_hp: at.nu_hp, mf, b: b_at,
                              phi_lp: at.phi_lp },
            ratio: lagged_side.span / inst.span.max(1e-300),
            deleted: inst.span < 1e-9,
            restored: lagged_side.span > 1e-4 && lagged_side.monotone,
            inst,
            lagged: lagged_side,
        }
    }
}
