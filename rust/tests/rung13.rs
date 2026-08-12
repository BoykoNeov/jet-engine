//! Rung-13 verification: the RESOLVED MIXING PDF — rung-12's two lumps replaced by a
//! CONTINUOUS mean-preserving β-PDF of mixture fraction, integrated against the ideal bell.
//!
//! ```text
//! ⟨EI⟩ = ∫₀¹ EI_bell(φ(ξ))·P_β(ξ; ξ̄, g(C)) dξ,   g(C) = min(g_max, k_g·|ln(C/C_opt)|)
//! ```
//!
//! the SAME kinked Holdeman distance that drove rung-12's `w(C)`. `g = 0` at `C_opt` ⇒ a delta
//! ⇒ the well-mixed point value, with both flanks lifting by orders: the Holdeman optimum
//! LOCATION recovered from a continuous distribution.
//!
//! **A MECHANISM SEPARATION, not a rung-12 reproduction.** Rung 12's over-penetration CLIMB was
//! the DWELL — a TIME mechanism. This rung isolates COMPOSITION and drops the quench chain, so
//! it pins the optimum but structurally CANNOT climb: the far flank DESCENDS, because ⟨EI⟩(g) is
//! HUMPED (the β-PDF goes bimodal and piles mass off the stoich peak at both ends). Combining
//! the two mechanisms is rung 15.
//!
//! **The lesson, framed correctly — NOT generic convexity.** The bell is convex on its flanks
//! and CONCAVE at its peak, so there is no global convexity to invoke. NO production is sharply
//! PEAKED at stoich, so spreading φ around a fixed mean RAISES mean NO whenever the mean is
//! OFF-stoich, and REVERSES SIGN at a stoichiometric mean. `docs/rung13-spec.md`.
//!
//! Gates, priority order:
//!
//! 1. **reduce (LOAD-BEARING)** — no `pdf` leaves every rung-13 field `None` and the shared path
//!    bit-for-bit; `g → 0` is the well-mixed point value, EXACTLY.
//! 2. **the quadrature preserves the mean and the variance** — the closure's own deliverable.
//! 3. **the minimum is PINNED AT `C_opt`**, both flanks lifting by orders.
//! 4. **it sits AT the Holdeman group** and shifts as `(H/S)²`.
//! 5. **⟨EI⟩(g) is HUMPED** — which is WHY the far flank descends.
//! 6. **the convexity jump and its SIGN REVERSAL at a stoich mean.**
//! 7. `g(C)` is the kink; 8. cycle untouched; 9. guards.

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{f_stoich, hf_fuel_default, Gas};
use turbojet::nox::{
    bell_interpolator, beta_pdf_nodes_weights, ideal_bell_ei, Bell, JetMixing, MixingPdf,
    Unmixedness, ZonedNoxOpts,
};

const TAU: f64 = 3e-3;
// Coarse grids — the gates test SHAPE and DIRECTION (the pin, the shift, the hump, the sign),
// not digits. The Python's own rung-13 values, so the two suites certify the same resolution.
const NG: usize = 32;
const NB: usize = 120;
const NQ: usize = 160;

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
    ZonedNoxOpts { tau: TAU, quench_ngrid: NG, ..ZonedNoxOpts::default() }
}
fn cfg(s: f64) -> MixingPdf {
    MixingPdf { s, n_bell: NB, n_quad: NQ, ..MixingPdf::default() }
}

/// The design point, and the ONE ideal bell every sweep here reuses. The bell depends on neither
/// `g` nor `J`, which is the whole reason a J-sweep in this rung is affordable.
struct Dp {
    g: Gas,
    tt3: f64,
    tt4: f64,
    far: f64,
    p: f64,
    xibar: f64,
    bell: Bell,
}

/// The design point WITHOUT the bell — for the guard tests, which must build everything they
/// touch INSIDE the `catch_unwind` closure (`Gas` holds a `RefCell` property cache, so it is not
/// `UnwindSafe` and cannot be captured across the boundary).
fn design_state() -> (Gas, f64, f64, f64, f64) {
    let g = Gas::reacting_equilibrium();
    let r = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, flight().p0, losses())
        .run(&flight(), 50.0);
    (g, r.station("3").tt, r.station("4").tt, r.station("4").far, r.station("4").pt)
}

