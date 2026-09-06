//! SLICE AG step 2 — **THE FOUR RIGHT-HAND SIDES, WHICH ARE NOT THE FOUR TARGETS.**
//!
//! # THE ONE THING THIS FILE EXISTS TO PIN
//!
//! Rung 75's `_rhs_laws` docstring names the trap and six prior instances of it: the tracking term
//! `(mf_app - w)/tau_t` is in **no leg's target** and `tau_t` is **not in `taus`**, so an inherited
//! gains reader run on the `track` cell would report the masked diagonal unchanged, `det J` still
//! dead and the spectrum invariant — *a perfect refutation of this rung's headline, having measured
//! nothing.* The port inherits two ways to rebuild that failure, and **both compile and pass every
//! reduce gate in the crate**:
//!
//! 1. spelling `rhs_laws` as [`demand_laws`] plus a `/tau` at the call site, which RECONSTRUCTS
//!    the diagonal instead of measuring it;
//! 2. returning rung 74's [`DemandLaws`] type, which makes a TARGET and a RATE interchangeable at
//!    every call site — step 3's `_rhs_gains_at` would consume either without complaint.
//!
//! So the kind difference is pinned twice over: by a separate return type, and by
//! [`the_four_laws_are_RATES_where_rung_74s_are_TARGETS`], which requires the two `F`s to return
//! DIFFERENT NUMBERS at one point and the difference to be exactly the algebra Python writes.
//!
//! # WHAT IS DELIBERATELY NOT HERE
//!
//! **The Jacobian**, the fourteen central differences and everything that reads them: `_rhs_gains_at`
//! is step 3's, and § 5.31.1 (f) registered the ruling that decides P7 on steps 2 and 4 keeping
//! bodies of their own. Pulling it forward would make step 3 a remainder and void that prediction.
//!
//! **NOTHING HERE READS A GOLDEN.** Every assertion is a same-run difference, a bit comparison, or
//! a panic message.

use std::panic::{catch_unwind, AssertUnwindSafe};

use turbojet::anti_windup::{
    build_anti_windup_cascade, rhs_laws, windup_march, WindupScope, WINDUP_LAW_NONE,
    WINDUP_LAW_TRACK,
};
use turbojet::bleed_transient::LeverArm;
use turbojet::applied_reference::REF_LAW_APPLIED as REF_APPLIED;
use turbojet::demand_coordinate::{
    applied_demand, build_demand_coordinate_cascade, coord_march, demand_laws, IC_CAP_DECLARED,
    LAG_COORD_CLIP, LAG_COORD_DEMAND, LAG_COORD_LATCHED,
};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{Floor, FuelPoint, PointExtra, SurgeLimiter};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::sensed_cap::build_sensed_cap_cascade;
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------- the grid
//
// `slice_af_laws.rs`'s grid verbatim, INCLUDING its measured operating point, because this file
// drives the same four laws on the same plant one rung up. The one number this slice adds is the
// tracking clock, and it is the shipped `tau_ts` sweep's own slow end.
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const B: f64 = 0.10;
const PHI: f64 = 0.80;
const V_MAX: f64 = 0.20;
const SM: f64 = PHI / FLOOR - 1.0;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TAU_ATT: f64 = 0.05;
const TT4_MAX: f64 = 1200.0;
const TAU_GOV: f64 = 0.05;
const TAU_T: f64 = 0.05;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;

/// **THE LAW GRID's FIVE CLOCKS ARE DISTINCT ON PURPOSE, AND THE MARCH GRID's ARE UNIFORM ON
/// PURPOSE — and the first draft of this file had only the second, which cost it a whole class of
/// discriminator.**
///
/// Rung 75 is the FIRST rung whose laws *divide* by a clock: rung 74's four rows return targets and
/// solved values, so no `tau` appears in their algebra at all. The march grid above was inherited
/// from there, where every clock being `0.05` is harmless. Here it is not — with `tau_f`, `tau_gov`,
/// `bleed_lim.tau`, `_stator_leg().tau` and `tau_t` all equal, **any permutation of the five
/// divisors passes every assertion in this section**, which a mutation sweep proved by surviving
/// `F ÷ tau_gov` against a gate whose whole subject is which clock each row rides.
///
/// It is a precondition for the NEXT step as well, not a tidy-up of this one. `_jac4` writes
/// `−1/tau` on the diagonal, and this rung's central refused spelling is *`tau_t` added to `taus`*
/// — on a uniform grid `−1/tau_t` and `−1/tau_f` are the same number, so the detector for the trap
/// the module doc pre-registers **cannot exist**. The gains gates inherit these constants.
///
/// The march gates keep the uniform grid deliberately: their cells (the 341-point runs, the
/// `applied` refusal) were measured there, and a march is not where a divisor permutation shows.
const TAU_F_D: f64 = 0.05;
const TAU_GOV_D: f64 = 0.07;
const TAU_Q_D: f64 = 0.11;
const TAU_S_D: f64 = 0.13;
const TAU_T_D: f64 = 0.17;

