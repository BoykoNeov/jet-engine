//! RUNG 47 — THE LAGGED TOPPING GOVERNOR: a first-order lag is a TRAILING-edge tool, so it
//! cannot reach the EARLY LP surge minimum — and buying realism BREAKS rung 46's redline hold.
//!
//! Port of `tests/test_rung47.py`, gate for gate. That file defines **9 test functions** and
//! collects **9 items** — no `parametrize` — which is the middle term of § 5.17's `31 = 6 + 9 + 16`
//! counted with `--collect-only` rather than read off a header.
//!
//! | # | `tests/test_rung47.py` | here |
//! |---|---|---|
//! | 1 | `test_reduce_tau_none_bit_for_bit_rung46` | [`gate1_tau_none_is_the_instantaneous_rung46_governor`] |
//! | 2 | `test_reduce_dormant_lag_bit_for_bit_rung45` | [`gate2_dormant_lag_is_bit_for_bit_rung45`] |
//! | 3 | `test_reduce_lp_disabled_and_tau_needs_redline_assert` | [`gate3_lp_disabled_and_a_lag_without_a_redline_both_refuse`] |
//! | 4 | `test_decel_lagged_bit_for_bit_rung45` | [`gate4_decel_lagged_is_bit_for_bit_rung45`] |
//! | 5 | `test_cycle_untouched_by_lagged_governor_bit_for_bit_rung6` | [`gate5_cycle_untouched_bit_for_bit_rung6`] |
//! | 6 | `test_lagged_governor_overshoots_erodes_hp_and_misses_lp` | [`gate6_the_lag_overshoots_erodes_the_hp_and_misses_the_lp`] |
//! | 7 | `test_overshoot_grows_and_hp_erodes_monotone_in_tau` | [`gate7_the_cost_of_the_lag_is_monotone_in_tau`] |
//! | 8 | `test_fast_ramp_lp_relief_eroded_by_lag_never_enhanced` | [`gate8_fast_ramp_lp_relief_is_eroded_never_enhanced`] |
//! | 9 | `test_valve_lag_inert_topping_command_monotone` | [`gate9_the_topping_command_rises_so_a_valve_lag_is_inert`] |
//!
//! # ONE GAS, AND THAT IS THE DIFFERENCE FROM RUNG 46
//!
//! `test_rung46.py` runs gates 1/7/8 on CPG and switches gates 3-6 to `Gas.thermally_perfect()`,
//! which is why `rung46.rs` had to re-measure its cells on TPG before it could be written.
//! **`test_rung47.py` is CPG THROUGHOUT** — every `_ft` call takes `_cpg_gas()` — with the single
//! exception of gate 5's single-spool cycle object, which is `Gas.reacting_equilibrium()` as every
//! rung-6 reduce gate's is. § 5.17's four probes ran the CPG grid, so their measurements apply to
//! this file DIRECTLY rather than by analogy. *A census is a property of the grid*, and here the
//! grids coincide — which is a measurement, not a convenience.
//!
//! # Where Rust has to say out loud what Python leaves implicit
//!
//! * **GATE 1's byte-identity half is VACUOUS IN RUST, BY CONSTRUCTION.** Python compares
//!   `integrate_fuel(…, Tt4_max=REDLINE)` against `integrate_fuel(…, Tt4_max=REDLINE,
//!   tau_gov=None)` — two calls that differ by a keyword *defaulting to the value passed*. In Rust
//!   both build the SAME [`FuelLimiters`] value, so the comparison loop is `x == x` and the
//!   `topping_relief` pair below it is one call compared with itself. The port's own lesson —
//!   *a ported test can go VACUOUS* — says to name that rather than let a green tick imply
//!   coverage. What is LIVE in the gate is kept and labelled: the residue Python asserts last
//!   (`held`, `overshoot <= 1e-6`, measured `9.09e-13` here — a **1.1e6×** margin), plus one added
//!   line that a fall-through to the bare march WOULD break. See the gate.
//! * **GATE 2 IS THE ONE THAT PAYS, AND IT IS NOT VACUOUS.** `Tt4_max = huge, tau_gov = 0.3`
//!   dispatches into [`integrate_fuel_lagged`](turbojet::fuel_transient::FuelTransientCore::integrate_fuel_lagged)
//!   — a THREE-state RK4 — and demands it reproduce the two-state bare march bit-for-bit with
//!   `g ≡ 0`. This is the first gate any suite points at that method.
//! * **THE TWO REDUCE GATES COMPARE DIFFERENT KEY SETS, AND THE DIFFERENCE IS DELIBERATE.**
//!   Gate 1 compares **7** keys (`mf` INCLUDED); gate 2 compares **6** (`mf` excluded, as
//!   `test_rung46.py`'s own dormant gate does). Both are spelled as Python spells them.
//! * **`pytest.raises(AssertionError)` becomes a `catch_unwind` that reads the MESSAGE.** Rung
//!   45's precedent: a gate expecting *a* raise from two entry points must assert WHICH refusal
//!   escaped.
//! * **GATE 7's FIRST COMPARISON IS SATISFIED BY ITS SEED.** Python opens with
//!   `prev_ov, prev_hp = -1.0, 1.0`, and the first loop iteration compares `55.59 > -1.0` and
//!   `3.56e-3 < 1.0` — both trivially true. The gate's content is the **4 later strict steps**,
//!   measured `+39.7 / +42.2 / +32.7 / +20.7` K and `-6.87e-4 / -7.38e-4 / -7.26e-4 / -5.50e-4`.
//!   The seeds are reproduced anyway, because changing them would be a different test.
//! * **GATE 9's TWO ASSERTIONS ARE LOAD-BEARING ONLY TOGETHER.** `monotone_nondecreasing` is
//!   VACUOUSLY true on 0 or 1 engaged points (Python's `all(...)` over an empty range, which
//!   [`windows(2)`](slice::windows) reproduces), so it is the `n_engaged > 10` bar that stops the
//!   monotone flag from being free. Measured **45** engaged points here, a 4.5× margin.
//!
//! # What this file adds that rung 46 could not
//!
//! § 5.17 step 1 finding 2 measured that rung 46's LP half **cannot gate its own sign**: a flipped
//! `relief_lp` is invisible there because the quantity is exactly `0.0` at moderate `r`, and the
//! one gate that carries the sign is rung 46's fast-ramp lever. [`gate8…`](gate8_fast_ramp_lp_relief_is_eroded_never_enhanced)
//! asserts `0.0 < relief_lp < prev` at four `tau_gov` values on a STRICTLY POSITIVE quantity
//! (measured `1.51e-2 → 1.03e-2 → 6.03e-3 → 3.17e-3`), so it is a second LP-sign carrier and it is
//! not one of the two gates Python marks `slow`. That partially closes the gap step 1 booked.
//!
//! # `#[ignore]`
//!
//! `test_rung47.py` marks **nothing** `slow` — § 5.17 counted the slice's two slow marks as rung
//! 46's pair — so there is no marker question to decide here. The file's own measured runtime is
//! recorded in the plan, not guessed.

