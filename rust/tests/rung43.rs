//! RUNG 43 — TWO-SHAFT FUEL METERING: the two spools sit at DIFFERENT points in ONE overshoot
//! loop, so NEITHER clock governs it.
//!
//! Port of `tests/test_rung43.py`, gate for gate. That file names **10 gates**, defines **11 test
//! functions** and collects **11 items** — no `parametrize` anywhere, and the eleventh function is
//! the scope concession (`test_reacting_gas_fuel_control_is_refused`) rather than a gate. § 5.16
//! counted the pair as **20 items, 11 + 9** with `--collect-only` rather than off a header, and
//! this file is the 11.
//!
//! | # | `tests/test_rung43.py` | here |
//! |---|---|---|
//! | 1 | `test_reduce_control_invariance_is_rung40_point` | [`gate1_control_invariance_is_the_rung40_point`] |
//! | 2 | `test_reduce_lp_disabled_is_rung35_fuel_path_bit_for_bit` | [`gate2_lp_disabled_is_rung35s_fuel_path_bit_for_bit`] |
//! | 3 | `test_reduce_tt4_control_untouched_is_rung40_bit_for_bit` | [`gate3_tt4_control_untouched_is_rung40_bit_for_bit`] |
//! | 4 | `test_reduce_settle_lands_on_the_equilibrium` | [`gate4_settle_lands_on_the_equilibrium`] |
//! | 5 | `test_finding_mechanism_both_spools_relieve_the_overshoot` | [`gate5_both_spools_relieve_the_overshoot`] |
//! | 6 | `test_finding_lp_frozen_is_the_rho_free_ceiling` | [`gate6_lp_frozen_is_the_rho_free_ceiling`] |
//! | 7 | `test_finding_overshoot_rises_monotonically_with_rho` | [`gate7_overshoot_rises_monotonically_with_rho`] |
//! | 8 | `test_inherited_tit_limited_before_surge` | [`gate8_tit_limited_before_surge`] |
//! | 9 | `test_withdrawn_no_effective_clock_ratio` | [`gate9_the_withdrawn_effective_clock_ratio`] |
//! | 10 | `test_cycle_untouched_rung6_bit_for_bit` | [`gate10_cycle_untouched_rung6_bit_for_bit`] |
//! | — | `test_reacting_gas_fuel_control_is_refused` | [`concession_reacting_gas_fuel_control_is_refused`] |
//!
//! # Where Rust has to say out loud what Python leaves implicit
//!
//! * **`R_c = 286.9`, WRITTEN AS THE LITERAL IT IS.** `test_rung43.py:62` hard-codes it;
//!   `test_rung45.py:83` writes `(gamma_c-1)/gamma_c*cp_c = 286.8571428571428`. The two suites
//!   therefore run DIFFERENT cold sections, one rung apart, and § 5.16 probe 1 measured that the
//!   whole 400-key fuel-path dump is bit-identical across the two — only a THRUST key witnesses
//!   the difference at all. Slice R step 3 shipped rung 40's constant into `rung44.rs` and no gate
//!   in that file could see it. So each suite's gas is built from its OWN expression here, and
//!   this comment is the reason.
//! * **Three silent defaults.** `ramp_excursion_fuel` and `freeze_channels` both default to
//!   `s_settle = 8.0, ds = 0.02` (`engine.py:5180`, `:5237`) and `collapse_exponent` to `nb = 6`
//!   (`:5261`); the suite never names any of them. Rust has no defaults, so every call below
//!   writes them out.
//! * **`==` on a returned record.** Python compares floats out of dicts with `==`; the records
//!   here have no `PartialEq`, so the exact comparisons are on `to_bits()`, which is STRICTER
//!   (it separates `-0.0` from `0.0`).
//! * **The refusal is a `Result`, not a panic.** § 5.16 probe 4 (A) measured that Python's
//!   `_tt4_from_f` assert fires INSIDE the closure's bracket scan, which swallows it — so the port
//!   makes it an [`Abort`](turbojet::gas::Abort) that the scan can catch, and the concession gate
//!   below pokes it directly and asserts its IDENTITY through
//!   [`classify`](turbojet::fuel_transient::classify) rather than only its message.
//! * **`equilibrium_fuel` on the degenerate object needed a Rust home.** Python's forwards to the
//!   held rung-35 `SpoolTransient` and silently DROPS the `start` the caller passed; the return
//!   type changes across that dispatch, so gate 2 goes through
//!   `TwoSpoolFuelTransient::equilibrium_fuel_lp_disabled`, added for this gate and carrying the
//!   argument-drop in its doc comment.

