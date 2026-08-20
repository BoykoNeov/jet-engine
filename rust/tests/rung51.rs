//! RUNG 51 — THE RELEASE RATE: the debit is not a functional of the applied-fuel trajectory.
//!
//! Port of `tests/test_rung51.py`, gate for gate. That file defines **16 test functions** and
//! collects **16 items** — no `parametrize`, no `slow` mark.
//!
//! | # | `tests/test_rung51.py` | here |
//! |---|---|---|
//! | 1 | `test_reduce_tau_rel_none_and_zero_are_bit_for_bit_rung50` | [`contract1_tau_rel_none_and_zero_are_bit_for_bit_rung50`] |
//! | 2 | `test_reduce_release_relief_tau_none_is_rung50_bit_for_bit` | [`contract1b_release_relief_tau_none_is_rung50`] |
//! | 3 | `test_reduce_tau_rel_without_s_off_asserts` | [`contract2_tau_rel_without_s_off_refuses`] |
//! | 4 | `test_reduce_lp_disabled_asserts` | [`contract3_lp_disabled_refuses`] |
//! | 5 | `test_reduce_s_off_past_the_natural_release_makes_tau_rel_inert` | [`contract4_s_off_past_the_natural_release_makes_tau_rel_inert`] |
//! | 6 | `test_cycle_untouched_by_the_release_rate_bit_for_bit_rung6` | [`contract5_cycle_untouched_bit_for_bit_rung6`] |
//! | 7 | `test_headline_the_faded_release_lands_OUTSIDE_its_own_bracket` | [`gate3_the_faded_release_lands_outside_its_own_bracket`] |
//! | 8 | `test_headline_the_POINTWISE_applied_fuel_sandwich` | [`gate4_the_pointwise_applied_fuel_sandwich`] |
//! | 9 | `test_SCOPE_the_shallow_regime_INTERPOLATES_a_negative_gate` | [`gate5_scope_the_shallow_regime_interpolates`] |
//! | 10 | `test_cross_family_the_violation_flips_the_SIGN_and_rung48s_exact_zero_survives` | [`gate6_cross_family_the_violation_flips_the_sign`] |
//! | 11 | `test_the_naturally_occurring_MATCHED_DEFICIT_pair` | [`gate7_the_naturally_occurring_matched_deficit_pair`] |
//! | 12 | `test_location_the_minimum_tracks_the_COMPLETION_point_then_DETACHES` | [`gate8_the_minimum_tracks_the_completion_point_then_detaches`] |
//! | 13 | `test_rung50s_precondition_a_is_MIS_STATED` | [`gate9_rung50s_precondition_a_is_mis_stated`] |
//! | 14 | `test_not_the_ramp_rate_lever_the_non_tautology` | [`gate10_not_rung_44s_ramp_rate_lever`] |
//! | 15 | `test_robustness_ds_convergence` | [`gate11_ds_convergence`] |
//! | 16 | `test_robustness_the_bracket_violation_survives_rho` | [`gate12_the_bracket_violation_survives_rho`] |
//!
//! # THE GRID IS **NOT** RUNG 50's, AND THE TRAP IS THE SETTLE TIME
//!
//! `SETTLE` is **4.0** here against `test_rung50.py`'s **2.0** — the very pair § 5.18's own probe
//! got wrong on its first run, where it used 2.0 for all four rung files and had to be re-run.
//! The numbers happened to be settle-invariant there, which is exactly why the mistake would not
//! have announced itself. It is written out again rather than shared.
//!
//! **AND `_rel`'s DEFAULTS ARE THE DEEP-DIVE CELL, NOT THE `r = 0.5` ONE.** Python's helper
//! defaults to `phi_lim = PHI_LIM_2` (`0.7725`) and `r = R2` (`2.0`), so a gate that "says
//! nothing about the ramp rate" is running at `r = 2.0`. Rung 50's `_sweep` defaults the other
//! way. Every call below spells its cell out.
//!
//! # § 5.18 P6 — RUNG 51 ADDS NO NEW PLANT OR READER LOGIC, AND THIS FILE IS THE CHECK
//!
//! P6 was registered before rung 50's step was written: `release_relief` would land COMPLETE
//! with `tau_rel` at rung 50, because `tau_rel` is its kwarg and not a separate path, so rung
//! 51's two readers are LOOPS over it. Discharged: step 3 adds
//! [`rate_sweep`](turbojet::fuel_transient::FuelTransientCore::rate_sweep) and
//! [`deficit_curve`](turbojet::fuel_transient::FuelTransientCore::deficit_curve), whose bodies are
//! a `map` each, plus one assert. No new field, no new branch, no new constant.
//!
//! `deficit_curve` has **no consumer in `test_rung51.py` at all** — Python ships it and the
//! suite never calls it (its own docstring says the two-sided bracket replaced it). Ported
//! because the source ships it; its only gate is the step-5 oracle. *COPY vs REDERIVATION.*
//!
//! # Where Rust has to say out loud what Python leaves implicit
//!
//! * **`_ROWS` becomes a `Mutex<HashMap>` keyed on BITS.** Python memoises on the tuple
//!   `(s_off, tau_rel, phi_lim, margin, r, rho, ds)` and several gates share rows, each of which
//!   is a PAIR of full marches. Floats are not `Hash` in Rust, so the key is `to_bits()` with
//!   `Option` carried through — never a rounded or formatted key, which would merge two cells
//!   that Python keeps apart.
//! * **GATE 4's `round(s, 3)` DICT BECOMES AN INDEX ALIGNMENT.** Python keys three marches by
//!   `round(p["s"], 3)` and intersects. § 5.18 finding 5b measured that all three marchers
//!   produce the SAME `s` sequence from the same `0.0` (301 points at `s_end = 6.0`), so the
//!   lookup is an index — and this gate asserts the `s` BITS agree before it compares, rather
//!   than rebuilding a float-keyed map that could silently drop a point.
//! * **CONTRACT 3 asserts the FULL refusal message**, as `rung50.rs` does — § 5.18 P2's
//!   instrument at its THIRD of four data points. Python matches `"inherently two-shaft"`; the
//!   assert that actually fires is named for **rung 49**, because arming `surge=` reaches that
//!   refusal before the block ever looks at `s_off` or `tau_rel`.
//! * **CONTRACT 2's refusal is rung 51's OWN**, and it is reachable — unlike its `lp_disabled`
//!   one. `tau_rel` without `s_off` is refused by the composition assert at the top of
//!   `integrate_fuel`, which no earlier refusal precedes.
//!
//! # `#[ignore]`
//!
//! None. `test_rung51.py` carries no `slow` mark.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

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
/// **4.0 — NOT `test_rung50.py`'s 2.0.** See the header.
const SETTLE: f64 = 4.0;
const DS: f64 = 0.02;
const R: f64 = 0.5;
const R2: f64 = 2.0;
const REDLINE: f64 = 1480.0;

