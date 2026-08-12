//! Rung-23 verification: the DERIVED DWELL SPECTRUM — the half rung 22 left open.
//!
//! Rung 22 resolved the cross-plane and derived the β-PDF WIDTH, but fed it through the
//! per-pocket quench with the IMPORTED rung-16 kinked SCALAR dwell — which BAKES `C_opt` in.
//! Rung 23 develops the SAME plane in TIME and reads each pocket's dwell from first principles:
//! no `C_opt`, no `τ_res`, no `b_u`. It adds no new dwell knob — geometry from rung 22, time
//! scale from rung 11.
//!
//! **THE ONE GENUINELY NEW QUANTITY is the ξ–τ CORRELATION.** Rich pockets are the LATE-ARRIVING
//! ones, so dwell correlates with composition; rung-16's scalar is ONE dwell for ALL pockets,
//! zero correlation by construction. Its effect on NO is a COMPUTATION, not an intuition — a
//! longer dwell means more time crossing stoich (more NO), but the correlated pocket is also
//! richer, further from stoich, cooling faster (less NO). The MATCHED-MEAN twin isolates it: the
//! same integral at the scalar `⟨τ⟩_PDF` removes the correlation and nothing else.
//!
//! **THE PYTHON'S `test_helper_matches_production` IS NOT PORTED**, for the reason slice C
//! recorded about rung 16's twin of it: the Rust already splits this closure into a bank plus an
//! integration, so re-assembling production out of its own parts and comparing would be a
//! function compared to itself. Two statements the Python cannot make are gated instead — that a
//! CONSTANT spectrum reproduces the scalar path BIT-for-bit (the new `Dwell` enum's own reduce),
//! and that the matched-mean arm IS rung 16's closure at a derived scalar. Both can fail; the
//! transcription could not. `docs/plans/todo-rust-port.md` § 4.7.
//!
//! Gates, priority order:
//!
//! 1. **reduce** — `spatial_dwell: None` is the prior path; the terminal field reproduces rung
//!    22's `g` **BIT-EXACTLY** (tighter than the Python's own `< 1e-9`; see § 4.7 for why).
//! 2. **THE CORRELATION SIGN** — `corr_ratio > 1`, strictly off 1, one-signed across `τ_mix`.
//! 3. **the `Dwell` enum's own reduce**, and the rung-16 identity that replaces the vacuous test.
//! 4. the rung-18 tie, clamp dormancy, cycle untouched.
//! 5. guards — mixing required, ≤1-of-EIGHT, RQL, positivity.

use std::panic::catch_unwind;
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{hf_fuel_default, Gas};
use turbojet::nox::{
    beta_pdf_nodes_weights, pocket_quench_grid_dwell, pocket_quench_integrate,
    pocket_quench_mean_ei, spatial_dwell_field, spatial_segregation, two_stream_ceiling, Dwell,
    JetMixing, PocketOpts, SpatialDwellPdf, SpatialPdf, TauSpectrum, ZonedNoxOpts,
};

const TAU: f64 = 3e-3;
const PHI_P: f64 = 1.5;
const CE: f64 = 0.20;
const NB: usize = 20;
const NQ: usize = 64;
const NG: usize = 9;
const NSTEPS: usize = 100;
const NY: usize = 20;
const NT: usize = 16;
const S0: f64 = 0.0625;
const H0: f64 = 0.10;

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
    JetMixing { j, h: H0, c_e: CE, shape_n: 2.0, ..JetMixing::default() }
}
fn cfg() -> SpatialDwellPdf {
    SpatialDwellPdf {
        s: S0, ny: NY, nz: NY, nt: NT, n_bell: NB, n_quad: NQ, ..SpatialDwellPdf::default()
    }
}

struct Dp {
    g: Gas,
    tt3: f64,
    tt4: f64,
    far: f64,
    p: f64,
}

fn design_state() -> (Gas, f64, f64, f64, f64) {
    let g = Gas::reacting_equilibrium();
    let r = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, flight().p0, losses())
        .run(&flight(), 50.0);
    (g, r.station("3").tt, r.station("4").tt, r.station("4").far, r.station("4").pt)
}

