//! RUNG 74 — **THE DEMAND COORDINATE**: rung 73 § 11's own sharpest seam, and the last place
//! `n_live = 4` could still hide.
//!
//! Every fuel-side leg since rung 47 carries the CLIP as its state — the CUT, floored at zero. A
//! real fuel control carries the DEMAND, the fuel it would allow, and the lowest wins.
//!
//! **THE HEADLINE: A COORDINATE ON THE LAG IS PURE BILL. IT CANNOT TOUCH THE RANK, AND IT MOVES
//! THE CUT BY THE SCHEDULE'S OWN SLOPE.** Substituting `w = mf_sched - g` gives
//! `dg/ds = (req - g)/tau + d(mf_sched)/ds` — a STATE-INDEPENDENT forcing, so it appears in no
//! Jacobian; and `min()` is flat in the masked demand exactly as `max()` was flat in the masked
//! clip, so the masked column is still zero and `n_live` is still <= 3.
//!
//! Ported from `tests/test_rung74.py` — **16 collected tests, of which 8 carry `slow` there.**
//! Both numbers are MEASURED (`pytest --collect-only -q -n0`, once plain and once with `-m slow`),
//! never read off a sentence, and the count here is reconciled against `cargo test -- --list`
//! rather than against a `grep`: slice AC step 4, slice AD step 4 and slice AE step 3 each had
//! `grep` over-count by one because the extra sat inside a doc comment. **This header quotes the
//! number 17, so this file is the fourth instance waiting to happen.** The `slow` marker is
//! dropped per slice M's rule; `#[ignore]` is re-introduced only against a MEASURED Rust cost.
//!
//! # THE PYTHON↔RUST MAP — 1:1 IN ORDER, **1 ADDED**, 0 COLLAPSED, 0 SPLIT BY PARAMETER
//!
//! The Python file carries no `@pytest.mark.parametrize`, so nothing splits. **One gate is ADDED**
//! and it is declared here rather than absorbed:
//!
//! | # | `tests/test_rung74.py` | here |
//! |---|---|---|
//! | 1 | `reduces_to_rung73_in_clip_coordinates` | [`reduces_to_rung73_in_clip_coordinates`] |
//! | 2 | `the_clip_reduce_is_not_vacuous` | [`the_clip_reduce_is_not_vacuous`] |
//! | 3 | `reduces_by_identity_on_a_flat_schedule` | [`reduces_by_identity_on_a_flat_schedule`] |
//! | 4 | `the_spectrum_is_invariant_and_the_entries_are_not` | [`the_spectrum_is_invariant_and_the_entries_are_not`] |
//! | 5 | `min_select_is_flat_in_the_masked_demand_too` | [`min_select_is_flat_in_the_masked_demand_too`] |
//! | 6 | `the_forcing_is_the_schedules_slope_times_the_clock` | [`the_forcing_is_the_schedules_slope_times_the_clock`] |
//! | 7 | `the_demand_coordinate_holds_the_redline_the_clip_one_breaks` | [`the_demand_coordinate_holds_the_redline_the_clip_one_breaks`] |
//! | 8 | `at_the_inherited_floor_the_demand_plant_does_not_accelerate` | [`at_the_inherited_floor_the_demand_plant_does_not_accelerate`] |
//! | 9 | `the_arrest_is_an_interval_whose_lower_edge_is_the_free_operating_point` | [`the_arrest_is_an_interval_whose_lower_edge_is_the_free_operating_point`] |
//! | 10 | `the_arrest_is_the_demand_coordinates_and_it_ends_at_the_valves_saturation` | [`the_arrest_is_the_demand_coordinates_and_it_ends_at_the_valves_saturation`] |
//! | 11 | `a_masked_applied_referenced_leg_has_no_equilibrium_without_a_stop` | [`a_masked_applied_referenced_leg_has_no_equilibrium_without_a_stop`] |
//! | 12 | `the_lag_returns_attack_on_a_known_attack_point` | [`the_lag_returns_attack_on_a_known_attack_point`] |
//! | 13 | `the_coordinate_is_declared` | [`the_coordinate_is_declared`] |
//! | 14 | `demand_refuses_the_sum_composition` | [`demand_refuses_the_sum_composition`] |
//! | 15 | `at_lever_carries_all_three_knobs` | [`at_lever_carries_all_three_knobs`] |
//! | 16 | `the_unfloored_cap_is_the_shipped_one_wherever_the_leg_binds` | [`the_unfloored_cap_is_the_shipped_one_wherever_the_leg_binds`] |
//! | **+1** | **— none —** | [`the_isolation_instrument_splits_the_rung_at_the_specs_own_numbers`] |
//!
//! # THE ADDED GATE, AND WHY ITS BARS COME FROM A DOC RATHER THAN FROM A TEST
//!
//! `latch_discriminator` **has no caller anywhere in the shipped tree** — the name occurs in
//! `turbojet/engine.py` and in the port plan and nowhere else, which slice AF step 4 § (h) both
//! measured and booked here. So it is the one reader of six with no Python test to mirror, and a
//! file that ported only what the suite gates would leave the rung's § 3 instrument entirely
//! ungated on both sides.
//!
//! Its bars are transcribed from **`docs/rung74-spec.md` § 3's own prose** — *"Measured
//! `floor_dTt4 = 65.2 K` with 332 of 341 points riding"* — and NOT from step 4's drive output.
//! That distinction is § 5.30 (viii) item 1 exactly: the port reproducing a number is not a bar on
//! the port, because the port is what supplied it. The spec sentence was written from the PYTHON
//! reader, months before this module existed.
//!
//! # THE TWO NEEDLES ARE TRANSCRIBED FROM **PYTHON's** LITERALS, NOT FROM THE PORT's
//!
//! Python gates two of rung 74's nine shipped messages, with `match="DECLARED"` and
//! `match="two declared laws"`. Both needles here are read off `turbojet/engine.py:17762` and
//! `:17776` — never off `demand_coordinate.rs`. Step 4 § (c) measured the port's own refusal text
//! **four formatting divergences wide** against Python's (`{:.3e}` vs `%e`, `{:?}` vs `!r`), so a
//! needle copied from the Rust side would certify the divergence instead of catching it. The other
//! seven shipped messages stay ungated HERE because Python leaves them ungated (§ 5.30 (iv)); they
//! are `slice_af_cells.rs`'s subject, and P6's answer is the oracle's.
//!
//! # WHAT OVERLAPS THE THREE `slice_af_*.rs` FILES, AND WHY IT IS PORTED ANYWAY
//!
//! Steps 1–3 already gate close relatives of **13, 14, 15** and of **16**'s table route. They are
//! ported here regardless, on `rung72.rs`/`rung73.rs`'s precedent: this file's contract is a **1:1
//! map of the shipped suite**, and a hole in it is invisible to every count on both sides. The
//! files also gate different things — the step files drive the CELL directly through
//! `R74_TRIPLE`, these drive it through `stator_march_scoped` with the suite's own arming, which
//! is the entry the suite actually uses.
//!
//! # EVERY READER ARGUMENT BELOW THE FIFTH IS THE READER's OWN DEFAULT
//!
//! § 5.27.6 (i) burned a slice on exactly this: a shipped row measured at `every = 40` while the
//! fixture passed `every = 10`. Every rung-74 reader call in the Python file passes at most
//! `FLIGHT, LO, HI, TT4_MAX, phi_lim=` (or `FLIGHT, Tt4_flat, phi_lim=`), so `taus`, `inc`, `r`,
//! `s_settle`, `ds`, `v_max` and `every` come from **`turbojet/engine.py`'s own `def` lines** —
//! never from this module's [`DS`], which agrees with three of the four and NOT with
//! `demand_gains`'s.
//!
//! | reader | `ds` | `every` | called by |
//! |---|---|---|---|
//! | `demand_gains` | **0.002** | **4** | gates 4, 5 |
//! | `forcing_openloop` | **0.005** | — | gate 6 |
//! | `windup_law` | **0.005** | — | gate 11 |
//! | `flat_schedule_identity` | **0.005** | — | gate 3 |
//! | `latch_discriminator` | **0.005** | — | the ADDED gate (its own default, no caller exists) |
//!
//! **There is no overridden default anywhere in the Python file** — measured by reading all four
//! call sites, and recorded because rung 73's file had exactly one and the sentence read the other
//! way would be § 5.27.6 (i)'s defect.

