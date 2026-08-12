//! Rung-18 verification: the TRANSPORTED-variance closure — what a 0-D variance transport CAN
//! and CANNOT derive.
//!
//! Rungs 12–17 IMPOSE the β-PDF width as a kinked `g(C) = min(g_max, k_g·|ln(C/C_opt)|)`. This
//! rung solves `g(C)` instead as the residual of a variance DECAY ODE `dg/dt = −C_φ·ω(C)·g` from
//! a DERIVED two-stream ceiling, and feeds it through the SAME rung-13 ideal bell — only the
//! SOURCE of `g` changes.
//!
//! **THE LOAD-BEARING RESULT IS NEGATIVE, and the rung is stronger for it.** With any MEAN-FIELD
//! ω(J) — constant, √J, or linear in J — the residual `g(J)` is monotone or flat: no interior
//! optimum. An optimum needs `C_φ·ω·τ` non-monotone in J, i.e. ω with an interior MAXIMUM, i.e. a
//! PREFERRED LENGTH SCALE. A mean-field ω is built from J, τ_q(J), U_c and H — no spacing S — so
//! it has no scale to single out a J and cannot peak. The optimum can enter ONLY through
//! `ω(C = (S/H)√J)`, which is the spatial spacing injected BY HAND. So the coverage below is an
//! EXPLICITLY IMPOSED closure, the honest successor of rung-13's kink, NOT a derivation.
//!
//! **What transport legitimately DOES add** (and these are the positive gates): a ceiling DERIVED
//! from φ_p rather than the free `g_max = 0.3` (~4.4× too large); a RESIDUAL floor
//! `g(C_opt) = g_ceiling·e^(−Da_opt) > 0`, so the emissions optimum is ELEVATED off the well-mixed
//! value instead of touching it; and SMOOTHNESS — both one-sided slopes vanish at `C_opt`, so the
//! kink's SHARPNESS was the artifact and not its location. `docs/rung18-spec.md`.
//!
//! Gates, priority order:
//!
//! 1. **reduce** — no `transported` leaves the prior path untouched; `Da_opt → ∞` recovers the
//!    kinked notch; a vanishing ceiling gives the point value.
//! 2. **THE NEGATIVE RESULT** — mean-field ω is monotone/flat; only the spatial ω(C) peaks.
//! 3. **the DERIVED ceiling** from φ_p, and the RQL guard that a lean primary has none.
//! 4. **the residual floor ELEVATES the optimum**, which still sits AT `C_opt`.
//! 5. **smoothness** — the transported basin's slopes vanish where the kink's do not.
//! 6. **the basin ROUNDS the notch**; 7. cycle untouched; 8. guards.

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{f_stoich, hf_fuel_default, Gas};
use turbojet::nox::{
    bell_interpolator, ideal_bell_ei, pdf_mean_ei_on_bell, transport_variance,
    two_stream_ceiling, Bell, JetMixing, MixingPdf, PocketQuenchPdf, TransportedPdf,
    ZonedNoxOpts,
};

const TAU: f64 = 3e-3;
const PHI_P: f64 = 1.5;
const CE: f64 = 0.20;
const NB: usize = 48; // coarse ideal-bell / β-PDF grids (shape, not digits)
const NQ: usize = 64;
const NG: usize = 24;
const NSTEPS: usize = 200;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}
fn losses() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
        ..Losses::default()
    }
}
fn opts() -> ZonedNoxOpts {
    ZonedNoxOpts { tau: TAU, quench_ngrid: NG, quench_nsteps: NSTEPS, ..ZonedNoxOpts::default() }
}
fn mix(j: f64) -> JetMixing {
    JetMixing { j, c_e: CE, shape_n: 2.0, ..JetMixing::default() }
}
fn cfg() -> TransportedPdf {
    TransportedPdf { s: 0.0625, n_bell: NB, n_quad: NQ, n_ode: 200, ..TransportedPdf::default() }
}
fn argmin(v: &[f64]) -> usize {
    let mut b = 0;
    for (i, &x) in v.iter().enumerate() {
        if x < v[b] {
            b = i;
        }
    }
    b
}

