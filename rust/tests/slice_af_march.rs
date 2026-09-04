//! SLICE AF step 3 — **THE SIX-STATE DEMAND MARCH, AND THE FIRST POINT VARIANT WHOSE INHERITED
//! KEYS CHANGE SIGN.**
//!
//! # THE FINDING — widening the readers is necessary and NOT sufficient, because the projections
//! are UNFLOORED
//!
//! Slice AD added `PointExtra::Shared` and measured how far the crate's *the next variant breaks
//! the build* convention reaches: 7 of 20 sites. Adding `PointExtra::Demand` re-runs that
//! measurement one variant on — **32 `match … .extra` sites in `src`, 7 exhaustive, 25 carrying a
//! `_ =>` wildcard**, with the compiler stopping at 6 of the 7 (the seventh, `key_count`, was
//! already updated) and the other 25 compiling in silence. All 32 are widened here, `cross_extra`
//! excepted, which refuses rung 74 for the reason it already refuses rungs 66, 68 and 72.
//!
//! **But the arms that now admit a rung-74 point are being handed a DIFFERENT DOMAIN, and the
//! widening does not notice.** Every variant before this one floors its clips — rung 52's
//! `max(0, ·)` runs after every RK4 step — so `g_fuel`, `g_gov`, `required_fuel` and `required_gov`
//! are `>= 0` by construction, and eight inherited arms read exactly that as a LIVENESS predicate
//! (`required_fuel > 0.0` = *is this leg cutting*). Rung 74's are unfloored projections
//! `mf_sched - w`, and `cap > mf_sched` is REACHABLE on its own shipped arms:
//!
//! | arm | `required_gov < 0` | min | inherited predicate answers |
//! |---|---|---|---|
//! | `demand` | **21 of 341** | `-2.8638e-03` | *not live*, on 21 points where the leg IS tracking |
//! | `demand-latched` | **0 of 341** | `0.0` | unchanged |
//!
//! Python reads the dict key and gets the negative number, so *admitting* is faithful and
//! *refusing* would be stricter than the source — but the two coordinate tags now differ in the
//! SIGN SET every inherited reader sees, which is the sharpest thing this coordinate does to code
//! that predates it. Named at each arm rather than treated as mechanical.
//!
//! # THE ONE PARENT LINE THAT MUST NOT CARRY, AND WHY NO GATE COULD HAVE CAUGHT IT
//!
//! Rung 72 ends every step with `gf = max(0, gf); gr = max(0, gr)`. Rung 74 REPLACES that pair
//! with the latch's conditional clamp, and under plain `"demand"` there is **no state stop at
//! all** — § 4's *no interior equilibrium*. Copying the parent's two lines forward would hand the
//! unlatched arm an anti-windup device by accident and **pass every reduce gate in the crate**,
//! because the `clip` arm never enters this function.
//!
//! It is gated in the direction that CAN be gated: applying the clamp unconditionally cuts the
//! unlatched arm's 20 above-schedule states, so a port that dropped the `if latched` guard dies.
//! **The clamp's own arithmetic never fires** — 0 of 340 on the latched arm, because
//! `demand_target` has already capped the target — which is measured and booked rather than
//! claimed as a bite. The other direction — an ADDED `max(0, w)` — is **arithmetically inert on
//! every shipped arm** and a mutation adding it SURVIVES: `w` never goes negative, minimum
//! `9.444498e-03` over 682 marched points on the two demand tags. That is booked as a measured
//! survivor with its proof, and [`w_never_goes_negative_so_the_added_floor_is_inert`] keeps the
//! proof measured rather than remembered.
//!
//! # `windup_tau` IS THE POSITIVE CONTROL, AND IT DISARMS ITSELF ON THE LEG THAT HOLDS
//!
//! Rung 75's hook returns `None` here, so three ported sites are unreachable — the `2/tau_t` term
//! in the RK4 rate sum, the two back-calculation lines, and `_relax`'s far branch. Rather than
//! book three blind survivors, the hook is INJECTED and the march measured: `Some(1.0)` moves
//! `w_gov` and leaves `w_fuel` **exactly** unchanged, because the fuel leg holds the actuator on
//! 340 of 341 points and `(mf_app - wf)/tau_t` is then identically zero. `Some(0.005)` trips the
//! floor at exactly `2.400`, which is the `2/tau_t` term alone.
//!
//! [`w_never_goes_negative_so_the_added_floor_is_inert`]: w_never_goes_negative_so_the_added_floor_is_inert

use std::panic::{catch_unwind, AssertUnwindSafe};

use turbojet::bleed_transient::{LeverArm, LeverArming};
use turbojet::demand_coordinate::{
    applied_demand, build_demand_coordinate_cascade, IC_CAP_DECLARED, LAG_COORD_CLIP,
    LAG_COORD_DEMAND, LAG_COORD_LATCHED, R74, R74_FUEL, R74_STATOR, R74_TRIPLE, R74_TWO,
};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, Authority, AsymmetricLag, Floor, FuelPoint, PointExtra, SurgeLimiter,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{BleedLimiter, Regime};
use turbojet::map::ComponentMap;
use turbojet::shared_actuator::{IC_ORDER4_DECLARED, REF_LAW_DEFAULT, SHARE_LAW_DEFAULT};
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::three_loop::{StatorLimiter, TripleHooks};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::TwoSpoolTransientCore;