use std::panic::catch_unwind;
use std::ptr::fn_addr_eq;

use turbojet::applied_reference::{build_applied_reference_cascade, REF_LAW_APPLIED};
use turbojet::bleed_transient::LeverArm;
use turbojet::demand_coordinate::{
    build_demand_coordinate_cascade, demand_gains, demand_tau, flat_schedule_identity,
    forcing_openloop, latch_discriminator, windup_law, WindupCell, FLAT_KEYS, R74_FUEL,
    R74_TRIPLE,
};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelPoint, PointExtra, SurgeLimiter,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::shared_actuator::REF_LAW_DEFAULT;
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ============================================================================== the grid
//
// `tests/test_rung74.py`'s module constants, verbatim.

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
const R: f64 = 0.5;
const B: f64 = 0.10;
const PHI: f64 = 0.80;
const V_MAX: f64 = 0.20;
/// `PHI / FLOOR - 1.0` — the expression Python spells, never a typed decimal. Rung 69's
/// constructor asserts `m_lim == T_c - 1/phi_lim`, so a rounded constant breaks a wall identity.
const SM: f64 = PHI / FLOOR - 1.0;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TAU_GOV: f64 = 0.05;
const TAU_ATT: f64 = 0.05;
const TAU_REL: f64 = 0.15;
/// **RUNG 67's imposed redline, VERBATIM through rungs 70–73.**
const TT4_MAX: f64 = 1200.0;

/// **THE THREE `phi` ARMS, and the middle one is a DISCLOSURE.** At the INHERITED floor (0.80) the
/// surge cap sits AT the scheduled fuel from `s = 0` and a leg that TRACKS it permits no
/// acceleration at all — which is gate 8's finding, not a defect. 0.76 is where all three plants
/// accelerate; 0.70 is below the clip plant's own droop, so only the GOVERNOR is live there.
const PHI_ARREST: f64 = 0.80;
const PHI_BOTH: f64 = 0.76;
const PHI_GOV: f64 = 0.70;

// ----------------------------------------------------------- THE READERS' OWN DEFAULTS
//
// See the module header's table. Read off `turbojet/engine.py`'s `def` lines, not off `DS`.

/// Shared by all five readers: `taus = (0.05,)*4`, `inc = False`, `r = 0.5`, `s_settle = 1.2`,
/// `v_max = 0.20`.
const RDR_TAUS: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
const RDR_INC: bool = false;
const RDR_R: f64 = 0.5;
const RDR_SETTLE: f64 = 1.2;
const RDR_V_MAX: f64 = 0.20;

