//! RUNG 57 — **THE STATOR SCHEDULE ON THE TRANSIENT PLANT: a wall-moving lever has no CLOCK.**
//!
//! `tests/test_rung57.py` ported one-to-one: **14 Python `def test_` → 16 collected** (two
//! two-way `parametrize`s), and **16 `#[test]` here**, the parametrized pair expanded into
//! `_primary` / `_tilted` so the Rust count matches Python's COLLECTED count rather than its
//! `def` count. Slice T step 3's shape.
//!
//! # THE GRID IS THIS FILE'S OWN, AND IT LOOKS SHAREABLE
//!
//! `N_LO` here is **0.75574**; rungs 58, 59 and 60 all use **0.7557**. [`KEYS`] here is nine
//! names; rung 58's list is eleven (it adds `mf_sched` and `s`), rung 60's is twelve and leads
//! with `s`. None of that is shared through a helper module: a single `N_LO` would silently
//! re-grid one suite or the other in the fourth decimal and no gate in either file would flag
//! it. Slice T step 3's *two grids that look shareable and are not*, one digit finer.
//!
//! **AND THE `ds` TRAP IS THE SHARPEST OF THEM, BECAUSE THE SHIPPED SOURCE POINTS THE WRONG WAY.**
//! [`Ramp::fine`]'s doc comment calls `ds = 0.005` *"rungs 58/59/60's default"*, and as a
//! statement about the READER METHODS' signatures that is exactly right. It is NOT what the
//! suites march on. Measured off the four files: `test_rung57.py` and `test_rung60.py` take
//! their reader defaults (0.01 and 0.005), while **`test_rung58.py` and `test_rung59.py` declare
//! `DS = 0.01` and pass it EXPLICITLY at every call site**, overriding the 0.005 default they
//! would otherwise inherit. Porting 58 or 59 through [`Ramp::fine`] because its doc names those
//! rungs would halve their step and change every number they assert. A method default and a
//! suite constant are two different claims about one parameter.
//!
//! # WHAT THESE 16 GATES DO NOT ESTABLISH
//!
//! **They are RELATIONAL.** Every one asserts a relation among values this crate computed — a
//! sign, an ordering, a ratio, a bit-for-bit equality between two RUST marches. A Rust/Python
//! arithmetic divergence moves both sides of every one of them and leaves all 16 green. That is
//! § 5.20 (ii)'s own headline one level up (59/59 green in BOTH modes), and the instrument that
//! establishes agreement with Python is **step 4's oracle**, not this file. `slice_v_smoke.rs`
//! covers the value question structurally at `ds = 0.05`; nothing here re-does it.
//!
//! **And the HP arm is barely exercised.** § 5.20 P4 measured Python's four suites leaving
//! `map_hp` mutated in **0 of 920 262** closes, so the ported gates inherit that blindness
//! exactly. The HP-scheduled path is reached by the smoke's section C and by step 4's dump.
//!
//! [`Ramp::new`]: turbojet::stator_transient::Ramp::new
//! [`Ramp::fine`]: turbojet::stator_transient::Ramp::fine