fn design_point() -> Dp {
    let g = Gas::reacting_equilibrium();
    let r = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, flight().p0, losses())
        .run(&flight(), 50.0);
    let (tt3, tt4, far, p) =
        (r.station("3").tt, r.station("4").tt, r.station("4").far, r.station("4").pt);
    let bell = bell_interpolator(p, tt3, hf_fuel_default(), TAU, NB, false);
    Dp { g, tt3, tt4, far, p, xibar: far / (1.0 + far), bell }
}

/// ⟨EI⟩ over the β-PDF on the prebuilt bell — production's arithmetic with the bell hoisted.
/// [`zoned_nox_matches_the_pdf_helper`] pins it to the production path.
fn pdf_ei(dp: &Dp, xibar: f64, g_seg: f64) -> f64 {
    if g_seg <= 1e-9 {
        return dp.bell.at(xibar);
    }
    let (nodes, w) = beta_pdf_nodes_weights(xibar, g_seg, NQ);
    w.iter().zip(nodes.iter()).fold(0.0, |acc, (&wi, &x)| acc + wi * dp.bell.at(x))
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

/// The uniformity optimum `J_opt` where `C = (S/H)√J_opt = C_opt`.
fn j_opt(c: &MixingPdf) -> f64 {
    let x = c.c_opt * JetMixing::default().h / c.s;
    x * x
}

// ---------------------------------------------------------------------------------------
// GATE 1 — reduce: `pdf = None` is the rung-12 path; `g → 0` is the point value.
// ---------------------------------------------------------------------------------------

#[test]
fn reduce_no_pdf_is_the_rung12_path() {
    let dp = design_point();
    for j in [9.0f64, 36.0] {
        let m = JetMixing { j, ..JetMixing::default() };
        let a = dp.g.zoned_nox(
            dp.far, dp.tt3, dp.tt4, dp.p, 1.5,
            ZonedNoxOpts { mixing: Some(m), ..opts() },
        );
        let b = dp.g.zoned_nox(
            dp.far, dp.tt3, dp.tt4, dp.p, 1.5,
            ZonedNoxOpts { mixing: Some(m), pdf: None, ..opts() },
        );
        for s in [&a, &b] {
            assert!(s.pdf.is_none() && s.ei_no_pdf.is_none() && s.g_seg.is_none());
            assert!(s.c_holdeman.is_none());
        }
        assert_eq!(
            a.ei_no_quenched.unwrap().to_bits(),
            b.ei_no_quenched.unwrap().to_bits(),
            "the shared mean-field path must be bit-for-bit"
        );
        assert_eq!(a.max_a_quench.unwrap().to_bits(), b.max_a_quench.unwrap().to_bits());
    }
}

/// `g → 0` is a delta at ξ̄, i.e. the well-mixed point value — and production returns the EXACT
/// bell there, not the interpolant.
///
/// The distinction is deliberate and the Python has it too: the helper's delta short-circuit
/// reads the interpolated grid, while `zoned_nox`'s reads `ideal_bell_ei` directly. Asserting
/// `==` against the exact value is what pins the production branch rather than the helper's.
#[test]
fn reduce_g_to_zero_is_the_well_mixed_point_value() {
    let dp = design_point();
    assert_eq!(
        pdf_ei(&dp, dp.xibar, 0.0).to_bits(),
        dp.bell.at(dp.xibar).to_bits(),
        "the helper's delta short-circuit must be the interpolant at ξ̄"
    );
    let c = cfg(0.0625);
    let s = dp.g.zoned_nox(
        dp.far, dp.tt3, dp.tt4, dp.p, 1.5,
        ZonedNoxOpts {
            mixing: Some(JetMixing { j: j_opt(&c), ..JetMixing::default() }),
            pdf: Some(c),
            ..opts()
        },
    );
    assert_eq!(s.g_seg.unwrap(), 0.0, "a jet placed exactly AT C_opt has g = 0");
    assert!((s.c_holdeman.unwrap() - c.c_opt).abs() < 1e-12);
    let exact = ideal_bell_ei(dp.far, dp.p, dp.tt3, hf_fuel_default(), TAU, false);
    assert_eq!(
        s.ei_no_pdf.unwrap().to_bits(),
        exact.to_bits(),
        "at C_opt (g=0) production must return the EXACT point value, not the interpolant"
    );
}

/// Pin the hoisted-bell helper to the PRODUCTION path, so every sweep below exercises the same
/// arithmetic `zoned_nox` does.
#[test]
fn zoned_nox_matches_the_pdf_helper() {
    let dp = design_point();
    let c = cfg(0.0625);
    let m = JetMixing { j: 36.0, ..JetMixing::default() };
    let s = dp.g.zoned_nox(
        dp.far, dp.tt3, dp.tt4, dp.p, 1.5,
        ZonedNoxOpts { mixing: Some(m), pdf: Some(c), ..opts() },
    );
    let g_seg = c.segregation(c.c(&m));
    let h = pdf_ei(&dp, dp.xibar, g_seg);
    assert!(
        (s.ei_no_pdf.unwrap() - h).abs() < 1e-9 * h.max(1e-12),
        "helper {h} vs production {}",
        s.ei_no_pdf.unwrap()
    );
    assert!((s.c_holdeman.unwrap() - c.c(&m)).abs() < 1e-12);
    assert!((s.g_seg.unwrap() - g_seg).abs() < 1e-12);
}

// ---------------------------------------------------------------------------------------
// GATE 2 — the quadrature's own deliverable: it integrates AT the specified mean.
// ---------------------------------------------------------------------------------------

/// The mean-preserving closure MUST integrate at ξ̄ and reproduce the target variance.
///
/// The `u = ξ^a` substitution makes this near-exact in the singular regime, and the CENTERED
/// window keeps the near-delta regime inside 1 % down to the delta floor. The SMALL-g values
/// matter: production's `g(C) → 0` continuously near `C_opt`, so a fine J-sweep through `J_opt`
/// hits arbitrarily small g and the window has to stay resolved there.
#[test]
fn the_quadrature_preserves_the_mean_and_the_variance() {
    let dp = design_point();
    for g_seg in [1e-6f64, 1e-4, 1e-3, 0.005, 0.01, 0.02, 0.05, 0.10, 0.20, 0.30] {
        let (nodes, w) = beta_pdf_nodes_weights(dp.xibar, g_seg, NQ);
        let mean = w.iter().zip(nodes.iter()).fold(0.0, |a, (&wi, &x)| a + wi * x);
        let var = w.iter().zip(nodes.iter()).fold(0.0, |a, (&wi, &x)| {
            let d = x - dp.xibar;
            a + wi * (d * d)
        });
        let var_tgt = g_seg * dp.xibar * (1.0 - dp.xibar);
        assert!(
            (mean - dp.xibar).abs() <= 0.01 * dp.xibar,
            "g={g_seg}: ⟨ξ⟩={mean} vs ξ̄={}",
            dp.xibar
        );
        assert!((var - var_tgt).abs() <= 0.05 * var_tgt, "g={g_seg}: var {var} vs {var_tgt}");
    }
}

// ---------------------------------------------------------------------------------------
// GATE 3/4 — the minimum is PINNED AT `C_opt`, and it sits at the Holdeman group.
// ---------------------------------------------------------------------------------------

/// At `C_opt` the jet is perfectly mixed (g=0 ⇒ delta ⇒ a uniform lean mixture ⇒ ≈0 NO); a small
/// step to EITHER flank segregates the mixture and lifts ⟨EI⟩ by ORDERS.
///
/// This is NOT rung-12's "falls then rises" bowl: that climb was the DWELL. Composition variance
/// ALONE pins the LOCATION, and the far flank instead descends (see the hump gate).
#[test]
fn the_emissions_minimum_is_pinned_at_c_opt_with_both_flanks_up() {
    let dp = design_point();
    let c = cfg(0.0625);
    let h = JetMixing::default().h;
    let jo = j_opt(&c);
    let at = |j: f64| pdf_ei(&dp, dp.xibar, c.segregation((c.s / h) * j.sqrt()));
    let (opt, under, over) = (at(jo), at(jo / 1.3), at(jo * 1.3));
    assert!(opt < 1e-3, "at C_opt the mixture is uniform and lean ⇒ ⟨EI⟩≈0, got {opt}");
    assert!(
        under > 1e3 * opt.max(1e-12) && over > 1e3 * opt.max(1e-12),
        "both immediate flanks must lift by orders: under={under}, over={over}"
    );
}

/// `J_min == J_opt = (C_opt·H/S)²`, so shrinking the spacing moves the minimum up EXACTLY as
/// `(H/S)²` — the Holdeman group made literal.
///
/// Four spacings, where the Python's own gate samples two. That is not padding: slice B measured
/// a location key refuting a claim precisely because the source's gate only sampled where the
/// claim held.
#[test]
fn the_optimum_sits_at_the_holdeman_group_and_shifts_as_h_over_s_squared() {
    let dp = design_point();
    let h = JetMixing::default().h;
    for s in [0.0800f64, 0.0625, 0.0500, 0.0400] {
        let c = cfg(s);
        let jo = j_opt(&c);
        let js = [jo / 4.0, jo / 2.0, jo, 2.0 * jo, 4.0 * jo];
        let eis: Vec<f64> =
            js.iter().map(|&j| pdf_ei(&dp, dp.xibar, c.segregation((s / h) * j.sqrt()))).collect();
        assert_eq!(
            argmin(&eis),
            2,
            "S={s}: the EI-min must sit AT J_opt={jo}, got J={}: {eis:?}",
            js[argmin(&eis)]
        );
    }
}

// ---------------------------------------------------------------------------------------
// GATE 5 — ⟨EI⟩(g) is HUMPED, which is WHY the far flank descends.
// ---------------------------------------------------------------------------------------

/// ⟨EI⟩ is NON-monotone in the segregation: it peaks at moderate `g` and DESCENDS toward large
/// `g`, because at extreme segregation the β-PDF goes BIMODAL — mass piles at pure air (ξ→0) and
/// at the rich cap, BOTH off the stoich peak. A tested feature, not a surprise.
///
/// The grid is COARSE on purpose. The quadrature's regime boundary sits at `g = ξ̄/(1+ξ̄) ≈ 0.026`
/// for this mean — right beside the hump — and across it the two schemes disagree by ~0.03 %,
/// enough to make the curve locally non-monotone. Measured, the peak cell clears both neighbours
/// by ~20 %, which no artefact of that size can move; a finer grid would put the argmax inside
/// the perturbation and turn a real detector into a flaky one.
#[test]
fn the_mean_ei_is_humped_in_the_segregation() {
    let dp = design_point();
    let grid = [0.005f64, 0.01, 0.02, 0.05, 0.12, 0.30];
    let v: Vec<f64> = grid.iter().map(|&g| pdf_ei(&dp, dp.xibar, g)).collect();
    let mut im = 0;
    for (i, &x) in v.iter().enumerate() {
        if x > v[im] {
            im = i;
        }
    }
    assert!(im > 0 && im < v.len() - 1, "the peak must be INTERIOR, got index {im}: {v:?}");
    assert!(
        v[im] / v[im - 1] > 1.02 && v[im] / v[im + 1] > 1.02,
        "the argmax must clear both neighbours by a margin the branch switch cannot supply \
         (~0.03%), got {:.4} and {:.4}",
        v[im] / v[im - 1],
        v[im] / v[im + 1]
    );
    assert!(v[2] > v[5], "⟨EI⟩(g) must descend toward high g (bimodal): {:?}", v);
}

// ---------------------------------------------------------------------------------------
// GATE 6 — the convexity jump, and its SIGN REVERSAL at a stoichiometric mean.
// ---------------------------------------------------------------------------------------

/// For a LEAN mean, spreading raises ⟨EI⟩ by orders — the stoich-ward tail samples the bell peak
/// while EI(ξ̄) sits tiny in the lean wing. For a STOICH mean, spreading LOWERS it, because mass
/// moves OFF the peak.
///
/// That reversal is what certifies the "peaked-at-stoich × off-stoich-mean" framing over a loose
/// "convexity ⇒ always raises" claim, and it is the discriminator rung 15 inherits.
#[test]
fn the_convexity_jump_reverses_sign_at_a_stoichiometric_mean() {
    let dp = design_point();
    let xibar_st = f_stoich() / (1.0 + f_stoich());
    assert!(
        pdf_ei(&dp, dp.xibar, 0.10) > 1e3 * dp.bell.at(dp.xibar),
        "lean mean: segregation must RAISE ⟨EI⟩ by orders"
    );
    assert!(dp.bell.at(xibar_st) > 5.0, "sanity: the stoich point value sits near the peak");
    assert!(
        pdf_ei(&dp, xibar_st, 0.10) < dp.bell.at(xibar_st),
        "stoich mean: segregation must LOWER ⟨EI⟩ — the sign reversal"
    );
}

// ---------------------------------------------------------------------------------------
// GATE 7 — `g(C)` is the kink: zero at `C_opt`, rising on both flanks, capped.
// ---------------------------------------------------------------------------------------

#[test]
fn the_segregation_is_kinked_and_zero_at_the_optimum() {
    let c = MixingPdf::default();
    assert_eq!(c.segregation(c.c_opt), 0.0, "g must be EXACTLY 0 at C_opt (perfect ⇒ delta)");
    assert!(c.segregation(c.c_opt / 1.3) > 0.0 && c.segregation(c.c_opt * 1.3) > 0.0);
    assert!(
        (c.segregation(c.c_opt / 1.4) - c.segregation(c.c_opt * 1.4)).abs() < 1e-12,
        "symmetric in ln C — an L1 |ln| distance"
    );
    assert!(c.segregation(c.c_opt * 1.05) > 0.0, "KINKED: a non-zero slope just off C_opt");
    assert_eq!(c.segregation(c.c_opt * 1e6), c.g_max, "capped at g_max");
}

// ---------------------------------------------------------------------------------------
// GATE 8/9 — cycle untouched, and the guards.
// ---------------------------------------------------------------------------------------

#[test]
fn the_cycle_is_untouched_by_a_pdf_call() {
    let g = Gas::reacting_equilibrium();
    let run = || {
        build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, flight().p0, losses())
            .run(&flight(), 50.0)
    };
    let r = run();
    let (tt3, tt4, far, p) =
        (r.station("3").tt, r.station("4").tt, r.station("4").far, r.station("4").pt);
    g.zoned_nox(
        far, tt3, tt4, p, 1.5,
        ZonedNoxOpts {
            mixing: Some(JetMixing { j: 36.0, ..JetMixing::default() }),
            pdf: Some(cfg(0.0625)),
            ..opts()
        },
    );
    assert_eq!(
        run().station("4").far.to_bits(),
        far.to_bits(),
        "a pdf call perturbed the cycle far — NO is trace, so it must stay bit-for-bit rung 6"
    );
}