/// The `r = 0.5` working floor — its natural `s_rel` is `0.440`.
const PHI_LIM: f64 = 0.7450;
/// The `r = 2.0` floor — its natural `s_rel` is `2.100`. **This is `_rel`'s DEFAULT.**
const PHI_LIM_2: f64 = 0.7725;
/// `r = 2.0` bare minima, re-measured in gate 9 rather than trusted.
const S_LP_STAR_2: f64 = 0.320;
const S_HP_STAR_2: f64 = 0.640;

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

fn single() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.90, e_c: None, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, e_t: None,
        eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: false,
    }
}

/// [`single`] plus the one constant contract 3 needs to have a degenerate object at all.
fn single_matchable() -> Losses {
    Losses { nozzle_convergent: true, ..single() }
}

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

/// `test_rung51.py`'s `_ft` — ONE map pair, like rung 50's and unlike rungs 46–49's `SHAPES`.
fn ft(rho: f64) -> TwoSpoolFuelTransient {
    TwoSpoolFuelTransient::new(design(cpg_gas()), flight(), 1.0, lp_shaped(), hp_shaped(), rho)
}

/// Python's `_ramp` — `min(1.0, s/r)`, NOT the marcher's branch form. Deliberately not unified.
fn ramp(core: &FuelTransientCore, r: f64) -> (impl Fn(f64) -> f64, (f64, f64)) {
    let f = flight();
    let mf0 = core.fuel_for_tt4(&f, LO);
    let mf1 = core.fuel_for_tt4(&f, HI);
    let eq0 = core.inner.equilibrium(&f, LO);
    (move |s: f64| mf0 + (mf1 - mf0) * (s / r).min(1.0), (eq0.nu_lp, eq0.nu_hp))
}

/// A NAMED FIELD ACCESSOR. Python compares its two dicts by looping a tuple of key
/// STRINGS; Rust's records are separate types, so the same loop needs the name and the
/// getter together. The alias exists because `clippy::type_complexity` asks for it — the
/// shape is the point, not the spelling.
type Field = (&'static str, fn(&ReleaseRelief) -> f64);

fn keys7(p: &FuelPoint) -> [u64; 7] {
    [p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.phi_lp.to_bits(), p.phi_hp.to_bits(),
     p.tt4.to_bits(), p.f.to_bits(), p.mf.to_bits()]
}

fn same(a: &[FuelPoint], b: &[FuelPoint]) {
    assert_eq!(a.len(), b.len(), "trajectory lengths differ: {} vs {}", a.len(), b.len());
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(keys7(x), keys7(y), "trajectories diverge at s={} / {}", x.s, y.s);
    }
}

/// Python's `_ROWS` key — bits, with `Option` carried through so `None` and `0.0` stay APART
/// (contract 1 turns on their being the same MARCH through a different BRANCH argument).
type RelKey = (Option<u64>, Option<u64>, Option<u64>, Option<u64>, u64, u64, u64);