fn design_point() -> Dp {
    let (g, tt3, tt4, far, p) = design_state();
    Dp { g, tt3, tt4, far, p }
}

impl Dp {
    fn run(&self, j: f64, c: SpatialDwellPdf) -> turbojet::nox::ZonedNoxState {
        self.g.zoned_nox(
            self.far, self.tt3, self.tt4, self.p, PHI_P,
            ZonedNoxOpts { mixing: Some(mix(j)), spatial_dwell: Some(c), ..opts() },
        )
    }
}

// ------------------------------------------------------------------------------------------
// GATE 1 — reduce, and the BIT-EXACT consistency anchor.
// ------------------------------------------------------------------------------------------

#[test]
fn reduce_none_leaves_the_prior_path_untouched() {
    let dp = design_point();
    let base = dp.g.zoned_nox(
        dp.far, dp.tt3, dp.tt4, dp.p, PHI_P,
        ZonedNoxOpts { mixing: Some(mix(16.0)), ..opts() },
    );
    let none = dp.g.zoned_nox(
        dp.far, dp.tt3, dp.tt4, dp.p, PHI_P,
        ZonedNoxOpts { mixing: Some(mix(16.0)), spatial_dwell: None, ..opts() },
    );
    assert_eq!(base.ei_no_quenched.unwrap().to_bits(), none.ei_no_quenched.unwrap().to_bits());
    assert!(none.ei_no_spatial_dwell.is_none() && none.corr_ratio.is_none());
}

#[test]
fn the_terminal_field_reproduces_rung_22_BIT_for_bit() {
    // THE CONSISTENCY ANCHOR, and it is TIGHTER than the source's own bar. The Python asserts
    // `< 1e-9` here and its docstring only claims "<1%"; measured, the two are EXACTLY equal at
    // every J and grid, because rung 23's terminal plume goes through `_plume(1.0)` — whose
    // `1.0^(1/3)` and `√1.0` are exactly 1 — and every accumulator on the way is FLAT, matching
    // rung 22's single pass. Asserting the exactness is what makes this gate able to catch a
    // re-ordered accumulation, which `< 1e-9` never could. See § 4.7.
    let dp = design_point();
    for j in [4.0f64, 16.0, 100.0] {
        for ny in [16usize, 20, 32] {
            let m = mix(j);
            let (g23, _) = spatial_dwell_field(
                dp.far, PHI_P, S0, m.h, m.j, m.tau_q(), 0.316, 0.28, 0.28, ny, ny, NT,
            );
            let g22 =
                spatial_segregation(dp.far, PHI_P, S0, m.h, m.j, 0.316, 0.28, 0.28, ny, ny);
            assert_eq!(
                g23.to_bits(),
                g22.to_bits(),
                "terminal field {g23} != rung-22 {g22} at J={j} ny={ny} — a re-ordered \
                 accumulation is the first thing to look at"
            );
        }
    }
}

#[test]
fn production_reports_the_same_width_as_rung_22_at_a_matched_grid() {
    // AT A MATCHED GRID — stated explicitly because the two configs' DEFAULTS differ (rung 22
    // ships 48/48, rung 23 ships 40/40). At defaults these two widths are NOT equal and that is
    // not a defect; the Python's own tests dodge it by passing grids on both sides, which makes
    // the mismatch invisible in the source. Recorded here so the next reader does not rediscover
    // it as a bug.
    let dp = design_point();
    let s = dp.run(16.0, cfg());
    let g22 = SpatialPdf { s: S0, ny: NY, nz: NY, ..SpatialPdf::default() }
        .segregation(&mix(16.0), dp.far, PHI_P)
        .0;
    assert_eq!(s.g_spatial_dwell.unwrap().to_bits(), g22.to_bits());
}

// ------------------------------------------------------------------------------------------
// GATE 2 — THE CORRELATION SIGN (the load-bearing positive).
// ------------------------------------------------------------------------------------------

