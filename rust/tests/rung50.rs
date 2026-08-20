//! RUNG 50 — THE RELEASE EDGE, ISOLATED: the closing edge relocates BOTH spools' minima to
//! itself, and a limiter's immunity is TIMING, not clip SHAPE.
//!
//! Port of `tests/test_rung50.py`, gate for gate. That file defines **15 test functions** and
//! collects **15 items** — no `parametrize`, and **no `slow` mark** (§ 5.18 counted the slice's
//! four with `--collect-only -m slow` and all four are in `test_rung52.py`).
//!
//! **PYTHON LABELS ITS OWN GATES AND THE NUMBERING RESTARTS AT 3 AGAIN**, exactly as rung 49's
//! did — six `CONTRACT`s, then `GATE 3, 4, 5, 6, 7, 8, 9, 10, 10b`. The names below carry
//! Python's label, not a renumbering.
//!
//! | # | `tests/test_rung50.py` | here |
//! |---|---|---|
//! | 1 | `test_reduce_s_off_none_never_gates_the_legs_bit_for_bit` | [`contract1_s_off_none_never_gates_the_legs`] |
//! | 2 | `test_reduce_release_relief_none_is_rung49_surge_relief_bit_for_bit` | [`contract1b_release_relief_none_is_rung49s_reader`] |
//! | 3 | `test_reduce_late_s_off_is_inert_and_early_s_off_is_bare_bit_for_bit` | [`contract2_3_late_s_off_inert_early_s_off_bare`] |
//! | 4 | `test_reduce_s_off_without_an_armed_leg_asserts` | [`contract4_s_off_without_an_armed_leg_refuses`] |
//! | 5 | `test_reduce_lp_disabled_asserts` | [`contract5_lp_disabled_refuses`] |
//! | 6 | `test_cycle_untouched_by_the_forced_release_bit_for_bit_rung6` | [`contract6_cycle_untouched_bit_for_bit_rung6`] |
//! | 7 | `test_headline_the_release_edge_relocates_BOTH_minima_to_itself` | [`gate3_the_release_edge_relocates_both_minima_to_itself`] |
//! | 8 | `test_discriminator_the_debit_is_RAMP_clocked_deconfounded` | [`gate4_the_debit_is_ramp_clocked_deconfounded`] |
//! | 9 | `test_the_watched_spool_is_DEBITED_when_released_early_rung49_bounded` | [`gate5_the_watched_spool_is_debited_when_released_early`] |
//! | 10 | `test_SEAM_rung48s_immunity_is_TIMING_not_clip_SHAPE` | [`gate6_seam_rung48s_immunity_is_timing_not_clip_shape`] |
//! | 11 | `test_SEAM_cross_regime_at_r2_and_rung48s_exact_zero_survives` | [`gate7_seam_cross_regime_and_rung48s_exact_zero_survives`] |
//! | 12 | `test_the_deficit_factor_at_FIXED_release_rung49_section4_corrected` | [`gate8_the_deficit_factor_at_a_fixed_release`] |
//! | 13 | `test_not_the_ramp_rate_lever_the_non_tautology` | [`gate9_not_rung_44s_ramp_rate_lever`] |
//! | 14 | `test_robustness_ds_convergence_of_the_relocation_and_the_debit` | [`gate10_ds_convergence_of_the_relocation_and_the_debit`] |
//! | 15 | `test_robustness_the_split_survives_rho` | [`gate10b_the_split_survives_rho`] |
//!
//! # THE GRID IS THIS FILE'S — AND ITS MEMO STRUCTURE IS *NOT* RUNG 49'S
//!
//! `SETTLE` is **2.0**, the same as rung 49's and not rung 48's 4.0. But the sweep structure is
//! different in a way that would be easy to "tidy" wrongly: Python memoises on the OFFSET TUPLE
//! as well as the leg, so of eleven distinct sweep keys **exactly one has two consumers**
//! (`R2_OFFS` at `PHI_LIM_2`, `r = 2.0`, `settle = 2.0` — gates 3 and 4). Two traps follow:
//!
//! * **gate 9 runs the SAME offsets and the SAME floor at `settle = 4.0`.** It is a different
//!   memo key and a different march length. Collapsing it into gates 3/4's entry is a different
//!   measurement, not an optimisation.
//! * **gate 10's `(1.10, 1.56)` is a SUBSET of `R2_OFFS` at an otherwise identical key.** Python
//!   still recomputes it, because the tuple is part of the key. Slicing the larger sweep instead
//!   would give the same numbers *here* and quietly stop being a port.
//!
//! So there is ONE [`OnceLock`], not rung 49's two, and every other sweep is called directly.
//!
//! # Where Rust has to say out loud what Python leaves implicit
//!
//! * **CONTRACT 4 asserts the FULL refusal text, where Python matches nothing at all.** Python
//!   uses a bare `pytest.raises(AssertionError)` three times, which cannot say WHICH of the
//!   marcher's asserts fired. Two of the three fire the composition assert and the third fires
//!   `release_relief`'s own; asserted apart here.
//! * **CONTRACT 5 IS § 5.18 P2's INSTRUMENT, AND IT ASSERTS THE WHOLE MESSAGE.** Python matches
//!   the substring `"inherently two-shaft"`, which four rungs' `lp_disabled` gates all satisfy.
//!   § 5.18 finding 1 measured that rungs 50/51/52's own `lp_disabled` refusals are **unreachable
//!   over all 255 arming combinations** — arming `s_off` requires an armed leg, and the `surge`
//!   refusal precedes it inside the block — so this gate, named for rung 50, fires **rung 49's**
//!   assert. Asserting the full string turns P2 from an inference off that mechanism into a
//!   measurement, at its second of four data points. Rung 49's file could not do this: its own
//!   gate is the one that *is* named correctly.
//! * **THE FORCED-RELEASE COMPARISON SITE IS A ONE-*CELL* KNIFE EDGE.** Measured before these
//!   gates were written: over the eighteen `s_off` values this suite passes at `ds ∈ {0.02,
//!   0.01}`, six comparison sites change the last armed index by a WHOLE GRID CELL between the
//!   accumulated march coordinate and a "cleaner" `k * ds`, and **two are live cells here** —
//!   `s_off = 0.20` and `0.26` at `ds = 0.02`, read by gates 5 and 10b. § 5.18 finding 3's
//!   one-ulp boolean is the finer hazard; this is the coarser one, and it is a different site.
//!   Both languages accumulate, so both agree. See
//!   [`release_relief`](turbojet::fuel_transient::FuelTransientCore::release_relief)'s note.
//! * **`relief_watched` / `relief_other` ARE `Option` HERE AND PLAIN FLOATS IN RUNG 49**, because
//!   this reader's `surge` is optional. Gates 6 and 7 run accel-only cells where they are `None`.
//!   No gate in this file reads either key — the census below — so the unwrapping happens only in
//!   the step-5 oracle.
//! * **CONTRACT 1b COMPARES TEN KEYS ACROSS TWO DIFFERENT STRUCTS.** Python's dicts share the
//!   names; Rust's [`ReleaseRelief`] and [`SurgeRelief`] are separate types (§ 5.18 finding 5), so
//!   the comparison is field by field and the list is Python's ten, not "every common field".
//!
//! # THE READER CENSUS — **16** OF THE 27 KEYS HAVE NO READER IN THIS SUITE
//!
//! Measured by stripping the docstrings out of `test_rung50.py` and counting `["key"]` sites, then
//! confirmed at the call sites — and the count is the measurement's, not an eyeball's: a first
//! writing of this paragraph said 13 and was wrong.
//!
//! **Read (11)**: `s_off` (22 sites), `relief_hp` (21), `relief_lp` (13), `s_rel` (10),
//! `s_min_lp` (5), `fuel_removed` (4), `nu_hp_end` (2), `nu_hp_end_bare` (2), `min_phi_lp_bare`
//! (1), `s_eng` (1), `s_min_hp` (1).
//!
//! **Unread (16)**: `deficit_at_release`, `ds`, `margin`, `min_phi_hp_bare`, `min_phi_hp_lim`,
//! `min_phi_lp_lim`, `n_engaged`, `phi_lim`, `r`, `relief_other`, `relief_watched`, `rho`,
//! `s_hp_bare`, `s_lp_bare`, `spool`, `tau_rel`.
//!
//! `deficit_at_release` is worth naming: it is **the rung's own named quantity** — gate 8's whole
//! subject is the deficit — and the gate reads `fuel_removed` as its proxy instead, so the key
//! that carries the concept has no gate anywhere in the project. That is step 5's oracle's job,
//! not this file's.
//!
//! # FOURTEEN INJECTIONS, AND THE DEFENDER AND THE EXPOSURE ARE ON DISJOINT CELLS
//!
//! Bit-exactness says the port is faithful; it says nothing about which of the 15 gates has POWER.
//! Each plausible port defect was written into the shipped `release_relief` on purpose (§ 5.18
//! step 2). Uncaught, and measured live rather than inferred:
//!
//! * **the march coordinate spelled `k * ds`** — moves `n_engaged` from 8 to 7 and `s_rel` by a
//!   WHOLE GRID CELL on the two knife-edge rows, and **no gate notices**. `gate10` IS this file's
//!   only reader of the release edge's LOCATION — it catches the same one-cell shift written
//!   directly at the comparison site (`s <= s_off`) — but gate 10 sweeps `0.30 / 0.40 / 0.44` and
//!   `1.10 / 1.56`, and **not one of those is a knife-edge cell**. The two gates that DO sweep
//!   them (5 at `0.20` and `0.26`, 10b at `0.26`) read only "some `relief_lp < 0`" and "the worst
//!   row is upstream of `s_hp*`", neither of which a one-cell shift flips.
//! * **`deficit_at_release`'s value, and its `eng[-1]`-vs-`eng[0]` choice** — no reader.
//! * **`relief_watched` / `relief_other` swapped** — no reader.
//! * **`nu_hp_end_bare` read off the LIMITED march** — and its twin `nu_hp_end` off the BARE one
//!   IS caught, by `contract1b` alone. The only difference between them is that Python's ten-key
//!   list in that contract names one and not the other.
//! * **`fuel_removed` losing its `0.5`** is caught by `contract1b` — but only as a DIFFERENCE
//!   against `surge_relief`'s copy. Break all THREE copies in the module identically and rungs
//!   48, 49 and 50 are all green again, measured. Slice T step 3's finding stands, sharpened: a
//!   duplicated computation is held against its duplicate, never against the truth.
//!
//! Caught, for the record: `s_min_lp`/`s_min_hp` swapped and `s_min_lp` off the bare march (gates
//! 3 and 10); `s_eng`/`s_rel` swapped (five gates); `min_phi_lp_lim`/`_bare` swapped (contract 1b
//! and gate 5). And **the engaged mask losing its `1e-9` slack moves NOTHING** on all 49 cells —
//! a statement about the grid, not about the gates.
//!
//! # `#[ignore]`
//!
//! None. `test_rung50.py` carries no `slow` mark.
//!
//! [`OnceLock`]: std::sync::OnceLock
//! [`ReleaseRelief`]: turbojet::fuel_transient::ReleaseRelief
//! [`SurgeRelief`]: turbojet::fuel_transient::SurgeRelief

