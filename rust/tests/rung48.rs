//! RUNG 48 — THE `Wf/pt3` ACCELERATION SCHEDULE: a fuel-side limiter rebates a spool IFF it
//! engages UPSTREAM of THAT spool's OWN surge minimum.
//!
//! Port of `tests/test_rung48.py`, gate for gate. That file defines **16 test functions** and
//! collects **16 items** — no `parametrize` — the last term of § 5.17's `31 = 6 + 9 + 16`, counted
//! with `--collect-only` rather than read off a header. It is the largest of the slice's three
//! suites and the one that first reaches [`AccelSchedule::cap`](turbojet::fuel_transient::AccelSchedule::cap),
//! [`accel_schedule`](turbojet::fuel_transient::FuelTransientCore::accel_schedule) and
//! [`try_sched_fuel`](turbojet::fuel_transient::FuelTransientCore::try_sched_fuel).
//!
//! **PYTHON LABELS ITS OWN GATES AND THE LABELS ARE NOT THE FILE ORDER** — six `CONTRACT`s, then
//! `GATE 6, 7, 8, 8b, 9b, 9, 10, 11, 12, 13`. The names below carry Python's label, not a
//! renumbering, so a reader holding the two files side by side is never translating.
//!
//! | # | `tests/test_rung48.py` | here |
//! |---|---|---|
//! | 1 | `test_reduce_accel_none_never_consults_the_leg_bit_for_bit` | [`contract1_accel_none_never_consults_the_leg`] |
//! | 2 | `test_reduce_dormant_schedule_bit_for_bit_rung45` | [`contract2_dormant_schedule_is_bit_for_bit_rung45`] |
//! | 3 | `test_reduce_two_leg_composite_min_select` | [`contract3_two_leg_composite_min_select`] |
//! | 4 | `test_reduce_lp_disabled_asserts` | [`contract4_lp_disabled_refuses_the_leg`] |
//! | 5 | `test_decel_never_fires_bit_for_bit_rung45` | [`contract5_decel_never_fires_bit_for_bit_rung45`] |
//! | 6 | `test_cycle_untouched_by_accel_schedule_bit_for_bit_rung6` | [`contract6_cycle_untouched_bit_for_bit_rung6`] |
//! | 7 | `test_kappa_derived_from_running_line_and_pt3_identity` | [`gate6_kappa_is_derived_and_the_pt3_identity_holds`] |
//! | 8 | `test_window_exists_ratio_rises_through_the_lp_minimum` | [`gate7_the_window_exists_the_ratio_rises_through_the_lp_minimum`] |
//! | 9 | `test_engagement_crossing_lp_switches_off_exactly_at_s_lp` | [`gate8_the_lp_rebate_switches_off_exactly_at_s_lp`] |
//! | 10 | `test_downstream_clip_is_bit_identical_through_the_minimum` | [`gate8b_a_downstream_clip_is_bit_identical_through_the_minimum`] |
//! | 11 | `test_hp_crossing_demonstrated_on_a_slow_ramp` | [`gate9b_the_hp_crossing_on_a_slow_ramp`] |
//! | 12 | `test_engagement_crossing_hp_is_later_the_split` | [`gate9_the_hp_crossing_is_later_and_that_is_the_split`] |
//! | 13 | `test_not_ramp_rate_lever_the_non_tautology` | [`gate10_not_rung_44s_ramp_rate_lever`] |
//! | 14 | `test_degeneracy_boundary_small_margin_is_the_ramp_rate_lever` | [`gate11_the_degeneracy_boundary_is_gated_not_hidden`] |
//! | 15 | `test_fast_ramp_single_crossing_when_the_minima_coincide` | [`gate12_coincident_minima_give_one_crossing`] |
//! | 16 | `test_crossing_rule_robust_across_map_shapes` | [`gate13_the_crossing_rule_across_map_shapes`] |
//!
//! # ONE GAS, AND A GRID THAT IS **NOT** RUNG 47's
//!
//! Every `_ft` here takes `_cpg_gas()` (gate 6's single-spool cycle object excepted, which is
//! `Gas::reacting_equilibrium` as every rung-6 reduce gate's is), so the file is CPG throughout
//! like `test_rung47.py`. **The map set is NOT the same, and copying rung 47's helper would have
//! silently widened it**: `test_rung47.py`'s `SHAPES` has **four** entries, this file's has
//! **three** — there is no `press/flow` here. Nothing would have failed (gate 13 iterates two
//! named keys), which is exactly why it is written out separately: *a census is a property of the
//! grid*, and a shared helper with a different grid is the port's most-repeated defect.
//!
//! `SETTLE` differs too — **4.0 here against rung 47's 2.0** — because the engagement sweep reads
//! a SETTLED endpoint (`nu_hp_end`) and rung 47 only needed the peak.
//!
//! # Where Rust has to say out loud what Python leaves implicit
//!
//! * **CONTRACT 1's monkeypatch becomes a COUNTER, and the counter is proved live first.** Python
//!   replaces `_sched_fuel` with a raiser and runs three marches. Rust cannot rebind a method, so
//!   the claim — *the leg is never consulted* — is read off
//!   [`counters::take`](turbojet::fuel_transient::counters::take)`().sched_calls`, which is
//!   STRICTLY stronger than "no exception escaped". But an `== 0` assertion is worthless on an
//!   instrument that never counts, so the gate first ARMS an accel and asserts `sched_calls > 0`
//!   on the same object, the same march and the same thread. `slice_s_dispatch.rs` paid for that
//!   rule: *a gate whose expected result is "nothing" passes when the swap silently fails to
//!   take.*
//! * **CONTRACT 5's row-0 ARTIFACT REPRODUCES, and the port gates its MECHANISM where Python only
//!   describes it.** Python excludes row 0 because a `(a/b)*b` round-trip fires the min-select by
//!   0–3 ulp. Measured here: **1 ulp, 1 of 226 rows, and exactly ONE of 904 leg consultations
//!   non-dormant** — so the counter pins "confined to row 0" as a gate rather than a docstring.
//!   The first draft of this gate asserted row 0 was bit-identical, on the reasoning that the
//!   dormant fast path returns `mf_sched` itself; the run refuted it in one line. It is recorded
//!   because *a prediction from a mechanism is still a prediction*.
//! * **CONTRACT 4's `nu0` cannot be spelled.** Python passes a two-shaft PAIR into a degenerate
//!   object; the refusal fires before it is read. Rust's `integrate_fuel_lp_disabled` takes the
//!   scalar rung 35's marcher wants, so the gate passes `1.0` — rung 47 gate 3's precedent, and
//!   the same class as `equilibrium_fuel_lp_disabled` dropping `start`. The constructors differ
//!   too: Python's `_ft(lp_disabled=True)` takes the TWO-spool design and a flag, Rust's
//!   [`lp_disabled`](turbojet::fuel_transient::TwoSpoolFuelTransient::lp_disabled) a single-spool
//!   [`Engine`].
//! * **GATE 6's `len(acc.n_H) == 13` is a WEAKER gate here.** Python's 13 is `accel_schedule`'s
//!   DEFAULT; Rust makes `n` explicit, so the assertion can only check that this caller passed 13.
//!   It is written against the literal to keep it from becoming `x == x`.
//! * **THE TEST HELPER's RAMP IS NOT THE MARCHER's RAMP.** Python's `_ramp` is
//!   `mf0 + (mf1-mf0)*min(1, s/r)`; `_fuel_ramp_march`'s internal closure returns `mf_hi` EXACTLY
//!   at `s >= r`. Those differ by an ulp at and past the ramp end. Contracts 1/2/3/5 and gate 7 use
//!   the helper form (they call `integrate_fuel` directly); gates 8b/9b get the branch form through
//!   [`fuel_ramp_march`](turbojet::fuel_transient::FuelTransientCore::fuel_ramp_march). They are
//!   deliberately NOT unified.
//! * **THE SIX `== 0.0` SITES ARE ONE CLAIM.** § 5.17 finding 3 measured why: for a downstream
//!   clip the bare and limited marches are bit-identical for their first 14–20 points and the LP
//!   argmin sits at index 12, so the two `min` calls read the SAME float and the difference is
//!   `0.0` for the reason `x − x` is. They appear in gates 8, 9, 9b, 10, 12 and 13 — prediction P2
//!   says they pass or fail as a block, and a SPLIT outcome would refute the mechanism.
//!
//! # `#[ignore]`
//!
//! `test_rung48.py` marks **nothing** `slow` — § 5.17 counted the slice's two slow marks as rung
//! 46's pair — so there is no marker question here either. The three-gate sweep is shared through
//! one [`OnceLock`] exactly as Python shares it through `_SWEEPS`, which halves the file's marches
//! (12 → 6) without changing a number.

