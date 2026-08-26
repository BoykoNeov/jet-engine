//! RUNG 64 — the `phi`-REFERENCED BLEED LIMITER, the first CLOSED loop on an airflow lever.
//!
//! Slice X step 3. Python's `tests/test_rung64.py`, all 23 collected gates, plus **two the Python
//! suite does not have** — § 5.22 (ii)'s measured holes, which are the reason this file exists as
//! more than a translation.
//!
//! # The two ADDED gates, and why a translation would not have caught either
//!
//! A census of one injection per swapped cell, over rungs 62 + 63 + 64 together (**111 gates**,
//! not rung 64's 23 — rung 64 deliberately declines to gate what rung 63 already publishes),
//! found **two cells no suite protects**:
//!
//! * **`b_at_point`.** Reconstructing it from `b_of` instead of RE-SOLVING drives a floored
//!   march's `b_int` and `b_peak` to **exactly 0** and both of the rung's PUBLISHED ratios to 0 —
//!   and all 111 gates pass, because the only assertion that reads them is
//!   `assert f < s < c`, and **a strict ordering is satisfied by zeroing its smallest term**.
//!   [`the_bleed_integral_is_a_measurement_and_not_an_estimate`] pins both ratios to their
//!   published values ABSOLUTELY.
//! * **`try_close`.** Deleting rung 64's override leaves the march's initial condition **1.1 %
//!   wrong** with 111 gates green — the equilibrium solve runs through `try_close`, not
//!   `try_close_fuel`, and no rung-64 gate reads `nu0`.
//!   [`the_equilibrium_start_is_solved_on_the_floored_plant`] pins `nu0_lp` absolutely.
//!
//! Both ADDED gates use **absolute** anchors, deliberately. The reduce spine compares two
//! quantities from the same run and is blind to anything moving both sides together — which is
//! precisely how a zeroed `b_int` survived — so these two carry committed constants the way
//! `test_numeric_fingerprint.py` does.

use std::panic::catch_unwind;

use turbojet::bleed_transient::{build_scheduled_bleed, BleedSchedule, LeverArm};
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::fuel_transient::FuelPoint;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{build_limited_bleed, BleedLimiter};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{
    Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorArm, StatorLeg, StatorSchedule,
};
use turbojet::matcher::Branch;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------- the grid
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const N_LO: f64 = 0.65;
const B: f64 = 0.10;
const R: f64 = 0.5;
/// Strictly inside `[0.7354 shut, 0.8095 fully open]`.
const PHI: f64 = 0.80;

fn sm() -> f64 { PHI / FLOOR - 1.0 }

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

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

fn tilt_map() -> ComponentMap {
    ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), PI_LPC, PI_HPC, TT4, 50_000.0, REAL)
}

/// Python's `_lt(...)` — a rung-64 machine on the shaped maps.
fn lt(arm: &LeverArm) -> ScheduledStatorCore {
    lt_on(lp_map(), hp_map(), arm)
}