// ---------------------------------------------------------------------------- the grid
//
// `tests/test_rung74.py`'s module constants, and `slice_af_cells.rs`/`slice_af_laws.rs`'s copies
// of them. This step adds no constant of its own.
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
const B: f64 = 0.10;
/// Python's `PHI_ARREST`, which is also `slice_af_cells.rs`'s `PHI`.
///
/// **THE ARM IS CHOSEN BY MEASUREMENT AND NOT BY INHERITANCE**, which is step 2 § (a)'s lesson
/// applied to a THIRD arming. A Python sweep over the three shipped `phi` arms × the three
/// coordinate tags reports this as the only one where all four of this step's gates have something
/// to read: the valve is OFF its stop (`b_cmd = 3.6626e-02` against a `b_max` of `0.10`, so
/// `riding` is a 66-point set rather than the empty one 0.76 gives), the governor leg's projection
/// goes NEGATIVE on 21 points, 20 unlatched post-step states sit above the next schedule value
/// (which is what makes the `if latched` guard observable), and the `clip` and `demand`
/// trajectories separate by 36% in the applied fuel.
const PHI: f64 = 0.80;
const V_MAX: f64 = 0.20;
const SM: f64 = PHI / FLOOR - 1.0;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TAU_GOV: f64 = 0.05;
const TAU_ATT: f64 = 0.05;
const TAU_REL: f64 = 0.15;
const TT4_MAX: f64 = 1200.0;

/// `int(round((R + SETTLE) / DS)) + 1` — Python's own point count, and it is 341 on this ramp.
const NPTS: usize = 341;

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
fn lag() -> AsymmetricLag { AsymmetricLag::new(TAU_ATT, TAU_REL) }
fn ramp() -> Ramp { Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds: DS } }

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

/// The valve AND the stator loop — Python's `_demand(design)` default arming.
fn arm() -> LeverArm {
    LeverArm {
        bleed_lim: Some(valve()),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU_S))),
        ..Default::default()
    }
}

fn demand_machine(a: &LeverArm) -> ScheduledStatorCore {
    full_of(build_demand_coordinate_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, a))
}

/// A rung-74 machine under a NAMED coordinate and rung 72's `"sched"` reference — Python's
/// `_demand(design, coord=…, ref="sched")`, which is what every marching cell in `test_rung74.py`
/// uses.
///
/// **THE `ref_law` LINE IS NOT DECORATION.** `build_demand_coordinate_cascade` writes `"applied"`,
/// because Python's class attribute is INHERITED from rung 73 — and `('demand', 'applied')` is the
/// cell whose joint initial condition provably does not converge (§ 4). A gate that forgot this
/// line would be measuring a refusal, which is exactly what
/// [`the_joint_ic_refuses_demand_times_applied_and_that_is_section_4`] drives on purpose.
fn marching(coord: &'static str) -> ScheduledStatorCore {
    let m = demand_machine(&arm());
    m.fuel.inner.ref_law.set(REF_LAW_DEFAULT);
    m.fuel.inner.lag_coord.set(coord);
    m
}

/// The FULLY ARMED march — Python's `_march`: the surge floor, the asymmetric lag, the governor
/// clock and its set point, reached through the real march entry so the ramp supplies the schedule
/// the plant is matched to (`slice_af_cells.rs` § (e)'s recorded reason).
fn march(m: &ScheduledStatorCore) -> Vec<FuelPoint> {
    let leg = StatorLeg { accel: None::<&AccelSchedule>, surge: Some(Floor::Phi(surge())),
                          tt4_max: Some(TT4_MAX) };
    m.stator_march_scoped(&flight(), &ramp(), None, &leg,
                          &MarchScope { lag: Some(lag()), tau_gov: Some(TAU_GOV),
                                        ..MarchScope::DEFAULT }).0
}

fn message_of<F: FnOnce()>(f: F) -> String {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = catch_unwind(AssertUnwindSafe(f));
    std::panic::set_hook(prev);
    match out {
        Ok(()) => String::new(),
        Err(e) => e.downcast_ref::<String>().cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default(),
    }
}

/// The five-tuple per point the value gates compare, BIT for bit.
fn keys(traj: &[FuelPoint]) -> Vec<[u64; 5]> {
    traj.iter()
        .map(|p| [p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.mf.to_bits(), p.tt4.to_bits(),
                  demand_of(p).0.to_bits()])
        .collect()
}

/// `(w_fuel, w_gov, cap_fuel, cap_gov, g_fuel, g_gov)` off a rung-74 point, refusing anything else
/// — so a gate that was handed the parent's trajectory fails LOUDLY instead of comparing nothing.
fn demand_of(p: &FuelPoint) -> (f64, f64, f64, f64, f64, f64) {
    match p.extra {
        PointExtra::Demand { w_fuel, w_gov, cap_fuel, cap_gov, g_fuel, g_gov, .. } =>
            (w_fuel, w_gov, cap_fuel, cap_gov, g_fuel, g_gov),
        _ => panic!("this gate needs a rung-74 DEMAND point and was handed another route"),
    }
}

