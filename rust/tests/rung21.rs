//! Rung-21 verification: super-equilibrium [O] threaded through the IDEAL-BELL PDF integrals —
//! the last equilibrium-O seam in the mixing family, discharged.
//!
//! Rung 19 lifted the primary's [O] by the Westenberg `m(T)`; rung 20 threaded that lift through
//! the finite quench. What was left was a HYBRID: rung-15's total was term 1 (lifted) plus term 2
//! (an ideal-bell integral still riding the equilibrium-O lower bound). Rung 21 lifts the bell
//! too, so `pdf`, `pdf_quench`'s term 2 and `transported` all carry `m(T)` and the sum is
//! internally consistent. The rung-20 "forbidden to combine" guard is DISCHARGED: `super_eq_o`
//! now composes with every closure.
//!
//! **The lift is a SHAPE-PRESERVING CONSISTENCY lift, and that is the whole finding.** It moves
//! every value and moves NO location: the optimum stays at `J_opt`, the shift stays `(H/S)²`, the
//! sign stays. And it is peak-concentrated and SMALLER than the primary's — the bell integral is
//! EI-weighted onto the near-stoich peak, which is the HOTTEST point on the bell, and `m(T)`
//! DECREASES in T. That is the rung-20 inversion generalised to composition variance:
//!
//! ```text
//!   lean POINT value  >  the φ_p=1.5 PRIMARY  >  the EI-weighted PDF integral
//! ```
//!
//! the deep-lean point value carrying the largest fractional lift (a cool flame ⇒ large m) on a
//! negligible EI. `docs/rung21-spec.md`.
//!
//! Gates, priority order:
//!
//! 1. **reduce (LOAD-BEARING)** — `super_eq_o = false` is BIT-FOR-BIT the default, at the helper
//!    level and through production.
//! 2. **the lift is modest, peak-concentrated and BELOW the primary's** — the ordering above.
//! 3. **the lift DECREASES with segregation** — the measured corollary of "peak-concentrated".
//! 4. **the HYBRID is resolved** — the composite lift sits BETWEEN its two terms, which is the
//!    measured proof that both now carry m(T); and every closure combines without raising.
//! 5. **`g → 0` consistency** — the delta limit threads the same flag.
//! 6. **SHAPE PRESERVED** — both arms minimise AT `J_opt`. (The wider version of this key — four
//!    spacings and two design points — is in `pdf_oracle.rs`, where it is also checked to be a
//!    real lift rather than a tautology.)

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{f_stoich, hf_fuel_default, Gas};
use turbojet::nox::{
    bell_interpolator, beta_pdf_nodes_weights, ideal_bell_ei, pdf_mean_ei, Bell, JetMixing,
    MixingPdf, QuenchPdf, TransportedPdf, ZonedNoxOpts,
};

const TAU: f64 = 3e-3;
const PHI_P: f64 = 1.5;
const NG: usize = 24;
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
fn argmin(v: &[f64]) -> usize {
    let mut b = 0;
    for (i, &x) in v.iter().enumerate() {
        if x < v[b] {
            b = i;
        }
    }
    b
}
fn j_opt(s: f64, c_opt: f64) -> f64 {
    let x = c_opt * JetMixing::default().h / s;
    x * x
}

/// The design point and BOTH bells — the eq-O one and the super-eq-O one. Two builds serve every
/// comparison here, because a bell depends on neither `g` nor `J`.
struct Dp {
    g: Gas,
    tt3: f64,
    tt4: f64,
    far: f64,
    p: f64,
    xibar: f64,
    eq: Bell,
    su: Bell,
}

fn design_state() -> (Gas, f64, f64, f64, f64) {
    let g = Gas::reacting_equilibrium();
    let r = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, flight().p0, losses())
        .run(&flight(), 50.0);
    (g, r.station("3").tt, r.station("4").tt, r.station("4").far, r.station("4").pt)
}