use std::sync::OnceLock;

use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::fuel_transient::{
    counters, FuelLimiters, FuelPoint, FuelTransientCore, ScheduleRelief,
    TwoSpoolFuelTransient,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;

const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
/// `test_rung48.py:62`. Rungs 46/47's redline, for the composite gates only — no gate here reads
/// the bare peak, which § 5.17 finding 6 measured at 1690.5–1703.0 over the four shapes against
/// the `~1645` / `~1670` the two neighbouring suites quote.
const REDLINE: f64 = 1480.0;
const R: f64 = 0.5;
/// **4.0, not rung 47's 2.0** — the engagement sweep reads a SETTLED `nu_hp_end`.
const SETTLE: f64 = 4.0;
const DS: f64 = 0.02;
/// Python's `accel_schedule` DEFAULT `n`, made explicit by the Rust signature. Gate 6 asserts the
/// table length against the literal 13 rather than against this name, so the assertion cannot
/// degenerate into a comparison of the constant with itself.
const N_SCHED: usize = 13;

const MARGINS: [f64; 6] = [0.15, 0.25, 0.35, 0.42, 0.45, 0.48];

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

/// `test_rung48.py`'s `SINGLE`. No `nozzle_convergent`, which is admissible for the rung-6 cycle
/// gate that is its only consumer.
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

/// `test_rung48.py`'s `_cpg_gas` — `R_c`/`R_t` DERIVED from the pair above them.
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

fn tilted() -> ComponentMap {
    ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..ComponentMap::flat() }
}

/// `test_rung48.py`'s `SHAPES` — **THREE entries, in Python's dict order**. Rung 47's four-entry
/// set is a different grid and is deliberately not reused; see the header.
fn shape_maps(name: &str) -> (ComponentMap, ComponentMap) {
    match name {
        "flow/press" => (lp_shaped(), hp_shaped()),
        "tilted" => (tilted(), tilted()),
        // LP FLAT ⇒ NO rung-40 complex inter-spool mode — gate 13's discriminator.
        "hp-only" => (ComponentMap::flat(), hp_shaped()),
        other => panic!("test_rung48.py has no SHAPES entry {other:?}"),
    }
}

fn design(gas: Gas) -> TwoSpoolEngine {
    build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, 50_000.0, real())
}

fn ft(d: &TwoSpoolEngine, map_lp: ComponentMap, map_hp: ComponentMap) -> TwoSpoolFuelTransient {
    TwoSpoolFuelTransient::new(d.clone(), flight(), 1.0, map_lp, map_hp, 1.0)
}