#[allow(clippy::type_complexity)]
fn rows_memo() -> &'static Mutex<HashMap<RelKey, ReleaseRelief>> {
    static M: OnceLock<Mutex<HashMap<RelKey, ReleaseRelief>>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Python's `_rel`, memo and all. **The defaults are the DEEP-DIVE cell** — `phi_lim = PHI_LIM_2`
/// and `r = R2` — so every caller below passes its cell explicitly rather than leaning on them.
fn rel(s_off: Option<f64>, tau_rel: Option<f64>, phi_lim: Option<f64>, margin: Option<f64>,
       r: f64, rho: f64, ds: f64) -> ReleaseRelief
{
    let key: RelKey = (s_off.map(f64::to_bits), tau_rel.map(f64::to_bits),
                       phi_lim.map(f64::to_bits), margin.map(f64::to_bits),
                       r.to_bits(), rho.to_bits(), ds.to_bits());
    if let Some(hit) = rows_memo().lock().expect("memo poisoned").get(&key) {
        return *hit;
    }
    let t = ft(rho);
    let c = t.core();
    let leg = phi_lim.map(|p| SurgeLimiter::new(Spool::Lp, p));
    let acc: Option<AccelSchedule> = margin.map(|m| c.accel_schedule(&flight(), LO, HI, m, 13));
    let out = c.release_relief(&flight(), LO, HI, s_off, leg.as_ref(), acc.as_ref(), r, SETTLE,
                               ds, tau_rel);
    rows_memo().lock().expect("memo poisoned").insert(key, out);
    out
}

/// The default cell: the `r = 2.0` deep dive on the `phi` floor.
fn deep(s_off: f64, tau_rel: Option<f64>) -> ReleaseRelief {
    rel(Some(s_off), tau_rel, Some(PHI_LIM_2), None, R2, 1.0, DS)
}

/// The rung-48 cross-family cell — `phi_lim = None`, `margin = 0.15`, still `r = 2.0`.
fn sched(s_off: f64, tau_rel: Option<f64>) -> ReleaseRelief {
    rel(Some(s_off), tau_rel, None, Some(0.15), R2, 1.0, DS)
}

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
/// CONTRACT 1 — `tau_rel = None` AND `tau_rel = 0.0` both reach the IDENTICAL branch of
/// [`release_weight`], which returns exactly `1.0` or `0.0`, so the rung-50 march is reproduced
/// bit-identically through the NEW signature. Rung 48 gate 2's lesson, applied to a fade whose
/// `w == 1` case returns the cap ITSELF rather than an arithmetic reconstruction of it.
///
/// [`release_weight`]: turbojet::fuel_transient::release_weight
#[test]
fn contract1_tau_rel_none_and_zero_are_bit_for_bit_rung50() {
    let f = flight();
    let t = ft(1.0);
    let core = t.core();
    let (sch, nu0) = ramp(core, R);
    let acc = core.accel_schedule(&f, LO, HI, 0.25, 13);
    let leg = SurgeLimiter::new(Spool::Lp, PHI_LIM);
    let end = R + 1.0;

    let forced: [FuelLimiters; 4] = [
        FuelLimiters { surge: Some(leg), s_off: Some(0.30), ..Default::default() },
        FuelLimiters { accel: Some(&acc), s_off: Some(0.30), ..Default::default() },
        FuelLimiters { accel: Some(&acc), surge: Some(leg), s_off: Some(0.30),
                       ..Default::default() },
        FuelLimiters { tt4_max: Some(REDLINE), tau_gov: Some(0.2), surge: Some(leg),
                       s_off: Some(0.30), ..Default::default() },
    ];
    for lim in &forced {
        let base = core.integrate_fuel(&f, &sch, nu0, end, DS, lim);
        for t_rel in [None, Some(0.0)] {
            same(&base, &core.integrate_fuel(&f, &sch, nu0, end, DS,
                                             &FuelLimiters { tau_rel: t_rel, ..*lim }));
        }
    }
    // ... and the rung-49/50 UNforced legs, through the new signature
    for lim in [FuelLimiters { surge: Some(leg), ..Default::default() },
                FuelLimiters { accel: Some(&acc), ..Default::default() }]
    {
        same(&core.integrate_fuel(&f, &sch, nu0, end, DS, &lim),
             &core.integrate_fuel(&f, &sch, nu0, end, DS,
                                  &FuelLimiters { tau_rel: None, ..lim }));
    }
}