use std::sync::OnceLock;

use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::fuel_transient::{
    AccelSchedule, FuelLimiters, FuelPoint, FuelTransientCore, ReleaseRelief, SurgeLimiter,
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
/// **2.0.** Only gate 9 wants a settled endpoint and it passes `4.0` itself.
const SETTLE: f64 = 2.0;
const DS: f64 = 0.02;
const REDLINE: f64 = 1480.0;

/// The bare march's raw surge minima at this config, from
/// `docs/plans/rung50-anchor-release-edge.md` via `test_rung50.py:70-73`.
/// **DECLARED AND NEVER READ, IN PYTHON TOO.** `test_rung50.py` names it on line 71 beside
/// `S_HP_STAR` and no gate uses it — measured by stripping the docstrings and counting word
/// occurrences: exactly **one**, its own binding. Kept because the port is a copy, not a tidy-up
/// (*COPY vs REDERIVATION*); the `allow` is the whole cost, and Rust reporting it is the whole
/// benefit — Python never says a module constant is dead.
#[allow(dead_code)]
const S_LP_STAR: f64 = 0.240;
const S_HP_STAR: f64 = 0.400;
const S_LP_STAR_2: f64 = 0.320;
const S_HP_STAR_2: f64 = 0.640;
/// The `r = 0.5` working floor.
const PHI_LIM: f64 = 0.7450;
/// The `r = 2.0` floor — its natural `s_rel` is 2.10.
const PHI_LIM_2: f64 = 0.7725;

const R2_OFFS: [f64; 7] = [0.30, 0.66, 1.10, 1.56, 1.80, 2.06, 2.20];

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

/// `test_rung50.py`'s `SINGLE`. No `nozzle_convergent` — admissible for contract 6's cycle run,
/// which is its only consumer.
fn single() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.90, e_c: None, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, e_t: None,
        eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: false,
    }
}