/// **`slice_af_laws.rs`'s MEASURED point** — the one cell in its five-by-six sweep where all four
/// laws return, both fuel legs are RIDING and neither the valve nor the stator sits on a stop. The
/// last of those is what the flatness assertions here need: two saturated zeros satisfy an equality
/// for the wrong reason.
const A: f64 = 0.85;
const H: f64 = 0.90;
const MF: f64 = 0.040;

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp_map() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn hp_map() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn valve() -> BleedLimiter { BleedLimiter::from_margin_tau(&lp_map(), B, SM, Some(TAU)) }
fn surge() -> SurgeLimiter { SurgeLimiter::from_margin(&lp_map(), Spool::Lp, SM) }

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

/// The valve AND the stator loop — `C` needs a limiter and `V` needs a leg, and both need a `tau`
/// because rung 75's rows DIVIDE by them where rung 74's returned the solved value.
fn arm() -> LeverArm {
    LeverArm {
        bleed_lim: Some(valve()),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU_S))),
        ..Default::default()
    }
}

/// The same two loops as [`arm`], with the valve and the stator on clocks that differ from each
/// other and from the two the caller passes. Only the DIVISORS move: a limiter's `tau` is read by
/// the rate and by nothing else, which the RATES gate asserts rather than assumes.
fn arm_clocks() -> LeverArm {
    LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, SM, Some(TAU_Q_D))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU_S_D))),
        ..Default::default()
    }
}

fn windup(a: &LeverArm) -> ScheduledStatorCore {
    full_of(build_anti_windup_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, a))
}

fn sensed(a: &LeverArm) -> ScheduledStatorCore {
    full_of(build_sensed_cap_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, a))
}

fn demand(a: &LeverArm) -> ScheduledStatorCore {
    full_of(build_demand_coordinate_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, a))
}

/// Arm the one live cell — `demand × applied × track`. The coordinate is load-bearing: `track` is
/// REFUSED anywhere else, which [`the_clock_is_read_at_the_TOP_so_both_refusals_reach_the_rhs`]
/// drives.
fn arm_track(m: &ScheduledStatorCore, tau_t: Option<f64>) {
    m.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);
    m.fuel.inner.windup_law.set(WINDUP_LAW_TRACK);
    m.fuel.inner.tau_t.set(tau_t);
}

fn arm_none(m: &ScheduledStatorCore) {
    m.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);
    m.fuel.inner.windup_law.set(WINDUP_LAW_NONE);
    m.fuel.inner.tau_t.set(None);
}

/// The panic message a closure produces, or `""` — slice AB–AF's helper, for its recorded reason:
/// `assert!(panics(…))` is satisfied by an unrelated bug as readily as by the refusal it names.
fn message_of<F: FnOnce()>(f: F) -> String {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = catch_unwind(AssertUnwindSafe(f));
    std::panic::set_hook(prev);
    match out {
        Ok(()) => String::new(),
        Err(e) => e
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default(),
    }
}

/// The four state coordinates the laws are closures of. `wf < wr` on purpose: under `min`-select
/// that makes the FUEL leg the one holding the actuator, which is what
/// [`the_device_disarms_itself_on_the_leg_that_holds_the_actuator`] needs.
const WF: f64 = 0.9 * MF;
const WR: f64 = 1.1 * MF;
const Q: f64 = 0.02;
const V: f64 = 0.05;

/// The two masked states and `Tt4`, as bits — `slice_ag_cells.rs`'s reader, and its reason: the
/// device moves the STATES and `min`-select hides that from the plant.
fn state_of(p: &FuelPoint) -> (u64, u64, u64) {
    match p.extra {
        PointExtra::Demand { w_fuel, w_gov, .. } =>
            (w_fuel.to_bits(), w_gov.to_bits(), p.tt4.to_bits()),
        _ => panic!("this arming marches the demand coordinate, so every point is `Demand`"),
    }
}