#[test]
fn the_correlation_adds_no_and_the_instrument_is_alive() {
    let dp = design_point();
    let s = dp.run(16.0, cfg());
    let corr = s.ei_no_spatial_dwell.unwrap();
    let mean = s.ei_no_spatial_dwell_meanfield.unwrap();
    let ratio = s.corr_ratio.unwrap();
    assert!(
        corr > mean,
        "the ξ–τ correlation must ADD NO: correlated {corr:.6} vs matched-mean {mean:.6}"
    );
    assert!(ratio > 1.0, "corr_ratio={ratio} must exceed 1");
    // THE DEAD-INSTRUMENT GUARD. A τ(ξ) accidentally wired flat — or a `Dwell::PerPocket` arm
    // that silently fell back to the scalar — gives corr_ratio EXACTLY 1.0, which without this
    // bar reads as "the correlation has no effect" rather than "the instrument measured
    // nothing". The two are opposite conclusions from the same number.
    assert!(
        (ratio - 1.0).abs() > 1e-6,
        "corr_ratio is {ratio}, i.e. indistinguishable from 1 — the matched-mean twin and the \
         correlated run produced the same integral, so this measured NOTHING"
    );
    // ...and the spectrum really is non-constant, which is the upstream reason.
    let m = mix(16.0);
    let (_, spec) =
        spatial_dwell_field(dp.far, PHI_P, S0, m.h, m.j, m.tau_q(), 0.316, 0.28, 0.28, NY, NY, NT);
    let lo = spec.taus.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = spec.taus.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    assert!(hi > 2.0 * lo, "the derived τ(ξ) must actually SPREAD: {lo:.3e} .. {hi:.3e}");
}

#[test]
fn the_correlation_sign_is_one_signed_across_tau_mix() {
    // The magnitude rides on rung-11's un-anchored τ_mix, so only the SIGN is derived — and the
    // sign must survive a wide sweep of the thing it rides on. `C_e` scales τ_mix inversely, so
    // this walks τ_mix over ~×0.2–×5 without touching the geometry.
    let dp = design_point();
    for c_e in [0.04f64, 0.10, 0.20, 0.50, 1.00] {
        let s = dp.g.zoned_nox(
            dp.far, dp.tt3, dp.tt4, dp.p, PHI_P,
            ZonedNoxOpts {
                mixing: Some(JetMixing { j: 16.0, h: H0, c_e, shape_n: 2.0, ..Default::default() }),
                spatial_dwell: Some(cfg()),
                ..opts()
            },
        );
        let r = s.corr_ratio.unwrap();
        assert!(r > 1.0, "the correlation must ADD NO at every τ_mix: corr_ratio={r} at C_e={c_e}");
        // formation-limited throughout — the Jensen concavity never wins
        assert!(
            s.max_a_quench.unwrap() < 1.0,
            "pockets must stay formation-limited (max_a<1) at C_e={c_e}, got {}",
            s.max_a_quench.unwrap()
        );
    }
}

// ------------------------------------------------------------------------------------------
// GATE 3 — the `Dwell` enum's own reduce, and the rung-16 identity.
// ------------------------------------------------------------------------------------------
// These two REPLACE the Python's `test_helper_matches_production`, which in Rust would compare
// a function to itself (see the module docs). Both can fail.

