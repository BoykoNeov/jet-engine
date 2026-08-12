//! Rung-15 verification: the PDF THROUGH the finite quench — the first rung where the two
//! mixing mechanisms rungs 12 and 13 kept apart finally COMBINE.
//!
//! ```text
//! ⟨EI⟩₁₅ = EI_bulk_quench(τ_mean)   [term 1: the rung-11 mean field — a FINITE floor, all C]
//!        + D(u) · ⟨EI_bell⟩(g)      [term 2: the rung-13 integral × a rung-12 dwell]
//! ```
//!
//! Rung 13 pinned the optimum LOCATION but, on the ideal bell with the quench chain dropped, its
//! optimum collapsed to ≈0 and its far flank DESCENDED. Rung 12 had the dwell — an absolute,
//! off-optimum-growing residence — so its over-penetration flank CLIMBED. Adding them gives a
//! result distinguishable from BOTH parents: a FINITE floor at `C_opt`, both flanks up, and the
//! far flank climbing again.
//!
//! **The discriminator that matters** is the STOICH-MEAN SIGN REVERSAL. Term 2 samples the
//! nonlinear, peaked bell, so it reverses sign when the mean moves onto the peak. A "dwell-only
//! PDF through the quench" — variance riding the ~linear `EI_quench(τ)` — collapses to rung-12's
//! mean and carries the WRONG sign, so it cannot reverse. That is what says rung 15 is genuine
//! composition work and not rung 12 in disguise. `docs/rung15-spec.md`.
//!
//! Gates, priority order:
//!
//! 1. **reduce (LOAD-BEARING)** — no `pdf_quench` leaves every rung-15 field `None` and the
//!    shared path bit-for-bit; at `C_opt` the total IS the finite bulk quench NO, not ≈0.
//! 2. **the FINITE floor (headline)** — rung-13's ≈0 optimum BECOMES a finite bulk value.
//! 3. **the optimum PINNED AT `C_opt`**, both flanks up, and the far flank CLIMBING.
//! 4. **it sits AT the Holdeman group** and shifts as `(H/S)²`.
//! 5. **the stoich-mean SIGN REVERSAL** — the rung-12-in-disguise discriminator.
//! 6. the two kinks; 7. cycle untouched; 8. guards.

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{equilibrium_composition, f_stoich, hf_fuel_default, Gas};
use turbojet::nox::{
    bell_interpolator, beta_pdf_nodes_weights, primary_aft, quench_no, quench_trajectory,
    thermal_no, Bell, JetMixing, MixingPdf, QuenchOpts, QuenchPdf, QuenchPoint, Unmixedness,
    ZonedNoxOpts,
};