// =============================================================================================
// 1 — THE LAWS ARE RATES, AND THE TRACKING TERM IS THE WHOLE OF WHAT THE DEVICE ADDS
// =============================================================================================

/// **A TARGET AND A RATE ARE DIFFERENT NUMBERS, AND THE DIFFERENCE IS PYTHON'S ALGEBRA.**
///
/// With the device DISARMED, every row here is its rung-74 counterpart differenced against its own
/// state and divided by its own clock — exactly, to the bit, because that is the only arithmetic
/// between them. The inequality half is what a shared return type would have made unaskable: if
/// `rhs.F` merely *equalled* `demand.F` the port would be returning the target and nothing in the
/// crate could tell.
///
/// `C` and `V` change ARITY as well as kind — rung 74's take three states because they return the
/// solved `b` and `v`, and these take four because a rate needs the state it is a rate against.
#[test]
#[allow(non_snake_case)]
fn the_four_laws_are_RATES_where_rung_74s_are_TARGETS() {
    let a = arm_clocks();
    let m = windup(&a);
    arm_none(&m);
    let f = flight();
    let floor = Floor::Phi(surge());

    let d = demand_laws(&m, &f, A, H, MF, None, Some(&floor), TT4_MAX);
    let x = rhs_laws(&m, &f, A, H, MF, None, Some(&floor), TT4_MAX, TAU_F_D, TAU_GOV_D);

    let (df, _) = (d.f)(WF, WR, Q, V).expect("rung 74's F returns at this point");
    let (xf, _) = (x.f)(WF, WR, Q, V).expect("rung 75's F returns at this point");
    assert_ne!(df.to_bits(), xf.to_bits(), "a target and a rate must not be the same number");
    assert_eq!(xf.to_bits(), ((df - WF) / TAU_F_D).to_bits(),
               "F is the target differenced against its own state, on `tau_f`");

    let (dr, _) = (d.r)(WF, WR, Q, V).expect("rung 74's R returns");
    let (xr, _) = (x.r)(WF, WR, Q, V).expect("rung 75's R returns");
    assert_ne!(dr.to_bits(), xr.to_bits());
    assert_eq!(xr.to_bits(), ((dr - WR) / TAU_GOV_D).to_bits(), "R rides `tau_gov`, not `tau_f`");

    let (db, _) = (d.c)(WF, WR, V).expect("rung 74's C returns");
    let (xc, _) = (x.c)(WF, WR, Q, V).expect("rung 75's C returns");
    assert_ne!(db.to_bits(), xc.to_bits());
    assert_eq!(xc.to_bits(), ((db - Q) / TAU_Q_D).to_bits(), "C is `(b - q)/bleed_lim.tau`");

    let (dv, _) = (d.v)(WF, WR, Q).expect("rung 74's V returns");
    let (xv, _) = (x.v)(WF, WR, Q, V).expect("rung 75's V returns");
    assert_ne!(dv.to_bits(), xv.to_bits());
    assert_eq!(xv.to_bits(), ((dv - V) / TAU_S_D).to_bits(), "V is `(v - v)/_stator_leg().tau`");

    // **AND THE DISTINCT RIG MUST MOVE ONLY THE DIVISOR.** A limiter's `tau` is a lag clock, so it
    // has no business inside `_solve_b` or `_solve_v` — but if it leaked in, BOTH sides of the two
    // comparisons above would move together and this rig would be measuring less than the uniform
    // one it replaced, silently. So the two solved values are pinned against the uniform rig.
    let u = arm();
    let mu = windup(&u);
    arm_none(&mu);
    let du = demand_laws(&mu, &f, A, H, MF, None, Some(&floor), TT4_MAX);
    assert_eq!((du.c)(WF, WR, V).unwrap().0.to_bits(), db.to_bits(),
               "the valve solve is clock-free, so only the divisor differs between the two rigs");
    assert_eq!((du.v)(WF, WR, Q).unwrap().0.to_bits(), dv.to_bits(),
               "and so is the stator solve");
}