use std::panic::{catch_unwind, AssertUnwindSafe};

use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::fuel_transient::{FuelLimiters, FuelPoint, PhiExcursionFuel, TwoSpoolFuelTransient};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;

/// The accel band and a redline in the GAP — above the 1400 endpoint, below the bare peak.
///
/// **`test_rung47.py:88` sites that peak at `~1670` and `test_rung46.py:79` at `~1645`, on a
/// BYTE-IDENTICAL grid.** § 5.17 finding 6 measured it over the four shapes: **1690.5 / 1695.4 /
/// 1702.4 / 1703.0**, and this file's own gate-1 cell re-measures `1695.4058398939349`. Neither
/// quoted figure matches any shape and the two files cannot both be right about one march. No gate
/// reads the number — 1480 clears every peak by more than 200 K — so it is a doc correction booked
/// to step 4, carried here so the next reader does not re-derive it.
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const REDLINE: f64 = 1480.0;
const R: f64 = 0.5;
/// `test_rung47.py:89`. The surge min and the `Tt4` peak both live inside the ramp, so rung 45's
/// `6.0` is not needed.
const SETTLE: f64 = 2.0;
/// `topping_relief` / `phi_excursion_fuel` / `integrate_fuel`'s step — the suite writes `0.02`
/// explicitly at the two `integrate_fuel` calls and leans on the same value as a default
/// everywhere else (`engine.py:5439`, `:5346`).
const DS: f64 = 0.02;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn real() -> TwoSpoolLosses {
    TwoSpoolLosses {
        pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
        eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98,
        p_exit: None, nozzle_convergent: true,
    }
}

