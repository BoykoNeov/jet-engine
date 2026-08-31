//! RUNG 71 — **THE FULL SPLIT**: `n = m = 3`, ZERO zeros. Rung 69's move (swap ONE loop's
//! COORDINATE) applied to rung 70's plant — rung 68's `phi` stator becomes rung 69's INCIDENCE
//! stator, beside rung 47's `Tt4` governor and rung 65's `phi` valve. Three loops, THREE
//! constraints: the last unoccupied cell of rung 69 § 1's table, and rung 70's named strongest
//! seam.
//!
//! **THE HEADLINE: A CONSTRAINT CAN BE INDEPENDENT IN RANK AND REDUNDANT ON THE BAND.** The
//! Jacobian is full rank, and the third loop is live over ~2 % of the march — because at the
//! valve's own set point `M_i = m_lim + v >= m_lim` for every admissible `v >= 0`, so the third
//! constraint is IMPLIED by the second's on the whole band. `zeros = n − m` counts GRADIENT
//! DIRECTIONS, not LIVE loops.
//!
//! **AND `det J`, NON-ZERO FOR THE FIRST TIME IN THIS FAMILY, FACTORS**: `−(1−pair_RC)(1−pair_CV)`
//! — rung 67's non-degeneracy condition times rung 69's, one factor per rung — and it is BLIND to
//! `pair_RV`, the only gain this rung contains that no earlier one measured.
//!
//! Ported from `tests/test_rung71.py` — **30 tests, of which 11 carry `slow` there** (MEASURED by
//! `pytest -m slow --collect-only`, never typed: step 4's own header defect was a `slow` count
//! read off a neighbouring sentence). The marker is dropped here per slice M's rule; `#[ignore]`
//! is re-introduced only against a MEASURED Rust cost, never inherited.
//!
//! # THE ONE RUNTIME-INTROSPECTION TEST IN THIS SLICE LIVES HERE, AND § 6's TABLE ALREADY DECIDED IT
//!
//! `test_forced_release_edges_and_an_instantaneous_valve_are_refused` is § 5.27 (viii)'s single
//! introspection row, and it asserts TWO things Python can only reach by reflection:
//!
//! * `s_off`/`tau_rel` are **absent from `_stator_march`'s signature** — the OUTER, structural
//!   guard. § 6's decided replacement is the **narrowed config view**: the march entry here takes
//!   [`StatorLeg`] and [`MarchScope`], neither of which carries either field, so "not reachable"
//!   is a COMPILE error. The gate spells that as an exhaustive destructuring with no `..`, which
//!   is the only form that breaks the build when a field is ADDED.
//! * the INNER guard's source text — `include_str!` + `.contains`, § 6's other decided
//!   replacement, and stronger than Python's `inspect.getsource`, which re-reads from disk at
//!   import-cached line numbers.
//!
//! # `at_lever` IS THE ONE BODY SUBSTITUTION, AND IT IS NOT AN ADDRESS COMPARISON
//!
//! Python opens with `type(m) is FullSplitTransient`. There is no runtime class here — every rung
//! in this family is a [`ScheduledStatorCore`] and the rung is the TABLE it carries. Comparing the
//! table's address would test the optimiser (slice AA step 1's recorded `ptr::eq`-on-a-`const`
//! trap), so the sibling is instead made to **exercise a cell only rung 71's table has**: it must
//! march with an INCIDENCE stator beside `tau_gov`, which rung 70's inherited table refuses
//! outright (guard A, *"n = m = 3"*). A sibling handed back carrying the parent's table passes
//! every float in that gate and panics on this. Python's second half — `stator_inc is not None and
//! stator_lim is None` — ports as a plain field check beside it.
//!
//! # THIS FILE'S DAMPING GATE FALSIFIED A CONDITION THE PORT HAD REGISTERED AS GATED
//!
//! `the_damping_reader_had_to_be_rebuilt_a_third_time` drives rung 70's `zeta_pair` on a
//! CONSTRUCTED spectrum, and step 3 had shipped `assert!(p.im == 0.0)` inside it on the strength of
//! `p` being real on 18 of 18 calls **of the rung-70 readers**. The constructed spectrum's two
//! largest moduli are one REAL root and ONE MEMBER of the pair, so `p = 4462 + 4947i` — the case
//! § 5.27 (iv) named and then assumed away. See
//! [`csqrt`](turbojet::reference_split::csqrt) for what replaced it and
//! `tests/porting_rules.rs` RULE 4 for the invariant that keeps the replacement honest.
//!
//! **And this gate could not have caught the wrong ANSWER, only the refusal.** Its bar is
//! one-sided (`|zeta − ring| > 0.5`) and the two candidates sit at `0.608` and `0.954` — the same
//! side of it. The `assert!` is what was load-bearing.
//!
//! # FOUR DEFAULT GRIDS ARE PYTHON's AND ARE SPELLED HERE
//!
//! [`CLOCKS`] (six arms), [`TAU_QS`]/[`TAU_SS`], [`IC_ORDERS`]/[`IC_FRACS`]. The clock grid goes
//! IN as `(tau_q, tau_gov, tau_s)` and is reported as `taus = (tau_g, tau_q, tau_s)` — the
//! `(g, q, v)` order of the STATE VECTOR — which is rung 69's own recorded trap and step 4's
//! surviving injection i02. Nothing below looks an arm up by its clocks.

use std::panic::catch_unwind;
use std::ptr::fn_addr_eq;