use std::panic::catch_unwind;

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::fuel_transient::{FuelLimiters, FuelPoint, FuelTransientCore};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{
    counters as scount, Ramp, ScheduledStatorCore, ScheduledStatorTransient, Shape, StatorArm,
    StatorLeg, StatorSchedule,
};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------------- the grid
// `test_rung57.py:47-62`, spelled out rather than imported from a shared module — see the header.
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const V: f64 = 0.20;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const DS: f64 = 0.01;
const SETTLE: f64 = 1.2;
/// **0.75574** — rung 57's own five-digit spelling of the bare machine's running-line start
/// speed. Rungs 58/59/60 write 0.7557.
const N_LO: f64 = 0.75574;
/// The nine keys rung 57's reduce compares. Rung 58's list is TEN.
const KEYS: [&str; 9] =
    ["nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "f", "mf", "pi_lpc", "pi_hpc"];
/// The five ramp rates P1/P2 sweep — a 20× range.
const RATES: [f64; 5] = [0.1, 0.25, 0.5, 1.0, 2.0];

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

/// `test_rung57.py:70` — `R_c` DERIVED as `(g-1)/g*cp`.
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

/// The TILTED pair — `c = 0.06` on both, the second `parametrize` cell.
fn tilt_map() -> ComponentMap {
    ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

/// The FLAT-ETA ISLAND — rung 53's own zeroing control.
fn flat_lp() -> ComponentMap {
    ComponentMap { sigma: 0.1, l: 0.7, ..ComponentMap::flat() }.with_phi_surge(FLOOR)
}

fn flat_hp() -> ComponentMap {
    ComponentMap { sigma: 0.1, l: 1.0, ..ComponentMap::flat() }.with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

/// Python's `_st(...)`.
fn st_maps(lp: ComponentMap, hp: ComponentMap, arm: StatorArm) -> ScheduledStatorCore {
    match ScheduledStatorTransient::new(design(), flight(), 1.0, Some(lp), Some(hp), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("rungs 57-60 never disable LP"),
    }
}

fn st(arm: StatorArm) -> ScheduledStatorCore {
    st_maps(lp_map(), hp_map(), arm)
}

fn ramp(r: f64) -> Ramp {
    Ramp { tt4_lo: LO, tt4_hi: HI, r, s_settle: SETTLE, ds: DS }
}

fn sched() -> StatorSchedule {
    StatorSchedule::new(V, N_LO)
}

fn pt(p: &FuelPoint, k: &str) -> f64 {
    match k {
        "nu_lp" => p.nu_lp,
        "nu_hp" => p.nu_hp,
        "phi_lp" => p.phi_lp,
        "phi_hp" => p.phi_hp,
        "Tt4" => p.tt4,
        "f" => p.f,
        "mf" => p.mf,
        "pi_lpc" => p.pi_lpc,
        "pi_hpc" => p.pi_hpc,
        _ => unreachable!("{k}"),
    }
}

/// The text of a caught panic — [`pdf_oracle`]'s helper, fifth use. `catch_unwind` catches ANY
/// panic, so "it panicked" is a weaker statement than it reads; Python's `pytest.raises` gates
/// below are all on named asserts, so naming them here is fidelity, not defensiveness.
fn panic_text(e: Box<dyn std::any::Any + Send>) -> String {
    e.downcast_ref::<String>()
        .cloned()
        .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_string()))
        .unwrap_or_default()
}

/// Asserts `f` panics AND that the message names the guard under test.
fn refuses(what: &str, f: impl FnOnce() + std::panic::UnwindSafe) {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = catch_unwind(f);
    std::panic::set_hook(prev);
    match out {
        Ok(()) => panic!("expected a refusal naming {what:?}, but the call SUCCEEDED"),
        Err(e) => {
            let msg = panic_text(e);
            assert!(msg.contains(what), "panicked, but not on {what:?}: {msg}");
        }
    }
}

// =============================================================================================
// THE REDUCE — rung 57 off is rungs 43-52, bit-for-bit
// =============================================================================================

/// Python `test_reduce_no_schedule_bit_for_bit`. An unarmed [`ScheduledStatorCore`] IS rung
/// 43/45's plant: `arm` returns on its first line, so both closures run the inherited bodies
/// with the maps untouched.
#[test]
fn test_reduce_no_schedule_bit_for_bit() {
    let f = flight();
    let bare43 = FuelTransientCore::new(design(), f, 1.0, lp_map(), hp_map(), 1.0);
    let (a, b) = (bare43.fuel_for_tt4(&f, LO), bare43.fuel_for_tt4(&f, HI));
    let eq = bare43.inner.equilibrium(&f, LO);
    let r = 0.5;
    let s = move |x: f64| a + (b - a) * (x / r).min(1.0);
    let reference =
        bare43.integrate_fuel(&f, s, (eq.nu_lp, eq.nu_hp), r + SETTLE, DS, &FuelLimiters::default());
    let (got, _) = st(StatorArm::default()).stator_march(&f, &ramp(r), None, &StatorLeg::default());

    // Ahead of the zip, both halves — Python asserts `len(got) == len(ref) > 100` and Rust's
    // `zip` truncates just as silently as Python's.
    assert_eq!(got.len(), reference.len(), "the two marches must land on the SAME grid");
    assert!(reference.len() > 100, "npts = {}", reference.len());
    for (i, (x, y)) in reference.iter().zip(got.iter()).enumerate() {
        for k in KEYS {
            assert_eq!(pt(x, k).to_bits(), pt(y, k).to_bits(),
                       "{k} at row {i} (s = {}): {} vs {}", x.s, pt(x, k), pt(y, k));
        }
    }
}

/// Python `test_reduce_zero_schedule_bit_for_bit_and_map_identity`.
///
/// **THIS IS A RE-GATE, AND WHAT IT GIVES UP IS STATED (§ 5.20 P3).** Python's second half is
/// `assert ft.map_lp is ft.map_lp_design` — an OBJECT-IDENTITY claim that `_arm` hands back the
/// very map it was given at `v == 0.0`, so the swap machinery is witnessed *inert* rather than
/// merely not taken. [`ComponentMap`] is `Copy` and has no identity, so `is` is unspellable here
/// and value equality alone would be a WEAKER gate — it cannot distinguish "handed back
/// untouched" from "rebuilt to the same numbers", which is precisely the branch the Python is
/// pinning.
///
/// What replaces it is the pair the identity claim reduces to on a value type: the march is
/// bit-for-bit (below), **and the crate's own dispatch counter says every one of `arm`'s LP
/// decisions took the ZERO arm** — `arm_lp_zero == arm_calls` with `arm_lp_moved == 0`. A
/// rebuild-to-the-same-numbers implementation would land in `arm_lp_moved` and fail. The counter
/// is the stronger half; the equality is kept beside it because a counter on a dead instrument is
/// permanently green.
#[test]
fn test_reduce_zero_schedule_bit_for_bit_and_map_identity() {
    let f = flight();
    let bare43 = FuelTransientCore::new(design(), f, 1.0, lp_map(), hp_map(), 1.0);
    let (a, b) = (bare43.fuel_for_tt4(&f, LO), bare43.fuel_for_tt4(&f, HI));
    let eq = bare43.inner.equilibrium(&f, LO);
    let r = 0.5;
    let s = move |x: f64| a + (b - a) * (x / r).min(1.0);
    let reference =
        bare43.integrate_fuel(&f, s, (eq.nu_lp, eq.nu_hp), r + SETTLE, DS, &FuelLimiters::default());

    let z = StatorSchedule::new(0.0, 0.75);
    for arm in [StatorArm::scheduled_lp(z),
                StatorArm { sched_lp: Some(z), sched_hp: Some(z), ..Default::default() }] {
        let core = st(arm);
        let (got, _) = core.stator_march(&f, &ramp(r), None, &StatorLeg::default());
        assert_eq!(got.len(), reference.len());
        for (i, (x, y)) in reference.iter().zip(got.iter()).enumerate() {
            for k in KEYS {
                assert_eq!(pt(x, k).to_bits(), pt(y, k).to_bits(), "{k} at row {i}");
            }
        }

        // --- the re-gate ---------------------------------------------------------------------
        scount::reset();
        let tt2 = core.fuel.inner.inlet(&f).0;
        core.fuel.inner.arm(0.8, 0.8, tt2);
        let c = scount::take();
        assert_eq!(c.arm_calls, 1, "the direct call must reach `arm` exactly once");
        assert_eq!(c.arm_lp_zero, 1, "a v_max = 0 LP schedule must take the ZERO arm");
        assert_eq!(c.arm_lp_moved, 0, "...and never the moved one");
        assert_eq!(core.fuel.inner.inner.map_lp(), core.arming().map_lp_design);
        assert_eq!(core.fuel.inner.inner.map_hp(), core.arming().map_hp_design);
    }
}

/// Python `test_cycle_untouched_rung6`.
#[test]
fn test_cycle_untouched_rung6() {
    let eng = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1600.0, 50_000.0, Losses {
        pi_d: 0.97, eta_c: 0.90, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, eta_m: 0.99,
        pi_n: 0.98, ..Losses::default()
    });
    assert!(eng.run(&flight(), 1.0).performance.specific_thrust > 0.0);
    assert_eq!(ComponentMap::flat().vsv, 0.0);
    assert!(ComponentMap::flat().is_flat());
}