/// `test_rung47.py`'s `SINGLE`. No `nozzle_convergent`, which is admissible for the rung-6 cycle
/// gate that is its only consumer.
fn single() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.90, e_c: None, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, e_t: None,
        eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: false,
    }
}

/// [`single`] plus the one constant gate 3 needs to have a degenerate object at all.
fn single_matchable() -> Losses {
    Losses { nozzle_convergent: true, ..single() }
}

/// `test_rung47.py`'s `_cpg_gas` — `R_c`/`R_t` DERIVED from the pair above them, as rungs 45/46
/// spell it and unlike `test_rung43.py`'s literal `286.9`.
fn cpg_gas() -> Gas {
    let (gc, cc, gt, ct) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64);
    Gas::new(GasSpec {
        gamma_c: gc, cp_c: cc, r_c: (gc - 1.0) / gc * cc,
        gamma_t: gt, cp_t: ct, r_t: (gt - 1.0) / gt * ct,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp_shaped() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
}

fn hp_shaped() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
}

/// `SHAPES`, in Python's dict order — `hp-only` (LP FLAT ⇒ no rung-40 complex mode) last, because
/// it is the discriminator gate 6 leans on.
fn shapes() -> [(&'static str, ComponentMap, ComponentMap); 4] {
    let f = ComponentMap::flat();
    let m = |a: f64, b: f64, c: f64, sigma: f64, l: f64| ComponentMap { a, b, c, sigma, l, ..f };
    let tilted = m(0.14, 0.10, 0.06, 0.2, 0.85);
    [
        ("flow/press", lp_shaped(), hp_shaped()),
        ("press/flow", m(0.05, 0.20, 0.0, 0.1, 1.0), m(0.20, 0.05, 0.0, 0.1, 0.7)),
        ("tilted", tilted, tilted),
        ("hp-only", f, hp_shaped()),
    ]
}

fn design(gas: Gas) -> TwoSpoolEngine {
    build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, 50_000.0, real())
}

fn ft(d: &TwoSpoolEngine, map_lp: ComponentMap, map_hp: ComponentMap, rho: f64)
    -> TwoSpoolFuelTransient
{
    TwoSpoolFuelTransient::new(d.clone(), flight(), 1.0, map_lp, map_hp, rho)
}

/// The nine fields of [`PhiExcursionFuel`] as raw bits, by EXHAUSTIVE destructure — rung 45/46's
/// helper, repeated rather than shared because integration-test crates do not link to each other.
/// A tenth field breaks the build instead of silently narrowing the comparison.
fn phi_exc_bits(e: &PhiExcursionFuel) -> [u64; 9] {
    let PhiExcursionFuel { ext_lp, ext_hp, s_lp, s_hp, min_phi_lp, min_phi_hp, tt4_peak, ratio,
                           npts } = *e;
    [ext_lp.to_bits(), ext_hp.to_bits(), s_lp.to_bits(), s_hp.to_bits(), min_phi_lp.to_bits(),
     min_phi_hp.to_bits(), tt4_peak.to_bits(), ratio.to_bits(), npts as u64]
}

/// Gate 2's SIX-key tuple, in Python's order — `mf` deliberately OUTSIDE, as `test_rung47.py:141`
/// spells it.
fn six(p: &FuelPoint) -> [u64; 6] {
    [p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.phi_lp.to_bits(), p.phi_hp.to_bits(),
     p.tt4.to_bits(), p.f.to_bits()]
}