#[test]
fn the_pdf_closure_requires_a_mixing_config() {
    let r = std::panic::catch_unwind(|| {
        let (g, tt3, tt4, far, p) = design_state();
        g.zoned_nox(
            far, tt3, tt4, p, 1.5,
            ZonedNoxOpts { pdf: Some(MixingPdf::default()), ..opts() },
        )
    });
    assert!(r.is_err(), "pdf without mixing must be rejected — it needs J and H for C");
}

/// The ≤1-of-N guard, which slice B could not write because only one closure existed then.
///
/// Its comment in `nox.rs` said the check was deliberately omitted while one closure was ported,
/// because a bar that cannot fail is not a bar, and promised it would arrive with the second.
/// This is that bar.
#[test]
fn pdf_and_unmixedness_are_mutually_exclusive() {
    let r = std::panic::catch_unwind(|| {
        let (g, tt3, tt4, far, p) = design_state();
        g.zoned_nox(
            far, tt3, tt4, p, 1.5,
            ZonedNoxOpts {
                mixing: Some(JetMixing { j: 16.0, ..JetMixing::default() }),
                pdf: Some(MixingPdf::default()),
                unmixedness: Some(Unmixedness::default()),
                ..opts()
            },
        )
    });
    assert!(r.is_err(), "two closures of the SAME variance physics must be rejected");
}

#[test]
fn mixing_pdf_positivity_guards() {
    MixingPdf::default().validate();
    MixingPdf { k_g: 0.0, ..MixingPdf::default() }.validate(); // k_g=0 ⇒ g≡0, allowed
    let bad = [
        MixingPdf { s: 0.0, ..MixingPdf::default() },
        MixingPdf { s: -0.1, ..MixingPdf::default() },
        MixingPdf { c_opt: 0.0, ..MixingPdf::default() },
        MixingPdf { k_g: -0.1, ..MixingPdf::default() },
        MixingPdf { g_max: 0.0, ..MixingPdf::default() },
        MixingPdf { g_max: 1.0, ..MixingPdf::default() },
        MixingPdf { g_max: 1.5, ..MixingPdf::default() },
        MixingPdf { n_bell: 1, ..MixingPdf::default() },
        MixingPdf { n_quad: 0, ..MixingPdf::default() },
    ];
    for c in bad {
        let r = std::panic::catch_unwind(move || c.validate());
        assert!(r.is_err(), "MixingPdf {c:?} should be rejected");
    }
}