/// **THE DEVICE ADDS EXACTLY ONE TERM, TO EXACTLY TWO ROWS.**
///
/// `track` minus `none` is `(mf_app - w)/tau_t` on the fuel rows and **bit-for-bit ZERO** on the
/// valve and stator rows — not small, absent. That second half is the one a tolerance-based gate
/// would have blurred: the two non-fuel laws never see `tau_t` at all, and if a port had put the
/// term in the shared closure they would move by ~`1e-3` and pass any `1e-6` comparison of the
/// wrong quantity.
#[test]
fn the_tracking_term_is_the_only_difference_between_the_two_arms() {
    let a = arm_clocks();
    let m = windup(&a);
    let f = flight();
    let floor = Floor::Phi(surge());

    arm_none(&m);
    let none = rhs_laws(&m, &f, A, H, MF, None, Some(&floor), TT4_MAX, TAU_F_D, TAU_GOV_D);
    let n_f = (none.f)(WF, WR, Q, V).unwrap().0;
    let n_r = (none.r)(WF, WR, Q, V).unwrap().0;
    let n_c = (none.c)(WF, WR, Q, V).unwrap().0;
    let n_v = (none.v)(WF, WR, Q, V).unwrap().0;
    drop(none);

    arm_track(&m, Some(TAU_T_D));
    let trk = rhs_laws(&m, &f, A, H, MF, None, Some(&floor), TT4_MAX, TAU_F_D, TAU_GOV_D);
    let ma = applied_demand(MF, WF, WR);
    assert_eq!((trk.f)(WF, WR, Q, V).unwrap().0.to_bits(),
               (n_f + (ma - WF) / TAU_T_D).to_bits(), "F gains the track term on its OWN state");
    assert_eq!((trk.r)(WF, WR, Q, V).unwrap().0.to_bits(),
               (n_r + (ma - WR) / TAU_T_D).to_bits(), "R gains it on ITS own state");
    assert_eq!((trk.c)(WF, WR, Q, V).unwrap().0.to_bits(), n_c.to_bits(),
               "the VALVE row carries no device term at all");
    assert_eq!((trk.v)(WF, WR, Q, V).unwrap().0.to_bits(), n_v.to_bits(),
               "and neither does the STATOR row");
}

/// **THE DEVICE DISARMS ITSELF ON THE LEG THAT HOLDS THE ACTUATOR — an EXACT zero, not a small
/// one.**
///
/// `mf_app = min(mf_sched, wf, wr)`, so on the authoritative leg `mf_app == w_own` and the added
/// term is `0.0/tau_t`. Driven from both sides: at `WF < WR` the fuel leg holds and `F` is
/// unchanged bit-for-bit while `R` moves; swap the two and the roles swap with them. **The control
/// is the half that makes this a measurement** — a body that had dropped the term entirely would
/// pass the first assertion and fail the second.
#[test]
fn the_device_disarms_itself_on_the_leg_that_holds_the_actuator() {
    let a = arm_clocks();
    let m = windup(&a);
    let f = flight();
    let floor = Floor::Phi(surge());

    let rates = |wf: f64, wr: f64| -> ((f64, f64), (f64, f64)) {
        arm_none(&m);
        let n = rhs_laws(&m, &f, A, H, MF, None, Some(&floor), TT4_MAX, TAU_F_D, TAU_GOV_D);
        let pair_n = ((n.f)(wf, wr, Q, V).unwrap().0, (n.r)(wf, wr, Q, V).unwrap().0);
        drop(n);
        arm_track(&m, Some(TAU_T_D));
        let t = rhs_laws(&m, &f, A, H, MF, None, Some(&floor), TT4_MAX, TAU_F_D, TAU_GOV_D);
        let pair_t = ((t.f)(wf, wr, Q, V).unwrap().0, (t.r)(wf, wr, Q, V).unwrap().0);
        (pair_n, pair_t)
    };

    // The FUEL leg holds: `mf_app == wf`, so F's term is exactly zero and R's is not.
    let ((nf, nr), (tf, tr)) = rates(WF, WR);
    assert_eq!(tf.to_bits(), nf.to_bits(), "the holder's own row is untouched, to the bit");
    assert_ne!(tr.to_bits(), nr.to_bits(), "and the MASKED row is where the device acts");

    // The GOVERNOR holds — the same statement with the legs exchanged.
    let ((nf2, nr2), (tf2, tr2)) = rates(WR, WF);
    assert_eq!(tr2.to_bits(), nr2.to_bits(), "now the governor is the holder");
    assert_ne!(tf2.to_bits(), nf2.to_bits(), "and the fuel leg is the masked one");
}