/// Gate 1's SEVEN-key tuple — the same six plus `mf`, which `test_rung47.py:117` includes and
/// gate 2 does not.
fn seven(p: &FuelPoint) -> [u64; 7] {
    let s = six(p);
    [s[0], s[1], s[2], s[3], s[4], s[5], p.mf.to_bits()]
}

/// The message of an `assert!` that fired, or `None` if the call returned. Rung 45/46's helper, and
/// its caveat travels with it: this swaps the GLOBAL panic hook, so the restore can race a parallel
/// test's backtrace output — it cannot change a `catch_unwind` RESULT.
fn refusal<F: FnOnce()>(f: F) -> Option<String> {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let r = catch_unwind(AssertUnwindSafe(f));
    std::panic::set_hook(hook);
    match r {
        Ok(()) => None,
        Err(e) => Some(
            e.downcast_ref::<String>().cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_string()))
                .unwrap_or_else(|| "<non-string panic>".to_string()),
        ),
    }
}

// ================================================================================== gate 1
/// GATE 1 — REDUCE: `tau_gov = None` IS the idealised instantaneous rung-46 min-select.
///
/// **THE COMPARISON HALVES ARE VACUOUS IN RUST AND THE HEADER SAYS WHY.** Python's two calls
/// differ by a keyword whose default equals the value passed; Rust builds one [`FuelLimiters`]
/// value for both, so the trajectory loop compares a call with itself and so does the
/// `topping_relief` pair. They are written out anyway — deleting them would hide that Python has a
/// gate here — but nothing about the port is established by their passing.
///
/// **WHAT IS LIVE IS BELOW THE FOLD, AND IT IS WHAT PYTHON ENDS ON.** The instantaneous governor
/// must HOLD the redline. Measured on THIS grid (CPG, `flow/press`): `overshoot = 9.09e-13`
/// against a `1e-6` bar — a **1.1e6×** margin, § 5.17 finding 2's "machine zero or 54.7 K, nothing
/// between". `test_rung46.py` gates that hold on `Gas.thermally_perfect()` only, so this is the
/// CPG witness of it and not a repeat.
///
/// **ONE ADDED LINE, BECAUSE THE VACUITY LEAVES A REAL HOLE.** If the `(Some, Some)` dispatch in
/// [`integrate_fuel`](turbojet::fuel_transient::FuelTransientCore::integrate_fuel) were mis-spelled
/// so that a `tau_gov = None` call fell through to the BARE march, every assertion Python writes
/// here would still pass — the two vacuous halves trivially, and `held` because a bare peak
/// compared against nothing is not read. So the applied fuel is checked to have been CLIPPED below
/// the schedule at least once, which a bare march cannot do. Measured: the bare peak is 1695.4 K
/// against a topped 1480.0 K, so the fall-through is far from a knife-edge.
#[test]
fn gate1_tau_none_is_the_instantaneous_rung46_governor() {
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let f = flight();
    let core = t.core();
    let (mf0, mf1) = (core.fuel_for_tt4(&f, LO), core.fuel_for_tt4(&f, HI));
    let eq0 = core.inner.equilibrium(&f, LO);
    let nu0 = (eq0.nu_lp, eq0.nu_hp);
    let sched = |s: f64| mf0 + (mf1 - mf0) * (s / R).min(1.0);

    // VACUOUS: `governed` and Python's `tau_gov=None` call are the same value.
    let governed = FuelLimiters { tt4_max: Some(REDLINE), ..Default::default() };
    let pa = core.integrate_fuel(&f, sched, nu0, R + SETTLE, DS, &governed);
    let pb = core.integrate_fuel(&f, sched, nu0, R + SETTLE, DS, &governed);
    assert_eq!(pa.len(), pb.len());
    for (a, b) in pa.iter().zip(pb.iter()) {
        assert_eq!(seven(a), seven(b), "at s={}", a.s);
    }

    // LIVE: the governor actually clipped. A fall-through to the bare march cannot produce this.
    let clipped = pa.iter().filter(|p| p.mf < p.mf_sched).count();
    assert!(clipped > 0,
            "tau_gov=None must still ARM the governor -- no point was clipped below the schedule");
    let peak = pa.iter().fold(f64::NEG_INFINITY, |m, p| m.max(p.tt4));
    assert!(peak <= REDLINE + 1e-6,
            "the instantaneous governor must pin the marched peak at the redline ({peak})");

    // VACUOUS again: one `topping_relief` call compared with itself.
    let r46 = t.topping_relief(&f, LO, HI, REDLINE, R, SETTLE, DS, None);
    let r_none = t.topping_relief(&f, LO, HI, REDLINE, R, SETTLE, DS, None);
    assert_eq!(
        (r46.relief_lp.to_bits(), r46.relief_hp.to_bits(), r46.tt4_peak_top.to_bits(), r46.held),
        (r_none.relief_lp.to_bits(), r_none.relief_hp.to_bits(), r_none.tt4_peak_top.to_bits(),
         r_none.held));

    // LIVE: Python's last line, and the CPG witness of rung 46's TPG-only hold.
    assert!(r46.held && r46.overshoot <= 1e-6,
            "the INSTANTANEOUS governor holds the redline ({})", r46.overshoot);
}