/// `demand_gains(..., ds=0.002, every=4)` — **the one reader whose `ds` is not `DS`.**
const DG_DS: f64 = 0.002;
const DG_EVERY: usize = 4;
/// `forcing_openloop(..., ds=0.005)`, `windup_law(..., ds=0.005)`,
/// `flat_schedule_identity(..., ds=0.005)`, `latch_discriminator(..., ds=0.005)`.
const RDR_DS: f64 = 0.005;
/// `flat_schedule_identity(..., s_end=1.2, Tt4_max=1200.0, nu_offset=0.94)`.
const FSI_S_END: f64 = 1.2;
const FSI_TT4_MAX: f64 = 1200.0;
const FSI_NU_OFFSET: f64 = 0.94;

/// `flat_schedule_identity`'s nine keys, by NAME — gate 3 asserts on `Tt4`, `mf` and `g_gov`, and
/// resolving them by index off [`FLAT_KEYS`] rather than by a typed `2`/`4`/`8` is what keeps the
/// gate pinned to the key Python names if the tuple ever reorders.
fn flat_key(name: &str) -> usize {
    FLAT_KEYS.iter().position(|k| *k == name)
        .unwrap_or_else(|| panic!("`flat_schedule_identity` publishes no key {name:?}"))
}

// ============================================================================== the fixtures

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
    build_two_spool_turbojet(cpg(), PI_LPC, PI_HPC, TT4, 50_000.0, REAL)
}

/// Python's `_sm(phi_lim)`.
fn sm_of(phi_lim: f64) -> f64 { phi_lim / FLOOR - 1.0 }

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

/// Python's `_demand(design, sm=…, inc=…, coord=…, ref=…)`.
///
/// The two knobs are set by PLAIN ASSIGNMENT (`m._lag_coord, m._ref_law = coord, ref`), which is
/// what Python's fixture does — not through `_with_coord`. Routing them through the table here
/// would test a different line.
fn suite_arm(sm: f64, inc: bool) -> LeverArm {
    LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, sm, Some(TAU))),
        stator_lim: if inc { None }
                    else { Some(StatorLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S))) },
        stator_inc: if inc {
            Some(turbojet::reference_split::StatorIncidenceLimiter::from_margin(
                &lp_map(), V_MAX, sm, Some(TAU_S)))
        } else { None },
        ..Default::default()
    }
}

fn demand_rig(sm: f64, inc: bool, coord: &'static str, ref_law: &'static str)
              -> ScheduledStatorCore {
    let arm = suite_arm(sm, inc);
    let m = full_of(build_demand_coordinate_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm));
    m.fuel.inner.lag_coord.set(coord);
    m.fuel.inner.ref_law.set(ref_law);
    m
}

/// The default arm of Python's `_demand`: `sm=SM, inc=False, coord="demand", ref="sched"`.
fn demand_default() -> ScheduledStatorCore {
    demand_rig(SM, false, "demand", REF_LAW_DEFAULT)
}

/// Python's `_applied(design)` — rung 73's own machine, gate 1's other side.
fn applied_rig(sm: f64) -> ScheduledStatorCore {
    let arm = LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, sm, Some(TAU))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S))),
        ..Default::default()
    };
    full_of(build_applied_reference_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm))
}

fn surge_of(sm: f64) -> SurgeLimiter { SurgeLimiter::from_margin(&lp_map(), Spool::Lp, sm) }
fn lag() -> AsymmetricLag { AsymmetricLag::new(TAU_ATT, TAU_REL) }

/// Python's `_march(m, sm=…)` — all four loops, at the suite's own ramp.
fn march(m: &ScheduledStatorCore, sm: f64) -> Vec<FuelPoint> {
    let leg = StatorLeg {
        accel: None::<&AccelSchedule>,
        surge: Some(Floor::Phi(surge_of(sm))),
        tt4_max: Some(TT4_MAX),
    };
    let ramp = Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds: DS };
    m.stator_march_scoped(&flight(), &ramp, None, &leg,
                          &MarchScope { lag: Some(lag()), tau_gov: Some(TAU_GOV),
                                        ..MarchScope::DEFAULT }).0
}

/// Python's `_keys(traj)` — the NINE-tuple, compared BIT for bit.
fn keys(traj: &[FuelPoint]) -> Vec<[u64; 9]> {
    traj.iter().map(|p| [
        p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.phi_lp.to_bits(),
        p.phi_hp.to_bits(), p.tt4.to_bits(), p.mf.to_bits(), b_of(p).to_bits(),
        v_of(p).to_bits(),
    ]).collect()
}

/// Python's `p["b"]` — a `KeyError` off a trajectory that records no valve, which is a panic here.
fn b_of(p: &FuelPoint) -> f64 {
    match p.extra {
        PointExtra::Demand { b, .. } | PointExtra::Shared { b, .. }
        | PointExtra::Triple { b, .. } | PointExtra::CrossCascade { b, .. }
        | PointExtra::Cascade { b, .. } | PointExtra::Valve { b, .. } => b,
        _ => panic!("rung-74's `_keys` reads `b` with a bare index"),
    }
}

/// Python's `p["v"]` — likewise.
fn v_of(p: &FuelPoint) -> f64 {
    match p.extra {
        PointExtra::Demand { v, .. } | PointExtra::Shared { v, .. }
        | PointExtra::Triple { v, .. } => v,
        _ => panic!("rung-74's `_keys` reads `v` with a bare index"),
    }
}

/// Python's `p["required_fuel"]` — gate 16's probe reads it off a CLIP trajectory, so the point is
/// a [`PointExtra::Shared`]; the demand arm is admitted too because the projection carries the key.
fn required_fuel_of(p: &FuelPoint) -> f64 {
    match p.extra {
        PointExtra::Shared { required_fuel, .. } | PointExtra::Demand { required_fuel, .. }
            => required_fuel,
        _ => panic!("rung-74's cap probe reads `required_fuel` with a bare index"),
    }
}