// =============================================================================================
// The instrument
// =============================================================================================

/// Python `test_schedule_shape_and_guards`. `v(n_ref)` is EXACTLY 0 — asserted, not relied on.
#[test]
fn test_schedule_shape_and_guards() {
    let s = sched();
    assert_eq!(s.at(1.0), 0.0);
    assert_eq!(s.at(1.5), 0.0);
    assert_eq!(s.at(N_LO), V);
    assert_eq!(s.at(0.5), V);
    assert!(0.0 < s.at(0.9) && s.at(0.9) < s.at(0.8) && s.at(0.8) < V,
            "{} {} {V}", s.at(0.9), s.at(0.8));

    let lin = StatorSchedule::with_shape(V, N_LO, StatorSchedule::N_REF, Shape::Linear);
    assert_eq!(lin.at(1.0), 0.0);
    assert_eq!(lin.at(N_LO), V);
    assert_ne!(s.at(0.85), lin.at(0.85), "the two shapes must be genuinely different");

    // `n_lo >= n_ref`.
    refuses("needs n_lo < n_ref", || {
        StatorSchedule::new(V, 1.2);
    });
    // The bad shape STRING. A `Shape` enum makes this unrepresentable, so it has to go through
    // `try_from_str` — which exists for exactly this gate (§ 5.20's port note at `Shape`).
    refuses("shape must be", || {
        StatorSchedule::try_from_str(V, N_LO, StatorSchedule::N_REF, "cubic");
    });
}