// ================================================================================== gate 2
/// GATE 2 — REDUCE: a DORMANT lag is bit-for-bit rung 45, through the THREE-state marcher.
///
/// **THIS IS THE GATE THAT PAYS.** A redline above the bare peak leaves the clip un-consulted, so
/// the required clip is 0, so the third state `g` stays at `0.0` and the applied fuel is
/// `mf_sched - 0.0`. The claim is that the three-state RK4 in
/// [`integrate_fuel_lagged`](turbojet::fuel_transient::FuelTransientCore::integrate_fuel_lagged)
/// then reproduces the two-state bare march **float-for-float**, not merely to tolerance — and
/// this is the first coverage any suite puts on that method (§ 5.17: slice S shipped it with
/// ~40% of the limiter machinery ungated).
///
/// It fails on a `mf_sched + g`, on any mis-weight of the `a`/`b` RK4 stages introduced by the
/// third one, and on a `required` that returns non-zero while dormant. The exactness is real
/// rather than incidental: `dg = (0 - 0)/tau` is exactly `0.0` at every stage, so `g` never leaves
/// zero and `mf_sched - 0.0` is the identity.
///
/// Six keys, `mf` outside — Python's tuple.
#[test]
fn gate2_dormant_lag_is_bit_for_bit_rung45() {
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let f = flight();
    let core = t.core();
    let (mf0, mf1) = (core.fuel_for_tt4(&f, LO), core.fuel_for_tt4(&f, HI));
    let eq0 = core.inner.equilibrium(&f, LO);
    let nu0 = (eq0.nu_lp, eq0.nu_hp);
    let sched = |s: f64| mf0 + (mf1 - mf0) * (s / R).min(1.0);

    let bare = core.integrate_fuel(&f, sched, nu0, R + SETTLE, DS, &FuelLimiters::default());
    let huge = bare.iter().fold(f64::NEG_INFINITY, |m, p| m.max(p.tt4)) + 500.0;
    let dormant = FuelLimiters { tt4_max: Some(huge), tau_gov: Some(0.3), ..Default::default() };
    let lagged = core.integrate_fuel(&f, sched, nu0, R + SETTLE, DS, &dormant);
    assert_eq!(bare.len(), lagged.len());
    for (a, b) in bare.iter().zip(lagged.iter()) {
        assert_eq!(six(a), six(b), "a dormant lag must not move the march, at s={}", a.s);
    }
}

