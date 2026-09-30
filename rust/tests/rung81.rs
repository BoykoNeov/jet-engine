//! RUNG 81 — **THE AUTHORITY CLOCK**: `docs/rung80-spec.md` § 10's first seam.
//!
//! **HEADLINE: a leg that never holds the actuator has no clock.** In `clip` at the split wall a
//! 10× sweep of the fuel leg's own time constant moves not one bit of the march, while the same
//! sweep at the SHARED wall, where that leg does take the actuator, is live. And the mechanism:
//! authority is decided by the LAG, not by the SET POINT.
//!
//! Ported from `tests/test_rung81.py`: **12 collected tests, 10 of them `slow` there** (the port
//! has no tier — slice M step 5). **This file has 12**, 1:1 in order; rung 81 returns no void
//! string, so none of the ten driven voids (plan § 5.34 (vi)) lands here.
//!
//! # THE PYTHON↔RUST MAP — 1:1 IN ORDER, 0 ADDED, 0 COLLAPSED, **1 SUBSTITUTED**
//!
//! | # | `tests/test_rung81.py` | here |
//! |---|---|---|
//! | 1 | `reduce_is_bit_for_bit_rung80` | [`reduce_the_reader_reads_split_marchs_own_march`] — **SUBSTITUTED**, below |
//! | 2 | `matched_clock_control_reproduces_rung80` | [`matched_clock_control_reproduces_rung80`] |
//! | 3 | `the_fuel_leg_takes_the_actuator` | [`the_fuel_leg_takes_the_actuator`] |
//! | 4 | `the_valve_clock_is_not_the_cause` | [`the_valve_clock_is_not_the_cause`] |
//! | 5 | `the_losing_leg_is_the_more_severe_one` | [`the_losing_leg_is_the_more_severe_one`] |
//! | 6 | `tau_gov_still_modulates_the_window` | [`tau_gov_still_modulates_the_window`] |
//! | 7 | `the_masked_legs_clock_moves_not_one_bit` | [`the_masked_legs_clock_moves_not_one_bit`] |
//! | 8 | `the_same_clock_is_live_where_the_leg_holds` | [`the_same_clock_is_live_where_the_leg_holds`] |
//! | 9 | `demand_columns_are_not_inert` | [`demand_columns_are_not_inert`] |
//! | 10 | `the_mask_survives_the_switch` | [`the_mask_survives_the_switch`] |
//! | 11 | `the_zero_is_falsifiable_on_one_code_path` | [`the_zero_is_falsifiable_on_one_code_path`] |
//! | 12 | `the_criterion_predicts_the_authority_label` | [`the_criterion_predicts_the_authority_label`] |
//!
//! # WHERE THE PORT DIFFERS FROM A LINE-BY-LINE COPY
//!
//! * **#1 is SUBSTITUTED, because a verbatim copy would compare a function with itself.** Python
//!   marches an `AuthorityClockTransient` and a `SplitWallTransient` and requires the same march
//!   to the last bit: two CLASSES, so a real check that rung 81 overrode nothing. The port has no
//!   rung-81 class — rung 81 adds no cell, so its readers take an `R80` core (plan § 5.34 (ii)) —
//!   and both sides would be one `split_march` on one `build_split_wall_cascade` core. What the
//!   Rust reader CAN get wrong is the THREADING of its arguments into that march (the four clocks
//!   in order, the ramp, the step, which wall goes where). So the gate takes the `clock` fixture's
//!   `(demand, tau_f = 0.20, tau_gov = 0.02)` row — two DIFFERENT clocks, so a swapped pair is
//!   visible — and requires every count and every cell `s`/`setpoint_gap`/`tau_gov` to equal what
//!   an independent `split_march` at `(0.20, 0.02, 0.05, 0.05)` gives, read off the points' own
//!   `authority`/`required_*` fields rather than through the reader's helpers. Python's
//!   `len == 341` and eight-key check are kept on the MATCHED march, with every point asserted a
//!   `Demand` point (`slice_af_dispatch.rs`: a `clip` march also returns 341).
//! * **`is None` comparisons become `Option`s:** `mask["cyc_gov_auth"] == 0.0` is `== Some(0.0)`
//!   (Python's `None == 0.0` is `False`, so a `None` fails there too), and `clock["agreement"] >=
//!   0.95` expects `Some` (Python's `None >= 0.95` raises).
//! * `clock["fuel_cells"]["demand"]` is a lookup in an insertion-ordered `Vec` of pairs; a
//!   missing key is Python's `KeyError`, here a panic by name.