/// Rebuild a machine with an injected third-loop table — `slice_af_cells.rs`/`slice_af_laws.rs`'s
/// `with_triple`, verbatim, plus the coordinate carry a march needs.
fn with_triple(
    core: &ScheduledStatorCore, a: &LeverArm, triple: &'static TripleHooks,
) -> ScheduledStatorCore {
    let c = full_of(ScheduledStatorTransient::with_ref_tables(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(),
        a.stator, &R74_TWO, &R74_STATOR, &R74_FUEL, &R74,
        LeverArming { bleed: a.bleed, sched: a.bleed_sched, lim: a.bleed_lim },
        triple, a.stator_lim, a.stator_inc));
    c.fuel.inner.ref_law.set(core.fuel.inner.ref_law.get());
    c.fuel.inner.lag_coord.set(core.fuel.inner.lag_coord.get());
    c
}

// =============================================================================================
// 1 — THE MARCH RUNS, AND EVERY POINT IS THIS RUNG'S
// =============================================================================================

/// **THE CONVERTED STEP-1 GATE.** `slice_af_cells.rs` asserted that a legal demand call REACHED
/// the `unimplemented!` — its purpose being *prove the demand arm does not silently delegate to
/// rung 73*. That obligation did not expire when the body landed; it changed shape, and this is
/// the shape.
///
/// **THE WEAK FORM WOULD BE ASSERTING THE POINTS COME BACK AS `Demand`** — a march that built
/// `Demand` points out of a delegated parent trajectory would pass it. The strong form is beside
/// it in [`the_reduce_arm_and_the_demand_march_are_different_trajectories`]: a measured value
/// separation on the same arming.
#[test]
fn the_demand_march_dispatches_to_rung_74_and_carries_thirty_five_keys() {
    for coord in [LAG_COORD_DEMAND, LAG_COORD_LATCHED] {
        let traj = march(&marching(coord));
        assert_eq!(traj.len(), NPTS, "{coord:?}: `int(round((R+SETTLE)/DS)) + 1` points");
        for p in &traj {
            // 35 is READ OFF THE LIVE PYTHON DICT (`len(traj[0])`), never counted off the struct
            // ([[rust-port-guessed-census-bars]]).
            assert_eq!(p.key_count(), 35, "rung 74's point is the widest in the port");
            let PointExtra::Demand { ic_order, share_law, lag_coord, ic_res, ic_iters, .. } =
                p.extra
            else {
                panic!("a legal demand march produced a point that is not this rung's — the one \
                        failure step 1's `unimplemented!` existed to make impossible");
            };
            assert_eq!(ic_order, IC_ORDER4_DECLARED);
            assert_eq!(share_law, SHARE_LAW_DEFAULT);
            assert_eq!(lag_coord, coord, "the coordinate is recorded per point, as Python does");
            assert!(ic_res <= 1e-12 && ic_iters >= 1);
        }
    }
}

/// **THE STRONG ANTI-DELEGATION FORM: the `clip` arm and the demand arm are DIFFERENT PLANTS on
/// one arming, by 36% in the applied fuel.**
///
/// `clip` leaves through the dispatch and marches rung 73/72 — so it comes back as
/// [`PointExtra::Shared`], which is a second, structurally independent statement of the same fact.
/// A port that delegated the demand arm would agree with this trajectory bit for bit and produce
/// the wrong variant; one that produced the right variant from a delegated march would agree in
/// VALUE. Both are refused here.
#[test]
fn the_reduce_arm_and_the_demand_march_are_different_trajectories() {
    let clip = march(&marching(LAG_COORD_CLIP));
    let dem = march(&marching(LAG_COORD_DEMAND));
    assert_eq!(clip.len(), NPTS);
    assert_eq!(dem.len(), NPTS);
    assert!(matches!(clip[0].extra, PointExtra::Shared { .. }),
            "P3 — `clip` is rung 73 by NOT ENTERING, so its points are the parent's variant");
    assert!(matches!(dem[0].extra, PointExtra::Demand { .. }));

    // The two marches start from the same running line, so a separation is the coordinate's.
    assert_eq!(clip[0].nu_lp.to_bits(), dem[0].nu_lp.to_bits());
    assert_eq!(clip[0].mf.to_bits(), dem[0].mf.to_bits());
    let (a, b) = (clip[NPTS - 1].mf, dem[NPTS - 1].mf);
    let sep = (a - b).abs() / a;
    assert!(sep > 0.30,
            "the demand coordinate is a different plant by the end of the ramp: applied fuel \
             {a:.6e} (clip) against {b:.6e} (demand), separation {sep:.3}");
    assert!((clip[NPTS - 1].nu_lp - dem[NPTS - 1].nu_lp).abs() > 1e-3,
            "and the spools land somewhere else too");
}

/// **THE APPLIED DEMAND CLOSES THE FUEL IDENTITY**, and the reference is the RECORDED STATES rather
/// than the march's own answer: `mf` is re-derived from `w_fuel`, `w_gov` and `mf_sched` through
/// the shipped law and compared bit for bit.
///
/// This is the demand coordinate's replacement for rung 72's `mf = mf_sched - applied_clip(gf, gr)`
/// — and `g` on the point is `mf_sched - mf`, so an inherited reader asking *what did the limiter
/// take off* still gets the answer that keeps `mf = mf_sched - g` true.
#[test]
fn the_applied_demand_closes_the_fuel_identity_from_the_recorded_states() {
    let traj = march(&marching(LAG_COORD_DEMAND));
    for p in &traj {
        let PointExtra::Demand { g, w_fuel, w_gov, .. } = p.extra else { panic!() };
        let re = 1e-9f64.max(applied_demand(p.mf_sched, w_fuel, w_gov));
        assert_eq!(re.to_bits(), p.mf.to_bits(),
                   "s = {}: the applied fuel is `max(1e-9, min(mf_sched, wf, wr))` of the states \
                    this very point records", p.s);
        assert_eq!(g.to_bits(), (p.mf_sched - p.mf).to_bits(),
                   "and `g` is the projection an inherited reader needs");
    }
}