// ================================================================================== gate 3
/// GATE 3 — REDUCE: the lag REFUSES a degenerate engine, and refuses to exist without a redline.
///
/// Two refusals from two entry points, and each names its own reason:
///
/// * `lp_disabled` — the finding is a SPLIT between spools, which a single-shaft engine cannot
///   state, so this is not a reduce axis (rung 46's contract, inherited);
/// * `tau_gov` with no `Tt4_max` — a governor lag with no governor to lag is meaningless.
///
/// Both fire before any marching, so neither reaches the phase-8 assert-vs-panic divergence
/// `rung46.rs`'s disclosed-divergence test books.
#[test]
fn gate3_lp_disabled_and_a_lag_without_a_redline_both_refuse() {
    let f = flight();

    let se: Engine = build_turbojet(cpg_gas(), 10.0, TT4, 50_000.0, single_matchable());
    let deg = TwoSpoolFuelTransient::lp_disabled(se, flight(), 1.0, hp_shaped());
    let governed_lagged =
        FuelLimiters { tt4_max: Some(REDLINE), tau_gov: Some(0.2), ..Default::default() };
    let m1 = refusal(|| {
        deg.integrate_fuel_lp_disabled(&f, |_s| 0.5, 1.0, 1.0, 0.05, &governed_lagged);
    })
    .expect("a lagged governor on an lp_disabled object must refuse");
    assert!(m1.contains("two-shaft"), "the refusal must name the reason: {m1}");

    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let lag_no_redline = FuelLimiters { tau_gov: Some(0.2), ..Default::default() };
    let m2 = refusal(|| {
        t.core().integrate_fuel(&f, |_s| 0.5, (1.0, 1.0), 1.0, 0.05, &lag_no_redline);
    })
    .expect("tau_gov without Tt4_max must refuse");
    assert!(m2.contains("needs a redline"), "the refusal must name the reason: {m2}");
}

// ================================================================================== gate 4
/// GATE 4 — DECEL: the clip never fires, so the lagged decel is bit-for-bit rung 45, every shape.
///
/// The topping governor is an ACCELERATION limiter. On a decel `Tt4` undershoots, the required
/// clip stays 0, `g` stays 0, and the lagged march equals the bare one float-for-float — the same
/// mechanism as gate 2, reached from the other direction and swept over all four map shapes.
#[test]
fn gate4_decel_lagged_is_bit_for_bit_rung45() {
    let d = design(cpg_gas());
    let f = flight();
    for (name, ml, mh) in shapes() {
        let t = ft(&d, ml, mh, 1.0);
        let bare = t.phi_excursion_fuel(&f, HI, LO, R, SETTLE, DS, None, None, None, None);
        let top = t.phi_excursion_fuel(&f, HI, LO, R, SETTLE, DS, Some(REDLINE), Some(0.3), None,
                                       None);
        assert_eq!(phi_exc_bits(&top), phi_exc_bits(&bare),
                   "{name}: decel must never fire the clip, even lagged");
    }
}

// ================================================================================== gate 5
/// GATE 5 — CYCLE UNTOUCHED: exercising the lagged governor does not perturb the rung-6 design run.
///
/// `s_settle = 1.5` here, not this file's `SETTLE = 2.0` — `test_rung47.py:180` writes it that way
/// and nothing numeric turns on it, since the gate reads the SINGLE-spool engine either side.
#[test]
fn gate5_cycle_untouched_bit_for_bit_rung6() {
    let eng: Engine = build_turbojet(Gas::reacting_equilibrium(), 10.0, TT4, 50_000.0, single());
    let f = flight();
    let a = eng.run(&f, 1.0);
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    t.topping_relief(&f, LO, HI, REDLINE, R, 1.5, DS, Some(0.2));
    let b = eng.run(&f, 1.0);
    assert_eq!(a.performance.specific_thrust.to_bits(), b.performance.specific_thrust.to_bits());
    assert_eq!(a.station("4").far.to_bits(), b.station("4").far.to_bits());
}