/// **THE `none` ARM IS EXACT BY DISPATCH**, one rung early and one rung late.
///
/// A rung-76 machine on its own reduce arm (`cap_law = "solve"`) runs this inherited body and must
/// produce the identical four numbers: `windup_tau` is read THROUGH the table, so the chain is what
/// answers, and `Ok(None)` from rung 76's cap keeps the shipped solve. Any tolerance here would be
/// the wrong instrument — the branch is not taken, so the bits are equal or the port is wrong.
#[test]
fn the_reduce_arm_is_bit_identical_one_rung_up() {
    let a = arm_clocks();
    let f = flight();
    let floor = Floor::Phi(surge());

    let four = |m: &ScheduledStatorCore| -> [u64; 4] {
        let l = rhs_laws(m, &f, A, H, MF, None, Some(&floor), TT4_MAX, TAU_F_D, TAU_GOV_D);
        [(l.f)(WF, WR, Q, V).unwrap().0.to_bits(), (l.r)(WF, WR, Q, V).unwrap().0.to_bits(),
         (l.c)(WF, WR, Q, V).unwrap().0.to_bits(), (l.v)(WF, WR, Q, V).unwrap().0.to_bits()]
    };

    let m75 = windup(&a);
    arm_none(&m75);
    let m76 = sensed(&a);
    arm_none(&m76);
    assert_eq!(four(&m75), four(&m76), "rung 76 on `solve` IS rung 75 on `none`, by dispatch");

    // And with the device armed, the two still agree — the chain reaches rung 75's body from a
    // rung-76 receiver, which is the half a pointer gate cannot show.
    arm_track(&m75, Some(TAU_T_D));
    arm_track(&m76, Some(TAU_T_D));
    assert_eq!(four(&m75), four(&m76), "and the ARMED device reaches through the same chain");
}

/// **THE CLOCK IS READ AT THE TOP OF THE BODY, SO BOTH REFUSALS FIRE BEFORE ANY LAW RUNS.**
///
/// Python's `tau_t = self._windup_tau()` is the third statement of `_rhs_laws`, above the four
/// closures — so building the laws on a refused cell raises, and no caller can evaluate a single
/// row on a machine running two anti-windup devices at once. A port that read the clock lazily
/// INSIDE `_track` would defer both refusals to the first evaluation, and a reader that only ever
/// touches `C` and `V` would never see them at all.
#[test]
fn the_clock_is_read_at_the_top_so_both_refusals_reach_the_rhs() {
    let a = arm();
    let f = flight();
    let floor = Floor::Phi(surge());

    for coord in [LAG_COORD_CLIP, LAG_COORD_LATCHED] {
        let m = windup(&a);
        arm_track(&m, Some(TAU_T));
        m.fuel.inner.lag_coord.set(coord);
        let msg = message_of(|| {
            rhs_laws(&m, &f, A, H, MF, None, Some(&floor), TT4_MAX, TAU_ATT, TAU_GOV);
        });
        assert!(msg.contains("REFUSED outside the plain DEMAND coordinate"),
                "the coordinate refusal must fire while BUILDING the laws; got {msg:?}");
        assert!(msg.contains(coord), "and it must name the coordinate it got: {msg:?}");
    }

    let m = windup(&a);
    arm_track(&m, None);
    let msg = message_of(|| {
        rhs_laws(&m, &f, A, H, MF, None, Some(&floor), TT4_MAX, TAU_ATT, TAU_GOV);
    });
    assert!(msg.contains("DECLARED"), "an unset clock is refused at the same place: {msg:?}");
}

// =============================================================================================
// 2 — THE SCOPE GUARD: A PAIR, RESTORED AS A PAIR
// =============================================================================================