/// [`single`] plus the one constant contract 5 needs to have a degenerate object at all.
fn single_matchable() -> Losses {
    Losses { nozzle_convergent: true, ..single() }
}

/// `test_rung50.py`'s `_cpg_gas` — `R_c` / `R_t` DERIVED from the pair above them.
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

fn design(gas: Gas) -> TwoSpoolEngine {
    build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, 50_000.0, real())
}

/// `test_rung50.py`'s `_ft`. **Only ONE map pair here** — this file has no `SHAPES` table at all,
/// where rung 49 has two entries and rung 48 three.
fn ft(d: &TwoSpoolEngine, rho: f64) -> TwoSpoolFuelTransient {
    TwoSpoolFuelTransient::new(d.clone(), flight(), 1.0, lp_shaped(), hp_shaped(), rho)
}

/// Python's `_ramp`.
///
/// **`min(1.0, s/r)`, NOT the marcher's `s >= r ⇒ mf_hi` branch** — those differ by an ulp at and
/// past the ramp end. Contracts 1/3/4/5 use this form (they call `integrate_fuel` directly); every
/// gate that goes through `release_relief` gets the branch form. Deliberately not unified.
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

/// Python's `_sweep` body, un-memoised. `phi_lim = None` is a surge-less (accel-only) cell and
/// `margin = None` an accel-less one, exactly as Python's two `if … is not None` guards read.
#[allow(clippy::too_many_arguments)]
fn sweep(s_offs: &[f64], phi_lim: Option<f64>, margin: Option<f64>, r: f64, settle: f64, ds: f64,
         rho: f64) -> Vec<ReleaseRelief>
{
    let d = design(cpg_gas());
    let t = ft(&d, rho);
    let c = t.core();
    let leg = phi_lim.map(|p| SurgeLimiter::new(Spool::Lp, p));
    let acc: Option<AccelSchedule> = margin.map(|m| c.accel_schedule(&flight(), LO, HI, m, 13));
    c.release_sweep(&flight(), LO, HI, s_offs, leg.as_ref(), acc.as_ref(), r, settle, ds)
}

/// The ONE sweep key with two consumers — gates 3 and 4. See the header for why gate 9's
/// `settle = 4.0` and gate 10's `(1.10, 1.56)` are NOT folded in here.
fn r2_sweep() -> &'static [ReleaseRelief] {
    static S: OnceLock<Vec<ReleaseRelief>> = OnceLock::new();
    S.get_or_init(|| sweep(&R2_OFFS, Some(PHI_LIM_2), None, 2.0, SETTLE, DS, 1.0))
}