/// A named mean-field mixing frequency ω(J) — the three shapes the negative-result gate drives
/// the same ODE with. None of them contains the spacing S, which is the whole point.
type MeanFieldOmega = (&'static str, fn(f64) -> f64);

struct Dp {
    g: Gas,
    tt3: f64,
    tt4: f64,
    far: f64,
    p: f64,
    xibar: f64,
    bell: Bell,
}

/// The design point WITHOUT the bell — for guard tests, which must build everything INSIDE the
/// `catch_unwind` closure (`Gas` holds a `RefCell` cache, so it is not `UnwindSafe`).
fn design_state() -> (Gas, f64, f64, f64, f64) {
    let g = Gas::reacting_equilibrium();
    let r = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, flight().p0, losses())
        .run(&flight(), 50.0);
    (g, r.station("3").tt, r.station("4").tt, r.station("4").far, r.station("4").pt)
}

fn design_point() -> Dp {
    let (g, tt3, tt4, far, p) = design_state();
    let bell = bell_interpolator(p, tt3, hf_fuel_default(), TAU, NB, false);
    Dp { g, tt3, tt4, far, p, xibar: far / (1.0 + far), bell }
}

impl Dp {
    fn run(&self, j: f64, c: TransportedPdf) -> turbojet::nox::ZonedNoxState {
        self.g.zoned_nox(
            self.far, self.tt3, self.tt4, self.p, PHI_P,
            ZonedNoxOpts { mixing: Some(mix(j)), transported: Some(c), ..opts() },
        )
    }
}

// ---------------------------------------------------------------------------------------
// GATE 1 — the reduces.
// ---------------------------------------------------------------------------------------

#[test]
fn reduce_no_transported_leaves_the_prior_path_untouched() {
    let dp = design_point();
    let base = dp.g.zoned_nox(
        dp.far, dp.tt3, dp.tt4, dp.p, PHI_P,
        ZonedNoxOpts { mixing: Some(mix(16.0)), ..opts() },
    );
    assert!(base.transported.is_none());
    assert!(base.ei_no_transported.is_none() && base.g_ceiling.is_none());
    assert!(base.ei_no_quenched.unwrap() > 0.0, "the mean-field term is present and untouched");
}

/// `Da_opt → ∞` is perfect best-jet mixing, so `g(C_opt) → 0` and the transported closure becomes
/// rung-13's kinked notch: the well-mixed point value. The kinked model IS the infinite-mixing
/// limit of the transported one.
#[test]
fn reduce_perfect_mixing_recovers_the_kinked_notch() {
    let dp = design_point();
    let point = ideal_bell_ei(dp.far, dp.p, dp.tt3, hf_fuel_default(), TAU, false);
    let st = dp.run(16.0, TransportedPdf { da_opt: 60.0, ..cfg() });
    let g = st.g_transported.unwrap();
    assert!(g < 1e-9, "g(C_opt) should collapse under near-perfect mixing, got {g:.2e}");
    let ei = st.ei_no_transported.unwrap();
    assert!(
        (ei - point).abs() <= 0.02 * point.max(1e-300),
        "perfect-mixing ei {ei:.5} vs point value {point:.5}"
    );
}

/// A primary only marginally richer than the overall mean injects almost no segregation, so the
/// ceiling — and therefore the transported width — vanishes.
#[test]
fn reduce_a_vanishing_ceiling_gives_the_point_value() {
    let dp = design_point();
    let phi_ov = dp.far / f_stoich();
    let gc = two_stream_ceiling(dp.far, phi_ov * 1.02);
    assert!(0.0 < gc && gc < 5e-3, "a near-overall primary gives a tiny ceiling, got {gc:.4}");
    let g = transport_variance(gc, 500.0, 2.5e-3, 2.0, 400);
    assert!(g < gc && g > 0.0, "the decay must stay inside (0, g_ceiling]");
}

// ---------------------------------------------------------------------------------------
// GATE 2 — THE NEGATIVE RESULT: 0-D transport cannot derive the optimum.
// ---------------------------------------------------------------------------------------