/// **A HALF-RESTORE IS SILENT, WHICH IS WHY BOTH FIELDS ARE READ AFTER THE DROP.**
///
/// `_windup_tau` refuses an UNSET clock and says nothing about a stale one, so a guard that put the
/// law back and left `_tau_t` armed would hand the next reader a live clock it never declared — and
/// every existing gate would pass, because the law field alone reads correctly.
#[test]
fn the_scope_restores_both_fields_and_reports_the_pair_it_displaced() {
    let a = arm();
    let m = windup(&a);
    arm_none(&m);
    let core = &m.fuel.inner;

    {
        let g = WindupScope::set(core, WINDUP_LAW_TRACK, Some(TAU_T));
        assert_eq!(g.displaced(), (WINDUP_LAW_NONE, None), "the guard reports the PAIR");
        assert_eq!(core.windup_law.get(), WINDUP_LAW_TRACK);
        assert_eq!(core.tau_t.get(), Some(TAU_T));
    }
    assert_eq!(core.windup_law.get(), WINDUP_LAW_NONE, "the law is restored");
    assert_eq!(core.tau_t.get(), None, "AND SO IS THE CLOCK — the half a lone-law guard leaks");

    // Nested, from a non-default base: the inner guard must restore the OUTER one's pair and not
    // the class default, which is the failure mode a single saved field cannot even express.
    core.windup_law.set(WINDUP_LAW_TRACK);
    core.tau_t.set(Some(0.02));
    {
        let _outer = WindupScope::set(core, WINDUP_LAW_NONE, None);
        let g = WindupScope::set(core, WINDUP_LAW_TRACK, Some(TAU_T));
        assert_eq!(g.displaced(), (WINDUP_LAW_NONE, None));
    }
    assert_eq!((core.windup_law.get(), core.tau_t.get()), (WINDUP_LAW_TRACK, Some(0.02)));
}

/// The restore is a `Drop`, so it runs on the UNWIND too — Python's `finally`, and the reason
/// `contraction_law` (step 3) can catch a refusal and go on using the same receiver.
#[test]
fn the_scope_restores_through_a_panic() {
    let a = arm();
    let m = windup(&a);
    arm_none(&m);
    let core = &m.fuel.inner;

    let msg = message_of(|| {
        let _g = WindupScope::set(core, WINDUP_LAW_TRACK, Some(TAU_T));
        panic!("rung-75 test: a reader raising under the guard");
    });
    assert!(msg.contains("a reader raising under the guard"), "{msg:?}");
    assert_eq!((core.windup_law.get(), core.tau_t.get()), (WINDUP_LAW_NONE, None));
}

// =============================================================================================
// 3 — THE MARCH: FIVE KNOBS, SET BEFORE THE MARCH RUNS — AND A PLANT THAT DID NOT EXIST
// =============================================================================================

/// **THE DEVICE CREATES THE PLANT RUNG 74 PROVES DOES NOT EXIST**, and that is this step's leading
/// measurement rather than a gate written from the spec.
///
/// Three of this file's first-draft march gates FAILED, all three inside rung 74's own joint-IC
/// refusal — *a MASKED applied-referenced leg obeys `dw/ds = (cap - mf_app)/tau`, state-independent
/// and POSITIVE, so with no stop in its path it has NO INTERIOR EQUILIBRIUM AT ALL*. That is rung
/// 74 § 4, and the gates were driving the exact cell it describes. Read as a measurement instead of
/// as a bug, the 2×2 is the rung:
///
/// | law | reference | outcome |
/// |---|---|---|
/// | `none` | `sched` | 341 points |
/// | `track` | `sched` | 341 points |
/// | **`none`** | **`applied`** | **RAISES** — the residual stalls at `2.864e-03` after 60 sweeps |
/// | **`track`** | **`applied`** | **341 points** |
///
/// So the tracking term is what gives the masked leg an equilibrium to converge to. The `none` half
/// is the control that makes it a measurement: without it, *the armed march returns* would be
/// satisfied by a cell that was never hard.
#[test]
fn the_device_creates_the_plant_rung_74_says_does_not_exist() {
    let a = arm();
    let f = flight();
    let taus = (TAU_ATT, TAU_GOV, TAU, TAU_S);
    let run = |law: &'static str, tau_t: Option<f64>, refl: &'static str| -> Result<usize, String> {
        let m = windup(&a);
        let mut n = 0usize;
        let msg = message_of(|| {
            n = windup_march(&m, &f, LO, HI, TT4_MAX, SM, taus, R, SETTLE, DS, V_MAX, false,
                             LAG_COORD_DEMAND, refl, law, tau_t, None).3.len();
        });
        if msg.is_empty() { Ok(n) } else { Err(msg) }
    };

    assert!(run(WINDUP_LAW_NONE, None, "sched").is_ok(), "rung 74 HAS a plant under `sched`");
    assert!(run(WINDUP_LAW_TRACK, Some(TAU_T), "sched").is_ok(), "and so does the armed device");

    let bare = run(WINDUP_LAW_NONE, None, REF_APPLIED)
        .expect_err("rung 74 § 4: `demand × applied` has NO interior equilibrium");
    assert!(bare.contains("NO INTERIOR EQUILIBRIUM AT ALL"),
            "and it must be THAT refusal, not some other failure: {bare:?}");
    assert!(run(WINDUP_LAW_TRACK, Some(TAU_T), REF_APPLIED).is_ok(),
            "and the DECLARED device is what makes the same cell converge");
}

