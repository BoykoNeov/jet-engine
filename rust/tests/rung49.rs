//! RUNG 49 — THE `phi` / SURGE-MARGIN FEEDBACK LIMITER: a limiter acts on a spool through BOTH
//! its edges, and the two edges answer to DIFFERENT clocks.
//!
//! Port of `tests/test_rung49.py`, gate for gate. That file defines **17 test functions** and
//! collects **17 items** — no `parametrize`, and **no `slow` mark** (§ 5.18 counted the slice's
//! four slow marks with `--collect-only -m slow` and all four are in `test_rung52.py`, despite
//! three docstrings here saying "SLOW").
//!
//! **PYTHON LABELS ITS OWN GATES AND THE NUMBERING STARTS AT 3** — six `CONTRACT`s, then
//! `GATE 3, 4, 5, 6, 7, 8, 9, 9b, 10, 11, 12`. The names below carry Python's label, not a
//! renumbering, so a reader holding the two files side by side is never translating.
//!
//! | # | `tests/test_rung49.py` | here |
//! |---|---|---|
//! | 1 | `test_reduce_surge_none_never_consults_the_leg_bit_for_bit` | [`contract1_surge_none_never_consults_the_leg`] |
//! | 2 | `test_reduce_dormant_floor_bit_for_bit_rung45` | [`contract2_dormant_floor_is_bit_for_bit_rung45`] |
//! | 3 | `test_reduce_composite_min_select_with_the_prior_legs` | [`contract3_composite_min_select_with_the_prior_legs`] |
//! | 4 | `test_reduce_lp_disabled_asserts` | [`contract4_lp_disabled_refuses_the_leg`] |
//! | 5 | `test_decel_never_fires_bit_for_bit_rung45` | [`contract5_decel_never_fires_bit_for_bit_rung45`] |
//! | 6 | `test_cycle_untouched_by_the_phi_leg_bit_for_bit_rung6` | [`contract6_cycle_untouched_bit_for_bit_rung6`] |
//! | 7 | `test_the_hold_is_a_sliding_mode_not_chatter` | [`gate3_the_hold_is_a_sliding_mode_not_chatter`] |
//! | 8 | `test_both_edges_close_inside_the_ramp_the_unreachable_object` | [`gate4_both_edges_close_inside_the_ramp`] |
//! | 9 | `test_headline_one_clip_credits_the_watched_spool_and_DEBITS_the_other` | [`gate5_one_clip_credits_the_watched_spool_and_debits_the_other`] |
//! | 10 | `test_mechanism_the_unwatched_minimum_relocates_to_just_after_the_release` | [`gate6_the_unwatched_minimum_relocates_to_just_after_the_release`] |
//! | 11 | `test_sign_flips_when_the_release_lands_far_past_the_ramp_rung48_regime` | [`gate7_the_sign_flips_when_the_release_lands_far_past_the_ramp`] |
//! | 12 | `test_discriminator_the_debit_is_clocked_by_the_RAMP_not_the_spools_own_minimum` | [`gate8_the_debit_is_clocked_by_the_ramp_not_the_spools_own_minimum`] |
//! | 13 | `test_cross_instrument_rung48_crossing_reproduced_exactly` | [`gate9_rung48s_crossing_reproduced_on_a_new_instrument_class`] |
//! | 14 | `test_the_exposed_spool_is_the_LATE_one_inverting_rungs_41_44_45` | [`gate9b_the_exposed_spool_is_the_late_one`] |
//! | 15 | `test_not_the_ramp_rate_lever_the_non_tautology` | [`gate10_not_rung_44s_ramp_rate_lever`] |
//! | 16 | `test_honest_boundary_a_floor_above_the_running_line_destroys_the_accel` | [`gate11_the_honest_boundary_is_gated_not_hidden`] |
//! | 17 | `test_robustness_the_debit_survives_ds_and_rho` | [`gate12_the_debit_survives_ds_and_rho`] |
//!
//! # THE GRID IS THIS FILE'S, NOT ITS NEIGHBOUR'S
//!
//! Two traps, both of the class rung48.rs's header calls *the port's most-repeated defect*:
//!
//! * **`SETTLE` is 2.0 here, against rung 48's 4.0.** Only gate 10 asks for a full settle, and it
//!   passes `4.0` explicitly — exactly as Python does.
//! * **`SHAPES` has TWO entries here, against rung 48's three and rung 47's four.** They are
//!   `"flow/press"` and `"flat-lp"`, and `"flat-lp"` is the same `(FLAT, HP_SHAPED)` pair rung 48
//!   calls `"hp-only"`. Reusing that file's helper would compile, would give gate 11 the right
//!   numbers, and would silently offer `tilted` — a shape this rung's grid does not contain. So
//!   the map set is written out again rather than shared. *A census is a property of its grid.*
//!
//! # Where Rust has to say out loud what Python leaves implicit
//!
//! * **CONTRACT 1's monkeypatch becomes a COUNTER, and the counter is proved live first.** Python
//!   rebinds `_surge_fuel` to a raiser and runs four marches. Rust reads
//!   [`counters::take`](turbojet::fuel_transient::counters::take)`().surge_calls`, which witnesses
//!   *no call happened* where the raiser witnesses *no call escaped*. An `== 0` on a dead
//!   instrument is permanently green, so the gate ARMS a floor on the same object, the same march
//!   and the same thread and asserts the counter MOVES before it asserts the zero.
//! * **CONTRACT 4 is ONE assert that four rungs' gates all fire.** § 5.18 finding 1 swept all 255
//!   arming combinations through the degenerate object and measured that rungs 50/51/52's own
//!   `lp_disabled` refusals are **unreachable** — arming `s_off` / `tau_rel` / `lag` at all
//!   requires an armed leg, and the `accel` / `surge` refusals precede them inside the block. All
//!   four rungs' `test_reduce_lp_disabled_asserts` arm `surge=`, so all four fire the refusal
//!   asserted HERE. Rust copied the three unreachable asserts faithfully (*COPY vs REDERIVATION*);
//!   the defect is the source's, and it is written up at step 5.
//! * **CONTRACT 4's `nu0` cannot be spelled as Python spells it** — Python hands a degenerate
//!   object a two-shaft PAIR and the refusal fires before it is read; Rust's
//!   `integrate_fuel_lp_disabled` takes the scalar rung 35's marcher wants, so `1.0` is passed.
//!   Rung 47 gate 3 / rung 48 contract 4's precedent.
//! * **CONTRACT 5 gets its mechanism gated where Python only describes it.** Python asserts the
//!   decel trajectory is bit-identical and that the bare march clears the floor. The counter adds
//!   the *reason*: the leg was consulted on every point and found DORMANT on every one, so the
//!   float-identity return (`mf_sched` itself) is what makes the trajectory bit-for-bit rather
//!   than a solve that happened to agree.
//! * **GATE 6 REPRODUCES A LOOP-VARIABLE LEAK IN THE PYTHON.** Its last assertion reads
//!   `row["s_eng"]` *after* the `for row in _sweep(LP_FLOORS)` loop has ended, so `row` is the
//!   **`0.7400`** row while the march it filters was run at **`0.7450`**. The natural Rust — that
//!   march's own `s_eng` — is a different window and a different gate. The stale binding is copied
//!   and named. *COPY vs REDERIVATION*, at a call site rather than in a formula.
//! * **GATE 6's `bmap` is an INDEX alignment here.** Python keys the bare march by
//!   `round(p["s"], 6)` and looks the limited march's points up in it. Both marches accumulate the
//!   SAME `s` sequence from the same `0.0` (§ 5.18 finding 5b: 201 points at `s_end = 4.0` on all
//!   three marchers), so the lookup is an index and the gate asserts the `s` BITS match rather
//!   than rebuilding a float-keyed map that could silently miss.
//! * **FIVE LIVE DEFECTS PASS ALL 17 GATES, MEASURED.** Twelve injections into the shipped reader
//!   (§ 5.18 step 1 finding 3): `hold_err` reading the wrong spool moves only the HP rows, which
//!   **gate 3 never sweeps**; `fuel_removed` losing its `0.5`, `tt4_peak_lim` and `tt4_peak_bare`
//!   read off the wrong march, and the march coordinate spelled `k * ds` are all invisible here.
//!   The two `both_edges_inside_ramp` guard clauses are a different case — each is **inert on all
//!   23 cells**, so breaking one moves nothing at all. Those five keys are step 5's oracle's job,
//!   not this file's.
//! * **THE `relief_other == 0.0` SITES ARE EXACT ZEROS, not tolerances.** Gate 9's downstream arm
//!   is the same `x − x` mechanism as rung 48's six sites: for a clip downstream of `s_lp*` the
//!   bare and limited marches are bit-identical through the LP argmin, so the two `min` calls read
//!   the SAME float. If it misses, the diagnosis is the march coordinate or [`first_raw_min`]'s
//!   strict `<`, never physics.
//!
//! # THE BAR MARGINS, TABULATED — AND THREE OF THEM ARE TIGHT
//!
//! Slice T shipped 9/9 green and was blind to a 24 % value error, because no bar's margin had been
//! measured; its lesson is to tabulate them. Measured here on the suite's OWN cells, before the
//! 17/17 green was believed:
//!
//! | gate | bar | worst measured | slack |
//! |---|---|---:|---:|
//! | 3 | `hold_err < 1e-9` | `8.88e-16` | **1.1e6 ×** |
//! | 3 | `\|relief_watched − (phi_lim − min_phi_lp)\| < 1e-5` | `1.86e-8` | 539 × |
//! | 5 | `min relief_other < −0.005` | `−0.010403` | 2.1 × |
//! | 6 | `s_min_other − s_rel ≤ 3·ds = 0.06` | `0.040` | **1.5 × — ONE GRID CELL** |
//! | 9 | `\|relief_other − forecast(s_eng)\| < 2e-3` | `1.7765e-3` | **1.13 × — 11 %** |
//! | 10 | `\|nu_hp_end − nu_hp_end_bare\| < 5e-4` | `1.22e-5` | 41 × |
//! | 10 | `\|relief_other[0]\| < \|relief_other[2]\|` | `3.25e-4` vs `1.04e-2` | 32 × |
//! | 11 | `nu_hp_end_bare − nu_hp_end > 0.2` | `0.2198` | **1.10 × — 10 %** |
//! | 12 | `\|Δrelief_other\| < 0.25·\|relief_other\|` | 8.8 % | 2.8 × |
//! | 4 | `both_edges_inside_ramp` | `r − s_rel = −0.02 / +0.06` | **one grid cell** |
//!
//! **THIS FILE IS NOT SLICE T's.** Slice T measured 5–7 ORDERS of slack on all four of its bars
//! and concluded the port's risk was not in the decisions. Here **four** of the ten sit inside
//! `1.5 ×`, and three of them are VALUE bars that § 5.18 finding 3's table — which covered the
//! engaged mask, `hold_err`, `lag_relief`'s `eps` and the boolean — does not reach. Gate 9's is
//! the tightest thing in the file: an 11 % error in either `relief_other` or the bare-march
//! forecast breaks it, and nothing looser would have noticed.
//!
//! # `#[ignore]`
//!
//! None. Nothing in `test_rung49.py` carries `slow`, and the two shared sweeps are memoised
//! through [`OnceLock`] exactly as Python shares them through `_SWEEPS` — which takes the file
//! from ~60 marches to ~44 without changing a number.
//!
//! [`first_raw_min`]: turbojet::fuel_transient::first_raw_min

