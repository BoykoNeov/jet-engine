//! RUNG 37 — THE TWO INTERNAL CLOCKS: volume-filling CONFIRMS, heat-soak CORRECTS.
//!
//! Rung 34 filed everything below the rotor as ONE bundled concession — *"no combustor
//! volume-filling, no heat soak … faster clocks below `tau_spool`, they do not change the `r`
//! framing"*. That sentence makes two claims about two different clocks, and they fall on
//! opposite sides of `tau_spool`:
//!
//! * a combustor **PLENUM** is genuinely fast, so it **CONFIRMS** — the `r -> 0` peak excursion
//!   still lands on rung 35's algebraic `E0`, independent of the fill clock. Its content is
//!   structural rather than dynamic: it is the **first rung where compressor mass flow differs
//!   from NGV mass flow**;
//! * **HEAT-SOAK** is not fast, so it **CORRECTS** — a second STATE carries thermal memory and
//!   the transient becomes `E(r, theta0)`. Surge is PROTECTED (`cold < hot < adiabatic`, so rung
//!   34/35's adiabatic combustor was the conservative worst case) and the cost is the
//!   acceleration-time LAG.
//!
//! The seven gates of `tests/test_rung37.py`, in file order. **All seven port, none deferred.**
//!
//! | # | `tests/test_rung37.py` | here |
//! |---|---|---|
//! | 1 | `test_reduce_both_off_is_rung35_bit_for_bit` | [`gate1_both_off_is_rung35_bit_for_bit`] |
//! | 2 | `test_plenum_equilibrium_is_rung35` | [`gate2_the_plenum_equilibrium_is_rung35`] |
//! | 3 | `test_plenum_peak_is_E0_and_the_split_is_real` | [`gate3_the_peak_is_e0_and_the_split_is_real`] |
//! | 4 | `test_heat_soak_equilibrium_is_rung35_transient_only` | [`gate4_the_soak_equilibrium_is_rung35`] |
//! | 5 | `test_heat_soak_cold_below_hot_below_adiabatic` | [`gate5_cold_below_hot_below_adiabatic`] |
//! | 6 | `test_heat_soak_accel_time_lag` | [`gate6_the_accel_time_lag`] |
//! | 7 | `test_cycle_untouched_bit_for_bit_rung6` | [`gate7_the_design_cycle_is_untouched`] |
//!
//! **PLUS THREE THE PORT ADDS**, listed so a name diff against the Python file reads correctly:
//! [`the_exhaustion_arm_is_reached_and_at_exactly_one_site`],
//! [`the_plenum_instant_reaches_the_hook_and_never_the_nozzle`] and
//! [`the_marches_run_to_length_with_no_bracket_failure`]. All three are COUNT gates for things
//! § 5.14 pre-registered and no value can see — the arm slice P shipped as unreachable, the
//! branch the plenum path structurally cannot take, and the `try` rung 37's marches do not have.
//!
//! **Python marks gate 5 `slow`; the Rust does not.** Slice M's rule — *port the gate, drop the
//! marker, re-introduce `#[ignore]` only against a MEASURED cost* — and § 5.14 prediction 8
//! registered that it must be measured in BOTH directions rather than applied reflexively,
//! because phase 6 is the first phase where an `#[ignore]` might genuinely be earned. The
//! measurement is in § 5.14's step-2 write-up.

use turbojet::combustor::{counters, CombustorTransient, Theta0};
use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::gas::Gas;
use turbojet::map::ComponentMap;
use turbojet::spool::{counters as scount, SpoolTransient};

const PI_C: f64 = 10.0;
const TT4: f64 = 1500.0;

fn flight() -> FlightCondition {
    FlightCondition { t0: 250.0, p0: 50_000.0, m0: 0.85 }
}

fn real() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.88, e_c: None, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, e_t: None,
        eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
    }
}

fn design() -> Engine {
    build_turbojet(Gas::thermally_perfect(), PI_C, TT4, 50_000.0, real())
}

/// Python's `_build`.
fn build(cmap: ComponentMap, r_v: f64, g: f64, r_m: f64) -> CombustorTransient {
    CombustorTransient::new(design(), flight(), 1.0, cmap, r_v, g, r_m)
}