// ================================================================================== gate 6
/// GATE 6 — THE HEADLINE: the lag OVERSHOOTS, ERODES the HP rebate, and STILL misses the LP.
///
/// At every shape including the mode-free `hp-only`, with `tau_gov = 0.2`:
///
/// * **the hold is LOST** — `held` false and `overshoot > 1.0` K. Measured **134.6–141.1 K**, a
///   134× margin on the bar. This is the cost of realism, and the inversion of rung 46's gate 3.
/// * **`relief_lp` stays EXACTLY `0.0`** — the refutation. A first-order lag is a TRAILING-edge
///   tool; it delays the governor's action and never anticipates it, so it cannot reach an LP
///   surge minimum that sits UPSTREAM of engagement.
/// * **`relief_hp` stays positive but is ERODED** below the instantaneous rebate. Measured
///   `2.14e-3 < 5.27e-3` (`flow/press`) through `1.99e-3 < 5.04e-3` (`hp-only`) — a ratio of
///   **0.40–0.45** at every shape, with the smallest gap `3.05e-3`, four orders above any
///   float-noise concern.
///
/// `hp-only` (LP map FLAT ⇒ no rung-40 complex inter-spool mode) witnesses that the refutation is
/// the WINDOW/timing mechanism and not a mode artifact. Magnitudes are disclaimed; the SIGNS are
/// gated.
#[test]
fn gate6_the_lag_overshoots_erodes_the_hp_and_misses_the_lp() {
    let d = design(cpg_gas());
    let f = flight();
    for (name, ml, mh) in shapes() {
        let t = ft(&d, ml, mh, 1.0);
        let inst = t.topping_relief(&f, LO, HI, REDLINE, R, SETTLE, DS, None);
        let lag = t.topping_relief(&f, LO, HI, REDLINE, R, SETTLE, DS, Some(0.2));
        assert!(!lag.held && lag.overshoot > 1.0,
                "{name}: a lagged governor must OVERSHOOT the redline ({})", lag.overshoot);
        assert!(lag.relief_lp.abs() < 1e-9,
                "{name}: the lag still misses the early LP min ({})", lag.relief_lp);
        assert!(0.0 < lag.relief_hp && lag.relief_hp < inst.relief_hp,
                "{name}: HP rebate positive but ERODED vs the instantaneous governor ({} vs {})",
                lag.relief_hp, inst.relief_hp);
    }
}

// ================================================================================== gate 7
/// GATE 7 — THE COST OF THE LAG IS MONOTONE IN `tau_gov`.
///
/// On `flow/press` at `r = 0.5`, as the governor gets slower: the overshoot GROWS, the HP rebate
/// ERODES, and `relief_lp` stays pinned at `0.0`. Rung 46's gate 3 inverted, resolved as a knob.
///
/// **THE FIRST OF THE FIVE COMPARISONS IS SATISFIED BY THE SEED, NOT BY THE PHYSICS.** Python
/// opens at `prev_ov = -1.0, prev_hp = 1.0`, so iteration one asserts `55.59 > -1.0` and
/// `3.56e-3 < 1.0`. The content is the four LATER strict steps: overshoot `+39.7 / +42.2 / +32.7 /
/// +20.7` K and rebate `-6.87e-4 / -7.38e-4 / -7.26e-4 / -5.50e-4`. The seeds are reproduced as
/// Python writes them, because changing them would be a different test — but a reader should not
/// have to discover that the first row is free.
#[test]
fn gate7_the_cost_of_the_lag_is_monotone_in_tau() {
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let f = flight();
    // Python's seeds. The first comparison against each is trivially true — see the doc comment.
    let (mut prev_ov, mut prev_hp) = (-1.0f64, 1.0f64);
    for tau in [0.05, 0.1, 0.2, 0.4, 0.8] {
        let o = t.topping_relief(&f, LO, HI, REDLINE, R, SETTLE, DS, Some(tau));
        assert!(o.overshoot > prev_ov, "overshoot must grow with tau ({tau}, {})", o.overshoot);
        assert!(o.relief_hp < prev_hp, "HP rebate must erode with tau ({tau}, {})", o.relief_hp);
        assert!(o.relief_lp.abs() < 1e-9, "relief_lp pinned at 0 ({tau}, {})", o.relief_lp);
        prev_ov = o.overshoot;
        prev_hp = o.relief_hp;
    }
}