/// **THE SIBLING CARRIES FIVE KNOBS WHERE RUNG 74's CARRIES THREE**, and `_ic_cap` is the new one.
///
/// The cap is read from the CALLER: `contraction_law` (step 3) is the first reader in the ladder
/// that raises it, and every march it drives has to see the raised value. Rung 74's `coord_march`
/// on the same arming is the control — it leaves the sibling at [`IC_CAP_DECLARED`], so *rung 75
/// carries it* is a difference between two runs rather than a reading of one.
///
/// **AND THE CARRY IS LOAD-BEARING AS A SET, NOT AS THIS LINE** — a correction this gate's first
/// draft got wrong and a mutation sweep supplied. Removing `windup_march`'s own `_ic_cap`
/// assignment changes nothing anywhere in the slice, and it cannot: rung 75 writes that field from
/// the same source at THREE sites — `r75_at_lever`, `r75_shared_rig` and here — and the march
/// reaches its sibling through the second of them, so any one of the three suffices. Python is
/// identical in this (`engine.py:18673`, `18682` and `18694` all read `self._ic_cap`), so the
/// duplication is the SOURCE's and is kept for the recorded reason. Removing all three at once
/// DOES kill — here and in `slice_ag_cells` — which is what makes the set live rather than dead.
///
/// The behavioural half stands, and is why the set matters: on the cell that needs the device, a
/// cap of 7 makes the same march RAISE — with the residual already down at `2.2e-05`, an order the
/// uncapped sweep clears — while the inherited 60 returns 341 points. Step 3's whole subject is
/// that iteration COUNT.
#[test]
fn the_march_carries_five_knobs_and_the_parent_carries_three() {
    let a = arm();
    let f = flight();
    let taus = (TAU_ATT, TAU_GOV, TAU, TAU_S);

    let m = windup(&a);
    m.fuel.inner.ic_cap.set(1000);
    let (sib, _, _, traj) = windup_march(
        &m, &f, LO, HI, TT4_MAX, SM, taus, R, SETTLE, DS, V_MAX, false,
        LAG_COORD_DEMAND, REF_APPLIED, WINDUP_LAW_TRACK, Some(TAU_T), None);
    assert!(!traj.is_empty());
    assert_eq!(sib.fuel.inner.lag_coord.get(), LAG_COORD_DEMAND);
    assert_eq!(sib.fuel.inner.ref_law.get(), REF_APPLIED);
    assert_eq!(sib.fuel.inner.windup_law.get(), WINDUP_LAW_TRACK);
    assert_eq!(sib.fuel.inner.tau_t.get(), Some(TAU_T));
    assert_eq!(sib.fuel.inner.ic_cap.get(), 1000, "the CALLER's cap — rung 75's new line");

    // THE CONTROL: rung 74's march, same arming, same raised cap on the caller.
    let p = demand(&a);
    p.fuel.inner.ic_cap.set(1000);
    let (psib, _, _, _) = coord_march(
        &p, &f, LO, HI, TT4_MAX, SM, taus, R, SETTLE, DS, V_MAX, false,
        LAG_COORD_DEMAND, "sched", None);
    assert_eq!(psib.fuel.inner.ic_cap.get(), IC_CAP_DECLARED,
               "rung 74's march does NOT carry the cap — that is the line rung 75 adds");

    // AND THE VALUE REACHES THE SWEEP ITSELF: 7 is not enough on the cell that needs the device.
    let low = windup(&a);
    low.fuel.inner.ic_cap.set(7);
    let msg = message_of(|| {
        windup_march(&low, &f, LO, HI, TT4_MAX, SM, taus, R, SETTLE, DS, V_MAX, false,
                     LAG_COORD_DEMAND, REF_APPLIED, WINDUP_LAW_TRACK, Some(TAU_T), None);
    });
    assert!(msg.contains("after 7 iterations"),
            "the carried cap must reach the joint-IC sweep itself; got {msg:?}");
}