fn max_tt4(traj: &[FuelPoint]) -> f64 {
    traj.iter().fold(f64::NEG_INFINITY, |a, p| a.max(p.tt4))
}

fn min_phi_lp(traj: &[FuelPoint]) -> f64 {
    traj.iter().fold(f64::INFINITY, |a, p| a.min(p.phi_lp))
}

/// Python's `pytest.raises(AssertionError, match=…)` — the message, or `""` on a clean return.
fn message_of<F: FnOnce()>(f: F) -> String {
    match catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(()) => String::new(),
        Err(e) => match e.downcast_ref::<String>() {
            Some(s) => s.clone(),
            None => e.downcast_ref::<&str>().map(|s| (*s).to_string())
                     .unwrap_or_else(|| "<non-string panic>".into()),
        },
    }
}

// ======================================================================================
// THE REDUCE SPINE — TWO arms. One by DISPATCH, one by IDENTITY, and the second is the
// one that matters: it is the only arm in which this rung's own integrator runs.
// ======================================================================================

/// **ARM 1, by DISPATCH**: `_lag_coord = 'clip'` never enters this rung's march at all, so the
/// plant is rung 73 BIT-FOR-BIT — under the APPLIED reference, which is rung 73's own.
#[test]
fn reduces_to_rung73_in_clip_coordinates() {
    let a = keys(&march(&demand_rig(SM, false, "clip", REF_LAW_APPLIED), SM));
    let b = keys(&march(&applied_rig(SM), SM));
    assert_eq!(a, b, "the clip coordinate IS rung 73, bit for bit");
}

/// **AND ARM 1 MUST BE A TEST, NOT A TAUTOLOGY.** If `_lag_coord` were ignored, the reduce above
/// would compare rung 74 with rung 74 and pass. The SAME machine under `demand` must differ.
#[test]
fn the_clip_reduce_is_not_vacuous() {
    let sm = sm_of(PHI_BOTH);
    let a = keys(&march(&demand_rig(sm, false, "clip", REF_LAW_DEFAULT), sm));
    let b = keys(&march(&demand_rig(sm, false, "demand", REF_LAW_DEFAULT), sm));
    assert_ne!(a, b, "the coordinate is READ: the same machine marches two different plants");
}

/// **ARM 2, by IDENTITY, and it is the load-bearing one** — the only reduce in which
/// `_integrate_fuel_demand` actually runs.
///
/// On a FLAT schedule the forcing `mf_dot*tau` is identically zero and the latch's stop coincides
/// with the clip plant's `g >= 0`, so `demand-latched` IS the clip plant. It is NOT bit-for-bit
/// (anchor P7, scored REFUTED-as-stated): the two marches compute the same quantity through
/// different float expressions — `cap - w` against `-(req - g)` — so the agreement is `~1e-15`
/// relative, not exact. **Every bar below is transcribed from `tests/test_rung74.py:154-156`.**
#[test]
fn reduces_by_identity_on_a_flat_schedule() {
    let d = flat_schedule_identity(&demand_rig(sm_of(PHI_BOTH), false, "demand", REF_LAW_DEFAULT),
                                   &flight(), 1150.0, PHI_BOTH, RDR_TAUS, RDR_INC, FSI_S_END,
                                   RDR_DS, RDR_V_MAX, FSI_TT4_MAX, FSI_NU_OFFSET);
    assert!(d.non_vacuous && d.riding == d.n,
            "non_vacuous={} riding={} n={}", d.non_vacuous, d.riding, d.n);
    assert!(d.span_tt4.1 - d.span_tt4.0 > 20.0, "span_Tt4 = {:?}", d.span_tt4);
    assert!(d.worst[flat_key("Tt4")] < 1e-9, "worst Tt4 = {}", d.worst[flat_key("Tt4")]);
    assert!(d.worst[flat_key("mf")] < 1e-15 && d.worst[flat_key("g_gov")] < 1e-15,
            "worst mf = {}, worst g_gov = {}",
            d.worst[flat_key("mf")], d.worst[flat_key("g_gov")]);
}

// ======================================================================================
// § 1 — THE ENTRIES MOVE, THE SPECTRUM DOES NOT
// ======================================================================================

/// § 1 (anchor P1/P2/P3). The two Jacobians at the SAME state, through DIFFERENT closures.
///
/// **THE LAST ASSERTION IS THE GATE THAT MATTERS.** Rung 73's `_reference` bug returned a perfect
/// confirmation having measured nothing; a coordinate port that silently did nothing would pass
/// every other assertion here.
///
/// The `1e-8` bar sits one decade above the measured `1.24e-9` **because the number is the
/// residual of a CENTRAL DIFFERENCE of step `1e-7` taken through two different closures** —
/// tightening it to the measured value would gate the differencing noise, not the invariance.
/// That reasoning is Python's comment at `tests/test_rung74.py:178-180`, and the bar is its
/// number, not the port's.
#[test]
fn the_spectrum_is_invariant_and_the_entries_are_not() {
    let d = demand_gains(&demand_default(), &flight(), LO, HI, TT4_MAX, PHI_ARREST, RDR_TAUS,
                         RDR_INC, RDR_R, RDR_SETTLE, DG_DS, RDR_V_MAX, DG_EVERY);
    assert!(d.n >= 20, "n = {}, skipped = {:?}", d.n, d.skipped);
    let poly_rel = d.worst_poly_rel.expect("n >= 20 interior rows, so the fold is not empty");
    assert!(poly_rel < 1e-8, "worst_poly_rel = {poly_rel}");
    let flip = d.worst_flip.expect("non-empty");
    assert!(flip < 1e-8, "worst_flip = {flip}");
    let keep = d.worst_keep.expect("non-empty");
    assert_eq!(keep, 0.0, "worst_keep = {keep}");
    let pairs = d.worst_pairs_gap.expect("non-empty");
    assert!(pairs < 1e-9, "worst_pairs_gap = {pairs}");
    // NOT A NO-OP: the coordinate really moved, and by a lot
    let signs = d.min_sign_changed.expect("non-empty");
    assert!(signs >= 4, "min_sign_changed = {signs}");
    let moved = d.biggest_moved.expect("non-empty");
    assert!(moved > 1.0, "biggest_moved = {moved}");
}