#[test]
fn a_constant_spectrum_reproduces_the_scalar_path_bit_for_bit() {
    // The new `Dwell` enum's reduce contract: `PerPocket` at a flat spectrum IS `Scalar`. This is
    // the arm that would catch a `TauSpectrum::at` that mis-handles its flat extrapolation, or a
    // bank that reads τ at the wrong ξ — neither of which any ⟨EI⟩ tolerance would localise.
    let dp = design_point();
    let tau_flat = 4.0e-3;
    let flat = TauSpectrum { centers: vec![0.0, 1.0], taus: vec![tau_flat, tau_flat] };
    let po = PocketOpts { n_bell: NB, quench_ngrid: NG, quench_nsteps: NSTEPS, super_eq_o: false };
    let a = pocket_quench_grid_dwell(
        dp.far, dp.tt3, dp.p, hf_fuel_default(), TAU, Dwell::PerPocket(&flat), po,
    );
    let b = pocket_quench_grid_dwell(
        dp.far, dp.tt3, dp.p, hf_fuel_default(), TAU, Dwell::Scalar(tau_flat), po,
    );
    assert_eq!(a.vals.len(), b.vals.len());
    for (i, (x, y)) in a.vals.iter().zip(b.vals.iter()).enumerate() {
        assert_eq!(x.to_bits(), y.to_bits(), "pocket {i} differs: {x} vs {y}");
    }
    assert_eq!(a.max_a.to_bits(), b.max_a.to_bits());
}

#[test]
fn the_matched_mean_arm_is_rung_16s_closure_at_a_derived_scalar() {
    // A CROSS-RUNG identity the Python does not state: rung 23's correlation-off twin is exactly
    // rung 16's per-pocket closure driven at a dwell rung 23 derived. If the `Dwell::Scalar` arm
    // ever diverged from the rung-16 entry point — a different bank, a different integration —
    // this fails while every rung-16 gate still passes.
    let dp = design_point();
    let m = mix(16.0);
    let (g_seg, spec) =
        spatial_dwell_field(dp.far, PHI_P, S0, m.h, m.j, m.tau_q(), 0.316, 0.28, 0.28, NY, NY, NT);
    let xibar = dp.far / (1.0 + dp.far);
    let (nodes, wts) = beta_pdf_nodes_weights(xibar, g_seg, NQ);
    let tau_mean = wts.iter().zip(nodes.iter()).fold(0.0, |a, (&w, &x)| a + w * spec.at(x));

    let (via_rung16, _) = pocket_quench_mean_ei(
        dp.far, dp.tt3, dp.p, hf_fuel_default(), TAU, tau_mean, g_seg, NQ,
        PocketOpts { n_bell: NB, quench_ngrid: NG, quench_nsteps: NSTEPS, super_eq_o: false },
    );
    let s = dp.run(16.0, cfg());
    let twin_term2 = s.ei_no_spatial_dwell_meanfield.unwrap() - s.ei_no_quenched.unwrap();
    assert_eq!(
        s.tau_mean_dwell.unwrap().to_bits(),
        tau_mean.to_bits(),
        "production's ⟨τ⟩ must be the β-PDF mean of the derived spectrum"
    );
    assert!(
        (twin_term2 - via_rung16).abs() <= 1e-12 * via_rung16.abs().max(1e-30),
        "the matched-mean term 2 must BE rung 16's closure at ⟨τ⟩: {twin_term2} vs {via_rung16}"
    );
}

#[test]
fn the_bank_is_independent_of_the_segregation_width() {
    // The Rust's factorisation, asserted rather than assumed: `g_seg` enters ONLY the final
    // β-quadrature, never the bank. This is what makes a `g` sweep cheap, and it is the property
    // a future "optimisation" that threaded `g` into the bank would silently break.
    let dp = design_point();
    let m = mix(16.0);
    let (_, spec) =
        spatial_dwell_field(dp.far, PHI_P, S0, m.h, m.j, m.tau_q(), 0.316, 0.28, 0.28, NY, NY, NT);
    let po = PocketOpts { n_bell: NB, quench_ngrid: NG, quench_nsteps: NSTEPS, super_eq_o: false };
    let bank = pocket_quench_grid_dwell(
        dp.far, dp.tt3, dp.p, hf_fuel_default(), TAU, Dwell::PerPocket(&spec), po,
    );
    let mut last = f64::NAN;
    for g in [0.005f64, 0.02, 0.05] {
        let v = pocket_quench_integrate(&bank, dp.far, g, NQ);
        assert!(v.is_finite() && v != last, "the integral must respond to g");
        last = v;
    }
}