/// A GENUINE variance ODE with any MEAN-FIELD ω(J) gives a monotone-or-flat `g(J)` — the minimum
/// sits at an END, or the curve is flat to within 1e-4.
///
/// These curves ILLUSTRATE the structural argument rather than carrying it: an interior optimum
/// needs `C_φ·ω·τ` non-monotone in J ⇒ ω with an interior maximum ⇒ a preferred length scale, and
/// a mean-field ω has no spacing S to supply one.
#[test]
fn a_mean_field_omega_is_monotone_and_has_no_optimum() {
    let js = [4.0f64, 9.0, 16.0, 25.0, 49.0, 100.0, 225.0, 625.0];
    let forms: [MeanFieldOmega; 3] = [
        ("const", |_| 250.0),
        ("sqrtJ", |j| 250.0 * (j / 16.0).sqrt()),
        ("linJ", |j| 250.0 * (j / 16.0)),
    ];
    for (name, om) in forms {
        let v: Vec<f64> =
            js.iter().map(|&j| transport_variance(0.0675, om(j), 2.5e-3, 2.0, 400)).collect();
        let imin = argmin(&v);
        let hi = v.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let lo = v.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let flat = (hi - lo) <= 1e-4 * hi;
        assert!(
            flat || imin == 0 || imin == v.len() - 1,
            "mean-field ω={name} produced an INTERIOR optimum at J={} — impossible in 0-D",
            js[imin]
        );
    }
}

/// The interior optimum appears ONLY once ω depends on `C = (S/H)√J` — i.e. once the SPATIAL
/// spacing is injected. Same ODE, same fixed τ, ω peaked at `C_opt` ⇒ an interior min at `J_opt`.
#[test]
fn only_the_spatial_coverage_produces_an_interior_optimum() {
    let c = cfg();
    let js = [4.0f64, 9.0, 16.0, 25.0, 49.0, 100.0, 225.0, 625.0];
    let v: Vec<f64> = js
        .iter()
        .map(|&j| {
            let cc = (c.s / JetMixing::default().h) * j.sqrt();
            transport_variance(0.0675, c.coverage_omega(cc), c.tau_mix, c.c_phi, 400)
        })
        .collect();
    let imin = argmin(&v);
    assert!(imin > 0 && imin < v.len() - 1, "the spatial ω(C) must give an INTERIOR optimum");
    assert_eq!(js[imin], 16.0, "the optimum must sit at J_opt=16 (C_opt), got J={}", js[imin]);
}

// ---------------------------------------------------------------------------------------
// GATE 3 — the DERIVED ceiling (the lead positive result).
// ---------------------------------------------------------------------------------------

/// `g_ceiling = (ξ_p − ξ̄)/(1 − ξ̄)` from φ_p, to machine precision — a pure composition quantity,
/// independent of J and `C_e`, and ~4.4× smaller than rung-13's free `g_max = 0.3`.
#[test]
fn the_ceiling_is_derived_from_the_primary_richness() {
    let dp = design_point();
    let far_p = PHI_P * f_stoich();
    let xi_p = far_p / (1.0 + far_p);
    let expect = (xi_p - dp.xibar) / (1.0 - dp.xibar);
    assert!((two_stream_ceiling(dp.far, PHI_P) - expect).abs() <= 1e-12 * expect);
    assert!(0.0 < expect && expect < 0.3, "the derived ceiling {expect:.4} must be < g_max=0.3");
    assert!(0.3 / expect > 4.0, "g_max=0.3 should be >4× the derived ceiling {expect:.4}");
    let a = dp.run(16.0, cfg()).g_ceiling.unwrap();
    let b = dp.run(100.0, cfg()).g_ceiling.unwrap();
    assert!((a - b).abs() <= 1e-12 * a, "the ceiling is J-independent");
    assert!((a - expect).abs() <= 1e-9 * expect);
}