/// § 1 (anchor D3) — **the refutation, third running.** `n_live = 4` needed the masked leg to
/// reach the plant. `min()` is flat in its non-minimal argument exactly as `max()` was, so the
/// masked column is EXACTLY zero in BOTH coordinates and the block form survives.
///
/// `n >= 20` is asserted BEFORE the zero, and that ordering is the whole defence: an exact zero is
/// what a fold over an EMPTY set also produces, and step 4 § (a) measured this reader's folds
/// precisely because `None`-on-both-sides agrees perfectly and measures nothing.
#[test]
fn min_select_is_flat_in_the_masked_demand_too() {
    let d = demand_gains(&demand_default(), &flight(), LO, HI, TT4_MAX, PHI_ARREST, RDR_TAUS,
                         RDR_INC, RDR_R, RDR_SETTLE, DG_DS, RDR_V_MAX, DG_EVERY);
    assert!(d.n >= 20, "n = {}", d.n);
    let leak = d.worst_mask_leak.expect("the fold is over a non-empty set, which `n >= 20` says");
    assert_eq!(leak, 0.0, "worst_mask_leak = {leak}");
}

// ======================================================================================
// § 1.2 — THE FORCING, ISOLATED (open loop, one trajectory)
// ======================================================================================

/// § 1.2 — the rung's central number, and it is read OPEN LOOP because the closed-loop difference
/// cannot isolate it (§ 3 / anchor P6).
///
/// Along ONE trajectory, both lag laws integrated against their own targets:
/// `g_demand - g_clip -> mf_dot * tau` while the schedule moves, and -> 0 after it stops.
#[test]
fn the_forcing_is_the_schedules_slope_times_the_clock() {
    let d = forcing_openloop(&demand_rig(sm_of(PHI_BOTH), false, "demand", REF_LAW_DEFAULT),
                             &flight(), LO, HI, TT4_MAX, PHI_BOTH, RDR_TAUS, RDR_INC, RDR_R,
                             RDR_SETTLE, RDR_DS, RDR_V_MAX);
    assert!(d.n_on_ramp > 20 && d.n_post > 20,
            "n_on_ramp = {}, n_post = {}", d.n_on_ramp, d.n_post);
    let ratio = d.ratio_late.expect("the late half is non-empty, which `n_on_ramp > 20` says");
    assert!((ratio - 1.0).abs() < 0.05, "ratio_late = {ratio}");
    let worst = d.worst_rel_late.expect("non-empty");
    assert!(worst < 0.05, "worst_rel_late = {worst}");
    // and it DIES with the ramp -- a forcing, not a gain
    assert!(d.decayed.expect("n_post > 20"), "decayed = {:?}", d.decayed);
    let (first, last) = (d.delta_post_first.expect("n_post > 20"),
                         d.delta_post_last.expect("n_post > 20"));
    assert!(last.abs() < 1e-9 && 1e-9 < first.abs(),
            "delta_post_first = {first}, delta_post_last = {last}");
}

// ======================================================================================
// § 2 — THE BILL: the coordinate hands back the redline
// ======================================================================================

/// § 2 (anchor P4) — **the correction of rung 47.** Rung 47 shipped *the cost of realism is that a
/// lagged governor breaks the redline hold*. With the SAME clock and the SAME plant, read in the
/// coordinate a fuel control actually uses, it does not.
///
/// Run on the `phi_lim = 0.70` arm, where the surge leg is below the clip plant's own droop so
/// only the GOVERNOR is live — which is what makes this a statement about rung 47's leg and not
/// about rung 49's.
#[test]
fn the_demand_coordinate_holds_the_redline_the_clip_one_breaks() {
    let sm = sm_of(PHI_GOV);
    let clip = march(&demand_rig(sm, false, "clip", REF_LAW_DEFAULT), sm);
    let dem = march(&demand_rig(sm, false, "demand", REF_LAW_DEFAULT), sm);
    let over_clip = max_tt4(&clip) - TT4_MAX;
    let over_dem = max_tt4(&dem) - TT4_MAX;
    assert!(over_clip > 50.0, "over_clip = {over_clip}");
    assert!(over_dem <= 0.0, "over_dem = {over_dem}");
    assert!(over_clip - over_dem > 50.0, "over_clip = {over_clip}, over_dem = {over_dem}");
}