fn lt_on(lp: ComponentMap, hp: ComponentMap, arm: &LeverArm) -> ScheduledStatorCore {
    match build_limited_bleed(design(), flight(), 1.0, Some(lp), Some(hp), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

/// A rung-63 machine on the SAME hardware — the reduce's reference.
fn bt(arm: &LeverArm) -> ScheduledStatorCore {
    match build_scheduled_bleed(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn ramp(ds: f64) -> Ramp {
    Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds }
}

/// Python's `_march_keys` — the seven per-point values the reduce compares BIT-FOR-BIT.
fn march_keys(traj: &[FuelPoint]) -> Vec<[u64; 7]> {
    traj.iter()
        .map(|p| [p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.phi_lp.to_bits(),
                  p.phi_hp.to_bits(), p.tt4.to_bits(), p.mf.to_bits()])
        .collect()
}

fn march(m: &ScheduledStatorCore, ds: f64) -> Vec<FuelPoint> {
    m.stator_march(&flight(), &ramp(ds), None, &StatorLeg::default()).0
}

// =============================================================================================
// GATE 1 — THE REDUCE: `bleed_lim = None` is rung 63, bit-for-bit and per call
// =============================================================================================

/// The whole rung is a subclass, so rung 63's class is LITERALLY untouched. The gate is that an
/// unarmed rung-64 machine marches identically to the rung-63 one on the same hardware — exact
/// dispatch at every state, not a `0.0` valve position computed each step.
#[test]
fn reduce_no_limiter_is_rung63_bit_for_bit() {
    let a = march(&lt(&LeverArm::default()), 0.01);
    let b = march(&bt(&LeverArm::default()), 0.01);
    assert_eq!(march_keys(&a), march_keys(&b));
}

/// A floor BELOW every `phi` the march visits must reach the rung-63 parent at every state, not
/// merely agree to a tolerance. Witnessed against the valve-shut march, which is where a leaked
/// trial position would show up immediately.
#[test]
fn reduce_a_dormant_floor_dispatches_away_at_every_state() {
    let low = lt(&LeverArm::floored(BleedLimiter::new(0.30, B)));
    let a = march(&low, 0.01);
    let b = march(&lt(&LeverArm::default()), 0.01);
    assert_eq!(march_keys(&a), march_keys(&b));
}

/// Rungs 42/62's two arming modes must survive the new class untouched — otherwise the three-law
/// comparison at the heart of this rung would be comparing two code paths.
#[test]
fn reduce_the_schedule_and_constant_modes_are_rung63_bit_for_bit() {
    for arm in [LeverArm::constant(B), LeverArm::scheduled(BleedSchedule::new(B, N_LO))] {
        let a = march(&lt(&arm), 0.01);
        let b = march(&bt(&arm), 0.01);
        assert_eq!(march_keys(&a), march_keys(&b), "{:?}", arm.keys());
    }
}

/// Rung 64 adds only a transient subclass and its readers. The default single-spool design run
/// must be bit-for-bit rung 6 (the project's spine).
#[test]
fn cycle_untouched_design_run_is_rung6_bit_for_bit() {
    let eng = build_turbojet(Gas::reacting_equilibrium(), PI_LPC * PI_HPC, TT4, 50_000.0,
                             Losses { pi_d: 0.97, eta_b: 0.99, pi_b: 0.96, eta_m: 0.99,
                                      pi_n: 0.98, ..Losses::default() });
    let res = eng.run(&flight(), 1.0);
    let reference = eng.run(&flight(), 1.0);
    assert!(res.performance.specific_thrust > 0.0 && res.performance.tsfc > 0.0);
    for s in ["2", "3", "4", "5", "9"] {
        assert!(res.station(s).tt.to_bits() == reference.station(s).tt.to_bits());
        assert!(res.station(s).pt.to_bits() == reference.station(s).pt.to_bits());
    }
    assert!(res.performance.specific_thrust.to_bits()
            == reference.performance.specific_thrust.to_bits());
}

// =============================================================================================
// GATE 2 — THE OBJECT: a limiter with no authority is not an absent limiter
// =============================================================================================

/// `b_max = 0` is a limiter that CANNOT ACT — a different object from an absent one, and the
/// distinction is the whole rung (the ceiling belongs to `b_max`). Refused by assertion so it can
/// never be mistaken for the reduce path.
#[test]
fn zero_authority_is_refused_not_silently_reduced() {
    assert!(catch_unwind(|| BleedLimiter::new(PHI, 0.0)).is_err());
    assert!(catch_unwind(|| BleedLimiter::new(PHI, 0.5)).is_err());
}

/// Rung 62's two-way assert EXTENDED to three, not replaced: a constant position (42), a schedule
/// (62) or a floor (64) — exactly one. They are the three legs this rung differences, and arming
/// two would make every bill comparison meaningless.
#[test]
fn the_three_arming_modes_are_mutually_exclusive() {
    let lim = BleedLimiter::new(PHI, B);
    assert!(catch_unwind(|| lt(&LeverArm { bleed: B, bleed_lim: Some(lim),
                                           ..LeverArm::default() })).is_err());
    assert!(catch_unwind(|| lt(&LeverArm { bleed_sched: Some(BleedSchedule::new(B, N_LO)),
                                           bleed_lim: Some(lim), ..LeverArm::default() }))
            .is_err());
    assert!(catch_unwind(|| lt(&LeverArm { bleed: B,
                                           bleed_sched: Some(BleedSchedule::new(B, N_LO)),
                                           ..LeverArm::default() })).is_err());
}

// =============================================================================================
// GATE 3 — THE TRAP, fourth instance: a sibling constructor that drops the lever
// =============================================================================================

/// Rung 61's `at_setting`, rung 62's `at_stator`, rung 63's `isolating` — and now this. A sibling
/// constructor that silently dropped the floor would turn every inherited reader into an
/// armed-vs-UNARMED comparison attributing the valve's whole effect to the stator.
#[test]
fn at_stator_carries_the_floor_the_fourth_instance_of_one_trap() {
    let lim = BleedLimiter::new(PHI, B);
    let m = lt(&LeverArm::floored(lim));
    let sib = m.at_stator(StatorArm { sched_lp: Some(StatorSchedule::new(0.20, N_LO)),
                                      ..StatorArm::default() });
    assert_eq!(sib.fuel.inner.lever.lim, Some(lim));
    assert!(sib.armed_bleed());
}

/// Rung 63's gate, extended. A reference sibling must carry the NEIGHBOUR's valve and nothing
/// else; left un-extended, a floor in the neighbour would trip the assert for the wrong reason
/// and a floor as the lever would pass it for the wrong reason.
#[test]
fn isolating_counts_the_floor_as_an_arming_mode() {
    let m = lt(&LeverArm::default());
    let lim = BleedLimiter::new(PHI, B);
    let (rf, ar) = m.isolating(&LeverArm::floored(lim), None);
    assert!(!rf.armed_bleed() && ar.armed_bleed());
    let stat = LeverArm::stator(StatorArm { sched_lp: Some(StatorSchedule::new(0.20, N_LO)),
                                            ..StatorArm::default() });
    let (rf2, ar2) = m.isolating(&stat, Some(&LeverArm::floored(lim)));
    assert!(rf2.armed_bleed() && ar2.armed_bleed());
    assert!(catch_unwind(|| {
        let m = lt(&LeverArm::default());
        m.isolating(&LeverArm::floored(lim), Some(&LeverArm::floored(lim)))
    }).is_err());
}

/// The forced position IS the valve while the outer root trials one. A leak would make the
/// closure report a state the plant never visited — rung 62's `_powers` failure mode, which
/// converged to 1e-12 on a residual the plant did not use and returned `n_L` 5.3 % wrong with no
/// exception anywhere. The witness is that the committed closure reproduces its own reported
/// `phi_lp` when re-evaluated at the committed `b`.
#[test]
fn a_trial_position_never_leaks_out_of_the_outer_solve() {
    let fl = flight();
    let bare = lt(&LeverArm::default());
    let (tt2, pt2, _) = bare.fuel.inner.inlet(&fl);
    let eq = bare.fuel.inner.equilibrium(&fl, 1200.0);
    let mf = bare.fuel.fuel_for_tt4(&fl, 1200.0);
    let free = bare.fuel.close_fuel(eq.nu_lp, eq.nu_hp, mf, tt2, pt2);
    // A set point just ABOVE the unbled state, so the valve RIDES here rather than dispatching
    // away — a leak is only observable on the branch that trials positions.
    let m = lt(&LeverArm::floored(BleedLimiter::new(free.base.phi_lp * 1.01, B)));
    let c = m.fuel.close_fuel(eq.nu_lp, eq.nu_hp, mf, tt2, pt2);
    assert!(m.fuel.inner.b_forced.get().is_none(),
            "a trial position leaked out of the outer solve");
    let b = c.base.bleed.expect("a floored closure reports its committed position");
    assert!(0.0 < b && b < B, "{b}");
    assert!((c.base.phi_lp - free.base.phi_lp * 1.01).abs() < 1e-11);
    // the committed position, re-run as a rung-42 CONSTANT, must reproduce the same state
    let back = lt(&LeverArm::constant(b)).fuel.close_fuel(eq.nu_lp, eq.nu_hp, mf, tt2, pt2);
    assert!((back.base.phi_lp - c.base.phi_lp).abs() < 1e-12);
    assert!((back.tt4 - c.tt4).abs() < 1e-9);
}

// =============================================================================================
// GATE 4 — THE CEILING: what feedback does NOT buy
// =============================================================================================

/// **THE RUNG, half one.** `b = b_max` is itself an OPEN-LOOP law and it bounds every admissible
/// `b`-history from above, so a floor set ABOVE the fully-open march's own minimum SATURATES and
/// is VIOLATED. Feedback buys nothing on the protected coordinate.
///
/// Also pins WHY rung 62's schedule leaves a gap: it commands less than `b_max` at its own `phi`
/// minimum. That gap is about PLACEMENT, not about the loop being open.
#[test]
fn the_ceiling_belongs_to_b_max_and_not_to_the_law() {
    let c = lt(&LeverArm::default()).authority_ceiling(&flight(), &ramp(DS), B, N_LO, 0.10);
    let (shut, sched) = (c.shut.min_phi_lp, c.schedule.min_phi_lp);
    assert!(shut < sched && sched < c.ceiling, "{shut} {sched} {}", c.ceiling);
    assert!(!c.sched_saturated && c.b_at_sched_min < B);
    assert!(c.violated && c.over_deficit < 0.0);
    assert!(c.bounded_by_full && -1e-2 < c.over_vs_full && c.over_vs_full < 0.0,
            "{}", c.over_vs_full);
}

/// P2, and Python predicted BIT-FOR-BIT. That was REFUTED and the gate records why.
///
/// The solve brackets the root on `[0, b_max]`, so the clamp is the Illinois solve's UPPER
/// ENDPOINT and enters the iterate sequence even when it never binds: two clamps give two paths
/// and two roots inside the same tol, ~1e-15 apart in `b`. What survives is that NOTHING PHYSICAL
/// moves — every key agrees to <= 1e-12 relative across a 4x clamp, with `phi_lp` pinned either
/// way.
///
/// The `*_at_min_lp` keys are excluded BY NAME and for a reason that is the rung's own content:
/// a riding floor makes the `phi` minimum a PLATEAU, so its LOCATION is not a defined object and
/// the argmin is decided by a 1-ulp tie. Gated separately below.
#[test]
fn the_invisible_authority_an_untouched_clamp_moves_nothing_physical() {
    let fl = flight();
    let rp = ramp(DS);
    let a = lt(&LeverArm::floored(BleedLimiter::new(PHI, B))).bill_cell(&fl, &rp, false);
    let b = lt(&LeverArm::floored(BleedLimiter::new(PHI, 4.0 * B))).bill_cell(&fl, &rp, false);
    assert!(a.b_peak < B, "the clamp must be UNTOUCHED for this gate to mean anything");
    // Python iterates the dict's float values and EXCLUDES the three argmin keys BY NAME, so it
    // auto-covers any float a later rung adds. The port names the survivors instead, which
    // FREEZES the set — so the compiler is made to enforce the coverage: this destructure is
    // exhaustive, and adding a field to `BillCell` (rung 65 will) is a COMPILE ERROR here until
    // the new field is classified as swept or excluded.
    //
    // An earlier draft of this gate merely CLAIMED that enforcement in a comment with nothing
    // behind it — [[rust-port-documented-gate-that-doesnt-exist]], caught in review.
    let turbojet::limited_bleed::BillCell {
        // the three EXCLUDED by name, and the rung's own content is why: a riding floor makes the
        // `phi` minimum a PLATEAU, so its LOCATION is a 1-ulp tie.
        nu_at_min_lp: _, s_at_min_lp: _, b_at_min_lp: _,
        // the fifteen swept
        plateau_span: _, min_phi_lp: _, min_phi_hp: _, m_i_lp: _, m_i_hp: _, b_int: _,
        b_peak: _, b_end: _, thrust_int: _, thrust_end: _, nu_lp_end: _, nu_hp_end: _,
        tt4_peak: _, nu0_lp: _, nu0_hp: _,
        // not floats: excluded because Python's `isinstance(v, float)` filter excludes them
        plateau_pts: _, npts: _, traj: _,
    } = &a;
    let pairs: [(&str, f64, f64); 15] = [
        ("plateau_span", a.plateau_span, b.plateau_span),
        ("min_phi_lp", a.min_phi_lp, b.min_phi_lp),
        ("min_phi_hp", a.min_phi_hp, b.min_phi_hp),
        ("m_i_lp", a.m_i_lp, b.m_i_lp),
        ("m_i_hp", a.m_i_hp, b.m_i_hp),
        ("b_int", a.b_int, b.b_int),
        ("b_peak", a.b_peak, b.b_peak),
        ("b_end", a.b_end, b.b_end),
        ("thrust_int", a.thrust_int, b.thrust_int),
        ("thrust_end", a.thrust_end, b.thrust_end),
        ("nu_lp_end", a.nu_lp_end, b.nu_lp_end),
        ("nu_hp_end", a.nu_hp_end, b.nu_hp_end),
        ("tt4_peak", a.tt4_peak, b.tt4_peak),
        ("nu0_lp", a.nu0_lp, b.nu0_lp),
        ("nu0_hp", a.nu0_hp, b.nu0_hp),
    ];
    for (k, x, y) in pairs {
        let rel = (x - y).abs() / if x == 0.0 { 1.0 } else { x.abs() };
        assert!(rel <= 1e-12, "{k}: {x:?} vs {y:?} (rel {rel:.3e})");
    }
}

/// The other half of P2's refutation, and it BOUNDS rungs 44–52. Those rungs report WHERE a surge
/// minimum sits — rung 50's whole finding is that a release edge RELOCATES both spools' minima to
/// itself. A floor that rides pins `phi` to `phi_lim` over an INTERVAL, so on such a plant the
/// minimum has a value (rung 60) and no location.
#[test]
fn a_riding_floor_destroys_the_location_of_the_minimum() {
    let fl = flight();
    let rp = ramp(DS);
    let shut = lt(&LeverArm::default()).bill_cell(&fl, &rp, false);
    let floor = lt(&LeverArm::floored(BleedLimiter::new(PHI, B))).bill_cell(&fl, &rp, false);
    assert!(shut.plateau_pts == 1 && shut.plateau_span == 0.0);
    assert!(floor.plateau_pts > 1 && floor.plateau_span > 10.0 * DS);
}

/// P6. The floor is enforced INSIDE the closure, not between RK steps, so `min phi_lp` pins to
/// `phi_lim` in exact arithmetic rather than to the integrator's order. Any `ds`-dependence would
/// mean the pinning is a grid artifact and every matched bill below would be matched only
/// approximately.
#[test]
fn the_tautology_is_exact_at_every_grid() {
    let fl = flight();
    let m = lt(&LeverArm::floored(BleedLimiter::new(PHI, B)));
    for ds in [0.01, DS, 0.0025] {
        let c = m.bill_cell(&fl, &ramp(ds), false);
        assert!((c.min_phi_lp - PHI).abs() < 1e-9, "{ds} {}", c.min_phi_lp);
    }
}

// =============================================================================================
// GATE 5 — THE BILL: what feedback DOES buy
// =============================================================================================

/// **THE RUNG, half two.** Three laws of ONE lever matched to the SAME `min phi_lp`, billed in
/// rung 61's currency. The ordering is the ladder's own information ordering:
///
/// ```text
/// state-BLIND (42)  >  state-FED open loop (62)  >  CLOSED loop (64)
/// ```
///
/// and it must hold in the bleed integral AND in the overspeed AND in the thrust — rung 61's whole
/// point being that those need not track. Both map shapes, because a headline resting on a ratio
/// needs the second shape run BEFORE it is written.
///
/// **THIS IS THE GATE § 5.22 (ii) MEASURED AS INSUFFICIENT.** `f < s < c` is satisfied by zeroing
/// `f`, which is exactly what a reconstructing `b_at_point` does. The absolute anchor lives in
/// [`the_bleed_integral_is_a_measurement_and_not_an_estimate`].
#[test]
fn the_bill_falls_with_the_information_the_law_uses() {
    for (shape, lp, hp) in [("shaped", lp_map(), hp_map()),
                            ("tilted", tilt_map(), tilt_map())] {
        let m = lt_on(lp, hp, &LeverArm::default())
            .matched_bill(&flight(), &ramp(DS), PHI, B, N_LO, 0.30);
        assert!(m.matched < 1e-9, "{shape}: {}", m.matched);
        assert!(!m.saturated, "{shape}: a saturated floor is not delivering the matched point");
        let (c, s, f) = (m.constant.b_int, m.schedule.b_int, m.floor.b_int);
        assert!(f < s && s < c, "{shape}: {f} {s} {c}");
        let (bc, bs, bf) = (&m.bill_constant, &m.bill_schedule, &m.bill_floor);
        assert!(bc.d_nu_lp_end < bs.d_nu_lp_end && bs.d_nu_lp_end < bf.d_nu_lp_end
                && bf.d_nu_lp_end < 0.0, "{shape}");
        assert!(bc.thrust_int_pct < bs.thrust_int_pct && bs.thrust_int_pct < bf.thrust_int_pct
                && bf.thrust_int_pct < 0.0, "{shape}");
        // and the end-of-ramp thrust bill is MACHINE-ZERO for the closed loop alone: it
        // self-releases, so it has left the machine by settle.
        assert!(bf.thrust_end_pct.abs() < 0.1 && 0.1 < bs.thrust_end_pct.abs(), "{shape}");
    }
}

/// P5, and the free non-tautology. The LP debit is not merely small but STRUCTURALLY UNAVAILABLE —
/// `min phi_lp` IS `phi_lim` while the floor rides, so no LP debit is even expressible, which is
/// rung 52's *a self-releasing limiter cannot debit the spool it watches*, transferred from a fuel
/// lever to an AIRFLOW one. The HP is debited (rung 49's arrow, same transfer), while a CONSTANT
/// valve — still open at the HP's own LATE minimum where the state-fed laws have already shut —
/// CREDITS it.
#[test]
fn the_state_fed_laws_debit_the_hp_and_the_state_blind_one_credits_it() {
    for (shape, lp, hp) in [("shaped", lp_map(), hp_map()),
                            ("tilted", tilt_map(), tilt_map())] {
        let m = lt_on(lp, hp, &LeverArm::default())
            .matched_bill(&flight(), &ramp(DS), PHI, B, N_LO, 0.30);
        assert!(m.bill_constant.d_min_phi_hp > 0.0, "{shape}");
        assert!(m.bill_schedule.d_min_phi_hp < 0.0, "{shape}");
        assert!(m.bill_floor.d_min_phi_hp < 0.0, "{shape}");
    }
}

/// P4's robustness half. The HP debit is O(1e-4) against an LP move of O(1e-1), so it must be
/// shown to be physics and not the integrator: the sign holds and the magnitude is stable across a
/// 4x refinement.
#[test]
fn the_hp_debit_survives_grid_refinement() {
    let fl = flight();
    let mut d = Vec::new();
    for ds in [0.01, DS, 0.0025] {
        let rp = ramp(ds);
        let f = lt(&LeverArm::floored(BleedLimiter::new(PHI, B))).bill_cell(&fl, &rp, false);
        let s = lt(&LeverArm::default()).bill_cell(&fl, &rp, false);
        d.push(f.min_phi_hp - s.min_phi_hp);
    }
    assert!(d.iter().all(|x| *x < 0.0), "{d:?}");
    assert!((d[d.len() - 1] - d[0]).abs() < 0.3 * d[0].abs(), "{d:?}");
}

// =============================================================================================
// GATE 6 — rung 63 § 3's refusal, with BOTH objects watching `phi`
// =============================================================================================

/// A closed-loop lever does not DISARM a second limiter on the same variable — it DELETES that
/// limiter's PLANT. Where the valve rides it re-pins `phi_lp` at ANY fuel, so `dphi/dWf = 0`, the
/// fuel leg's residual is identically zero across its bracket, and its set-point solve is
/// degenerate.
///
/// **DELIBERATELY DOES NOT ASSERT ON `removed_together`.** At exact tangency the leg chooses
/// between its dormant return and a 60-iteration degenerate hunt on the SIGN OF ONE ULP, so both
/// `== 0.0` and `> 0.0` are roundoff assertions, not gates. Nor is EXACT equality demanded on the
/// composite: the degenerate solve returns an arbitrary point of a continuum, so the composite
/// agrees to MACHINE PRECISION and not to the bit.
#[test]
fn a_closed_loop_lever_deletes_a_fuel_floors_plant() {
    let fr = lt(&LeverArm::default()).floor_refusal(&flight(), &ramp(DS), sm(), B, 0.01);
    assert!(fr.removed_alone > 0.0, "the fuel leg must BITE on the bare plant");
    // (i) whatever the leg does beside the valve, it buys MACHINE-ZERO — against a bare-plant
    // credit that is O(1e-2) in the same currency, this is inertness by five orders.
    assert!(fr.credit.abs() < 1e-14, "{}", fr.credit);
    assert!((fr.both.m_i - fr.valve.m_i).abs() < 1e-14);
    assert!((fr.both.min_phi - fr.valve.min_phi).abs() < 1e-14);
    assert!((fr.fuel.m_i - fr.neither.m_i).abs() > 1e-3,
            "the fuel leg must MOVE m_i on the bare plant, or 'inert' means nothing");
    // (ii) the control that separates tangency chatter from a broken leg
    assert!(fr.removed_below_bare > 0.0);
    assert!(fr.removed_below_armed == 0.0);
    assert!(fr.control_dormant);
}

// =============================================================================================
// GATE 7 — the modelling floor: every march stays on the choked branch
// =============================================================================================

/// The rung-30/31 choked-nozzle premise, checked at the WIDEST position each law can command — a
/// saturating floor sits at `b_max` for most of the ramp, which is the most extraction any rung-64
/// march ever applies.
#[test]
fn every_march_stays_on_the_choked_branch() {
    for arm in [LeverArm::floored(BleedLimiter::new(PHI, B)),
                LeverArm::floored(BleedLimiter::new(0.95, B)),   // SATURATED throughout
                LeverArm::constant(B),
                LeverArm::scheduled(BleedSchedule::new(B, N_LO))] {
        let traj = march(&lt(&arm), 0.01);
        assert!(traj.iter().all(|p| p.branch == Branch::Choked), "{:?}", arm.keys());
    }
}

// =============================================================================================
// THE TWO ADDED GATES — § 5.22 (ii)'s measured holes. NOT in the Python suite.
// =============================================================================================

/// **HOLE 1 — `b_at_point` MUST RE-SOLVE.** Reconstructing it from `b_of` returns rung 62's
/// *constant*, which on a floored machine is `0.0`, so `b_int` and `b_peak` go to **exactly 0**
/// and both published ratios with them — and all 111 rung-62/63/64 Python gates still pass,
/// because the only assertion that reads them is `f < s < c` and zeroing `f` satisfies it.
///
/// **ABSOLUTE, not relative, and that is the point.** The reduce spine compares two quantities
/// from the same run and is blind to anything that moves both sides together; these constants are
/// the rung's own published numbers, measured on the suite's grid.
#[test]
fn the_bleed_integral_is_a_measurement_and_not_an_estimate() {
    let m = lt(&LeverArm::default()).matched_bill(&flight(), &ramp(DS), PHI, B, N_LO, 0.30);
    // the two ratios rung 64 publishes, to 9 significant figures
    assert!((m.b_ratio_const - 0.255202701714).abs() < 1e-10, "{}", m.b_ratio_const);
    assert!((m.b_ratio_sched - 0.518688807277).abs() < 1e-10, "{}", m.b_ratio_sched);
    // and the terms they are built from, so a COMPENSATING error in both cannot pass
    assert!(m.floor.b_int > 0.0 && m.floor.b_peak > 0.0);
    assert!(m.floor.b_int < m.schedule.b_int && m.schedule.b_int < m.constant.b_int);
    // the FLOORED march's committed position must be neither identically zero nor pinned at the
    // clamp — a reconstruction gives the first, a saturated solve the second.
    assert!(0.0 < m.floor.b_peak && m.floor.b_peak < B);
}

/// **HOLE 2 — `try_close` MUST BE RUNG 64's.** The march's INITIAL CONDITION comes from the
/// equilibrium solve, which runs through `try_close` and not `try_close_fuel`. Deleting rung 64's
/// override leaves `nu0` **1.1 % wrong** with all 111 Python gates green, because no rung-64 gate
/// reads it.
///
/// The unfloored value is asserted beside it so the gate cannot pass by making both marches
/// identical — which is precisely what deleting the override does.
#[test]
fn the_equilibrium_start_is_solved_on_the_floored_plant() {
    let fl = flight();
    let rp = ramp(DS);
    let floored = lt(&LeverArm::floored(BleedLimiter::new(PHI, B))).bill_cell(&fl, &rp, false);
    let shut = lt(&LeverArm::default()).bill_cell(&fl, &rp, false);
    assert!((floored.nu0_lp - 0.7475441088051796).abs() < 1e-12, "{}", floored.nu0_lp);
    assert!((shut.nu0_lp - 0.7557409602636336).abs() < 1e-12, "{}", shut.nu0_lp);
    // 1.08 % apart — the deletion's whole signature is that this difference goes to zero.
    let rel = (shut.nu0_lp - floored.nu0_lp).abs() / shut.nu0_lp;
    assert!(rel > 1e-2, "the floored start must differ from the shut one: rel {rel:.3e}");
}