use std::sync::OnceLock;

use turbojet::authority_clock::{
    authority_clock, authority_mask, AuthorityClock, AuthorityMask, ClockRow, CLOCK_COORDS,
    CLOCK_TAU_FS, CLOCK_TAU_GOVS, CLOCK_TAU_Q, CLOCK_TAU_S, MASK_CLOCKS, MASK_EVERY,
};
use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{Authority, FuelPoint, PointExtra};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::shared_actuator::riding4_idx;
use turbojet::split_wall::{build_split_wall_cascade, split_march};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// ============================================================================== the grid
//
// `tests/test_rung81.py`'s module constants, verbatim.

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const TT4_MAX: f64 = 1200.0;
const B: f64 = 0.10;
const V_MAX: f64 = 0.20;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
/// RUNG 80's OWN CELL, unchanged.
const PHI_FUEL: f64 = 0.75;
const PHI_AIR: Option<f64> = Some(0.77);
/// Rung 80's clocks: the control.
const MATCHED: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);

// `authority_clock`'s / `authority_mask`'s unspelled defaults (`engine.py:21904`, `:22069`) —
// the suite's two fixtures pass only the walls, so every other argument is the reader's default.
const R: f64 = 0.5;
const S_SETTLE: f64 = 1.2;
const DS: f64 = 0.005;

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

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

/// Python's `_rig(design)`, with its four knob assignments. The class is
/// `AuthorityClockTransient` there; here it is a rung-80 core (plan § 5.34 (ii)).
fn rig() -> ScheduledStatorCore {
    let sm = 0.80 / FLOOR - 1.0;
    let arm = LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, sm, Some(TAU))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S))),
        ..Default::default()
    };
    let m = match build_split_wall_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    };
    let t = &m.fuel.inner;
    t.lag_coord.set("demand");
    t.ref_law.set("sched");
    t.windup_law.set("none");
    t.cap_law.set("solve");
    m
}

/// Python's `_march(m, taus)` — `coord = "demand"`, `r = 0.5`, `s_settle = 1.2`, `ds = 0.005`.
fn march(m: &ScheduledStatorCore, taus: (f64, f64, f64, f64)) -> (ScheduledStatorCore, Vec<FuelPoint>) {
    let (built, _, _, traj) = split_march(m, &flight(), LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR,
                                          "demand", taus, R, S_SETTLE, DS, V_MAX, false);
    (built, traj)
}

// ============================================================================== the fixtures
//
// Python's module-scoped fixtures, each on a FRESH rig; computed once per binary.

fn clock() -> &'static AuthorityClock {
    static S: OnceLock<AuthorityClock> = OnceLock::new();
    S.get_or_init(|| authority_clock(&rig(), &flight(), LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR,
                                     &CLOCK_TAU_FS, &CLOCK_TAU_GOVS, CLOCK_TAU_Q, CLOCK_TAU_S,
                                     &CLOCK_COORDS, R, S_SETTLE, DS, V_MAX, false))
}

fn mask() -> &'static AuthorityMask {
    static S: OnceLock<AuthorityMask> = OnceLock::new();
    S.get_or_init(|| authority_mask(&rig(), &flight(), LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR,
                                    &MASK_CLOCKS, "demand", R, S_SETTLE, DS, V_MAX, false,
                                    MASK_EVERY)
        .expect("no Abort on the suite's rig"))
}

fn demand_rows(c: &AuthorityClock, tau_f: f64) -> Vec<&ClockRow> {
    c.rows.iter().filter(|x| x.coord == "demand" && x.tau_f == tau_f).collect()
}

// ============================================================================== § 7.1: the reduce

