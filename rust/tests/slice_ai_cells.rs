//! SLICE AI step 1 — **SIX RE-AIMED POINTERS ACROSS TWO RUNGS, ZERO NEW TABLE FIELDS, AND THE
//! SETTER RE-AIM THAT CARRIES A LATENT PYTHON DEFECT ON PURPOSE.**
//!
//! # WHY THIS FILE IS SHAPED THE WAY IT IS
//!
//! Plan § 5.33 (ii)'s census is `0 ADD`, the sixth slice running, so **a forgotten re-aim
//! compiles**: no `E0063` width tripwire exists anywhere in this slice. The instrument that works
//! under that condition is FUNCTION-POINTER IDENTITY IN BOTH DIRECTIONS —
//! [`the_slice_re_aims_six_pointers_and_adds_none`].
//!
//! # AND FOR ONE OF THE SIX, POINTER IDENTITY IS NOT ENOUGH — A FIELD READBACK IS
//!
//! `with_coord`'s re-aim is § 5.33 (i)'s mechanism: on a rung-79 machine the SAME guard
//! ([`CoordScope`]) moves a DIFFERENT field. The pre-flight measured it VALUE-INVISIBLE over nine
//! walls (0 of 196 keys move while rung 79's branch runs 128 times), so no value gate anywhere can
//! see whether this cell is aimed right. [`coord_scope_moves_lag_coord_at_78_and_phi_ref_at_79`]
//! reads the two fields back inside and after the scope, and
//! [`a_dropped_with_coord_re_aim_is_caught_by_the_readback_too`] BUILDS the defect — a rung-79
//! table with the setter left at rung 74's body — and shows the readback sees it. That is the
//! demonstration slice AH step 1 could not make for its own gate; `with_ref_tables` is public, so
//! a test can choose the table.
//!
//! **If the Python defect is ever repaired** (for instance by renaming rung 79's setter), the re-aim
//! goes away, and the inside-the-scope pair on a rung-79 machine becomes `("demand", "phi")`. That
//! flips exactly ONE assertion here, which is written as a tuple so that it stays one.
//!
//! # THE COUNTERS ARE PER-THREAD, AND THIS FILE MEASURES THAT RATHER THAN ASSERTING IT
//!
//! [`the_counters_are_per_thread_and_move_on_the_real_plant`] bumps them through the REAL
//! incidence plant on the test's own thread and reads zero from a spawned one. Every counter gate
//! here RESETS FIRST: the harness reuses threads, so per-thread state from an earlier test can
//! still be sitting there.
//!
//! # WHAT THIS FILE DELIBERATELY DOES NOT GATE
//!
//! The readers (steps 2–4), the refusal gates and the ulp band (step 5), and bit-equality against
//! Python (the oracle, step 6). **The rung-79 refusal is exercised here only as a control** — that
//! the incidence branch refuses a non-identity gauge and the `phi` branch does not — with a needle
//! unique in `engine.py`.
//!
//! **NOTHING HERE READS A GOLDEN.** Every assertion is a pointer comparison, a field readback, a
//! same-run difference, or a compile-time property.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::fn_addr_eq;

use turbojet::bleed_transient::{LeverArm, LeverArming};
use turbojet::demand_coordinate::{CoordScope, LAG_COORD_CLIP, LAG_COORD_DEMAND};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{Floor, SurgeLimiter};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::residual_gauge::{
    build_residual_gauge_cascade, R78, R78_FUEL, R78_STATOR, R78_TRIPLE, R78_TWO,
};
use turbojet::shared_actuator::SharedRigArm;
use turbojet::split_wall::{
    build_split_wall_cascade, walls_of, R80, R80_FUEL, R80_STATOR, R80_TRIPLE, R80_TWO,
};
use turbojet::state_coordinate::{
    build_state_coordinate_cascade, coord_counters, coord_probe_armed, reset_coord_counters,
    CoordCounters, PHI_REF_INCIDENCE, PHI_REF_PHI, R79, R79_FUEL, R79_STATOR, R79_TRIPLE, R79_TWO,
};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::TripleHooks;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------- the grid
//
// `tests/test_rung79.py` / `tests/test_rung80.py`'s module constants, which are rung 76's
// unchanged. This slice adds no number of its own: `_phi_ref` is a label and `sm_air` is swept.

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const PHI: f64 = 0.80;
const SM: f64 = PHI / FLOOR - 1.0;
const TAU: f64 = 0.05;
const B: f64 = 0.10;

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

