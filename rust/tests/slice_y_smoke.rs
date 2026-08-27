//! SLICE Y step 2 — the SMOKE section: does the rung-65 port march the same machine Python does?
//!
//! Not a gate on the rung's claims (that is `tests/rung65.rs`) and not the oracle (that is
//! `tests/slice_y_oracle.rs`). This is the first-contact check: **one lagged `_bill_cell` at
//! `ds = 0.01`, against the values `probe_y6`'s baseline witness printed off PyPy**, plus the two
//! reduce arms that must be bit-for-bit.
//!
//! The anchors below are PyPy's, transcribed from
//! `M:\claud_projects\temp\rust-phase7\y6_wit_baseline.txt`. They are ABSOLUTE on purpose: the
//! reduce spine compares two quantities from one run and is blind to anything that moves both
//! sides together — slice X's own reason for adding absolute gates.
//!
//! **AND THE FIRST SET OF ANCHORS WAS WRONG BY SEVEN ULPS, FOR A REASON WORTH RECORDING.** The
//! probe harness spelled the gas constant `R_c = 0.4/1.4*1004.0` where every suite — and
//! `rust/oracle/dump_slice_x.py` — spells it `(1.4 - 1.0)/1.4*1004.0`. `1.4 - 1.0` is **not** the
//! double nearest `0.4`, so the probe was running a machine one ulp away from the suite's, and
//! that one ulp moved `nu0_lp` by SEVEN ulps and `min_phi_lp` by five. It presented exactly as a
//! port defect — the port was right and the ORACLE HARNESS was wrong. Slice S step 4's lesson
//! (*a probe's HEADER claimed the suites' grids and its code ran another*) one level down: not
//! the grid, the GAS. **What settled it was re-running `dump_slice_x.py` against its committed
//! TSV — 0 of 318 keys differed**, which proves a divergence is the probe's and not the shipped
//! code's before a single line of the port is touched. Do that check FIRST next time.

use turbojet::bleed_transient::{BleedSchedule, LeverArm};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{FuelPoint, PointExtra};
use turbojet::gas::{Gas, GasSpec};
use turbojet::lagged_bleed::build_lagged_bleed;
use turbojet::limited_bleed::{build_limited_bleed, BleedLimiter};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const R: f64 = 0.5;
const SETTLE: f64 = 1.2;
const B: f64 = 0.10;
const PHI: f64 = 0.80;
const TAU: f64 = 0.05;
const N_LO: f64 = 0.65;

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