/// § 2, **THE ARREST ARM, and it is DISCLOSED rather than tuned away.** At `phi_lim = 0.80` the
/// surge cap equals the scheduled fuel at `s = 0`, so a leg that tracks its cap pins `phi` on the
/// floor and the accel never starts: `max Tt4 == Tt4_lo`, exactly.
///
/// **The whole accel in rungs 49–73 at this floor is powered by the clip coordinate's own tracking
/// error** — which is the strongest form of this rung's claim.
#[test]
fn at_the_inherited_floor_the_demand_plant_does_not_accelerate() {
    let dem = march(&demand_rig(SM, false, "demand", REF_LAW_DEFAULT), SM);
    let clip = march(&demand_rig(SM, false, "clip", REF_LAW_DEFAULT), SM);
    assert!((max_tt4(&dem) - LO).abs() < 1e-6, "max Tt4 (demand) = {}", max_tt4(&dem));
    assert!(max_tt4(&clip) - LO > 200.0, "max Tt4 (clip) = {}", max_tt4(&clip));
    // and it is held ON the floor, not below it -- the leg is tracking, not failing
    assert!((min_phi_lp(&dem) - PHI).abs() < 1e-6, "min phi_lp (demand) = {}", min_phi_lp(&dem));
    assert!(min_phi_lp(&clip) < PHI - 1e-3, "min phi_lp (clip) = {}", min_phi_lp(&clip));
}

// ======================================================================================
// § 2.2 CORRECTED IN SCOPE — the arrest is an INTERVAL (docs/rung74-arrest-interval.md)
// ======================================================================================

/// The FREE operating point: no floor armed anywhere, no fuel-side leg, no governor. **Read rather
/// than hardcoded**, because it is the arrest's own lower edge — which is exactly what makes
/// gate 9 a measurement and not a restatement of a typed bracket.
fn free_phi() -> f64 {
    let m = full_of(build_demand_coordinate_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &LeverArm::default()));
    let leg = StatorLeg { accel: None::<&AccelSchedule>, surge: None, tt4_max: None };
    let ramp = Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds: DS };
    m.stator_march_scoped(&flight(), &ramp, None, &leg, &MarchScope::DEFAULT).0[0].phi_lp
}

/// § 2.2 CORRECTED IN SCOPE — **the arrest is not the cell `phi_lim = 0.80`.**
///
/// MECHANISM: an airflow floor above the free operating point LIFTS `phi(0)` exactly onto its
/// wall, so rung 49's leg opens ON its own floor with no authority left and the accel never
/// starts. Below that point no lift is needed and the leg opens with margin.
///
/// The bracket `(0.7731, 0.7732)`, not an absolute value, is what is gated: pinning the digits
/// here would duplicate the fingerprint gate's job on a number that is allowed to move with the
/// map. `free_phi` reaches a SHALLOWER march than the assertions below — that is deliberate, it is
/// the FREE plant.
#[test]
fn the_arrest_is_an_interval_whose_lower_edge_is_the_free_operating_point() {
    let free = free_phi();
    let below = march(&demand_rig(sm_of(0.7731), false, "demand", REF_LAW_DEFAULT), sm_of(0.7731));
    let above = march(&demand_rig(sm_of(0.7732), false, "demand", REF_LAW_DEFAULT), sm_of(0.7732));
    assert!(0.7731 < free && free < 0.7732, "free = {free}");
    // BELOW the free point: the leg opens with margin and the plant accelerates.
    assert!(max_tt4(&below) - LO > 1.0, "max Tt4 (below) = {}", max_tt4(&below));
    // ABOVE it: lifted onto the wall, zero margin, and the accel never starts.
    assert!((max_tt4(&above) - LO).abs() < 1e-6, "max Tt4 (above) = {}", max_tt4(&above));
    assert!((above[0].phi_lp - 0.7732).abs() < 1e-9, "above[0].phi_lp = {}", above[0].phi_lp);
    // ... and the lift is what did it -- below the free point NO floor has moved at s = 0.
    assert!(b_of(&below[0]).abs() < 1e-12 && v_of(&below[0]).abs() < 1e-12,
            "below[0]: b = {}, v = {}", b_of(&below[0]), v_of(&below[0]));
}

/// The two CONTROLS that make the bracket above a statement rather than a coincidence.
///
/// **THE COORDINATE:** at the bracket's own wall the `clip` plant marches ~280 K. Without this the
/// bracket would also pass on a rig that was broken for any other reason at 0.7732.
///
/// **THE UPPER EDGE:** the arrest ends where the lifting lever RUNS OUT — `b/b_max` is 0.987 at
/// the last arrested wall and exactly 1.000 at the first non-arrested one. It is read off the
/// hardware, not chosen. And the plant does not RECOVER there: past saturation `max Tt4` falls
/// BELOW `Tt4_lo`.
#[test]
fn the_arrest_is_the_demand_coordinates_and_it_ends_at_the_valves_saturation() {
    let clip = march(&demand_rig(sm_of(0.7732), false, "clip", REF_LAW_DEFAULT), sm_of(0.7732));
    assert!(max_tt4(&clip) - LO > 200.0, "max Tt4 (clip at the wall) = {}", max_tt4(&clip));

    let lo_w = march(&demand_rig(sm_of(0.850), false, "demand", REF_LAW_DEFAULT), sm_of(0.850));
    let hi_w = march(&demand_rig(sm_of(0.855), false, "demand", REF_LAW_DEFAULT), sm_of(0.855));
    assert!((max_tt4(&lo_w) - LO).abs() < 1e-6, "max Tt4 (lo_w) = {}", max_tt4(&lo_w));
    // NOT saturated -> still arrested
    assert!(b_of(&lo_w[0]) < B - 1e-9, "lo_w[0].b = {}", b_of(&lo_w[0]));
    // saturated -> the arrest ends
    assert!((b_of(&hi_w[0]) - B).abs() < 1e-9, "hi_w[0].b = {}", b_of(&hi_w[0]));
    assert!(max_tt4(&hi_w) < LO - 0.5, "max Tt4 (hi_w) = {}", max_tt4(&hi_w));
}