use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::fuel_transient::{classify, FuelAbort, FuelLimiters, TwoSpoolFuelTransient};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::spool::SpoolTransient;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::TwoSpoolTransientCore;

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;

/// Rung 35's own step, so the two rungs' excursions are apples-to-apples.
const LO: f64 = 1250.0;
const HI: f64 = 1450.0;

/// Python's `ramp_excursion_fuel` / `freeze_channels` defaults.
const S_SETTLE: f64 = 8.0;
const DS: f64 = 0.02;
/// Python's `collapse_exponent` default bin count.
const NB: usize = 6;

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

/// `test_rung43.py`'s `SINGLE` — note `eta_t = 0.92` (the HPT's), and `nozzle_convergent`.
fn single() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.88, e_c: None, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, e_t: None,
        eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
    }
}

/// `test_rung43.py`'s `_cpg_gas` — `R_c` the LITERAL `286.9`, `R_t` derived. See the header.
fn cpg_gas() -> Gas {
    let (g, cp) = (1.3f64, 1239.0f64);
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9,
        gamma_t: g, cp_t: cp, r_t: (g - 1.0) / g * cp,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp_shaped() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
}

fn hp_shaped() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
}

/// `SHAPES`, in Python's dict order — gates 5 and 6 read the first and the third by NAME.
fn shapes() -> [(&'static str, ComponentMap, ComponentMap); 3] {
    let m = |a: f64, b: f64, c: f64, sigma: f64, l: f64| ComponentMap {
        a, b, c, sigma, l, ..ComponentMap::flat()
    };
    let tilted = m(0.14, 0.10, 0.06, 0.2, 0.85);
    [
        ("flow/press", lp_shaped(), hp_shaped()),
        ("press/flow", m(0.05, 0.20, 0.0, 0.1, 1.0), m(0.20, 0.05, 0.0, 0.1, 0.7)),
        ("tilted", tilted, tilted),
    ]
}

/// The two shape pairs gates 5 and 6 sweep — `flow/press` and `tilted`, NOT `press/flow`.
fn two_shapes() -> [(&'static str, ComponentMap, ComponentMap); 2] {
    let [fp, _, ti] = shapes();
    [fp, ti]
}

fn design(gas: Gas) -> TwoSpoolEngine {
    build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, 50_000.0, real())
}

/// `test_rung43.py`'s `_ft`. Python rebuilds `_design(gas)` on every call; the build is a pure
/// function of its arguments, so a clone of one build carries the same numbers — `rung44.rs`'s
/// shape.
///
/// **THE CLONE IS NOT FREE, AND GATE 3 IS WHERE IT COSTS.** Python's gate 3 hands ONE design object
/// to both the rung-40 transient and the fuel transient specifically so that a MUTATION of the
/// design by the fuel path would surface in the rung-40 side; cloning severs that channel and
/// leaves only "the same inputs give the same numbers". Same honesty as gate 10's note: the gate is
/// THINNER here than in Python, and Rust's shared-`&`/`Clone` discipline is what makes the channel
/// hard to open in the first place rather than anything gate 3 checks.
fn ft(d: &TwoSpoolEngine, map_lp: ComponentMap, map_hp: ComponentMap, rho: f64)
    -> TwoSpoolFuelTransient
{
    TwoSpoolFuelTransient::new(d.clone(), flight(), 1.0, map_lp, map_hp, rho)
}

/// The default `_ft(gas)` — `LP_SHAPED`, `HP_SHAPED`, `rho = 1.0`.
fn ft_default(d: &TwoSpoolEngine) -> TwoSpoolFuelTransient {
    ft(d, lp_shaped(), hp_shaped(), 1.0)
}

// ------------------------------------------------------------------------------------ gate 1
/// GATE 1 — a steady point is the SAME however it is named.
///
/// NON-TAUTOLOGICAL: `equilibrium_fuel` reaches the point through the forward BURNER (`Tt4` an
/// OUTPUT of `f = mdot_fuel/mdot_air`) and a 2-D Newton on the fuel closure — it never calls the
/// `Tt4`-control path. Two closures, one point.
///
/// This is also the empirical death of the withdrawn framing "fuel metering breaks rung 39's
/// (dagger) cancellation and re-couples LP into the HP core": if the two controls land on the same
/// manifold, the control knob cannot change the coupling.
#[test]
fn gate1_control_invariance_is_the_rung40_point() {
    let d = design(cpg_gas());
    let t = ft_default(&d);
    let core = t.core();
    for tt4 in [1500.0f64, 1300.0, 1100.0] {
        let eq = core.inner.equilibrium(&flight(), tt4);
        let mf = eq.close.f * eq.close.mdot_air;
        let (fq, _passes) = core.equilibrium_fuel(&flight(), mf, None);
        assert!((fq.base.nu_lp / eq.nu_lp - 1.0).abs() < 1e-12, "nu_lp at {tt4}: {}", fq.base.nu_lp);
        assert!((fq.base.nu_hp / eq.nu_hp - 1.0).abs() < 1e-12, "nu_hp at {tt4}: {}", fq.base.nu_hp);
        assert!((fq.base.tt4 / tt4 - 1.0).abs() < 1e-12, "Tt4 at {tt4}: {}", fq.base.tt4);
        assert!((fq.base.close.pi_lpc / eq.close.pi_lpc - 1.0).abs() < 1e-11, "pi_lpc at {tt4}");
        assert!((fq.base.close.pi_hpc / eq.close.pi_hpc - 1.0).abs() < 1e-11, "pi_hpc at {tt4}");
        // and the residuals really are zero (not merely the speeds agreeing)
        assert!(fq.base.phi_lp_dot.abs() < 1e-9 && fq.base.phi_hp_dot.abs() < 1e-9,
                "residuals at {tt4}: {} {}", fq.base.phi_lp_dot, fq.base.phi_hp_dot);
    }
}

// ------------------------------------------------------------------------------------ gate 2
/// GATE 2 — EXACT DISPATCH. `lp_disabled` builds no two-shaft state at all; the fuel methods
/// forward to the held rung-35 [`SpoolTransient`], so the fields compare bit-for-bit.
///
/// The Python compares seven dict keys with `==`. Both sides are rung 35's `Instant` here, because
/// the forward hands back exactly what the held object returned, so the comparison is on
/// `to_bits()` of the same seven fields.
///
/// **ONE AXIS OF PYTHON'S GATE IS UNREPRESENTABLE IN RUST, AND IT WAS DELETED AT RUNG 40, NOT
/// HERE.** Python builds the degenerate object with BOTH maps (`map_lp=LP_SHAPED,
/// map_hp=HP_SHAPED`) and its `__init__` picks `map_hp`; this gate is what proves it picked the
/// right one of the two. Every `lp_disabled` constructor in the port — rung 39's, rung 40's and
/// this one — takes only `map_hp`, so "the constructor held the wrong one of the two maps" cannot
/// fail by construction, in this file or in `rung40.rs`. Measured rather than assumed: the CALLER's
/// choice IS live — passing `lp_shaped()` here fails the gate — but it fails by exactly **1 ULP on
/// `nu` at `Tt4 = 1500`**, the first and thinnest cell the gate has, because that is the design
/// point where the running line barely reads the map at all. So the surviving discrimination is
/// real and narrow, and this note is the honest width of it.
#[test]
fn gate2_lp_disabled_is_rung35s_fuel_path_bit_for_bit() {
    let gas = cpg_gas();
    let single_engine = build_turbojet(gas, PI_HPC, TT4, 50_000.0, single());
    let st = SpoolTransient::new(single_engine.clone(), flight(), 1.0, hp_shaped());
    let deg = TwoSpoolFuelTransient::lp_disabled(single_engine, flight(), 1.0, hp_shaped());
    for tt4 in [1500.0f64, 1300.0, 1150.0] {
        let mf = st.fuel_for_tt4(&flight(), tt4, None);
        let a = st.equilibrium_fuel(&flight(), mf, None);
        let b = deg.equilibrium_fuel_lp_disabled(&flight(), mf, None);
        for (k, x, y) in [
            ("nu", a.nu, b.nu),
            ("pi_c", a.pi_c, b.pi_c),
            ("Tt4", a.tt4, b.tt4),
            ("mdot_air", a.mdot_air, b.mdot_air),
            ("f", a.f, b.f),
            ("tau_t", a.tau_t, b.tau_t),
            ("sp_thrust", a.sp_thrust, b.sp_thrust),
        ] {
            assert_eq!(x.to_bits(), y.to_bits(), "lp_disabled {k} at Tt4={tt4}: {x} != {y}");
        }
    }
}

// ------------------------------------------------------------------------------------ gate 3
/// GATE 3 — rung 40's `Tt4` control is inherited UNCHANGED. Building AND exercising the fuel
/// control must not perturb it.
#[test]
fn gate3_tt4_control_untouched_is_rung40_bit_for_bit() {
    let d = design(cpg_gas());
    let t40 = TwoSpoolTransientCore::new(d.clone(), flight(), 1.0, lp_shaped(), hp_shaped(), 1.0);
    let t = ft_default(&d);
    // exercise the new path
    t.core().constant_speed_excursion_fuel(&flight(), LO, HI);
    for tt4 in [1500.0f64, 1300.0, 1150.0] {
        let a = t40.equilibrium(&flight(), tt4);
        let b = t.core().inner.equilibrium(&flight(), tt4);
        for (k, x, y) in [
            ("nu_lp", a.nu_lp, b.nu_lp),
            ("nu_hp", a.nu_hp, b.nu_hp),
            ("pi_lpc", a.close.pi_lpc, b.close.pi_lpc),
            ("pi_hpc", a.close.pi_hpc, b.close.pi_hpc),
            ("Tt4", a.tt4, b.tt4),
            ("mdot_air", a.close.mdot_air, b.close.mdot_air),
            ("f", a.close.f, b.close.f),
            ("tau_hpt", a.tau_hpt, b.tau_hpt),
            ("tau_lpt", a.tau_lpt, b.tau_lpt),
            ("sp_thrust", a.sp_thrust, b.sp_thrust),
        ] {
            assert_eq!(x.to_bits(), y.to_bits(), "rung-40 {k} at Tt4={tt4}: {x} != {y}");
        }
    }
}

// ------------------------------------------------------------------------------------ gate 4
/// GATE 4 — the DYNAMICAL reduce: hold the fuel at its high value and march; the trajectory
/// relaxes onto the matched two-shaft equilibrium.
#[test]
fn gate4_settle_lands_on_the_equilibrium() {
    let d = design(cpg_gas());
    let t = ft_default(&d);
    let core = t.core();
    let mf_hi = core.fuel_for_tt4(&flight(), HI);
    let eq_hi = core.inner.equilibrium(&flight(), HI);
    let eq_lo = core.inner.equilibrium(&flight(), LO);
    let traj = core.integrate_fuel(
        &flight(), |_s| mf_hi, (eq_lo.nu_lp, eq_lo.nu_hp), 14.0, 0.02, &FuelLimiters::default());
    let last = *traj.last().expect("the settle march produced no trajectory");
    assert!(last.s > 13.0, "settle stopped early at s={}", last.s);
    assert!((last.nu_lp / eq_hi.nu_lp - 1.0).abs() < 1e-6, "nu_lp {}", last.nu_lp);
    assert!((last.nu_hp / eq_hi.nu_hp - 1.0).abs() < 1e-6, "nu_hp {}", last.nu_hp);
    assert!((last.tt4 / HI - 1.0).abs() < 1e-5, "Tt4 {}", last.tt4);
}

// ------------------------------------------------------------------------------------ gate 5
/// GATE 5 — THE MECHANISM (the rung).
///
/// `f = mdot_fuel/mdot_air` is set at the LP FACE, but the `Tt4` it produces is metered back
/// through the HP-FED NGV choke: the two spools sit at DIFFERENT points in the ONE overshoot loop.
/// Freezing EITHER spool therefore makes the overshoot WORSE — neither is a bystander, which is
/// WHY no single spool's clock can govern it.
///
/// SIGN / EXISTENCE ONLY. `d_lp` and `d_hp` do not sum to the total and are NOT calibrated
/// weights; only their positivity, and the DIRECTION in which the share trades with `rho`, are
/// asserted.
#[test]
fn gate5_both_spools_relieve_the_overshoot() {
    let d = design(cpg_gas());
    let mut seen: Vec<(&str, f64, f64, f64, f64)> = Vec::new(); // (name, r, rho, d_lp, d_hp)
    for (name, ml, mh) in two_shapes() {
        for rho in [0.5f64, 1.0, 2.0] {
            let t = ft(&d, ml, mh, rho);
            for r in [0.25f64, 1.0] {
                let fc = t.core().freeze_channels(&flight(), LO, HI, r, S_SETTLE, DS);
                assert!(fc.d_lp > 0.0, "{name} rho={rho} r={r}: d_lp={}", fc.d_lp);
                assert!(fc.d_hp > 0.0, "{name} rho={rho} r={r}: d_hp={}", fc.d_hp);
                seen.push((name, r, rho, fc.d_lp, fc.d_hp));
            }
        }
    }

    // THE CONTRAST: the share trades with rho — as the LP spool slows, the LP channel's relief
    // SHRINKS and the HP channel's GROWS. Asserted as a direction on the ratio, never as weights.
    for (name, _, _) in two_shapes() {
        for r in [0.25f64, 1.0] {
            let mut row: Vec<(f64, f64, f64)> = seen
                .iter()
                .filter(|&&(n, rr, ..)| n == name && rr == r)
                .map(|&(_, _, rho, dl, dh)| (rho, dl, dh))
                .collect();
            row.sort_by(|a, b| a.partial_cmp(b).expect("the rho rows are finite"));
            let ratios: Vec<f64> = row.iter().map(|&(_, dl, dh)| dl / dh).collect();
            for w in ratios.windows(2) {
                assert!(w[0] >= w[1], "{name} r={r}: d_lp/d_hp not non-increasing in rho: {ratios:?}");
                // STRONGER THAN PYTHON, AND MEASURED BEFORE IT WAS WRITTEN. Python's
                // `ratios == sorted(ratios, reverse=True)` is NON-STRICT, so a CONSTANT ratio
                // satisfies the one assertion whose whole subject is that the share MOVES with
                // `rho` — and that is not hypothetical: dropping the `/rho` from the LP ODE
                // entirely (measured, step 2) leaves gate 5 GREEN while gates 6, 7 and 9 all fail.
                // The shipped ratios fall 1.426 -> 0.794 -> 0.419 (flow/press, r=0.25) and
                // 1.384 -> 0.774 -> 0.409 (tilted, r=0.25) over `rho` in {0.5, 1, 2}, i.e. by ~3.4x
                // across the sweep; the TIGHTEST adjacent pair anywhere in the four rows is
                // 1.005 -> 0.728 (flow/press, r=1), a 28 % drop. So the strict `>` is nowhere near
                // marginal, and it closes the hole for free.
                assert!(w[0] > w[1], "{name} r={r}: d_lp/d_hp did not MOVE with rho: {ratios:?}");
            }
        }
    }
}

// ------------------------------------------------------------------------------------ gate 6
/// GATE 6 — THE CEILING. `rho` multiplies ONLY the LP ODE (`dnu_L/ds = Phi_L/rho`), so
/// `rho -> infinity` IS the LP-frozen system: the LP-frozen march is `rho`-independent
/// BIT-FOR-BIT, and the measured overshoot rises monotonically toward it.
///
/// This is what turns the `rho`-monotonicity (gate 7) from a bare sign into a BOUNDED claim: the
/// worst TIT excursion a heavy LP spool can produce is computable without marching the LP spool at
/// all.
#[test]
fn gate6_lp_frozen_is_the_rho_free_ceiling() {
    let d = design(cpg_gas());
    for (name, ml, mh) in two_shapes() {
        for r in [0.25f64, 1.0] {
            // rho-freeness of the ceiling: bit-for-bit across very different rho
            let ceil: Vec<f64> = [1.0f64, 7.0, 50.0]
                .iter()
                .map(|&rho| {
                    ft(&d, ml, mh, rho)
                        .core()
                        .ramp_excursion_fuel(&flight(), LO, HI, r, Some(Spool::Lp), S_SETTLE, DS)
                        .x
                })
                .collect();
            assert_eq!(ceil[0].to_bits(), ceil[1].to_bits(), "{name} r={r}: ceiling moved {ceil:?}");
            assert_eq!(ceil[1].to_bits(), ceil[2].to_bits(), "{name} r={r}: ceiling moved {ceil:?}");
            // and X(rho) climbs toward it from BELOW, monotonically
            let xs: Vec<f64> = [1.0f64, 8.0, 32.0]
                .iter()
                .map(|&rho| {
                    ft(&d, ml, mh, rho)
                        .core()
                        .ramp_excursion_fuel(&flight(), LO, HI, r, None, S_SETTLE, DS)
                        .x
                })
                .collect();
            for w in xs.windows(2) {
                assert!(w[0] <= w[1], "{name} r={r}: X not monotone in rho: {xs:?}");
            }
            let top = xs[xs.len() - 1];
            assert!(top < ceil[0], "{name} r={r}: {top} not below the ceiling {}", ceil[0]);
            assert!(top > 0.90 * ceil[0], "{name} r={r}: {top} far under the ceiling {}", ceil[0]);
        }
    }
}

// ------------------------------------------------------------------------------------ gate 7
/// GATE 7 — a heavier LP spool worsens the TIT excursion, because the LP-FACE airflow lag is what
/// spikes `f`. SIGN ONLY across three shape pairs x two ramp durations; every magnitude rides on
/// `rho`, the maps, the step and the band (disclaimed).
#[test]
fn gate7_overshoot_rises_monotonically_with_rho() {
    let d = design(cpg_gas());
    for (name, ml, mh) in shapes() {
        for r in [0.25f64, 1.0] {
            let mut xs: Vec<f64> = Vec::new();
            for rho in [0.25f64, 0.5, 1.0, 2.0, 4.0] {
                let e = ft(&d, ml, mh, rho)
                    .core()
                    .ramp_excursion_fuel(&flight(), LO, HI, r, None, S_SETTLE, DS);
                assert!(e.complete, "{name} r={r} rho={rho}: the ramp did not complete");
                xs.push(e.x);
            }
            for w in xs.windows(2) {
                assert!(w[0] <= w[1], "{name} r={r}: X not monotone in rho: {xs:?}");
            }
            assert!(xs[xs.len() - 1] > xs[0], "{name} r={r}: no rise at all: {xs:?}");
        }
    }
}

// ------------------------------------------------------------------------------------ gate 8
/// GATE 8 — INHERITED from rung 35, re-measured on two shafts (NOT this rung's finding): the TIT
/// excursion dwarfs the surge excursion on both spools, so the acceleration is temperature-limited
/// before it is surge-limited on these maps.
///
/// Also witnesses the `r -> 0` step being EXACTLY `rho`-free (a pure algebraic map property —
/// rung 34/35's argument doubled), which is the `r_eff -> 0` endpoint of the ramp family rather
/// than a separate object.
///
/// The multiple is DISCLOSED, not tuned: measured 4.41x (flow/press), 6.33x (press/flow), 5.21x
/// (tilted), so the gate asserts >4x. It is an ORDERING claim on these maps, in the rung-32/35
/// register — which limit binds first is map-dependent, and no TIT redline is modelled.
#[test]
fn gate8_tit_limited_before_surge() {
    let d = design(cpg_gas());
    for (name, ml, mh) in shapes() {
        let cs = ft(&d, ml, mh, 1.0).core().constant_speed_excursion_fuel(&flight(), LO, HI);
        assert!(cs.e_temp > 4.0 * cs.e_lp.max(cs.e_hp),
                "{name}: E_temp={} vs surge {} / {}", cs.e_temp, cs.e_lp, cs.e_hp);
        // exactly rho-free: both spools are frozen, so no clock can enter
        let a = ft(&d, ml, mh, 0.2).core().constant_speed_excursion_fuel(&flight(), LO, HI);
        let b = ft(&d, ml, mh, 5.0).core().constant_speed_excursion_fuel(&flight(), LO, HI);
        for (k, x, y) in [
            ("Tt4_peak", a.tt4_peak, b.tt4_peak),
            ("E_temp", a.e_temp, b.e_temp),
            ("E_lp", a.e_lp, b.e_lp),
            ("E_hp", a.e_hp, b.e_hp),
            ("f", a.f, b.f),
        ] {
            assert_eq!(x.to_bits(), y.to_bits(), "{name} {k}: {x} != {y}");
        }
    }
}

// ------------------------------------------------------------------------------------ gate 9
/// GATE 9 — THE WITHDRAWN CLAIM, asserted as such (rung 40's gate-7 move).
///
/// Rung 43 deliberately claims NO effective clock ratio `r_eff = r/rho^q`. Two facts are asserted
/// so the tempting reading cannot creep back:
///
///   (a) the referenced currencies are CIRCULAR — the best-fit `q` READS BACK whichever spool sits
///       in the denominator, so `E_temp_H`'s `q ~ 0` was never evidence that "the HP clock
///       governs";
///   (b) even on the spool-neutral `X` there is NO collapse — the best exponent cuts the spread
///       ~4.9x against the `q = 0` endpoint but bottoms out near 14 %, i.e. points a real
///       effective clock would place on ONE curve still differ by a seventh.
///
/// Together these kill both the "geometric-mean composite clock" reading and the "slow spool
/// rate-limits it" reading. NOTHING about the exponent is currency-independent.
///
/// **§ 5.16 probe 3 measured this gate BLIND to a tie-break the reported `q` depends on**: the
/// score is piecewise-constant in `q` and each argmin is tied with its neighbour at a gap of
/// exactly zero, so first-of-equals and last-of-equals both satisfy every assertion below
/// (`0.05/0.35/0.65` and `0.10/0.40/0.70` alike). `slice_s_dispatch.rs` is where the tie-break is
/// pinned; this file reproduces the Python gate and no more, and says so rather than letting the
/// green tick imply the stronger claim.
#[test]
fn gate9_the_withdrawn_effective_clock_ratio() {
    let d = design(cpg_gas());
    let (_, ml, mh) = shapes()[0]; // "flow/press"
    let mut pts: Vec<(f64, f64, f64, f64, f64)> = Vec::new(); // (r, rho, E_temp_H, X, E_temp_L)
    for rho in [0.25f64, 1.0, 4.0, 8.0] {
        let t = ft(&d, ml, mh, rho);
        for r in [0.25f64, 0.5, 1.0, 2.0] {
            let e = t.core().ramp_excursion_fuel(&flight(), LO, HI, r, None, S_SETTLE, DS);
            if e.complete {
                pts.push((r, rho, e.e_temp_h, e.x, e.e_temp_l));
            }
        }
    }
    assert!(pts.len() >= 12, "only {} complete points", pts.len());

    let currency = |pick: fn(&(f64, f64, f64, f64, f64)) -> f64| -> Vec<(f64, f64, f64)> {
        pts.iter().map(|p| (p.0, p.1, pick(p))).collect()
    };
    let (q_h, _) = turbojet::fuel_transient::FuelTransientCore::collapse_exponent(
        &currency(|p| p.2), NB, None);
    let (q_x, s_x) = turbojet::fuel_transient::FuelTransientCore::collapse_exponent(
        &currency(|p| p.3), NB, None);
    let (q_l, _) = turbojet::fuel_transient::FuelTransientCore::collapse_exponent(
        &currency(|p| p.4), NB, None);

    // (a) CIRCULARITY: the exponent tracks the denominator, HP -> none -> LP.
    assert!(q_h < q_x && q_x < q_l, "not ordered: {q_h} {q_x} {q_l}");
    assert!(q_l - q_h > 0.3, "the currencies barely differ: {q_h} {q_x} {q_l}");

    // (b) NO COLLAPSE on the spool-neutral currency.
    assert!(s_x > 0.10, "X collapsed: spread {s_x}");

    // ... and the neutral currency's own best exponent is INTERIOR — it matches neither
    // single-spool clock (q=0 "HP governs" nor q=1 "LP governs"). This is the ONLY exponent
    // statement rung 43 makes, and it is made only on X; it is NOT a refutation of q=1 in general,
    // since on X the q=0 fit is the worse of the two.
    assert!(0.0 < q_x && q_x < 1.0, "qX not interior: {q_x}");
}

// ----------------------------------------------------------------------------------- gate 10
/// GATE 10 — the default single-spool design run is untouched by rung 43 (the rungs-7+ invariant):
/// building AND marching the fuel transient must not perturb it.
///
/// ONE engine object, run either side of the diagnostics — Python's shape, kept deliberately.
/// Rust's `run(&self)` makes the mutation channel hard to open in the first place, so the gate is
/// thinner here than in Python; that is worth saying rather than letting the tick imply otherwise.
#[test]
fn gate10_cycle_untouched_rung6_bit_for_bit() {
    let eng: Engine =
        build_turbojet(Gas::reacting_equilibrium(), 10.0, TT4, 50_000.0, single());
    let a = eng.run(&flight(), 1.0);
    let d = design(cpg_gas());
    let t = ft_default(&d);
    t.core().constant_speed_excursion_fuel(&flight(), LO, HI);
    t.core().freeze_channels(&flight(), LO, HI, 0.25, S_SETTLE, DS);
    let b = eng.run(&flight(), 1.0);
    assert_eq!(a.performance.specific_thrust.to_bits(), b.performance.specific_thrust.to_bits());
    assert_eq!(a.station("4").far.to_bits(), b.station("4").far.to_bits());
    assert_eq!(a.station("9").pt.to_bits(), b.station("9").pt.to_bits());
}

// ------------------------------------------------------------------------- scope / concession
/// CONCESSION (rung 35's, carried verbatim): the forward burner is built for the NON-equilibrium
/// gas and must REFUSE an equilibrium one rather than mis-solve. The reacting reduce is the
/// `Tt4`-control path, which still works.
///
/// § 5.16 prediction 8: poked DIRECTLY this is the REFUSAL, and the identity is asserted through
/// [`classify`] as well as the substring Python matches on — reached through an ordinary entry
/// point the same input yields the BRACKET error instead, which is `slice_s_dispatch.rs`'s gate.
#[test]
fn concession_reacting_gas_fuel_control_is_refused() {
    let d = design(Gas::reacting_equilibrium());
    let t = ft_default(&d);
    let core = t.core();
    match core.try_tt4_from_f(700.0, 0.025) {
        Ok(tt4) => panic!("rung-43 forward burner accepted an equilibrium gas: Tt4={tt4}"),
        Err(e) => {
            assert!(e.0.contains("non-equilibrium"), "wrong refusal: {}", e.0);
            assert_eq!(classify(&e), FuelAbort::Refusal, "wrong abort identity: {}", e.0);
        }
    }
    // the reacting Tt4-control path (rung 40) is unaffected
    let eq = core.inner.equilibrium(&flight(), 1400.0);
    assert!(eq.nu_lp > 0.0 && eq.close.pi_lpc > 1.0, "{} {}", eq.nu_lp, eq.close.pi_lpc);
}