// =============================================================================================
// 2 — THE LEADING FINDING: THE INHERITED KEYS CHANGE SIGN
// =============================================================================================

/// **THE UNFLOORED PROJECTIONS GO NEGATIVE, AND THE TWO COORDINATE TAGS DIFFER IN THE SIGN SET
/// EVERY INHERITED READER SEES.**
///
/// The counts are Python's, measured against the shipped `_integrate_fuel_demand` before a line of
/// this file was written — a reference from outside the code under test. `demand-latched` produces
/// none, because [`demand_target`] caps each target at the schedule, so the same reader sees two
/// different domains depending on a tag it has never heard of.
///
/// [`demand_target`]: turbojet::demand_coordinate::demand_target
#[test]
fn the_unfloored_projections_go_negative_and_the_latch_removes_them() {
    let dem = march(&marching(LAG_COORD_DEMAND));
    let lat = march(&marching(LAG_COORD_LATCHED));

    let neg = |t: &[FuelPoint]| -> (usize, usize, f64) {
        let mut ng = 0usize;
        let mut nr = 0usize;
        let mut lo = f64::INFINITY;
        for p in t {
            let PointExtra::Demand { g_gov, required_gov, .. } = p.extra else { panic!() };
            if g_gov < 0.0 { ng += 1; }
            if required_gov < 0.0 { nr += 1; }
            lo = lo.min(required_gov);
        }
        (ng, nr, lo)
    };

    let (ng, nr, lo) = neg(&dem);
    assert_eq!((ng, nr), (21, 21),
               "Python measures 21 of 341 points over-schedule on the governor leg");

    // **AND THE OVER-SCHEDULE REGION IS ONE-LEGGED AT THIS ARM, WHICH IS WHY THE LATCH IS GATED
    // ON THE GOVERNOR AND NOT ON THE FUEL LEG.** `cap_fuel > mf_sched` at 0 of 341 points and
    // `cap_gov > mf_sched` at 21, so `demand_target`'s latch is arithmetically inert on the fuel
    // leg here and a mutation deleting THAT call survives — measured, and the counts are the
    // reachability statement rather than a by-product.
    let (of, og) = dem.iter().fold((0usize, 0usize), |(a, b), p| {
        let (_, _, cf, cg, ..) = demand_of(p);
        (a + usize::from(cf > p.mf_sched), b + usize::from(cg > p.mf_sched))
    });
    assert_eq!((of, og), (0, 21),
               "the fuel leg never rides above the schedule on this arm; the governor does");
    assert!(lo < -1e-3, "and the deepest is {lo:.4e}, nowhere near a rounding artefact");
    assert_eq!(neg(&lat), (0, 0, 0.0),
               "the LATCH removes every one of them, so the two tags hand the same reader two \
                different sign sets");

    // AND THE INHERITED PREDICATE SEES IT. `over` is a point Python says is over-schedule; the
    // rung-72 liveness test `required_gov > 0.0` answers *not live* there, which is what Python's
    // own `p["required_gov"] > 0` answers on the same dict.
    let over = dem.iter().find(|p| {
        let PointExtra::Demand { required_gov, .. } = p.extra else { panic!() };
        required_gov < 0.0
    }).expect("21 such points");
    let (_, _, _, cap_gov, _, g_gov) = demand_of(over);
    assert!(cap_gov > over.mf_sched,
            "the negative projection IS the over-schedule cap: {cap_gov:.6e} > {:.6e}",
            over.mf_sched);
    assert!(g_gov < 0.0);
    // rung 52's reader, widened at slice AF site 5, answering rather than refusing — and the
    // number it answers with is NEGATIVE, which slice AD's own version of this gate asserted
    // could not happen (`assert!(g >= 0.0 && req >= 0.0)`).
    let (g, req) = turbojet::fuel_transient::asym_extra(over);
    assert!(g.is_finite() && req.is_finite());
    assert!(req >= 0.0, "`required` is the BINDING one, `max(ms-cf, ms-cr)`, so it survives");
}

// =============================================================================================
// 3 — THE LATCH IS THE ONLY STATE STOP, AND THE PARENT'S FLOOR IS NOT COPIED
// =============================================================================================

