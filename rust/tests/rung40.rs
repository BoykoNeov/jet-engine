//! RUNG 40 — THE TWO-SHAFT TRANSIENT: the LP map opens a COMPLEX mode.
//!
//! Port of `tests/test_rung40.py`, gate for gate. **8 test functions / 9 collected items** —
//! gate 5 is `@pytest.mark.parametrize`d by gas and becomes two `#[test]` fns here, which is why
//! the function count and the item count differ. § 5.15's opening said "9 + 7" for the two suites
//! and step 1 corrected it to **9 + 8** by collecting; this file is the 9.
//!
//! Its eight gates:
//!
//!   1. REDUCE — the 2-D equilibrium (`Phi_L = Phi_H = 0`) reproduces rung 39's
//!      `TwoSpoolMapMatcher::match_point`, via the FORWARD closure only (never calling the
//!      matcher, so the reduce is non-circular). CPG and reacting.
//!   2. REDUCE — `lp_disabled` EXACT DISPATCH to rung 34's `SpoolTransient`, bit-for-bit.
//!   3. NON-TAUTOLOGICAL — an INDEPENDENT bare-math CPG two-shaft closure reproduces
//!      `(nu_L, nu_H, pi_lpc, pi_hpc)` AND `sigma_crit` ON SHAPED MAPS.
//!   4. `sigma_crit` — the INHERITED identity (`== 1` on flat + CPG) + its two breaking channels
//!      (the `cp(T)` curve, the map; map larger), + the REFUTATION that the map's shift direction
//!      is shape-dependent.
//!   5. FINDING (i) — STABILITY: `a<0, d<0, a*d>b*c` at every sampled point (MEASURED), hence both
//!      eigenvalues negative at every `rho` in `[0.05, 100]` (DERIVED from those signs).
//!   6. FINDING (ii) — THE COMPLEX MODE: `b*c<0` for every SHAPED-LP pair, `b*c>=0` for every
//!      FLAT-LP pair (`hp-only` is the discriminator).
//!   7. SCOPE — `sigma_crit` is FIRST-INSTANT only: the marched threshold does NOT converge to it.
//!   8. CYCLE UNTOUCHED — the default single-spool design path is bit-for-bit rung 6.
//!
//! # The two things gated here that Python does NOT gate
//!
//! § 5.15 registered two predictions that come due at this step and that no Python assertion
//! covers, so they are written here rather than left to the writeup:
//!
//! * **Prediction 9** — both `eigenvalues` branches gated against their **COUNTS**, not against
//!   silence. Gate 5 is the only grid in this suite that reaches the complex arm, and it reaches it
//!   a handful of times against a couple of hundred real ones; a port that took only one arm would
//!   otherwise pass every value assertion. The four numbers below were **MEASURED off this Rust
//!   run, per gas**, and are NOT the plan's "245 real / 7 complex" — that figure is the two gases
//!   SUMMED on Python's grid, and a count derived from a superset is a guess.
//! * **Prediction 10's second half** — every two-shaft accessor on the `Degenerate` variant
//!   PANICS, mirroring Python's uncaught `AttributeError`. Python's gate 2 only tests the forward,
//!   so the panic half is asserted inside gate 2 here (with `catch_unwind`) rather than as a tenth
//!   function, which would move the collected count this step just finished correcting.

use std::panic::{catch_unwind, AssertUnwindSafe};

use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::spool::SpoolTransient;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::{counters as tcount, TwoSpoolTransient,
                                    TwoSpoolTransientCore};

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;

// The CPG constants, written out ONCE so gate 3's reference can read them without touching `Gas`.
const GAMMA_C: f64 = 1.4;
const CP_C: f64 = 1004.0;
const R_C: f64 = 286.9;
const GAMMA_T: f64 = 1.3;
const CP_T: f64 = 1239.0;
const HPR: f64 = 42.8e6;

// The loss set, likewise — gate 3's reference reads these, not the shipped object.
const ETA_LPC: f64 = 0.90;
const ETA_HPC: f64 = 0.88;
const ETA_HPT: f64 = 0.92;
const ETA_LPT: f64 = 0.90;
const ETA_M: f64 = 0.99;
const ETA_B: f64 = 0.99;
const PI_N: f64 = 0.98;
const PI_D: f64 = 0.97;
const PI_B: f64 = 0.96;

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

