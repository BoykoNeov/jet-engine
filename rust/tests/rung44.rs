//! RUNG 44 — THE TRANSIENT TWO-SPOOL SURGE LINE: the excursion is SCHEDULE-slaved, LP eats it.
//!
//! Port of `tests/test_rung44.py`, gate for gate. That file names **6 gates**, defines **8 test
//! functions** and collects **8 items** — no `parametrize` anywhere, so for once the three counts
//! reduce to two. § 5.15's opening said "9 + 7" for the two suites; step 1 corrected it to
//! **9 + 8** by collecting, and this file is the 8.
//!
//! **This file carries NINE `#[test]` fns, not eight.** The ninth is `rung41.rs`'s roster #2,
//! `test_reduce_transient_untouched_by_surge_line_bit_for_bit` — the LAST outstanding deferral
//! from slice L, due here. It lands in THIS file rather than in `rung40.rs` because what it gates
//! is a *surge line* left unread by a *transient*, which is rung 44's whole subject; slice P's
//! precedent (put the discharged item where its object lives) points here.
//!
//! Its six gates:
//!
//!   1. REDUCE — the rung-44 methods are READ-ONLY: arming `phi_surge` leaves rung 40's
//!      `integrate` / `equilibrium` / `jacobian` bit-for-bit. Plus: the default single-spool design
//!      run is bit-for-bit rung 6 (Python folds gate 6 into a second function).
//!   2. NON-TAUTOLOGICAL — an INDEPENDENT bare-math CPG two-shaft accel closure, with its OWN
//!      EULER march, reproduces the excursion SIGN, the LP-over-HP ordering, and the slaving.
//!   3. THE SPLIT SURVIVES DYNAMICALLY — `ext < 0` on both spools on accel (toward surge), `> 0`
//!      on decel; `|ext_lp| > 1.4 |ext_hp|` at every shape pair, the mode-free one included.
//!   4. SCHEDULE-SLAVED — `rho`-invariant, ramp-rate-monotone, mode-independent.
//!   5. REPORT THE CROSSING, GATE THE FLIP — transient min LP margin below the steady one on
//!      accel (above on decel); a floor in the gap ⇒ `crossed_lp` while the steady point clears.
//!   6. CYCLE UNTOUCHED — the default single-spool design path is bit-for-bit rung 6.
//!
//! # Where Rust has to say out loud what Python leaves implicit
//!
//! * **Three silent defaults.** `phi_excursion` and `transient_surge_margin` both default to
//!   `r_ramp = 0.5, s_end = 3.0, ds = 0.02` (`engine.py:3844`, `:3874`), and the suite leans on
//!   them at six of its nine call sites while naming `r_ramp` at the other three. Rust has no
//!   defaults, so every call below writes all three out. Step 2's lesson, second instance.
//! * **`==` on a returned record.** Python compares whole dicts; `Instant2` has a hand-written
//!   `PartialEq`, and the marched points / `PhiExcursion` have none — so those are compared field
//!   by field over `to_bits()`, which is STRICTER than Python's `==` (it separates `-0.0` from
//!   `0.0`).
//! * **`pytest.raises(AssertionError)`** on an unarmed map becomes `catch_unwind` around the
//!   `assert!` in `transient_surge_margin`.

use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::TwoSpoolTransientCore;

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;

// The CPG constants, written out ONCE so gate 2's reference can read them without touching `Gas`.
const GAMMA_C: f64 = 1.4;
const CP_C: f64 = 1004.0;
/// **DERIVED, not `286.9`.** `test_rung40.py` hard-codes `R_c = 286.9`; `test_rung44.py` writes
/// `R_c=(gamma_c-1.0)/gamma_c*cp_c` = `286.8571428571428`, and the two suites therefore run
/// DIFFERENT cold sections. Step 3 shipped `286.9` here, so every gate in this file ran rung 40's
/// gas — invisible, because every assertion below is a sign, an ordering or a spread, and step 3's
/// own value probe used `286.9` on BOTH sides so it could not see it either. Found at step 4, by
/// enumerating each suite's grid for the oracle instead of reading the constant off its neighbour.
const R_C: f64 = (GAMMA_C - 1.0) / GAMMA_C * CP_C;
const GAMMA_T: f64 = 1.3;
const CP_T: f64 = 1239.0;
const HPR: f64 = 42.8e6;

// The two-spool loss set, likewise — gate 2's reference reads these, not the shipped object.
const ETA_LPC: f64 = 0.90;
const ETA_HPC: f64 = 0.88;
const ETA_HPT: f64 = 0.92;
const ETA_LPT: f64 = 0.90;
const ETA_M: f64 = 0.99;
const ETA_B: f64 = 0.99;
const PI_N: f64 = 0.98;
const PI_D: f64 = 0.97;
const PI_B: f64 = 0.96;

