//! The φ-RATE LIMITER — investigated, NEGATIVE (`docs/phi-rate-limiter-negative.md`).
//!
//! The Rust port of `tests/test_phi_rate_limiter_negative.py` (slice AT — plan § 8.1 (v): *"a
//! real gap. CLAUDE.md calls it the one negative with a gate; no Rust file names it"*). Four gates,
//! 1:1, in file order, plus one declared addition.
//!
//! Rung 60's seam asked for a leg that caps `dφ/ds` rather than `φ`. No fuel-side leg can: fuel's
//! authority over φ INVERTS between the level and the derivative.
//!
//! ```text
//! level:  more fuel -> hotter Tt4 -> less choked-NGV capacity -> less phi     [DIRECT]
//! rate:   less fuel -> cooler Tt4 -> less shaft accel -> the STATE term dies  [INDIRECT]
//! ```
//!
//! **WHY THIS NEGATIVE CARRIES A GATE.** It BOUNDS a claim six shipped rungs rest on: rung 49's
//! bracket — "φ falls monotonically with fuel, so cutting fuel RAISES φ" — is load-bearing under
//! rungs 49, 50, 51, 52, 58 and 60, and it is a LEVEL property that reverses one derivative up.
//! No per-rung gate looks at a derivative, so a change to the fuel → `Tt4` → shaft-acceleration
//! channel could flip the sign unobserved. Both halves are pinned in ONE file on purpose: rung
//! 49's monotonicity must HOLD and the rate inversion must ALSO hold.
//!
//! **THE LOCAL LEG.** Python subclasses `ScheduledStatorTransient` with a `rate_of` method that
//! lives in the test, not in `engine.py` — formulation A is unrealisable, so nothing shipped. Here
//! it is a free function over the bare rung-57 machine (no stator armed, so its instant is rung
//! 43's). Python catches `AssertionError` where the plant leaves its modeled speed-line region;
//! the fallible twin [`FuelTransientCore::try_instant_fuel`] returns that as an `Err`, and
//! [`rate_of`] propagates it, so the walk in gate 4 skips exactly the cuts Python skips. The
//! declared addition [`the_walk_is_cut_where_the_python_cut_it`] pins that, because a twin that
//! panicked on one path or returned on another would move the count while every bar still passed.
//!
//! Python's module fixture is rebuilt per test here: one coarse ramp of 26 points, cheap.

use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{FuelLimiters, FuelPoint, FuelTransientCore};
use turbojet::gas::{Abort, Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{ScheduledStatorTransient, StatorArm};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolLosses};

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const R: f64 = 0.5;
const DS: f64 = 0.02;
const RHO: f64 = 1.0;
/// On the DESCENT of both spools, before either minimum (bare min `phi_lp` at `s ≈ 0.235`,
/// `phi_hp` at `s ≈ 0.390` — the doc's § 2).
const SAMPLES: [f64; 2] = [0.10, 0.20];

#[derive(Clone, Copy, Debug, PartialEq)]
enum Key {
    Lp,
    Hp,
}
const KEYS: [Key; 2] = [Key::Lp, Key::Hp];

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

/// Python's `_cpg` — `R` DERIVED as `(g-1)/g*cp`.
fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

struct Rig {
    m: FuelTransientCore,
    traj: Vec<FuelPoint>,
    slope: f64,
}

/// Python's `rig` fixture: (machine, trajectory, schedule slope) — one coarse bare ramp.
fn rig() -> Rig {
    let lp = ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR);
    let hp = ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR);
    let design = build_two_spool_turbojet(cpg(), PI_LPC, PI_HPC, TT4, flight().p0, REAL);
    let m = match ScheduledStatorTransient::new(
        design, flight(), 1.0, Some(lp), Some(hp), RHO, StatorArm::default(),
    ) {
        ScheduledStatorTransient::Full(c) => c.fuel,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("LP is not disabled"),
    };
    let f = flight();
    let (mf_lo, mf_hi) = (m.fuel_for_tt4(&f, LO), m.fuel_for_tt4(&f, HI));
    let slope = (mf_hi - mf_lo) / R;
    let sched = |s: f64| {
        if s <= 0.0 {
            mf_lo
        } else if s >= R {
            mf_hi
        } else {
            mf_lo + (mf_hi - mf_lo) * (s / R)
        }
    };
    let eq0 = m.inner.equilibrium(&f, LO);
    let traj = m.integrate_fuel(&f, sched, (eq0.nu_lp, eq0.nu_hp), R, DS, &FuelLimiters::default());

    // THE SHARED PRECONDITION. Every gate frames its claim on the DESCENT, so both samples must
    // sit before BOTH minima; `s = 0.20` is only ~1.5 cells clear of LP's at this `ds`.
    let rig = Rig { m, traj, slope };
    for key in KEYS {
        for s in SAMPLES {
            let p = at(&rig.traj, s);
            let r = rate_of(&rig.m, p.nu_lp, p.nu_hp, p.mf, rig.slope, key).unwrap();
            assert!(
                r < 0.0,
                "sample s={s} is NOT on the {key:?} descent (dphi/ds = {r:+.6} at s={:.4}). Move \
                 SAMPLES earlier -- every gate in this file frames its claim on the descent.",
                p.s
            );
        }
    }
    rig
}