// ============================================================================= contract 1b
/// CONTRACT 1b — the finding METHOD reduces too: `release_relief(tau_rel = None)` is bit-for-bit
/// rung 50's own call, and [`rate_sweep`]'s `None` row is that same record.
///
/// **THIS RUNS AT `r = 0.5` AND `PHI_LIM`**, not at `_rel`'s deep-dive defaults — Python builds
/// it by hand rather than through the memo helper. Copied as spelled.
///
/// [`rate_sweep`]: turbojet::fuel_transient::FuelTransientCore::rate_sweep
#[test]
fn contract1b_release_relief_tau_none_is_rung50() {
    let f = flight();
    let t = ft(1.0);
    let c = t.core();
    let leg = SurgeLimiter::new(Spool::Lp, PHI_LIM);
    let a = c.release_relief(&f, LO, HI, Some(0.30), Some(&leg), None, R, SETTLE, DS, None);
    let b = c.release_relief(&f, LO, HI, Some(0.30), Some(&leg), None, R, SETTLE, DS, None);
    let sweep = c.rate_sweep(&f, LO, HI, 0.30, &[None], Some(&leg), None, R, SETTLE, DS);
    let cc = sweep[0];

    let pick: [Field; 10] = [
        ("relief_lp", |x| x.relief_lp),
        ("relief_hp", |x| x.relief_hp),
        ("fuel_removed", |x| x.fuel_removed),
        ("min_phi_lp_lim", |x| x.min_phi_lp_lim),
        ("min_phi_hp_lim", |x| x.min_phi_hp_lim),
        ("s_min_lp", |x| x.s_min_lp),
        ("s_min_hp", |x| x.s_min_hp),
        ("s_eng", |x| x.s_eng),
        ("s_rel", |x| x.s_rel),
        ("nu_hp_end", |x| x.nu_hp_end),
    ];
    for (k, g) in pick {
        assert_eq!(g(&a).to_bits(), g(&b).to_bits(), "{k}: a vs b");
        assert_eq!(g(&a).to_bits(), g(&cc).to_bits(), "{k}: a vs rate_sweep's None row");
    }
}

// ============================================================================== contract 2
/// CONTRACT 2 — a rate needs a PINNED trigger. Without `s_off` the release edge moves WITH the
/// rate, which is the asymmetric LAG (rung 52) and a different instrument. Refused loudly.
///
/// **THIS ONE IS REACHABLE**, unlike rung 51's own `lp_disabled` refusal (§ 5.18 finding 1): it
/// sits in the composition block at the top of `integrate_fuel`, with nothing before it that a
/// `tau_rel`-without-`s_off` call trips first. The message is asserted in full.
#[test]
fn contract2_tau_rel_without_s_off_refuses() {
    let f = flight();
    let t = ft(1.0);
    let core = t.core();
    let (sch, nu0) = ramp(core, R);
    let leg = SurgeLimiter::new(Spool::Lp, PHI_LIM);
    let m = refusal(|| {
        core.integrate_fuel(&f, &sch, nu0, R + 1.0, DS,
                            &FuelLimiters { surge: Some(leg), tau_rel: Some(0.1),
                                            ..Default::default() });
    })
    .expect("tau_rel without s_off must refuse");
    assert!(m.starts_with("rung-51 tau_rel is the RATE of a FORCED release"),
            "the rung-51 composition assert must fire, got: {m}");
}

// ============================================================================== contract 3
/// CONTRACT 3 — inherited from rung 50: the finding is a split BETWEEN spools, so the
/// single-spool degeneracy is not a reduce axis for it.
///
/// **§ 5.18 P2's INSTRUMENT, THIRD OF FOUR DATA POINTS.** Python matches
/// `"inherently two-shaft"`; this gate arms `surge=`, `s_off=` AND `tau_rel=`, and the assert
/// that fires is named for **rung 49** — the `surge` refusal precedes both later ones inside the
/// degenerate block, so rung 51's own is unreachable over all 255 arming combinations.
#[test]
fn contract3_lp_disabled_refuses() {
    let f = flight();
    let se: Engine = build_turbojet(cpg_gas(), 10.0, TT4, 50_000.0, single_matchable());
    let deg = TwoSpoolFuelTransient::lp_disabled(se, flight(), 1.0, hp_shaped());
    let leg = SurgeLimiter::new(Spool::Lp, 0.75);

    let m = refusal(|| {
        deg.integrate_fuel_lp_disabled(
            &f, |_s| 0.5, 1.0, R + 0.5, DS,
            &FuelLimiters { surge: Some(leg), s_off: Some(0.30), tau_rel: Some(0.1),
                            ..Default::default() });
    })
    .expect("the rung-51 release rate on an lp_disabled object must refuse");
    assert!(m.contains("inherently two-shaft"), "the refusal must name the reason: {m}");
    assert_eq!(
        m,
        "the rung-49 phi floor is inherently two-shaft (its finding is the CREDIT on the \
         watched spool against the DEBIT on the other); lp_disabled is not a reduce axis \
         for a split BETWEEN spools.");
    assert!(!m.contains("rung-51"),
            "rung 51's own lp_disabled refusal is UNREACHABLE — § 5.18 finding 1: {m}");
}