/// Python `test_constructor_guards` — three of rung 57's four constructor asserts. (The fourth,
/// a BARE `lp_disabled`, is a Rust-only typing refusal and is gated at `lp_disabled`'s own
/// definition, not here; Python has no such call site.)
#[test]
fn test_constructor_guards() {
    refuses("capture discipline", || {
        st_maps(lp_map().with_vsv(0.1), hp_map(), StatorArm::default());
    });
    refuses("a CONSTANT setting or a SCHEDULE, not both", || {
        st(StatorArm { vsv_lp: V, sched_lp: Some(sched()), ..Default::default() });
    });
    refuses("not a reduce axis", || {
        st(StatorArm { vsv_lp: V, lp_disabled: true, ..Default::default() });
    });
}

/// Python `test_offmap_guard_is_an_assertion_not_a_typeerror`. RUNG 57's by-product: past
/// `phi_H ~ 4` the loading law gives `Tt3 < 0` and `pr_c` of a negative base returns a COMPLEX in
/// Python — which used to reach the bracket comparison as a `TypeError` that no caller catches.
/// In Rust the same state is a named refusal from rung 40's closure.
#[test]
fn test_offmap_guard_is_an_assertion_not_a_typeerror() {
    refuses("off-map compressor trial", || {
        st_maps(flat_lp(), flat_hp(), StatorArm::default()).fuel.inner.equilibrium(&flight(), LO);
    });
}