fn flat() -> ComponentMap { ComponentMap::flat() }

fn lp_shaped() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
}

fn hp_shaped() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
}

/// Disclosed shape pairs (`a_t = 0` throughout — compressor islands only), in Python's dict order.
fn shapes() -> Vec<(&'static str, ComponentMap, ComponentMap)> {
    let m = |a: f64, b: f64, c: f64, sigma: f64, l: f64| ComponentMap {
        a, b, c, sigma, l, ..ComponentMap::flat()
    };
    vec![
        ("flat", flat(), flat()),
        ("flow/press", lp_shaped(), hp_shaped()),
        ("press/flow", m(0.05, 0.20, 0.0, 0.1, 1.0), m(0.20, 0.05, 0.0, 0.1, 0.7)),
        ("tilted", m(0.14, 0.10, 0.06, 0.2, 0.85), m(0.14, 0.10, 0.06, 0.2, 0.85)),
        ("steep", m(0.25, 0.12, 0.0, 0.3, 1.2), m(0.25, 0.12, 0.0, 0.3, 1.2)),
        ("lp-only", lp_shaped(), flat()),   // LP shaped, HP flat
        ("hp-only", flat(), hp_shaped()),   // HP shaped, LP FLAT — the discriminator
    ]
}

fn lp_is_flat(name: &str) -> bool {
    name == "flat" || name == "hp-only"
}

fn two_spool_design(gas: Gas) -> TwoSpoolEngine {
    build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, 50_000.0, real())
}

fn tt(gas: Gas, map_lp: ComponentMap, map_hp: ComponentMap) -> TwoSpoolTransientCore {
    TwoSpoolTransientCore::new(two_spool_design(gas), flight(), 1.0, map_lp, map_hp, 1.0)
}

/// A plain single-spool design (its compressor plays the HPC role) — the reduce ladder.
fn single_design() -> Engine {
    build_turbojet(Gas::reacting_equilibrium(), PI_HPC, TT4, 50_000.0, Losses {
        pi_d: PI_D, eta_c: ETA_HPC, eta_b: ETA_B, pi_b: PI_B, eta_t: ETA_HPT, eta_m: ETA_M,
        pi_n: PI_N, nozzle_convergent: true, ..Losses::default()
    })
}

// ----------------------------------------------------------------------------------- gate 1
/// GATE 1 — the 2-D root (`Phi_L = Phi_H = 0`) lands on rung 39's matched point.
///
/// NON-CIRCULAR: `equilibrium` uses the FORWARD closure only and never calls
/// `TwoSpoolMapMatcher::match_point`. Rung 34's reduce was a 1-D bracket; this is a genuine 2-D
/// Newton from the design start, so it also witnesses that the design point is reachable.
///
/// **`eq["mdot_air"]` is `close.mdot_air` — `mdot4/(1+f)`, NOT the LP-face flow `m_lp`.** Step 1's
/// injection table makes that one line the most load-bearing in the module (122 smoke keys), and
/// this gate's bar is a *relative* `1e-9`, so the wrong pick is not guaranteed to fail here. It is
/// therefore named as the field it is, rather than as "whatever passes".
#[test]
fn gate1_reduce_2d_equilibrium_is_rung39() {
    let cases: Vec<(Gas, Vec<f64>)> = vec![
        (cpg_gas(), vec![1500.0, 1300.0, 1200.0]),
        (Gas::reacting_equilibrium(), vec![1500.0, 1200.0]),
    ];
    for (gas, sweep) in cases {
        let t = tt(gas, lp_shaped(), hp_shaped());
        for tt4 in sweep {
            let od = t.match_point(&flight(), tt4);
            let eq = t.equilibrium(&flight(), tt4);
            assert!((eq.nu_lp / od.n_lp_ratio - 1.0).abs() < 1e-10, "{tt4} {}", eq.nu_lp);
            assert!((eq.nu_hp / od.n_hp_ratio - 1.0).abs() < 1e-10, "{tt4} {}", eq.nu_hp);
            assert!((eq.close.pi_lpc / od.base.pi_lpc - 1.0).abs() < 1e-9,
                    "{tt4} {}", eq.close.pi_lpc);
            assert!((eq.close.pi_hpc / od.base.pi_hpc - 1.0).abs() < 1e-9,
                    "{tt4} {}", eq.close.pi_hpc);
            assert!((eq.close.mdot_air / od.base.mdot_air - 1.0).abs() < 1e-9,
                    "{tt4} {}", eq.close.mdot_air);
            // And the residuals really are zero (not just the speeds agreeing).
            assert!(eq.phi_lp_dot.abs() < 1e-9 && eq.phi_hp_dot.abs() < 1e-9,
                    "{tt4} {} {}", eq.phi_lp_dot, eq.phi_hp_dot);
        }
    }
}