// ============================================================================== contract 4
/// CONTRACT 4 — there is nothing left to fade. At `r = 0.5` the `phi` floor's LAST ENGAGED point
/// is `0.440`, so a trigger past it (`s_off = 0.60`) finds no clip and EVERY `tau_rel` is
/// bit-identical — and identical to the unforced rung-49 leg.
///
/// **Kept as a gate because it is the boundary that makes the instrument interpretable**: this is
/// how the first probe of this rung came back inert, and reading it as "the fade does nothing"
/// rather than "the fade was placed outside the window" would have killed the rung.
#[test]
fn contract4_s_off_past_the_natural_release_makes_tau_rel_inert() {
    let f = flight();
    let t = ft(1.0);
    let c = t.core();
    let leg = SurgeLimiter::new(Spool::Lp, PHI_LIM);
    let rows = c.rate_sweep(&f, LO, HI, 0.60, &[None, Some(0.04), Some(0.32)], Some(&leg), None,
                            R, SETTLE, DS);
    let free = c.surge_relief(&f, LO, HI, &leg, R, SETTLE, DS, None, None, None);

    let five: [Field; 5] = [
        ("relief_lp", |x| x.relief_lp),
        ("relief_hp", |x| x.relief_hp),
        ("fuel_removed", |x| x.fuel_removed),
        ("s_min_lp", |x| x.s_min_lp),
        ("s_min_hp", |x| x.s_min_hp),
    ];
    for x in &rows[1..] {
        for (k, g) in five {
            assert_eq!(g(x).to_bits(), g(&rows[0]).to_bits(), "{k}: {} vs {}", g(x), g(&rows[0]));
        }
        assert_eq!(x.relief_lp.to_bits(), free.relief_lp.to_bits(), "relief_lp vs rung 49's own");
        assert_eq!(x.relief_hp.to_bits(), free.relief_hp.to_bits(), "relief_hp vs rung 49's own");
    }
}

// ============================================================================== contract 5
/// CONTRACT 5 — the design run never sees any of this: the project's spine.
#[test]
fn contract5_cycle_untouched_bit_for_bit_rung6() {
    let f = flight();
    let eng = build_turbojet(Gas::reacting_equilibrium(), 10.0, TT4, 50_000.0, single());
    let a = eng.run(&f, 1.0);

    let t = ft(1.0);
    let leg = SurgeLimiter::new(Spool::Lp, PHI_LIM);
    let _ = t.core().release_relief(&f, LO, HI, Some(0.30), Some(&leg), None, R, SETTLE, DS,
                                    Some(0.08));

    let b = eng.run(&f, 1.0);
    assert_eq!(a.performance.specific_thrust.to_bits(), b.performance.specific_thrust.to_bits());
    assert_eq!(a.station("4").far.to_bits(), b.station("4").far.to_bits());
}

// ================================================================================== gate 3
/// GATE 3 — THE HEADLINE. For a fade over `[s_off, s_off + tau_rel]`, the two HARD releases at
/// the two ENDS bracket it: pointwise in applied fuel (gate 4) and in total `fuel_removed` (here).
/// If the debit were any monotone functional of the fuel LEVEL, or any function of the TOTAL
/// DEFICIT, the faded run would have to land BETWEEN them.
///
/// It lands OUTSIDE — shallower than BOTH brackets, on BOTH spools, at two placements and two
/// rates. The cleanest instance is `s_off = 1.56` / `tau_rel = 0.20`, whose two brackets AGREE
/// (`−0.09049` / `−0.09042`: postponing a HARD release over that interval does essentially
/// nothing) while the faded run over exactly that interval is 1.47× shallower. There is no timing
/// story left; what differs is the RATE.
#[test]
fn gate3_the_faded_release_lands_outside_its_own_bracket() {
    for (s_off, tau, far) in [(1.10, 0.20, 1.30), (1.10, 0.40, 1.50),
                              (1.56, 0.20, 1.76), (1.56, 0.40, 1.96)]
    {
        let near_b = deep(s_off, None);
        let far_b = deep(far, None);
        let mid = deep(s_off, Some(tau));
        // (i) the deficit is BRACKETED
        assert!(near_b.fuel_removed < mid.fuel_removed
                && mid.fuel_removed < far_b.fuel_removed,
                "({s_off}, {tau}): {} {} {}",
                near_b.fuel_removed, mid.fuel_removed, far_b.fuel_removed);
        // (ii) the debit is OUTSIDE the bracket — shallower than both, on BOTH spools
        for (k, g) in [("relief_lp", (|x: &ReleaseRelief| x.relief_lp) as fn(&ReleaseRelief) -> f64),
                       ("relief_hp", |x: &ReleaseRelief| x.relief_hp)]
        {
            assert!(g(&mid) > g(&near_b) && g(&mid) > g(&far_b),
                    "({s_off}, {tau}) {k}: {} {} {}", g(&near_b), g(&mid), g(&far_b));
        }
    }
    // the cleanest instance, quantified
    let n = deep(1.56, None);
    let f_ = deep(1.76, None);
    let m = deep(1.56, Some(0.20));
    assert!((n.relief_hp - f_.relief_hp).abs() < 0.001 * n.relief_hp.abs() * 10.0,
            "the two brackets must AGREE: {} vs {}", n.relief_hp, f_.relief_hp);
    assert!(m.relief_hp.abs() < n.relief_hp.abs() / 1.4,
            "the faded run must be >1.4x shallower: {} vs {}", m.relief_hp, n.relief_hp);
}