// ------------------------------------------------------------------------------------------
// GATE 4 — the rung-18 tie, dormancy, and the cycle.
// ------------------------------------------------------------------------------------------

#[test]
fn the_width_stays_below_the_two_stream_ceiling_and_the_clamp_stays_dormant() {
    let dp = design_point();
    for j in [4.0f64, 16.0, 64.0] {
        let s = dp.run(j, cfg());
        assert!(s.g_spatial_dwell.unwrap() < s.g_ceiling.unwrap());
        assert_eq!(s.g_ceiling.unwrap().to_bits(), two_stream_ceiling(dp.far, PHI_P).to_bits());
        assert!(
            s.max_a_quench.unwrap() < 1.0,
            "the dropped exhaust clamp must stay dormant at station 4 (max_a<1) at J={j}"
        );
    }
}

#[test]
fn the_cycle_is_untouched_a_pure_diagnostic() {
    let a = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, flight().p0, losses())
        .run(&flight(), 50.0);
    let dp = design_point();
    let _ = dp.run(16.0, cfg());
    let b = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, flight().p0, losses())
        .run(&flight(), 50.0);
    assert_eq!(
        a.performance.specific_thrust.to_bits(),
        b.performance.specific_thrust.to_bits()
    );
}

// ------------------------------------------------------------------------------------------
// GATE 5 — guards, and the derived-not-imposed signature.
// ------------------------------------------------------------------------------------------

#[test]
fn c_opt_is_derived_here_too_no_knob() {
    for k_p in [0.25f64, 0.316, 0.40] {
        let c = SpatialDwellPdf { k_p, ..cfg() };
        assert_eq!(c.c_opt().to_bits(), (1.0 / (4.0 * k_p * k_p)).to_bits());
    }
    assert!((cfg().c_opt() - 2.5).abs() < 0.02);
}

#[test]
fn spatial_dwell_requires_a_mixing_config() {
    let e = catch_unwind(|| {
        let (g, tt3, tt4, far, p) = design_state();
        g.zoned_nox(
            far, tt3, tt4, p, PHI_P,
            ZonedNoxOpts { spatial_dwell: Some(cfg()), tau_q: Some(1e-3), ..opts() },
        )
    })
    .expect_err("spatial_dwell without mixing must panic");
    let msg = e
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_string()))
        .unwrap_or_default();
    assert!(msg.contains("REQUIRES a `mixing` config"), "wrong panic: {msg}");
}

#[test]
fn at_most_one_closure_of_the_eight() {
    let e = catch_unwind(|| {
        let (g, tt3, tt4, far, p) = design_state();
        g.zoned_nox(
            far, tt3, tt4, p, PHI_P,
            ZonedNoxOpts {
                mixing: Some(mix(16.0)),
                spatial_dwell: Some(cfg()),
                spatial: Some(SpatialPdf { ny: NY, nz: NY, ..SpatialPdf::default() }),
                ..opts()
            },
        )
    })
    .expect_err("two spatial closures must panic");
    let msg = e
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_string()))
        .unwrap_or_default();
    assert!(msg.contains("AT MOST ONE"), "wrong panic: {msg}");
}

#[test]
fn the_config_and_the_field_reject_bad_input() {
    for bad in [
        SpatialDwellPdf { s: 0.0, ..cfg() },
        SpatialDwellPdf { k_p: -1.0, ..cfg() },
        SpatialDwellPdf { nt: 1, ..cfg() },
    ] {
        assert!(catch_unwind(move || bad.validate()).is_err(), "{bad:?} must be rejected");
    }
    // a non-positive τ_mix is not a geometry error, so it has its own guard
    let e = catch_unwind(|| {
        spatial_dwell_field(0.0295, PHI_P, S0, H0, 16.0, 0.0, 0.316, 0.28, 0.28, 8, 8, 4)
    })
    .expect_err("tau_mix=0 must panic");
    let msg = e
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_string()))
        .unwrap_or_default();
    assert!(msg.contains("must be positive"), "wrong panic: {msg}");
    cfg().validate();
}