/// `pi_b` off the captured hardware — the divisor of the `pt4 → pt3` step, which rung 48 needs in
/// three gates and Python spells inline as `ft.pi_b`.
fn pi_b(core: &FuelTransientCore) -> f64 {
    core.inner.inner.base.pi_b
}

/// Python's `_ramp`: the accel fuel ramp + its running-line start.
///
/// **`min(1.0, s/r)`, NOT the marcher's `s >= r ⇒ mf_hi` branch.** See the header.
fn ramp(core: &FuelTransientCore, lo: f64, hi: f64, r: f64) -> (impl Fn(f64) -> f64, (f64, f64)) {
    let f = flight();
    let mf0 = core.fuel_for_tt4(&f, lo);
    let mf1 = core.fuel_for_tt4(&f, hi);
    let eq0 = core.inner.equilibrium(&f, lo);
    (move |s: f64| mf0 + (mf1 - mf0) * (s / r).min(1.0), (eq0.nu_lp, eq0.nu_hp))
}

/// Python's `KEYS` — the SEVEN-key tuple `_same` compares, `mf` INCLUDED, by EXHAUSTIVE read so a
/// renamed field breaks the build rather than narrowing the comparison silently.
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

/// The FIRST argmin's `s` — Python's `min(traj, key=...)` keeps the first of equals, so the fold
/// is STRICT. (The tie itself is § 5.17 finding 5's, booked to the step-4 oracle; nothing on this
/// file's grid exercises it — the closest gap to a second-smallest is `1.61e-5`.)
fn arg_min_s(traj: &[FuelPoint], key: fn(&FuelPoint) -> f64) -> f64 {
    let mut best = key(&traj[0]);
    let mut at = traj[0].s;
    for p in &traj[1..] {
        if key(p) < best {
            best = key(p);
            at = p.s;
        }
    }
    at
}

/// The first `s` at which the applied fuel sits below the schedule by more than the `1e-9`
/// relative bar — Python's engagement test, spelled the same way in `schedule_relief`.
fn first_clip_s(traj: &[FuelPoint]) -> Option<f64> {
    traj.iter().find(|p| p.mf < p.mf_sched * (1.0 - 1e-9)).map(|p| p.s)
}

/// The first `s` at which two trajectories differ on ANY of the seven keys.
fn first_diff_s(a: &[FuelPoint], b: &[FuelPoint]) -> Option<f64> {
    a.iter().zip(b.iter()).find(|(x, y)| keys7(x) != keys7(y)).map(|(x, _)| x.s)
}

fn sweep(margins: &[f64], r: f64, shape: &str) -> Vec<ScheduleRelief> {
    let (ml, mh) = shape_maps(shape);
    let d = design(cpg_gas());
    let t = ft(&d, ml, mh);
    t.core().engagement_sweep(&flight(), LO, HI, margins, r, SETTLE, DS, N_SCHED)
}

/// Python's `_SWEEPS` memo, for the ONE key three gates share (`MARGINS`, `r = 0.5`,
/// `flow/press`) — 12 marches become 6. The other four sweep keys have a single consumer each and
/// are built in place, as Python's memo also effectively does.
fn main_sweep() -> &'static [ScheduleRelief] {
    static S: OnceLock<Vec<ScheduleRelief>> = OnceLock::new();
    S.get_or_init(|| sweep(&MARGINS, R, "flow/press"))
}

/// The message of an `assert!` that fired, or `None` if the call returned. Rung 45/46/47's helper,
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
/// CONTRACT 1 — `accel = None` leaves rungs 45/46/47 bit-for-bit, because the leg is never
/// CONSULTED.
///
/// **PYTHON'S RAISER BECOMES A COUNTER, AND THE COUNTER IS PROVED LIVE BEFORE IT IS BELIEVED.**
/// `ft._sched_fuel = boom` witnesses "no call escaped as an exception";
/// [`counters`](turbojet::fuel_transient::counters)`::take().sched_calls == 0` witnesses "no call
/// happened", which is what the contract actually says. The risk the swap trades for is that an
/// `== 0` on a dead instrument is permanently green — so the first half of this gate ARMS a
/// `m = 0.25` schedule on the same object and the same march and asserts the counter MOVES.
///
/// The counters are thread-locals and libtest gives each `#[test]` its own thread, so the
/// reset→take window is this test's alone; it is kept tight around the three marches anyway.
#[test]
fn contract1_accel_none_never_consults_the_leg() {
    let f = flight();
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped());
    let core = t.core();
    let (sched, nu0) = ramp(core, LO, HI, R);

    // POSITIVE CONTROL: the instrument counts when the leg IS armed.
    let live = core.accel_schedule(&f, LO, HI, 0.25, N_SCHED);
    counters::reset();
    let armed = core.integrate_fuel(&f, &sched, nu0, R + 1.5, DS,
                                    &FuelLimiters { accel: Some(&live), ..Default::default() });
    let c_armed = counters::take();
    assert!(c_armed.sched_calls > 0,
            "the sched_calls counter must MOVE on an armed leg, or the zero below means nothing");
    assert!(!armed.is_empty());

    // THE CONTRACT: three `accel = None` marches consult the leg exactly zero times.
    counters::reset();
    let bare = core.integrate_fuel(&f, &sched, nu0, R + 1.5, DS, &FuelLimiters::default());
    let top = core.integrate_fuel(&f, &sched, nu0, R + 1.5, DS,
                                  &FuelLimiters { tt4_max: Some(REDLINE), ..Default::default() });
    let lag = core.integrate_fuel(
        &f, &sched, nu0, R + 1.5, DS,
        &FuelLimiters { tt4_max: Some(REDLINE), tau_gov: Some(0.2), ..Default::default() });
    let c = counters::take();
    assert_eq!((c.sched_calls, c.sched_dormant, c.sched_skips), (0, 0, 0),
               "an accel=None march must not consult the rung-48 leg at all: {c:?}");

    assert!(!bare.is_empty() && !top.is_empty() && !lag.is_empty());
    // ... and the three are genuinely different marches (the gate is not vacuous)
    assert!(peak_tt4(&bare) > peak_tt4(&top), "the governor must bite on the topped march");
    assert!(peak_tt4(&lag) > peak_tt4(&top), "the lag must overshoot the instantaneous governor");
}