/// **THE `if latched` GUARD IS LIVE; THE CLAMP IT GUARDS IS A DEAD SITE — AND THE FIRST DRAFT OF
/// THIS GATE CLAIMED THE OPPOSITE.**
///
/// The draft asserted *the latch bites*, reading the 20-vs-0 split between the two tags as this
/// march's own clamp working. It is not: [`demand_target`] (step 2's code) already caps each
/// TARGET at the schedule under `demand-latched`, and the ramp is non-decreasing, so a state that
/// starts at `mf_sched(0)` and tracks a target never above the schedule can never rise above the
/// NEXT schedule value. Measured against the shipped Python: on the latched arm the clamp fires
/// **0 of 340 times**, and the one exact equality in that trajectory is at `s = 0`, which is the
/// joint IC's `_stop` and not this block.
///
/// So what this gate can hold, and does:
///
/// * **the GUARD** — applying the clamp unconditionally cuts the unlatched arm's 20
///   above-schedule states, so a port that dropped `if latched` dies here;
/// * **the IC's `_stop`** — live, and gated below: latched, the governor's demand starts exactly
///   AT `mf_sched(0)`; unlatched it settles above it.
///
/// **The clamp's own arithmetic — the `min` fold and the `s + ds` read — is UNREACHABLE on every
/// arm this rung ships**, so mutations to either survive. The one shape that would reach it is a
/// DECREASING schedule (`nxt < ms`), and that route is closed by a second shipped refusal: a decel
/// ramp down from `mf_hi` raises `_cap_free`'s *the UNFLOORED cap is unreachable above mf_sched*,
/// the same guard step 2 § (a) hit. Disclosed rather than dropped.
///
/// [`demand_target`]: turbojet::demand_coordinate::demand_target
#[test]
fn the_latch_guard_is_live_and_the_clamp_it_guards_is_a_dead_site() {
    let dem = march(&marching(LAG_COORD_DEMAND));
    let lat = march(&marching(LAG_COORD_LATCHED));
    let above = |t: &[FuelPoint]| -> usize {
        (0..t.len() - 1)
            .filter(|&i| demand_of(&t[i]).1 > t[i + 1].mf_sched)
            .count()
    };
    assert_eq!(above(&dem), 20,
               "unlatched, the governor demand sits above the NEXT schedule value on 20 points -- \
                Python's own count, and the states an unconditional clamp would have cut");
    assert_eq!(above(&lat), 0,
               "latched, none does -- because `demand_target` already capped the TARGET, which is \
                why the clamp itself never has to fire");

    // THE CLAMP NEVER FIRES ON THE LATCHED ARM, stated as the measurement it is. A fired clamp
    // writes `w = mf_sched` EXACTLY, so the count of exact equalities after `s = 0` IS its
    // firing count.
    let fired = lat.iter().skip(1)
        .filter(|p| { let (wf, wr, ..) = demand_of(p);
                      wf == p.mf_sched || wr == p.mf_sched })
        .count();
    assert_eq!(fired, 0,
               "0 of 340 -- the clamp's arithmetic is a DEAD SITE at this rung, so a mutation to \
                its fold or to its `s + ds` read SURVIVES. Booked, not papered over.");

    // THE IC's `_stop`, WHICH **IS** LIVE. Latched, the joint sweep stops the governor demand at
    // the schedule; unlatched, it settles above it. One bit of the same `latched` flag, read in a
    // different place.
    assert_eq!(demand_of(&lat[0]).1.to_bits(), lat[0].mf_sched.to_bits(),
               "latched: the sweep's `_stop` pins the governor demand AT mf_sched(0)");
    assert!(demand_of(&dem[0]).1 > dem[0].mf_sched,
            "unlatched: it settles ABOVE it, {:.6e} against {:.6e}",
            demand_of(&dem[0]).1, dem[0].mf_sched);
    assert_eq!(demand_of(&dem[0]).0.to_bits(), demand_of(&lat[0]).0.to_bits(),
               "and the FUEL leg, which is below the schedule either way, is bit-identical -- so \
                the difference is the stop and not a second change");
}

/// **THE BOOKED SURVIVOR, WITH ITS PROOF KEPT MEASURED.**
///
/// The dangerous half of *do not copy the parent's `max(0, ·)`* is gated above. The other half —
/// an ADDED floor on `w`, on top of the latch — is arithmetically INERT on every arm this suite
/// runs, so a mutation adding it SURVIVES and no value gate can be written for it. That is a
/// measurement, not a hope, and this gate is what keeps it one: the day a plant drives a demand
/// state to zero, this fails and the booking is re-opened rather than remembered.
#[test]
fn w_never_goes_negative_so_the_added_floor_is_inert() {
    let mut lo = f64::INFINITY;
    let mut n = 0usize;
    for coord in [LAG_COORD_DEMAND, LAG_COORD_LATCHED] {
        for p in &march(&marching(coord)) {
            let (wf, wr, ..) = demand_of(p);
            lo = lo.min(wf).min(wr);
            n += 1;
        }
    }
    assert_eq!(n, 2 * NPTS);
    assert!(lo > 1e-3,
            "the smallest demand state over {n} marched points is {lo:.6e} — three orders of \
             magnitude above the floor a copied `max(0, w)` would impose, which is exactly why \
             that mutation survives and is booked rather than gated");
}

// =============================================================================================
// 4 — THE JOINT INITIAL CONDITION: A CELL WITH ITS FIRST READER, AND § 4's REFUSAL
// =============================================================================================