use std::sync::OnceLock;

use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::fuel_transient::{
    counters, FuelLimiters, FuelPoint, FuelTransientCore, SurgeLimiter, SurgeRelief,
    TwoSpoolFuelTransient,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;

const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const R: f64 = 0.5;
/// **2.0, NOT rung 48's 4.0.** Only gate 10 wants a settled endpoint and it passes `4.0` itself.
const SETTLE: f64 = 2.0;
const DS: f64 = 0.02;
/// Rungs 46/47's redline, for the composite gates.
const REDLINE: f64 = 1480.0;

/// The bare march's raw surge minima at this config (`ds = 0.02`), from
/// `docs/plans/rung49-anchor-phi-limiter.md` via `test_rung49.py:66-68`.
const S_LP_STAR: f64 = 0.240;
const S_HP_STAR: f64 = 0.400;
const MIN_PHI_LP: f64 = 0.735466;
const PHI_LP_START: f64 = 0.773116;

const LP_FLOORS: [f64; 4] = [0.7550, 0.7500, 0.7450, 0.7400];
const HP_FLOORS: [f64; 4] = [0.9000, 0.8800, 0.8700, 0.8650];

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

/// `test_rung49.py`'s `SINGLE`. No `nozzle_convergent` — admissible for contract 6's cycle run,
/// which is its only consumer.
fn single() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.90, e_c: None, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, e_t: None,
        eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: false,
    }
}