/// Python's `_at`: the trajectory point nearest `s` (the FIRST, on a tie, as `min` takes it).
fn at(traj: &[FuelPoint], s: f64) -> &FuelPoint {
    let mut best = &traj[0];
    for p in traj {
        if (p.s - s).abs() < (best.s - s).abs() {
            best = p;
        }
    }
    best
}

/// `φ` on a spool at the quasi-steady instant `(ν_L, ν_H, w)`.
fn phi(m: &FuelTransientCore, a: f64, b: f64, w: f64, key: Key) -> Result<f64, Abort> {
    let i = m.try_instant_fuel(&flight(), a, b, w)?;
    Ok(match key {
        Key::Lp => i.base.close.phi_lp,
        Key::Hp => i.base.close.phi_hp,
    })
}

/// Python's `_RateProbe.rate_of`: `dφ/ds` by the chain rule,
/// `φ_νL·ν̇_L + φ_νH·ν̇_H + φ_mf·ṁf`. `ν̇` is what the instant already returns, which is why
/// formulation A needs no state (the doc's § 1). Same term order as the Python.
fn rate_of(m: &FuelTransientCore, a: f64, b: f64, w: f64, slope: f64, key: Key)
    -> Result<f64, Abort>
{
    let f = flight();
    let i = m.try_instant_fuel(&f, a, b, w)?;
    let i_key = match key {
        Key::Lp => i.base.close.phi_lp,
        Key::Hp => i.base.close.phi_hp,
    };
    let h = 1e-6;
    let pa = (phi(m, a + h, b, w, key)? - i_key) / h;
    let pb = (phi(m, a, b + h, w, key)? - i_key) / h;
    let hw = 1e-6 * 1.0f64.max(w.abs());
    let pw = (phi(m, a, b, w + hw, key)? - i_key) / hw;
    Ok(pa * (i.base.phi_lp_dot / RHO) + pb * i.base.phi_hp_dot + pw * slope)
}

// =============================================================================================
// HALF ONE — rung 49's LEVEL monotonicity, which must HOLD
// =============================================================================================

/// Cutting fuel RAISES φ, on both spools, at every sampled point — the bracket `_surge_fuel`
/// relies on, and rungs 49/50/51/52/58/60 ride on it. SOUND: the negative bounds it, it does not
/// refute it.
#[test]
fn rung49_level_monotonicity_holds() {
    let g = rig();
    for key in KEYS {
        for s in SAMPLES {
            let p = at(&g.traj, s);
            let mut prev: Option<f64> = None;
            for k in 0..5 {
                let frac = 0.97f64.powi(k);
                let ph = phi(&g.m, p.nu_lp, p.nu_hp, p.mf * frac, key).unwrap();
                if let Some(pv) = prev {
                    assert!(
                        ph > pv,
                        "rung-49's LEVEL bracket FAILED: cutting fuel lowered {key:?} at s={s} \
                         (w/mf={frac:.4}): {pv:.9} -> {ph:.9}. Every floor in rungs 49-60 \
                         depends on this monotonicity."
                    );
                }
                prev = Some(ph);
            }
        }
    }
}

// =============================================================================================
// HALF TWO — the RATE inversion, which must ALSO hold
// =============================================================================================

/// THE NEGATIVE. Cutting fuel makes `dφ/ds` MORE negative — the opposite sign to the level. Both
/// spools, every sampled point, monotone in the cut.
#[test]
fn rate_inversion_cutting_fuel_steepens_the_descent() {
    let g = rig();
    for key in KEYS {
        for s in SAMPLES {
            let p = at(&g.traj, s);
            let mut prev: Option<f64> = None;
            for k in 0..5 {
                let frac = 0.97f64.powi(k);
                let r = rate_of(&g.m, p.nu_lp, p.nu_hp, p.mf * frac, g.slope, key).unwrap();
                if let Some(pv) = prev {
                    assert!(
                        r < pv,
                        "the RATE inversion FAILED: cutting fuel made d{key:?}/ds shallower at \
                         s={s} (w/mf={frac:.4}): {pv:.9} -> {r:.9}. If this is a real plant \
                         change, docs/phi-rate-limiter-negative.md is wrong and rung 60's seam \
                         RE-OPENS -- a phi-rate leg may now be buildable on fuel."
                    );
                }
                prev = Some(r);
            }
        }
    }
}