/// **`ic_cap` IS A LIVE CELL AT THIS RUNG, WHICH STEP 2 BOOKED AS AN OPEN QUESTION.**
///
/// Step 2 § (h) asked what distinguishes `ic_cap = 60` from `1000` and warned that the answer might
/// be *nothing*, in which case it is a measurement to book rather than a gate to manufacture. It is
/// half of each: the sweep settles in **2** passes with `ic_res` exactly `0.0`, so every cap at or
/// above 2 gives a bit-identical trajectory — and at **1** the same arm RAISES. So the field has a
/// reader, the reader has a threshold, and the threshold sits one below the shipped value.
///
/// A port that hardcoded the parent's `1..=60` would pass the first two rows and fail the third.
#[test]
fn ic_cap_is_a_live_cell_and_its_threshold_is_one_below_the_declared_value() {
    let at = |cap: usize| -> ScheduledStatorCore {
        let m = marching(LAG_COORD_DEMAND);
        m.fuel.inner.ic_cap.set(cap);
        m
    };
    let declared = march(&at(IC_CAP_DECLARED));
    assert_eq!(declared[0].key_count(), 35);
    let PointExtra::Demand { ic_iters, ic_res, .. } = declared[0].extra else { panic!() };
    assert_eq!(ic_iters, 2, "the four-way sweep settles in two passes on this arm");
    assert_eq!(ic_res, 0.0, "and to an EXACT zero, which is why 60 and 1000 cannot differ");

    assert_eq!(keys(&march(&at(2))), keys(&declared),
               "so `ic_cap = 2` is the same trajectory bit for bit");

    let msg = message_of(|| { march(&at(1)); });
    assert!(msg.contains("rung-74") && msg.contains("did not converge"),
            "and `ic_cap = 1` cuts the sweep short: {msg:?}");
    assert!(msg.contains("after 1 iterations"),
            "with the CAP itself in the message, which is what makes the cell observable: \
             {msg:?}");
}

/// **§ 4's FINDING, AS A SHIPPED REFUSAL: `('demand', 'applied')` HAS NO INTERIOR EQUILIBRIUM.**
///
/// A masked applied-referenced leg obeys `dw/ds = (cap - mf_app)/tau`, which is state-independent
/// and positive, so with no stop in its path the sweep does not settle. It is reached without
/// touching anything: `build_demand_coordinate_cascade` writes `"applied"` because Python's class
/// attribute is inherited, so the DEFAULT rung-74 machine under `"demand"` is this cell.
///
/// **AND THE CONTROL IS THE LATCH**, on the same machine and the same reference: the stop restores
/// the equilibrium and the march completes. That is what makes the refusal a statement about the
/// missing stop rather than about the reference law.
#[test]
fn the_joint_ic_refuses_demand_times_applied_and_that_is_section_4() {
    let m = demand_machine(&arm());
    m.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);
    let msg = message_of(|| { march(&m); });
    assert!(msg.contains("rung-74") && msg.contains("did not converge"), "{msg:?}");
    assert!(msg.contains("NO INTERIOR EQUILIBRIUM AT ALL"),
            "the refusal states the mechanism, not just the residual: {msg:?}");
    assert!(msg.contains("after 60 iterations"), "and it ran the declared cap: {msg:?}");
    assert!(msg.contains("Neither is a cap to raise"), "{msg:?}");

    let l = demand_machine(&arm());
    l.fuel.inner.lag_coord.set(LAG_COORD_LATCHED);
    let traj = march(&l);
    assert_eq!(traj.len(), NPTS,
               "THE CONTROL — the same reference WITH the latch's stop marches to the end, so the \
                refusal is about the missing stop and not about `applied`");

    // **AND THIS IS THE ONLY MARCHING ARM ON WHICH `demand_reference` IS LIVE INSIDE `der`**, so
    // the control is also the gate for it. Every other arm carries `ref_law = "sched"`, where the
    // body returns the cap untouched — the sweep found the call deletable and survived, and this
    // is the repair. Against the `"sched"` twin: `w_gov` moves by `1.11e-02` while the applied
    // fuel and both speeds are BIT-IDENTICAL, because the governor leg is the masked one here and
    // the applied reference reaches only a masked leg.
    let s = marching(LAG_COORD_LATCHED);
    let sched = march(&s);
    let dw = (0..NPTS).map(|i| (demand_of(&traj[i]).1 - demand_of(&sched[i]).1).abs())
                      .fold(0.0f64, f64::max);
    assert!(dw > 1e-3,
            "`applied` moves the governor demand against `sched`: max |dw_gov| = {dw:.6e}");
    for i in 0..NPTS {
        assert_eq!(traj[i].mf.to_bits(), sched[i].mf.to_bits(),
                   "and moves NOTHING else — the masked leg is masked, which is § 1's whole \
                    claim surviving the change of coordinate (point {i})");
        assert_eq!(traj[i].nu_lp.to_bits(), sched[i].nu_lp.to_bits());
        assert_eq!(demand_of(&traj[i]).0.to_bits(), demand_of(&sched[i]).0.to_bits());
    }
}

// =============================================================================================
// 5 — RUNG 75's HOOK: THE POSITIVE CONTROL FOR THREE SITES THAT ARE DEAD HERE
// =============================================================================================

thread_local! {
    static TAU_T: std::cell::Cell<Option<f64>> = const { std::cell::Cell::new(None) };
}

fn injected_windup_tau(_: &TwoSpoolTransientCore) -> Option<f64> {
    TAU_T.with(|c| c.get())
}

static INJ: TripleHooks = TripleHooks { windup_tau: injected_windup_tau, ..R74_TRIPLE };