// ======================================================================================
// § 4 — THE STOP WAS DOING THE ANTI-WINDUP (rung 73 § 0.2, CORRECTED)
// ======================================================================================

/// § 4 — the correction of rung 73 § 0.2, which reads *an applied-referenced leg is
/// self-anti-winding under min-select — that is a property of the composition*.
///
/// The MOTION is a property of the composition and reproduces here. **Where it STOPS is not:** in
/// clip coordinates the leg runs INTO the floor at `g = 0`; in demand coordinates the same motion
/// has nothing in its path, and the joint IC has no interior fixed point at all.
///
/// The evidence is the pair: WITH the stop the masked leg parks at EXACTLY the stop
/// (`w/mf_sched == 1.0`), WITHOUT it there is no plant.
#[test]
fn a_masked_applied_referenced_leg_has_no_equilibrium_without_a_stop() {
    let d = windup_law(&demand_rig(sm_of(PHI_BOTH), false, "demand", REF_LAW_DEFAULT), &flight(),
                       LO, HI, TT4_MAX, PHI_BOTH, RDR_TAUS, RDR_INC, RDR_R, RDR_SETTLE, RDR_DS,
                       RDR_V_MAX);
    assert!(d.no_equilibrium_without_a_stop, "cells = {:?}", d.cells);
    assert!(d.both_sched_exist, "cells = {:?}", d.cells);
    // `d["cells"]["demand-latched|applied"]` — index 3 in Python's own insertion order.
    let latched = match &d.cells[3] {
        WindupCell::Present(v) => v,
        WindupCell::Absent { why } => panic!("`demand-latched|applied` must EXIST: {why}"),
    };
    let over = latched.max_masked_over_sched
        .expect("the cell exists, so something held the actuator somewhere");
    assert!((over - 1.0).abs() < 1e-12, "max_masked_over_sched = {over}");
    // and the UNLATCHED, scheduled-reference leg keeps the headroom the clip floor erases
    let free = match &d.cells[0] {
        WindupCell::Present(v) => v,
        WindupCell::Absent { why } => panic!("`demand|sched` must EXIST: {why}"),
    };
    let free_over = free.max_masked_over_sched.expect("the cell exists");
    assert!(free_over > 1.05, "demand|sched max_masked_over_sched = {free_over}");
}

// ======================================================================================
// THE DECLARED KNOB, AND THE PORT'S ONE INVERTIBLE DETAIL
// ======================================================================================

/// Anchor § 0.4 / P8 — **the one line of this port that would have inverted silently.**
///
/// Attack in clip coordinates is `required > g`; in demand coordinates it is `cap < w`. A port
/// that kept rung 52's argument order would select `tau_rel` on ATTACK — a 3x clock error in the
/// direction that SLOWS protection, which would have read as a finding and passed every other gate
/// here.
#[test]
fn the_lag_returns_attack_on_a_known_attack_point() {
    let lg = lag();
    // the leg wants to CUT: its cap is BELOW what it is currently allowing
    assert_eq!(demand_tau(&lg, 0.9, 1.0), TAU_ATT);
    // and it is handing fuel back
    assert_eq!(demand_tau(&lg, 1.1, 1.0), TAU_REL);
    // the clip-coordinate law it must agree with, at the MIRRORED arguments
    assert_eq!(lg.tau(0.1, 0.0), TAU_ATT);
    assert_eq!(lg.tau(0.0, 0.1), TAU_REL);
}

/// Three knobs now (`_share_law`, `_ref_law`, `_lag_coord`), and an undeclared one must REFUSE
/// rather than pick a plant.
///
/// The needle `"DECLARED"` is Python's `match=` at `tests/test_rung74.py:387`, and the message it
/// matches is `turbojet/engine.py:17762`'s literal — transcribed from THERE, not from
/// `demand_coordinate.rs`, for the module header's reason.
#[test]
fn the_coordinate_is_declared() {
    let m = demand_rig(SM, false, "demand", REF_LAW_DEFAULT);
    m.fuel.inner.lag_coord.set("wishful");
    let msg = message_of(|| { march(&m, SM); });
    assert!(msg.contains("DECLARED"), "the refusal says the coordinate is declared: {msg:?}");
}

/// `min(mf_sched, wf, wr)` has no `sum` reading that keeps the schedule as an input, so marching it
/// would swap two declared laws at once — rung 73's refusal of `applied x sum`, inherited in its
/// reasoning.
///
/// The needle `"two declared laws"` is Python's `match=`, and the message it matches is
/// `turbojet/engine.py:17776`'s literal.
#[test]
fn demand_refuses_the_sum_composition() {
    let m = demand_rig(SM, false, "demand", REF_LAW_DEFAULT);
    m.fuel.inner.share_law.set("sum");
    let msg = message_of(|| { march(&m, SM); });
    assert!(msg.contains("two declared laws"),
            "the DEMAND x SUM conjunction is refused by name: {msg:?}");
}

/// **THE TWELFTH INSTANCE of the rung-61..73 trap**: a sibling machine that drops a knob reports
/// this rung while marching another.
#[test]
fn at_lever_carries_all_three_knobs() {
    let m = demand_rig(SM, false, "demand-latched", REF_LAW_APPLIED);
    let n = m.at_lever(&suite_arm(SM, false));
    // `isinstance(n, DemandCoordinateTransient)` — the type is structural here, so the check is
    // that the sibling still dispatches through rung 74's OWN cells. Compared against the shipped
    // `R74_TRIPLE`, never against the receiver's own table: `m`'s table is what built `n`, so
    // `n.table == m.table` would be § 5.30 (viii) item 1's defect in one line.
    assert!(fn_addr_eq(n.triple_hooks().with_coord, R74_TRIPLE.with_coord),
            "at_lever must hand back a RUNG-74 machine, not a parent");
    assert!(fn_addr_eq(n.fuel.hooks.integrate_fuel, R74_FUEL.integrate_fuel),
            "and this rung's fuel table, so its refusals are armed on the sibling too");
    assert_eq!(
        (n.fuel.inner.lag_coord.get(), n.fuel.inner.ref_law.get(), n.fuel.inner.share_law.get()),
        ("demand-latched", "applied", "max"));
}