// ================================================================================== gate 4
/// GATE 4 — what upgrades gate 3 from "interpolation is violated" to "no monotone functional of
/// the fuel LEVEL can produce this": the faded march's APPLIED FUEL is bounded at EVERY march
/// point by the two hard marches,
///
/// ```text
///   hard@(s_off + tau_rel)  <=  faded  <=  hard@s_off
/// ```
///
/// Structural (a fading clip is strictly between full clip and no clip) but NOT a priori
/// guaranteed, because each leg's cap is solved at the CURRENT state and the three marches
/// diverge — so it is measured, and the count of violations must be exactly zero.
///
/// **PYTHON'S `round(s, 3)` DICT IS AN INDEX ALIGNMENT HERE**, and the `s` bits are asserted
/// equal before anything is compared. See the header.
#[test]
fn gate4_the_pointwise_applied_fuel_sandwich() {
    let f = flight();
    let t = ft(1.0);
    let c = t.core();
    let leg = SurgeLimiter::new(Spool::Lp, PHI_LIM_2);
    let march = |s_off: f64, tau_rel: Option<f64>| -> Vec<FuelPoint> {
        c.fuel_ramp_march(&f, LO, HI, R2, SETTLE, DS,
                          &FuelLimiters { surge: Some(leg), s_off: Some(s_off), tau_rel,
                                          ..Default::default() }).0
    };
    let lo_ = march(1.56, None);
    let mid = march(1.56, Some(0.20));
    let hi_ = march(1.76, None);

    assert_eq!(lo_.len(), mid.len());
    assert_eq!(lo_.len(), hi_.len());
    assert!(lo_.len() > 250, "the shared grid must be the whole march: {}", lo_.len());
    for i in 0..lo_.len() {
        assert_eq!(lo_[i].s.to_bits(), mid[i].s.to_bits(), "the three marches must share `s`");
        assert_eq!(lo_[i].s.to_bits(), hi_[i].s.to_bits(), "the three marches must share `s`");
        assert!(hi_[i].mf - 1e-15 <= mid[i].mf && mid[i].mf <= lo_[i].mf + 1e-15,
                "sandwich violated at s={}: {} / {} / {}",
                lo_[i].s, hi_[i].mf, mid[i].mf, lo_[i].mf);
    }
    // and the fade is not inert: it differs from BOTH inside the release interval
    assert!((0..lo_.len()).any(|i| {
        let s = lo_[i].s;
        s > 1.56 && s < 1.76 && mid[i].mf != lo_[i].mf && mid[i].mf != hi_[i].mf
    }), "the fade must differ from BOTH brackets somewhere inside the interval");
}

// ================================================================================== gate 5
/// GATE 5 — THE SCOPE, gated as a NEGATIVE so the claim cannot silently widen.
///
/// The bracket violation is a DEEP-DIVE phenomenon. At `s_off = 0.30` (`r = 2.0`) the same
/// construction puts the faded point INSIDE its bracket. There, rate and deficit are NOT
/// separable and this rung claims nothing.
///
/// It also FALSIFIES this rung's own written prediction P2 (`|relief|` monotone falling in
/// `tau_rel` at fixed `s_off`): here it DEEPENS with `tau_rel`. The postponement-vs-rate
/// decomposition that reconciles the two regimes is POST-HOC and is deliberately not gated.
#[test]
fn gate5_scope_the_shallow_regime_interpolates() {
    let near_b = deep(0.30, None);
    let mid = deep(0.30, Some(0.20));
    let far_b = deep(0.50, None);
    assert!(near_b.fuel_removed < mid.fuel_removed && mid.fuel_removed < far_b.fuel_removed);
    assert!(far_b.relief_hp < mid.relief_hp && mid.relief_hp < near_b.relief_hp,
            "{} {} {}", near_b.relief_hp, mid.relief_hp, far_b.relief_hp);
    // P2 falsified in this regime: DEEPER with tau_rel, not shallower
    let deeper = deep(0.30, Some(0.40));
    assert!(deeper.relief_hp < mid.relief_hp && mid.relief_hp < near_b.relief_hp,
            "{} {} {}", near_b.relief_hp, mid.relief_hp, deeper.relief_hp);
}