fn valve_arm() -> LeverArm { LeverArm { bleed_lim: Some(valve()), ..Default::default() } }

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

/// Python's `ResidualGaugeTransient(design, …)` — rung 78, the parent and the control.
fn gauge(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_residual_gauge_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

/// Python's `StateCoordinateTransient(design, …)` — rung 79.
fn coord(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_state_coordinate_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

/// Python's `SplitWallTransient(design, …)` — rung 80.
fn split(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_split_wall_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

/// The fuel leg's floor at the suite's margin — rung 49's, off the map's own surge line.
fn floor() -> Floor { Floor::Phi(SurgeLimiter::from_margin(&lp_map(), Spool::Lp, SM)) }

/// Two frozen states for the plant gates, on a machine with NO VALVE — measured, not chosen.
///
/// The first draft of this file used `slice_af_cells.rs`'s `(1, 1, 0.02)` on the valve rig and
/// three gates aborted inside RUNG 74's cap: `phi` there never falls to the floor. And on the valve
/// rig it cannot anywhere, because the valve is armed at the SAME `0.80` wall and holds `phi` at
/// exactly `0.8000` inside every instant solve — `G = phi_lim − phi` is pinned at zero from above
/// and the bracket never finds `G > 0`. That is rung 74's own measured fact (the airflow levers
/// act only inside the fuel leg's tracking error), arriving as a test-point defect. Probed with the
/// valve off, at `(0.8, 0.9)`: `phi(0.02) = 0.8600`, `phi(0.03) = 0.7980`, `phi(0.035) = 0.7705`.
///
/// * [`SLACK`] — `phi(mf_sched) > phi_lim`, so the leg is slack and `cap_free` BRACKETS the
///   residual: the coordinate is live and the same root may come back as different floats.
/// * [`BINDING`] — `phi(mf_sched) < phi_lim`, so `G(mf_sched) > 0` and `cap_free` short-circuits to
///   the shipped `_surge_fuel`, which brackets its own hardcoded `phi` residual: the coordinate is
///   UNREACHABLE, and the two coordinates must agree to the bit.
const SLACK: (f64, f64, f64) = (0.8, 0.9, 0.02);
const BINDING: (f64, f64, f64) = (0.8, 0.9, 0.035);

/// The rung-79 cap cell at a frozen state, phi leg only (no accel leg, so `a_cap = +inf`).
fn cap_at(m: &ScheduledStatorCore, s: (f64, f64, f64)) -> Result<f64, turbojet::gas::Abort> {
    (m.fuel.inner.triple_hooks.cap_fuel)(
        &m.fuel, &flight(), s.0, s.1, s.2, None, Some(&floor()), None)
}

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

// ---------------------------------------------------------------------------------------------
// 1 — THE POINTERS
// ---------------------------------------------------------------------------------------------

/// **THE STEP's INSTRUMENT** — six swaps that must differ, and every other cell that must not.
#[test]
fn the_slice_re_aims_six_pointers_and_adds_none() {
    // RUNG 79's FOUR, against rung 78.
    assert!(!fn_addr_eq(R79.at_lever, R78.at_lever),
            "swap 1 of 6 — `at_lever`, SEVENTEENTH instance; the eighth knob is `_phi_ref`");
    assert!(!fn_addr_eq(R79_TRIPLE.shared_rig, R78_TRIPLE.shared_rig),
            "swap 2 of 6 — `_shared_rig`, the coordinate carried onto the rig");
    assert!(!fn_addr_eq(R79_TRIPLE.cap_fuel, R78_TRIPLE.cap_fuel),
            "swap 3 of 6 — `_cap_fuel`, the incidence branch and the rung's one refusal");
    assert!(!fn_addr_eq(R79_TRIPLE.with_coord, R78_TRIPLE.with_coord),
            "swap 4 of 6 — `_with_coord`, the NAME REUSE the cell was built in advance for");

    // RUNG 80's TWO, against rung 79.
    assert!(!fn_addr_eq(R80.at_lever, R79.at_lever),
            "swap 5 of 6 — `at_lever`, EIGHTEENTH and last; the ninth knob is `_sm_air`");
    assert!(!fn_addr_eq(R80_TRIPLE.shared_rig, R79_TRIPLE.shared_rig),
            "swap 6 of 6 — `_shared_rig`, the split applied on the machine rung 79 returns");

    // THE TWO INHERITANCES THAT CARRY § 5.33 (i) ONTO RUNG 80, NAMED rather than left to the sweep.
    assert!(fn_addr_eq(R80_TRIPLE.with_coord, R79_TRIPLE.with_coord),
            "rung 80 inherits RUNG 79's setter — so `demand_gains` on a rung-80 machine writes \
             `phi_ref` too, exactly as the pre-flight measured");
    assert!(fn_addr_eq(R80_TRIPLE.cap_fuel, R79_TRIPLE.cap_fuel),
            "and rung 79's plant — the incidence branch that setter's `\"demand\"` reaches");

    // AND THE CROSS CHECK: rung 79 re-aims `cap_fuel` and leaves `sensed_cap` at rung 76's body,
    // which rung 78's accel branch — reached through rung 79's `super()` call — still consults.
    assert!(fn_addr_eq(R79_TRIPLE.sensed_cap, R78_TRIPLE.sensed_cap));

    for (a, b, name) in triple_pairs(&R79_TRIPLE, &R78_TRIPLE) {
        if matches!(name, "shared_rig" | "cap_fuel" | "with_coord") {
            assert_ne!(a, b, "rung 79 RE-AIMS `{name}`");
        } else {
            assert_eq!(a, b, "rung 79 INHERITS `{name}`");
        }
    }
    for (a, b, name) in triple_pairs(&R80_TRIPLE, &R79_TRIPLE) {
        if name == "shared_rig" {
            assert_ne!(a, b, "rung 80 RE-AIMS `{name}`");
        } else {
            assert_eq!(a, b, "rung 80 INHERITS `{name}`");
        }
    }

    // THE ALIAS TABLES CARRY NO SWAP ON EITHER RUNG.
    assert!(fn_addr_eq(R79_TWO.try_close, R78_TWO.try_close));
    assert!(fn_addr_eq(R80_TWO.try_close, R79_TWO.try_close));
    assert!(fn_addr_eq(R79_STATOR.stator_march, R78_STATOR.stator_march));
    assert!(fn_addr_eq(R80_STATOR.stator_march, R79_STATOR.stator_march));
    assert!(fn_addr_eq(R79_FUEL.integrate_fuel, R78_FUEL.integrate_fuel),
            "rung 79's one refusal lives in the CAP, not in the march");
    assert!(fn_addr_eq(R80_FUEL.integrate_fuel, R79_FUEL.integrate_fuel));
    assert!(fn_addr_eq(R79.b_at_point, R78.b_at_point));
    assert!(fn_addr_eq(R80.b_at_point, R79.b_at_point));
    assert!(fn_addr_eq(R79.legs, R78.legs));
    assert!(fn_addr_eq(R80.legs, R79.legs));
}

/// **`TripleHooks` IS STILL EIGHTEEN AFTER THIS SLICE** — an exhaustive destructuring, so a
/// nineteenth field is a compile error here. P6.
#[test]
fn triple_hooks_is_still_eighteen_fields() {
    let TripleHooks {
        stator_leg: _, lagged_stator: _, clamp_v: _, check_v0: _, rk4_floor: _, solve_v: _,
        manifold_v: _, triple_laws: _, triple_rig: _, with_ref: _, reference: _, quad_gains_at: _,
        rk4_floor_shared: _, cap_fuel: _, windup_tau: _, with_coord: _, sensed_cap: _,
        shared_rig: _,
    } = R80_TRIPLE;
}

// ---------------------------------------------------------------------------------------------
// 2 — THE CARRIERS
// ---------------------------------------------------------------------------------------------

/// **THE TWO NEW KNOBS DEFAULT TO THEIR REDUCE ARMS AND ARE CARRIED, NOT RE-DEFAULTED.**
#[test]
fn at_lever_carries_phi_ref_and_sm_air_and_the_defaults_are_the_reduce_arms() {
    let arm = valve_arm();

    // Defaults, on the core's own constructor — Python's class attributes.
    let c = coord(&arm);
    assert_eq!(c.fuel.inner.phi_ref.get(), PHI_REF_PHI);
    assert_eq!(c.fuel.inner.sm_air.get(), None);
    assert_eq!(gauge(&arm).fuel.inner.phi_ref.get(), PHI_REF_PHI,
               "the carrier is on the SHARED core, so a rung-78 machine has it at the default");

    // Rung 79 carries the coordinate onto its sibling, and builds that sibling on ITS table.
    c.fuel.inner.phi_ref.set(PHI_REF_INCIDENCE);
    let sib = c.at_lever(&arm);
    assert_eq!(sib.fuel.inner.phi_ref.get(), PHI_REF_INCIDENCE,
               "`at_lever` must carry `_phi_ref` — the SEVENTEENTH instance of the trap");
    assert!(fn_addr_eq(sib.fuel.inner.triple_hooks.cap_fuel, R79_TRIPLE.cap_fuel));
    assert!(fn_addr_eq(sib.fuel.inner.lever_hooks.at_lever, R79.at_lever));

    // Rung 80 carries both, and builds on its own table.
    let s = split(&arm);
    s.fuel.inner.phi_ref.set(PHI_REF_INCIDENCE);
    s.fuel.inner.sm_air.set(Some(0.5));
    let sib = s.at_lever(&arm);
    assert_eq!(sib.fuel.inner.phi_ref.get(), PHI_REF_INCIDENCE);
    assert_eq!(sib.fuel.inner.sm_air.get(), Some(0.5),
               "`at_lever` must carry `_sm_air` — the EIGHTEENTH instance, and the last");
    assert!(fn_addr_eq(sib.fuel.inner.triple_hooks.shared_rig, R80_TRIPLE.shared_rig));
    assert!(fn_addr_eq(sib.fuel.inner.lever_hooks.at_lever, R80.at_lever));

    // And the rigs carry them too — through the re-aimed `_shared_rig`, on top of `at_lever`.
    let rig = SharedRigArm { sm: SM, tt4_max: 1200.0, ..Default::default() };
    s.fuel.inner.sm_air.set(Some(SM));
    let (m, _, _) = (s.triple_hooks().shared_rig)(&s, &rig);
    assert_eq!(m.fuel.inner.phi_ref.get(), PHI_REF_INCIDENCE);
    assert_eq!(m.fuel.inner.sm_air.get(), Some(SM));
}

/// **RUNG 80's REDUCE, AT THE RIG: `sm_air = None` IS RUNG 79, AND `sm_air = sm` AGREES WITH IT TO
/// THE LAST BIT** — Python's docstring: it rebuilds the same floors from the same factory, so any
/// difference is a bug in the rebuild and not a finding. And a real split moves the airflow walls
/// ABOVE the fuel leg's while leaving the fuel leg's alone.
#[test]
fn the_split_rig_reduces_to_rung_79_and_moves_only_the_airflow_walls() {
    let arm = valve_arm();
    let rig = SharedRigArm { sm: SM, tt4_max: 1200.0, ..Default::default() };

    let c = coord(&arm);
    let (m79, s79, _) = (c.triple_hooks().shared_rig)(&c, &rig);
    let w79 = walls_of(&m79, s79.as_ref());

    let s = split(&arm);
    let (m_none, s_none, _) = (s.triple_hooks().shared_rig)(&s, &rig);
    assert_eq!(walls_of(&m_none, s_none.as_ref()), w79, "`sm_air = None` is rung 79's rig");

    s.fuel.inner.sm_air.set(Some(SM));
    let (m_eq, s_eq, _) = (s.triple_hooks().shared_rig)(&s, &rig);
    let w_eq = walls_of(&m_eq, s_eq.as_ref());
    for (x, y) in [(w_eq.phi_lim, w79.phi_lim), (w_eq.phi_air, w79.phi_air),
                   (w_eq.phi_valve, w79.phi_valve), (w_eq.phi_stator, w79.phi_stator)] {
        assert_eq!(x.map(f64::to_bits), y.map(f64::to_bits),
                   "`sm_air == sm` must rebuild the SAME floors to the last bit");
    }
    assert!(w79.phi_air.is_some() && w79.phi_stator.is_some(),
            "the rig arms the valve and the stator, so the comparison above is not over `None`s");

    s.fuel.inner.sm_air.set(Some(SM + 0.1));
    let (m_up, s_up, _) = (s.triple_hooks().shared_rig)(&s, &rig);
    let w_up = walls_of(&m_up, s_up.as_ref());
    assert_eq!(w_up.phi_lim, w79.phi_lim, "the FUEL leg's wall does not move");
    assert!(w_up.phi_air.unwrap() > w_up.phi_lim.unwrap(), "the airflow wall sits above it");
    assert_eq!(w_up.phi_valve, w_up.phi_air, "the valve reads the airflow wall");
    // The hardware is kept; only the floor moved.
    let (old, new) = (m79.fuel.inner.lever.lim.unwrap(), m_up.fuel.inner.lever.lim.unwrap());
    assert_eq!((new.b_max, new.tau), (old.b_max, old.tau));
}

// ---------------------------------------------------------------------------------------------
// 3 — THE SETTER RE-AIM, BY FIELD READBACK
// ---------------------------------------------------------------------------------------------

/// **§ 5.33 (i)'s MECHANISM IN ITS SMALLEST FORM.** `demand_gains` pins `lag_coord = "clip"` and
/// then enters `CoordScope::set(…, "demand")`. On a rung-78 machine that moves `lag_coord`; on a
/// rung-79 or rung-80 machine the SAME guard moves `phi_ref` and leaves `lag_coord` at `"clip"`.
///
/// The inside-the-scope pair on the rung-79/80 machines is the defect, reproduced on purpose, and
/// it is asserted as ONE tuple so that a future Python repair flips exactly one assertion.
#[test]
fn coord_scope_moves_lag_coord_at_78_and_phi_ref_at_79() {
    let arm = valve_arm();
    let pair = |m: &ScheduledStatorCore| (m.fuel.inner.lag_coord.get(), m.fuel.inner.phi_ref.get());

    let g = gauge(&arm);
    g.fuel.inner.lag_coord.set(LAG_COORD_CLIP);
    {
        let _cs = CoordScope::set(&g.fuel.inner, LAG_COORD_DEMAND);
        assert_eq!(pair(&g), (LAG_COORD_DEMAND, PHI_REF_PHI),
                   "rung 78: the scope moves `lag_coord`, as rung 74 meant");
    }
    assert_eq!(pair(&g), (LAG_COORD_CLIP, PHI_REF_PHI));

    for (m, rung) in [(coord(&arm), 79), (split(&arm), 80)] {
        m.fuel.inner.lag_coord.set(LAG_COORD_CLIP);
        {
            let cs = CoordScope::set(&m.fuel.inner, LAG_COORD_DEMAND);
            assert_eq!(pair(&m), (LAG_COORD_CLIP, "demand"),
                       "rung {rung}: THE LATENT DEFECT — the guard writes `phi_ref`, a THIRD value \
                        rung 79's docstring does not declare, and `lag_coord` stays `clip`. If the \
                        Python is repaired this reads (\"demand\", \"phi\") and this is the ONE \
                        assertion that flips.");
            assert_eq!(cs.displaced(), PHI_REF_PHI,
                       "rung {rung}: what the scope displaced is the REFERENCE, not the coordinate");
        }
        assert_eq!(pair(&m), (LAG_COORD_CLIP, PHI_REF_PHI),
                   "rung {rung}: restored — the field the scope moved, not the one it was meant to");
    }
}

/// **THE DEFECT BUILT: a rung-79 table whose `with_coord` was never re-aimed.** Pointer identity
/// catches it ([`the_slice_re_aims_six_pointers_and_adds_none`]); this shows the field readback
/// catches it too, which is what makes the readback a gate rather than a restatement.
#[test]
fn a_dropped_with_coord_re_aim_is_caught_by_the_readback_too() {
    static DROPPED: TripleHooks = TripleHooks { with_coord: R78_TRIPLE.with_coord, ..R79_TRIPLE };
    let arm = valve_arm();
    let c = full_of(ScheduledStatorTransient::with_ref_tables(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm.stator,
        &R79_TWO, &R79_STATOR, &R79_FUEL, &R79,
        LeverArming { bleed: arm.bleed, sched: arm.bleed_sched, lim: arm.bleed_lim },
        &DROPPED, arm.stator_lim, arm.stator_inc));
    c.fuel.inner.lag_coord.set(LAG_COORD_CLIP);
    let _cs = CoordScope::set(&c.fuel.inner, LAG_COORD_DEMAND);
    assert_eq!((c.fuel.inner.lag_coord.get(), c.fuel.inner.phi_ref.get()),
               (LAG_COORD_DEMAND, PHI_REF_PHI),
               "with the setter left at rung 74's body the scope moves `lag_coord` — so the \
                shipped table's (\"clip\", \"demand\") above is the RE-AIM's doing, measured");
}

// ---------------------------------------------------------------------------------------------
// 4 — THE PLANT
// ---------------------------------------------------------------------------------------------

/// **THE REDUCE, AT THE CELL: `phi_ref = "phi"` IS RUNG 78's BODY, BIT FOR BIT** — and the
/// incidence branch lands on the same ROOT at both states, bit-identical where the fallback
/// answers, with the third value `"demand"` taking the incidence branch exactly as `"incidence"`
/// does.
#[test]
fn the_phi_arm_is_rung_78_and_incidence_finds_the_same_root() {
    let c = coord(&LeverArm::default());
    let g = gauge(&LeverArm::default());
    for s in [SLACK, BINDING] {
        c.fuel.inner.phi_ref.set(PHI_REF_PHI);
        let phi = cap_at(&c, s).expect("the phi leg solves");
        let parent = cap_at(&g, s).expect("and so does rung 78's");
        assert_eq!(phi.to_bits(), parent.to_bits(), "{s:?}: `\"phi\"` IS rung 78's body");

        c.fuel.inner.phi_ref.set(PHI_REF_INCIDENCE);
        let inc = cap_at(&c, s).expect("the incidence leg solves");
        assert!((inc - phi).abs() <= 1e-9 * phi.abs(),
                "{s:?}: same root SET — `h(w) > 0` has no zero: {inc:e} vs {phi:e}");
        if s == BINDING {
            assert_eq!(inc.to_bits(), phi.to_bits(),
                       "binding: the fallback brackets the HARDCODED `phi` residual in both \
                        coordinates, so the coordinate is unreachable and the floats are one");
        } else {
            // THE COORDINATE IS LIVE HERE, AND THE STATE WAS CHOSEN FOR IT. Measured at step 1:
            // at SLACK the two solves land ONE ulp apart (…529140e-2 vs …529106e-2); at
            // `mf_sched = 0.028` on the same spools they land on the SAME bits. Without this line
            // an injection that dropped the incidence arm — `Gi` quietly written as `Gs` — would
            // pass every gate in this file, because "same root to 1e-9" is exactly what it
            // returns. Rung 79's own docstring: *same root, DIFFERENT FLOATS*.
            assert_ne!(inc.to_bits(), phi.to_bits(),
                       "slack: the incidence residual must actually be SOLVED — a bit-identical \
                        answer here means the coordinate was never applied");
        }

        c.fuel.inner.phi_ref.set("demand");
        let third = cap_at(&c, s).expect("the third value proceeds, as `== \"phi\"` lets it");
        assert_eq!(third.to_bits(), inc.to_bits(),
                   "{s:?}: `\"demand\"` IS the incidence branch — the reason `phi_ref` is not an \
                    enum");
    }
}

/// **THE REFUSAL'S DOMAIN, AS A CONTROL**: an incidence leg × a non-identity gauge is refused as an
/// `Abort` carrying `engine.py:20910`'s message; the `phi` leg at the same gauge is not. Its full
/// gate is step 5's.
#[test]
fn the_gauge_refusal_fires_only_off_the_phi_arm() {
    let c = coord(&LeverArm::default());
    c.fuel.inner.gauge_k.set(2.0);
    assert!(cap_at(&c, SLACK).is_ok(),
            "at `\"phi\"` the cell is rung 78's, which gauges only an accel leg, and there is none");
    c.fuel.inner.phi_ref.set(PHI_REF_INCIDENCE);
    let e = cap_at(&c, SLACK).expect_err("incidence x gauge 2 must refuse");
    assert!(e.0.contains("an INCIDENCE phi leg x a non-identity rung-78 GAUGE is REFUSED"),
            "the refusal is rung 79's own, not some other abort: {}", e.0);
    assert!(e.0.contains("_phi_ref = 'incidence'") && e.0.contains("_gauge_k = 2.0"), "{}", e.0);
}

/// **THE COUNTERS ARE PER-THREAD AND MOVE ON THE REAL PLANT** — P3's mechanism, measured.
///
/// No accel leg, so `a_cap` is `+inf` and the phi leg binds the `min` on every call. SLACK brackets
/// the incidence residual (`fb_inc` stays put); BINDING short-circuits to the fallback (`fb_inc`
/// moves) — § 5's non-vacuity condition `fb_inc < calls_inc` met by the first and failed by the
/// second, on purpose. The `phi` arm moves nothing: it is rung 78's body. And a spawned thread
/// reads zeros while this one keeps its counts.
#[test]
fn the_counters_are_per_thread_and_move_on_the_real_plant() {
    let c = coord(&LeverArm::default());

    reset_coord_counters();
    cap_at(&c, SLACK).expect("phi arm");
    assert_eq!(coord_counters(), CoordCounters::default(),
               "the `\"phi\"` arm is rung 78's body and touches none of rung 79's instruments");

    c.fuel.inner.phi_ref.set(PHI_REF_INCIDENCE);
    cap_at(&c, SLACK).expect("incidence, slack");
    let slack = coord_counters();
    assert_eq!(slack, CoordCounters { hits: 1, binds: 1, calls_inc: 1, ..Default::default() },
               "SLACK: the incidence residual was BRACKETED, not substituted");

    cap_at(&c, BINDING).expect("incidence, binding");
    let both = coord_counters();
    assert_eq!(both, CoordCounters { hits: 2, binds: 2, calls_inc: 2, fb_inc: 1,
                                     ..Default::default() },
               "BINDING: the fallback answered, and it was counted on the INCIDENCE side");
    assert!(!coord_probe_armed(), "nothing in this step raises the probe flag");

    let other = std::thread::spawn(coord_counters).join().expect("the reader does not panic");
    assert_eq!(other, CoordCounters::default(),
               "a spawned thread sees its OWN counters — the property that removes slice AH step \
                7's race");
    assert_eq!(coord_counters(), both, "and this thread's are untouched by the other's read");

    assert_eq!(reset_coord_counters(), both, "the reset RETURNS what it cleared, all six at once");
    assert_eq!(coord_counters(), CoordCounters::default());
}

/// **THE THIRD VALUE REACHES THE REFUSAL, NOT A PANIC** — P4's cell-level half. Rung 74's reader
/// turns the `Abort` into a panic carrying its message; that end is step 6's oracle arm.
#[test]
fn the_third_value_reaches_rung_79s_refusal() {
    let c = coord(&LeverArm::default());
    c.fuel.inner.gauge_k.set(2.0);
    let msg = message_of(|| {
        let _cs = CoordScope::set(&c.fuel.inner, LAG_COORD_DEMAND);
        let _ = cap_at(&c, SLACK).map_err(|e| panic!("{}", e.0));
    });
    assert!(msg.contains("an INCIDENCE phi leg x a non-identity rung-78 GAUGE is REFUSED")
                && msg.contains("_phi_ref = 'demand'"),
            "the scope's `\"demand\"` lands in `phi_ref` and reaches `engine.py:20910`: {msg:?}");
}

/// The eighteen `TripleHooks` cells of two tables, paired by name, as `usize` addresses —
/// exhaustive destructurings, so a nineteenth field is a compile error here too.
fn triple_pairs(a: &TripleHooks, b: &TripleHooks) -> Vec<(usize, usize, &'static str)> {
    let TripleHooks {
        stator_leg: a1, lagged_stator: a2, clamp_v: a3, check_v0: a4, rk4_floor: a5, solve_v: a6,
        manifold_v: a7, triple_laws: a8, triple_rig: a9, with_ref: a10, reference: a11,
        quad_gains_at: a12, rk4_floor_shared: a13, cap_fuel: a14, windup_tau: a15, with_coord: a16,
        sensed_cap: a17, shared_rig: a18,
    } = *a;
    let TripleHooks {
        stator_leg: b1, lagged_stator: b2, clamp_v: b3, check_v0: b4, rk4_floor: b5, solve_v: b6,
        manifold_v: b7, triple_laws: b8, triple_rig: b9, with_ref: b10, reference: b11,
        quad_gains_at: b12, rk4_floor_shared: b13, cap_fuel: b14, windup_tau: b15, with_coord: b16,
        sensed_cap: b17, shared_rig: b18,
    } = *b;
    vec![
        (a1 as usize, b1 as usize, "stator_leg"),
        (a2 as usize, b2 as usize, "lagged_stator"),
        (a3 as usize, b3 as usize, "clamp_v"),
        (a4 as usize, b4 as usize, "check_v0"),
        (a5 as usize, b5 as usize, "rk4_floor"),
        (a6 as usize, b6 as usize, "solve_v"),
        (a7 as usize, b7 as usize, "manifold_v"),
        (a8 as usize, b8 as usize, "triple_laws"),
        (a9 as usize, b9 as usize, "triple_rig"),
        (a10 as usize, b10 as usize, "with_ref"),
        (a11 as usize, b11 as usize, "reference"),
        (a12 as usize, b12 as usize, "quad_gains_at"),
        (a13 as usize, b13 as usize, "rk4_floor_shared"),
        (a14 as usize, b14 as usize, "cap_fuel"),
        (a15 as usize, b15 as usize, "windup_tau"),
        (a16 as usize, b16 as usize, "with_coord"),
        (a17 as usize, b17 as usize, "sensed_cap"),
        (a18 as usize, b18 as usize, "shared_rig"),
    ]
}