/// The message of an `assert!` that fired, or `None` if the call returned. Rungs 45–49's helper,
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
/// CONTRACT 1 — `s_off = None` leaves rungs 43/45/46/47/48/49 bit-for-bit. Guaranteed at CODE
/// level ([`release_weight`] short-circuits on `s_off = None` to exactly `1.0`), which is what
/// this gate witnesses: the seven prior configurations are reproduced through the NEW signature,
/// bit-identically.
///
/// [`release_weight`]: turbojet::fuel_transient::release_weight
#[test]
fn contract1_s_off_none_never_gates_the_legs() {
    let f = flight();
    let d = design(cpg_gas());
    let t = ft(&d, 1.0);
    let core = t.core();
    let (sched, nu0) = ramp(core, LO, HI, R);
    let acc = core.accel_schedule(&f, LO, HI, 0.25, 13);
    let leg = SurgeLimiter::new(Spool::Lp, 0.7500);
    let end = R + 1.0;

    // Python's seven kwarg dicts, in Python's order.
    let cases: [FuelLimiters; 7] = [
        FuelLimiters::default(),
        FuelLimiters { tt4_max: Some(REDLINE), ..Default::default() },
        FuelLimiters { tt4_max: Some(REDLINE), tau_gov: Some(0.2), ..Default::default() },
        FuelLimiters { accel: Some(&acc), ..Default::default() },
        FuelLimiters { surge: Some(leg), ..Default::default() },
        FuelLimiters { accel: Some(&acc), surge: Some(leg), ..Default::default() },
        FuelLimiters { tt4_max: Some(REDLINE), tau_gov: Some(0.2), accel: Some(&acc),
                       surge: Some(leg), ..Default::default() },
    ];
    for lim in &cases {
        let a = core.integrate_fuel(&f, &sched, nu0, end, DS, lim);
        let b = core.integrate_fuel(&f, &sched, nu0, end, DS,
                                    &FuelLimiters { s_off: None, ..*lim });
        same(&a, &b);
    }
    // ... and the gate is not vacuous: the armed leg genuinely clips.
    let armed = core.integrate_fuel(&f, &sched, nu0, end, DS,
                                    &FuelLimiters { surge: Some(leg), ..Default::default() });
    assert!(armed.iter().any(|p| p.mf < p.mf_sched), "the rung-49 leg must genuinely clip");
}

// ============================================================================= contract 1b
/// CONTRACT 1b — the rung-50 FINDING method at `s_off = None` IS rung 49's finding method: the
/// same two marches, the same reference-free surge object, bit-for-bit.
///
/// **TEN FIELDS ACROSS TWO STRUCTS.** Python's two dicts share their key names, so it loops a
/// tuple of ten strings; Rust's records are separate types (§ 5.18 finding 5), so the comparison
/// is spelled field by field — Python's ten, not "every common field", which would silently widen
/// the gate.
#[test]
fn contract1b_release_relief_none_is_rung49s_reader() {
    let f = flight();
    let d = design(cpg_gas());
    let t = ft(&d, 1.0);
    let c = t.core();
    let leg = SurgeLimiter::new(Spool::Lp, PHI_LIM);
    let a = c.release_relief(&f, LO, HI, None, Some(&leg), None, R, SETTLE, DS, None);
    let b = c.surge_relief(&f, LO, HI, &leg, R, SETTLE, DS, None, None, None);

    let pairs: [(&str, f64, f64); 10] = [
        ("s_eng", a.s_eng, b.s_eng),
        ("s_rel", a.s_rel, b.s_rel),
        ("relief_lp", a.relief_lp, b.relief_lp),
        ("relief_hp", a.relief_hp, b.relief_hp),
        ("fuel_removed", a.fuel_removed, b.fuel_removed),
        ("nu_hp_end", a.nu_hp_end, b.nu_hp_end),
        ("min_phi_lp_bare", a.min_phi_lp_bare, b.min_phi_lp_bare),
        ("min_phi_hp_bare", a.min_phi_hp_bare, b.min_phi_hp_bare),
        ("min_phi_lp_lim", a.min_phi_lp_lim, b.min_phi_lp_lim),
        ("min_phi_hp_lim", a.min_phi_hp_lim, b.min_phi_hp_lim),
    ];
    for (k, x, y) in pairs {
        assert_eq!(x.to_bits(), y.to_bits(), "{k}: {x} vs {y}");
    }
}

// ============================================================================ contract 2/3
/// CONTRACT 2/3 — forcing a release the leg would have made anyway is INERT (bit-for-bit the
/// unforced leg); forcing one BEFORE the leg ever engages leaves the march bit-for-bit BARE. The
/// two ends of the sweep are exact, not approximate.
///
/// The natural window at this floor is `[0.12, 0.44]`.
#[test]
fn contract2_3_late_s_off_inert_early_s_off_bare() {
    let f = flight();
    let d = design(cpg_gas());
    let t = ft(&d, 1.0);
    let core = t.core();
    let (sched, nu0) = ramp(core, LO, HI, R);
    let leg = SurgeLimiter::new(Spool::Lp, PHI_LIM);
    let end = R + SETTLE;

    let bare = core.integrate_fuel(&f, &sched, nu0, end, DS, &FuelLimiters::default());
    let free = core.integrate_fuel(&f, &sched, nu0, end, DS,
                                   &FuelLimiters { surge: Some(leg), ..Default::default() });
    let late = core.integrate_fuel(
        &f, &sched, nu0, end, DS,
        &FuelLimiters { surge: Some(leg), s_off: Some(1.50), ..Default::default() });
    let early = core.integrate_fuel(
        &f, &sched, nu0, end, DS,
        &FuelLimiters { surge: Some(leg), s_off: Some(0.10), ..Default::default() });
    same(&late, &free);
    same(&early, &bare);
    assert!(free.iter().any(|p| p.mf < p.mf_sched), "not vacuous: the free leg must clip");
}