// ============================================================================== contract 2
/// CONTRACT 2 — a margin above the march's max ratio leaves the cap above the schedule EVERYWHERE,
/// so [`try_sched_fuel`](turbojet::fuel_transient::FuelTransientCore::try_sched_fuel) returns its
/// argument float-identically and the trajectory is the bare rung-45 one BIT-for-bit, not merely
/// equal.
#[test]
fn contract2_dormant_schedule_is_bit_for_bit_rung45() {
    let f = flight();
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped());
    let core = t.core();
    let (sched, nu0) = ramp(core, LO, HI, R);
    let acc = core.accel_schedule(&f, LO, HI, 0.60, N_SCHED);

    let bare = core.integrate_fuel(&f, &sched, nu0, R + SETTLE, DS, &FuelLimiters::default());
    let dorm = core.integrate_fuel(&f, &sched, nu0, R + SETTLE, DS,
                                   &FuelLimiters { accel: Some(&acc), ..Default::default() });
    same(&bare, &dorm);
    assert!(dorm.iter().all(|p| p.mf == p.mf_sched), "a dormant leg must not clip");
}

// ============================================================================== contract 3
/// CONTRACT 3, both directions — the min-select ORDERING gate. Armed together, the pair reproduces
/// whichever single leg actually binds, bit-for-bit.
#[test]
fn contract3_two_leg_composite_min_select() {
    let f = flight();
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped());
    let core = t.core();
    let (sched, nu0) = ramp(core, LO, HI, R);
    let dorm = core.accel_schedule(&f, LO, HI, 0.60, N_SCHED); // never binds
    let live = core.accel_schedule(&f, LO, HI, 0.25, N_SCHED); // binds; its peak Tt4 ~1546

    // (a) accel dormant + redline armed  ==  redline only
    let top = core.integrate_fuel(&f, &sched, nu0, R + SETTLE, DS,
                                  &FuelLimiters { tt4_max: Some(REDLINE), ..Default::default() });
    let both_a = core.integrate_fuel(
        &f, &sched, nu0, R + SETTLE, DS,
        &FuelLimiters { tt4_max: Some(REDLINE), accel: Some(&dorm), ..Default::default() });
    same(&top, &both_a);

    // (b) accel armed + redline above the resulting peak  ==  accel only
    let acc_only = core.integrate_fuel(&f, &sched, nu0, R + SETTLE, DS,
                                       &FuelLimiters { accel: Some(&live), ..Default::default() });
    let peak = peak_tt4(&acc_only);
    let both_b = core.integrate_fuel(
        &f, &sched, nu0, R + SETTLE, DS,
        &FuelLimiters { tt4_max: Some(peak + 50.0), accel: Some(&live), ..Default::default() });
    same(&acc_only, &both_b);

    let bare = core.integrate_fuel(&f, &sched, nu0, R + SETTLE, DS, &FuelLimiters::default());
    assert!(acc_only.iter().any(|p| p.mf < p.mf_sched), "the (b) leg must genuinely bind");
    assert!(peak < peak_tt4(&bare) - 100.0,
            "...and genuinely move the march (peak {peak})");
}

// ============================================================================== contract 4
/// CONTRACT 4 — the finding is a PER-SPOOL split, so it is inherently two-shaft and the degenerate
/// object REFUSES the leg.
///
/// **THE `nu0` ARGUMENT CANNOT BE SPELLED AS PYTHON SPELLS IT.** Python hands the degenerate
/// object a two-shaft PAIR and the assert fires before anything reads it; Rust's
/// `integrate_fuel_lp_disabled` takes the scalar rung 35's marcher wants. `1.0` is passed, as rung
/// 47 gate 3 does, and the divergence is named here rather than hidden behind a helper.
#[test]
fn contract4_lp_disabled_refuses_the_leg() {
    let f = flight();
    let d = design(cpg_gas());
    let full = ft(&d, lp_shaped(), hp_shaped());
    let acc = full.core().accel_schedule(&f, LO, HI, 0.25, N_SCHED);

    let se: Engine = build_turbojet(cpg_gas(), 10.0, TT4, 50_000.0, single_matchable());
    let deg = TwoSpoolFuelTransient::lp_disabled(se, flight(), 1.0, hp_shaped());
    let m = refusal(|| {
        deg.integrate_fuel_lp_disabled(&f, |_s| 0.5, 1.0, R + 1.0, DS,
                                       &FuelLimiters { accel: Some(&acc), ..Default::default() });
    })
    .expect("the rung-48 leg on an lp_disabled object must refuse");
    assert!(m.contains("two-shaft"), "the refusal must name the reason: {m}");
}