const TAU: f64 = 3e-3;
const PHI_P: f64 = 1.5; // the RQL rich primary every mixing rung anchors on
const CE: f64 = 0.20; // the ANCHORED jet-entrainment regime (rungs 11-16)
// The Python's own rung-15 grids, so both suites certify the same resolution.
const NG: usize = 32;
const NSTEPS: usize = 400;
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
    ZonedNoxOpts { tau: TAU, quench_ngrid: NG, quench_nsteps: NSTEPS, ..ZonedNoxOpts::default() }
}
fn mix(j: f64) -> JetMixing {
    JetMixing { j, c_e: CE, shape_n: 2.0, ..JetMixing::default() }
}
fn cfg(s: f64) -> QuenchPdf {
    QuenchPdf { s, n_bell: NB, n_quad: NQ, ..QuenchPdf::default() }
}
fn j_opt(c: &QuenchPdf) -> f64 {
    let x = c.c_opt * JetMixing::default().h / c.s;
    x * x
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

/// The design point plus the two objects every sweep here reuses: the shared τ-independent
/// quench trajectory (term 1) and the J-independent ideal bell (term 2). Neither depends on J,
/// which is what makes a seven-point jet sweep affordable.
struct Dp {
    g: Gas,
    tt3: f64,
    tt4: f64,
    far: f64,
    p: f64,
    xibar: f64,
    comp_p: Vec<(&'static str, f64)>,
    t_p: f64,
    alpha: f64,
    n0: f64,
    tab: Vec<QuenchPoint>,
    bell: Bell,
}

/// The design point WITHOUT the caches — for guard tests, which must build everything INSIDE the
/// `catch_unwind` closure (`Gas` holds a `RefCell` cache, so it is not `UnwindSafe`).
fn design_state() -> (Gas, f64, f64, f64, f64) {
    let g = Gas::reacting_equilibrium();
    let r = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, flight().p0, losses())
        .run(&flight(), 50.0);
    (g, r.station("3").tt, r.station("4").tt, r.station("4").far, r.station("4").pt)
}

fn design_point() -> Dp {
    let (g, tt3, tt4, far, p) = design_state();
    let hf = hf_fuel_default();
    let far_p = PHI_P * f_stoich();
    let alpha = far / far_p;
    let t_p = primary_aft(far_p, p, tt3, hf);
    let comp_p = equilibrium_composition(far_p, t_p, p);
    let ntot: f64 = comp_p.iter().map(|&(_, v)| v).sum();
    let n0 = alpha * thermal_no(&comp_p, t_p, p, TAU, far_p, 4000, 1.0).x_no * ntot;
    let tab = quench_trajectory(&comp_p, t_p, alpha, far, tt3, p, NG);
    let bell = bell_interpolator(p, tt3, hf, TAU, NB, false);
    Dp { g, tt3, tt4, far, p, xibar: far / (1.0 + far), comp_p, t_p, alpha, n0, tab, bell }
}

impl Dp {
    /// Term 1 — the rung-11 mean-field bulk quench at `τ_mean = mixing.tau_q`, exactly as
    /// production's `ei_no_quenched` computes it.
    fn floor(&self, j: f64) -> f64 {
        let m = mix(j);
        let sched = move |x: f64| m.schedule(x);
        quench_no(
            &self.comp_p, self.t_p, self.alpha, self.far, self.tt3, self.p, self.n0, m.tau_q(),
            QuenchOpts {
                nsteps: NSTEPS,
                ngrid: NG,
                tab: Some(&self.tab),
                schedule: Some(&sched),
                super_eq_o: false,
            },
        )
        .ei
    }

    /// ⟨EI_bell⟩ over the β-PDF on the prebuilt bell — the rung-13 integral, reused verbatim.
    fn bell_pdf(&self, mean_xi: f64, g_seg: f64) -> f64 {
        if g_seg <= 1e-9 {
            return self.bell.at(mean_xi);
        }
        let (nodes, w) = beta_pdf_nodes_weights(mean_xi, g_seg, NQ);
        w.iter().zip(nodes.iter()).fold(0.0, |a, (&wi, &x)| a + wi * self.bell.at(x))
    }

    /// ⟨EI⟩₁₅ = term1 + term2 — the production arithmetic, with both caches hoisted.
    fn ei15(&self, c: &QuenchPdf, j: f64) -> f64 {
        let cc = c.c(&mix(j));
        let term2 = c.dwell_factor(cc, TAU) * self.bell_pdf(self.xibar, c.segregation(cc));
        self.floor(j) + term2
    }
}

// ---------------------------------------------------------------------------------------
// GATE 1 — reduce: `pdf_quench = None` is rung 13; at `C_opt` it is the finite floor.
// ---------------------------------------------------------------------------------------

#[test]
fn reduce_no_pdf_quench_is_the_rung13_path() {
    let dp = design_point();
    for j in [9.0f64, 36.0] {
        let a = dp.g.zoned_nox(
            dp.far, dp.tt3, dp.tt4, dp.p, PHI_P,
            ZonedNoxOpts { mixing: Some(mix(j)), ..opts() },
        );
        let b = dp.g.zoned_nox(
            dp.far, dp.tt3, dp.tt4, dp.p, PHI_P,
            ZonedNoxOpts { mixing: Some(mix(j)), pdf_quench: None, ..opts() },
        );
        for s in [&a, &b] {
            assert!(s.pdf_quench.is_none());
            assert!(s.ei_no_pdf_quench.is_none() && s.ei_no_pdf_excess.is_none());
        }
        assert_eq!(a.ei_no_quenched.unwrap().to_bits(), b.ei_no_quenched.unwrap().to_bits());
        assert_eq!(a.max_a_quench.unwrap().to_bits(), b.max_a_quench.unwrap().to_bits());
    }
}

/// THE new reduce, and what separates rung 15 from rung 13: at `C_opt` the jet is perfectly mixed
/// (g = 0 ⇒ term 2 → 0), so ⟨EI⟩₁₅ is the FINITE mean-field bulk quench NO — not ≈0.
///
/// The residual term 2 is `D(0)·EI_bell(ξ̄)`, i.e. the dwell factor times a deep-lean point value,
/// which is negligible against the floor rather than exactly zero. Asserted as < 0.01 %.
#[test]
fn reduce_at_c_opt_is_the_finite_bulk_quench_no() {
    let dp = design_point();
    let c = cfg(0.0625);
    let s = dp.g.zoned_nox(
        dp.far, dp.tt3, dp.tt4, dp.p, PHI_P,
        ZonedNoxOpts { mixing: Some(mix(j_opt(&c))), pdf_quench: Some(c), ..opts() },
    );
    assert_eq!(s.g_seg.unwrap(), 0.0);
    assert!((s.c_holdeman.unwrap() - c.c_opt).abs() < 1e-12);
    let floor = s.ei_no_quenched.unwrap();
    assert!(floor > 0.3, "the bulk floor must be a FINITE non-trace value, got {floor}");
    let rel = (s.ei_no_pdf_quench.unwrap() - floor).abs() / floor;
    assert!(rel < 1e-4, "at C_opt ⟨EI⟩₁₅ must equal the finite bulk NO to <0.01 %, rel={rel:.2e}");
}

#[test]
fn zoned_nox_matches_the_ei15_helper() {
    let dp = design_point();
    let c = cfg(0.0625);
    let j = 36.0;
    let s = dp.g.zoned_nox(
        dp.far, dp.tt3, dp.tt4, dp.p, PHI_P,
        ZonedNoxOpts { mixing: Some(mix(j)), pdf_quench: Some(c), ..opts() },
    );
    let h = dp.ei15(&c, j);
    assert!(
        (s.ei_no_pdf_quench.unwrap() - h).abs() < 1e-9 * h.max(1e-12),
        "helper {h} vs production {}",
        s.ei_no_pdf_quench.unwrap()
    );
    assert!((s.c_holdeman.unwrap() - c.c(&mix(j))).abs() < 1e-12);
}

// ---------------------------------------------------------------------------------------
// GATE 2 — the FINITE floor: rung-13's ≈0 optimum becomes a finite bulk value.
// ---------------------------------------------------------------------------------------

#[test]
fn the_optimum_floor_is_finite_not_zero() {
    let dp = design_point();
    let c = cfg(0.0625);
    let m = mix(j_opt(&c));
    let s15 = dp.g.zoned_nox(
        dp.far, dp.tt3, dp.tt4, dp.p, PHI_P,
        ZonedNoxOpts { mixing: Some(m), pdf_quench: Some(c), ..opts() },
    );
    let s13 = dp.g.zoned_nox(
        dp.far, dp.tt3, dp.tt4, dp.p, PHI_P,
        ZonedNoxOpts {
            mixing: Some(m),
            pdf: Some(MixingPdf { s: 0.0625, n_bell: NB, n_quad: NQ, ..MixingPdf::default() }),
            ..opts()
        },
    );
    let (e15, e13) = (s15.ei_no_pdf_quench.unwrap(), s13.ei_no_pdf.unwrap());
    assert!(e15 > 0.3, "the rung-15 optimum floor must be the finite bulk NO, got {e15}");
    assert!(e13 < 1e-3, "the rung-13 optimum (ideal bell) is ≈0, got {e13}");
    assert!(
        e15 > 1e3 * e13.max(1e-12),
        "the ≈0 rung-13 floor must BECOME a finite, orders-larger bulk NO"
    );
}

// ---------------------------------------------------------------------------------------
// GATE 3/4 — the optimum PINNED AT `C_opt`, far flank CLIMBING, shifting as `(H/S)²`.
// ---------------------------------------------------------------------------------------

/// The finite floor sits AT `C_opt`, both immediate flanks lift, and — UNLIKE rung 13 — the far
/// over-penetration flank CLIMBS again: the dwell is restored, and it survives J→∞ because
/// `τ_core` is an ABSOLUTE residence rather than the vanishing jet time.
#[test]
fn the_optimum_is_pinned_at_c_opt_and_the_far_flank_climbs() {
    let dp = design_point();
    let c = cfg(0.0625);
    let jo = j_opt(&c);
    let (opt, under, over) = (dp.ei15(&c, jo), dp.ei15(&c, jo / 1.7), dp.ei15(&c, jo * 1.7));
    assert!(
        under > opt && over > opt,
        "both immediate flanks must lift above the C_opt floor: under={under}, opt={opt}, over={over}"
    );
    let (far1, far2) = (dp.ei15(&c, jo * 9.0), dp.ei15(&c, jo * 25.0));
    assert!(far2 > far1, "the far over-flank must CLIMB (restored dwell): {far1} → {far2}");
    assert!(
        far1 > 0.5 * opt && far2 > 0.5 * opt,
        "the far flank must stay ELEVATED, not collapse toward 0 the way rung 13's does"
    );
    let js = [jo / 4.0, jo / 1.7, jo, jo * 1.7, jo * 4.0, jo * 9.0, jo * 25.0];
    let eis: Vec<f64> = js.iter().map(|&j| dp.ei15(&c, j)).collect();
    assert_eq!(argmin(&eis), 2, "the global EI-min must sit AT J_opt={jo}: {eis:?}");
}

#[test]
fn the_optimum_shifts_as_h_over_s_squared() {
    let dp = design_point();
    for s in [0.0625f64, 0.0500] {
        let c = cfg(s);
        let jo = j_opt(&c);
        let js = [jo / 4.0, jo / 1.7, jo, jo * 1.7, jo * 4.0];
        let eis: Vec<f64> = js.iter().map(|&j| dp.ei15(&c, j)).collect();
        assert_eq!(argmin(&eis), 2, "S={s}: the EI-min must sit AT J_opt={jo}: {eis:?}");
    }
}

// ---------------------------------------------------------------------------------------
// GATE 5 — the STOICH-MEAN SIGN REVERSAL: the rung-12-in-disguise discriminator.
// ---------------------------------------------------------------------------------------

#[test]
fn term2_reverses_sign_at_a_stoichiometric_mean() {
    let dp = design_point();
    let xibar_st = f_stoich() / (1.0 + f_stoich());
    assert!(
        dp.bell_pdf(dp.xibar, 0.10) > 1e3 * dp.bell.at(dp.xibar),
        "lean mean: segregation must RAISE ⟨EI_bell⟩ by orders"
    );
    assert!(dp.bell.at(xibar_st) > 5.0, "sanity: the stoich point value sits near the peak");
    assert!(
        dp.bell_pdf(xibar_st, 0.10) < dp.bell.at(xibar_st),
        "stoich mean: segregation must LOWER ⟨EI_bell⟩ — a lumped dwell cannot reverse"
    );
}

// ---------------------------------------------------------------------------------------
// GATE 6 — the two kinks read the same `u`, and the dwell grows off-optimum.
// ---------------------------------------------------------------------------------------

#[test]
fn the_segregation_and_the_dwell_are_kinked_at_the_optimum() {
    let c = QuenchPdf::default();
    assert_eq!(c.segregation(c.c_opt), 0.0);
    assert_eq!(c.u(c.c_opt), 0.0);
    assert!(c.segregation(c.c_opt / 1.3) > 0.0 && c.segregation(c.c_opt * 1.3) > 0.0);
    assert!((c.segregation(c.c_opt / 1.4) - c.segregation(c.c_opt * 1.4)).abs() < 1e-12);
    assert!(c.segregation(c.c_opt * 1.05) > 0.0, "KINKED: a non-zero slope just off C_opt");
    assert_eq!(c.segregation(c.c_opt * 1e6), c.g_max);
    // The dwell factor is `τ_res/τ_ref` AT the optimum and GROWS on both flanks — its
    // off-optimum growth is exactly what makes the far flank climb.
    assert!((c.dwell_factor(c.c_opt, TAU) - c.tau_res / TAU).abs() < 1e-12);
    assert!(c.dwell_factor(c.c_opt * 1.3, TAU) > c.dwell_factor(c.c_opt, TAU));
    assert!(c.dwell_factor(c.c_opt / 1.3, TAU) > c.dwell_factor(c.c_opt, TAU));
}

// ---------------------------------------------------------------------------------------
// GATE 7/8 — cycle untouched, and the guards.
// ---------------------------------------------------------------------------------------

#[test]
fn the_cycle_is_untouched_by_a_pdf_quench_call() {
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
        ZonedNoxOpts { mixing: Some(mix(36.0)), pdf_quench: Some(cfg(0.0625)), ..opts() },
    );
    assert_eq!(run().station("4").far.to_bits(), far.to_bits());
}