fn design_point() -> Dp {
    let (g, tt3, tt4, far, p) = design_state();
    let hf = hf_fuel_default();
    Dp {
        g,
        tt3,
        tt4,
        far,
        p,
        xibar: far / (1.0 + far),
        eq: bell_interpolator(p, tt3, hf, TAU, NB, false),
        su: bell_interpolator(p, tt3, hf, TAU, NB, true),
    }
}

fn pdf_ei(bell: &Bell, xibar: f64, g_seg: f64) -> f64 {
    if g_seg <= 1e-9 {
        return bell.at(xibar);
    }
    let (nodes, w) = beta_pdf_nodes_weights(xibar, g_seg, NQ);
    w.iter().zip(nodes.iter()).fold(0.0, |a, (&wi, &x)| a + wi * bell.at(x))
}

fn cfg() -> MixingPdf {
    MixingPdf { s: 0.0625, n_bell: NB, n_quad: NQ, ..MixingPdf::default() }
}

// ---------------------------------------------------------------------------------------
// GATE 1 — reduce (LOAD-BEARING): `super_eq_o = false` is BIT-FOR-BIT the prior rung.
// ---------------------------------------------------------------------------------------

/// The flag gates a single `m = 1` multiply, so the eq-O arm must be machine-exact against the
/// pre-rung-21 path.
///
/// **This is NOT the Python's version of the test, on purpose.** The Python compares
/// `_pdf_mean_ei(…)` against `_pdf_mean_ei(…, super_eq_o=False)` — a defaulted keyword against an
/// explicit one. In Rust the flag is a required positional argument, so that comparison is `false`
/// against `false`: a tautology. Three claims replace it, none of which can pass by construction:
///
/// * `false` is what the option struct DEFAULTS to, so every rung-13/15/18 call site gets the
///   equilibrium-O lower bound unless it asks otherwise;
/// * the wrapper's eq-O arm IS the eq-O bell integrated, bit-for-bit — rung 21 added a branch and
///   left the arm beside it untouched;
/// * the flag is LIVE — the two arms differ at every width. Without this the first two would pass
///   just as happily if `super_eq_o` did nothing at all.
#[test]
fn reduce_super_eq_o_false_is_the_pre_rung21_path() {
    let dp = design_point();
    let hf = hf_fuel_default();
    assert!(
        !ZonedNoxOpts::default().super_eq_o,
        "the equilibrium-O lower bound must remain the DEFAULT — that is what makes every \
         rung-13/15/18 call site bit-for-bit its pre-rung-21 self"
    );
    for g_seg in [0.02f64, 0.1, 0.3] {
        let a = pdf_mean_ei(dp.far, dp.tt3, dp.p, hf, TAU, g_seg, NB, NQ, false);
        let b = pdf_ei(&dp.eq, dp.xibar, g_seg);
        assert_eq!(a.to_bits(), b.to_bits(), "g={g_seg}: the eq-O arms must agree to the bit");
        let su = pdf_ei(&dp.su, dp.xibar, g_seg);
        assert!(su > b, "g={g_seg}: the flag must be LIVE — the su arm must exceed the eq arm");
    }
    // `g → 0` is the ONE place the wrapper and the hoisted helper deliberately differ: the
    // wrapper's delta short-circuit reads the EXACT bell, the helper's the interpolant. Asserted
    // here rather than skipped, so the loop above cannot quietly grow a case that measures nothing.
    assert_eq!(
        pdf_mean_ei(dp.far, dp.tt3, dp.p, hf, TAU, 0.0, NB, NQ, false).to_bits(),
        ideal_bell_ei(dp.far, dp.p, dp.tt3, hf, TAU, false).to_bits(),
        "the wrapper's g→0 branch must return the EXACT bell, not the interpolant"
    );
}

// ---------------------------------------------------------------------------------------
// GATE 2/3 — the lift is modest, PEAK-CONCENTRATED, and below the primary's.
// ---------------------------------------------------------------------------------------