/// **SUBSTITUTED for Python's `reduce_is_bit_for_bit_rung80`** (header). The reader's row at two
/// DIFFERENT clocks must be exactly what an independent march at those clocks gives — so the
/// reader threads `(tau_f, tau_gov, tau_q, tau_s)`, `r`, `ds` and both walls into rung 80's march
/// unchanged. Python's `341` and its eight keys are kept, on the matched march.
#[test]
fn reduce_the_reader_reads_split_marchs_own_march() {
    // Python's own assertion, on the matched march: 341 points, each carrying the eight keys --
    // which in this crate means each is a `Demand` point
    let (_, traj) = march(&rig(), MATCHED);
    assert_eq!(traj.len(), 341);
    assert!(traj.iter().all(|p| matches!(p.extra, PointExtra::Demand { .. })),
            "DECLARED: the march did not run `demand` -- a clip march also returns 341 points");

    // the substitution: the reader's (demand, 0.20, 0.02) row against its own march, rebuilt
    let (tf, tg) = (0.20, 0.02);
    let row = clock().rows.iter()
        .find(|x| x.coord == "demand" && x.tau_f == tf && x.tau_gov == tg)
        .expect("the grid carries (demand, 0.20, 0.02)");
    let (built, traj) = march(&rig(), (tf, tg, CLOCK_TAU_Q, CLOCK_TAU_S));
    let b_max = built.fuel.inner.lever.lim.expect("a valve").b_max;
    let ride = riding4_idx(&traj, b_max);
    let auth = |i: usize| match traj[i].extra {
        PointExtra::Demand { authority, .. } => authority,
        _ => unreachable!("every point is a Demand point"),
    };
    let max_tt4 = traj.iter().map(|p| p.tt4).fold(f64::NEG_INFINITY, f64::max);
    assert_eq!(row.max_tt4.to_bits(), max_tt4.to_bits(), "max Tt4");
    assert_eq!(row.n_riding4, ride.len(), "n_riding4");
    assert!(row.n_riding4 > 0, "an empty window would make every count below vacuous");
    assert_eq!(row.n_fuel, ride.iter().filter(|&&i| auth(i) == Authority::Fuel).count(), "n_fuel");
    assert_eq!(row.n_gov, ride.iter().filter(|&&i| auth(i) == Authority::Gov).count(), "n_gov");
    let interior: Vec<usize> = ride.iter().copied().filter(|&i| 0 < i && i < traj.len() - 1).collect();
    assert_eq!(row.cells.len(), interior.len(), "the interior cell count");
    assert_eq!(row.n_edge, ride.len() - interior.len(), "n_edge");
    for (c, &i) in row.cells.iter().zip(&interior) {
        let PointExtra::Demand { required_fuel, required_gov, .. } = traj[i].extra else {
            unreachable!("every point is a Demand point")
        };
        assert_eq!(c.s.to_bits(), traj[i].s.to_bits(), "cell s at index {i}");
        assert_eq!(c.setpoint_gap.to_bits(), (required_gov - required_fuel).to_bits(),
                   "setpoint gap at s = {}", c.s);
        assert_eq!(c.tau_gov, tg, "the governor clock the cell was scored with");
    }
}

// ============================================================================== § 7.2: the control

/// THE CONTROL FIXES THE RIG: rung 80's own cell returns its own number — 33 four-loop points,
/// every one held by the GOVERNOR.
#[test]
fn matched_clock_control_reproduces_rung80() {
    let c = clock();
    assert!(c.control_all_gov, "rung 80's cell no longer reports an all-governor census");
    assert_eq!(c.control_n_riding4, vec![33],
               "rung 80 § 2 measured 33 four-loop points here; got {:?}", c.control_n_riding4);
    assert!(!c.all_fuel,
            "fuel authority at EVERY cell would mean the reader is not reading the clock at all");
    assert_eq!(c.n_invalid, 0, "a row arrested; its counts are void (rung 80 § 8)");
}

// ============================================================================== § 7.3: the seam's cell