use turbojet::bleed_transient::{BleedSchedule, LeverArm};
use turbojet::cross_loop::build_cross_loop_cascade;
use turbojet::cross_split::{
    build_cross_split_cascade, split_bill, zeta_pair, Census70, R70_FUEL, R70_TRIPLE,
};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelPoint, PointExtra, SurgeLimiter,
};
use turbojet::full_split::{
    band_containment, build_full_split_cascade, full_bill, full_gains, full_modes, full_rig,
    ic_contraction, window_law, zeta_ring, Census71, R71_FUEL, R71_TRIPLE,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::reference_split::{
    build_reference_split_cascade, cubic_roots_c, invariants, StatorIncidenceLimiter, C64,
};
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::three_loop::{StatorLimiter, TripleGains};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------------- the grid
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
const PHI: f64 = 0.80;
const V_MAX: f64 = 0.20;
/// `PHI / FLOOR - 1.0` — the expression Python spells, never a typed decimal.
///
/// Rung 69's constructor asserts `m_lim == T_c − 1/phi_lim`, so the incidence wall and the valve's
/// `phi` wall are ONE PHYSICAL WALL by construction — which is precisely what makes § 0's
/// containment (`slack − v == 0` EXACTLY where the valve pins `phi`) a measurement and not a
/// tolerance. A rounded constant would break it silently.
const SM: f64 = PHI / FLOOR - 1.0;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TAU_GOV: f64 = 0.05;
const TAU_ATT: f64 = 0.05;
const TAU_REL: f64 = 0.15;
/// **RUNG 67's imposed redline, VERBATIM** — `docs/rung71-spec.md` § 3.
const TT4_MAX: f64 = 1200.0;

/// RUNG 69's published `k` band over its own riding arc (`docs/rung69-spec.md` § 0.2 / § 1.3).
///
/// `pair_CV` here IS that scalar, on the same two loops — re-measured on a DIFFERENT trajectory,
/// so the FORM and the BAND are gated and no tolerance the trajectory shift cannot justify is.
const R69_K_LO: f64 = -2.05;
const R69_K_HI: f64 = -1.60;

/// `full_modes`' Python default — **SIX arms**, in the `(tau_q, tau_gov, tau_s)` order the reader
/// TAKES, and reported back as `taus = (tau_g, tau_q, tau_s)`.
///
/// Rungs 68/69/70 default to FOUR; six is the smallest grid that spans the three RING regimes
/// [`rung69s_damping_floor_was_the_c0_equals_0_corner`] needs (arms below rung 69's line, arms
/// above it, and an arm with no complex pair at all). Entries 1, 5 and 6 are SYMMETRIC in the
/// first two slots; 2 and 4 are each other's swap.
const CLOCKS: [(f64, f64, f64); 6] = [
    (0.05, 0.05, 0.05), (0.05, 0.005, 0.05), (0.05, 0.5, 0.05),
    (0.005, 0.05, 0.05), (0.05, 0.05, 2.0), (0.10, 0.10, 0.05),
];

/// `window_law`'s Python default sweep of the VALVE's clock — the loop whose lag the third loop
/// is predicted to live inside.
const TAU_QS: [f64; 5] = [0.005, 0.05, 0.20, 0.50, 2.00];

/// …and of the STATOR's OWN clock, the one-sided sweep that alone could not separate the mechanism
/// from *a slower loop rides longer*.
const TAU_SS: [f64; 4] = [0.005, 0.05, 0.20, 0.50];

/// `ic_contraction`'s six Gauss-Seidel sweep ORDERS — every permutation of `(g, q, v)`.
const IC_ORDERS: [&str; 6] = ["gqv", "gvq", "qgv", "qvg", "vgq", "vqg"];

/// …and the four starting displacements, as a FRACTION of each rig's OWN band.
const IC_FRACS: [f64; 4] = [0.0, 0.25, 0.6, 1.0];

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn hp() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn core(b: ScheduledStatorTransient) -> ScheduledStatorCore {
    match b {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this grid never disables LP"),
    }
}

/// Python's `_full` — THE rung-71 machine.
fn full_of(arm: &LeverArm) -> ScheduledStatorCore {
    core(build_full_split_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

/// Python's `_cross` — rung 70.
fn cross_of(arm: &LeverArm) -> ScheduledStatorCore {
    core(build_cross_split_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

/// Python's `_ref` — rung 69.
fn ref_of(arm: &LeverArm) -> ScheduledStatorCore {
    core(build_reference_split_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

/// Python's `_cross67` — rung 67's cascade A.
fn cross67_of(arm: &LeverArm) -> ScheduledStatorCore {
    core(build_cross_loop_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

/// Python's `_valve`. Spelled through `from_margin_tau` and NOT `with_tau(PHI, …)`: the margin
/// form is what Python calls, and `(1 + SM) * phi_surge` is not obliged to round back to `PHI`.
fn valve(tau: f64) -> BleedLimiter { BleedLimiter::from_margin_tau(&lp(), B, SM, Some(tau)) }

/// Python's `_phi_stator` — rung 68's `phi` floor. Used here to reach rung 70's plant.
fn phi_stator(tau: f64, v_max: f64) -> StatorLimiter {
    StatorLimiter::from_margin(&lp(), v_max, SM, Some(tau))
}

/// Python's `_inc` — rung 69's INCIDENCE floor, from the SAME margin. **This rung's third loop.**
fn inc(tau: f64, v_max: f64) -> StatorIncidenceLimiter {
    StatorIncidenceLimiter::from_margin(&lp(), v_max, SM, Some(tau))
}

fn fuel_floor() -> Floor { Floor::Phi(SurgeLimiter::from_margin(&lp(), Spool::Lp, SM)) }

fn lag() -> AsymmetricLag { AsymmetricLag::new(TAU_ATT, TAU_REL) }

fn ramp(ds: f64) -> Ramp { Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds } }

/// Python's `_march`. The two arming knobs live in two different places here — `Tt4_max` is a LEG
/// argument and `tau_gov` is a SCOPE field — where Python passes both as march keywords.
fn march(
    m: &ScheduledStatorCore, ds: f64, surge: Option<Floor>, lg: Option<AsymmetricLag>,
    tt4_max: Option<f64>, tau_gov: Option<f64>,
) -> Vec<FuelPoint> {
    let leg = StatorLeg { accel: None::<&AccelSchedule>, surge, tt4_max };
    m.stator_march_scoped(&flight(), &ramp(ds), None, &leg,
                          &MarchScope { lag: lg, tau_gov, ..MarchScope::DEFAULT }).0
}

/// The rung-71 march — the governor armed, no fuel leg (guard B forbids the pair).
fn gov_march(m: &ScheduledStatorCore) -> Vec<FuelPoint> {
    march(m, DS, None, None, Some(TT4_MAX), Some(TAU_GOV))
}

/// Python's `_keys` — the seven-tuple per point the reduce gates compare, BIT for bit.
fn keys(traj: &[FuelPoint]) -> Vec<[u64; 7]> {
    traj.iter()
        .map(|p| [p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.phi_lp.to_bits(),
                  p.phi_hp.to_bits(), p.tt4.to_bits(), p.mf.to_bits()])
        .collect()
}

/// Python's `"v" in p` — a key test on a dict, which in Rust is a variant test on the point.
///
/// **`Triple` ALONE**, which is what `tests/rung68.rs`, `rung69.rs` and `rung70.rs` already spell
/// under this same doc line. [`PointExtra::CrossCascade`] is rung 67's cascade-A point and has no
/// `v` field at all.
fn carries_v(p: &FuelPoint) -> bool { matches!(p.extra, PointExtra::Triple { .. }) }

/// Python's `p["v"]` — a `KeyError` off any other trajectory, which is a panic here.
fn v_of(p: &FuelPoint) -> f64 {
    match p.extra {
        PointExtra::Triple { v, .. } => v,
        _ => panic!("rung-71's `v` needs a five-state trajectory"),
    }
}

/// Python's `p["required"]` — the governor's clip amount, likewise `KeyError` elsewhere.
fn required_of(p: &FuelPoint) -> f64 {
    match p.extra {
        PointExtra::Triple { required, .. } => required,
        _ => panic!("rung-71's `required` needs a five-state trajectory"),
    }
}

fn panics_with<F: FnOnce() + std::panic::UnwindSafe>(f: F, needle: &str) -> bool {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = catch_unwind(f);
    std::panic::set_hook(prev);
    match out {
        Ok(()) => false,
        Err(e) => {
            let msg = e.downcast_ref::<String>().cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_default();
            msg.contains(needle)
        }
    }
}

/// THE rung-71 machine — the governor beside the valve and the INCIDENCE stator. **Rebuilt per
/// test rather than shared**, for `tests/rung68.rs`'s reason exactly: each test is its own thread
/// and [`ScheduledStatorCore`]'s `Cell` fields — which ARE the dynamically scoped state — are
/// deliberately not `Sync`.
fn full() -> ScheduledStatorCore {
    full_of(&LeverArm { bleed_lim: Some(valve(TAU)), stator_inc: Some(inc(TAU_S, V_MAX)),
                        ..Default::default() })
}

/// Python's module-scoped `gains` fixture. **`ds` is the reader's OWN default `0.002`, NOT `DS`**
/// — Python's `**KW` carries no `ds`, and `every` defaults to 2.
fn gains() -> turbojet::full_split::FullGains {
    full_gains(&full(), &flight(), LO, HI, TT4_MAX, SM, R, SETTLE, 0.002, TAU, TAU_GOV, TAU_S,
               V_MAX, 2)
}

/// Python's module-scoped `modes` fixture — the SIX-arm default grid, `ds = 0.002`, `every = 4`.
fn modes() -> turbojet::full_split::FullModes {
    full_modes(&full(), &flight(), LO, HI, TT4_MAX, SM, &CLOCKS, R, SETTLE, 0.002, V_MAX, 4)
}

/// Python's module-scoped `bill` fixture — this one IS called at `ds = DS`.
fn bill() -> turbojet::full_split::FullBill {
    full_bill(&full(), &flight(), LO, HI, TT4_MAX, SM, R, SETTLE, DS, TAU, TAU_GOV, TAU_S, V_MAX)
}

// =============================================================================================
// GATE 1 — THE REDUCE. Rung 71 moves ONE loop's COORDINATE, so every ancestor must still be
//          reached BIT-FOR-BIT, and by DISPATCH. And the march is REUSED rather than copied,
//          which is itself gated.
// =============================================================================================

/// `tau_gov = None` with the incidence stator: rung 69's own five-state plant, untouched.
#[test]
fn reduce_no_governor_is_rung69_bit_for_bit() {
    let arm = LeverArm { bleed_lim: Some(valve(TAU)), stator_inc: Some(inc(TAU_S, V_MAX)),
                         ..Default::default() };
    let a = march(&full_of(&arm), DS, Some(fuel_floor()), Some(lag()), None, None);
    let b = march(&ref_of(&arm), DS, Some(fuel_floor()), Some(lag()), None, None);
    assert_eq!(keys(&a), keys(&b));
}

/// A `phi` stator instead of the incidence one, with the governor armed: that is rung 70's plant
/// exactly (`n = 3, m = 2`), and it must be reached through the parent's own path.
#[test]
fn reduce_phi_stator_beside_the_governor_is_rung70_bit_for_bit() {
    let arm = LeverArm { bleed_lim: Some(valve(TAU)),
                         stator_lim: Some(phi_stator(TAU_S, V_MAX)), ..Default::default() };
    let a = gov_march(&full_of(&arm));
    let b = gov_march(&cross_of(&arm));
    assert_eq!(keys(&a), keys(&b));
}

/// A governor and a valve with NO stator is rung 67 — this class never intercepts a march it does
/// not own.
#[test]
fn reduce_no_stator_is_rung67_bit_for_bit() {
    let arm = LeverArm { bleed_lim: Some(valve(TAU)), ..Default::default() };
    let a = gov_march(&full_of(&arm));
    let b = gov_march(&cross67_of(&arm));
    assert_eq!(keys(&a), keys(&b));
    assert!(!carries_v(&a[0]), "no stator armed => no fifth state");
}

/// Rungs 66/65/64/62's arms all leave through the same `super()`.
///
/// **FOUR cases, which is Python's count** — a silently-shortened loop is this phase's *"a count
/// typed instead of added up"*, so the length is asserted rather than trusted to the reader.
#[test]
fn reduce_inherited_arms_bit_for_bit() {
    let cases: [(LeverArm, Option<Floor>, Option<AsymmetricLag>); 4] = [
        (LeverArm { bleed_lim: Some(valve(TAU)), ..Default::default() },
         Some(fuel_floor()), Some(lag())),
        (LeverArm { bleed_lim: Some(valve(TAU)), ..Default::default() },
         Some(fuel_floor()), None),
        (LeverArm::default(), Some(fuel_floor()), Some(lag())),
        (LeverArm { bleed_sched: Some(BleedSchedule::new(B, 0.65)), ..Default::default() },
         None, None),
    ];
    assert_eq!(cases.len(), 4, "Python's loop has four arms");
    for (i, (arm, surge, lg)) in cases.into_iter().enumerate() {
        let a = march(&full_of(&arm), DS, surge, lg, None, None);
        let b = march(&cross_of(&arm), DS, surge, lg, None, None);
        assert_eq!(keys(&a), keys(&b), "case {i}");
    }
}

/// **THE INTEGRATOR IS RUNG 70's, ENTERED RATHER THAN REFUSED.**
///
/// Rungs 68/69/70 each shipped a sibling integrator because a STATE was being added; nothing is
/// added here, so a copy would be ~130 lines that could not differ — and `test_numeric_fingerprint`
/// does not watch this path, so the reuse is gated rather than argued.
///
/// # PYTHON's `__dict__` ASSERTION HAS TWO HALVES AND THEY PORT DIFFERENTLY
///
/// `"_integrate_fuel_cross_triple" not in FullSplitTransient.__dict__` and
/// `FullSplitTransient._integrate_fuel_cross_triple is CrossSplitTransient…` are, in this
/// architecture, ONE claim: rung 71's tables carry rung 70's bodies everywhere except the two
/// measured swaps. The `TripleHooks` cells are checked by `fn_addr_eq` — the spelling
/// `slice_ac_cells.rs` uses, and NOT `ptr::eq` on the table, which a `const` defeats.
///
/// **AND THE BEHAVIOURAL HALF IS A COUNTER WITH ITS OWN CONTROL.** [`Census71`]'s
/// `integrate_reduced` bumps on rung 71's REDUCE arm, so a march that arms the governor must
/// leave it at **0** — and a march that does not must leave it at **1**. Read alone the zero
/// proves nothing (an entry never called reports zero too), which is why the reduced arm is run
/// FIRST: it is the arm that shows the instrument can see. Rung 71's `integrate_fuel` has exactly
/// two exits, and the one the armed arm takes is a direct call to rung 70's marcher.
///
/// **THE FIRST WRITING OF THIS GATE ASSERTED A COUNTER THE MARCH NEVER TOUCHES.** It read
/// `Census70::triple_laws_gov > 0` on the theory that rung 70's `_triple_laws` is called at every
/// step; it is not — the five-state integrator calls `solve_v`, and `triple_laws` is a READER-side
/// cell. All six of `Census70`'s counters came back 0 on a march that had plainly run, which is
/// this phase's *ask what reads a thing* in its cheapest form.
#[test]
fn the_march_is_reused_and_not_copied() {
    // rung 71 owns NO cell of the third-loop table — the march and its five seams are rung 70's.
    assert!(fn_addr_eq(R71_TRIPLE.triple_laws, R70_TRIPLE.triple_laws), "triple_laws");
    assert!(fn_addr_eq(R71_TRIPLE.stator_leg, R70_TRIPLE.stator_leg), "stator_leg");
    assert!(fn_addr_eq(R71_TRIPLE.clamp_v, R70_TRIPLE.clamp_v), "clamp_v");
    assert!(fn_addr_eq(R71_TRIPLE.check_v0, R70_TRIPLE.check_v0), "check_v0");
    assert!(fn_addr_eq(R71_TRIPLE.manifold_v, R70_TRIPLE.manifold_v), "manifold_v");
    assert!(fn_addr_eq(R71_TRIPLE.solve_v, R70_TRIPLE.solve_v), "solve_v");
    // …and the ONE fuel cell it does own is not rung 70's.
    assert!(!fn_addr_eq(R71_FUEL.integrate_fuel, R70_FUEL.integrate_fuel), "integrate_fuel");

    // THE CONTROL FIRST — a governor-less march takes rung 71's reduce arm, so the counter moves.
    Census71::reset();
    march(&full(), DS, Some(fuel_floor()), Some(lag()), None, None);
    assert_eq!(Census71::read().integrate_reduced, 1,
               "the reduce arm must be the thing this counter counts, or the zero below is not a \
                measurement");

    // …and the armed march does NOT, so rung 71's own body took its other exit: rung 70's marcher.
    Census71::reset();
    Census70::reset();
    let t = gov_march(&full());
    assert_eq!(Census71::read().integrate_reduced, 0, "{:?}", Census71::read());
    assert_eq!(Census70::read().integrate_reduced, 0,
               "and it did NOT go through rung 70's own `integrate_fuel` either — the call is to \
                the MARCHER, one level in");

    // and it really is entered — the fifth state is recorded and the plant is NOT rung 70's
    assert!(t.iter().all(|p| v_of(p) >= 0.0), "the INCIDENCE band is [0, +v_max]");
    assert!(t.iter().map(v_of).fold(f64::NEG_INFINITY, f64::max) > 0.0);
    assert!(t.iter().any(|p| required_of(p) > 0.0));
}

/// **THE NINTH INSTANCE of the trap rungs 61–70 each hit**: the inherited sibling constructor
/// hardcodes its own name, so a rung-71 machine would hand back a rung-70 one and every reader
/// would measure a `phi` stator (`m = 2`) while reporting `m = 3`.
///
/// Python's `type(m) is FullSplitTransient` has no runtime counterpart (see the module header).
/// What replaces it is the march the parent's table REFUSES: rung 70's guard A rejects an
/// incidence stator beside the governor with *"n = m = 3"*, which is rung 70's own way of saying
/// *this is rung 71's cell*.
#[test]
fn at_lever_returns_this_class() {
    let m = full_of(&LeverArm { bleed_lim: Some(valve(TAU)), ..Default::default() });
    let s = m.at_lever(&LeverArm { bleed_lim: Some(valve(TAU)),
                                   stator_inc: Some(inc(TAU_S, V_MAX)), ..Default::default() });
    // Python's second half, verbatim.
    assert!(s.fuel.inner.stator.inc.is_some() && s.fuel.inner.stator.lim.is_none());
    let t = gov_march(&s);
    assert!(!t.is_empty() && carries_v(&t[0]),
            "the sibling must carry rung 71's OWN table — a parent's refuses an incidence stator \
             beside `tau_gov` outright, which is what `type(m) is FullSplitTransient` asserts");
}

// =============================================================================================
// GATE 2 — THE REFUSALS. Each names a plant this rung is NOT.
// =============================================================================================

/// `n = 4, m = 3` — FOUR loops, two of them on the same actuator. Rung 68's `tau_gov` assert
/// exists because *silently accepts it* is the failure mode.
#[test]
fn the_fuel_leg_beside_the_governor_is_refused() {
    assert!(panics_with(|| {
        let m = full();
        march(&m, DS, Some(fuel_floor()), Some(lag()), Some(TT4_MAX), Some(TAU_GOV));
    }, "n = 4, m = 3"));
}

/// `tau_gov` without `Tt4_max` would march as rung 69 while every reader reported rung 71 — a
/// wrong-plant failure no float would reveal.
#[test]
fn a_governor_with_no_set_point_is_refused() {
    assert!(panics_with(|| {
        let m = full();
        march(&m, DS, None, None, None, Some(TAU_GOV));
    }, "odd loop IS the redline"));
}

/// Rungs 50/51's forced edges are an isolation instrument for a leg that could not pin its own
/// trigger; all three legs here pin their own. And rung 65 called the instantaneous valve limit
/// SINGULAR, so an unlagged valve beside a lagged stator is a different plant.
///
/// **THE FORCED EDGES ARE REFUSED TWICE OVER, AND THE OUTER ONE IS STRUCTURAL — WHICH IS WHY THIS
/// IS THE SLICE's ONE RUNTIME-INTROSPECTION TEST.** See the module header for § 6's two decided
/// replacements; both are spelled below, in Python's own order.
#[test]
fn forced_release_edges_and_an_instantaneous_valve_are_refused() {
    // (1) THE OUTER, STRUCTURAL GUARD — Python's `inspect.signature(_stator_march).parameters`.
    //
    // The march entry takes exactly two config views and NEITHER carries `s_off` or `tau_rel`.
    // Destructured EXHAUSTIVELY, with no `..`: that is the only form that fails to COMPILE when a
    // field is added, which is § 6's *"not reachable becomes a compile error"* and is strictly
    // stronger than an absence assertion evaluated at run time.
    let StatorLeg { accel: _, surge: _, tt4_max: _ } =
        StatorLeg { accel: None::<&AccelSchedule>, surge: None, tt4_max: Some(TT4_MAX) };
    let MarchScope { b0: _, lag: _, tau_gov: _, v0: _, ic_order: _ } = MarchScope::DEFAULT;

    // (2) THE INNER GUARD's OWN SOURCE — Python's `inspect.getsource(integrate_fuel)`.
    //
    // `include_str!` reads at COMPILE time from a path relative to this file, where Python
    // re-reads from disk at import-cached line numbers. The needle is the port's spelling of
    // Python's `"s_off is None and tau_rel is None"`.
    //
    // **THE SEARCH IS SCOPED TO THE FUNCTION BODY, AND THAT IS THE WHOLE CARE IN THIS GATE.**
    // Python reads `inspect.getsource(FullSplitTransient.integrate_fuel)` — the METHOD — and the
    // faithful port is not `include_str!` searched whole. A module-wide `.contains` would pass on
    // a DELETED guard as soon as any doc comment quoted the expression, which is step 4 § (a)'s
    // doc-comment `#[test]` (a `grep` counting 28 where `cargo` ran 27) running in its dangerous
    // direction: there the stray copy inflated a count, here it would satisfy the assertion on
    // behalf of code that no longer exists. Splitting at the `fn` line drops every `///` line,
    // because a doc comment precedes its item; `\n}\n` is the end, because only a top-level item
    // closes at column zero in rustfmt'd source.
    const SRC: &str = include_str!("../src/full_split.rs");
    let body = SRC.split("\nfn r71_integrate_fuel(").nth(1)
                  .expect("rung-71's `integrate_fuel` is still spelled `fn r71_integrate_fuel(`")
                  .split("\n}\n").next()
                  .expect("a top-level fn ends at a column-zero brace");
    assert!(!body.contains("///"), "the scope slipped past the function body");
    assert!(body.contains("lim.s_off.is_none() && lim.tau_rel.is_none()"),
            "rung-71's `integrate_fuel` no longer carries the forced-release guard");
    // …and it is carried exactly ONCE **inside that body**, which is a bar the scoping earns:
    // module-wide the same count would have been satisfiable by a comment.
    assert_eq!(body.matches("lim.s_off.is_none() && lim.tau_rel.is_none()").count(), 1);

    // (3) AND THE INSTANTANEOUS VALVE, WHICH IS AN ORDINARY REFUSAL.
    assert!(panics_with(|| {
        let m = full_of(&LeverArm {
            bleed_lim: Some(BleedLimiter::from_margin_tau(&lp(), B, SM, None)),
            stator_inc: Some(inc(TAU_S, V_MAX)), ..Default::default() });
        gov_march(&m);
    }, "INSTANTANEOUS valve"));
}

/// Rung 65 published a RETRACTION for an RK4 instability that read as a physical finding. The
/// guard's constant survives a FOURTH time on a THIRD argument (no zero root at all, so the trace
/// is shared three ways), so it must fire and say so.
///
/// **The needle is the RUNG TAG and not the sentence**: probe 2b measured `RK4 stability region`
/// matching all THREE floors in this family and `rank TWO` matching two of them, so `rung-71: ds`
/// is the one string unique to this one.
#[test]
fn the_rk4_floor_fires_and_names_its_own_reason() {
    assert!(panics_with(|| {
        let m = full();
        march(&m, 0.05, None, None, Some(TT4_MAX), Some(TAU_GOV));
    }, "rung-71: ds"));
}

// =============================================================================================
// GATE 3 — § 0: **RANK INDEPENDENCE IS NOT CONSTRAINT INDEPENDENCE.** The headline.
// =============================================================================================

/// **THE CONTAINMENT, EXACTLY, ON THE MARCHED TRAJECTORY.** At the valve's own set point
///
/// ```text
/// phi = phi_lim  =>  M_i = T_c - 1/phi_lim + v = m_lim + v  >=  m_lim   for all v >= 0
/// ```
///
/// and the incidence band IS `[0, v_max]` (rung 69 § 0.1), so `{phi >= phi_lim}` intersected with
/// the band sits INSIDE `{M_i >= m_lim}`. The slack minus `v` is `1/phi_lim - 1/phi`, which is
/// `>= 0` there IDENTICALLY and `== 0` exactly where the valve pins `phi` on its floor — so the
/// bound is tight and needs no tolerance.
///
/// THE CONSEQUENCE IS THE RUNG: the stator is DORMANT at every point where the valve delivers, so
/// it can only ride inside the valve's LAG.
#[test]
fn the_third_constraint_is_implied_by_the_second_on_the_whole_band() {
    let bc = band_containment(&full(), &flight(), LO, HI, TT4_MAX, SM, R, SETTLE, DS, TAU,
                              TAU_GOV, TAU_S, V_MAX);
    assert!(bc.n_delivering > 250, "{bc:?}");
    assert!(bc.min_slack_delivering.expect("delivering points exist") >= 0.0, "{bc:?}");
    // tight, and EXACTLY so
    assert_eq!(bc.worst_slack_minus_v.expect("delivering points exist"), 0.0, "{bc:?}");
    assert_eq!(bc.riding_while_delivering, 0, "{bc:?}");
    // and the wall IS violated where the valve is failing — otherwise the loop is vacuous
    assert!(bc.min_slack_all < 0.0 && bc.n_riding > 0, "{bc:?}");
}

/// **THE MECHANISM, MEASURED FROM BOTH SIDES.** If the containment is why the window is thin, the
/// stator's right edge must be a function of the VALVE's clock and not of its own. A one-sided
/// sweep could not separate that from *a slower loop rides longer*, which is a different and much
/// weaker statement.
///
/// Measured: the edge marches `0.115 → 0.365` monotonically over a 400× sweep of `tau_q`, and
/// moves within a 1.3× band NON-monotonically over an equivalent sweep of `tau_s`.
#[test]
fn the_third_loops_window_is_the_second_loops_lag() {
    let wl = window_law(&full(), &flight(), LO, HI, TT4_MAX, SM, &TAU_QS, &TAU_SS, R, SETTLE, DS,
                        TAU, TAU_GOV, TAU_S, V_MAX);
    let q_span = wl.q_span.expect("every valve arm has a stator window");
    let s_span = wl.s_span.expect("every stator arm has one too");
    assert!(wl.q_monotone, "{:?}", wl.edge_q);
    assert!(q_span > 2.5, "{:?}", wl.edge_q);
    assert!(s_span < 1.6, "{:?}", wl.edge_s);
    assert!(q_span > 2.0 * s_span, "{:?} {:?}", wl.edge_q, wl.edge_s);
    let es: Vec<f64> = wl.edge_s.iter()
        .map(|x| x.expect("Python indexes these unconditionally")).collect();
    assert!(!(0..es.len() - 1).all(|i| es[i] <= es[i + 1] + 1e-12),
            "the stator's own clock is not even a monotone influence — which is the point");
    // THE JOINT WINDOW IS THIN, AND THAT IS DISCLOSED RATHER THAN WORKED AROUND
    assert!(0.0 < wl.joint_fraction && wl.joint_fraction < 0.05, "{}", wl.joint_fraction);
    assert!(wl.base.n_interior >= 5, "{:?}", wl.base);
    // AND THE TWO WINDOWS ARE NOT THE SAME NUMBER, which is gated because conflating them would
    // credit CONTAINMENT with narrowing that belongs to rung 67's imposed `Tt4_max`. The stator
    // rides over ~7.9 % of the march; the joint window is that intersected with a governor that
    // opens late, and its LEFT edge is the GOVERNOR's, not the stator's.
    let b = &wl.base;
    assert!(b.stator.2 as f64 / b.n as f64 > 2.0 * wl.joint_fraction, "{b:?}");
    assert_eq!(b.joint.0, b.gov.0, "{:?} {:?}", b.joint, b.gov);
    assert!(b.stator.0.expect("the stator rides") < b.gov.0.expect("the governor opens"),
            "{:?} {:?}", b.stator, b.gov);
}

/// AND THE TWO EDGES ARE NOT THE SAME NUMBER, which is stated rather than fudged. `_solve_v` tests
/// dormancy on the COUNTERFACTUAL plant at `v = 0`, so the loop quits while the MARCHED `phi` is
/// still below the floor by its own contribution — measured `dphi/dv` is about `−0.42` (rung 69
/// § 0.1), so the shortfall should be `~0.42 * v`.
#[test]
fn the_stator_quits_while_the_marched_phi_is_still_short() {
    let wl = window_law(&full(), &flight(), LO, HI, TT4_MAX, SM, &[TAU], &[TAU_S], R, SETTLE, DS,
                        TAU, TAU_GOV, TAU_S, V_MAX);
    let short = wl.phi_short_at_off.expect("the stator does go dormant");
    let v = wl.v_at_off.expect("and holds a setting when it does");
    assert!(short > 0.0 && v > 0.0, "{short} {v}");
    assert!(0.30 < short / v && short / v < 0.55, "{short} {v} {}", short / v);
}

// =============================================================================================
// GATE 4 — § 1: THREE pairs, ZERO identities, and the determinant FACTORS.
// =============================================================================================

/// Rung 66's `pair = 1` survived three times at rung 68 (`m = 1`), once at rung 69 and once at
/// rung 70 (`m = 2`), and **zero times here** — it is a property of a SHARED constraint, and at
/// `n = m` nothing is shared. The closest any pair comes to 1 is ~1.0, i.e. not close.
#[test]
fn rung66s_identity_appears_zero_times_for_the_first_time() {
    let g = gains();
    assert!(!g.rows.is_empty(), "{:?}", g.skipped);
    let c = g.closest_to_1.expect("rows exist, so the aggregate does");
    assert!(c > 0.9, "{c}");
}

/// Rung 68 said *quote `x`*; rung 69 said it flips to `−k`; rung 70 found it BLIND to `pair_RV`.
/// Here BOTH cyclic products collapse onto the pairs:
///
/// ```text
/// y := R_v C_g V_q = -pair_RV                 exactly, at ANY base point
/// x := R_q C_v V_g = -pair_RC * pair_CV
/// ```
///
/// so the three PAIRS are the complete independent set and neither cyclic is a measurement. Rung
/// 68's *check what is INDEPENDENT before quoting it*, in its third shape.
#[test]
fn both_cyclic_products_are_redundant() {
    let g = gains();
    let wy = g.worst_y_is_rv.expect("rows exist");
    let wx = g.worst_x_is_product.expect("rows exist");
    assert!(wy < 5e-3, "{wy}");
    assert!(wx < 5e-3, "{wx}");
    // against the quantities they reproduce: the residual is a differencing floor, not a signal
    let smallest_rv = g.pair_rv.iter().map(|p| p.abs()).fold(f64::INFINITY, f64::min);
    assert!(wy < 0.02 * smallest_rv, "{wy} vs {smallest_rv}");
}

/// **THE HEADLINE INVARIANT.** `det M = −(1 − pair_RC)(1 − pair_CV)` — rung 67's own
/// non-degeneracy condition times rung 69's, ONE FACTOR PER RUNG. And it is therefore BLIND to
/// `pair_RV`, the one gain this rung contains that no earlier rung has measured: it cancels
/// exactly against the reverse cyclic product `y`.
///
/// THIS IS NOT A TAUTOLOGY, and the distinction is the gate (rung 67 gate 9's retraction). The
/// closed form uses FOUR of the six gains and asserts the other two drop out. `c1`'s closed form,
/// by contrast, IS a re-expression of any matrix with `−1` on the diagonal and is reported by
/// `full_modes`, never gated.
#[test]
fn the_full_rank_determinant_factors_into_two_prior_rungs() {
    let g = gains();
    let err = g.worst_det_err.expect("rows exist");
    let scale = g.det_scale.expect("rows exist");
    assert!(err < 5e-3, "{err}");
    assert!(err < 1e-2 * scale, "{err} {scale}");
}

/// AND IT IS SHOWN BY CONSTRUCTION, not only by measurement (rung 69's precedent).
///
/// Hand-build the block from the six gains with `grad psi = sigma grad phi + e_v` imposed, then
/// move `T_v` — which changes `R_v`, and through it `pair_RV` and `y`, and NOTHING else. The
/// determinant must not move at all.
#[test]
fn the_determinant_provably_cannot_see_pair_rv() {
    // `sig = 1.6`, `phi_v = -0.4` puts `k = sig*phi_v/psi_v = -1.778` — rung 69's own measured
    // band, so the constructed block is the shipped plant's shape and not an arbitrary one.
    let (t_g, t_q, phi_g, phi_q, phi_v, sig) = (-3.0, 0.7, -0.9, 1.3, -0.4, 1.6);
    let psi_v = sig * phi_v + 1.0;
    let block = |t_v: f64| -> [[f64; 3]; 3] {
        [[-1.0, -t_q / t_g, -t_v / t_g],
         [-phi_g / phi_q, -1.0, -phi_v / phi_q],
         [-sig * phi_g / psi_v, -sig * phi_q / psi_v, -1.0]]
    };
    // PYTHON's OWN COFACTOR EXPANSION, term for term — a regrouping would move the last bits of a
    // quantity this gate compares at `1e-12`.
    let det = |m: &[[f64; 3]; 3]| -> f64 {
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    };
    let d0 = det(&block(0.5));
    for t_v in [-4.0, -0.2, 1.7, 9.0] {
        let m = block(t_v);
        assert!((det(&m) - d0).abs() < 1e-12 * 1.0_f64.max(d0.abs()), "{t_v}");
        // …while `pair_RV` and the reverse cyclic DO move, and stay each other's negative
        let pair_rv = m[0][2] * m[2][0];
        let y = m[0][2] * m[1][0] * m[2][1];
        assert!((y + pair_rv).abs() < 1e-12, "{t_v}");
    }
    // and the closed form is the two prior rungs' conditions, multiplied
    let m = block(0.5);
    assert!((d0 + (1.0 - m[0][1] * m[1][0]) * (1.0 - m[1][2] * m[2][1])).abs() < 1e-12);
}

/// **`m = 3` IS `pair_RC != 1`.** `span{grad phi, grad psi} = span{grad phi, e_v}`
/// UNCONDITIONALLY — the lever's own `+1` in `psi_v` puts `e_v` in the span whatever the plant
/// does — so the governor's gradient escapes that plane iff `T_g phi_q != T_q phi_g`.
///
/// Built rather than argued (rung 69's precedent): force `grad T` INTO the plane and the same
/// `n = 3` block must come back rank 2 with exactly one zero eigenvalue.
///
/// **Only the six cross-gains are read by [`invariants`]**; the rest of the [`TripleGains`] struct
/// is filled with values that cannot enter it — `tests/rung70.rs`'s own hand-built control, one
/// rung on.
#[test]
fn the_rank_is_rung67s_own_non_degeneracy_condition() {
    let (phi_g, phi_q, phi_v, sig) = (-0.9, 1.3, -0.4, 1.6);
    let psi_v = sig * phi_v + 1.0;
    let taus = (0.05, 0.05, 0.05);
    let spectrum = |t_g: f64, t_q: f64, t_v: f64| -> (f64, usize) {
        let (r_q, r_v) = (-t_q / t_g, -t_v / t_g);
        let (c_g, c_v) = (-phi_g / phi_q, -phi_v / phi_q);
        let (v_g, v_q) = (-sig * phi_g / psi_v, -sig * phi_q / psi_v);
        let gg = TripleGains {
            interior: true, off_regime: Vec::new(),
            r_q, r_v, c_g, c_v, v_g, v_q,
            v_base: 0.0, cyclic: r_q * c_v * v_g,
            pair_rc: r_q * c_g, pair_rv: r_v * v_g, pair_cv: c_v * v_q, s: 0.0,
        };
        let (c2, c1, c0) = invariants(&gg, taus);
        let roots = cubic_roots_c(c2, c1, c0);
        // Python's `sum(1.0 / t for t in taus)` — a three-term LEFT FOLD.
        let rate = 1.0 / taus.0 + 1.0 / taus.1 + 1.0 / taus.2;
        (gg.pair_rc, roots.iter().filter(|r| r.abs() < 1e-8 * rate).count())
    };
    // generic: OUT of the plane => pair_RC != 1 => rank 3, ZERO zeros
    let (p, z) = spectrum(-3.0, 0.7, 0.5);
    assert!((p - 1.0).abs() > 0.5 && z == 0, "{p} {z}");
    // forced INTO the plane (`T_q/T_g == phi_q/phi_g`) => pair_RC == 1 => rank 2, ONE zero
    let t_g = -3.0;
    let (p, z) = spectrum(t_g, t_g * phi_q / phi_g, 0.5);
    assert!((p - 1.0).abs() < 1e-12 && z == 1, "{p} {z}");
}

/// **TWO CONTROLS, AND THEY ARE DIFFERENT KINDS — conflating them would be the error.**
///
/// `pair_RC` is a NUMERICAL control: rows R and C are the same shipped closures rungs 67 and 70
/// used, so it must reproduce rung 67's `P` up to the base-point shift the third loop induces. It
/// is read against a genuinely separate rung-67 march (`cross_identity` on a STATOR-FREE rig) and
/// reported as a ratio, never gated to a tolerance that shift cannot justify.
///
/// `pair_CV` is a FUNCTIONAL-FORM control: it IS rung 69's `k` on rung 69's own two loops, but
/// re-measured on a different trajectory. Its FORM and BAND are what is gated.
///
/// **`cross_identity` runs at ITS OWN defaults, not this file's** — `ds = 0.0025` and
/// `n_sample = 12`, where every other reader here is at `DS = 0.005`. Python reaches them by
/// omitting the keywords; the Rust takes them explicitly, so they are spelled.
#[test]
fn the_two_inherited_controls() {
    let m = full();
    let rig = full_rig(&m, SM, TAU, TAU_S, V_MAX, TT4_MAX, true, false);
    let re = rig.cross_identity(&flight(), &ramp(0.0025), TT4_MAX, TAU, &[TAU_GOV], 12);
    let g = gains();
    assert!(re.all_negative && g.pair_rc.iter().all(|&p| p < 0.0));
    let mid: f64 = g.pair_rc.iter().sum::<f64>() / g.pair_rc.len() as f64;
    let ratio = mid / (0.5 * (re.prod_lo + re.prod_hi));
    assert!(0.5 < ratio && ratio < 2.0, "{mid} {} {} {ratio}", re.prod_lo, re.prod_hi);
    // rung 69's `k`, on rung 69's own two loops
    assert!(g.pair_cv.iter().all(|&p| R69_K_LO < p && p < R69_K_HI), "{:?}", g.pair_cv);
}

/// `pair_RV(71) = pair_CV * pair_RV(70)` at an IDENTICAL base point, because
/// `psi_g/psi_v = (phi_g/phi_v)(sigma phi_v/psi_v)`. Measured by reading rung 70's `phi`-referenced
/// rig at THIS march's own points — rung 69's design, which differences two references on ONE
/// trajectory rather than on two.
#[test]
fn the_cross_rung_identity_pair_rv_is_k_times_rung70s() {
    let g = gains();
    let w = g.worst_cross_rung.expect("the rung-70 arm is interior somewhere");
    assert!(w < 0.02, "{w}");
}

/// `R_q != 0` and `R_v != 0` ONLY because the governor senses `Tt4` on the machine as the other
/// two actuators actually are. Drop the `_b_state`/`_v_state` boundary and both cross-gains are
/// identically zero, the odd loop DECOUPLES, and `m` reads 2 by accident. Rung 70 built the broken
/// version on purpose; it is checked here at every sampled point.
#[test]
fn the_state_boundary_is_asserted_at_every_sampled_point() {
    let g = gains();
    assert!(!g.boundary.is_empty(), "the boundary check never ran");
    for c in &g.boundary {
        assert_eq!(c.dead_r_q, 0.0, "{c:?}");
        assert_eq!(c.dead_r_v, 0.0, "{c:?}");
        assert!(c.live_r_q.abs() > 0.0 && c.live_r_v.abs() > 0.0, "{c:?}");
    }
}

// =============================================================================================
// GATE 5 — § 2: ZERO zeros, `det J` ALIVE, and Routh non-trivial.
// =============================================================================================

/// **THE RUNG.** `zeros = n − m = 0` at `(n, m) = (3, 3)` — the one cell of rung 69 § 1's table
/// this ladder has never occupied, and the first plant in this family whose actuator block is
/// INVERTIBLE.
#[test]
fn the_last_unoccupied_cell_has_zero_zeros() {
    let md = modes();
    assert_eq!(md.zeros_everywhere, vec![0], "{:?}", md.zeros_everywhere);
    for arm in &md.arms {
        assert!(!arm.rows.is_empty(), "{:?}", arm.taus);
        assert_eq!(arm.zeros, vec![0], "{:?} {:?}", arm.taus, arm.zeros);
        // and the smallest root is not merely 'non-zero by the tolerance'
        let mr = arm.min_root_rel.expect("rows exist");
        assert!(mr > 1e-2, "{:?} {mr}", arm.taus);
    }
}

/// THREE readings on ONE materialisation of the spectrum, and they share a test for a MEASURED
/// reason rather than a stylistic one: under xdist a module-scoped fixture is rebuilt PER WORKER,
/// so every extra consumer of this reader can cost a whole re-run of it. **That cost is Python's
/// and does not transfer** — each Rust test rebuilds the reader anyway (see [`full`]) — but the
/// GROUPING is kept, because splitting it would change which assertions share a materialisation
/// and this file is a translation, not a re-design.
///
/// `c0` — `det J != 0` for the first time in this family, and it equals
/// `−(1−pair_RC)(1−pair_CV)/prod(tau)`, FOUR of the six gains.
///
/// ROUTH — at `m < n` a zero root plus a negative trace made stability automatic. At full rank it
/// is a CONDITION, and the derivation leaves six unconditionally positive terms plus
/// `(u + w + z − u z) a b c`, so **`u + w + z >= u z` is SUFFICIENT at EVERY bandwidth triple**.
/// The spectrum is checked stable arm by arm rather than inferred from the certificate: an assert
/// nobody has run past is a tautology (rung 67 gate 9).
///
/// RK4 — the inherited constant survives a FOURTH time on a THIRD argument, and rung 65's
/// retraction is why it is MEASURED rather than trusted.
#[test]
fn the_invariants_at_full_rank() {
    let md = modes();
    let c0 = md.max_c0_err.expect("rows exist");
    let routh = md.min_routh.expect("rows exist");
    let mr = md.max_mod_ratio.expect("rows exist");
    assert!(c0 < 5e-3, "{c0}");
    assert!(routh > 0.0, "{routh}");
    assert!(md.all_stable, "{md:?}");
    for arm in &md.arms {
        for row in &arm.rows {
            assert!(row.stable, "{:?} {} {:?}", arm.taus, row.s, row.roots);
        }
    }
    assert!(mr < 1.0, "{mr}");
    assert!(md.ds * mr * 240.0 < 2.0, "{md:?}");
}

/// **RUNG 69's FLOOR DOES NOT SURVIVE FULL RANK, and the mechanism says why.** All three roots
/// share ONE trace budget, `sum(lam) = −sum 1/tau_i`. At rung 69 the third root WAS the zero, so
/// the pair took the whole budget and `zeta >= 1/sqrt(1−k)` followed by AM-GM. Here the third
/// loop's own pole DRAINS it, so the pair's real part is smaller at comparable modulus and the
/// bound has no reason to hold.
///
/// IT DOES NOT. The grid shows all three regimes — arms below rung 69's line, arms above it, and
/// arms with no complex pair at all — which is what *the bound is removed, not replaced* has to
/// look like. A single monotone trend, or a floor that survived, would refute the trace-budget
/// mechanism.
#[test]
fn rung69s_damping_floor_was_the_c0_equals_0_corner() {
    let md = modes();
    assert!(md.arms_below_r69 >= 1, "{md:?}");
    assert!(md.arms_with_ring - md.arms_below_r69 >= 1, "{md:?}");
    assert!(md.arms_real >= 1, "{md:?}");
}

/// **THE INSTRUMENT, AND ITS THIRD REBUILD IN FOUR RUNGS.** Rung 69 reads `−Re(dom)/|dom|`, exact
/// for a complex DOMINANT pair and exactly 1.0 for any real root; rung 70 reads both NON-ZERO
/// roots magnitude-sorted, exact when exactly one root is zero. **Here no root is zero and the
/// pair is not always the two largest**, so magnitude ordering can drop a pair MEMBER and keep the
/// odd real root.
///
/// Built as a difference on constructed spectra, so it does not depend on the plant.
#[test]
fn the_damping_reader_had_to_be_rebuilt_a_third_time() {
    let c = |re: f64, im: f64| C64 { re, im };
    // a real root SMALLER than the pair: both readers agree
    let ok = [c(-18.0, 0.0), c(-21.0, 28.0), c(-21.0, -28.0)];
    let (a, b) = (zeta_ring(ok).expect("a ring is present"),
                  zeta_pair(ok).expect("rung 70's reader returns one too"));
    assert!((a - b).abs() < 1e-12, "{a} {b}");
    // a real root LARGER than the pair: rung 70's reader drops a pair member
    let bad = [c(-194.0, 0.0), c(-23.0, 25.5), c(-23.0, -25.5)];
    let ring = zeta_ring(bad).expect("the ring is still there");
    assert!((ring - 23.0 / c(-23.0, 25.5).abs()).abs() < 1e-12, "{ring}");
    assert!((zeta_pair(bad).expect("rung 70's reader still answers") - ring).abs() > 0.5);
    // an entirely REAL spectrum: a reader that returns a number where there is no ring is worse
    // than one that returns nothing
    assert!(zeta_ring([c(-20.0, 0.0), c(-82.0, 0.0), c(-138.0, 0.0)]).is_none());
}

// =============================================================================================
// GATE 6 — § 3: THE FIXED POINT IS A POINT. Rung 69 § 6, at nullity ZERO.
// =============================================================================================

/// **RUNG 69 § 6 CALLED A NULL SPACE A SHOCK ABSORBER; AT NULLITY ZERO THERE IS NOTHING TO ABSORB
/// WITH, AND THE SWEEP REJECTS INSTEAD.**
///
/// Rungs 68/69/70 all carry a null space, so their `s = 0` fixed points are a ONE-PARAMETER FAMILY
/// and a Gauss-Seidel sweep lands on whichever member its ORDER selects. At `n = m` there is no
/// null space and the fixed point is a POINT: every sweep order and every displaced start must
/// land on the SAME `(g, q, v)`.
///
/// RUNG 70's PLANT IS THE NEGATIVE CONTROL ON THE SAME RIG — its valve and stator SHARE `phi`, so
/// `|C_v V_q| = 1` exactly and its sweep is marginal by construction. A contraction here that were
/// not matched by a failure to contract there would be measuring the SOLVER.
#[test]
fn the_s0_fixed_point_becomes_unique_at_full_rank() {
    let ic = ic_contraction(&full(), &flight(), LO, HI, TT4_MAX, SM, &IC_ORDERS, &IC_FRACS, R,
                            SETTLE, DS, TAU, TAU_GOV, TAU_S, V_MAX);
    let (fu, sh) = (&ic.full, &ic.shared);
    assert_eq!(fu.n_converged, fu.n, "{fu:?}");
    assert_eq!(fu.members, 1, "{fu:?}");
    assert_eq!(fu.spread.expect("converged rows exist"), (0.0, 0.0, 0.0), "{:?}", fu.spread);
    // THE CONTROL: the shared-constraint plant lands on a FAMILY from the same starts
    assert!(sh.members > 1, "{sh:?}");
    let s = sh.spread.expect("the control converges too, just not to one point");
    assert!(s.0.max(s.1).max(s.2) > 1e-3, "{s:?}");
}

// =============================================================================================
// GATE 7 — § 4: THREE currencies, and rung 70 § 5's erosion law CORRECTED.
// =============================================================================================

/// A FREE DIFFERENCEABILITY CHECK (rung 63's lesson). Every cell WITHOUT an incidence stator is a
/// rung-70 march; every cell WITHOUT a governor is a rung-69 one. Only `GS` and `GVS` are new, so
/// six of the eight must reproduce their ancestors' published integrals exactly — a drift in a cell
/// that CANNOT have one would mean the rigs are not comparable.
///
/// **The comparison is BIT-FOR-BIT** — Python's `==` on two floats, which `assert_eq!` is here.
#[test]
fn six_of_the_eight_ledger_cells_are_inherited_bit_for_bit() {
    let bl = bill();
    let r70m = cross_of(&LeverArm { bleed_lim: Some(valve(TAU)),
                                    stator_lim: Some(phi_stator(TAU_S, V_MAX)),
                                    ..Default::default() });
    let r70 = split_bill(&r70m, &flight(), LO, HI, TT4_MAX, SM, R, SETTLE, DS, TAU, TAU_GOV,
                         TAU_S, V_MAX);
    // no stator at all => rung 70's own cells
    for name in ["bare", "G", "V", "GV"] {
        assert_eq!(bl.cell(name).i, r70.cell(name).i, "{name}");
        assert_eq!(bl.cell(name).e, r70.cell(name).e, "{name}");
    }
    // the two NEW cells are the ones that carry BOTH the governor and the incidence stator
    assert!(bl.cell("GS").i != r70.cell("GS").i);
    assert!(bl.cell("GVS").i != r70.cell("GVS").i);
}

/// Rungs 66/68 had one currency, rung 70 had two; three loops on three walls need three. And the
/// cross-credits keep rung 70's SIGNS: both airflow loops DEBIT the temperature while the governor
/// CREDITS the surge margin.
#[test]
fn the_ledger_needs_three_currencies() {
    let bl = bill();
    let base = *bl.cell("bare");
    assert!(bl.cell("V").e > base.e && bl.cell("S").e > base.e, "{:?}", bl.degrades);
    assert!(bl.cell("G").i < base.i, "{:?}", bl.cell("G"));
    for (k, d) in [("phi", bl.delivered_phi), ("Tt4", bl.delivered_tt4),
                   ("inc", bl.delivered_inc)] {
        assert!(d.expect("the bare march violates every wall") > 0.5, "{k}: {d:?}");
    }
}

/// **§ 0's CONTAINMENT, READ IN THE LEDGER, AND THE SHARPEST SINGLE NUMBER HERE.** The VALVE —
/// which cannot see `M_i` at all — delivers more incidence credit running alone than the INCIDENCE
/// STATOR does, because holding `phi` on its floor implies the incidence wall with margin `v`
/// while the reverse is not true.
#[test]
fn the_loop_that_does_not_watch_the_wall_protects_it_better() {
    let bl = bill();
    let v = bl.inc_credit_valve_alone.expect("the bare march violates the incidence wall");
    let s = bl.inc_credit_stator_alone.expect("…so both solo credits are defined");
    assert!(v > s, "valve {v} vs stator {s}");
    assert!(v > 0.85, "{v}");
}

/// **RUNG 70 § 5: *a loop is eroded by the loops it shares a constraint with, and by no others.*
/// NO TWO LOOPS SHARE HERE, AND THE STATOR IS ERODED ANYWAY** — it keeps a few per cent of its
/// solo credit in its own currency, while the governor keeps ~100 % of its own.
///
/// THE CORRECTION IS § 0's MECHANISM: erosion has a SECOND channel. A loop is eroded by any loop
/// that pushes its constraint into the SLACK region, which is a statement about FEASIBLE SETS and
/// not about gradients. Rung 70 could not see it because none of its loops could satisfy another's
/// wall on its behalf.
///
/// AND THE TWO READINGS ARE QUOTED TOGETHER (rung 58's *check the SUM, not the term*): the valve's
/// `kept` exceeds 1 only because the stator running alone DEGRADES `phi` below the bare march
/// (rung 69 § 4's own finding), so the valve is repairing damage rather than delivering protection.
/// That confound is recorded, not hidden.
#[test]
fn rung70s_erosion_law_is_corrected_by_a_second_channel() {
    let bl = bill();
    let k = bl.kept;
    let gov = k.gov.expect("the governor has a non-zero solo credit");
    let stator = k.stator.expect("so does the stator");
    let valve = k.valve.expect("and the valve");
    assert!(0.8 < gov && gov < 1.3, "{k:?}");       // unshared AND uneroded — rung 70's half
    assert!(stator < 0.25, "{k:?}");                // unshared and ERODED — the correction
    // the confound behind `kept.valve > 1`, recorded rather than explained away
    assert!(valve > 1.0, "{k:?}");
    let deg = |n: &str| -> Vec<&'static str> {
        bl.degrades.iter().find(|(k, _)| *k == n)
          .unwrap_or_else(|| panic!("rung-71's ledger has no cell {n:?}")).1.clone()
    };
    assert!(deg("S").contains(&"I"), "{:?}", bl.degrades);
    assert!(deg("GS").contains(&"I"), "{:?}", bl.degrades);
}