/// The ordering `point > primary > PDF` is the rung's quantitative content.
///
/// `m(T)` DECREASES in T, and the bell integral is EI-weighted onto the near-stoich peak — the
/// hottest point on the bell, hotter than the φ_p=1.5 flame — so the PDF's fractional lift is the
/// SMALLEST of the three. The deep-lean point value is the largest, on a negligible EI.
#[test]
fn the_lift_is_modest_peak_concentrated_and_below_the_primary() {
    let dp = design_point();
    let hf = hf_fuel_default();
    let g_seg = 0.05; // a representative over-penetration flank
    let lift_pdf = pdf_ei(&dp.su, dp.xibar, g_seg) / pdf_ei(&dp.eq, dp.xibar, g_seg);
    assert!(
        (1.10..1.20).contains(&lift_pdf),
        "the ideal-bell PDF lift {lift_pdf:.3} left the measured (1.10, 1.20)"
    );
    let fl = PHI_P * f_stoich();
    let lift_primary = ideal_bell_ei(fl, dp.p, dp.tt3, hf, TAU, true)
        / ideal_bell_ei(fl, dp.p, dp.tt3, hf, TAU, false);
    assert!(
        (1.25..1.30).contains(&lift_primary),
        "the primary lift {lift_primary:.3} is not rung-19's ~×1.28"
    );
    let lift_point = pdf_ei(&dp.su, dp.xibar, 0.0) / pdf_ei(&dp.eq, dp.xibar, 0.0);
    assert!(
        lift_point > lift_primary && lift_primary > lift_pdf,
        "expected point ({lift_point:.3}) > primary ({lift_primary:.3}) > PDF ({lift_pdf:.3}) — \
         the lean point value largest, the EI-weighted integral smallest"
    );
}

/// A narrow PDF samples near the lean mean (cool ⇒ large m); a broad one pulls mass onto the
/// stoich peak (hot ⇒ small m). So the fractional lift DECREASES with segregation.
#[test]
fn the_lift_decreases_with_segregation() {
    let dp = design_point();
    let small = pdf_ei(&dp.su, dp.xibar, 0.005) / pdf_ei(&dp.eq, dp.xibar, 0.005);
    let large = pdf_ei(&dp.su, dp.xibar, 0.30) / pdf_ei(&dp.eq, dp.xibar, 0.30);
    assert!(
        large < small,
        "the lift must DECREASE with segregation (more variance ⇒ more stoich-peak weight): \
         g=0.30 → {large:.4} vs g=0.005 → {small:.4}"
    );
}

// ---------------------------------------------------------------------------------------
// GATE 4 — the HYBRID is resolved, and every closure now COMBINES.
// ---------------------------------------------------------------------------------------