/// THE SEAM'S OWN OBJECT: a `demand` four-loop cell with the φ fuel leg AUTHORITATIVE, scored on a
/// VALID row.
#[test]
fn the_fuel_leg_takes_the_actuator() {
    let hot = demand_rows(clock(), 0.20);
    assert!(!hot.is_empty(), "the grid no longer carries the slow-fuel row");
    for x in hot {
        assert!(x.riding4_valid, "the plant never left Tt4_lo -- the count is void");
        assert!(x.n_fuel > 0 && x.n_gov == 0,
                "tau_f = 0.20, tau_gov = {}: expected an all-fuel census, got {:?}",
                x.tau_gov, x.census);
    }
}

/// ANCHOR P2: with `tau_q` pinned at rung 80's 0.05 the fuel-authority cell opens anyway.
#[test]
fn the_valve_clock_is_not_the_cause() {
    let c = clock();
    assert!(c.tau_q == 0.05 && c.tau_s == 0.05);
    let dem = &c.fuel_cells.iter().find(|(k, _)| *k == "demand")
        .expect("Python's `fuel_cells[\"demand\"]` -- KeyError").1;
    assert!(!dem.is_empty(),
            "with the valve clock pinned there is no fuel-authority cell at all -- the \
             pre-check's result would then have been the VALVE's, not the fuel leg's");
    let lowest = dem.iter().map(|&(tf, _, _)| tf).fold(f64::INFINITY, f64::min);
    assert!(lowest >= 0.08, "the threshold moved below the measured 0.08 -- the spec's § 1 number \
                             is stale");
}

/// THE HEADLINE SENTENCE, GATED: wherever the fuel leg holds, the GOVERNOR's own demand is still
/// the larger — the lag and the set point DISAGREE.
#[test]
fn the_losing_leg_is_the_more_severe_one() {
    let fuel: Vec<f64> = clock().rows.iter()
        .filter(|x| x.riding4_valid)
        .flat_map(|x| x.cells.iter())
        .filter(|c| c.measured == Authority::Fuel)
        .map(|c| c.setpoint_gap)
        .collect();
    assert!(fuel.len() > 200, "only {} fuel-held points scored -- too few to claim this", fuel.len());
    let worst = fuel.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(worst > 0.0,
            "the fuel leg held the actuator while ALSO demanding the deeper cut (set-point gap \
             {worst:.4e}) -- the two nouns agree there, and the headline does not hold");
}

/// THE ANTI-OVER-CLAIM GATE: at fixed `tau_f` the governor's clock still moves the fuel-held
/// count.
#[test]
fn tau_gov_still_modulates_the_window() {
    let row: Vec<(f64, usize)> = demand_rows(clock(), 0.12).iter()
        .map(|x| (x.tau_gov, x.n_fuel)).collect();
    let mut vals: Vec<usize> = row.iter().map(|&(_, n)| n).collect();
    vals.sort_unstable();
    vals.dedup();
    assert!(vals.len() > 1, "tau_gov no longer moves the fuel-held count at tau_f = 0.12: {row:?}");
}

// ============================================================================== § 7.4: the headline

/// § 3, THE HEADLINE, the NEGATIVE half: in `clip` a 10× sweep of the masked leg's clock moves
/// not one of 1 364 floats.
#[test]
fn the_masked_legs_clock_moves_not_one_bit() {
    let cols: Vec<_> = clock().tau_f_inert.iter().filter(|(k, _)| k.starts_with("clip@")).collect();
    assert_eq!(cols.len(), 3, "expected three clip columns, got {:?}",
               cols.iter().map(|(k, _)| k).collect::<Vec<_>>());
    for (k, v) in cols {
        let v = v.as_ref().unwrap_or_else(|| panic!("{k}: no column"));
        assert!(v.all_valid && v.n_tau_f == 6);
        assert_eq!(v.n_floats, 1364, "{k}: the comparison lost its resolution");
        assert!(v.n_differing == 0 && v.march_identical,
                "{k}: a masked leg's clock moved {} of {} floats", v.n_differing, v.n_floats);
        assert!(v.riding4_identical, "{k}: the four-loop count moved: {:?}", v.n_riding4);
    }
}