/// **THE INSTRUMENT IS PROVED ABLE TO SEE BEFORE THREE SURVIVORS ARE BOOKED, AND IT MEASURES RUNG
/// 75's OWN CLAIM ON THE WAY.**
///
/// `windup_tau` returns `None` at rung 74, so the `2/tau_t` term in the RK4 rate sum, the two
/// back-calculation lines in `der` and `_relax`'s far branch are unreachable and a mutation
/// deleting any of them survives. Injecting a table that answers `Some` turns all three live:
///
/// * `Some(1.0)` moves `w_gov` and leaves `w_fuel` **EXACTLY** unchanged — not approximately.
///   `(mf_app - w)/tau_t` is identically zero on whichever leg HOLDS the actuator, and the fuel
///   leg holds it on 340 of 341 points here. That is the device disarming itself on the
///   authoritative leg, which is rung 75's headline, arriving as a step-3 measurement.
/// * `Some(0.005)` trips the RK4 floor at exactly `2.400` = `0.005 * (80 + 2/0.005)`, which is the
///   `2/tau_t` term alone and nothing else in the sum.
#[test]
fn the_windup_hook_is_dispatched_and_disarms_itself_on_the_leg_that_holds() {
    let base = march(&marching(LAG_COORD_DEMAND));
    let holds = base.iter()
        .filter(|p| matches!(p.extra, PointExtra::Demand { authority: Authority::Fuel, .. }))
        .count();
    assert_eq!(holds, NPTS - 1, "the fuel leg holds the actuator on all but the first point");

    let injected = |tau_t: Option<f64>| -> ScheduledStatorCore {
        TAU_T.with(|c| c.set(tau_t));
        let m = with_triple(&marching(LAG_COORD_DEMAND), &arm(), &INJ);
        assert!(std::ptr::eq(m.fuel.inner.triple_hooks, &INJ), "the injected table is installed");
        m
    };

    let lifted = march(&injected(Some(1.0)));
    assert_eq!(lifted.len(), NPTS);
    let mut moved_gov = 0.0f64;
    for (p, q) in lifted.iter().zip(base.iter()) {
        let (wf, wr, ..) = demand_of(p);
        let (wf0, wr0, ..) = demand_of(q);
        assert_eq!(wf.to_bits(), wf0.to_bits(),
                   "s = {}: the AUTHORITATIVE leg is bit-identical — the device's own term is \
                    exactly zero there", p.s);
        moved_gov = moved_gov.max((wr - wr0).abs());
    }
    assert!(moved_gov > 1e-5,
            "and the MASKED leg moves: max |dw_gov| = {moved_gov:.6e}");

    // **AND THE LAST POINT IS WHAT SEPARATES THE MARCH's TERM FROM THE SWEEP's.** A maximum over
    // the trajectory does NOT: the joint IC's `_relax` shifts `w_gov(0)` by exactly the same
    // amount, so `max |dw_gov| > 1e-5` is satisfied by the sweep alone and the mutation that
    // DELETES the two back-calculation lines survives it — which is what the sweep reported, and
    // it is this gate's defect and not the port's.
    //
    // The governor lag is `tau_gov = 0.05` against `s_end = 1.7`, i.e. THIRTY-FOUR time constants,
    // so a shift present only in the initial condition is gone long before the end. A shift still
    // there at `s = 1.7` can only be held by a term in the DERIVATIVE. It is, and its size is the
    // quasi-steady offset the two terms balance at: `(ma - tgt)/(1 + tau_t/tau_gov)`, which at
    // `tau_t = 1.0` is `(ma - tgt)/21` and reads `1.363735e-04` in Python.
    let (last_l, last_b) = (demand_of(&lifted[NPTS - 1]).1, demand_of(&base[NPTS - 1]).1);
    assert!((last_l - last_b).abs() > 1e-5,
            "the offset SURVIVES 34 governor time constants, so it is held by the derivative and \
             not inherited from the sweep: w_gov(1.7) is {last_l:.6e} injected against \
             {last_b:.6e} bare");

    // **AND `relax`'s FAR BRANCH IS ISOLATED BY POINT ZERO.** The first recorded point is written
    // before a single derivative is integrated, so its states come from the JOINT SWEEP alone and
    // nothing the march does can reach it. The governor demand moves there and the fuel demand
    // does not, for the same self-disarming reason: at the fixed point of a leg that HOLDS,
    // `ma == w`, and `(tau_t*tgt + tau*w)/(tau + tau_t)` is then `tgt` identically.
    assert!((demand_of(&lifted[0]).1 - demand_of(&base[0]).1).abs() > 1e-5,
            "`relax` is live in the sweep: w_gov(0) is {:.6e} injected against {:.6e} bare",
            demand_of(&lifted[0]).1, demand_of(&base[0]).1);
    assert_eq!(demand_of(&lifted[0]).0.to_bits(), demand_of(&base[0]).0.to_bits(),
               "and inert on the leg that holds, in the sweep as in the march");

    // THE `2/tau_t` TERM IN THE RATE SUM, ALONE.
    let msg = message_of(|| { march(&injected(Some(0.005))); });
    TAU_T.with(|c| c.set(None));
    assert!(msg.contains("rung-74") && msg.contains("2.400"),
            "0.005 * (80 + 2/0.005) = 2.400 — the factor TWO is what the message reads back, so a \
             port summing `1/tau_t` would print 1.400 here: {msg:?}");
    assert!(msg.contains("ACTIVE lag"), "and it is this rung's own floor message: {msg:?}");
}

// =============================================================================================
// 6 — THE THIRTY-TWO WIDENED ARMS, AND THE ONE THAT STAYS REFUSING
// =============================================================================================