/// Rung 15's composite lift must sit BETWEEN its two terms' lifts. That is the measured proof
/// that BOTH now carry `m(T)` — a half-lifted hybrid would land outside the bracket.
///
/// The same test also exercises the DISCHARGED rung-20 guard: `super_eq_o` combined with each of
/// the three ideal-bell closures must simply run.
#[test]
fn the_hybrid_is_resolved_and_every_closure_combines() {
    let dp = design_point();
    let m = JetMixing { j: 36.0, ..JetMixing::default() };
    let qp = QuenchPdf { s: 0.0625, n_bell: NB, n_quad: NQ, ..QuenchPdf::default() };
    let run = |o: ZonedNoxOpts| dp.g.zoned_nox(dp.far, dp.tt3, dp.tt4, dp.p, PHI_P, o);

    let a_eq = run(ZonedNoxOpts { mixing: Some(m), pdf_quench: Some(qp), ..opts() });
    let a_su =
        run(ZonedNoxOpts { mixing: Some(m), pdf_quench: Some(qp), super_eq_o: true, ..opts() });
    let p_eq = run(ZonedNoxOpts { mixing: Some(m), pdf: Some(cfg()), ..opts() });
    let p_su =
        run(ZonedNoxOpts { mixing: Some(m), pdf: Some(cfg()), super_eq_o: true, ..opts() });
    // the rung-20 forbid guard is DISCHARGED — this must not panic
    run(ZonedNoxOpts {
        mixing: Some(m),
        transported: Some(TransportedPdf {
            s: 0.0625,
            n_bell: NB,
            n_quad: NQ,
            ..TransportedPdf::default()
        }),
        super_eq_o: true,
        ..opts()
    });

    let composite = a_su.ei_no_pdf_quench.unwrap() / a_eq.ei_no_pdf_quench.unwrap();
    let bulk = a_su.ei_no_quenched.unwrap() / a_eq.ei_no_quenched.unwrap(); // term 1 (rung 20)
    let bell = p_su.ei_no_pdf.unwrap() / p_eq.ei_no_pdf.unwrap(); // term 2's ideal bell
    assert!(
        (1.10..1.20).contains(&composite),
        "the pdf_quench composite lift {composite:.3} left the measured (1.10, 1.20)"
    );
    let (lo, hi) = if bulk < bell { (bulk, bell) } else { (bell, bulk) };
    assert!(
        lo - 1e-9 <= composite && composite <= hi + 1e-9,
        "the composite lift {composite:.4} must sit between term 1 (bulk {bulk:.4}) and term 2 \
         (bell {bell:.4}) — that BOTH lift is what dissolves the rung-20 half-lifted hybrid"
    );
}

// ---------------------------------------------------------------------------------------
// GATE 5 — `g → 0` consistency: the delta limit threads the same flag.
// ---------------------------------------------------------------------------------------

#[test]
fn the_g_to_zero_limit_is_the_super_eq_point_value() {
    let dp = design_point();
    let c = cfg();
    let jo = j_opt(c.s, c.c_opt); // C = C_opt exactly ⇒ g = 0
    let s = dp.g.zoned_nox(
        dp.far, dp.tt3, dp.tt4, dp.p, PHI_P,
        ZonedNoxOpts {
            mixing: Some(JetMixing { j: jo, ..JetMixing::default() }),
            pdf: Some(c),
            super_eq_o: true,
            ..opts()
        },
    );
    assert_eq!(s.g_seg.unwrap(), 0.0);
    assert!((s.c_holdeman.unwrap() - c.c_opt).abs() < 1e-12);
    let point = ideal_bell_ei(dp.far, dp.p, dp.tt3, hf_fuel_default(), TAU, true);
    assert_eq!(
        s.ei_no_pdf.unwrap().to_bits(),
        point.to_bits(),
        "at C_opt (g=0) the lifted ei_no_pdf must BE the super-eq-O point value — the delta \
         short-circuit has to thread the same flag as the bell it replaces"
    );
}

// ---------------------------------------------------------------------------------------
// GATE 6 — SHAPE PRESERVED: both arms minimise AT `J_opt`.
// ---------------------------------------------------------------------------------------

#[test]
fn the_lift_preserves_the_shape_and_the_optimum() {
    let dp = design_point();
    let (s, h) = (0.0625f64, JetMixing::default().h);
    let c = MixingPdf { s, ..MixingPdf::default() };
    let jo = j_opt(s, c.c_opt);
    let js = [jo / 4.0, jo / 2.0, jo, 2.0 * jo, 4.0 * jo];
    for (bell, tag) in [(&dp.eq, "eq-O"), (&dp.su, "super-eq-O")] {
        let eis: Vec<f64> = js
            .iter()
            .map(|&j| pdf_ei(bell, dp.xibar, c.segregation((s / h) * j.sqrt())))
            .collect();
        assert_eq!(
            argmin(&eis),
            2,
            "{tag}: the ⟨EI⟩ minimum must stay pinned AT J_opt={jo} — the lift is \
             shape-preserving, not a relocated optimum. Got J={}: {eis:?}",
            js[argmin(&eis)]
        );
    }
}