/// § 3, THE POSITIVE HALF: at the SHARED wall the same sweep is live, and a slower leg holds on
/// FEWER points, monotonically.
#[test]
fn the_same_clock_is_live_where_the_leg_holds() {
    let c = clock();
    assert!(c.control_clip_fuel > 0,
            "the reader cannot say `fuel` in `clip` at all -- the null above is then a broken \
             reader, not a masked leg (anchor V3)");
    assert!(c.control_clip_tau_f_live,
            "the same 10x sweep is inert at the SHARED wall too -- § 3's headline does not hold");
    // `sorted(..., key=lambda x: x["tau_f"])` -- STABLE
    let mut rows: Vec<_> = c.control_clip_rows.iter().collect();
    rows.sort_by(|a, b| a.tau_f.partial_cmp(&b.tau_f).expect("tau_f is not NaN"));
    let n: Vec<usize> = rows.iter().map(|x| x.n_fuel).collect();
    assert!(n[0] > n[n.len() - 1], "a slower fuel leg did not lose authority in clip: {n:?}");
    assert!(n.windows(2).all(|w| w[0] >= w[1]), "the fall is not monotone: {n:?}");
}

/// The contrast that makes § 3 a SPLIT: the same knob is decisive in `demand`.
#[test]
fn demand_columns_are_not_inert() {
    for (k, v) in &clock().tau_f_inert {
        if !k.starts_with("demand@") {
            continue;
        }
        let v = v.as_ref().unwrap_or_else(|| panic!("{k}: demand went inert (no column)"));
        assert!(!v.march_identical, "{k}: demand went inert");
        assert!(v.n_differing > 1000, "{k}: only {} floats moved", v.n_differing);
        assert!(v.n_riding4.len() > 1, "{k}: the four-loop count never moved");
    }
}

// ============================================================================== § 7.5: the mirror mask

/// ANCHOR P4: `min` masks the GOVERNOR instead, and rung 72's block is indifferent to which leg
/// it masks.
#[test]
fn the_mask_survives_the_switch() {
    let m = mask();
    assert!(!m.vacuous, "only one authority regime is present (fuel {}, gov {}) -- nothing about \
                         the mask is scored (anchor V1)", m.n_fuel_interior, m.n_gov_interior);
    assert!(m.n_fuel_interior > 0 && m.n_gov_interior > 0);
    assert_eq!(m.max_mask_leak, Some(0.0), "the masked leg reached the plant: {:?}", m.max_mask_leak);
    assert!(!m.ever_two_authorities, "two legs held the actuator at one point");
    assert!(m.all_differenced, "some four-loop point was skipped, so the zeros above are computed \
                                over a subset (rung 78 § 5.1's trap)");
}

/// THE DISCRIMINATOR: the three-φ-loop cycle is exactly 0 where the fuel leg is masked and
/// non-zero where the governor is — both on one code path.
#[test]
fn the_zero_is_falsifiable_on_one_code_path() {
    let m = mask();
    assert_eq!(m.cyc_gov_auth, Some(0.0), "`min` is not flat in the masked fuel leg: {:?}",
               m.cyc_gov_auth);
    assert!(m.cyc_fuel_auth.is_some_and(|x| x > 0.5),
            "the SAME reader returns {:?} where the governor is masked -- if that is zero too, \
             the zero above is unfalsifiable and scores nothing", m.cyc_fuel_auth);
}

// ============================================================================== § 7.6: the criterion

/// ANCHOR P1, at the bar that was REGISTERED (>= 95 % worst cell), not at the value measured.
#[test]
fn the_criterion_predicts_the_authority_label() {
    let c = clock();
    assert!(c.n_scored > 900, "only {} points scored", c.n_scored);
    let a = c.agreement.expect("Python's `None >= 0.95` raises -- no row scored");
    assert!(a >= 0.95, "worst cell agrees at {a:.4}, below the registered 0.95");
    for x in &c.rows {
        assert_eq!(x.n_edge, 0,
                   "{} ({}, {}): {} four-loop points had no central difference and were dropped \
                    -- the spec's count is stale", x.coord, x.tau_f, x.tau_gov, x.n_edge);
    }
}