// ============================================================================== contract 4
/// CONTRACT 4 — `s_off` forces a min-select LEG to release; with none armed it is meaningless, and
/// the rung-46/47 governor is deliberately out of scope.
///
/// **THE THREE REFUSALS ARE ASSERTED APART, WHERE PYTHON MATCHES NOTHING.** Its three
/// `pytest.raises(AssertionError)` blocks cannot say which assert fired. The first two are the
/// marcher's composition assert (a redline is NOT an armed min-select leg — that is the point of
/// the second) and the third is `release_relief`'s own, a different function with a different
/// message.
#[test]
fn contract4_s_off_without_an_armed_leg_refuses() {
    let f = flight();
    let d = design(cpg_gas());
    let t = ft(&d, 1.0);
    let core = t.core();
    let (sched, nu0) = ramp(core, LO, HI, R);

    for lim in [FuelLimiters { s_off: Some(0.30), ..Default::default() },
                FuelLimiters { tt4_max: Some(REDLINE), s_off: Some(0.30), ..Default::default() }]
    {
        let m = refusal(|| {
            core.integrate_fuel(&f, &sched, nu0, R + 0.5, DS, &lim);
        })
        .expect("s_off with no armed min-select leg must refuse");
        assert!(m.starts_with("rung-50 s_off forces a min-select LEG to release early"),
                "the marcher's composition assert must fire, got: {m}");
    }

    let m = refusal(|| {
        core.release_relief(&f, LO, HI, Some(0.30), None, None, R, SETTLE, DS, None);
    })
    .expect("release_relief with no leg must refuse");
    assert_eq!(m, "rung-50 release_relief needs a leg to release: pass surge= and/or accel=.");
}

// ============================================================================== contract 5
/// CONTRACT 5 — the finding is inherently two-shaft (BOTH spools' minima relocate), so
/// `lp_disabled` is not a reduce axis for it.
///
/// **AND THIS GATE IS § 5.18 P2's INSTRUMENT.** Python asserts
/// `match="inherently two-shaft"` — a substring four rungs' `lp_disabled` gates all satisfy. It
/// arms `surge=` AND `s_off=`, and the `surge` refusal precedes the `s_off` one inside the
/// degenerate block, so the assert that actually fires is **rung 49's**, not rung 50's. § 5.18
/// finding 1 measured rung 50's own refusal to be unreachable over all **255** arming
/// combinations. The full message is asserted here so that is a measurement rather than an
/// inference.
#[test]
fn contract5_lp_disabled_refuses() {
    let f = flight();
    let se: Engine = build_turbojet(cpg_gas(), 10.0, TT4, 50_000.0, single_matchable());
    let deg = TwoSpoolFuelTransient::lp_disabled(se, flight(), 1.0, hp_shaped());
    let leg = SurgeLimiter::new(Spool::Lp, 0.75);

    let m = refusal(|| {
        deg.integrate_fuel_lp_disabled(
            &f, |_s| 0.5, 1.0, R + 0.5, DS,
            &FuelLimiters { surge: Some(leg), s_off: Some(0.30), ..Default::default() });
    })
    .expect("the rung-50 forced release on an lp_disabled object must refuse");
    assert!(m.contains("inherently two-shaft"), "the refusal must name the reason: {m}");
    // P2: the assert that fires is named for RUNG 49, not rung 50.
    assert_eq!(
        m,
        "the rung-49 phi floor is inherently two-shaft (its finding is the CREDIT on the \
         watched spool against the DEBIT on the other); lp_disabled is not a reduce axis \
         for a split BETWEEN spools.");
    assert!(!m.contains("rung-50"),
            "rung 50's own lp_disabled refusal is UNREACHABLE — § 5.18 finding 1: {m}");
}

// ============================================================================== contract 6
/// CONTRACT 6 — the rung-50 diagnostic is a separate entry point: the design-point run is
/// bit-for-bit rung 6 across it.
#[test]
fn contract6_cycle_untouched_bit_for_bit_rung6() {
    let f = flight();
    let eng = build_turbojet(Gas::reacting_equilibrium(), 10.0, TT4, 50_000.0, single());
    let a = eng.run(&f, 1.0);

    let d = design(cpg_gas());
    let t = ft(&d, 1.0);
    let leg = SurgeLimiter::new(Spool::Lp, PHI_LIM);
    let _ = t.core().release_relief(&f, LO, HI, Some(0.30), Some(&leg), None, R, SETTLE, DS, None);

    let b = eng.run(&f, 1.0);
    assert_eq!(a.performance.specific_thrust.to_bits(), b.performance.specific_thrust.to_bits());
    assert_eq!(a.station("4").far.to_bits(), b.station("4").far.to_bits());
}

// ================================================================================== gate 3
/// GATE 3 — THE HEADLINE. Whenever the DIVE BRANCH WINS on a spool, that spool's argmin `phi` sits
/// AT the release point — for the WATCHED spool and the UNWATCHED one alike. Rung 49 saw only half
/// of this: it measured the UNWATCHED minimum landing just after `s_rel`, and the watched one was
/// invisible because an LP floor's natural release always lands past the LP basin.
///
/// "The dive branch wins" is the conjunction of two measurable preconditions, and BOTH bite in
/// this sweep — which is what makes the gate a two-branch LAW and not a slogan:
///
/// * (a) the release lands at or after that spool's OWN bare minimum. Released upstream of it the
///   re-opened dive merges into the still-ongoing bare descent and bottoms in the bare basin
///   instead;
/// * (b) the dive actually beats rung 48's truncation branch — that spool's relief is NEGATIVE.
///   Where the CREDIT branch wins the minimum sits back at the arrest.
///
/// The anchor is `s_rel` (the last engaged point), NOT `s_off`: past the natural release the
/// forcing is inert and the leg lets go on its own.
#[test]
fn gate3_the_release_edge_relocates_both_minima_to_itself() {
    let rows = r2_sweep();
    let tol = 3.0 * DS;
    let mut hits = 0;
    for x in rows {
        for (name, val, star, rel) in [("s_min_lp", x.s_min_lp, S_LP_STAR_2, x.relief_lp),
                                       ("s_min_hp", x.s_min_hp, S_HP_STAR_2, x.relief_hp)]
        {
            if x.s_rel >= star && rel < 0.0 {
                assert!((val - x.s_rel).abs() <= tol,
                        "s_off {:?} {name} {} vs s_rel {} — must relocate TO the release point",
                        x.s_off, val, x.s_rel);
                hits += 1;
            }
        }
    }
    assert!(hits >= 8, "the gate must not be vacuous: {hits}");
    // precondition (a) bites — released upstream of s_hp*, the HP keeps its bare basin
    let early = &rows[0];
    assert!(early.s_rel < S_HP_STAR_2 && early.relief_hp < 0.0
            && early.s_min_hp > early.s_rel + tol,
            "precondition (a) must bite: {early:?}");
    // precondition (b) bites — where the CREDIT branch wins, the minimum is NOT at the release
    let late = &rows[rows.len() - 1];
    assert!(late.relief_lp > 0.0 && late.s_min_lp < late.s_rel - tol,
            "precondition (b) must bite: {late:?}");
}