/// A primary LEANER than the overall mean has no two-stream segregation at all — the RQL geometry
/// guard, and the reason the ceiling is a derivation rather than a fit.
#[test]
fn a_primary_leaner_than_the_mean_has_no_ceiling() {
    let r = std::panic::catch_unwind(|| {
        let (_g, _tt3, _tt4, far, _p) = design_state();
        two_stream_ceiling(far, (far / f_stoich()) * 0.5)
    });
    let Err(e) = r else { panic!("a leaner-than-mean primary must fail the RQL-geometry guard") };
    // `catch_unwind` catches ANY panic, and the closure also builds an engine — so name the
    // guard, exactly as the Python's version does (`"RQL geometry" in str(e)`).
    let msg = e
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_string()))
        .unwrap_or_default();
    assert!(msg.contains("RQL geometry"), "it panicked, but not on the RQL guard: {msg}");
}

// ---------------------------------------------------------------------------------------
// GATE 4 — the RESIDUAL FLOOR elevates the optimum, which still sits AT `C_opt`.
// ---------------------------------------------------------------------------------------

/// `g(C_opt) = g_ceiling·e^(−Da_opt) > 0` — perfect mixing is never reached, so the emissions
/// optimum sits ABOVE the well-mixed point value rather than touching the floor the kink touches.
/// And the minimum is still AT `C_opt`: both immediate flanks lift.
#[test]
fn the_residual_floor_elevates_the_optimum_which_stays_at_c_opt() {
    let dp = design_point();
    let point = ideal_bell_ei(dp.far, dp.p, dp.tt3, hf_fuel_default(), TAU, false);
    let st = dp.run(16.0, cfg());
    assert!(st.g_transported.unwrap() > 1e-3, "the residual floor must be > 0 (no perfect mixing)");
    let ei_opt = st.ei_no_transported.unwrap();
    assert!(
        ei_opt > 10.0 * point.max(1e-9),
        "the elevated optimum must sit well above the well-mixed point value"
    );
    let ei_lo = dp.run(9.0, cfg()).ei_no_transported.unwrap();
    let ei_hi = dp.run(25.0, cfg()).ei_no_transported.unwrap();
    assert!(
        ei_lo > ei_opt && ei_hi > ei_opt,
        "the min must be AT C_opt (J=16): {ei_lo:.4} > {ei_opt:.4} < {ei_hi:.4}"
    );
}

// ---------------------------------------------------------------------------------------
// GATE 5/6 — the kink is NON-GENERIC: the transported basin is smooth, and it rounds the notch.
// ---------------------------------------------------------------------------------------

/// The transported `g(C)` has both one-sided slopes → 0 at `C_opt` (a smooth analytic minimum);
/// the imposed kink is a CORNER, with equal-and-opposite one-sided slopes `±k_g/C_opt`.
///
/// This is a finite difference of two nearly-equal accumulated values, which is the most
/// drift-sensitive quantity in the slice — and it is bit-identical across both interpreters, so
/// the bar can stay absolute rather than relative.
#[test]
fn the_transported_width_is_smooth_where_the_kink_is_a_corner() {
    let c = cfg();
    let c0 = c.c_opt;
    const EPS: f64 = 1e-5;
    let g_tr =
        |x: f64| transport_variance(0.0675, c.coverage_omega(x), c.tau_mix, c.c_phi, 400);
    let sr = (g_tr(c0 * (1.0 + EPS)) - g_tr(c0)) / (EPS * c0);
    let sl = (g_tr(c0) - g_tr(c0 * (1.0 - EPS))) / (EPS * c0);
    assert!(
        sr.abs() < 1e-2 && sl.abs() < 1e-2,
        "the transported slopes must vanish at C_opt: L={sl}, R={sr}"
    );
    let kink = MixingPdf { s: c.s, c_opt: c0, ..MixingPdf::default() };
    let kr = (kink.segregation(c0 * (1.0 + EPS)) - kink.segregation(c0)) / (EPS * c0);
    let kl = (kink.segregation(c0) - kink.segregation(c0 * (1.0 - EPS))) / (EPS * c0);
    assert!(
        (kr - kl).abs() > 1e-2,
        "the imposed kink must be a CORNER (a non-zero one-sided slope jump): {kl} vs {kr}"
    );
}