/// [`single`] plus the one constant contract 4 needs to have a degenerate object at all.
fn single_matchable() -> Losses {
    Losses { nozzle_convergent: true, ..single() }
}

/// `test_rung49.py`'s `_cpg_gas` — `R_c`/`R_t` DERIVED from the pair above them.
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

/// `test_rung49.py`'s `SHAPES` — **TWO entries**, and deliberately not rung 48's three. See the
/// header: `"flat-lp"` is that file's `"hp-only"` pair under this file's name, and sharing the
/// helper would import a `tilted` shape rung 49's grid does not have.
fn shape_maps(name: &str) -> (ComponentMap, ComponentMap) {
    match name {
        "flow/press" => (lp_shaped(), hp_shaped()),
        // FLAT LP ⇒ the swept floor sits ABOVE the LP's running-line start: gate 11's degenerate
        // corner.
        "flat-lp" => (ComponentMap::flat(), hp_shaped()),
        other => panic!("test_rung49.py has no SHAPES entry {other:?}"),
    }
}

fn design(gas: Gas) -> TwoSpoolEngine {
    build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, 50_000.0, real())
}

fn ft(d: &TwoSpoolEngine, map_lp: ComponentMap, map_hp: ComponentMap, rho: f64)
    -> TwoSpoolFuelTransient
{
    TwoSpoolFuelTransient::new(d.clone(), flight(), 1.0, map_lp, map_hp, rho)
}

/// Python's `_ramp`: the accel fuel ramp + its running-line start.
///
/// **`min(1.0, s/r)`, NOT the marcher's `s >= r ⇒ mf_hi` branch** — those differ by an ulp at and
/// past the ramp end. Contracts 1/2/3/5 use this form (they call `integrate_fuel` directly); every
/// gate that goes through `surge_relief` gets the branch form. Deliberately not unified, as in
/// rung 48.
fn ramp(core: &FuelTransientCore, lo: f64, hi: f64, r: f64) -> (impl Fn(f64) -> f64, (f64, f64)) {
    let f = flight();
    let mf0 = core.fuel_for_tt4(&f, lo);
    let mf1 = core.fuel_for_tt4(&f, hi);
    let eq0 = core.inner.equilibrium(&f, lo);
    (move |s: f64| mf0 + (mf1 - mf0) * (s / r).min(1.0), (eq0.nu_lp, eq0.nu_hp))
}

/// Python's `KEYS` — the SEVEN-key tuple `_same` compares, by EXHAUSTIVE read so a renamed field
/// breaks the build rather than narrowing the comparison silently.
fn keys7(p: &FuelPoint) -> [u64; 7] {
    [p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.phi_lp.to_bits(), p.phi_hp.to_bits(),
     p.tt4.to_bits(), p.f.to_bits(), p.mf.to_bits()]
}

/// Python's `_same`.
fn same(a: &[FuelPoint], b: &[FuelPoint]) {
    assert_eq!(a.len(), b.len(), "trajectory lengths differ: {} vs {}", a.len(), b.len());
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(keys7(x), keys7(y), "trajectories diverge at s={} / {}", x.s, y.s);
    }
}

fn peak_tt4(traj: &[FuelPoint]) -> f64 {
    traj.iter().fold(f64::NEG_INFINITY, |m, p| m.max(p.tt4))
}

/// Python's `_sweep` body, un-memoised. The four single-consumer keys call this directly; the two
/// shared ones go through the [`OnceLock`]s below.
fn sweep(floors: &[f64], spool: Spool, r: f64, settle: f64, shape: &str, rho: f64)
    -> Vec<SurgeRelief>
{
    let (ml, mh) = shape_maps(shape);
    let d = design(cpg_gas());
    let t = ft(&d, ml, mh, rho);
    t.core().floor_sweep(&flight(), LO, HI, floors, spool, r, settle, DS)
}

/// `_SWEEPS[(LP_FLOORS, "lp", 0.5, 2.0, "flow/press", 1.0)]` — SEVEN consumers (gates 3, 4, 5, 6,
/// 7's contrast line, 9b's second loop, 11's last line), i.e. 8 marches instead of 56.
fn lp_sweep() -> &'static [SurgeRelief] {
    static S: OnceLock<Vec<SurgeRelief>> = OnceLock::new();
    S.get_or_init(|| sweep(&LP_FLOORS, Spool::Lp, R, SETTLE, "flow/press", 1.0))
}

/// `_SWEEPS[(HP_FLOORS, "hp", …)]` — two consumers (gates 9 and 9b).
fn hp_sweep() -> &'static [SurgeRelief] {
    static S: OnceLock<Vec<SurgeRelief>> = OnceLock::new();
    S.get_or_init(|| sweep(&HP_FLOORS, Spool::Hp, R, SETTLE, "flow/press", 1.0))
}