/// `_cap_free` must return the FAMILY's own number wherever the family has ever consulted a cap —
/// it only searches upward in the SLACK regime, which is the regime the shipped closures
/// short-circuit. Otherwise this rung would quietly re-bracket rungs 46–52.
///
/// **The probe point is taken off a MARCH rather than guessed**, and `shipped < mf` is asserted
/// before the equality: without it the gate would compare two functions that both short-circuit,
/// which is § 5.30 (viii) item 1's defect exactly.
#[test]
fn the_unfloored_cap_is_the_shipped_one_wherever_the_leg_binds() {
    let m = demand_default();
    let floor = Floor::Phi(surge_of(SM));
    // a REAL binding state, taken off a march rather than guessed: the clip plant's own
    // trajectory, at a point where rung 52's leg is riding
    let traj = march(&demand_rig(SM, false, "clip", REF_LAW_DEFAULT), SM);
    let p = traj.iter().find(|p| required_fuel_of(p) > 1e-6)
        .expect("the clip march must reach a point where the fuel leg is riding");
    let (a, h, mf) = (p.nu_lp, p.nu_hp, p.mf_sched);
    let shipped = m.fuel.try_surge_fuel(&flight(), a, h, mf, &floor)
        .expect("the shipped surge solve must not abort at a riding point");
    assert!(shipped < mf, "the probe point must BIND for this gate to mean anything: \
                           shipped = {shipped}, mf_sched = {mf}");
    let got = (m.fuel.inner.triple_hooks.cap_fuel)(&m.fuel, &flight(), a, h, mf, None,
                                                   Some(&floor), None)
        .expect("`_cap_fuel` must not abort where `_surge_fuel` did not");
    assert_eq!(got.to_bits(), shipped.to_bits(),
               "the unfloored cap IS the shipped one where the leg binds: {got} vs {shipped}");
}

// ======================================================================================
// THE ADDED GATE — § 3's ISOLATION INSTRUMENT, WHICH THE PYTHON SUITE NEVER CALLS
// ======================================================================================

/// § 3 — **`latched - clip` is the COORDINATE and `demand - latched` is the FLOOR's ADDRESS**, and
/// `latch_discriminator` is the reader that splits them.
///
/// **DECLARED ADDED.** Nothing in the shipped tree calls this reader (§ 5.30.4 (h) measured it:
/// the name occurs in `turbojet/engine.py` and the port plan and nowhere else), so there is no
/// Python test to mirror and the 1:1 map above records the addition rather than hiding it.
///
/// **WHAT SUPPLIES THE BARS.** `docs/rung74-spec.md` § 3's own sentence:
///
/// > Measured `floor_dTt4 = 65.2 K` with 332 of 341 points riding
///
/// transcribed from the SPEC, which was written from the PYTHON reader long before this module
/// existed. It is deliberately NOT taken from step 4's drive output — a number the port produced
/// is not a bar on the port, which is § 5.30 (viii) item 1 in one line. The `65.2` is quoted to
/// three significant figures there, so the bar is a rounding bracket rather than an equality; the
/// riding count is an integer and is asserted exactly.
///
/// Anchor **P6** predicted `demand - latched` would be *zero to machine precision wherever both
/// legs are riding* and was **REFUTED**: that is a property of the LAW, and the reader compares
/// TRAJECTORIES. The gate asserts the refutation's direction (the gap is LARGE), so a port that
/// accidentally made the two arms agree would fail here rather than look tidy.
#[test]
fn the_isolation_instrument_splits_the_rung_at_the_specs_own_numbers() {
    let d = latch_discriminator(&demand_rig(sm_of(PHI_BOTH), false, "demand", REF_LAW_DEFAULT),
                                &flight(), LO, HI, TT4_MAX, PHI_BOTH, RDR_TAUS, RDR_INC, RDR_R,
                                RDR_SETTLE, RDR_DS, RDR_V_MAX);
    // `332 of 341` — both integers, both asserted exactly.
    assert_eq!(d.n, 341, "the shipped grid is 341 points: n = {}", d.n);
    assert_eq!(d.n_both_riding, 332, "n_both_riding = {}", d.n_both_riding);
    // `floor_dTt4 = 65.2 K`, quoted to three figures, so the bar is the rounding bracket.
    assert!((65.15..65.25).contains(&d.floor_dtt4),
            "floor_dTt4 = {} K, and the spec's § 3 says 65.2", d.floor_dtt4);
    // P6's REFUTATION, in the direction it was refuted: the floor half is NOT machine zero on the
    // riding subset. A port that made the two arms agree would pass every bar above and fail here.
    let riding = d.floor_dg_riding
        .expect("332 of 341 points ride, so this fold is over a nearly-full set");
    assert!(riding > 1e-9,
            "anchor P6 predicted ~0 here and was REFUTED: floor_dg_riding = {riding}");
    // and the COORDINATE half is live too, which is what makes the split a split.
    assert!(d.coord_dtt4.abs() > 1.0, "coord_dTt4 = {}", d.coord_dtt4);
}