/// One step off `J_opt` the transported basin changes by O(1) — it is ROUNDED — while the kinked
/// ideal-bell notch dives by more than 10³×, because it touches the ≈0 well-mixed floor. The
/// sharpness was the artifact.
#[test]
fn the_emissions_basin_rounds_the_notch() {
    let dp = design_point();
    let c = cfg();
    let ei_opt = dp.run(16.0, c).ei_no_transported.unwrap();
    let ei_off = dp.run(9.0, c).ei_no_transported.unwrap();
    let ratio = ei_off / ei_opt;
    assert!(1.0 < ratio && ratio < 3.0, "the transported basin should round, got {ratio:.2}");
    let kink = MixingPdf { s: c.s, n_bell: NB, n_quad: NQ, ..MixingPdf::default() };
    let g_opt = kink.segregation(kink.c(&mix(16.0))).max(1e-12);
    let g_off = kink.segregation(kink.c(&mix(9.0)));
    let k_opt = pdf_mean_ei_on_bell(&dp.bell, dp.xibar, g_opt, NQ);
    let k_off = pdf_mean_ei_on_bell(&dp.bell, dp.xibar, g_off, NQ);
    assert!(
        k_off / k_opt.max(1e-12) > 1e3,
        "the kinked notch must dive >10³× one step off C_opt, got {:.2e}",
        k_off / k_opt.max(1e-12)
    );
}

// ---------------------------------------------------------------------------------------
// GATE 7/8 — cycle untouched, and the guards.
// ---------------------------------------------------------------------------------------

#[test]
fn the_cycle_is_untouched_by_a_transported_call() {
    let g = Gas::reacting_equilibrium();
    let run = || {
        build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, flight().p0, losses())
            .run(&flight(), 50.0)
    };
    let r = run();
    let (tt3, tt4, far, p) =
        (r.station("3").tt, r.station("4").tt, r.station("4").far, r.station("4").pt);
    g.zoned_nox(
        far, tt3, tt4, p, PHI_P,
        ZonedNoxOpts { mixing: Some(mix(16.0)), transported: Some(cfg()), ..opts() },
    );
    assert_eq!(run().station("4").far.to_bits(), far.to_bits());
}

#[test]
fn the_transported_closure_requires_a_mixing_config() {
    let r = std::panic::catch_unwind(|| {
        let (g, tt3, tt4, far, p) = design_state();
        g.zoned_nox(far, tt3, tt4, p, PHI_P, ZonedNoxOpts { transported: Some(cfg()), ..opts() })
    });
    assert!(r.is_err(), "transported without mixing must be rejected");
}

#[test]
fn at_most_one_closure_may_be_passed() {
    let r = std::panic::catch_unwind(|| {
        let (g, tt3, tt4, far, p) = design_state();
        g.zoned_nox(
            far, tt3, tt4, p, PHI_P,
            ZonedNoxOpts {
                mixing: Some(mix(16.0)),
                transported: Some(cfg()),
                pocket_quench: Some(PocketQuenchPdf::default()),
                ..opts()
            },
        )
    });
    assert!(r.is_err(), "two closures must trip the ≤1-of-N guard");
}

#[test]
fn transported_pdf_positivity_guards() {
    TransportedPdf::default().validate();
    let bad = [
        TransportedPdf { s: 0.0, ..TransportedPdf::default() },
        TransportedPdf { c_opt: 0.0, ..TransportedPdf::default() },
        TransportedPdf { c_phi: 0.0, ..TransportedPdf::default() },
        TransportedPdf { da_opt: 0.0, ..TransportedPdf::default() },
        TransportedPdf { w_cov: 0.0, ..TransportedPdf::default() },
        TransportedPdf { tau_mix: 0.0, ..TransportedPdf::default() },
        TransportedPdf { n_ode: 1, ..TransportedPdf::default() },
    ];
    for c in bad {
        let r = std::panic::catch_unwind(move || c.validate());
        assert!(r.is_err(), "TransportedPdf {c:?} should be rejected");
    }
}