#[test]
fn the_pdf_quench_closure_requires_a_mixing_config() {
    let r = std::panic::catch_unwind(|| {
        let (g, tt3, tt4, far, p) = design_state();
        g.zoned_nox(
            far, tt3, tt4, p, PHI_P,
            ZonedNoxOpts { pdf_quench: Some(QuenchPdf::default()), ..opts() },
        )
    });
    assert!(r.is_err(), "pdf_quench without mixing must be rejected");
}

#[test]
fn pdf_quench_is_exclusive_with_pdf_and_with_unmixedness() {
    for which in 0..2 {
        let r = std::panic::catch_unwind(move || {
            let (g, tt3, tt4, far, p) = design_state();
            let mut o = ZonedNoxOpts {
                mixing: Some(mix(16.0)),
                pdf_quench: Some(QuenchPdf::default()),
                ..opts()
            };
            if which == 0 {
                o.pdf = Some(MixingPdf::default());
            } else {
                o.unmixedness = Some(Unmixedness::default());
            }
            g.zoned_nox(far, tt3, tt4, p, PHI_P, o)
        });
        assert!(r.is_err(), "pdf_quench + closure #{which} must be rejected");
    }
}

#[test]
fn quench_pdf_positivity_guards() {
    QuenchPdf::default().validate();
    QuenchPdf { k_g: 0.0, ..QuenchPdf::default() }.validate(); // k_g=0 ⇒ floor only, allowed
    QuenchPdf { b_u: 0.0, ..QuenchPdf::default() }.validate(); // b_u=0 ⇒ flat dwell, allowed
    let bad = [
        QuenchPdf { s: 0.0, ..QuenchPdf::default() },
        QuenchPdf { s: -0.1, ..QuenchPdf::default() },
        QuenchPdf { c_opt: 0.0, ..QuenchPdf::default() },
        QuenchPdf { tau_res: 0.0, ..QuenchPdf::default() },
        QuenchPdf { k_g: -0.1, ..QuenchPdf::default() },
        QuenchPdf { b_u: -0.1, ..QuenchPdf::default() },
        QuenchPdf { g_max: 0.0, ..QuenchPdf::default() },
        QuenchPdf { g_max: 1.0, ..QuenchPdf::default() },
        QuenchPdf { n_bell: 1, ..QuenchPdf::default() },
        QuenchPdf { n_quad: 0, ..QuenchPdf::default() },
    ];
    for c in bad {
        let r = std::panic::catch_unwind(move || c.validate());
        assert!(r.is_err(), "QuenchPdf {c:?} should be rejected");
    }
}