// ================================================================================== gate 8
/// GATE 8 — THE LEVER, LAGGED: at fast `r` the lag ERODES rung 46's LP relief, never enhances it.
///
/// **THE AIRTIGHT HALF OF THE REFUTATION.** At `r = 0.15` rung 46's INSTANTANEOUS governor DOES
/// reach the LP (`relief_lp = 2.69e-2`, 27× its `1e-3` bar). If a lag could "reach earlier into
/// the LP surge point" — rung 46's own next-seam hope — the lagged relief would EXCEED it. It does
/// the opposite, monotonically: measured `1.51e-2 → 1.03e-2 → 6.03e-3 → 3.17e-3` across
/// `tau_gov = 0.05 … 0.4`, every value strictly positive and strictly below its predecessor. So in
/// BOTH regimes the lag reaches the LP no better than the ideal min-select — neutral at moderate
/// `r`, strictly WORSE at fast `r`.
///
/// **AND THIS IS THE FILE'S SECOND CONTRIBUTION, THE ONE RUNG 46 COULD NOT MAKE.** § 5.17 step 1
/// finding 2 measured rung 46's LP half blind to its own SIGN, because `relief_lp` is exactly
/// `0.0` there and a sign flip on an exact zero is invisible; only rung 46's fast-ramp gate — one
/// of its two `slow` ones — carries it. Here `relief_lp` is strictly positive at four `tau_gov`
/// values and the assertion is two-sided, so a sign flip fails four times over, in a gate carrying
/// no `slow` mark.
///
/// The `overshoot > 100.0` bar is the TIGHTEST in the file: measured **218.9 K** at `tau = 0.05`,
/// a 2.19× margin, rising to 388.5 K at `tau = 0.4`. Quoted as measured on this grid rather than
/// inherited from § 5.17's sweep.
#[test]
fn gate8_fast_ramp_lp_relief_is_eroded_never_enhanced() {
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let f = flight();
    let red = 1440.0;
    let inst = t.topping_relief(&f, LO, HI, red, 0.15, SETTLE, DS, None);
    assert!(inst.relief_lp > 1e-3,
            "rung 46 reaches the LP at fast r ({})", inst.relief_lp);
    let mut prev = inst.relief_lp;
    for tau in [0.05, 0.1, 0.2, 0.4] {
        let o = t.topping_relief(&f, LO, HI, red, 0.15, SETTLE, DS, Some(tau));
        assert!(0.0 < o.relief_lp && o.relief_lp < prev,
                "the lag ERODES the LP relief, never enhances it ({tau}, {}, prev {prev})",
                o.relief_lp);
        assert!(o.overshoot > 100.0,
                "and it overshoots hugely at fast r ({tau}, {})", o.overshoot);
        prev = o.relief_lp;
    }
}

// ================================================================================== gate 9
/// GATE 9 — THE SECONDARY: the topping command RISES, so a metering-VALVE lag is INERT.
///
/// The overshoot is not a property of *any* lag — it is a property of WHERE the lag lives. Once
/// the governor engages, the binding topping command rises monotonically (speed up ⇒ airflow up ⇒
/// more fuel needed to hold the same redline), and an instant-up valve tracks a rising command
/// with no lag at all. So the topping overshoot lives specifically in the sensing / limiter-LOOP
/// lag, which lags the clip AMOUNT — the `g` state gate 2 exercises — and not in the valve.
///
/// **THE TWO BARS ARE LOAD-BEARING ONLY TOGETHER.** `monotone_nondecreasing` is VACUOUSLY true on
/// 0 or 1 engaged points, so without `n_engaged > 10` the monotone flag would be free. Measured
/// **45** engaged points spanning `s = 0.30 … 1.18` — a 4.5× margin — with the command rising
/// `0.01766 → 0.02334` and the SMALLEST step `8.88e-5` against the `-1e-12` bar, § 5.17 finding
/// 2's 8.9e7× slack.
#[test]
fn gate9_the_topping_command_rises_so_a_valve_lag_is_inert() {
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let f = flight();
    let tr = t.core().topping_command_trace(&f, LO, HI, REDLINE, R, SETTLE, DS);
    assert!(tr.n_engaged > 10,
            "the clip must engage over a real window ({})", tr.n_engaged);
    assert!(tr.monotone_nondecreasing, "the binding topping command must rise monotonically");
    assert!(tr.engaged[tr.engaged.len() - 1].1 > tr.engaged[0].1,
            "the command genuinely rises across the window");
}