// ----------------------------------------------------------------------------------- gate 2
/// GATE 2 — EXACT DISPATCH: `lp_disabled` builds NO two-shaft state at all.
///
/// Python's `__init__` returns before `super().__init__`, so it constructs and holds a plain
/// rung-34 `SpoolTransient` and forwards to it: the fields compare `==` (not a converged limit) —
/// the rung 38/39 contract, one rung on.
///
/// **And the other half of § 5.15's prediction 10, which Python does not gate**: every two-shaft
/// accessor on that variant PANICS, mirroring Python's uncaught `AttributeError`. Without this the
/// prediction's "every other method panics" would ship unmeasured. The mirror is checked too — the
/// single-spool accessor panics on the FULL variant — so the guard is a discriminator rather than
/// a blanket panic that would pass the same assertion for the wrong reason.
#[test]
fn gate2_reduce_lp_disabled_is_rung34_bit_for_bit() {
    let deg = TwoSpoolTransient::lp_disabled(single_design(), flight(), 1.0, hp_shaped());
    let reference = SpoolTransient::new(single_design(), flight(), 1.0, hp_shaped());
    for tt4 in [1500.0, 1200.0] {
        let a = deg.degenerate().equilibrium(&flight(), tt4, None);
        let b = reference.equilibrium(&flight(), tt4, None);
        for (k, x, y) in [
            ("nu", a.nu, b.nu), ("pi_c", a.pi_c, b.pi_c), ("tau_c", a.tau_c, b.tau_c),
            ("tau_t", a.tau_t, b.tau_t), ("mdot_air", a.mdot_air, b.mdot_air),
            ("f", a.f, b.f), ("Phi", a.phi, b.phi), ("sp_thrust", a.sp_thrust, b.sp_thrust),
        ] {
            assert_eq!(x.to_bits(), y.to_bits(), "Tt4={tt4} {k}: {x} != {y}");
        }
    }

    // PREDICTION 10's second half — the two-shaft accessors are unreachable on this variant.
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let core_panics = catch_unwind(AssertUnwindSafe(|| { let _ = deg.core(); })).is_err();
    let mut deg_mut = TwoSpoolTransient::lp_disabled(single_design(), flight(), 1.0, hp_shaped());
    let core_mut_panics =
        catch_unwind(AssertUnwindSafe(|| { let _ = deg_mut.core_mut(); })).is_err();
    let full = TwoSpoolTransient::new(two_spool_design(cpg_gas()), flight(), 1.0,
                                      lp_shaped(), hp_shaped(), 1.0);
    let degenerate_panics =
        catch_unwind(AssertUnwindSafe(|| { let _ = full.degenerate(); })).is_err();
    std::panic::set_hook(hook);
    assert!(core_panics, "core() must panic on the lp_disabled variant");
    assert!(core_mut_panics, "core_mut() must panic on the lp_disabled variant");
    assert!(degenerate_panics, "degenerate() must panic on the full variant");
}

// ----------------------------------------------------------------------------------- gate 3
/// The bare bisection — Python's `bisect`, 400 halvings to `1e-15`. NOT the crate's Illinois.
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

/// The loading law, read off the map's plain FIELDS — no `ComponentMap` method is called.
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
    pi_lpc: f64,
    pi_hpc: f64,
    f: f64,
}