// ============================================================================== contract 5
/// CONTRACT 5 — on a DECEL the fuel falls BELOW the running line, so `Wf/pt3` stays under
/// `kappa_ss` and the leg cannot fire at any margin ≥ 0 ⇒ the bare rung-45 march.
///
/// # ROW 0 IS WHERE THIS PORT SAYS SOMETHING PYTHON CANNOT
///
/// Python EXCLUDES row 0 and its docstring explains why: the decel starts ON the running line at
/// the band endpoint `kappa_ss` is tabulated at, so the leg computes `(a/b)*b` and compares it to
/// `a`. In binary that round-trip is 0–3 ulp off, and when it lands low the min-select "fires" by
/// 3 ulp. Its own text calls the passing case grid-luck — *"a ~40 % coincidence that HI = 1400
/// round-tripped exactly"*.
///
/// **MEASURED HERE INSTEAD OF INHERITED, AND THE PREDICTION WAS WRONG.** The first draft asserted
/// row 0 was BIT-IDENTICAL in Rust, reasoning that the dormant return is `mf_sched` ITSELF
/// (§ 5.16's structural zero) so a high round-trip leaves nothing to perturb. The run refuted it
/// immediately: the round-trip lands **LOW** here, exactly as it does in Python. Measured on this
/// cell:
///
/// | quantity | measured |
/// |---|---|
/// | `cap` vs `mf_sched` at row 0 | **1 ulp low** (`g = +3.47e-18`) |
/// | rows whose `mf` differs | **1 of 226** |
/// | leg consultations / of which DORMANT | **904 / 903** |
/// | row-0 ratio `dec/bare − 1` | `−1.11e-16`, against Python's `1e-12` bar |
/// | other row-0 keys | `Tt4` and `f` move 1 ulp; `nu_lp`, `nu_hp`, `phi_lp`, `phi_hp` do not |
///
/// So Python's exclusion is kept exactly, and what is ADDED is what its docstring only asserts in
/// prose: that the discrepancy is **confined to row 0**, and that the leg was consulted 903 times
/// WITHOUT firing and once WITH. The counter is the direct witness of the mechanism — one
/// non-dormant call, at `s = 0` — where the `[1..]` comparison can only witness its absence
/// downstream. The ulp bar is Python's own measured envelope (`0–3`), not this cell's `1`, because
/// its docstring records the die being re-rolled by an unrelated change to the operands.
#[test]
fn contract5_decel_never_fires_bit_for_bit_rung45() {
    let f = flight();
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped());
    let core = t.core();
    let (sched, nu0) = ramp(core, HI, LO, R);
    let acc = core.accel_schedule(&f, HI, LO, 0.0, N_SCHED); // the TIGHTEST schedule

    let bare = core.integrate_fuel(&f, &sched, nu0, R + SETTLE, DS, &FuelLimiters::default());
    counters::reset();
    let dec = core.integrate_fuel(&f, &sched, nu0, R + SETTLE, DS,
                                  &FuelLimiters { accel: Some(&acc), ..Default::default() });
    let c = counters::take();
    assert_eq!(bare.len(), dec.len());
    assert!(bare.len() > 1);

    // Python's own comparison, from row 1 — its contract, unchanged.
    same(&bare[1..], &dec[1..]);
    assert!(dec[1..].iter().all(|p| p.mf == p.mf_sched), "the leg must not clip on a decel");
    assert!((dec[0].mf / bare[0].mf - 1.0).abs() < 1e-12,
            "row 0 may differ only by the kappa_ss round-trip, never physically: {} vs {}",
            bare[0].mf, dec[0].mf);

    // ... and what Python's docstring only claims in prose, gated. (a) the ulp scale, against its
    // own measured 0-3 envelope rather than this cell's 1; (b) confinement to row 0; (c) the
    // mechanism itself — every consultation but ONE found the leg dormant, and that one is at s=0.
    let ulps = (dec[0].mf.to_bits() as i64 - bare[0].mf.to_bits() as i64).abs();
    assert!(ulps <= 3, "the row-0 round-trip must stay at the ulp scale, got {ulps}");
    let moved: Vec<f64> = bare.iter().zip(dec.iter())
        .filter(|(a, b)| keys7(a) != keys7(b))
        .map(|(a, _)| a.s)
        .collect();
    assert_eq!(moved, vec![0.0], "the discrepancy must be CONFINED to row 0: {moved:?}");
    assert_eq!(c.sched_calls - c.sched_dormant, 1,
               "exactly one of the leg's {} consultations may leave the dormant fast path, and it \
                is the s=0 round-trip: {c:?}", c.sched_calls);
}

// ============================================================================== contract 6
/// CONTRACT 6 — exercising the leg must not perturb the default design run.
#[test]
fn contract6_cycle_untouched_bit_for_bit_rung6() {
    let f = flight();
    let eng = build_turbojet(Gas::reacting_equilibrium(), 10.0, TT4, 50_000.0, single());
    let a = eng.run(&f, 1.0);

    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped());
    let core = t.core();
    let acc = core.accel_schedule(&f, LO, HI, 0.25, N_SCHED);
    let _ = core.schedule_relief(&f, LO, HI, &acc, R, 1.5, DS, None, None);

    let b = eng.run(&f, 1.0);
    assert_eq!(a.performance.specific_thrust.to_bits(), b.performance.specific_thrust.to_bits());
    assert_eq!(a.station("4").far.to_bits(), b.station("4").far.to_bits());
}

