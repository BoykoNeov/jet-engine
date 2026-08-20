//! RUNG 52 — THE ASYMMETRIC FAST-ATTACK / SLOW-RELEASE LAG: a self-releasing limiter cannot debit
//! the spool it watches.
//!
//! Port of `tests/test_rung52.py`, gate for gate. That file defines **15 test functions**, and
//! **FOUR of them carry `slow`** — counted with `pytest --collect-only -m slow`, not by grepping
//! for the decorator, because a module-level `pytestmark` would not show in a grep. They are the
//! whole of the slice's `slow` budget: rungs 49, 50 and 51 have none.
//!
//! | # | `tests/test_rung52.py` | here | `slow` |
//! |---|---|---|---|
//! | 1 | `test_reduce_lag_none_is_bit_for_bit_rungs_49_50_51` | [`contract1_lag_none_is_bit_for_bit_rungs_49_50_51`] | |
//! | 2 | `test_reduce_lag_refuses_to_compose_with_the_forced_release` | [`contract2_lag_refuses_to_compose_with_the_forced_release`] | |
//! | 3 | `test_reduce_lag_refuses_the_two_lag_cascade_and_the_unarmed_leg` | [`contract3_lag_refuses_the_cascade_and_the_unarmed_leg`] | |
//! | 4 | `test_reduce_lp_disabled_asserts` | [`contract4_lp_disabled_refuses`] | |
//! | 5 | `test_cycle_untouched_by_the_lag_bit_for_bit_rung6` | [`contract5_cycle_untouched_bit_for_bit_rung6`] | |
//! | 6 | `test_headline_the_trigger_PINS_ITSELF_and_the_credit_is_MACHINE_ZERO` | [`gate1_the_trigger_pins_itself_and_the_credit_is_machine_zero`] | |
//! | 7 | `test_headline_a_self_releasing_leg_CANNOT_DEBIT_THE_SPOOL_IT_WATCHES` | [`gate2_a_self_releasing_leg_cannot_debit_the_spool_it_watches`] | |
//! | 8 | `test_headline_the_two_clocks_separate_ONE_WAY` | [`gate3_the_two_clocks_separate_one_way`] | |
//! | 9 | `test_the_non_factorization_survives_the_ramp_rate` | [`gate4_the_non_factorization_survives_the_ramp_rate`] | ✅ |
//! | 10 | `test_rung51s_rate_verdict_TRANSFERS_with_the_anti_deflation_pair` | [`gate5_rung51s_rate_verdict_transfers`] | |
//! | 11 | `test_the_debit_crosses_zero_into_a_CREDIT_with_its_anti_degeneracy_pair` | [`gate6_the_debit_crosses_zero_into_a_credit`] | |
//! | 12 | `test_the_attack_constant_is_rung48s_ENGAGEMENT_TIME_axis` | [`gate7_the_attack_constant_is_rung48s_engagement_time_axis`] | |
//! | 13 | `test_robustness_ds_stability_of_the_crossing` | [`gate8_ds_stability_of_the_crossing`] | ✅ |
//! | 14 | `test_robustness_the_instantaneous_limit_approaches_rung49` | [`gate9_the_instantaneous_limit_approaches_rung49`] | ✅ |
//! | 15 | `test_robustness_the_headline_survives_rho` | [`gate10_the_headline_survives_rho`] | ✅ |
//!
//! # THE `armed` SEED IS THE ONE PLACE THIS PORT COULD GO WRONG SILENTLY
//!
//! `lag_relief`'s crossing loop seeds `armed = None` and guards `if armed is False`, so the FIRST
//! crossing is not counted as a re-crossing. The natural Rust `let mut armed = false` counts it
//! and puts `n_recross` one high on every row. § 5.18 finding 2 measured over six lag cells — rung
//! 52's own `(tau_att, tau_rel)` grid at both ramp rates — that the first point with `g > 0` is
//! **always still ATTACKING**, so both seeds give `n_recross = 1` everywhere. Gate 1 below asserts
//! `n_recross == 1`, and **the wrong seed passes it**. The `Option<bool>` in the source is
//! load-bearing and untested by any marched cell; the step-5 oracle carries a MANUFACTURED
//! trajectory for it, on `first_raw_min`'s tie-gate template.
//!
//! Second trap in the same eight lines, registered so the port does not "tidy" it: the
//! `g <= 0.0` arm CONTINUES — an unclipped point does NOT disarm — so folding the guard into one
//! `if / else` is wrong.
//!
//! # THE GRID
//!
//! `SETTLE` is **4.0**, as in `test_rung51.py` and NOT `test_rung50.py`'s 2.0. `_lag`'s defaults
//! are the deep-dive cell (`phi_lim = PHI_LIM_2`, `r = R2`), like rung 51's `_rel` and unlike rung
//! 50's `_sweep`; every call below spells its cell out.
//!
//! # Where Rust has to say out loud what Python leaves implicit
//!
//! * **`eps` IS A SLICE, NOT A KEY SUFFIX.** Python returns `s_eng_0.05` / `s_rel_0.01` as
//!   f-string keys; Rust returns [`LagRelief::eps_edges`], one `(eps, s_eng, s_rel)` triple per
//!   threshold in the order given. The gates index it by position, and the helper below asserts
//!   the `eps` VALUE at that position rather than trusting the order.
//! * **CONTRACT 4 asserts the FULL refusal text** — § 5.18 P2's instrument, FOURTH and last data
//!   point. Python matches `"not a reduce axis"`, which all four rungs' refusals contain. This
//!   gate arms `surge=` and `lag=`, and the assert that fires is named for **rung 49**: rung 52's
//!   own `lp_disabled` refusal is unreachable over all 255 arming combinations. With this file,
//!   P2 is measured on all four of its data points and holds on every one.
//! * **CONTRACT 3's "and it runs" line becomes a TYPE check as well as a count.** Python asserts
//!   `all("g" in p for p in ok)`; Rust asserts every point carries [`PointExtra::Asym`], which is
//!   the same claim with the failure mode (a march that silently took the non-lag route) made
//!   unrepresentable.
//!
//! # `#[ignore]`
//!
//! **None**, and the decision is measured rather than inherited. Slice M's rule is an in-suite
//! cost against the crate total; this file's four `slow` gates are `slow` in PYTHON because
//! PyPy re-marches, and the measured Rust cost is reported in § 5.18 step 4. Nothing here is
//! ignored unless that measurement says so.
//!
//! [`PointExtra::Asym`]: turbojet::fuel_transient::PointExtra::Asym

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::fuel_transient::{
    AsymmetricLag, FuelLimiters, FuelPoint, FuelTransientCore, LagRelief, PointExtra,
    SurgeLimiter, TwoSpoolFuelTransient,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;

const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const SETTLE: f64 = 4.0;
const DS: f64 = 0.02;
const R: f64 = 0.5;
const R2: f64 = 2.0;
const REDLINE: f64 = 1480.0;

const PHI_LIM: f64 = 0.7450;
const PHI_LIM_2: f64 = 0.7725;
const S_LP_STAR_2: f64 = 0.32;

/// Python's `lag_relief` default `eps=(0.05, 0.01)`. **The ORDER is load-bearing** — the gates
/// read index 0 as `0.05` and index 1 as `0.01`.
const EPS: [f64; 2] = [0.05, 0.01];

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

fn ft(rho: f64) -> TwoSpoolFuelTransient {
    TwoSpoolFuelTransient::new(design(cpg_gas()), flight(), 1.0, lp_shaped(), hp_shaped(), rho)
}

fn ramp(core: &FuelTransientCore, r: f64) -> (impl Fn(f64) -> f64, (f64, f64)) {
    let f = flight();
    let mf0 = core.fuel_for_tt4(&f, LO);
    let mf1 = core.fuel_for_tt4(&f, HI);
    let eq0 = core.inner.equilibrium(&f, LO);
    (move |s: f64| mf0 + (mf1 - mf0) * (s / r).min(1.0), (eq0.nu_lp, eq0.nu_hp))
}

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

/// Python's `_ROWS` key `(tau_att, tau_rel, phi_lim, r, rho, ds)`, in BITS.
type LagKey = (u64, u64, u64, u64, u64, u64);

#[allow(clippy::type_complexity)]
fn rows_memo() -> &'static Mutex<HashMap<LagKey, LagRelief>> {
    static M: OnceLock<Mutex<HashMap<LagKey, LagRelief>>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Python's `_lag`, memo and all. Defaults are the DEEP-DIVE cell.
fn lag(tau_att: f64, tau_rel: f64, phi_lim: f64, r: f64, rho: f64, ds: f64) -> LagRelief {
    let key: LagKey = (tau_att.to_bits(), tau_rel.to_bits(), phi_lim.to_bits(), r.to_bits(),
                       rho.to_bits(), ds.to_bits());
    if let Some(hit) = rows_memo().lock().expect("memo poisoned").get(&key) {
        return hit.clone();
    }
    let t = ft(rho);
    let leg = SurgeLimiter::new(Spool::Lp, phi_lim);
    let out = t.core().lag_relief(&flight(), LO, HI, AsymmetricLag::new(tau_att, tau_rel),
                                  Some(&leg), None, r, SETTLE, ds, &EPS);
    rows_memo().lock().expect("memo poisoned").insert(key, out.clone());
    out
}

/// The default cell.
fn deep(tau_att: f64, tau_rel: f64) -> LagRelief {
    lag(tau_att, tau_rel, PHI_LIM_2, R2, 1.0, DS)
}

/// `s_eng_<eps>` / `s_rel_<eps>` by INDEX, with the `eps` VALUE asserted rather than assumed —
/// Python reads these off f-string keys and cannot get the order wrong.
fn edges(x: &LagRelief, i: usize) -> (f64, f64) {
    let (e, s_eng, s_rel) = x.eps_edges[i];
    assert_eq!(e.to_bits(), EPS[i].to_bits(),
               "eps_edges[{i}] must be eps={}, got {e}", EPS[i]);
    (s_eng, s_rel)
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
/// CONTRACT 1 — `lag = None` never enters `integrate_fuel_asym` (exact dispatch, rung 47's own
/// contract), so every earlier march is reproduced bit-identically through the NEW signature.
/// Checked on SIX arming combinations, so the new parameter is proved inert against the bare leg,
/// the rung-49 floor, rung 48's schedule, rung 50's forced release and rung 51's fade.
#[test]
fn contract1_lag_none_is_bit_for_bit_rungs_49_50_51() {
    let f = flight();
    let t = ft(1.0);
    let core = t.core();
    let (sch, nu0) = ramp(core, R);
    let acc = core.accel_schedule(&f, LO, HI, 0.25, 13);
    let leg = SurgeLimiter::new(Spool::Lp, PHI_LIM);
    let end = R + 1.0;

    let cases: [FuelLimiters; 6] = [
        FuelLimiters::default(),
        FuelLimiters { surge: Some(leg), ..Default::default() },
        FuelLimiters { accel: Some(&acc), ..Default::default() },
        FuelLimiters { surge: Some(leg), s_off: Some(0.30), ..Default::default() },
        FuelLimiters { surge: Some(leg), s_off: Some(0.30), tau_rel: Some(0.10),
                       ..Default::default() },
        FuelLimiters { tt4_max: Some(REDLINE), tau_gov: Some(0.2), surge: Some(leg),
                       ..Default::default() },
    ];
    for lim in &cases {
        same(&core.integrate_fuel(&f, &sch, nu0, end, DS, lim),
             &core.integrate_fuel(&f, &sch, nu0, end, DS,
                                  &FuelLimiters { lag: None, ..*lim }));
    }
}

// ============================================================================== contract 2
/// CONTRACT 2 — `s_off` / `tau_rel` and the lag are ALTERNATIVE release instruments. Forcing a
/// release on a leg whose clip is already a STATE would have to zero that state — a third
/// instrument, and exactly the argument rung 50 makes when it refuses the rung-46/47 governor.
#[test]
fn contract2_lag_refuses_to_compose_with_the_forced_release() {
    let f = flight();
    let t = ft(1.0);
    let core = t.core();
    let (sch, nu0) = ramp(core, R);
    let leg = SurgeLimiter::new(Spool::Lp, PHI_LIM);
    let lg = AsymmetricLag::new(0.02, 0.10);

    for lim in [FuelLimiters { surge: Some(leg), lag: Some(lg), s_off: Some(0.30),
                               ..Default::default() },
                FuelLimiters { surge: Some(leg), lag: Some(lg), s_off: Some(0.30),
                               tau_rel: Some(0.10), ..Default::default() }]
    {
        let m = refusal(|| {
            core.integrate_fuel(&f, &sch, nu0, R + 1.0, DS, &lim);
        })
        .expect("the lag must refuse to compose with a forced release");
        assert!(m.contains("not composable"), "the refusal must name the reason: {m}");
    }
}

// ============================================================================== contract 3
/// CONTRACT 3 — `tau_gov` (rung 47) and `lag` are both a clip AMOUNT carried as a state, on two
/// different legs: a cascade, not this rung. And a lag with no leg to lag is meaningless. The
/// INSTANTANEOUS redline (`tt4_max` alone) DOES compose, and is checked to run.
#[test]
fn contract3_lag_refuses_the_cascade_and_the_unarmed_leg() {
    let f = flight();
    let t = ft(1.0);
    let core = t.core();
    let (sch, nu0) = ramp(core, R);
    let leg = SurgeLimiter::new(Spool::Lp, PHI_LIM);
    let lg = AsymmetricLag::new(0.02, 0.10);

    let m = refusal(|| {
        core.integrate_fuel(&f, &sch, nu0, R + 1.0, DS,
                            &FuelLimiters { surge: Some(leg), lag: Some(lg),
                                            tt4_max: Some(REDLINE), tau_gov: Some(0.2),
                                            ..Default::default() });
    })
    .expect("the two-lag cascade must refuse");
    assert!(m.contains("two-lag cascade"), "{m}");

    let m = refusal(|| {
        core.integrate_fuel(&f, &sch, nu0, R + 1.0, DS,
                            &FuelLimiters { lag: Some(lg), ..Default::default() });
    })
    .expect("a lag with no leg must refuse");
    assert!(m.contains("arm one"), "{m}");

    // tau <= 0 is rung 49, not a lag — the constructor's own refusal
    assert!(refusal(|| { let _ = AsymmetricLag::new(0.0, 0.10); }).is_some(),
            "AsymmetricLag(0.0, ...) must refuse");

    // ... and the instantaneous redline composes and RUNS
    let ok = core.integrate_fuel(&f, &sch, nu0, R + 1.0, DS,
                                 &FuelLimiters { surge: Some(leg), lag: Some(lg),
                                                 tt4_max: Some(REDLINE), ..Default::default() });
    assert!(!ok.is_empty());
    assert!(ok.iter().all(|p| matches!(p.extra, PointExtra::Asym { .. })),
            "every point must carry the lag route's `g` / `required`");
}

// ============================================================================== contract 4
/// CONTRACT 4 — inherited from rungs 49/50/51: the finding is a split BETWEEN spools, so the
/// single-spool degeneracy is not a reduce axis for it.
///
/// **§ 5.18 P2's INSTRUMENT, FOURTH AND LAST DATA POINT.** Python matches `"not a reduce axis"`,
/// which all four rungs' refusals contain. The assert that fires is named for **rung 49**.
#[test]
fn contract4_lp_disabled_refuses() {
    let f = flight();
    let se: Engine = build_turbojet(cpg_gas(), 10.0, TT4, 50_000.0, single_matchable());
    let deg = TwoSpoolFuelTransient::lp_disabled(se, flight(), 1.0, hp_shaped());
    let leg = SurgeLimiter::new(Spool::Lp, 0.75);

    let m = refusal(|| {
        deg.integrate_fuel_lp_disabled(
            &f, |_s| 0.5, 1.0, R + 0.5, DS,
            &FuelLimiters { surge: Some(leg), lag: Some(AsymmetricLag::new(0.02, 0.10)),
                            ..Default::default() });
    })
    .expect("the rung-52 lag on an lp_disabled object must refuse");
    assert!(m.contains("not a reduce axis"), "the refusal must name the reason: {m}");
    assert_eq!(
        m,
        "the rung-49 phi floor is inherently two-shaft (its finding is the CREDIT on the \
         watched spool against the DEBIT on the other); lp_disabled is not a reduce axis \
         for a split BETWEEN spools.");
    assert!(!m.contains("rung-52"),
            "rung 52's own lp_disabled refusal is UNREACHABLE — § 5.18 finding 1: {m}");
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
    let _ = t.core().lag_relief(&f, LO, HI, AsymmetricLag::new(0.02, 0.10), Some(&leg), None,
                                R, SETTLE, DS, &EPS);

    let b = eng.run(&f, 1.0);
    assert_eq!(a.performance.specific_thrust.to_bits(), b.performance.specific_thrust.to_bits());
    assert_eq!(a.station("4").far.to_bits(), b.station("4").far.to_bits());
}

// ================================================================================== gate 1
/// GATE 1 — RUNG 51'S DEFERRAL REASON 1 IS FALSE.
///
/// Rung 51: *"a lag's release edge is EMERGENT … sweep its time constant and the release time
/// moves with it — reinstating exactly the confound `s_off` was built to kill."* It does not.
/// `tau_rel` is never READ while `required > g`, so the crossing, the clip state AT the crossing,
/// the engagement edge and the watched spool's relief are all invariant across a 20× sweep.
///
/// **The credit spread is asserted EXACTLY ZERO, not merely small** — a tolerance would hide the
/// point, which is that the pre-crossing march is BIT-identical and not just close.
///
/// WHERE THE BIT-IDENTITY ACTUALLY STOPS, stated because the gate measures it: strictly, up to the
/// RK4 step that STRADDLES the crossing. That step's later sub-stages already have
/// `required < g`, so they read `tau_rel`, and the crossing is RECORDED at the next grid point.
/// So `s_cross` and `s_eng` are exact (grid coordinates) and `relief_watched` is exact (the
/// watched minimum lies strictly upstream of the straddling step), but `g_at_cross` carries a
/// partial-step residual, ~4e-4 relative. That is the integrator's granularity, not the argument.
///
/// **AND THE `n_recross == 1` LINE IS THE ONE THE WRONG `armed` SEED ALSO PASSES** — see the
/// header. It is kept because Python has it, not because it discriminates.
#[test]
fn gate1_the_trigger_pins_itself_and_the_credit_is_machine_zero() {
    let rows: Vec<LagRelief> = [0.02, 0.10, 0.40].into_iter().map(|tr| deep(0.02, tr)).collect();
    for x in &rows[1..] {
        assert_eq!(x.s_cross.to_bits(), rows[0].s_cross.to_bits(),
                   "tau_rel {}: s_cross {}", x.tau_rel, x.s_cross);
        assert_eq!(edges(x, 0).0.to_bits(), edges(&rows[0], 0).0.to_bits(),
                   "tau_rel {}: s_eng_0.05", x.tau_rel);
        assert_eq!(x.relief_watched.expect("surge armed").to_bits(),
                   rows[0].relief_watched.expect("surge armed").to_bits(),
                   "tau_rel {}: relief_watched must be MACHINE ZERO apart", x.tau_rel);
        assert!((x.g_at_cross - rows[0].g_at_cross).abs() < 1e-3 * rows[0].g_at_cross,
                "tau_rel {}: {} vs {}", x.tau_rel, x.g_at_cross, rows[0].g_at_cross);
    }
    // the honest caveat, made measurable: the pinning is exact for the FIRST crossing
    assert!(rows.iter().all(|x| x.n_recross == 1),
            "{:?}", rows.iter().map(|x| x.n_recross).collect::<Vec<_>>());
    // ... while the RELEASE side genuinely moved (otherwise the invariance is vacuous)
    assert!(edges(&rows[2], 1).1 > edges(&rows[0], 1).1 + 0.5,
            "s_rel_0.01 must MOVE: {} vs {}", edges(&rows[0], 1).1, edges(&rows[2], 1).1);
}

// ================================================================================== gate 2
/// GATE 2 — THE CROSS-RUNG PAYOFF, and the NON-TAUTOLOGY.
///
/// *"`tau_rel` cannot touch anything upstream of the crossing"* is structural and, alone, a
/// tautology. The content is the SECOND step: the watched spool's OWN minimum lands upstream of
/// the crossing, because the lag's undershoot is largest EARLY (while `g` is still climbing) —
/// rung 48's arrest law through the lag's attack transient. Note it is the ACTUAL `phi_lp`
/// minimum, not `required`'s turnover: under a lag `phi_lp` dips BELOW `phi_lim`, so the two are
/// different objects.
///
/// Composed with gate 1: **a self-releasing limiter cannot debit the spool it watches** — which
/// BOUNDS rung 50's watched-side debit to FORCED releases and RESTORES rung 49's identity.
///
/// Gated with the credit POSITIVE, so there is a real credit for `tau_rel` to fail to move.
#[test]
fn gate2_a_self_releasing_leg_cannot_debit_the_spool_it_watches() {
    for (r, floors, lp_star) in [(R2, [0.7650, 0.7725], S_LP_STAR_2),
                                 (R, [0.7450, 0.7480], 0.24)]
    {
        for pl in floors {
            let a = lag(0.02, 0.02, pl, r, 1.0, DS);
            let b = lag(0.02, 0.40, pl, r, 1.0, DS);
            let (aw, bw) = (a.relief_watched.expect("armed"), b.relief_watched.expect("armed"));
            assert!(aw > 0.0, "r {r} floor {pl}: a real credit, got {aw}");
            assert_eq!(aw.to_bits(), bw.to_bits(), "r {r} floor {pl}: machine zero");
            assert!(a.s_min_lp < a.s_cross, "r {r} floor {pl}: {} vs {}", a.s_min_lp, a.s_cross);
            assert!(a.s_min_lp <= lp_star + 1e-9, "r {r} floor {pl}: {}", a.s_min_lp);
            assert_eq!(a.s_min_lp.to_bits(), b.s_min_lp.to_bits(), "r {r} floor {pl}");
        }
    }
}

// ================================================================================== gate 3
/// GATE 3 — DOES RUNG 49'S SPLIT FACTOR ACROSS THE TWO CONSTANTS?
///
/// A real fast-attack / slow-release limiter is DESIGNED on the premise that it does. This is the
/// first instrument on which rung 49's two clocks are independently dialable on ONE realisable
/// leg, so the premise is testable.
///
/// ANSWER: one way only. `tau_att` owns the credit EXACTLY; the debit's additive-separability
/// residual comes back the SAME ORDER as the main effects. The premise is HALF TRUE, and the half
/// that fails is the PROTECTIVE one.
///
/// **§ 5.18 P5**: `credit_spread` is asserted EXACTLY `0.0`, with no tolerance, on both grids.
#[test]
fn gate3_the_two_clocks_separate_one_way() {
    let f = flight();
    let t = ft(1.0);
    let leg = SurgeLimiter::new(Spool::Lp, PHI_LIM_2);
    let g = t.core().factorization_grid(&f, LO, HI, &[0.02, 0.20], &[0.02, 0.10, 0.40],
                                        Some(&leg), None, R2, SETTLE, DS, &EPS);
    for (ta, spread) in &g.credit_spread {
        assert_eq!(spread.to_bits(), 0.0f64.to_bits(),
                   "tau_att {ta}: credit_spread must be EXACTLY zero, got {spread}");
    }
    assert!(g.max_residual > 0.4 * g.max_main_effect,
            "{} vs {}", g.max_residual, g.max_main_effect);
    // and it is not multiplicatively separable either — the tau_rel ratio DRIFTS
    let other = |x: &LagRelief| x.relief_other.expect("armed");
    let r0 = other(&g.grid[0][1]) / other(&g.grid[0][0]);
    let r1 = other(&g.grid[1][1]) / other(&g.grid[1][0]);
    assert!((r1 - r0).abs() > 0.05, "{r0} vs {r1}");
}

// ================================================================================== gate 4
/// GATE 4 — rung 51 was burned by claiming beyond a swept regime (its own P2 falsified), so the
/// general-sounding half of gate 3 is checked at the OTHER ramp rate before it is claimed.
///
/// **§ 5.18 FINDING 7 CORRECTS THIS GATE'S DOCSTRING.** Python says "70 % at `r = 0.5` against
/// 62 % at `r = 2.0`". Measured on the gates' own cells at the right settle time: **65.0 % and
/// 58.9 %**. Four alternative denominators were tried and none reproduces both figures. Both
/// clear the `0.4` bar and **no gate reads the quoted numbers** — including this one, which
/// asserts the bar and not the figure.
#[test]
fn gate4_the_non_factorization_survives_the_ramp_rate() {
    let f = flight();
    let t = ft(1.0);
    let leg = SurgeLimiter::new(Spool::Lp, PHI_LIM);
    let g = t.core().factorization_grid(&f, LO, HI, &[0.02, 0.32], &[0.01, 0.16],
                                        Some(&leg), None, R, SETTLE, 0.01, &EPS);
    for (ta, spread) in &g.credit_spread {
        assert_eq!(spread.to_bits(), 0.0f64.to_bits(), "tau_att {ta}: {spread}");
    }
    assert!(g.max_residual > 0.4 * g.max_main_effect,
            "{} vs {}", g.max_residual, g.max_main_effect);
}

// ================================================================================== gate 5
/// GATE 5 — rung 51's headline (the debit is not a function of the total deficit) on a
/// PHYSICALLY-REALISABLE leg. A slower hand-back gives a SHALLOWER debit while `fuel_removed`
/// RISES: more fuel removed, smaller debit. That is the anti-deflation discipline rungs 48/49/50
/// all carry, and it is what excludes "any clip removes fuel and slows the accel".
#[test]
fn gate5_rung51s_rate_verdict_transfers() {
    let rows: Vec<LagRelief> = [0.02, 0.10, 0.40].into_iter().map(|tr| deep(0.02, tr)).collect();
    let debits: Vec<f64> = rows.iter().map(|x| x.relief_other.expect("armed")).collect();
    let removed: Vec<f64> = rows.iter().map(|x| x.fuel_removed).collect();
    assert!(debits[0] < debits[1] && debits[1] < debits[2] && debits[2] < 0.0, "{debits:?}");
    assert!(removed[0] < removed[1] && removed[1] < removed[2], "{removed:?}");
}

// ================================================================================== gate 6
/// GATE 6 — the sign flip is the strongest single number in the grid AND it sits where the leg
/// engages LEAST, so rungs 49/50's `nu_hp_end` pair must clear it before it is quoted: if the
/// accel failed to complete there, the flip would be degeneracy, not physics.
///
/// It clears — the flipped rows are the LEAST perturbed of all.
#[test]
fn gate6_the_debit_crosses_zero_into_a_credit() {
    let flip = deep(0.20, 0.40);
    let deep_corner = deep(0.02, 0.02);
    assert!(flip.relief_other.expect("armed") > 0.0, "{:?}", flip.relief_other);
    assert!(deep_corner.relief_other.expect("armed") < 0.0, "{:?}", deep_corner.relief_other);
    for x in [&flip, &deep_corner] {
        let rel = (x.nu_hp_end - x.nu_hp_end_bare).abs() / x.nu_hp_end_bare;
        assert!(rel < 1e-5, "({}, {}): the accel must COMPLETE, {rel}", x.tau_att, x.tau_rel);
    }
    assert!((flip.nu_hp_end - flip.nu_hp_end_bare).abs()
            < (deep_corner.nu_hp_end - deep_corner.nu_hp_end_bare).abs(),
            "the flipped row must be the LESS perturbed one");
}

// ================================================================================== gate 7
/// GATE 7 — the credit side is rung 48's law in realisable clothing: a slower attack engages LATER
/// and credits LESS. Reported because it is what makes `tau_att` the CREDIT axis — without it,
/// "`tau_att` owns the credit" would be a label rather than a mechanism.
#[test]
fn gate7_the_attack_constant_is_rung48s_engagement_time_axis() {
    let rows: Vec<LagRelief> = [0.02, 0.10, 0.40].into_iter().map(|ta| deep(ta, 0.10)).collect();
    let eng: Vec<f64> = rows.iter().map(|x| edges(x, 0).0).collect();
    let cred: Vec<f64> = rows.iter().map(|x| x.relief_watched.expect("armed")).collect();
    assert!(eng[0] < eng[1] && eng[1] < eng[2], "must engage LATER: {eng:?}");
    assert!(cred[0] > cred[1] && cred[1] > cred[2] && cred[2] > 0.0,
            "must credit LESS: {cred:?}");
    assert!(rows[0].s_cross > rows[2].s_cross, "{} vs {}", rows[0].s_cross, rows[2].s_cross);
}

// ================================================================================== gate 8
/// GATE 8 — this gate underwrites every invariance number above: if the KINK were resolved
/// differently at different resolutions, `s_cross` would wander and all of them would inherit it.
/// It moves by at most ONE GRID CELL per halving — the resolution limit of "first recorded point
/// with `required < g`", not motion of the crossing — and the reliefs converge.
#[test]
fn gate8_ds_stability_of_the_crossing() {
    let mut prev: Option<(f64, f64)> = None;
    for ds in [0.04, 0.02, 0.01] {
        let row = lag(0.02, 0.10, PHI_LIM_2, R2, 1.0, ds);
        if let Some((ps, php)) = prev {
            assert!((row.s_cross - ps).abs() <= 2.0 * ds + 1e-9,
                    "ds {ds}: {ps} -> {}", row.s_cross);
            assert!((row.min_phi_hp_lag - php).abs() < 1e-4, "ds {ds}: {php}");
        }
        prev = Some((row.s_cross, row.min_phi_hp_lag));
    }
}

// ================================================================================== gate 9
/// GATE 9 — `tau -> 0` must APPROACH rung 49's instantaneous min-select, never bit-for-bit: a lag
/// does not snap. `ds` is held FIXED while `tau` varies alone (halving both would measure
/// neither limit). The watched spool approaches the floor FROM BELOW — a lag cannot hold a floor
/// instantaneously, so it UNDERSHOOTS, and the undershoot shrinks with `tau`.
///
/// What this rules out is a structural mismatch between `required` and the min-select. The
/// observed order is SUB-first (~0.8) and is deliberately not gated as 1.
#[test]
fn gate9_the_instantaneous_limit_approaches_rung49() {
    let f = flight();
    let t = ft(1.0);
    let core = t.core();
    let (sch, nu0) = ramp(core, R);
    let leg = SurgeLimiter::new(Spool::Lp, PHI_LIM);
    let mut prev: Option<f64> = None;
    for tau in [0.08, 0.04, 0.02] {
        let traj = core.integrate_fuel(
            &f, &sch, nu0, R + SETTLE, 0.005,
            &FuelLimiters { surge: Some(leg), lag: Some(AsymmetricLag::new(tau, tau)),
                            ..Default::default() });
        let under = PHI_LIM - traj.iter().fold(f64::INFINITY, |m, p| m.min(p.phi_lp));
        assert!(under > 0.0, "tau {tau}: must UNDERSHOOT the floor, got {under}");
        if let Some(p) = prev {
            assert!(under < p, "tau {tau}: the undershoot must SHRINK, {under} vs {p}");
        }
        prev = Some(under);
    }
}

// ================================================================================= gate 10
/// GATE 10 — `rho = tau_L / tau_H` is rung 40's one parameter. Both headline signs survive it in
/// both directions: the credit stays exactly `tau_rel`-invariant, and a slower hand-back stays
/// shallower on the unwatched spool.
#[test]
fn gate10_the_headline_survives_rho() {
    for rho in [0.25, 4.0] {
        let a = lag(0.02, 0.02, PHI_LIM_2, R2, rho, DS);
        let b = lag(0.02, 0.40, PHI_LIM_2, R2, rho, DS);
        assert_eq!(a.relief_watched.expect("armed").to_bits(),
                   b.relief_watched.expect("armed").to_bits(), "rho {rho}");
        assert_eq!(a.s_cross.to_bits(), b.s_cross.to_bits(),
                   "rho {rho}: {} vs {}", a.s_cross, b.s_cross);
        assert!(b.relief_other.expect("armed") > a.relief_other.expect("armed"),
                "rho {rho}: {:?} vs {:?}", a.relief_other, b.relief_other);
    }
}