/// GATE 3 — an INDEPENDENT bare-math CPG two-shaft closure reproduces the solver.
///
/// No `Gas` / component / `ComponentMap`-method / `TwoSpoolTransient` call inside the reference:
/// closed-form CPG thermodynamics, its own choke bisection, its own FORWARD speed lines, its own
/// 2-D equilibrium by damped Newton. The only things it reads off the shipped objects are the three
/// throat AREAS and the maps' plain fields — exactly the line Python draws.
///
/// Reproduces `(nu_L, nu_H, pi_lpc, pi_hpc)` AND — the load-bearing part — `sigma_crit` ON SHAPED
/// MAPS (~1.2), which the `== 1` identity could not anchor. Two code paths, one operating point
/// (the rung-31/33/38/39 gate pattern).
///
/// **WHICH ASSERTION IS THE DISCRIMINATOR DEPENDS ON WHERE THE SWEEP STARTS, AND THE SWEEP STARTS
/// AT DESIGN.** Deleting the map's linear loading term `l` from `bare_psi` was injected to check
/// this gate is not vacuous, and it is caught — but by the `sigma_crit` bar, not by the four speed
/// and pressure-ratio bars four lines above it. At `Tt4 = 1500` the flow coefficient is `1` by
/// construction, so `l*(phi - 1)` vanishes identically and the injection moves `nu_L` by
/// **4.4e-15** — six orders INSIDE the `1e-8` bar. At the next throttle down it moves it by
/// **2.9e-2**, so those bars are live and merely blind at the point Python happens to visit first,
/// while `sigma_crit` reads `1.0000001` against the shipped `1.1483` at design itself. That is the
/// docstring's own *"reproducing the `==1` identity alone would only re-check the reduce"*,
/// measured rather than asserted.
///
/// **This reference is deliberately NOT bit-matched to Python's** — its own bisection, its own
/// power spelling (`powf`/`powi`/`sqrt`, not the crate's `powp`), its own damped Newton. The bars
/// are `1e-8`/`1e-6` for that reason. Tightening it toward bit-equality would mean copying the
/// shipped instruction sequence, which is exactly the independence that makes the gate worth
/// having (§ 5.4's COPY vs REDERIVATION).
#[test]
fn gate3_independent_cpg_two_shaft_closure() {
    let (map_lp, map_hp) = (lp_shaped(), hp_shaped());
    let t = tt(cpg_gas(), map_lp, map_hp);

    let gc = (GAMMA_C - 1.0) / GAMMA_C;
    let gt = (GAMMA_T - 1.0) / GAMMA_T;

    // Design point, closed form (same freestream as the solver: Tt2 == Tt2_d here).
    let stag = 1.0 + 0.5 * (GAMMA_C - 1.0) * 0.85f64.powi(2);
    let tt2 = 250.0 * stag;
    let tt25_d = tt2 * (1.0 + (PI_LPC.powf(gc) - 1.0) / ETA_LPC);
    let tt3_d = tt25_d * (1.0 + (PI_HPC.powf(gc) - 1.0) / ETA_HPC);
    let f_d = (CP_T * TT4 - CP_C * tt3_d) / (ETA_B * HPR - CP_T * TT4);
    let (tau_lpc_d, tau_hpc_d) = (tt25_d / tt2, tt3_d / tt25_d);
    let p_ref_lp = CP_C * (tt25_d - tt2);
    let p_ref_hp = CP_C * (tt3_d - tt25_d);

    // `pi_t/sqrt(tau_t) = area_ratio` (MFP* is Tt-independent on CPG — rung 38 gate 2).
    let turbine = |area_ratio: f64, eta_t: f64| -> f64 {
        let tau = move |p: f64| 1.0 - eta_t * (1.0 - p.powf(gt));
        let pi_t = bare_bisect(&|p: f64| p / tau(p).sqrt() - area_ratio, 0.02, 0.999);
        tau(pi_t)
    };

    let b = &t.inner.base;
    let (area_hp, area_lp) = (b.a4 / b.a45, b.a45 / (b.a8 * PI_N));
    let tau_hpt = turbine(area_hp, ETA_HPT);
    let tau_lpt = turbine(area_lp, ETA_LPT);

    // The bare forward closure: one bisection in `m_L`, then both power residuals.
    let phis = |nu_l: f64, nu_h: f64, tt4: f64| -> (f64, f64, BareEv) {
        let ev = |m_l: f64| -> BareEv {
            let phi_l = m_l / nu_l;                  // Tt2 == Tt2_d  =>  n_L == nu_L
            let tau_lpc = 1.0 + (tau_lpc_d - 1.0) * bare_psi(&map_lp, phi_l) * nu_l * nu_l;
            let tt25 = tt2 * tau_lpc;
            let e_l = bare_eta_at(&map_lp, ETA_LPC, phi_l, nu_l);
            let pi_lpc = (1.0 + e_l * (tau_lpc - 1.0)).powf(1.0 / gc);
            // Corrected-flow transfer to the HP face.
            let m_h = m_l * (PI_LPC / pi_lpc) * (tt25 / tt25_d).sqrt();
            let n_h = nu_h * (tt25_d / tt25).sqrt();
            let phi_h = m_h / n_h;
            let tau_hpc = 1.0 + (tau_hpc_d - 1.0) * bare_psi(&map_hp, phi_h) * n_h * n_h;
            let tt3 = tt25 * tau_hpc;
            let e_h = bare_eta_at(&map_hp, ETA_HPC, phi_h, n_h);
            let pi_hpc = (1.0 + e_h * (tau_hpc - 1.0)).powf(1.0 / gc);
            let f = (CP_T * tt4 - CP_C * tt3) / (ETA_B * HPR - CP_T * tt4);
            // NGV choke, referred to design (MFP* cancels on CPG).
            let m_imp = (pi_lpc * pi_hpc / (PI_LPC * PI_HPC)) * (TT4 / tt4).sqrt()
                * (1.0 + f_d) / (1.0 + f);
            BareEv { m_imp, tt25, tt3, pi_lpc, pi_hpc, f }
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
        for _ in 0..80 {
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

    for tt4 in [1500.0, 1300.0, 1100.0] {
        let (nl, nh, s) = equilibrium(tt4);
        let od = t.match_point(&flight(), tt4);
        assert!((od.n_lp_ratio - nl).abs() < 1e-8, "{tt4} {} {nl}", od.n_lp_ratio);
        assert!((od.n_hp_ratio - nh).abs() < 1e-8, "{tt4} {} {nh}", od.n_hp_ratio);
        assert!((od.base.pi_lpc - s.pi_lpc).abs() < 1e-8 * s.pi_lpc, "{tt4} {}", od.base.pi_lpc);
        assert!((od.base.pi_hpc - s.pi_hpc).abs() < 1e-8 * s.pi_hpc, "{tt4} {}", od.base.pi_hpc);

        // THE LOAD-BEARING PART: sigma_crit on SHAPED maps, from the bare closure.
        let d = 5.0;
        let (pl_p, ph_p, _) = phis(nl, nh, tt4 + d);
        let (pl_m, ph_m, _) = phis(nl, nh, tt4 - d);
        let bare_sigma = ((pl_p - pl_m) / nl) / ((ph_p - ph_m) / nh);
        assert!(bare_sigma > 1.1, "shaped sigma_crit must be materially off 1: {bare_sigma}");
        let ship_sigma = t.lead_threshold(&flight(), tt4, d, None);
        assert!((ship_sigma - bare_sigma).abs() < 1e-6 * bare_sigma,
                "{tt4} {ship_sigma} {bare_sigma}");
    }
}

// ----------------------------------------------------------------------------------- gate 4
/// GATE 4 — `sigma_crit`: the INHERITED identity, its two channels, and the REFUTATION.
///
/// The `== 1` identity is rung 39 B1 restated for the transient (on the running line `sigma_crit`
/// reduces to the steady slip, which B1 pins at 1) — this rung's reduce SPINE, labelled inherited,
/// not billed as discovery.
///
/// **The `d` argument is not uniform**: (a) and (b) pass `25.0`, (c) leaves Python's DEFAULT
/// `d = 5.0` (`engine.py:3644`). Rust has no default arguments, so it is written out — carrying
/// the `25.0` into (c) would change the physics without failing loudly.
#[test]
fn gate4_sigma_crit_identity_channels_and_direction() {
    // (a) the identity — flat maps + CPG, every throttle.
    let t = tt(cpg_gas(), flat(), flat());
    for tt4 in [900.0, 1100.0, 1300.0, 1500.0] {
        assert!((t.lead_threshold(&flight(), tt4, 25.0, None) - 1.0).abs() < 1e-11, "{tt4}");
    }

    // (b) the two channels, measured identically — the rung-31-gate-5 mirror + rung 39 B2.
    let flat_cpg = (t.lead_threshold(&flight(), 1100.0, 25.0, None) - 1.0).abs();
    let gas_ch = (tt(Gas::thermally_perfect(), flat(), flat())
                  .lead_threshold(&flight(), 1100.0, 25.0, None) - 1.0).abs();
    let map_ch = (tt(cpg_gas(), lp_shaped(), hp_shaped())
                  .lead_threshold(&flight(), 1100.0, 25.0, None) - 1.0).abs();
    assert!(flat_cpg < 1e-11 && 1e-11 < gas_ch, "{flat_cpg} {gas_ch}");     // the mirror
    assert!(map_ch > gas_ch, "the map channel is the larger: {map_ch} {gas_ch}");

    // (c) THE REFUTATION (kept visible): the map's shift DIRECTION is shape-dependent. `d` here is
    // Python's default 5.0, NOT the 25.0 above.
    let lp_only = tt(cpg_gas(), lp_shaped(), flat()).lead_threshold(&flight(), 1100.0, 5.0, None);
    let hp_only = tt(cpg_gas(), flat(), hp_shaped()).lead_threshold(&flight(), 1100.0, 5.0, None);
    assert!(lp_only < 1.0 && 1.0 < hp_only,
            "'the map favours the LP spool' is FALSE — both signs are reachable: \
             {lp_only} {hp_only}");
}

// ----------------------------------------------------------------------------------- gate 5
/// GATE 5's body — FINDING (i): the clock ratio cannot destabilize the two-shaft pair.
///
/// The MEASURED part is the sign structure `a<0, d<0, a*d>b*c` (no `rho` in it). The DERIVED part —
/// asserted on top — is that those signs give `tr<0` and `det>0`, hence both eigenvalues negative,
/// at EVERY `rho>0`. Spot-checked over `rho` in `[0.05, 100]` (a 2000x range).
///
/// `@pytest.mark.slow` in Python; slice M's rule stands — port the gate, drop the marker,
/// re-introduce `#[ignore]` only against a MEASURED Rust cost.
///
/// Returns the two eigenvalue-branch counts so the caller can gate PREDICTION 9.
fn stability_is_rho_free(gas: &dyn Fn() -> Gas) -> (u64, u64) {
    let _ = tcount::take();
    for (name, lp, hp) in shapes() {
        let mut t = tt(gas(), lp, hp);
        for tt4 in [1500.0, 1200.0, 950.0] {
            let od = t.match_point(&flight(), tt4);
            let nu = Some((od.n_lp_ratio, od.n_hp_ratio));
            t.rho = 1.0;
            let j = t.jacobian(&flight(), tt4, nu, 1e-6);
            let (a, b, c, d) = (j[0][0], j[0][1], j[1][0], j[1][1]);
            assert!(a < 0.0 && d < 0.0, "{name} {tt4} {a} {d}");
            assert!(a * d > b * c, "{name} {tt4} {} {}", a * d, b * c);
            for rho in [0.05, 0.2, 1.0, 5.0, 20.0, 100.0] {
                let jr = [[a / rho, b / rho], [c, d]];
                let (e0, e1) = TwoSpoolTransientCore::eigenvalues(jr);
                assert!(e0.max(e1) < 0.0, "{name} {tt4} {rho}");
            }
        }
    }
    let census = tcount::take();
    (census.eig_real, census.eig_complex)
}

/// PREDICTION 9's bar, on the CPG gas — **MEASURED off this run, per gas**. Gate 5 is the only
/// grid in this suite that reaches the complex arm at all, which is why the bar lives here.
#[test]
fn gate5_finding_stability_is_rho_free_cpg() {
    let (real_hits, complex_hits) = stability_is_rho_free(&cpg_gas);
    // THE GRID FIRST, then the split — step 1's lesson about assertion ORDER. Edit the shape
    // list or the `rho` sweep and the split bar would otherwise fire first, reading as "an
    // eigenvalue arm died" when what actually moved is the grid under it.
    assert_eq!(real_hits + complex_hits, EIG_GRID, "the grid itself must be intact");
    assert_eq!((real_hits, complex_hits), (EIG_CPG_REAL, EIG_CPG_COMPLEX),
               "both eigenvalue arms must stay live at their measured counts");
}

/// PREDICTION 9's bar, on the REACTING gas.
#[test]
fn gate5_finding_stability_is_rho_free_reacting() {
    let (real_hits, complex_hits) = stability_is_rho_free(&Gas::reacting_equilibrium);
    // THE GRID FIRST, then the split — step 1's lesson about assertion ORDER. Edit the shape
    // list or the `rho` sweep and the split bar would otherwise fire first, reading as "an
    // eigenvalue arm died" when what actually moved is the grid under it.
    assert_eq!(real_hits + complex_hits, EIG_GRID, "the grid itself must be intact");
    assert_eq!((real_hits, complex_hits), (EIG_REACTING_REAL, EIG_REACTING_COMPLEX),
               "both eigenvalue arms must stay live at their measured counts");
}

/// 7 shape pairs x 3 throttles x 6 clock ratios — the call count, so the split below is read
/// against a grid whose size is itself asserted rather than assumed.
const EIG_GRID: u64 = 7 * 3 * 6;
const EIG_CPG_REAL: u64 = 124;
const EIG_CPG_COMPLEX: u64 = 2;
const EIG_REACTING_REAL: u64 = 121;
const EIG_REACTING_COMPLEX: u64 = 5;

// ----------------------------------------------------------------------------------- gate 6
/// GATE 6 — FINDING (ii): a COMPLEX inter-spool mode exists iff the LP map is SHAPED.
///
/// `hp-only` is the DISCRIMINATOR — the HP map is shaped there and NO band appears, so the
/// mechanism is the LP map specifically, not shaping in general.
///
/// Gated: existence + the sign of `b*c` + the mechanism. DELIBERATELY NOT gated: the band's
/// LOCATION and `|Im/Re|` — both ride on the representative shapes and are disclaimed, exactly as
/// rung 39 disclaims its slip depth.
#[test]
fn gate6_finding_complex_mode_is_created_by_the_lp_map() {
    for (name, lp, hp) in shapes() {
        let mut t = tt(cpg_gas(), lp, hp);
        for tt4 in [1500.0, 1200.0] {
            let od = t.match_point(&flight(), tt4);
            let nu = Some((od.n_lp_ratio, od.n_hp_ratio));
            t.rho = 1.0;
            let j = t.jacobian(&flight(), tt4, nu, 1e-6);
            let bc = j[0][1] * j[1][0];
            let band = t.oscillatory_band(&flight(), tt4, nu);
            if lp_is_flat(name) {
                assert!(bc >= 0.0, "flat LP map must keep b*c >= 0: {name} {tt4} {bc}");
                assert!(band.is_none(), "no complex band with a flat LP map: {name} {tt4}");
                assert_eq!(t.damping_ratio_max(&flight(), tt4, nu), 0.0, "{name} {tt4}");
            } else {
                assert!(bc < 0.0, "a shaped LP map must flip b*c negative: {name} {tt4} {bc}");
                let (lo, hi) = band.unwrap_or_else(|| panic!("{name} {tt4}"));
                assert!(0.0 < lo && lo < hi, "{name} {tt4} {lo} {hi}");
                // The returned band really brackets the discriminant's sign change.
                let (a, b, c, d) = (j[0][0], j[0][1], j[1][0], j[1][1]);
                let disc = |rho: f64| (a / rho - d).powi(2) + 4.0 * b * c / rho;
                let mid = (lo * hi).sqrt();
                assert!(disc(mid) < 0.0, "complex inside the band: {name} {tt4} {}", disc(mid));
                assert!(disc(0.5 * lo) > 0.0 && disc(2.0 * hi) > 0.0, "{name} {tt4}");
                assert!(t.damping_ratio_max(&flight(), tt4, nu) > 0.0, "{name} {tt4}");
            }
        }
    }
}

// ----------------------------------------------------------------------------------- gate 7
/// GATE 7 — SCOPE: `sigma_crit` does NOT govern the finite-amplitude ramp.
///
/// Asserted as a DELIBERATE NON-convergence so the withdrawn claim cannot silently creep back into
/// the rung. The marched threshold `rho*` (bisected on the sign of the running-line-referenced slip
/// excursion) sits far from `sigma_crit`, because that excursion is dominated by the steady slip
/// SCHEDULE moving with `Tt4` while the speeds lag — schedule-slaved, not lead-governed.
///
/// **THE ASSERTION ORDER IS LOAD-BEARING.** `elo * ehi < 0.0` runs FOUR LINES BEFORE the `0.2`
/// margin, and step 1 measured that this — not the margin — is what a truncated step count breaks
/// (§ 5.15's prediction 8, refuted in both halves). Merging or reordering the two would silently
/// retire that finding, so the bracket check stays first and separate.
///
/// `r_ramp` is left at Python's DEFAULT `0.5` (`engine.py:3791`); only `s_end` and `ds` are named
/// at the call site there, and `1.2/0.05` is exactly the non-exact pair that makes
/// `int(round(s_end/ds))` live.
#[test]
fn gate7_scope_sigma_crit_is_first_instant_only() {
    let mut t = tt(cpg_gas(), lp_shaped(), hp_shaped());
    let (tt4_lo, dtt4) = (1100.0, 50.0);
    let sc = t.lead_threshold(&flight(), tt4_lo, 5.0, None);

    let exc = |t: &mut TwoSpoolTransientCore, rho: f64| -> f64 {
        t.rho = rho;
        t.slip_excursion(&flight(), tt4_lo, dtt4, 0.5, 1.2, 0.05)
    };

    let (mut lo, mut hi) = (0.6 * sc, 1.6 * sc);
    let (elo, ehi) = (exc(&mut t, lo), exc(&mut t, hi));
    assert!(elo * ehi < 0.0, "a threshold exists in the bracket: {elo} {ehi}");
    for _ in 0..18 {
        let mid = 0.5 * (lo + hi);
        if exc(&mut t, mid) * elo > 0.0 { lo = mid; } else { hi = mid; }
    }
    let rho_star = 0.5 * (lo + hi);
    assert!((rho_star / sc - 1.0).abs() > 0.2,
            "sigma_crit must NOT be billed as the marched threshold — this gate exists to keep \
             the withdrawn claim withdrawn: {rho_star} {sc}");
}

// ----------------------------------------------------------------------------------- gate 8
/// GATE 8 — the default single-spool design path is untouched by rung 40.
#[test]
fn gate8_cycle_untouched_rung6() {
    let plain = || build_turbojet(Gas::reacting_equilibrium(), 10.0, TT4, 50_000.0, Losses {
        pi_d: PI_D, eta_c: 0.88, eta_b: ETA_B, pi_b: PI_B, eta_t: ETA_HPT, eta_m: ETA_M,
        pi_n: PI_N, ..Losses::default()
    });
    let a = plain().run(&flight(), 1.0);
    // Building a rung-40 object must not perturb the design cycle in any way.
    let _ = tt(cpg_gas(), lp_shaped(), hp_shaped()).match_point(&flight(), 1200.0);
    let b = plain().run(&flight(), 1.0);
    assert_eq!(a.performance.specific_thrust.to_bits(), b.performance.specific_thrust.to_bits());
    assert_eq!(a.station("4").tt.to_bits(), b.station("4").tt.to_bits());
    assert_eq!(a.station("9").pt.to_bits(), b.station("9").pt.to_bits());
}