// ================================================================================== gate 4
/// GATE 4 — THE DISCRIMINATOR, on an axis that moves ONLY the release edge. Rung 49 § 3 measured
/// this ordering by sweeping `phi_lim`, which drags `s_eng` and the clip depth along, and hedged it
/// as WITHIN-FAMILY. Here `s_eng` is IDENTICAL in every row, so the hedge LIFTS.
///
/// The debit deepens monotonically as the release walks THROUGH the unwatched spool's own minimum
/// without noticing it, and peaks with the release just inside the RAMP END.
#[test]
fn gate4_the_debit_is_ramp_clocked_deconfounded() {
    let rows = r2_sweep();
    // Python's `len({round(x["s_eng"], 6)}) == 1`.
    let mut engs: Vec<i64> = rows.iter().map(|x| (x.s_eng * 1e6).round() as i64).collect();
    engs.dedup();
    assert_eq!(engs.len(), 1,
               "the engagement edge must be FIXED — that is the whole point of the axis: {engs:?}");

    // monotone deepening straight THROUGH s_hp* = 0.640 without noticing it
    let upto: Vec<&ReleaseRelief> =
        rows.iter().filter(|x| x.s_off.expect("forced") <= 1.10).collect();
    let mags: Vec<f64> = upto.iter().map(|x| -x.relief_hp).collect();
    assert!(mags.windows(2).all(|w| w[1] > w[0]), "must deepen monotonically: {mags:?}");
    assert!(upto.iter().any(|x| x.s_off.expect("forced") < S_HP_STAR_2)
            && upto.iter().any(|x| x.s_off.expect("forced") > S_HP_STAR_2),
            "must BRACKET s_hp*");

    let at_star = rows.iter()
        .min_by(|a, b| (a.s_off.expect("forced") - S_HP_STAR_2).abs()
                .total_cmp(&(b.s_off.expect("forced") - S_HP_STAR_2).abs()))
        .expect("non-empty");
    let peak = rows.iter().max_by(|a, b| (-a.relief_hp).total_cmp(&-b.relief_hp))
        .expect("non-empty");
    assert!(-peak.relief_hp > 2.5 * -at_star.relief_hp,
            "at s_hp* {} vs peak {}", -at_star.relief_hp, -peak.relief_hp);
    // the peak sits near the RAMP END, not at the unwatched spool's own minimum
    let ps = peak.s_off.expect("forced");
    assert!(ps > 2.0 * S_HP_STAR_2, "the peak is NOT at s_hp*: {ps}");
    assert!((0.6 * 2.0..=2.0).contains(&ps), "the peak is near the RAMP END: {ps}");
    // ... and it collapses once the release goes past the ramp end
    let past: Vec<&ReleaseRelief> =
        rows.iter().filter(|x| x.s_off.expect("forced") > 2.0).collect();
    assert!(!past.is_empty());
    assert!(-past[past.len() - 1].relief_hp < 0.6 * -peak.relief_hp,
            "must collapse past the ramp end: {} vs peak {}",
            -past[past.len() - 1].relief_hp, -peak.relief_hp);
}

// ================================================================================== gate 5
/// GATE 5 — Rung 49's gate 3 asserts `relief_watched == phi_lim − min phi_bare` identically and
/// calls it definitional. It is — UNDER THE UNFORCED INSTRUMENT. Force the release early and it
/// fails in the only direction that matters: the limiter leaves the spool it is PROTECTING worse
/// off than no limiter at all.
///
/// Rung 49 is BOUNDED, not corrected: as `s_off` runs past the natural release the identity comes
/// straight back.
///
/// **TWO OF THIS GATE'S SEVEN CELLS SIT ON THE ONE-CELL KNIFE EDGE** — `s_off = 0.20` and `0.26`
/// accumulate to just BELOW their own bar, so the leg stays armed one point longer than a
/// `k * ds` spelling would keep it. See the header.
#[test]
fn gate5_the_watched_spool_is_debited_when_released_early() {
    let rows = sweep(&[0.16, 0.20, 0.26, 0.30, 0.36, 0.44, 0.60], Some(PHI_LIM), None, R, SETTLE,
                     DS, 1.0);
    assert!(rows.iter().any(|x| x.relief_lp < 0.0),
            "an early release must DEBIT the watched spool: {:?}",
            rows.iter().map(|x| (x.s_off, x.relief_lp)).collect::<Vec<_>>());
    let worst = rows.iter().min_by(|a, b| a.relief_lp.total_cmp(&b.relief_lp)).expect("non-empty");
    assert!(worst.s_off.expect("forced") < S_HP_STAR,
            "the damage is done EARLY: {:?}", worst.s_off);
    // rung 49 recovered at the far end (its unforced instrument)
    let free = &rows[rows.len() - 1];
    assert!(free.relief_lp > 0.0, "{free:?}");
    assert!((free.relief_lp - (PHI_LIM - free.min_phi_lp_bare)).abs() < 1e-5, "{free:?}");
}