/// Python's `_gt(...)` — a rung-65 machine.
fn gt(arm: &LeverArm) -> ScheduledStatorCore {
    match build_lagged_bleed(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

/// A rung-64 machine on the SAME hardware — the reduce's reference.
fn lt(arm: &LeverArm) -> ScheduledStatorCore {
    match build_limited_bleed(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn ramp(ds: f64) -> Ramp {
    Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds }
}

fn valve(tau: Option<f64>) -> BleedLimiter { BleedLimiter::with_tau(PHI, B, tau) }

fn keys(t: &[FuelPoint]) -> Vec<(f64, f64, f64, f64, f64, f64, f64)> {
    t.iter().map(|p| (p.s, p.nu_lp, p.nu_hp, p.phi_lp, p.phi_hp, p.tt4, p.mf)).collect()
}

// =============================================================================================
// A — THE MARCH, against PyPy's own numbers
// =============================================================================================

/// The whole port in one cell. Every anchor is PyPy's, to the last printed digit.
#[test]
fn a_lagged_bill_cell_matches_pypy() {
    let c = gt(&LeverArm::floored(valve(Some(TAU)))).bill_cell(&flight(), &ramp(0.01), true);
    assert_eq!(c.min_phi_lp, 0.789129957606298, "min_phi_lp");
    assert_eq!(c.b_int, 0.04013022608947364, "b_int");
    assert_eq!(c.b_peak, 0.08994585756406961, "b_peak");
    assert_eq!(c.b_end, 3.646186254989534e-12, "b_end");
    assert_eq!(c.nu0_lp, 0.7475441088051796, "nu0_lp");
    assert_eq!(c.nu_lp_end, 0.9409407483365471, "nu_lp_end");
    assert_eq!(c.thrust_end, 606.3733076340791, "thrust_end");
    assert_eq!(c.plateau_pts, 1, "plateau_pts");
    assert_eq!(c.npts, 171, "npts");
    let t = c.traj.as_ref().unwrap();
    let (b0, cmd0) = match t[0].extra {
        PointExtra::Valve { b, b_cmd } => (b, b_cmd),
        _ => panic!("the lagged march must record the valve"),
    };
    assert_eq!(b0, 0.03662636367018262, "b(0)");
    assert_eq!(cmd0, b0, "b(0) IS the equilibrium command");
    let track = t.iter()
        .map(|p| match p.extra {
            PointExtra::Valve { b, b_cmd } => (b - b_cmd).abs(),
            _ => unreachable!(),
        })
        .fold(f64::NEG_INFINITY, f64::max);
    assert_eq!(track, 0.02566343697288586, "max tracking error");
    // Every rung-65 point carries SIXTEEN keys — a DIFFERENT two from rung 52's.
    assert!(t.iter().all(|p| p.key_count() == 16));
}

/// `b0` passed at the value the march would have chosen must reproduce it bit-for-bit, or the
/// isolation instrument is perturbing the thing it measures.
#[test]
fn b_b0_at_the_natural_value_is_bit_for_bit() {
    let m = gt(&LeverArm::floored(valve(Some(TAU))));
    let (a, _) = m.stator_march(&flight(), &ramp(0.01), None, &Default::default());
    let b0 = match a[0].extra { PointExtra::Valve { b, .. } => b, _ => unreachable!() };
    let (b, _) = m.stator_march_scoped(&flight(), &ramp(0.01), None, &Default::default(),
                                       &MarchScope { b0: Some(b0) });
    assert_eq!(keys(&a), keys(&b));
    // …and a DIFFERENT b0 must move it, or the instrument is inert and the gate above is vacuous.
    let (c, _) = m.stator_march_scoped(&flight(), &ramp(0.01), None, &Default::default(),
                                       &MarchScope { b0: Some(0.03) });
    assert_ne!(keys(&a), keys(&c));
    assert_eq!(match c[0].extra { PointExtra::Valve { b, .. } => b, _ => unreachable!() }, 0.03);
}

// =============================================================================================
// B — THE REDUCE, arm one: an UNLAGGED rung-65 machine IS rung 64, at every arming mode
// =============================================================================================

#[test]
fn c_no_lag_is_rung_64_bit_for_bit_at_every_arming() {
    for arm in [LeverArm::default(), LeverArm::constant(B),
                LeverArm::scheduled(BleedSchedule::new(B, N_LO)),
                LeverArm::floored(valve(None))] {
        let (a, _) = gt(&arm).stator_march(&flight(), &ramp(0.01), None, &Default::default());
        let (b, _) = lt(&arm).stator_march(&flight(), &ramp(0.01), None, &Default::default());
        assert_eq!(keys(&a), keys(&b), "{:?}", arm.keys());
        // and the un-lagged route must leave the 14-key point shape alone
        assert!(a.iter().all(|p| p.key_count() == 14));
    }
}

/// Arm one again, one level down: a DORMANT floor must reach rung 63's body at every state.
#[test]
fn d_a_dormant_floor_still_dispatches_away() {
    let m = gt(&LeverArm::default());
    let (a, _) = m.at_lever(&LeverArm::floored(BleedLimiter::new(0.30, B)))
                  .stator_march(&flight(), &ramp(0.01), None, &Default::default());
    let (b, _) = m.at_lever(&LeverArm::default())
                  .stator_march(&flight(), &ramp(0.01), None, &Default::default());
    assert_eq!(keys(&a), keys(&b));
}