// ================================================================================== gate 6
/// GATE 6 — the violation reproduces on rung 48's FEEDFORWARD leg (a different instrument, a
/// different clip shape), and cross-family it is large enough to FLIP THE SIGN of the relief.
///
/// And `relief_lp` is EXACTLY `0.0` in every row: rung 48's exact-zero law (`s_eng = 0.360` is
/// downstream of `s_lp* = 0.320`) survives the RATE axis as it survived rung 50's forcing. Three
/// rungs now, unmoved — and it is an `x − x` structural zero, not a tolerance.
///
/// NOT claimed: "a slow hand-back buys back rung 48's immunity". The claim is the bracket
/// violation; the sign flip is its evidence.
#[test]
fn gate6_cross_family_the_violation_flips_the_sign() {
    let near_b = sched(1.10, None);
    let mid = sched(1.10, Some(0.40));
    let far_b = sched(1.50, None);
    assert!(near_b.fuel_removed < mid.fuel_removed && mid.fuel_removed < far_b.fuel_removed,
            "{} {} {}", near_b.fuel_removed, mid.fuel_removed, far_b.fuel_removed);
    assert!(mid.relief_hp > near_b.relief_hp && mid.relief_hp > far_b.relief_hp);
    assert!(near_b.relief_hp < 0.0 && 0.0 < mid.relief_hp,
            "the sign must FLIP: {} -> {}", near_b.relief_hp, mid.relief_hp);
    for x in [near_b, mid, far_b, sched(1.10, Some(0.20))] {
        assert_eq!(x.relief_lp.to_bits(), 0.0f64.to_bits(),
                   "rung 48's exact zero must survive the RATE axis: {}", x.relief_lp);
    }
}

// ================================================================================== gate 7
/// GATE 7 — the sweep threw up a pair matched in TOTAL FUEL REMOVED to 0.02 % with
/// OPPOSITE-SIGNED relief — **found, not solved for**, which is what keeps it out of the
/// matched-currency trap that blocked rung 48 twice. The same fuel withheld; the debit on the
/// other side of zero.
#[test]
fn gate7_the_naturally_occurring_matched_deficit_pair() {
    let faded = sched(1.10, Some(0.40));
    let hard = sched(1.30, None);
    let rel_gap = (faded.fuel_removed - hard.fuel_removed).abs() / hard.fuel_removed;
    assert!(rel_gap < 1e-3, "the deficits must be MATCHED: {rel_gap} ({} vs {})",
            faded.fuel_removed, hard.fuel_removed);
    assert!(hard.relief_hp < 0.0 && 0.0 < faded.relief_hp,
            "{} vs {}", hard.relief_hp, faded.relief_hp);
    assert!(faded.relief_hp - hard.relief_hp > 0.01);
}

// ================================================================================== gate 8
/// GATE 8 (prediction P1, both halves, and P4). A faded release relocates the minima to its
/// COMPLETION point, not to its trigger — so with an interval it is the FAR end that governs. At
/// larger `tau_rel` the minimum DETACHES into the interior, the spin-up recovery having overtaken
/// the hand-back. Neither minimum is ever upstream of the trigger.
#[test]
fn gate8_the_minimum_tracks_the_completion_point_then_detaches() {
    for (s_off, tau) in [(0.56, 0.20), (0.44, 0.40), (0.30, 0.40)] {
        let x = deep(s_off, Some(tau));
        let end = s_off + tau;
        for (k, v) in [("s_min_lp", x.s_min_lp), ("s_min_hp", x.s_min_hp)] {
            assert!(s_off - 1e-9 <= v && v <= end + DS + 1e-9,
                    "({s_off}, {tau}) {k}: {v}");
        }
        assert!(x.s_min_hp > s_off + 0.5 * tau, "({s_off}, {tau}): {}", x.s_min_hp);
    }
    let fast = deep(1.56, Some(0.04));
    let slow = deep(1.56, Some(0.40));
    assert!((fast.s_min_hp - (1.56 + 0.04)).abs() <= DS + 1e-9,
            "the fast fade must bottom AT completion: {}", fast.s_min_hp);
    assert!(slow.s_min_hp < 1.56 + 0.40 - DS, "the slow fade must DETACH: {}", slow.s_min_hp);
    assert!(slow.s_min_lp < slow.s_min_hp, "{} vs {}", slow.s_min_lp, slow.s_min_hp);
}