// ============================================================================== gate 6
/// GATE 6 — `kappa_ss` is READ OFF the plant's own equilibria: at a steady point the `m = 0` cap
/// IS that point's own fuel. And `pt3 == pi_HPC*pi_LPC*pt2`, checked DIRECTLY against the inlet,
/// not by dividing out the factors it multiplies back.
///
/// **THE TABLE-LENGTH ASSERTION IS WEAKER HERE THAN IN PYTHON.** `13` is Python's DEFAULT `n`;
/// Rust makes the argument explicit, so all this can check is that the caller passed 13. Written
/// against the literal, never against [`N_SCHED`], so it cannot become `x == x`.
#[test]
fn gate6_kappa_is_derived_and_the_pt3_identity_holds() {
    let f = flight();
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped());
    let core = t.core();
    let acc0 = core.accel_schedule(&f, LO, HI, 0.0, N_SCHED);
    let (_tt2, pt2, _v0) = core.inner.inlet(&f);

    for tt4 in [1100.0, 1250.0, 1350.0] {
        let eq = core.inner.equilibrium(&f, tt4);
        let pt3 = eq.close.pt4 / pi_b(core);
        assert!((pt3 - eq.close.pi_hpc * eq.close.pi_lpc * pt2).abs() < 1e-9 * pt2,
                "pt3 identity at Tt4={tt4}");
        let wf = eq.close.f * eq.close.mdot_air;
        assert!((acc0.cap(eq.close.n_hp, pt3) / wf - 1.0).abs() < 2e-3,
                "the m=0 cap must BE the steady fuel at that speed (Tt4={tt4})");
    }

    // the one imposed scalar scales the cap exactly
    let acc = core.accel_schedule(&f, LO, HI, 0.30, N_SCHED);
    assert!((acc.cap(0.90, 1e5) / acc0.cap(0.90, 1e5) - 1.30).abs() < 1e-12);
    assert_eq!(acc.n_h.len(), 13, "the derived table is Python's 13-point band");
    assert_eq!(acc.kappa.len(), 13);
}

// ============================================================================== gate 7
/// GATE 7 / FINDING 1 (the ENABLING measurement) — on the BARE accel the ratio
/// `(Wf/pt3)/kappa_ss` rises MONOTONICALLY and is already far above 1 UPSTREAM of the LP surge
/// minimum. That is what makes `m` an engagement-TIME instrument: it can be placed on either side
/// of `s_lp*`. Gated as a SIGN (monotone + a floor), never as a level.
#[test]
fn gate7_the_window_exists_the_ratio_rises_through_the_lp_minimum() {
    let f = flight();
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped());
    let core = t.core();
    let (sched, nu0) = ramp(core, LO, HI, R);
    let traj = core.integrate_fuel(&f, &sched, nu0, R + SETTLE, DS, &FuelLimiters::default());
    let acc0 = core.accel_schedule(&f, LO, HI, 0.0, N_SCHED);
    let s_lp = arg_min_s(&traj, |p| p.phi_lp);

    // Python iterates the trajectory in order and BREAKS past the ramp end.
    let mut ratio: Vec<(f64, f64)> = Vec::new();
    for p in &traj {
        if p.s > R {
            break;
        }
        let i = core.instant_fuel(&f, p.nu_lp, p.nu_hp, p.mf);
        ratio.push((p.s, p.mf / acc0.cap(i.base.close.n_hp, i.base.close.pt4 / pi_b(core))));
    }
    assert!(!ratio.is_empty());
    assert!((ratio[0].1 - 1.0).abs() < 1e-6,
            "the march STARTS on the running line => ratio 1 (got {})", ratio[0].1);

    let upto: Vec<f64> = ratio.iter().filter(|&&(s, _)| s <= s_lp).map(|&(_, v)| v).collect();
    assert!(upto.windows(2).all(|w| w[1] > w[0]), "monotone through s_lp*");
    let at_lp = *ratio.iter().filter(|&&(s, _)| s <= s_lp).map(|(_, v)| v).last().unwrap();
    assert!(at_lp > 1.15,
            "the ratio at the LP min must clear kappa_ss with room -- otherwise engaging there \
             throttles the whole ramp ({at_lp})");
    let early = ratio.iter().filter(|&&(s, _)| s <= 0.5 * s_lp).map(|&(_, v)| v).last()
        .expect("the ramp has points upstream of half the LP minimum");
    assert!(early > 1.10, "and it must ALREADY be clear well UPSTREAM ({early})");
}

// ============================================================================== gate 8
/// GATE 8 / FINDING 2 (the headline) — `relief_lp > 0` for EVERY margin whose engagement is
/// UPSTREAM of the LP surge minimum, and EXACTLY 0 for every margin engaging downstream.
///
/// The `== 0.0` sites here are two of the six § 5.17 finding 3 says are ONE claim (P2).
#[test]
fn gate8_the_lp_rebate_switches_off_exactly_at_s_lp() {
    let rows = main_sweep();
    let s_lp = rows[0].s_lp_bare;
    assert!(rows.iter().all(|x| (x.s_lp_bare - s_lp).abs() < 1e-12), "one bare march, one s_lp*");

    let up: Vec<&ScheduleRelief> =
        rows.iter().filter(|x| x.n_engaged > 0 && x.s_eng < s_lp - 1e-12).collect();
    let down: Vec<&ScheduleRelief> =
        rows.iter().filter(|x| x.n_engaged > 0 && x.s_eng > s_lp + 1e-12).collect();
    assert!(up.len() >= 2 && down.len() >= 2,
            "the sweep must straddle the crossing: {:?}",
            rows.iter().map(|x| (x.margin, x.s_eng)).collect::<Vec<_>>());
    for x in &up {
        assert!(x.relief_lp > 0.0, "upstream engagement MUST rebate the LP (m={})", x.margin);
    }
    for x in &down {
        assert_eq!(x.relief_lp, 0.0,
                   "downstream engagement rebates the LP EXACTLY nothing (m={}, got {})",
                   x.margin, x.relief_lp);
    }

    // s_eng is monotone in m -- `m` really is the engagement-time dial
    let eng: Vec<(f64, f64)> =
        rows.iter().filter(|x| x.n_engaged > 0).map(|x| (x.margin, x.s_eng)).collect();
    assert!(eng.windows(2).all(|w| w[1].1 >= w[0].1), "{eng:?}");
}