/// Python's `SURGE_SHAPES`, with their names for the messages.
fn shapes() -> [(&'static str, ComponentMap); 3] {
    [
        ("surge_flow", ComponentMap::surge_flow()),
        ("surge_pressure", ComponentMap::surge_pressure()),
        ("surge_tilted", ComponentMap::surge_tilted()),
    ]
}

// ------------------------------------------------------------------------------------ gate 1
/// GATE 1 — with the plenum and heat-soak OFF (the defaults), a `CombustorTransient` IS rung
/// 34/35: the inherited `equilibrium_fuel`/`integrate_fuel` never read the OFF knobs, so they
/// equal a plain [`SpoolTransient`]'s outputs BIT-for-bit.
///
/// **The reduce is exact DISPATCH, not a stiff limit** — that is the whole design of the rung, and
/// it is why this compares bits rather than a tolerance. In Rust the OFF switches are simply
/// `f64`s the plenum and soak methods refuse to run without, and `inner` is untouched.
#[test]
fn gate1_both_off_is_rung35_bit_for_bit() {
    let cmap = ComponentMap::surge_flow();
    let ct = build(cmap, 0.0, 0.0, 0.0); // plenum / soak OFF
    let st = SpoolTransient::new(design(), flight(), 1.0, cmap);
    for tt4 in [1500.0f64, 1200.0, 900.0] {
        let mf = st.fuel_for_tt4(&flight(), tt4, None);
        let a = ct.inner.equilibrium_fuel(&flight(), mf, None);
        let b = st.equilibrium_fuel(&flight(), mf, None);
        assert_eq!(a.nu.to_bits(), b.nu.to_bits(), "both-off != rung 35 (nu) at Tt4={tt4}");
        assert_eq!(a.pi_c.to_bits(), b.pi_c.to_bits(), "both-off != rung 35 (pi_c) at Tt4={tt4}");
        assert_eq!(
            a.tau_t.to_bits(), b.tau_t.to_bits(),
            "both-off != rung 35 (tau_t) at Tt4={tt4}"
        );
    }
    // And the OFF constructor's dead coefficient really is zero — the branch no physics path
    // reaches, spelled anyway (slice N step 3's rule).
    assert_eq!(build(cmap, 0.0, 0.0, 0.0).plenum_k, 0.0);
}

// ------------------------------------------------------------------------------------ gate 2
/// GATE 2 — the PLENUM equilibrium (`dnu/ds = 0` AND `dpt4/ds = 0`) reproduces rung 35's
/// `equilibrium_fuel` via the BACK-PRESSURE closure — a genuinely different code path than rung
/// 35's NGV-continuity root find. Two closures, one operating point; and `mdot_c = mdot_NGV` holds
/// at the fixed point, which is where the decoupling closes.
#[test]
fn gate2_the_plenum_equilibrium_is_rung35() {
    for (name, cmap) in shapes() {
        let ct = build(cmap, 0.05, 0.0, 0.0);
        for tt4 in [1400.0f64, 1100.0, 900.0] {
            let mf = ct.inner.fuel_for_tt4(&flight(), tt4, Some(&cmap));
            let a = ct.equilibrium_plenum(&flight(), mf, Some(&cmap));
            let b = ct.inner.equilibrium_fuel(&flight(), mf, Some(&cmap));
            assert!(
                (a.pi_c - b.pi_c).abs() <= 1e-9 * b.pi_c,
                "{name}: plenum eq pi_c != rung 35 at Tt4={tt4} ({} vs {})", a.pi_c, b.pi_c
            );
            assert!(
                (a.nu - b.nu).abs() <= 1e-9,
                "{name}: plenum eq nu != rung 35 at Tt4={tt4} ({} vs {})", a.nu, b.nu
            );
            assert!(
                (a.mdot_c + mf - a.mdot_ngv).abs() <= 1e-9 * a.mdot_ngv,
                "{name}: plenum equilibrium mass balance not closed at Tt4={tt4}"
            );
        }
    }
}

// ------------------------------------------------------------------------------------ gate 3
/// GATE 3 — **THE PLENUM FINDING.** At `r -> 0` (a frozen spool) the plenum fills to its full
/// quasi-steady `pt4` before `nu` can move, so the peak surge excursion lands on rung 35's
/// algebraic `E0` to tolerance, INDEPENDENT of the fill clock `r_v` — the CONFIRMATION. The
/// structural content is the mass-flow SPLIT the plenum stores: `mdot_c != mdot_NGV` during the
/// fill, the first rung where the two differ at all.
#[test]
fn gate3_the_peak_is_e0_and_the_split_is_real() {
    for (name, cmap) in shapes() {
        let mut peaks = Vec::new();
        for r_v in [0.03f64, 0.1] {
            let ct = build(cmap, r_v, 0.0, 0.0);
            let r = ct.plenum_frozen_peak(&flight(), 1100.0, 1400.0, Some(&cmap), 1.0 / 15.0);
            assert!(
                r.peak_minus_e0.abs() <= 1e-6,
                "{name}: plenum peak != E0 (peak-E0={:.2e}) at r_v={r_v}", r.peak_minus_e0
            );
            assert!(
                r.split_max > 0.05,
                "{name}: the mass-flow split must be REAL (mdot_c != mdot_NGV); got {:.3e}",
                r.split_max
            );
            peaks.push(r.peak);
        }
        assert!(
            (peaks[0] - peaks[1]).abs() <= 1e-9,
            "{name}: the plenum peak must not depend on r_v ({} vs {})", peaks[0], peaks[1]
        );
    }
}

// ------------------------------------------------------------------------------------ gate 4
/// GATE 4 — the HEAT-SOAK equilibrium reproduces rung 35 because at steady state
/// `Tm = Tt4_burner` ⇒ `Q = 0` ⇒ `Tt4_turb = Tt4_burner`: heat-soak NEVER moves the running line.
/// The reduce is a fixed-point IDENTITY, not a knob-to-zero limit.
#[test]
fn gate4_the_soak_equilibrium_is_rung35() {
    for (name, cmap) in shapes() {
        let ct = build(cmap, 0.0, 0.1, 3.0);
        for tt4 in [1400.0f64, 1100.0] {
            let mf = ct.inner.fuel_for_tt4(&flight(), tt4, Some(&cmap));
            let a = ct.equilibrium_soak(&flight(), mf, Some(&cmap));
            let b = ct.inner.equilibrium_fuel(&flight(), mf, Some(&cmap));
            assert!(
                (a.inst.pi_c - b.pi_c).abs() <= 1e-9 * b.pi_c,
                "{name}: soak eq pi_c != rung 35 at Tt4={tt4}"
            );
            assert!(
                (a.inst.nu - b.nu).abs() <= 1e-9,
                "{name}: soak eq nu != rung 35 at Tt4={tt4}"
            );
        }
    }
}

// ------------------------------------------------------------------------------------ gate 5
/// GATE 5 — **THE HEAT-SOAK FINDING**, and the load-bearing SIGN. The peak surge excursion obeys
/// `cold first-accel < hot reslam < adiabatic`: the cold metal's heat sink depresses `Tt4_turb`,
/// the colder sonic throat passes more corrected flow, and the operating point moves AWAY from
/// surge; a hot reslam recovers most of the adiabatic worst case. Shape- AND knob-robust;
/// magnitudes disclaimed.
///
/// `s_end = 6.0` because `E_surge` peaks early, near `nu0` — Python's own comment, and the reason
/// the sweep is affordable at 12 configurations.
#[test]
fn gate5_cold_below_hot_below_adiabatic() {
    for (name, cmap) in shapes() {
        for g in [0.05f64, 0.15] {
            for r_m in [1.0f64, 5.0] {
                let ct = build(cmap, 0.0, g, r_m);
                let ad =
                    ct.adiabatic_excursion(&flight(), 1100.0, 1400.0, Some(&cmap), 0.05, 6.0)
                        .e_surge;
                let cold = ct
                    .soak_excursion(&flight(), 1100.0, 1400.0, Theta0::Cold, Some(&cmap), 0.05, 6.0)
                    .e_surge;
                let hot = ct
                    .soak_excursion(&flight(), 1100.0, 1400.0, Theta0::Hot, Some(&cmap), 0.05, 6.0)
                    .e_surge;
                assert!(
                    cold < hot && hot < ad,
                    "{name} G={g} r_m={r_m}: ordering broken — cold={cold} hot={hot} adiab={ad}"
                );
            }
        }
    }
}

// ------------------------------------------------------------------------------------ gate 6
/// GATE 6 — the PRIMARY heat-soak effect: the cold metal steals turbine work, so a cold
/// acceleration reaches its target speed LATER than the adiabatic one (the thrust-response lag),
/// and the lag grows with `G`; a hot reslam is ~adiabatic-fast because the metal RELEASES heat
/// early.
///
/// **`t_accel` is an `Option` and both arms are reachable** — § 5.14 probe 3 measured `None` on 4
/// of 24 cells at gate 5's `s_end = 6` and on none of gate 6's `s_end = 12`. Python orders a
/// `None` by substituting `1e9`; the `unwrap_or(f64::INFINITY)` here is that, spelled as what it
/// means.
#[test]
fn gate6_the_accel_time_lag() {
    let cmap = ComponentMap::surge_flow();
    let mut lags = Vec::new();
    for g in [0.05f64, 0.15] {
        let ct = build(cmap, 0.0, g, 3.0);
        let ad = ct.adiabatic_excursion(&flight(), 1100.0, 1400.0, Some(&cmap), 0.05, 12.0);
        let cold =
            ct.soak_excursion(&flight(), 1100.0, 1400.0, Theta0::Cold, Some(&cmap), 0.05, 12.0);
        let hot =
            ct.soak_excursion(&flight(), 1100.0, 1400.0, Theta0::Hot, Some(&cmap), 0.05, 12.0);
        let ad_t = ad.t_accel.expect("the adiabatic reference must reach 99 % within s_end");
        // "cold does not reach it as fast as adiabatic" — a `None` means EVEN SLOWER, so it passes.
        assert!(
            cold.t_accel.is_none() || cold.t_accel.unwrap() > ad_t,
            "cold accel must lag adiabatic at G={g}"
        );
        let hot_t = hot.t_accel.expect("the hot reslam must reach 99 % — it is ~adiabatic-fast");
        assert!(
            hot_t <= ad_t + 0.2,
            "hot reslam should be ~adiabatic-fast at G={g} ({hot_t} vs {ad_t})"
        );
        lags.push(cold.t_accel.unwrap_or(f64::INFINITY));
    }
    assert!(
        lags[1] >= lags[0],
        "the accel lag should grow with the heat-extraction gain G ({:?})", lags
    );
}

// ------------------------------------------------------------------------------------ gate 7
/// GATE 7 — the default design run is bit-for-bit rung 6; both effects are read-only extras on a
/// separate entry point.
///
/// **It is weaker than Python's, and the difference is written here rather than hidden.** Python
/// re-runs THE SAME engine object before and after, so its gate also covers aliasing — the
/// transient's `self.gas` IS the engine's, and an equilibrium gas carries a frozen station-4
/// mixture a constructor could reset. The Rust constructor CONSUMES its engine, so that half is
/// the compiler's and cannot be restated as a test. What remains testable is that a design run is
/// bit-reproducible across an intervening transient construction *and use*. Same repair, and same
/// reason, as `rung35.rs`'s gate 2.
#[test]
fn gate7_the_design_cycle_is_untouched() {
    let eng = design();
    let before = eng.run(&flight(), 1.0).performance.specific_thrust;
    let ct = build(ComponentMap::surge_flow(), 0.05, 0.1, 3.0);
    let _ = ct.plenum_frozen_peak(&flight(), 1100.0, 1400.0, None, 1.0 / 15.0);
    let _ = ct.soak_excursion(&flight(), 1100.0, 1400.0, Theta0::Cold, None, 0.05, 12.0);
    let after = eng.run(&flight(), 1.0).performance.specific_thrust;
    assert!(
        (after - before).abs() < 1e-12,
        "the combustor-dynamics diagnostics must not perturb the design run"
    );
}

// ============================================================ the three the PORT adds
//
// The counters are `thread_local!` and libtest gives every `#[test]` its own thread, so each of
// these sees only its own tallies even when the binary runs them concurrently. That is the
// property `spool.rs::counters::take`'s single-consumer caveat is about, and it is the reason
// these can be three tests rather than one.

/// **§ 5.14 PREDICTION 3, the count half.** The Illinois exhaustion arm is REACHED here, and at
/// exactly one call site.
///
/// Slice P shipped `try_illinois`'s `Ok(b)` with **zero** firings on its whole grid — measured, and
/// closed with a counter rather than a claim, because no value could see which endpoint it
/// returned. One rung later `_plenum_pt4_at` passes `N_TOL = 1e-12` as an ABSOLUTE bracket width
/// on a `pt4` of order 1e5 Pa, and **94.5 %** of that site's calls run out of iterations.
///
/// The gate is a CONTRAST, not a bare positive: the back-pressure invert is driven hard first and
/// exhausts nothing, then the pressure solve is driven and exhausts. A single positive count
/// would be satisfied by a port that exhausted everywhere.
#[test]
fn the_exhaustion_arm_is_reached_and_at_exactly_one_site() {
    let cmap = ComponentMap::surge_flow();
    let ct = build(cmap, 0.05, 0.0, 0.0);
    let fl = flight();
    let mf = ct.inner.fuel_for_tt4(&fl, 1400.0, Some(&cmap));
    let _ = scount::take();
    let _ = counters::take();

    // (a) the back-pressure invert, driven on its own at three speeds x three pressures.
    for nu in [0.85f64, 0.95, 1.0] {
        let (tt2, pt2, n, _) = ct.face(&fl, nu);
        let band = ct.pic_band(&cmap, n, tt2);
        for frac in [0.3f64, 0.5, 0.7] {
            let m = band.m_lo + frac * (band.m_hi - band.m_lo);
            let pt4 = ct.pic_of_m(&cmap, n, tt2, m).pi_c * ct.inner.inner.inner.pi_b * pt2;
            let _ = ct.try_compressor_from_backpressure(&cmap, n, tt2, pt2, pt4);
        }
    }
    let a = scount::take();
    let ka = counters::take();
    assert!(ka.backpressure_calls >= 9, "the invert did not run: {ka:?}");
    assert_eq!(
        a.illinois_exhausted, 0,
        "the back-pressure invert must NOT exhaust — it is written at a RELATIVE-scale tolerance \
         (1e-11 on an `m` of order 1); {} of {} calls did", a.illinois_exhausted, a.illinois_calls
    );

    // (b) the plenum pressure solve, which does.
    for nu in [0.85f64, 0.95, 1.0] {
        let _ = ct.try_plenum_pt4_at(&fl, nu, mf, &cmap);
    }
    let b = scount::take();
    let kb = counters::take();
    assert!(kb.pt4_at_calls >= 3, "the pressure solve did not run: {kb:?}");
    assert!(
        b.illinois_exhausted > 0,
        "`_plenum_pt4_at` passes an ABSOLUTE 1e-12 bracket width on a pt4 of order 1e5 Pa, so its \
         Illinois cannot converge and MUST fall out of the loop — 0 of {} calls did, which means \
         either the tolerance or the maxit arm was ported wrong",
        b.illinois_calls
    );
    // and the failure counters that § 5.14 predicted DEAD, gated against zero with a live sibling
    // in the same run rather than left absent:
    assert_eq!(ka.backpressure_bracket_fails + kb.backpressure_bracket_fails, 0);
    assert_eq!(ka.pt4_at_floor_fails + kb.pt4_at_floor_fails, 0);
}

/// **§ 5.14 PREDICTION 7.** The plenum instant reaches rung 34's turbine HOOK on every call and
/// the nozzle DISPATCH on none.
///
/// `_plenum_state` solves the choked `(★)` geometry and stops there — no `Nozzle`, no subsonic
/// re-solve, no `M9 > 0.985` escalation. So slice P's two rarest counters are STRUCTURALLY
/// unreachable from the plenum path, and quoting slice P's totals as coverage for this rung would
/// be reading a number off the wrong grid.
#[test]
fn the_plenum_instant_reaches_the_hook_and_never_the_nozzle() {
    let cmap = ComponentMap::surge_flow();
    let ct = build(cmap, 0.05, 0.0, 0.0);
    let fl = flight();
    let mf = ct.inner.fuel_for_tt4(&fl, 1400.0, Some(&cmap));
    let nu0 = ct.inner.equilibrium_fuel(&fl, mf, Some(&cmap)).nu;
    let pt4 = ct.plenum_pt4_at(&fl, nu0, mf, &cmap);
    let _ = scount::take();
    let _ = counters::take();

    for scale in [0.95f64, 0.98, 1.0, 1.02, 1.05] {
        let _ = ct.try_plenum_state(&fl, nu0, pt4 * scale, mf, &cmap);
    }
    let s = scount::take();
    let k = counters::take();
    assert_eq!(k.plenum_state_calls, 5);
    assert_eq!(
        s.r34_solve_turbine, k.plenum_state_calls,
        "the plenum instant must reach the rung-34 hook exactly ONCE per call"
    );
    assert_eq!(
        s.subsonic_fallbacks + s.subsonic_escalations, 0,
        "the plenum instant has no nozzle, so the subsonic solve cannot be reached from it"
    );
    assert_eq!(
        s.subsonic_escalations, 0,
        "nor can the M9 > 0.985 escalation guard — slice P's rarest branch is dead HERE"
    );
}

/// **§ 5.14 PREDICTIONS 4 AND 5.** Rung 37's marches have no `try`, and on the shipped grid
/// nothing inside them fails.
///
/// `SpoolTransient::march` `break`s out when an RK sub-stage leaves the valid region, which makes
/// trajectory LENGTH an output. All three of rung 37's marches run their step count
/// unconditionally and let a failure propagate — so fusing them with `march` would convert a raise
/// into a truncation. That difference is **LATENT**: nothing fails on this grid, which is exactly
/// why no value gate can see it and why the reachability is asserted as a count.
///
/// The soak closure's bracket failure is the discriminator. It is live from
/// [`CombustorTransient::equilibrium_soak`]'s march-in and dead from
/// [`CombustorTransient::soak_excursion`]'s RK stages — the same function under two callers, which
/// is the whole of *fallibility is per call site, not per function*.
#[test]
fn the_marches_run_to_length_with_no_bracket_failure() {
    let cmap = ComponentMap::surge_flow();
    let fl = flight();

    let ctp = build(cmap, 0.05, 0.0, 0.0);
    let _ = scount::take();
    let _ = counters::take();
    let r = ctp.plenum_frozen_peak(&fl, 1100.0, 1400.0, Some(&cmap), 1.0 / 15.0);
    let kp = counters::take();
    assert!(r.split_max > 0.05);
    // THE DECOMPOSITION IS THE GATE, and the first draft of it was wrong — it counted only the
    // march and came up 102 short of 703.
    //
    //   march:  151 recorded points (RK stage 1 each) + 3 more stages on each of the 150 steps;
    //   setup:  ONE `plenum_pt4_at` for the starting steady pressure — 2 bracket endpoints plus
    //           `ILLINOIS_MAXIT` = 100 residual evaluations, because THAT is the site whose
    //           absolute 1e-12 tolerance never converges. The exhaustion arm is visible right
    //           here, as an exact integer, in a gate that was not written to look for it.
    assert_eq!(kp.pt4_at_calls, 1, "the march sets its start pressure exactly once");
    assert_eq!(
        kp.plenum_state_calls, (151 + 3 * 150) + (2 + 100),
        "the plenum march did not run to its full length — a stage aborted, which in Python \
         would have RAISED rather than truncated (and `march`'s `break` would have HIDDEN)"
    );
    assert_eq!(kp.backpressure_bracket_fails, 0);

    let cts = build(cmap, 0.0, 0.1, 3.0);
    let _ = scount::take();
    let _ = counters::take();
    let e = cts.soak_excursion(&fl, 1100.0, 1400.0, Theta0::Cold, Some(&cmap), 0.05, 3.0);
    let ks = counters::take();
    assert!(e.e_surge > 0.0);
    let steps = 60i64; // s_end / ds = 3.0 / 0.05
    assert_eq!(
        ks.instant_soak_calls as i64, (steps + 1) + 3 * steps,
        "the soak march did not run to its full length"
    );
    assert_eq!(
        ks.soak_close_bracket_fails, 0,
        "the soak closure's bracket must never fail from INSIDE a march — Python has no `try` \
         there, so a failure would abort the whole excursion rather than shorten it"
    );

    // and the same function DOES fail from the other caller, which is what makes the zero above
    // evidence rather than silence.
    let _ = counters::take();
    let mf = cts.inner.fuel_for_tt4(&fl, 1400.0, Some(&cmap));
    let _ = cts.equilibrium_soak(&fl, mf, Some(&cmap));
    let ke = counters::take();
    assert!(
        ke.soak_close_bracket_fails > 0,
        "the soak closure's bracket MUST fail during `equilibrium_soak`'s march-in (the root \
         search probes speeds off the operable map); 0 of {} calls did", ke.soak_close_calls
    );
}