/// The finding in one assertion: over the SAME fuel cut, at the SAME state, the level rises while
/// the rate falls — and by a MARGIN, not just the sign. A change that merely WEAKENED the state
/// term would drive `d_rate` toward zero and still pass a sign test. Measured ratios 3.47–5.91
/// (LP 5.17/5.91, HP 3.47/3.92); the bar is the CLAIM (the rate response dominates), as the
/// Python's.
#[test]
fn the_two_halves_carry_opposite_signs() {
    let g = rig();
    for key in KEYS {
        for s in SAMPLES {
            let p = at(&g.traj, s);
            let (full, cut) = (p.mf, p.mf * 0.90);
            let d_level = phi(&g.m, p.nu_lp, p.nu_hp, cut, key).unwrap()
                - phi(&g.m, p.nu_lp, p.nu_hp, full, key).unwrap();
            let d_rate = rate_of(&g.m, p.nu_lp, p.nu_hp, cut, g.slope, key).unwrap()
                - rate_of(&g.m, p.nu_lp, p.nu_hp, full, g.slope, key).unwrap();
            assert!(
                d_level > 0.0 && 0.0 > d_rate,
                "{key:?} at s={s}: a 10% fuel cut moved the level by {d_level:+.6} and the rate by \
                 {d_rate:+.6} -- the signs must be OPPOSITE (level up, rate down)."
            );
            assert!(
                d_rate.abs() > d_level.abs(),
                "{key:?} at s={s}: the rate response {:.6} no longer DOMINATES the level response \
                 {:.6} (ratio {:.3}, was 3.5-5.9). The inversion is eroding -- re-measure before \
                 trusting docs/phi-rate-limiter-negative.md.",
                d_rate.abs(), d_level.abs(), d_rate.abs() / d_level.abs()
            );
        }
    }
}

/// The walk of gate 4, returning `(seen, first refused cut)` and asserting the no-root claim on
/// every evaluable cut. Shared by gate 4 and the declared addition below.
fn walk(g: &Rig, key: Key, s: f64) -> (usize, Option<usize>) {
    let p = at(&g.traj, s);
    let r0 = rate_of(&g.m, p.nu_lp, p.nu_hp, p.mf, g.slope, key).unwrap();
    assert!(r0 < 0.0, "{key:?} at s={s} is not descending -- bad sample point");
    let (target, mut w, mut seen, mut first_refused) = (0.5 * r0, p.mf, 0usize, None);
    for i in 0..40 {
        w *= 0.9;
        let r = match rate_of(&g.m, p.nu_lp, p.nu_hp, w, g.slope, key) {
            Ok(r) => r,
            Err(_) => {
                first_refused.get_or_insert(i + 1);
                continue; // off the modeled speed-line region: KEEP CUTTING
            }
        };
        seen += 1;
        assert!(
            r < target,
            "the arresting bracket FOUND A ROOT for {key:?} at s={s} (w/mf={:.5}, dphi/ds={r:+.6} \
             >= target {target:+.6}). Formulation A is buildable and \
             docs/phi-rate-limiter-negative.md is wrong -- rung 60's seam RE-OPENS.",
            w / p.mf
        );
    }
    (seen, first_refused)
}

/// The consequence, as the solver sees it: rung 49's bracket search walks the fuel DOWN looking
/// for a sign change, and on the derivative it never finds one — a leg demanding merely HALF the
/// current descent rate is unrealisable. The walk is SHORTER than it looks: the plant leaves its
/// modeled speed-line region after 13–16 cuts, so the coverage is asserted too (NO SILENT CAPS),
/// at the Python's floor of 12.
#[test]
fn the_arresting_bracket_has_no_root() {
    let g = rig();
    for key in KEYS {
        for s in SAMPLES {
            let (seen, _) = walk(&g, key, s);
            assert!(
                seen >= 12,
                "{key:?} at s={s}: only {seen}/40 fuel cuts were evaluable (expected 13-16), so \
                 the 'no root' claim rests on too short a walk to mean anything."
            );
        }
    }
}

/// DECLARED ADDITION (slice AT). The walk's coverage, MEASURED on the Python before this file was
/// written (PyPy, `rust/tests/coverage_ledger.tsv`'s source run): 14 / 16 / 14 / 16 evaluable
/// cuts for (LP, 0.10) / (LP, 0.20) / (HP, 0.10) / (HP, 0.20), the first refusal at cut 15 / 17 /
/// 15 / 17, and every refusal the rung-43 closure's "does not bracket". Gate 4's `>= 12` cannot
/// tell a faithful fallible twin from one that refuses a cut early or late; this can.
#[test]
fn the_walk_is_cut_where_the_python_cut_it() {
    let g = rig();
    let mut got = Vec::new();
    for key in KEYS {
        for s in SAMPLES {
            got.push(walk(&g, key, s));
        }
    }
    assert_eq!(
        got,
        vec![(14, Some(15)), (16, Some(17)), (14, Some(15)), (16, Some(17))],
        "(seen, first refused cut) per (LP 0.10, LP 0.20, HP 0.10, HP 0.20)"
    );
    // ...and the refusal IS the closure's, not some other abort that happens to land there.
    let p = at(&g.traj, 0.10);
    let e = rate_of(&g.m, p.nu_lp, p.nu_hp, p.mf * 0.9f64.powi(15), g.slope, Key::Lp).unwrap_err();
    assert!(e.0.contains("does not bracket"), "unexpected refusal: {}", e.0);
}