// ============================================================================== gate 8b
/// GATE 8b — the MECHANISM behind gate 8's `relief_lp == 0.0`, not just its consequence.
///
/// "EXACTLY 0" says the limited march never differs from the bare one ANYWHERE at or before the LP
/// minimum, so the minimum itself is the same float. Gate 8 checks the differenced minima; this
/// checks the cause. Without it, an upstream one-ULP perturbation that happened to leave
/// `min_phi_lp` rounding the same would pass gate 8 while the claim was false.
#[test]
fn gate8b_a_downstream_clip_is_bit_identical_through_the_minimum() {
    let f = flight();
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped());
    let core = t.core();
    let (bare, _) = core.fuel_ramp_march(&f, LO, HI, R, SETTLE, DS, &FuelLimiters::default());
    let s_lp = arg_min_s(&bare, |p| p.phi_lp);

    for m in [0.42, 0.45, 0.48] {
        let acc = core.accel_schedule(&f, LO, HI, m, N_SCHED);
        let (lim, _) = core.fuel_ramp_march(&f, LO, HI, R, SETTLE, DS,
                                            &FuelLimiters { accel: Some(&acc),
                                                            ..Default::default() });
        let s_eng = first_clip_s(&lim).unwrap_or_else(|| panic!("this gate needs a clip at m={m}"));
        let first_diff = first_diff_s(&bare, &lim)
            .unwrap_or_else(|| panic!("...that genuinely moves the march, at m={m}"));
        assert!(s_eng > s_lp, "this gate needs a DOWNSTREAM clip (m={m}, s_eng={s_eng})");
        assert!(first_diff > s_lp,
                "a downstream clip must leave the whole pre-minimum march BIT-IDENTICAL \
                 (m={m}, first_diff={first_diff}, s_lp={s_lp})");
        assert!((first_diff - s_eng).abs() < 1e-9,
                "and the march must diverge exactly AT engagement, not before \
                 (m={m}, {first_diff} vs {s_eng})");
    }
}

// ============================================================================== gate 9b
/// GATE 9b — the HP crossing to the LP's standard, on a SLOWER ramp.
///
/// At `r = 0.5` the ratio peak (~1.49) runs out of dial just as `s_eng` reaches `s_hp*`, so the HP
/// side there shows only a COLLAPSE (+0.000016), not a clean exact zero. At `r = 2.0`,
/// `s_hp* = 0.64` and `m = 0.20` engages at `s = 0.70`, strictly PAST it, with fuel still being
/// removed: `relief_hp` is then EXACTLY 0 and the march is bit-identical through BOTH minima.
#[test]
fn gate9b_the_hp_crossing_on_a_slow_ramp() {
    let f = flight();
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped());
    let core = t.core();
    let (bare, _) = core.fuel_ramp_march(&f, LO, HI, 2.0, SETTLE, DS, &FuelLimiters::default());
    let s_lp = arg_min_s(&bare, |p| p.phi_lp);
    let s_hp = arg_min_s(&bare, |p| p.phi_hp);

    let acc = core.accel_schedule(&f, LO, HI, 0.20, N_SCHED);
    let (lim, _) = core.fuel_ramp_march(&f, LO, HI, 2.0, SETTLE, DS,
                                        &FuelLimiters { accel: Some(&acc), ..Default::default() });
    let s_eng = first_clip_s(&lim).expect("the m=0.20 leg must engage on the slow ramp");
    assert!(s_eng > s_hp && s_hp > s_lp, "({s_eng}, {s_hp}, {s_lp})");

    let row = core.schedule_relief(&f, LO, HI, &acc, 2.0, SETTLE, DS, None, None);
    assert!(row.fuel_removed > 0.0, "fuel must genuinely be removed where the HP gets nothing");
    assert_eq!((row.relief_lp, row.relief_hp), (0.0, 0.0),
               "past BOTH minima, both reliefs are exactly zero");

    let first_diff = first_diff_s(&bare, &lim).expect("the clip must move the march");
    assert!(first_diff > s_hp,
            "bit-identical through BOTH minima -- the mechanism, on the HP side too ({first_diff})");
}

// ============================================================================== gate 9
/// GATE 9 / FINDING 2 (the split) — the SAME instrument crosses the HP minimum LATER: at the
/// margins where `relief_lp` is already exactly 0, `relief_hp` is STILL POSITIVE, and it survives
/// until `s_eng` reaches `s_hp*`. The rung-46/47 LP/HP split is this, and only this.
#[test]
fn gate9_the_hp_crossing_is_later_and_that_is_the_split() {
    let rows = main_sweep();
    let (s_lp, s_hp) = (rows[0].s_lp_bare, rows[0].s_hp_bare);
    assert!(s_hp > s_lp,
            "the HP minimum must sit LATER for the split to be readable at this r ({s_lp}, {s_hp})");

    let between: Vec<&ScheduleRelief> = rows.iter()
        .filter(|x| x.n_engaged > 0 && s_lp < x.s_eng && x.s_eng < s_hp - 1e-12)
        .collect();
    assert!(!between.is_empty(), "{:?}",
            rows.iter().map(|x| (x.margin, x.s_eng)).collect::<Vec<_>>());
    for x in &between {
        assert!(x.relief_lp == 0.0 && x.relief_hp > 0.0,
                "between the two minima the SAME clip rebates the HP and not the LP \
                 (m={}, {} / {})", x.margin, x.relief_lp, x.relief_hp);
    }

    // and the HP relief dies once engagement reaches its own minimum
    let at_hp: Vec<&ScheduleRelief> =
        rows.iter().filter(|x| x.n_engaged > 0 && x.s_eng >= s_hp - 1e-12).collect();
    if let Some(first) = at_hp.first() {
        assert!(first.relief_hp < between[between.len() - 1].relief_hp / 10.0,
                "the HP rebate must collapse as engagement reaches s_hp* ({})", first.relief_hp);
    }
}