/// The message of an `assert!` that fired, or `None` if the call returned. Rungs 45–48's helper,
/// and its caveat travels with it: this swaps the GLOBAL panic hook, so the restore can race a
/// parallel test's backtrace output — it cannot change a `catch_unwind` RESULT.
fn refusal<F: FnOnce()>(f: F) -> Option<String> {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
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

// ============================================================================== contract 1
/// CONTRACT 1 — `surge = None` leaves rungs 45/46/47/48 bit-for-bit, because the leg is never
/// CONSULTED.
///
/// **PYTHON'S RAISER BECOMES A COUNTER, AND THE COUNTER IS PROVED LIVE BEFORE IT IS BELIEVED** —
/// see the header. The four marches are Python's four: bare, topped, lagged, scheduled.
#[test]
fn contract1_surge_none_never_consults_the_leg() {
    let f = flight();
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let core = t.core();
    let (sched, nu0) = ramp(core, LO, HI, R);

    // POSITIVE CONTROL: the instrument counts when the leg IS armed.
    let live = SurgeLimiter::new(Spool::Lp, 0.7500);
    counters::reset();
    let armed = core.integrate_fuel(&f, &sched, nu0, R + 1.0, DS,
                                    &FuelLimiters { surge: Some(live), ..Default::default() });
    let c_armed = counters::take();
    assert!(c_armed.surge_calls > 0,
            "the surge_calls counter must MOVE on an armed leg, or the zero below means nothing");
    assert!(!armed.is_empty());

    // THE CONTRACT: four `surge = None` marches consult the rung-49 leg exactly zero times.
    let acc = core.accel_schedule(&f, LO, HI, 0.25, 13);
    counters::reset();
    let bare = core.integrate_fuel(&f, &sched, nu0, R + 1.0, DS, &FuelLimiters::default());
    let top = core.integrate_fuel(&f, &sched, nu0, R + 1.0, DS,
                                  &FuelLimiters { tt4_max: Some(REDLINE), ..Default::default() });
    let lag = core.integrate_fuel(
        &f, &sched, nu0, R + 1.0, DS,
        &FuelLimiters { tt4_max: Some(REDLINE), tau_gov: Some(0.2), ..Default::default() });
    let sch = core.integrate_fuel(&f, &sched, nu0, R + 1.0, DS,
                                  &FuelLimiters { accel: Some(&acc), ..Default::default() });
    let c = counters::take();
    assert_eq!((c.surge_calls, c.surge_dormant, c.surge_skips), (0, 0, 0),
               "a surge=None march must not consult the rung-49 leg at all: {c:?}");

    assert!(!bare.is_empty() && !top.is_empty() && !lag.is_empty() && !sch.is_empty());
    // ... and the four are genuinely different marches (the gate is not vacuous)
    assert!(peak_tt4(&bare) > peak_tt4(&top), "the governor must bite on the topped march");
    assert!(peak_tt4(&lag) > peak_tt4(&top), "the lag must overshoot the instantaneous governor");
    assert!(sch.iter().any(|p| p.mf < p.mf_sched), "the rung-48 leg must genuinely bind");
}

// ============================================================================== contract 2
/// CONTRACT 2 — a floor below the whole march leaves the cap above the schedule EVERYWHERE, so
/// `try_surge_fuel` returns its argument float-identically and the trajectory is the bare rung-45
/// one BIT-for-bit, not merely equal. **Both spools**, as Python sweeps them.
#[test]
fn contract2_dormant_floor_is_bit_for_bit_rung45() {
    let f = flight();
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let core = t.core();
    let (sched, nu0) = ramp(core, LO, HI, R);
    let bare = core.integrate_fuel(&f, &sched, nu0, R + SETTLE, DS, &FuelLimiters::default());

    for spool in [Spool::Lp, Spool::Hp] {
        let leg = SurgeLimiter::new(spool, 0.50);
        let dorm = core.integrate_fuel(&f, &sched, nu0, R + SETTLE, DS,
                                       &FuelLimiters { surge: Some(leg), ..Default::default() });
        same(&bare, &dorm);
        assert!(dorm.iter().all(|p| p.mf == p.mf_sched), "a dormant leg must not clip: {spool:?}");
    }
}

// ============================================================================== contract 3
/// CONTRACT 3, both directions — the min-select ORDERING gate. Armed together with rung 46's
/// governor, the pair reproduces whichever single leg actually binds, bit-for-bit.
#[test]
fn contract3_composite_min_select_with_the_prior_legs() {
    let f = flight();
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let core = t.core();
    let (sched, nu0) = ramp(core, LO, HI, R);
    let dorm = SurgeLimiter::new(Spool::Lp, 0.50); // never binds
    let live = SurgeLimiter::new(Spool::Lp, 0.7500); // binds hard

    // (a) phi floor dormant + redline armed  ==  redline only
    let top = core.integrate_fuel(&f, &sched, nu0, R + SETTLE, DS,
                                  &FuelLimiters { tt4_max: Some(REDLINE), ..Default::default() });
    let both_a = core.integrate_fuel(
        &f, &sched, nu0, R + SETTLE, DS,
        &FuelLimiters { tt4_max: Some(REDLINE), surge: Some(dorm), ..Default::default() });
    same(&top, &both_a);

    // (b) phi floor armed + redline above the resulting peak  ==  phi floor only
    let phi_only = core.integrate_fuel(&f, &sched, nu0, R + SETTLE, DS,
                                       &FuelLimiters { surge: Some(live), ..Default::default() });
    let peak = peak_tt4(&phi_only);
    let both_b = core.integrate_fuel(
        &f, &sched, nu0, R + SETTLE, DS,
        &FuelLimiters { tt4_max: Some(peak + 50.0), surge: Some(live), ..Default::default() });
    same(&phi_only, &both_b);
    assert!(phi_only.iter().any(|p| p.mf < p.mf_sched), "the (b) leg must genuinely bind");
}

// ============================================================================== contract 4
/// CONTRACT 4 — the finding is a per-spool SPLIT, so it is inherently two-shaft and the degenerate
/// object REFUSES the leg. See the header for both divergences: the unspellable `nu0`, and § 5.18
/// finding 1's measurement that this ONE assert is what all four of rungs 49–52's `lp_disabled`
/// gates actually fire.
#[test]
fn contract4_lp_disabled_refuses_the_leg() {
    let f = flight();
    let se: Engine = build_turbojet(cpg_gas(), 10.0, TT4, 50_000.0, single_matchable());
    let deg = TwoSpoolFuelTransient::lp_disabled(se, flight(), 1.0, hp_shaped());
    let leg = SurgeLimiter::new(Spool::Lp, 0.75);

    let m = refusal(|| {
        deg.integrate_fuel_lp_disabled(&f, |_s| 0.5, 1.0, R + 0.5, DS,
                                       &FuelLimiters { surge: Some(leg), ..Default::default() });
    })
    .expect("the rung-49 leg on an lp_disabled object must refuse");
    assert!(m.contains("inherently two-shaft"), "the refusal must name the reason: {m}");
}

// ============================================================================== contract 5
/// CONTRACT 5 — on a DECEL `phi` rises ABOVE the running line throughout, so a floor set for the
/// accel is never reached: the leg is structurally an accel instrument.
///
/// **AND THE MECHANISM IS GATED, NOT JUST THE RESULT.** Python asserts the trajectory is
/// bit-identical and that the bare march clears the floor. The counter adds why it can be
/// BIT-identical rather than merely equal: every consultation took the DORMANT arm, which returns
/// `mf_sched` itself — a float identity, not a solve that agreed to tolerance. There is no rung-48
/// style row-0 artifact here, because a `phi` floor has no `(a/b)*b` round-trip in its residual.
#[test]
fn contract5_decel_never_fires_bit_for_bit_rung45() {
    let f = flight();
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let core = t.core();
    let (sched, nu0) = ramp(core, HI, LO, R); // 1400 -> 1000 K
    let leg = SurgeLimiter::new(Spool::Lp, 0.7500);

    let bare = core.integrate_fuel(&f, &sched, nu0, R + SETTLE, DS, &FuelLimiters::default());
    counters::reset();
    let lim = core.integrate_fuel(&f, &sched, nu0, R + SETTLE, DS,
                                  &FuelLimiters { surge: Some(leg), ..Default::default() });
    let c = counters::take();
    same(&bare, &lim);
    assert!(bare.iter().all(|p| p.phi_lp > 0.7500), "the decel must clear the floor");

    assert!(c.surge_calls > 0, "the leg must have been consulted at all: {c:?}");
    assert_eq!(c.surge_calls, c.surge_dormant,
               "every consultation on a decel must take the DORMANT float-identity arm: {c:?}");
}

// ============================================================================== contract 6
/// CONTRACT 6 — the design run is a SEPARATE entry point; a fourth fuel-side leg cannot move it.
#[test]
fn contract6_cycle_untouched_bit_for_bit_rung6() {
    let f = flight();
    let eng = build_turbojet(Gas::reacting_equilibrium(), 10.0, TT4, 50_000.0, single());
    let a = eng.run(&f, 1.0);

    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let leg = SurgeLimiter::new(Spool::Lp, 0.7500);
    let _ = t.core().surge_relief(&f, LO, HI, &leg, R, 1.0, DS, None, None, None);

    let b = eng.run(&f, 1.0);
    assert_eq!(a.performance.specific_thrust.to_bits(), b.performance.specific_thrust.to_bits());
    assert_eq!(a.station("4").far.to_bits(), b.station("4").far.to_bits());
}

// ============================================================================== gate 3
/// GATE 3 — an IDENTITY check, deliberately NOT a finding. The clip RAISES `phi`, which would make
/// the leg dormant and restore fuel; the naive worry is chatter. Measured: the set-point solve
/// rides the floor to solver tolerance at EVERY engaged point.
///
/// The watched relief `phi_lim − min phi_bare` is DEFINITIONAL under a working set-point solve; it
/// is asserted here to gate the SOLVER and is never used as evidence for the rung's claims, which
/// all live on the UNWATCHED spool.
///
/// § 5.18 finding 3 measured `hold_err`'s decision at `7.77e-16 … 1.33e-15` against a `1e-9` bar;
/// re-measured on this cell set at a worst `8.88e-16`, **1.1e6 × of slack** — the loosest bar in
/// the file, so a miss here is a solver defect and never a knife-edge. The identity bar beside it
/// is much tighter (`1.86e-8` against `1e-5`, 539 ×) because it carries the anchor constant
/// `MIN_PHI_LP`'s own six-digit rounding.
#[test]
fn gate3_the_hold_is_a_sliding_mode_not_chatter() {
    for row in lp_sweep() {
        assert!(row.hold_err < 1e-9,
                "CHATTER at floor {}: hold_err {}", row.phi_lim, row.hold_err);
        assert!((row.relief_watched - (row.phi_lim - MIN_PHI_LP)).abs() < 1e-5,
                "the watched relief must be the definitional one at floor {}: {} vs {}",
                row.phi_lim, row.relief_watched, row.phi_lim - MIN_PHI_LP);
    }
}

// ============================================================================== gate 4
/// GATE 4 — THE ENABLING MEASUREMENT. `docs/both-edges-limiter-negative.md` proved no `pt3`-filter
/// limiter can close its window inside the ramp (every proxy signal rises monotonically through
/// it, so release is structurally post-ramp). A `phi` floor CAN, because `phi` has its minimum
/// inside the ramp by definition. This is the object that makes the closing edge testable at all.
///
/// **THE BOOLEAN IS THE SLICE'S TIGHTEST DECISION, AND THE MARGIN IS REGISTERED HERE RATHER THAN
/// GATED.** § 5.18 finding 3 measured `r − s_rel` over the eight floor cells at `0.06` / `0.16`
/// inside and `−0.10` / `−0.02` / `−0.42` / `−0.12` / `−0.02` outside — ONE GRID CELL at the
/// tightest — with an eighth cell (the HP floor `0.8650`, swept by gates 9/9b, which never read
/// the boolean) at **`−1.11e-16`**. Nothing in either language reads the boolean on that cell, so
/// the exposure is the one-cell margin here; the ulp cell is what shows how thin the boundary is.
/// It survives only because both languages accumulate `s += ds` — see the note at
/// `integrate_fuel`'s loop.
#[test]
fn gate4_both_edges_close_inside_the_ramp() {
    let rows = lp_sweep();
    let inside: Vec<&SurgeRelief> = rows.iter().filter(|x| x.both_edges_inside_ramp).collect();
    assert!(!inside.is_empty(),
            "the phi floor must produce a window with BOTH edges inside the ramp");
    for x in &inside {
        assert!(0.0 < x.s_eng && x.s_eng < x.s_rel && x.s_rel < R,
                "floor {}: 0 < {} < {} < {R} must hold", x.phi_lim, x.s_eng, x.s_rel);
    }
    // ...and the tight floors do NOT (the window opens at both ends as the floor rises):
    let (first, last) = (&rows[0], &rows[rows.len() - 1]);
    assert!(!first.both_edges_inside_ramp,
            "the tightest floor {} must NOT close inside the ramp: s_rel {}",
            first.phi_lim, first.s_rel);
    assert!(first.s_eng < last.s_eng && first.s_rel > last.s_rel,
            "a tighter floor must engage EARLIER and release LATER: {:?} vs {:?}",
            (first.s_eng, first.s_rel), (last.s_eng, last.s_rel));
}

// ============================================================================== gate 5
/// GATE 5 — THE RUNG. Every row engages UPSTREAM of `s_hp* = 0.400`, so rung 48's law predicts a
/// CREDIT on the HP in all of them. Measured: a DEBIT in all of them, from the very same clip that
/// credits the LP.
#[test]
fn gate5_one_clip_credits_the_watched_spool_and_debits_the_other() {
    for row in lp_sweep() {
        assert!(row.s_eng < S_HP_STAR,
                "floor {} must engage upstream of s_hp*: {}", row.phi_lim, row.s_eng);
        assert!(row.relief_watched > 0.0, "floor {}: {:?}", row.phi_lim, row.relief_watched);
        assert!(row.relief_other < 0.0,
                "floor {}: the unwatched spool must be DEBITED, got {}",
                row.phi_lim, row.relief_other);
    }
    // the debit is not a rounding artifact — it is up to ~1.2% of the bare min phi. Measured
    // worst `−0.010403` against the `−0.005` bar: 2.1 × of slack.
    let worst = lp_sweep().iter().fold(f64::INFINITY, |m, x| m.min(x.relief_other));
    assert!(worst < -0.005, "the debit must be physical, not a rounding artifact: {worst}");
}

// ============================================================================== gate 6
/// GATE 6 — THE MECHANISM. Inside the window the unwatched spool is BETTER off (the clip really
/// does slow its descent — rung 48's arrest). But it is SLOWED, not arrested: it falls right
/// through the window while the bare march has already turned around. Then the leg lets go, the
/// withheld fuel reaches a still-ramping plant, and the descent RE-OPENS. So the unwatched minimum
/// sits just AFTER the release edge — that is where the damage is made, and it is why the closing
/// edge is not causally inert.
///
/// **THE PLACEMENT BAR IS ONE GRID CELL WIDE.** `s_min_other − s_rel` measures `0.0` / `0.0` /
/// `0.020` / `0.040` over the four floors against a `3·ds = 0.06` bar — 1.5 × at the tightest, so
/// this is a boundary like gate 4's and not a comfortable one.
///
/// **PYTHON'S LOOP VARIABLE LEAKS INTO THE LAST ASSERTION AND THE LEAK IS COPIED.** `row` there is
/// the `0.7400` row (the loop's last), while `lim` is marched at `0.7450`. Writing `0.7450`'s own
/// `s_eng` would be a different window and a different gate — so the stale row is bound
/// explicitly, named, and used. See the header.
#[test]
fn gate6_the_unwatched_minimum_relocates_to_just_after_the_release() {
    let f = flight();
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let core = t.core();
    let (bare, _) =
        core.fuel_ramp_march(&f, LO, HI, R, SETTLE, DS, &FuelLimiters::default());

    for row in lp_sweep() {
        assert!(row.s_rel - 1e-9 <= row.s_min_other
                    && row.s_min_other <= row.s_rel + 3.0 * DS + 1e-9,
                "floor {}: the unwatched minimum at {} must land at or just after the release {}",
                row.phi_lim, row.s_min_other, row.s_rel);
    }

    let leg = SurgeLimiter::new(Spool::Lp, 0.7450);
    let (lim, _) = core.fuel_ramp_march(&f, LO, HI, R, SETTLE, DS,
                                        &FuelLimiters { surge: Some(leg), ..Default::default() });
    // Python's `bmap = {round(p["s"], 6): p for p in bare}` is an INDEX alignment here: both
    // marches accumulate the same `s` from the same `0.0`, which the bit comparison witnesses.
    assert_eq!(bare.len(), lim.len(), "the two marches must have the same length");
    for (a, b) in bare.iter().zip(lim.iter()) {
        assert_eq!(a.s.to_bits(), b.s.to_bits(), "the march coordinates must agree bit for bit");
    }
    let mid: Vec<usize> = (0..lim.len()).filter(|&i| (0.20..=0.30).contains(&lim[i].s)).collect();
    assert!(!mid.is_empty(), "the 0.20..0.30 window must contain points");
    for &i in &mid {
        assert!(lim[i].phi_hp > bare[i].phi_hp,
                "inside the window the clip must HELP the unwatched spool at s={}: {} vs {}",
                lim[i].s, lim[i].phi_hp, bare[i].phi_hp);
    }

    // ...while still descending through it (slowed, not arrested). THE LEAKED ROW — Python's
    // post-loop `row`, i.e. LP_FLOORS[-1] = 0.7400, NOT this march's 0.7450.
    let leaked = &lp_sweep()[lp_sweep().len() - 1];
    assert_eq!(
        leaked.phi_lim, 0.7400,
        "DO NOT REPAIR THIS: the window below is filtered by the LAST SWEEP ROW's s_eng, not by          the 0.7450 march's own — Python's `row` leaks out of the loop above it and the leak is          part of the gate. If this fires, LP_FLOORS was reordered; re-point the binding at the          loop's last row, never at the march's own floor. See this file's header.");
    let win: Vec<f64> = lim.iter()
        .filter(|p| leaked.s_eng <= p.s && p.s <= 0.42)
        .map(|p| p.phi_hp)
        .collect();
    assert!(!win.is_empty(), "the window must contain points");
    assert!(win[win.len() - 1] < win[0],
            "the unwatched spool must keep descending inside the window: {} -> {}",
            win[0], win[win.len() - 1]);
}

// ============================================================================== gate 7
/// GATE 7 — the two-term law predicting its own inversion. Push the release well past the ramp end
/// (`r = 0.15`, `s_rel/r = 2.4…3.2`) and the debit term dies, leaving rung 48's credit alone: the
/// SAME instrument on the SAME plant watching the SAME spool now REBATES the other one.
///
/// This is why rung 48 is BOUNDED, not refuted — its own leg released at `s_rel/r = 1.16–2.24`
/// (measured in `docs/both-edges-limiter-negative.md`), i.e. in exactly this regime.
#[test]
fn gate7_the_sign_flips_when_the_release_lands_far_past_the_ramp() {
    let fast = sweep(&[0.7500, 0.7400], Spool::Lp, 0.15, SETTLE, "flow/press", 1.0);
    for row in &fast {
        assert!(row.s_rel > 2.0 * 0.15,
                "floor {}: the release at {} must be late", row.phi_lim, row.s_rel);
        assert!(row.relief_other > 0.0,
                "floor {}: a far-past-ramp release must REBATE, got {}",
                row.phi_lim, row.relief_other);
    }
    // ...and it is the same instrument that debited at r=0.5
    assert!(lp_sweep().iter().all(|x| x.relief_other < 0.0), "the r=0.5 sign, for contrast");
}

// ============================================================================== gate 8
/// GATE 8 — THE DISCRIMINATOR: which clock sets the debit?
///
/// At `r = 0.5` the unwatched spool's own minimum (`0.400`) and the ramp end (`0.500`) are too
/// close to separate. At `r = 2.0` they are 3.1× apart (`s_hp* = 0.650` against a ramp end of
/// `2.0`). The debit tracks the RAMP END: it is far larger with the release at `s_rel ~ r` than at
/// `s_rel ~ s_hp*`, and it grows monotonically with `s_rel` straight THROUGH `s_hp*` without
/// noticing it. So the two edges answer to DIFFERENT clocks — the credit per-spool (rung 48), the
/// debit ramp-clocked (rung 44's clock).
///
/// Python's docstring calls this SLOW; `--collect-only -m slow` says the file marks nothing, so
/// there is no `#[ignore]` question here either.
#[test]
fn gate8_the_debit_is_clocked_by_the_ramp_not_the_spools_own_minimum() {
    let rows = sweep(&[0.7650, 0.7690, 0.7725], Spool::Lp, 2.0, 1.5, "flow/press", 1.0);
    assert_eq!(rows.len(), 3);
    let (at_spool_min, mid, at_ramp_end) = (&rows[0], &rows[1], &rows[2]);
    assert!(at_spool_min.s_rel < 1.0 && 1.0 < mid.s_rel && mid.s_rel < at_ramp_end.s_rel,
            "the three releases must straddle s_hp*: {:?}",
            (at_spool_min.s_rel, mid.s_rel, at_ramp_end.s_rel));
    for x in &rows {
        assert!(x.relief_other < 0.0, "floor {}: {}", x.phi_lim, x.relief_other);
    }
    assert!(at_ramp_end.relief_other.abs() > 5.0 * at_spool_min.relief_other.abs(),
            "the debit must be dominated by the RAMP clock, not the spool's own minimum: {} vs {}",
            at_spool_min.relief_other, at_ramp_end.relief_other);
    // monotone in s_rel straight through s_hp* = 0.650
    assert!(at_spool_min.relief_other.abs() < mid.relief_other.abs()
                && mid.relief_other.abs() < at_ramp_end.relief_other.abs(),
            "the debit must grow monotonically with s_rel: {:?}",
            (at_spool_min.relief_other, mid.relief_other, at_ramp_end.relief_other));
}

// ============================================================================== gate 9
/// GATE 9 — flip the watched spool. An HP floor engages LATE; the LP's minimum is EARLY
/// (`s_lp* = 0.240`), so rung 48's edge condition applies to the LP in pure form — and the
/// exact-zero lands where the law says, with no fitting and no limited march. A genuine forecast
/// off a BARE march, landing on a limiter class rung 48 never built.
///
/// **THE UPSTREAM FORECAST IS THE TIGHTEST BAR IN THE FILE.** `|relief_other − forecast(s_eng)|`
/// measures `1.7765e-3` at the `0.9000` floor and `1.1268e-3` at `0.8800`, against a `2e-3` bar —
/// **1.13 ×, i.e. 11 % of margin**. An error of that size in either the limited march's raw
/// minimum or the bare march's truncated one breaks this gate and nothing else in the file would
/// notice; that is what makes it worth having, and what makes it the first place to look if a
/// later step moves a number.
///
/// **THE DOWNSTREAM `== 0.0` IS AN EXACT ZERO AND ITS MECHANISM IS `x − x`** — for a clip
/// downstream of `s_lp*` the bare and limited marches are bit-identical through the LP argmin, so
/// the two `min` folds return the SAME float. A miss is the march coordinate or `first_raw_min`'s
/// strict `<`, never physics.
#[test]
fn gate9_rung48s_crossing_reproduced_on_a_new_instrument_class() {
    let f = flight();
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let (bare, _) =
        t.core().fuel_ramp_march(&f, LO, HI, R, SETTLE, DS, &FuelLimiters::default());
    let mpl = bare.iter().fold(f64::INFINITY, |m, p| m.min(p.phi_lp));
    let forecast = |s_eng: f64| -> f64 {
        bare.iter()
            .filter(|p| p.s <= s_eng + 1e-12)
            .fold(f64::INFINITY, |m, p| m.min(p.phi_lp))
            - mpl
    };

    let (mut seen_up, mut seen_down) = (false, false);
    for row in hp_sweep() {
        if row.s_eng < S_LP_STAR - 1e-12 {
            seen_up = true;
            assert!(row.relief_other > 0.0,
                    "floor {} engages upstream of s_lp* and must rebate: {}",
                    row.phi_lim, row.relief_other);
            // the truncated-descent forecast, to its own O(ds) accuracy
            assert!((row.relief_other - forecast(row.s_eng)).abs() < 2e-3,
                    "s_eng {}: relief {} vs forecast {}",
                    row.s_eng, row.relief_other, forecast(row.s_eng));
        } else if row.s_eng > S_LP_STAR + 1e-12 {
            seen_down = true;
            assert_eq!(row.relief_other, 0.0,
                       "floor {} engages downstream of s_lp*: EXACTLY nothing, got {}",
                       row.phi_lim, row.relief_other);
            assert_eq!(forecast(row.s_eng), 0.0, "the forecast must call it exactly too");
        }
    }
    assert!(seen_up && seen_down, "the sweep must straddle s_lp*");
}

// ============================================================================== gate 9b
/// GATE 9b — WHY the split has the direction it does. A release edge is structurally LATE (it
/// needs an accumulated window), so it lands inside the HP's basin and past the LP's: within
/// `0.005` of its own minimum the LP sits at `s ∈ [0.15, 0.32]`, the HP at `[0.29, 0.50]`.
///
/// So the early-LP / late-HP timing that ran through rungs 46/47/48 decides WHICH spool is exposed
/// to the closing edge — and it is the HP, exactly INVERTING rungs 41/44/45's "the LP eats the
/// excursion".
#[test]
fn gate9b_the_exposed_spool_is_the_late_one() {
    let f = flight();
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let (bare, _) =
        t.core().fuel_ramp_march(&f, LO, HI, R, SETTLE, DS, &FuelLimiters::default());
    let mpl = bare.iter().fold(f64::INFINITY, |m, p| m.min(p.phi_lp));
    let mph = bare.iter().fold(f64::INFINITY, |m, p| m.min(p.phi_hp));
    let b_lp: Vec<f64> =
        bare.iter().filter(|p| p.phi_lp - mpl <= 0.005).map(|p| p.s).collect();
    let b_hp: Vec<f64> =
        bare.iter().filter(|p| p.phi_hp - mph <= 0.005).map(|p| p.s).collect();
    assert!(!b_lp.is_empty() && !b_hp.is_empty());
    let (lp_lo, lp_hi) = (b_lp[0], b_lp[b_lp.len() - 1]);
    let (hp_lo, hp_hi) = (b_hp[0], b_hp[b_hp.len() - 1]);
    assert!(lp_lo < hp_lo && lp_hi < hp_hi,
            "the LP basin must be the EARLY one: [{lp_lo},{lp_hi}] vs [{hp_lo},{hp_hi}]");

    // every HP-watching release lands past the LP basin => no debit on the LP
    for row in hp_sweep() {
        assert!(row.s_rel > lp_hi,
                "floor {}: release {} must land past the LP basin end {lp_hi}",
                row.phi_lim, row.s_rel);
        assert!(row.relief_other >= 0.0, "floor {}: {}", row.phi_lim, row.relief_other);
    }
    // while every LP-watching release lands INSIDE the HP basin => the debit
    for row in lp_sweep() {
        assert!((hp_lo <= row.s_rel && row.s_rel <= hp_hi + 3.0 * DS) || row.s_rel > hp_hi,
                "floor {}: release {} vs the HP basin [{hp_lo},{hp_hi}]", row.phi_lim, row.s_rel);
        assert!(row.relief_other < 0.0, "floor {}: {}", row.phi_lim, row.relief_other);
    }
}

// ============================================================================== gate 10
/// GATE 10 — the deflation to exclude is "any clip removes fuel and slows the accel".
///
/// Three exclusions, all measured: (i) the endpoint is UNMOVED; (ii) `fuel_removed` is positive
/// and smooth, and the LARGEST fuel removal gives the SMALLEST debit — so the debit is not "how
/// much fuel" but WHEN it is given back; (iii) one clip moves the two spools in OPPOSITE
/// directions, which a ramp-rate lever cannot do. Uses a FULL settle (`4.0`, this file's `SETTLE`
/// being 2.0) because it reads a settled endpoint.
///
/// Margins, measured: the endpoint moves `1.22e-5` against a `5e-4` bar (41 ×); `fuel_removed`
/// falls by `3.7e-3` / `9.0e-4` / `5.9e-4` / `3.0e-4` so the ordering is not a tie; and the
/// biggest-removal-smallest-debit inversion is `3.25e-4` against `1.04e-2` — **32 ×**, not a
/// near-miss.
#[test]
fn gate10_not_rung_44s_ramp_rate_lever() {
    let rows = sweep(&[0.7650, 0.7550, 0.7500, 0.7450, 0.7400], Spool::Lp, R, 4.0,
                     "flow/press", 1.0);
    for row in &rows {
        assert!(row.fuel_removed > 0.0, "floor {}: {}", row.phi_lim, row.fuel_removed);
        assert!((row.nu_hp_end - row.nu_hp_end_bare).abs() < 5e-4,
                "floor {}: the endpoint must be unmoved, {} vs {}",
                row.phi_lim, row.nu_hp_end, row.nu_hp_end_bare);
        assert!(row.relief_watched > 0.0 && 0.0 > row.relief_other,
                "floor {}: {} / {}", row.phi_lim, row.relief_watched, row.relief_other);
    }
    // fuel_removed is monotone in the floor...
    let fr: Vec<f64> = rows.iter().map(|x| x.fuel_removed).collect();
    let mut desc = fr.clone();
    desc.sort_by(|a, b| b.partial_cmp(a).expect("no NaN in fuel_removed"));
    assert_eq!(fr, desc, "fuel_removed must fall monotonically as the floor drops");
    // ...but the debit is NOT: the biggest removal gives the smallest debit
    assert!(rows[0].relief_other.abs() < rows[2].relief_other.abs(),
            "the biggest removal ({} fuel, {} debit) must give the SMALLEST debit, against \
             ({}, {})",
            rows[0].fuel_removed, rows[0].relief_other, rows[2].fuel_removed,
            rows[2].relief_other);
}

// ============================================================================== gate 11
/// GATE 11 — THE HONEST BOUNDARY. `phi_lim` must sit BELOW the initial running-line `phi`, or the
/// leg binds from `s = 0` and never releases. On the FLAT LP map the swept floor sits above the
/// LP's start, and `nu_hp` at settle COLLAPSES — the accel does not complete and the leg HAS
/// degenerated into rung 44's ramp-rate lever. Structurally rung 48's `m → 0` degeneracy;
/// reported, not hidden.
///
/// **A 10 % BAR.** The collapse measures `0.2198` against the `0.2` literal — 1.10 ×, the file's
/// third tight bar. It is Python's own number and is kept, but it is close enough that a future
/// change to the FLAT map's start would move it.
#[test]
fn gate11_the_honest_boundary_is_gated_not_hidden() {
    let rows = sweep(&[0.7500], Spool::Lp, R, SETTLE, "flat-lp", 1.0);
    let row = &rows[0];
    assert_eq!(row.s_eng, 0.0,
               "a floor above the running line must bind from s=0, got {}", row.s_eng);
    assert!(row.nu_hp_end_bare - row.nu_hp_end > 0.2,
            "the accel must visibly fail to complete: {} vs {}",
            row.nu_hp_end, row.nu_hp_end_bare);
    // ...and the healthy band is precisely the one whose floor clears the start
    assert!(lp_sweep().iter().all(|x| x.phi_lim < PHI_LP_START),
            "every healthy cell's floor must clear the running-line start {PHI_LP_START}");
}

// ============================================================================== gate 12
/// GATE 12 — a minimum-LOCATION claim must survive refinement, and a two-spool claim must not ride
/// on rung 40's complex inter-spool mode.
///
/// The refinement moves `relief_other` from `−0.010403` to `−0.011317` — **8.8 %** against the
/// 25 % bar, 2.8 × of slack, and non-zero, so the convergence claim is not satisfied by the two
/// marches happening to agree.
#[test]
fn gate12_the_debit_survives_ds_and_rho() {
    let f = flight();
    let d = design(cpg_gas());
    let leg = SurgeLimiter::new(Spool::Lp, 0.7500);

    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let vals: Vec<f64> = [0.02, 0.01].iter()
        .map(|&ds| {
            t.core().surge_relief(&f, LO, HI, &leg, R, SETTLE, ds, None, None, None).relief_other
        })
        .collect();
    assert!(vals.iter().all(|&v| v < 0.0), "the debit must survive refinement: {vals:?}");
    assert!((vals[1] - vals[0]).abs() < 0.25 * vals[0].abs(),
            "ds-convergent: {vals:?}");

    for rho in [0.25, 4.0] {
        let tr = ft(&d, lp_shaped(), hp_shaped(), rho);
        let row = tr.core().surge_relief(&f, LO, HI, &leg, R, SETTLE, DS, None, None, None);
        assert!(row.relief_watched > 0.0 && 0.0 > row.relief_other,
                "rho {rho}: {} / {}", row.relief_watched, row.relief_other);
    }
}