/// **THE GATE THE WIDENING NEEDED.** Twenty-five of the thirty-two sites take a `_ =>` fallback for
/// any variant they do not name, so before this step every one would have answered a rung-74
/// trajectory with a default, an empty set, or a panic — and nothing would have failed to compile.
///
/// The `false`-in-a-filter sites are checked by requiring a NON-EMPTY result, which is the only
/// assertion shape that tells *widened* from *silently dropped*.
#[test]
fn the_widened_readers_answer_on_a_rung_74_trajectory() {
    let traj = march(&marching(LAG_COORD_DEMAND));

    // `v_at_point` / `ic_at_point` — rung 68's panic-fallback pair.
    assert!(turbojet::three_loop::v_at_point(&traj[0]).is_finite());
    let (its, res, order) = turbojet::three_loop::ic_at_point(&traj[0]);
    assert_eq!(order, IC_ORDER4_DECLARED);
    assert_eq!(its, 2);
    assert_eq!(res, 0.0);

    // `asym_extra` — rung 52's, and `valve_of` — rung 65's. **COMPARED AGAINST THE POINT's OWN
    // KEYS, NOT MERELY CHECKED FOR FINITENESS.** The first version of this gate asserted
    // `is_finite()`, and the mutation that replaces `asym_extra`'s rung-74 arm with a `(0.0, 0.0)`
    // fallback SURVIVED it: a zero is finite, and `required >= 0.0` holds for it too. The
    // reference here is the test's own `match` on the variant, which is a different code path
    // from the reader's — [[instrument-fed-by-what-it-certifies]] read forwards.
    for p in traj.iter() {
        let PointExtra::Demand { g, required, b, b_cmd, .. } = p.extra else { panic!() };
        assert_eq!(turbojet::fuel_transient::asym_extra(p), (g, required),
                   "rung 52's reader returns THIS point's `g` and `required`, at s = {}", p.s);
        assert_eq!(turbojet::lagged_bleed::valve_of(p), (b, b_cmd),
                   "and rung 65's returns its valve pair");
    }
    // `g` and `required` are NOT both zero on this trajectory, which is what makes the row above
    // a measurement rather than a comparison of two nothings — slice T's class, checked forwards.
    let PointExtra::Demand { g, required, b, .. } = traj[NPTS - 1].extra else { panic!() };
    assert!(g != 0.0 && required != 0.0 && b != 0.0);

    // **`v_at_point` IS NOT VALUE-DISCRIMINATING ON THIS ARM AND THAT IS BOOKED.** Its un-widened
    // form falls back to `0.0`, and the stator sits within `1.8e-15` of its design setting for
    // the whole march, so no assertion on `v` can separate the two. The site is covered
    // structurally (it is in the widened set) and by the crate's other trajectories, not here.
    for p in traj.iter() {
        let PointExtra::Demand { v, .. } = p.extra else { panic!() };
        assert_eq!(turbojet::three_loop::v_at_point(p), v);
    }

    // `authority_of` — rung 72's, whose un-widened `_ => None` is the QUIETEST fallback of all:
    // no panic, no wrong number, just a census that finds nothing.
    assert_eq!(turbojet::shared_actuator::authority_of(&traj[NPTS - 1]), Some(Authority::Fuel));

    // `riding` — THE `false`-IN-A-FILTER SITE, and the reason this file's rig is the `PHI = 0.80`
    // arm: the valve is off its stop there, so the set is non-empty and the bar can be
    // non-emptiness rather than a property of members that do not exist.
    let rid = turbojet::three_loop::riding(&traj, B);
    assert_eq!(rid.len(), 66,
               "Python's own count on this arm — an EMPTY set here is exactly the silent failure \
                the widening exists to prevent");
    for p in &rid {
        let PointExtra::Demand { v_regime, .. } = p.extra else { panic!() };
        assert_eq!(v_regime, Some(Regime::Riding));
    }
}

/// **AND `cross_extra` STILL REFUSES** — the one widening question answered NO, now for the third
/// consecutive slice. Rung 74's march iterates the joint sweep UNDAMPED, so it carries
/// `ic_iters`/`ic_res` and no `ic_damp`; admitting it would hand a rung-67 reader a damping factor
/// this integrator never computed.
#[test]
fn cross_extra_refuses_rung_74_for_rung_66s_reason() {
    let traj = march(&marching(LAG_COORD_DEMAND));
    let msg = message_of(|| { turbojet::cross_loop::cross_extra(&traj[0]); });
    assert!(msg.contains("no joint-IC record") || msg.contains("ic_damp"),
            "the refusal names the missing KEY rather than the variant: {msg:?}");
}

/// **A STATOR-LESS DEMAND MARCH RUNS, AND ITS `v_regime` IS `None`** — the arming that keeps
/// [`PointExtra::Demand`]'s field an `Option`, inherited from rung 72's own reason. It is also the
/// arming `slice_af_cells.rs` uses for every refusal gate, so this is where that file's machine is
/// shown to march.
#[test]
fn a_stator_less_demand_march_records_no_regime() {
    let bare = LeverArm { bleed_lim: Some(valve()), ..Default::default() };
    let m = demand_machine(&bare);
    m.fuel.inner.ref_law.set(REF_LAW_DEFAULT);
    m.fuel.inner.lag_coord.set(LAG_COORD_LATCHED);
    let traj = march(&m);
    assert_eq!(traj.len(), NPTS);
    for p in &traj {
        let PointExtra::Demand { v_regime, v, v_cmd, .. } = p.extra else { panic!() };
        assert_eq!(v_regime, None,
                   "no stator solve ran, so there is no regime — `Dormant` would be a label the \
                    integrator never produced");
        assert_eq!(v, 0.0);
        assert_eq!(v_cmd, 0.0);
    }
    assert!(turbojet::three_loop::riding(&traj, B).is_empty(),
            "and Python compares `None` against a string and gets `False` rather than raising");
}