// ============================================================================== gate 10
/// GATE 10 / FINDING 3 — the deflation *"any clip removes fuel and slows the accel, so this is
/// rung 44 restated"* is EXCLUDED on three counts measured together: the removed fuel stays
/// strictly positive and varies SMOOTHLY through the crossing at which `relief_lp` switches
/// exactly off; the settled endpoint is UNMOVED; and at a single margin ONE clip removing ONE
/// quantity of fuel rebates the HP and not the LP.
#[test]
fn gate10_not_rung_44s_ramp_rate_lever() {
    let rows = main_sweep();
    let s_lp = rows[0].s_lp_bare;
    let live: Vec<&ScheduleRelief> = rows.iter().filter(|x| x.n_engaged > 0).collect();
    assert!(live.iter().all(|x| x.fuel_removed > 0.0), "every armed margin removes fuel");
    assert!(live.windows(2).all(|w| w[1].fuel_removed < w[0].fuel_removed),
            "fuel removed must fall SMOOTHLY (monotonically) in m -- no step at the crossing: {:?}",
            live.iter().map(|x| (x.margin, x.fuel_removed)).collect::<Vec<_>>());
    for x in &live {
        // 5e-4 (0.05%), not 1e-4: the longest in-window engagement (m=0.15) moves the settled
        // endpoint by 0.012%. The m -> 0 corner, where it moves by ~9%, is gate 11's.
        assert!((x.nu_hp_end - x.nu_hp_end_bare).abs() < 5e-4,
                "the endpoint must be unmoved -- else the comparison is not same-endpoint \
                 (m={}, {} vs {})", x.margin, x.nu_hp_end, x.nu_hp_end_bare);
    }
    let split: Vec<&&ScheduleRelief> =
        live.iter().filter(|x| x.s_eng > s_lp && x.relief_hp > 0.0).collect();
    assert!(!split.is_empty(),
            "the per-spool split at fixed fuel-removed is the clincher -- it must exist");
    for x in &split {
        assert!(x.relief_lp == 0.0 && x.fuel_removed > 0.0);
    }
}

// ============================================================================== gate 11
/// GATE 11 / FINDING 4 (the HONEST BOUNDARY, gated so it cannot be quietly folded into the
/// finding) — at a small enough margin the leg binds from the start and never releases: the accel
/// does NOT complete inside the window and the leg HAS become rung 44's ramp-rate lever.
///
/// § 5.17 finding 7 measured that this corner COMPLETES rather than refusing (`m = 0.02 / 0.05 /
/// 0.10` all march to the end), so a Rust-side refusal here is a defect, not a divergence.
#[test]
fn gate11_the_degeneracy_boundary_is_gated_not_hidden() {
    let rows = sweep(&[0.05, MARGINS[0]], R, "flow/press");
    let (deg, ok) = (&rows[0], &rows[1]);
    assert!(deg.nu_hp_end_bare - deg.nu_hp_end > 1e-2,
            "m=0.05 must visibly fail to complete the accel ({} vs {})",
            deg.nu_hp_end, deg.nu_hp_end_bare);
    assert!(deg.tt4_peak_lim < deg.tt4_peak_bare - 300.0, "and de-fang the accel outright");
    assert!((ok.nu_hp_end - ok.nu_hp_end_bare).abs() < 5e-4,
            "while the in-window margin leaves the endpoint alone");
}

// ============================================================================== gate 12
/// GATE 12 / FINDING 5 — at `r = 0.15` the LP and HP minima COINCIDE, so the rule predicts ONE
/// crossing rather than a split, and both reliefs die together. A degenerate case that would have
/// broken a "the LP spool is special" reading.
///
/// § 5.17 finding 5 measured that the two minima are not merely close: they are the SAME POINT
/// (index 7 of one trajectory, `|s_lp − s_hp|` exactly `0.0`), so the `1e-9` bar below is not
/// measuring a near-coincidence.
#[test]
fn gate12_coincident_minima_give_one_crossing() {
    let rows = sweep(&[0.60, 0.70, 0.78], 0.15, "flow/press");
    let (s_lp, s_hp) = (rows[0].s_lp_bare, rows[0].s_hp_bare);
    assert!((s_lp - s_hp).abs() < 1e-9,
            "the minima must coincide at this ramp rate ({s_lp}, {s_hp})");
    for x in &rows {
        if x.n_engaged > 0 && x.s_eng < s_lp - 1e-12 {
            assert!(x.relief_lp > 0.0 && x.relief_hp > 0.0);
        } else if x.n_engaged > 0 && x.s_eng > s_lp + 1e-12 {
            assert!(x.relief_lp == 0.0 && x.relief_hp == 0.0,
                    "coincident minima => the two crossings coincide (m={})", x.margin);
        }
    }
}

// ============================================================================== gate 13
/// GATE 13 — the crossing rule is a TIMING statement, not an artifact of one map pair: it holds on
/// the shape set including the mode-free `hp-only` (LP map FLAT ⇒ no rung-40 complex inter-spool
/// mode), so the rule does not ride on that mode.
#[test]
fn gate13_the_crossing_rule_across_map_shapes() {
    for shape in ["tilted", "hp-only"] {
        let rows = sweep(&[0.25, 0.45], R, shape);
        let s_lp = rows[0].s_lp_bare;
        for x in &rows {
            if x.n_engaged == 0 {
                continue;
            }
            if x.s_eng < s_lp - 1e-12 {
                assert!(x.relief_lp > 0.0, "{shape} m={} upstream must rebate", x.margin);
            } else if x.s_eng > s_lp + 1e-12 {
                assert_eq!(x.relief_lp, 0.0, "{shape} m={} downstream exactly nothing", x.margin);
            }
        }
    }
}