/// Python's `phi_excursion` / `transient_surge_margin` defaults (`engine.py:3844`, `:3874`).
const DEF_R_RAMP: f64 = 0.5;
const DEF_S_END: f64 = 3.0;
const DEF_DS: f64 = 0.02;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn real() -> TwoSpoolLosses {
    TwoSpoolLosses {
        pi_d: PI_D, eta_lpc: ETA_LPC, eta_hpc: ETA_HPC, eta_b: ETA_B, pi_b: PI_B,
        eta_hpt: ETA_HPT, eta_lpt: ETA_LPT, eta_m: ETA_M, pi_n: PI_N,
        p_exit: None, nozzle_convergent: true,
    }
}

/// Self-consistent CPG dual gas (rung 31/38/39's recipe): `R_t = (g-1)/g*cp_t` exactly.
fn cpg_gas() -> Gas {
    Gas::new(GasSpec {
        gamma_c: GAMMA_C, cp_c: CP_C, r_c: R_C,
        gamma_t: GAMMA_T, cp_t: CP_T, r_t: (GAMMA_T - 1.0) / GAMMA_T * CP_T,
        hpr: HPR, ..GasSpec::default()
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

/// Rung 44's OWN shape roster — **five pairs, not rung 40's seven**: no all-flat pair and no
/// `lp-only`. In Python's dict order, because gate 4's `max` over the ratios is compared with `==`.
fn shapes() -> Vec<(&'static str, ComponentMap, ComponentMap)> {
    let m = |a: f64, b: f64, c: f64, sigma: f64, l: f64| ComponentMap {
        a, b, c, sigma, l, ..ComponentMap::flat()
    };
    let steep = m(0.25, 0.12, 0.0, 0.3, 1.2);
    vec![
        ("flow/press", lp_shaped(), hp_shaped()),
        ("press/flow", m(0.05, 0.20, 0.0, 0.1, 1.0), m(0.20, 0.05, 0.0, 0.1, 0.7)),
        ("tilted", tilted(), tilted()),
        ("steep", steep, steep),
        // rung 40's DISCRIMINATOR: LP FLAT => NO complex mode.
        ("hp-only", ComponentMap::flat(), hp_shaped()),
    ]
}

fn two_spool_design(gas: Gas) -> TwoSpoolEngine {
    build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, 50_000.0, real())
}

fn tt_rho(
    d: &TwoSpoolEngine, map_lp: ComponentMap, map_hp: ComponentMap, rho: f64,
) -> TwoSpoolTransientCore {
    TwoSpoolTransientCore::new(d.clone(), flight(), 1.0, map_lp, map_hp, rho)
}

/// The single-spool design the cycle-untouched gates read — `pi_c = 10`, `eta_c = 0.90`, and
/// **NOT** `nozzle_convergent` (rung 44's `SINGLE` omits it; rung 40's `single_design` sets it).
fn plain_single() -> Engine {
    build_turbojet(Gas::reacting_equilibrium(), 10.0, TT4, 50_000.0, Losses {
        pi_d: PI_D, eta_c: 0.90, eta_b: ETA_B, pi_b: PI_B, eta_t: ETA_HPT, eta_m: ETA_M,
        pi_n: PI_N, ..Losses::default()
    })
}

// ----------------------------------------------------------------------------------- gate 1
/// GATE 1 — the rung-44 methods are READ-ONLY: arming the surge line changes NOTHING upstream.
///
/// `phi_surge` is consumed by `transient_surge_margin` alone. Everything rung 40 shipped — the
/// marched trajectory, the 2-D equilibrium, the Jacobian — and rung 44's own running-line
/// excursion must come back bit-for-bit with the maps armed. Rung 44 adds no state.
///
/// Compared over `to_bits()` rather than `==`: `TwoSpoolTransientPoint` and `PhiExcursion` have no
/// `PartialEq` (they are plain data), and bits are the stricter reading of Python's `==` anyway.
#[test]
fn gate1_reduce_rung44_methods_are_read_only_bit_for_bit() {
    let d = two_spool_design(cpg_gas());
    for (ml, mh) in [(lp_shaped(), hp_shaped()), (tilted(), tilted())] {
        let bare = tt_rho(&d, ml, mh, 1.5);
        let armed = tt_rho(&d, ml.with_phi_surge(0.60), mh.with_phi_surge(0.55), 1.5);

        let sched = |s: f64| 1200.0 + 150.0 * 1.0f64.min(s / 0.5);
        let pa = bare.integrate(&flight(), sched, (0.95, 0.97), 2.0, 0.05);
        let pb = armed.integrate(&flight(), sched, (0.95, 0.97), 2.0, 0.05);
        assert_eq!(pa.len(), pb.len(), "the march LENGTH must be untouched too");
        for (a, b) in pa.iter().zip(pb.iter()) {
            for (k, x, y) in [
                ("nu_lp", a.nu_lp, b.nu_lp), ("nu_hp", a.nu_hp, b.nu_hp),
                ("phi_lp", a.phi_lp, b.phi_lp), ("phi_hp", a.phi_hp, b.phi_hp),
                ("pi_lpc", a.pi_lpc, b.pi_lpc), ("slip", a.slip, b.slip),
            ] {
                assert_eq!(x.to_bits(), y.to_bits(), "s={} {k}: {x} != {y}", a.s);
            }
        }

        for tt4 in [1500.0, 1100.0] {
            assert!(bare.equilibrium(&flight(), tt4) == armed.equilibrium(&flight(), tt4),
                    "equilibrium moved at Tt4={tt4}");
            let (ja, jb) = (bare.jacobian(&flight(), tt4, None, 1e-6),
                            armed.jacobian(&flight(), tt4, None, 1e-6));
            for i in 0..2 {
                for j in 0..2 {
                    assert_eq!(ja[i][j].to_bits(), jb[i][j].to_bits(),
                               "Tt4={tt4} J[{i}][{j}]: {} != {}", ja[i][j], jb[i][j]);
                }
            }
        }

        // The running-line excursion itself is identical — the trajectory never reads phi_surge.
        let ea = bare.phi_excursion(&flight(), 1000.0, 300.0, DEF_R_RAMP, DEF_S_END, DEF_DS);
        let eb = armed.phi_excursion(&flight(), 1000.0, 300.0, DEF_R_RAMP, DEF_S_END, DEF_DS);
        for (k, x, y) in [
            ("ext_lp", ea.ext_lp, eb.ext_lp), ("ext_hp", ea.ext_hp, eb.ext_hp),
            ("s_lp", ea.s_lp, eb.s_lp), ("s_hp", ea.s_hp, eb.s_hp),
            ("min_phi_lp", ea.min_phi_lp, eb.min_phi_lp),
            ("min_phi_hp", ea.min_phi_hp, eb.min_phi_hp), ("ratio", ea.ratio, eb.ratio),
        ] {
            assert_eq!(x.to_bits(), y.to_bits(), "phi_excursion {k}: {x} != {y}");
        }
        assert_eq!(ea.npts, eb.npts);
    }
}

/// GATE 1/6 — the default single-spool design path is untouched (bit-for-bit rung 6).
///
/// **ONE engine object, run either side of the diagnostics — Python's shape, kept deliberately.**
/// Rebuilding it between the two runs would be the weaker test: it can still see a global or
/// thread-local that the diagnostics disturb, but not a mutation of the engine itself, and the
/// shared object is the channel this gate exists for. (Rust's `run(&self)` makes that channel
/// hard to open in the first place, so the gate is thinner here than in Python — worth saying,
/// rather than letting the green tick imply otherwise.)
#[test]
fn gate1b_default_design_run_bit_for_bit_rung6() {
    let eng = plain_single();
    let a = eng.run(&flight(), 1.0);
    // Constructing and exercising the rung-44 diagnostics must not perturb it.
    let d = two_spool_design(cpg_gas());
    let t = tt_rho(&d, lp_shaped().with_phi_surge(0.60), hp_shaped().with_phi_surge(0.55), 1.0);
    t.phi_excursion(&flight(), 1000.0, 300.0, DEF_R_RAMP, DEF_S_END, DEF_DS);
    t.transient_surge_margin(&flight(), 1000.0, 300.0, DEF_R_RAMP, DEF_S_END, DEF_DS);
    let b = eng.run(&flight(), 1.0);
    assert_eq!(a.performance.specific_thrust.to_bits(), b.performance.specific_thrust.to_bits());
    assert_eq!(a.station("4").far.to_bits(), b.station("4").far.to_bits());
}

// ----------------------------------------------------------------------------------- gate 3
/// GATE 3 — THE SPLIT SURVIVES DYNAMICALLY: accel drives BOTH spools toward surge, LP eats more.
///
/// Sign and ordering only — every magnitude rides on the maps and the ramp, and is disclaimed.
/// The `hp-only` pair is in the roster precisely because it carries NO complex mode: if the
/// asymmetry needed the mode, this pair would be the one that broke.
#[test]
fn gate3_split_survives_dynamically() {
    let d = two_spool_design(cpg_gas());
    for (name, ml, mh) in shapes() {
        let t = tt_rho(&d, ml, mh, 1.0);
        let acc = t.phi_excursion(&flight(), 1000.0, 400.0, DEF_R_RAMP, DEF_S_END, DEF_DS);
        let dec = t.phi_excursion(&flight(), 1400.0, -400.0, DEF_R_RAMP, DEF_S_END, DEF_DS);
        assert!(acc.ext_lp < 0.0 && acc.ext_hp < 0.0,
                "{name}: accel must swing BOTH spools toward surge: {acc:?}");
        assert!(dec.ext_lp > 0.0 && dec.ext_hp > 0.0,
                "{name}: decel must swing BOTH away: {dec:?}");
        assert!(acc.ext_lp.abs() > 1.4 * acc.ext_hp.abs(),
                "{name}: LP eats more: {acc:?}");
    }
}

// ----------------------------------------------------------------------------------- gate 4
/// GATE 4 (a) — the accel excursion is `rho`-INVARIANT over a 25× range.
///
/// Rung 40's inter-spool clock ratio is POWERLESS over the surge excursion: that rung's own
/// scope-limit, read on the surge axis.
///
/// **AN INVARIANCE CLAIM IS SATISFIED BEST BY DELETING THE VARIABLE, so it needs the other
/// direction too.** Python asserts only `spread < 0.05`. Deleting `rho` from the shipped marcher
/// outright (`i.phi_lp_dot / self.rho` → `i.phi_lp_dot`) sends the spread to exactly ZERO and
/// passes this gate *more* comfortably — measured, and invisible to all nine of this file's gates;
/// what caught it was `rung40.rs::gate7_scope_sigma_crit_is_first_instant_only`, a rung away. The
/// `lo < hi` bar below is the missing half: `rho` must be READ (a live spread) and yet POWERLESS
/// (a small one). Two bars, because "powerless" and "absent" are otherwise the same reading.
#[test]
fn gate4a_excursion_is_rho_invariant() {
    let d = two_spool_design(cpg_gas());
    let vals: Vec<f64> = [0.2, 0.5, 1.0, 2.0, 5.0].iter().map(|&rho| {
        tt_rho(&d, lp_shaped(), hp_shaped(), rho)
            .phi_excursion(&flight(), 1000.0, 400.0, DEF_R_RAMP, DEF_S_END, DEF_DS).ext_lp
    }).collect();
    let lo = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mean = vals.iter().sum::<f64>() / vals.len() as f64;
    let spread = (hi - lo) / mean.abs();
    assert!(spread < 0.05, "rho-invariant: {vals:?} spread={spread}");
    // ...and rho is READ. Without this, deleting it from the marcher passes the bar above.
    assert!(lo < hi, "rho must be LIVE and merely powerless — a spread of exactly zero means \
                      the marcher stopped reading it at all: {vals:?}");
}

/// GATE 4 (b) — the excursion is RAMP-RATE-driven: `|ext_lp|` rises monotonically as `r_ramp`
/// falls. The schedule against the shaft clock is the governing variable — not `rho`, not the mode.
///
/// `s_end = 6.0` here (the slow ramps need the extra length) but `ds` is left at its **0.02**
/// default: Python names one keyword and defaults the other in the same call.
#[test]
fn gate4b_excursion_is_ramp_rate_driven() {
    let d = two_spool_design(cpg_gas());
    let t = tt_rho(&d, lp_shaped(), hp_shaped(), 1.0);
    let mut prev: Option<f64> = None;
    for r in [5.0, 2.0, 1.0, 0.5, 0.3, 0.1] {
        let e = t.phi_excursion(&flight(), 1000.0, 400.0, r, 6.0, DEF_DS).ext_lp.abs();
        if let Some(p) = prev {
            assert!(e > p, "faster ramp => deeper excursion: r={r} {e} !> {p}");
        }
        prev = Some(e);
    }
}

/// GATE 4 (c) — the complex mode is SURGE-IRRELEVANT.
///
/// The AIRTIGHT leg is the damping ratio: every `|Im/Re|max < 0.25`, so the ring e-folds before a
/// quarter cycle and cannot carry the point across a line the steady position clears. The
/// CORROBORATION (explicitly not a proof, in Python's own docstring) is that the mode-FREE pair
/// carries the LARGEST LP/HP ratio — the asymmetry is there with no mode to cause it.
#[test]
fn gate4c_complex_mode_is_surge_irrelevant() {
    let d = two_spool_design(cpg_gas());
    let mut ratios: Vec<(&str, f64)> = Vec::new();
    for (name, ml, mh) in shapes() {
        let t = tt_rho(&d, ml, mh, 1.0);
        let acc = t.phi_excursion(&flight(), 1000.0, 400.0, DEF_R_RAMP, DEF_S_END, DEF_DS);
        ratios.push((name, acc.ext_lp.abs() / acc.ext_hp.abs()));
        assert!(t.damping_ratio_max(&flight(), 1200.0, None) < 0.25,
                "{name}: the ring must e-fold fast");
        let band = t.oscillatory_band(&flight(), 1200.0, None);
        if name == "hp-only" {
            assert!(band.is_none(), "hp-only (LP flat) must carry NO complex mode: {band:?}");
        } else {
            assert!(band.is_some(), "{name}: a shaped-LP pair must carry a complex mode");
        }
    }
    let biggest = ratios.iter().map(|&(_, r)| r).fold(f64::NEG_INFINITY, f64::max);
    let hp_only = ratios.iter().find(|&&(n, _)| n == "hp-only").expect("hp-only in the roster").1;
    assert!(hp_only == biggest, "the mode-FREE pair must eat the most: {ratios:?}");
}

// ----------------------------------------------------------------------------------- gate 5
/// GATE 5 — REPORT THE CROSSING, GATE THE FLIP (rung 36's discipline, one rung on).
///
/// `transient_surge_margin` ALLOWS `phi < phi_surge` and records it rather than asserting against
/// it. What is GATED is the flip's SIGN: on an accel the transient minimum LP margin sits strictly
/// below the steady minimum at the same `Tt4`; the decel is the mirror. The crossing DEPTH is
/// reported, never gated. The LP floor `0.76` is placed in the gap so the transient crosses while
/// every steady point clears — and it lands on the LP spool, not the HP.
///
/// The last leg is Python's `pytest.raises(AssertionError)`: unarmed maps must make the method
/// refuse, so a surge line that is genuinely absent can never be read as a margin of zero.
#[test]
fn gate5_report_the_crossing_gate_the_flip() {
    let d = two_spool_design(cpg_gas());
    let (ml, mh) = (lp_shaped().with_phi_surge(0.76), hp_shaped().with_phi_surge(0.55));
    let t = tt_rho(&d, ml, mh, 1.0);

    // THE FLIP (sign): transient min LP margin strictly below the steady min LP margin on accel.
    let acc = t.transient_surge_margin(&flight(), 1000.0, 400.0, 0.3, DEF_S_END, DEF_DS);
    assert!(acc.margin_min_lp < acc.steady_min_lp, "accel flip (LP toward surge): {acc:?}");
    let dec = t.transient_surge_margin(&flight(), 1400.0, -400.0, 0.3, DEF_S_END, DEF_DS);
    assert!(dec.margin_min_lp > dec.steady_min_lp, "decel flip (LP away): {dec:?}");

    // THE CROSSING (reported, not gated as a magnitude): the floor sits IN the gap.
    assert!(acc.steady_min_lp > 0.0, "the steady running line CLEARS the floor: {acc:?}");
    assert!(acc.crossed_lp && !acc.crossed_hp,
            "the transient crossing lands on the LP spool: {acc:?}");

    // Unarmed maps => the method refuses (Python raises AssertionError; here it panics).
    let bare = tt_rho(&d, lp_shaped(), hp_shaped(), 1.0);
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let refused = catch_unwind(AssertUnwindSafe(|| {
        bare.transient_surge_margin(&flight(), 1000.0, 400.0, DEF_R_RAMP, DEF_S_END, DEF_DS)
    })).is_err();
    std::panic::set_hook(hook);
    assert!(refused, "an unarmed map must make transient_surge_margin refuse, not report 0");
}

// ----------------------------------------------------------------------------------- gate 2
/// Python's `bisect` — 400 halvings to `1e-15`, NOT the crate's Illinois solver.
fn bare_bisect(f: &dyn Fn(f64) -> f64, mut lo: f64, mut hi: f64) -> f64 {
    let mut flo = f(lo);
    assert!(flo * f(hi) < 0.0, "bare bracket fails");
    for _ in 0..400 {
        let mid = 0.5 * (lo + hi);
        let fm = f(mid);
        if flo * fm <= 0.0 { hi = mid; } else { lo = mid; flo = fm; }
        if hi - lo < 1e-15 { break; }
    }
    0.5 * (lo + hi)
}

/// Python's builtin `round(x, 3)`, written out here rather than imported from the crate: the
/// reference is supposed to be independent of `two_spool_transient`, and its `round3` lives there.
/// The rounding is LIVE, not cosmetic — the rounded value is the `Tt4` the steady solve runs at.
fn bare_round3(x: f64) -> f64 {
    format!("{x:.3}").parse::<f64>().expect("formatted float parses")
}

/// The loading law, read off the map's plain FIELDS — no `ComponentMap` method is called anywhere
/// inside the reference.
fn bare_psi(cm: &ComponentMap, phi: f64) -> f64 {
    1.0 - cm.sigma * (phi - 1.0).powi(2) - cm.l * (phi - 1.0)
}

/// The efficiency island, likewise from fields only.
fn bare_eta_at(cm: &ComponentMap, base: f64, phi: f64, n: f64) -> f64 {
    base - cm.a * (phi - 1.0).powi(2) - cm.b * (n - 1.0).powi(2)
        - cm.c * (phi - 1.0) * (n - 1.0)
}

/// One evaluation of the bare forward closure at a trial LP-face flow — Python's `ev` dict.
struct BareEv {
    m_imp: f64,
    tt25: f64,
    tt3: f64,
    phi_l: f64,
    phi_h: f64,
    f: f64,
}

/// GATE 2 — an INDEPENDENT bare-math CPG two-shaft closure, marched by EULER.
///
/// Nothing from `Gas` / the components / `ComponentMap`'s methods / `TwoSpoolTransient` is inside
/// the reference: closed-form CPG thermodynamics, its own choke bisection, its own FORWARD speed
/// lines, its own 2-D damped-Newton running line, and its own EULER shaft march against the same
/// ramp. It reads only the three throat AREAS and the maps' plain fields off the shipped objects —
/// the line Python draws.
///
/// **The integrators DIFFER on purpose** — Euler here, RK4 in the shipped march — so what is tied
/// is the SIGN, the LP-over-HP ordering and the qualitative slaving, plus one loose 25% tie to the
/// shipped value. Rung 40 gate 3's pattern, on a marched object instead of a fixed point. Tightening
/// it toward bit-equality would mean copying the shipped instruction sequence, which is exactly the
/// independence the gate exists to have (§ 5.4's COPY vs REDERIVATION).
#[test]
fn gate2_bare_math_accel_excursion_sign_ordering_and_slaving() {
    let (map_lp, map_hp) = (lp_shaped(), hp_shaped());
    let d = two_spool_design(cpg_gas());
    let t = tt_rho(&d, map_lp, map_hp, 1.0);

    let gc = (GAMMA_C - 1.0) / GAMMA_C;
    let gt = (GAMMA_T - 1.0) / GAMMA_T;

    let stag = 1.0 + 0.5 * (GAMMA_C - 1.0) * 0.85f64.powi(2);
    let tt2 = 250.0 * stag;
    let tt25_d = tt2 * (1.0 + (PI_LPC.powf(gc) - 1.0) / ETA_LPC);
    let tt3_d = tt25_d * (1.0 + (PI_HPC.powf(gc) - 1.0) / ETA_HPC);
    let f_d = (CP_T * TT4 - CP_C * tt3_d) / (ETA_B * HPR - CP_T * TT4);
    let (tau_lpc_d, tau_hpc_d) = (tt25_d / tt2, tt3_d / tt25_d);
    let p_ref_lp = CP_C * (tt25_d - tt2);
    // Python writes this as `cp_c * (Tt3_d - Tt2 * tau_lpc_d)` — algebraically `Tt3_d - Tt25_d`,
    // but kept in Python's spelling rather than simplified (§ 5.4's rule: do not factor away a
    // deliberate duplication, because the second derivation is what the gate is for).
    let p_ref_hp = CP_C * (tt3_d - tt2 * tau_lpc_d);

    let turbine = |area_ratio: f64, eta_t: f64| -> f64 {
        let tau = move |p: f64| 1.0 - eta_t * (1.0 - p.powf(gt));
        let pi_t = bare_bisect(&|p: f64| p / tau(p).sqrt() - area_ratio, 0.02, 0.999);
        tau(pi_t)
    };

    let b = &t.inner.base;
    let (area_hp, area_lp) = (b.a4 / b.a45, b.a45 / (b.a8 * PI_N));
    let tau_hpt = turbine(area_hp, ETA_HPT);
    let tau_lpt = turbine(area_lp, ETA_LPT);

    let phis = |nu_l: f64, nu_h: f64, tt4: f64| -> (f64, f64, BareEv) {
        let ev = |m_l: f64| -> BareEv {
            let phi_l = m_l / nu_l;                  // Tt2 == Tt2_d  =>  n_L == nu_L
            let tau_lpc = 1.0 + (tau_lpc_d - 1.0) * bare_psi(&map_lp, phi_l) * nu_l * nu_l;
            let tt25 = tt2 * tau_lpc;
            let e_l = bare_eta_at(&map_lp, ETA_LPC, phi_l, nu_l);
            let pi_lpc = (1.0 + e_l * (tau_lpc - 1.0)).powf(1.0 / gc);
            let m_h = m_l * (PI_LPC / pi_lpc) * (tt25 / tt25_d).sqrt();
            let n_h = nu_h * (tt25_d / tt25).sqrt();
            let phi_h = m_h / n_h;
            let tau_hpc = 1.0 + (tau_hpc_d - 1.0) * bare_psi(&map_hp, phi_h) * n_h * n_h;
            let tt3 = tt25 * tau_hpc;
            let e_h = bare_eta_at(&map_hp, ETA_HPC, phi_h, n_h);
            let pi_hpc = (1.0 + e_h * (tau_hpc - 1.0)).powf(1.0 / gc);
            let f = (CP_T * tt4 - CP_C * tt3) / (ETA_B * HPR - CP_T * tt4);
            let m_imp = (pi_lpc * pi_hpc / (PI_LPC * PI_HPC)) * (TT4 / tt4).sqrt()
                * (1.0 + f_d) / (1.0 + f);
            BareEv { m_imp, tt25, tt3, phi_l, phi_h, f }
        };
        let m_l = bare_bisect(&|m: f64| m - ev(m).m_imp, 0.05, 1.6);
        let s = ev(m_l);
        let tt45 = tt4 * tau_hpt;
        let tt5 = tt4 * tau_hpt * tau_lpt;
        let f = s.f;
        let pt_hp = ETA_M * (1.0 + f) * CP_T * (tt4 - tt45);
        let pt_lp = ETA_M * (1.0 + f) * CP_T * (tt45 - tt5);
        let pc_hp = CP_C * (s.tt3 - s.tt25);
        let pc_lp = CP_C * (s.tt25 - tt2);
        (m_l * (pt_lp - pc_lp) / (p_ref_lp * nu_l),
         m_l * (pt_hp - pc_hp) / (p_ref_hp * nu_h),
         s)
    };

    let equilibrium = |tt4: f64| -> (f64, f64, BareEv) {
        let (mut nl, mut nh) = (1.0f64, 1.0f64);
        for _ in 0..120 {
            let (fl, fh, s) = phis(nl, nh, tt4);
            if fl.abs().max(fh.abs()) < 1e-13 {
                return (nl, nh, s);
            }
            let h = 1e-6;
            let (al, ah, _) = phis(nl + h, nh, tt4);
            let (bl, bh, _) = phis(nl, nh + h, tt4);
            let (j11, j12, j21, j22) = ((al - fl) / h, (bl - fl) / h, (ah - fh) / h, (bh - fh) / h);
            let det = j11 * j22 - j12 * j21;
            let dl = (-fl * j22 + fh * j12) / det;
            let dh = (-j11 * fh + j21 * fl) / det;
            let damp = 1.0f64.min(0.25 / dl.abs().max(dh.abs()).max(1e-30));
            nl += damp * dl;
            nh += damp * dh;
        }
        panic!("bare 2-D equilibrium did not converge");
    };

    // Python's `_cache={}` default argument: ONE cache shared by every `excursion` call in this
    // test, keyed on the ROUNDED Tt4 — which is also the Tt4 the steady solve is run at, so the
    // rounding is part of the physics and not just a memo key.
    let mut steady_cache: HashMap<u64, (f64, f64)> = HashMap::new();

    let mut excursion = |tt4_lo: f64, dtt4: f64, r_ramp: f64, rho: f64| -> (f64, f64) {
        let s_end = 6.0f64;
        let ds = 0.01f64;
        let (mut nl, mut nh, _) = equilibrium(tt4_lo);
        let (mut ext_l, mut ext_h) = (0.0f64, 0.0f64);
        let mut s = 0.0f64;
        let n = (s_end / ds).round_ties_even() as usize;
        for _ in 0..=n {
            let tt4 = tt4_lo + dtt4 * 1.0f64.min(s / r_ramp);
            let (phi_l_dot, phi_h_dot, sd) = phis(nl, nh, tt4);
            let key = bare_round3(tt4);
            let (pl, ph) = *steady_cache.entry(key.to_bits()).or_insert_with(|| {
                let (_, _, se) = equilibrium(key);
                (se.phi_l, se.phi_h)
            });
            let (e_l, e_h) = (sd.phi_l - pl, sd.phi_h - ph);
            if e_l.abs() > ext_l.abs() { ext_l = e_l; }
            if e_h.abs() > ext_h.abs() { ext_h = e_h; }
            nl = 0.2f64.max(nl + ds * phi_l_dot / rho);
            nh = 0.2f64.max(nh + ds * phi_h_dot);
            s += ds;
        }
        (ext_l, ext_h)
    };

    // SIGN + ordering (accel toward surge, LP eats more).
    let (el, eh) = excursion(1000.0, 400.0, 0.5, 1.0);
    assert!(el < 0.0 && eh < 0.0, "bare accel toward surge: {el} {eh}");
    assert!(el.abs() > 1.4 * eh.abs(), "bare LP eats more: {el} {eh}");
    // Decel mirror.
    let (dl, dh) = excursion(1400.0, -400.0, 0.5, 1.0);
    assert!(dl > 0.0 && dh > 0.0, "bare decel away from surge: {dl} {dh}");

    // SCHEDULE-SLAVING: rho-invariance.
    let rho_vals: Vec<f64> =
        [0.2, 1.0, 5.0].iter().map(|&r| excursion(1000.0, 400.0, 0.5, r).0).collect();
    let lo = rho_vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = rho_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mean = rho_vals.iter().sum::<f64>() / rho_vals.len() as f64;
    let spread = (hi - lo) / mean.abs();
    assert!(spread < 0.05, "bare rho-invariance: {rho_vals:?} spread={spread}");
    // The same second bar as gate 4 (a), on the reference: dropping `/ rho` from the Euler step
    // below sends this spread to exactly zero, which SATISFIES the line above. Measured.
    assert!(lo < hi, "the bare march must READ rho: {rho_vals:?}");

    // SCHEDULE-SLAVING: ramp-rate monotonicity.
    let fast = excursion(1000.0, 400.0, 0.2, 1.0).0.abs();
    let slow = excursion(1000.0, 400.0, 2.0, 1.0).0.abs();
    assert!(fast > slow, "bare faster ramp => deeper: {fast} !> {slow}");

    // The loose tie to the shipped object (Euler vs RK4): same sign, within 25%.
    let ship = t.phi_excursion(&flight(), 1000.0, 400.0, 0.5, DEF_S_END, DEF_DS).ext_lp;
    assert!((el - ship).abs() < 0.25 * ship.abs(), "bare ~ shipped: {el} {ship}");
}

// ------------------------------------------------------- the slice-L deferral, discharged here
/// **`rung41.rs`'s roster #2, DISCHARGED** — `tests/test_rung41.py`'s
/// `test_reduce_transient_untouched_by_surge_line_bit_for_bit`, the last outstanding item from
/// slice L. It deferred because it reaches `TwoSpoolTransient`, which did not exist until step 1
/// of this slice.
///
/// It is rung 41's reduce read one rung on: rung 40's two-shaft transient never touches
/// `phi_surge`, so the closure, the equilibrium and the residuals are bit-for-bit unchanged by
/// arming the surge line.
///
/// **It is a strict SUBSET of `gate1_...` apart from the floor value** (`0.55` on both maps, where
/// rung 44 uses `0.60`/`0.55`) — `Instant2`'s hand-written `PartialEq` already compares
/// `phi_lp_dot`/`phi_hp_dot` and `close.mdot_air` explicitly, so naming them here adds a label, not
/// coverage. It is kept as its own function anyway so the roster that discharges it has something
/// to point at; claiming more than that would be the over-statement rung 63 shipped and had to
/// correct.
///
/// **Its closing line stays WITHDRAWN.** Python ends with
/// `ComponentMap.flat().with_phi_surge(0.6).is_flat()`; § 5.8.2 (b) withdrew that from the port
/// because a Rust `is_flat` would be Python's predicate minus its `vsv` conjunct, and there is no
/// flat-reduce BRANCH here for it to guard. Its content is gated as a value in
/// `rung41.rs::gate1_surge_line_is_a_pure_diagnostic`, which is where it stays.
#[test]
fn rung41_deferred_transient_untouched_by_surge_line_bit_for_bit() {
    let d = two_spool_design(cpg_gas());
    let bare = tt_rho(&d, lp_shaped(), hp_shaped(), 1.5);
    let armed =
        tt_rho(&d, lp_shaped().with_phi_surge(0.55), hp_shaped().with_phi_surge(0.55), 1.5);
    for tt4 in [1500.0, 1100.0] {
        let (a, b) = (bare.equilibrium(&flight(), tt4), armed.equilibrium(&flight(), tt4));
        for (k, x, y) in [
            ("nu_lp", a.nu_lp, b.nu_lp), ("nu_hp", a.nu_hp, b.nu_hp),
            ("pi_lpc", a.close.pi_lpc, b.close.pi_lpc),
            ("pi_hpc", a.close.pi_hpc, b.close.pi_hpc),
            ("Phi_lp", a.phi_lp_dot, b.phi_lp_dot), ("Phi_hp", a.phi_hp_dot, b.phi_hp_dot),
            ("mdot_air", a.close.mdot_air, b.close.mdot_air),
        ] {
            assert_eq!(x.to_bits(), y.to_bits(), "Tt4={tt4} {k}: {x} != {y}");
        }
    }
}