/// **THE KNOBS ARE SET BEFORE THE MARCH RUNS, AND THE `applied` CELL IS HOW YOU KNOW.**
///
/// This is the gate a DELEGATING spelling fails. Rung 74's `_coord_march` marches before it
/// returns, so a `windup_march` written as *call the parent, then set the three knobs* would hand
/// back a correctly-labelled sibling carrying a rung-74 trajectory — every field assertion above
/// would still pass and the reduce arm would still be exact.
///
/// On `sched` the defect shows only as a difference in the masked states. On `applied` it is not a
/// difference at all: the parent RAISES there, so a delegating body could not return a trajectory
/// to mislabel. Both halves are asserted, because the first survives if a later rung ever gives
/// rung 74 a plant on that cell.
#[test]
fn the_march_sets_the_knobs_before_it_marches() {
    let a = arm();
    let f = flight();
    let taus = (TAU_ATT, TAU_GOV, TAU, TAU_S);
    let one = |law: &'static str, tau_t: Option<f64>, refl: &'static str| -> Vec<(u64, u64, u64)> {
        let m = windup(&a);
        windup_march(&m, &f, LO, HI, TT4_MAX, SM, taus, R, SETTLE, DS, V_MAX, false,
                     LAG_COORD_DEMAND, refl, law, tau_t, None)
            .3.iter().map(state_of).collect()
    };

    let none = one(WINDUP_LAW_NONE, None, "sched");
    let track = one(WINDUP_LAW_TRACK, Some(TAU_T), "sched");
    assert_eq!(none.len(), track.len(), "same ramp, same number of points");
    let moved = none.iter().zip(&track).filter(|(x, y)| x.0 != y.0 || x.1 != y.1).count();
    assert!(moved > 0, "the device must move the masked states; {moved} of {} did", none.len());

    // The second half — a body that marched before it armed could not produce this at all.
    assert!(!one(WINDUP_LAW_TRACK, Some(TAU_T), REF_APPLIED).is_empty());
}

/// **THE REDUCE: `law = "none"` IS RUNG 74, BIT FOR BIT — AND ON THE CELL RUNG 74 REFUSES, IT IS
/// RUNG 74's REFUSAL.**
///
/// Both halves are dispatch, not tolerance: the three sites slice AF ported dead are guarded by
/// `windup_tau` returning `None`, so on this arm they are not taken at all. The refusal half is the
/// stronger of the two — an equality of trajectories can be satisfied by two bodies that both
/// ignore the device, while reproducing the parent's message character for character means the port
/// took the parent's path through a failure as well as through a success.
#[test]
fn the_none_arm_is_rung_74_including_where_rung_74_refuses() {
    let a = arm();
    let f = flight();
    let taus = (TAU_ATT, TAU_GOV, TAU, TAU_S);

    let m = windup(&a);
    let mine: Vec<_> = windup_march(&m, &f, LO, HI, TT4_MAX, SM, taus, R, SETTLE, DS, V_MAX,
                                    false, LAG_COORD_DEMAND, "sched", WINDUP_LAW_NONE, None,
                                    None).3.iter().map(state_of).collect();
    let p = demand(&a);
    let theirs: Vec<_> = coord_march(&p, &f, LO, HI, TT4_MAX, SM, taus, R, SETTLE, DS, V_MAX,
                                     false, LAG_COORD_DEMAND, "sched", None)
        .3.iter().map(state_of).collect();
    assert_eq!(mine, theirs, "the disarmed device is rung 74, to the bit");

    let m2 = windup(&a);
    let p2 = demand(&a);
    let a_msg = message_of(|| {
        windup_march(&m2, &f, LO, HI, TT4_MAX, SM, taus, R, SETTLE, DS, V_MAX, false,
                     LAG_COORD_DEMAND, REF_APPLIED, WINDUP_LAW_NONE, None, None);
    });
    let b_msg = message_of(|| {
        coord_march(&p2, &f, LO, HI, TT4_MAX, SM, taus, R, SETTLE, DS, V_MAX, false,
                    LAG_COORD_DEMAND, REF_APPLIED, None);
    });
    assert!(!a_msg.is_empty(), "the disarmed device must refuse where rung 74 refuses");
    assert_eq!(a_msg, b_msg, "and with rung 74's message, character for character");
}