// ================================================================================== gate 6
/// GATE 6 — THE SEAM CLOSES. Rung 49's standing OPEN seam: *"WHY rung 48's leg is immune to the
/// release debit is an OPEN SEAM … the clip SHAPE is the obvious suspect, but it is NOT measured
/// here."*
///
/// Measured: rung 48's OWN leg, clip shape unchanged, forced to release inside the ramp DEBITS
/// BOTH spools — with the same relocation signature as the `phi` floor. Left alone (natural
/// release post-ramp) it delivers its rung-48 CREDIT. The immunity is TIMING.
///
/// This is an accel-only cell, so `spool` / `phi_lim` / `relief_watched` / `relief_other` are all
/// `None` — one of the two arms § 5.18 finding 4 measured live.
#[test]
fn gate6_seam_rung48s_immunity_is_timing_not_clip_shape() {
    let rows = sweep(&[0.30, 0.44, 0.50, 9.90], None, Some(0.25), R, SETTLE, DS, 1.0);
    let free = &rows[rows.len() - 1];
    assert!(free.s_rel > R, "rung 48's natural release must be POST-ramp: {}", free.s_rel);
    assert!(free.relief_lp > 0.0 && free.relief_hp > 0.0,
            "unforced, rung 48's leg CREDITS both spools: {free:?}");
    for x in rows.iter().filter(|x| x.s_off.expect("forced") < R + 1e-9) {
        assert!(x.relief_lp < 0.0 && x.relief_hp < 0.0,
                "forced inside the ramp, the SAME leg debits BOTH spools: s_off {:?}, {} / {}",
                x.s_off, x.relief_lp, x.relief_hp);
    }
}

// ================================================================================== gate 7
/// GATE 7 — the seam closure OUT of rung 49's own `s_hp*`-vs-`r` confound (at `r = 0.5` those sit
/// 2.5 cells apart). At `r = 2.0`, `m = 0.15` (the corrected band floor — `m = 0.25` never engages
/// on so slow a ramp) the same inversion holds.
///
/// And rung 48's EXACT ZERO survives the forcing untouched: `s_eng = 0.360` is downstream of
/// `s_lp* = 0.320`, and every release here lands past the LP basin, so `relief_lp` is exactly
/// `0.0` in every row.
///
/// **THAT ZERO IS AN `x − x` STRUCTURAL ZERO, NOT A TOLERANCE.** For a clip downstream of `s_lp*`
/// the bare and limited marches are bit-identical through the LP argmin, so the two `min` calls
/// read the SAME float. If it misses, the diagnosis is the march coordinate or `first_raw_min`'s
/// strict `<`, never physics — rung 48's six sites and rung 49's gate 9 have the same mechanism.
#[test]
fn gate7_seam_cross_regime_and_rung48s_exact_zero_survives() {
    let rows = sweep(&[0.66, 1.10, 1.80, 9.90], None, Some(0.15), 2.0, SETTLE, DS, 1.0);
    for x in &rows {
        assert_eq!(x.relief_lp.to_bits(), 0.0f64.to_bits(),
                   "rung 48's exact zero must SURVIVE at s_off {:?}: {}", x.s_off, x.relief_lp);
    }
    for x in rows.iter().filter(|x| x.s_off.expect("forced") <= 1.10) {
        assert!(x.relief_hp < 0.0, "forced inside the ramp must debit the HP: {x:?}");
    }
    assert!(rows[rows.len() - 1].relief_hp > 0.0,
            "unforced => rung 48's credit: {:?}", rows[rows.len() - 1]);
}

// ================================================================================== gate 8
/// GATE 8 — Rung 49 § 4 refuted hand-back MAGNITUDE as the explanation, measuring it
/// ANTI-correlated — but it swept magnitude and timing TOGETHER. Hold the release time fixed and
/// the sign reverses: the debit is MONOTONE INCREASING in the deficit, and it is monotone ACROSS
/// INSTRUMENT FAMILIES (two `phi` floors + rung 48's schedule, all released at the same `s_rel`).
/// Rung 48's clip is not gentler per unit deficit — it is WORSE.
///
/// Claimed as MONOTONE only: the functional form is measured, not derived.
///
/// **AND THE GATE READS `fuel_removed`, NOT `deficit_at_release`** — the key that carries the
/// rung's own named quantity has no reader here or anywhere else in the project. Copied as Python
/// spells it; the missing coverage is step 5's oracle's.
#[test]
fn gate8_the_deficit_factor_at_a_fixed_release() {
    let f = flight();
    let d = design(cpg_gas());
    let t = ft(&d, 1.0);
    let c = t.core();
    let acc = c.accel_schedule(&f, LO, HI, 0.25, 13);
    let l1 = SurgeLimiter::new(Spool::Lp, 0.7450);
    let l2 = SurgeLimiter::new(Spool::Lp, 0.7500);
    let out: Vec<ReleaseRelief> = [(Some(&l1), None), (Some(&l2), None), (None, Some(&acc))]
        .into_iter()
        .map(|(leg, a)| c.release_relief(&f, LO, HI, Some(0.44), leg, a, R, SETTLE, DS, None))
        .collect();

    // the release time is genuinely MATCHED across the three (that is the deconfounding)
    let mut rels: Vec<i64> = out.iter().map(|x| (x.s_rel * 1e6).round() as i64).collect();
    rels.dedup();
    assert_eq!(rels.len(), 1, "the release time must be MATCHED: {rels:?}");

    let rm: Vec<f64> = out.iter().map(|x| x.fuel_removed).collect();
    let db: Vec<f64> = out.iter().map(|x| -x.relief_hp).collect();
    assert!(rm.windows(2).all(|w| w[1] > w[0]), "deficits must be genuinely ordered: {rm:?}");
    assert!(db.windows(2).all(|w| w[1] > w[0]),
            "debit must ORDER WITH the deficit: {rm:?} / {db:?}");
}