// ================================================================================== gate 9
/// GATE 9 — the correction to a SHIPPED rung. Rung 50 stated relocation's precondition (a) as
/// *"the release must land at or AFTER that spool's own bare minimum"*.
///
/// Rung 50's OWN § 1 table already violated it: at `s_off = 0.30`, `r = 2.0` the LP release
/// (`0.280`) is upstream of `s_lp* = 0.320`, yet `s@min phi_lp = 0.300` — relocated, and
/// un-italicised. Asserted here first, because it is internal to rung 50's published measurement.
///
/// Then the quantitative locate, over the interval rung 50 skipped: the HP minimum walks
/// MONOTONICALLY toward the release from above and locks onto `s_off` at `0.44` — a release of
/// `0.420`, `0.66×` `s_hp*` and well UPSTREAM of it. **The condition is SUFFICIENT, not
/// necessary.** Rung 50's relocation headline is untouched; its boundary was wrong.
///
/// `s_hp*` is re-measured here rather than read off a constant.
#[test]
fn gate9_rung50s_precondition_a_is_mis_stated() {
    let x30 = deep(0.30, None);
    assert!((x30.s_lp_bare - S_LP_STAR_2).abs() < 1e-9, "{}", x30.s_lp_bare);
    assert!((x30.s_hp_bare - S_HP_STAR_2).abs() < 1e-9, "{}", x30.s_hp_bare);
    // (i) rung 50's own row: LP relocated with the release UPSTREAM of s_lp*
    assert!(x30.s_rel < S_LP_STAR_2, "{} vs {S_LP_STAR_2}", x30.s_rel);
    assert!((x30.s_min_lp - 0.30).abs() <= 1e-6, "{}", x30.s_min_lp);
    // (ii) the HP crossover, upstream of s_hp*
    let scan: Vec<ReleaseRelief> = [0.30, 0.36, 0.44].into_iter().map(|so| deep(so, None))
        .collect();
    let mins: Vec<f64> = scan.iter().map(|x| x.s_min_hp).collect();
    assert!(mins[0] > mins[1] && mins[1] > mins[2],
            "must walk toward the release from above: {mins:?}");
    assert!((mins[2] - 0.44).abs() <= 1e-6, "must lock ON at s_off=0.44: {}", mins[2]);
    assert!(scan[2].s_rel < S_HP_STAR_2, "{} vs {S_HP_STAR_2}", scan[2].s_rel);
}

// ================================================================================= gate 10
/// GATE 10 — the deflation to exclude is *"any clip removes fuel and slows the accel"*. Two
/// measurements kill it: the accel ENDPOINT is unmoved across the whole sweep, and fuel removal
/// rises MONOTONICALLY in `tau_rel` while the debit FALLS — the largest removal giving the
/// SMALLEST debit, which a ramp-rate lever cannot do.
#[test]
fn gate10_not_rung_44s_ramp_rate_lever() {
    let rows: Vec<ReleaseRelief> =
        [None, Some(0.20), Some(0.40)].into_iter().map(|t| deep(1.56, t)).collect();
    let bare = rows[0].nu_hp_end_bare;
    for x in &rows {
        assert!((x.nu_hp_end - bare).abs() < 5e-4,
                "tau_rel {:?}: {} vs {bare}", x.tau_rel, x.nu_hp_end);
    }
    let rem: Vec<f64> = rows.iter().map(|x| x.fuel_removed).collect();
    let deb: Vec<f64> = rows.iter().map(|x| x.relief_hp.abs()).collect();
    assert!(rem[0] < rem[1] && rem[1] < rem[2], "{rem:?}");
    assert!(deb[0] > deb[1] && deb[1] > deb[2], "{deb:?}");
}

// ================================================================================= gate 11
/// GATE 11 — the fade puts a SECOND edge on the `ds` grid (rung 50 had one). Both the debit and
/// the relocation survive halving the step.
#[test]
fn gate11_ds_convergence() {
    let a = rel(Some(1.56), Some(0.20), Some(PHI_LIM_2), None, R2, 1.0, 0.02);
    let b = rel(Some(1.56), Some(0.20), Some(PHI_LIM_2), None, R2, 1.0, 0.01);
    for (k, g) in [("relief_lp", (|x: &ReleaseRelief| x.relief_lp) as fn(&ReleaseRelief) -> f64),
                   ("relief_hp", |x: &ReleaseRelief| x.relief_hp)]
    {
        assert!((g(&a) - g(&b)).abs() < 0.01 * g(&a).abs(), "{k}: {} vs {}", g(&a), g(&b));
    }
    assert!((a.s_min_hp - b.s_min_hp).abs() <= 0.02 + 1e-9, "{} vs {}", a.s_min_hp, b.s_min_hp);
}

// ================================================================================= gate 12
/// GATE 12 — `rho = tau_L / tau_H` is rung 40's one parameter. The headline ordering — the faded
/// run shallower than the hard release at its own trigger — survives it in both directions.
#[test]
fn gate12_the_bracket_violation_survives_rho() {
    for rho in [0.25, 4.0] {
        let hard = rel(Some(1.56), None, Some(PHI_LIM_2), None, R2, rho, DS);
        let faded = rel(Some(1.56), Some(0.20), Some(PHI_LIM_2), None, R2, rho, DS);
        assert!(hard.relief_hp < 0.0 && faded.relief_hp < 0.0, "rho {rho}");
        assert!(faded.relief_hp > hard.relief_hp,
                "rho {rho}: {} vs {}", hard.relief_hp, faded.relief_hp);
        assert!(faded.relief_lp > hard.relief_lp,
                "rho {rho}: {} vs {}", hard.relief_lp, faded.relief_lp);
    }
}