/// Python `test_currency_split_replays_on_the_transient`. Rung 53's headline, dynamically:
/// closing the stators SHRINKS the `phi` margin while it GROWS the incidence margin.
#[test]
fn test_currency_split_replays_on_the_transient() {
    let f = flight();
    let rp = ramp(0.5);
    let bare = st(StatorArm::default()).stator_transient_margin(&f, &rp);
    let shut = st(StatorArm::constant(V, 0.0)).stator_transient_margin(&f, &rp);
    assert!(shut.read.lp.m_phi < bare.read.lp.m_phi,
            "the wall moved further than the point: {} vs {}", shut.read.lp.m_phi,
            bare.read.lp.m_phi);
    assert!(shut.read.lp.m_i > bare.read.lp.m_i,
            "...but the METAL is further away: {} vs {}", shut.read.lp.m_i, bare.read.lp.m_i);
    assert!(shut.nu0_lp > bare.nu0_lp,
            "rung 53: paid in SHAFT SPEED: {} vs {}", shut.nu0_lp, bare.nu0_lp);
}

// =============================================================================================
// P1 / P2 — THE HEADLINE: no clock, and the non-tautology that makes it content
// =============================================================================================

/// P1's body, shared by the two `parametrize` cells.
///
/// THE LOAD-BEARING CLAUSE IS THE CLOSED FORM: rung 53's design-point Jacobian `1 - 1/(2+l)`
/// predicts the erosion within 10 % at every ramp rate. The `r`-invariance is a loose sanity cap
/// only — its pre-registered two-point band was HIT on the primary shape and MISSED on `tilted`,
/// and a threshold fitted to the observation would pin the number rather than test the claim.
fn p1_erosion(lp: ComponentMap, hp: ComponentMap) {
    let f = flight();
    let rows: Vec<_> = RATES.iter()
        .map(|&r| st_maps(lp, hp, StatorArm::constant(V, 0.0))
                      .stator_credit(&f, &ramp(r), Spool::Lp))
        .collect();
    let er: Vec<f64> = rows.iter().map(|x| x.erosion).collect();
    let cf = 1.0 - rows[0].closed_form;
    for (r, e) in RATES.iter().zip(er.iter()) {
        assert!((e - cf).abs() / cf < 0.10,
                "THE claim, at r = {r}: erosion {e} vs closed form {cf}, rel {}",
                (e - cf).abs() / cf);
    }
    let (lo, hi) = (er.iter().cloned().fold(f64::INFINITY, f64::min),
                    er.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    assert!(hi - lo < 0.05, "sanity cap, not the claim: spread {} over {er:?}", hi - lo);
    for (r, x) in RATES.iter().zip(rows.iter()) {
        assert!(0.0 < x.credit && x.credit < V, "real but partial credit at r = {r}: {}", x.credit);
        assert!(x.pointwise_exact, "a CONSTANT setting's pointwise leg must be exact at r = {r}");
        assert!((x.credit_pointwise - V).abs() < 1e-12,
                "the reference IS v at r = {r}: {}", x.credit_pointwise);
    }
}

/// Python `test_p1_erosion_is_r_invariant_and_matches_rung53_closed_form[LP-HP]` (`slow`).
#[test]
fn test_p1_erosion_is_r_invariant_and_matches_rung53_closed_form_primary() {
    p1_erosion(lp_map(), hp_map());
}

/// Python `…[TILT_LP-TILT_HP]` (`slow`).
#[test]
fn test_p1_erosion_is_r_invariant_and_matches_rung53_closed_form_tilted() {
    p1_erosion(tilt_map(), tilt_map());
}

/// Python `test_scheduled_erosion_is_flagged_as_a_different_quantity`. The API trap, closed.
#[test]
fn test_scheduled_erosion_is_flagged_as_a_different_quantity() {
    let f = flight();
    let rp = ramp(0.5);
    let c = st(StatorArm::constant(V, 0.0)).stator_credit(&f, &rp, Spool::Lp);
    let g = st(StatorArm::scheduled_lp(sched())).stator_credit(&f, &rp, Spool::Lp);
    assert!(c.pointwise_exact);
    assert!((c.credit_pointwise - V).abs() < 1e-12, "{}", c.credit_pointwise);
    assert!(!g.pointwise_exact, "a SCHEDULE's pointwise leg carries a different setting");
}

/// P2's body — THE NON-TAUTOLOGY. P1 is only content if the dynamics are doing something large
/// over the same sweep.
fn p2_swing(lp: ComponentMap, hp: ComponentMap) {
    let f = flight();
    let rows: Vec<_> = RATES.iter()
        .map(|&r| st_maps(lp, hp, StatorArm::constant(V, 0.0))
                      .stator_credit(&f, &ramp(r), Spool::Lp))
        .collect();
    let bare: Vec<f64> = rows.iter().map(|x| x.bare).collect();
    let (blo, bhi) = (bare.iter().cloned().fold(f64::INFINITY, f64::min),
                      bare.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    let swing = (bhi - blo) / blo;
    let er: Vec<f64> = rows.iter().map(|x| x.erosion).collect();
    let spread = er.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
        - er.iter().cloned().fold(f64::INFINITY, f64::min);
    assert!(swing > 0.30, "swing {swing} over {bare:?}");
    assert!(swing > 10.0 * spread, "swing {swing} vs 10x spread {}", 10.0 * spread);
}

/// Python `test_p2_the_margin_itself_swings_far_more_than_the_credit[LP-HP]` (`slow`).
#[test]
fn test_p2_the_margin_itself_swings_far_more_than_the_credit_primary() {
    p2_swing(lp_map(), hp_map());
}

/// Python `…[TILT_LP-TILT_HP]` (`slow`).
#[test]
fn test_p2_the_margin_itself_swings_far_more_than_the_credit_tilted() {
    p2_swing(tilt_map(), tilt_map());
}

// =============================================================================================
// P3 / P4 — where the credit is delivered, and the schedule's self-cancellation
// =============================================================================================

/// Python `test_p3_p4_credit_decomposition` (`slow`).
///
/// **BOTH ORDERING ASSERTIONS ARE NON-STRICT, AND THEY ARE PORTED THAT WAY.** Python's
/// `ss == sorted(ss, reverse=True)` is satisfied by an INERT sequence (slice S step 3's lesson),
/// so `>=` is what the Python gate says and `>=` is what runs here. The strictness is MEASURED
/// instead of assumed: the adjacent gaps are printed by the failure message, and the step-3
/// write-up carries the margins.
#[test]
fn test_p3_p4_credit_decomposition() {
    let f = flight();
    let rows: Vec<_> = RATES.iter()
        .map(|&r| st(StatorArm::scheduled_lp(sched())).credit_decomposition(&f, &ramp(r), Spool::Lp))
        .collect();
    for (r, x) in RATES.iter().zip(rows.iter()) {
        assert!(x.full > 0.0, "full credit at r = {r}: {}", x.full);
    }
    let ss: Vec<f64> = rows.iter().map(|x| x.share_start).collect();
    for (r, s) in RATES.iter().zip(ss.iter()) {
        assert!(*s < 0.35, "the head start delivers under 35 % at r = {r}: {s}");
    }
    for w in ss.windows(2) {
        assert!(w[0] >= w[1], "share_start must FALL with r: {ss:?}");
    }
    assert!(*ss.last().unwrap() < 0.0, "...and change sign: {ss:?}");
    assert!(ss[0] > 0.0, "{ss:?}");

    let sc: Vec<f64> = rows.iter().map(|x| x.self_cancel).collect();
    for (r, c) in RATES.iter().zip(sc.iter()) {
        assert!(0.0 < *c && *c < 1.0, "FULL below RAMP-ONLY, always — at r = {r}: {c}");
    }
    for w in sc.windows(2) {
        assert!(w[0] >= w[1], "the surrender must DEEPEN with r: {sc:?}");
    }
    for (r, x) in RATES.iter().zip(rows.iter()) {
        assert!(x.nu0_armed > x.nu0_bare,
                "the mechanism, at r = {r}: {} vs {}", x.nu0_armed, x.nu0_bare);
    }
}

/// Python `test_schedule_is_not_a_margin_lever_beside_a_constant` (`slow`). The honest bound on
/// the SCHEDULE: against a constant setting matched at the schedule's own surge minimum, the
/// schedule's residual is a small fraction of the credit.
#[test]
fn test_schedule_is_not_a_margin_lever_beside_a_constant() {
    let f = flight();
    let rp = ramp(0.5);
    let g = st(StatorArm::scheduled_lp(sched())).stator_credit(&f, &rp, Spool::Lp);
    let c = st(StatorArm::constant(g.v_at_min, 0.0)).stator_credit(&f, &rp, Spool::Lp);
    assert!((g.credit - c.credit).abs() < 0.25 * g.credit.abs(),
            "residual {} against 0.25 * |credit| {}", (g.credit - c.credit).abs(),
            0.25 * g.credit.abs());
}

// =============================================================================================
// P5 — the CROSS-RUNG CORRECTION of rung 53's two exact zeros
// =============================================================================================

/// Python `test_p5_rung53_exact_zeros_break_on_the_transient` (`slow`).
///
/// Rung 53 measured, on the STEADY cascade and with `==`, that `vsv_lp -> d_phi_HP` is EXACTLY
/// zero and that `vsv_hp -> d_phi_LP` is EXACTLY zero on a flat-eta island. Both break here, at a
/// FIXED transient state, and both breaks SURVIVE the island — so neither is the eta-mediated
/// channel. The channel is `Tt25`.
#[test]
fn test_p5_rung53_exact_zeros_break_on_the_transient() {
    let f = flight();
    let rp = ramp(0.5);
    let state = st(StatorArm::default()).arrow_toggle(&f, &rp, V, Spool::Lp, None).state;
    for (lp, hp) in [(lp_map(), hp_map()), (flat_lp(), flat_hp())] {
        let a = st_maps(lp, hp, StatorArm::default())
            .arrow_toggle(&f, &rp, V, Spool::Lp, Some(state));
        let b = st_maps(lp, hp, StatorArm::default())
            .arrow_toggle(&f, &rp, V, Spool::Hp, Some(state));
        assert!(a.d_phi_hp.abs() > 1e-3, "rung 53: exactly zero. got {}", a.d_phi_hp);
        assert!(b.d_phi_lp.abs() > 1e-3,
                "rung 53: exactly zero on flat eta. got {}", b.d_phi_lp);
        assert!(a.d_tt25.abs() > 1.0, "...and Tt25 names the channel: {}", a.d_tt25);
        assert!(a.d_phi_hp.abs() < a.d_phi_lp.abs(),
                "still a MINOR arrow, not a rewrite: {} vs {}", a.d_phi_hp.abs(),
                a.d_phi_lp.abs());
    }
}

/// Python `test_p5_the_arrow_is_not_eta_mediated` (`slow`). The control carrying P5's second
/// half: the flat-eta island reproduces the shaped one's arrow to within 5 %.
#[test]
fn test_p5_the_arrow_is_not_eta_mediated() {
    let f = flight();
    let rp = ramp(0.5);
    let state = st(StatorArm::default()).arrow_toggle(&f, &rp, V, Spool::Lp, None).state;
    let sh = st_maps(lp_map(), hp_map(), StatorArm::default())
        .arrow_toggle(&f, &rp, V, Spool::Lp, Some(state));
    let fl = st_maps(flat_lp(), flat_hp(), StatorArm::default())
        .arrow_toggle(&f, &rp, V, Spool::Lp, Some(state));
    assert!((fl.d_phi_hp - sh.d_phi_hp).abs() < 0.05 * sh.d_phi_hp.abs(),
            "gap {} against 5 % of {}", (fl.d_phi_hp - sh.d_phi_hp).abs(), sh.d_phi_hp.abs());
    assert!(fl.d_phi_hp * sh.d_phi_hp > 0.0, "same sign: {} {}", fl.d_phi_hp, sh.d_phi_hp);
}