// ================================================================================== gate 9
/// GATE 9 — the deflation to exclude is *"any clip removes fuel and slows the accel"*. Two
/// measured exclusions: fuel removal is MONOTONE in `s_off` while the debit is PEAKED (the largest
/// removal is NOT the largest debit), and the endpoint is unmoved at rung 49's gate-10 settle.
///
/// **`settle = 4.0` HERE, at the same offsets and floor as gates 3/4's `2.0`** — a different memo
/// key and a different march length. See the header.
#[test]
fn gate9_not_rung_44s_ramp_rate_lever() {
    let rows = sweep(&R2_OFFS, Some(PHI_LIM_2), None, 2.0, 4.0, DS, 1.0);
    let rm: Vec<f64> = rows.iter().map(|x| x.fuel_removed).collect();
    assert!(rm.windows(2).all(|w| w[1] > w[0]), "fuel_removed must be monotone: {rm:?}");
    let peak = rows.iter().max_by(|a, b| (-a.relief_hp).total_cmp(&-b.relief_hp))
        .expect("non-empty");
    let last = &rows[rows.len() - 1];
    assert!(last.fuel_removed > peak.fuel_removed,
            "{} vs peak {}", last.fuel_removed, peak.fuel_removed);
    assert!(-last.relief_hp < 0.6 * -peak.relief_hp,
            "MORE fuel removed must give a SMALLER debit: {} vs peak {}",
            -last.relief_hp, -peak.relief_hp);
    for x in &rows {
        assert!((x.nu_hp_end - x.nu_hp_end_bare).abs() < 5e-4,
                "s_off {:?}: the endpoint must be UNMOVED, {} vs {}",
                x.s_off, x.nu_hp_end, x.nu_hp_end_bare);
    }
}

// ================================================================================= gate 10
/// GATE 10 — the headline is a statement about WHERE a minimum sits, so it is the most grid-prone
/// claim in the set. Measured at `ds ∈ {0.02, 0.01}` with `s_off` ON the grid: the relocation
/// offset is `0.000` at BOTH, and the depth converges to a few per cent — far tighter than rung
/// 49's ~13 % gate-12 drift, because a FORCED dive is anchored to an imposed `s_off` rather than to
/// a solved edge.
///
/// Checked at BOTH ramp rates, deliberately including `r = 2.0` — those dives are ~8× deeper than
/// the `r = 0.5` ones and are where a grid artifact would most plausibly hide. They converge
/// BETTER, not worse.
///
/// **THE `(1.10, 1.56)` PAIR IS RE-MARCHED, NOT SLICED OUT OF [`r2_sweep`].** Python keys its memo
/// on the offset tuple, so it recomputes; slicing would give the same numbers here and stop being
/// a port. See the header.
#[test]
fn gate10_ds_convergence_of_the_relocation_and_the_debit() {
    for (offs, phi, r_, tol) in [(&[0.30, 0.40, 0.44][..], PHI_LIM, R, 0.05),
                                 (&[1.10, 1.56][..], PHI_LIM_2, 2.0, 0.02)]
    {
        let a = sweep(offs, Some(phi), None, r_, SETTLE, 0.02, 1.0);
        let b = sweep(offs, Some(phi), None, r_, SETTLE, 0.01, 1.0);
        for (x, y) in a.iter().zip(b.iter()) {
            let (sx, sy) = (x.s_off.expect("forced"), y.s_off.expect("forced"));
            assert!((x.s_min_lp - sx).abs() <= 0.02 && (y.s_min_lp - sy).abs() <= 0.01,
                    "r {r_} s_off {sx}: {} / {}", x.s_min_lp, y.s_min_lp);
            assert!((y.relief_hp - x.relief_hp).abs() < tol * x.relief_hp.abs(),
                    "r {r_} s_off {sx}: {} vs {}", x.relief_hp, y.relief_hp);
        }
    }
}

// ================================================================================ gate 10b
/// GATE 10b — `rho = tau_L / tau_H` is rung 40's one parameter. The early-release debit on the
/// WATCHED spool — the new sign — survives it.
///
/// **BOTH CELLS INCLUDE `s_off = 0.26`, one of the two one-cell knife edges.** See the header.
#[test]
fn gate10b_the_split_survives_rho() {
    for rho in [0.25, 4.0] {
        let rows = sweep(&[0.26, 0.30, 0.36], Some(PHI_LIM), None, R, SETTLE, DS, rho);
        assert!(rows.iter().any(|x| x.relief_lp < 0.0),
                "rho {rho}: {:?}",
                rows.iter().map(|x| (x.s_off, x.relief_lp)).collect::<Vec<_>>());
    }
}
