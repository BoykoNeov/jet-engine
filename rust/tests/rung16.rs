//! Rung-16 verification: the PDF through the finite quench, PER POCKET — retiring rung-15's one
//! acknowledged linearisation.
//!
//! Rung 15 carried the composition β-PDF through the dwell as `term 2 = D(u)·⟨EI_bell⟩(g)`: the
//! CONSTANT-T ideal bell times a SCALAR dwell factor, exact only while `EI ∝ τ` — which ignores
//! that a lingering pocket COOLS. Rung 16 carries EACH rich-of-mean pocket through its OWN finite
//! quench at the dwell `τ_core`, so the dwell acts INSIDE the cooling chemistry.
//!
//! ```text
//! ⟨EI⟩₁₆ = EI_bulk_quench(τ_mean)              [term 1: unchanged, the rung-11 finite floor]
//!        + ⟨EI_pocket_quench(ξ; τ_core(C))⟩_g  [term 2: PER-POCKET, the only thing that changes]
//! ```
//!
//! **What this file certifies, and what it deliberately does NOT.** The rung's own scope note is
//! that it does not relocate the optimum: which of the two near-degenerate wells is GLOBALLY
//! lowest flips across the β-PDF quadrature, the φ>2 tail treatment and the `C_e` regime, all
//! comparable to the margin. So there is **no argmin assertion anywhere in this file** — contrast
//! rung 15's, which has one. What is asserted is the structure the rung does certify: the
//! composition excess vanishing at `C_opt` with both flanks up, the SUBLINEARITY of term 2 in the
//! dwell (the mechanism), and the far flank FLATTENING against rung-15's linear climb.
//! `docs/rung16-spec.md`.
//!
//! **One gate here is Rust's and not the Python's.** Production splits the closure into a
//! `tau_core`-dependent pocket bank and a cheap β-integration over it, because the bank does not
//! depend on `g` — that is what makes a segregation sweep free. The Python has no such split, so
//! it cannot test it; [`the_pocket_bank_is_independent_of_the_segregation`] does, exactly, and
//! the Python's own "helper matches production" pin is dropped rather than transcribed into a
//! tautology (in Rust the helper and production would be the same function call).

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{equilibrium_composition, f_stoich, hf_fuel_default, Gas};
use turbojet::nox::{
    pocket_quench_grid, pocket_quench_integrate, pocket_quench_mean_ei, primary_aft, quench_no,
    quench_trajectory, thermal_no, JetMixing, MixingPdf, PocketOpts, PocketQuenchPdf, QuenchOpts,
    QuenchPdf, QuenchPoint, Unmixedness, ZonedNoxOpts,
};

const TAU: f64 = 3e-3;
const PHI_P: f64 = 1.5;
const CE: f64 = 0.20;
// The Python's own rung-16 grids. Per-pocket quench is ~n_bell× costlier than rung 15's single
// bell, so these are deliberately coarser than rung 13/15's.
const NG: usize = 24;
const NSTEPS: usize = 200;
const NB: usize = 48;
const NQ: usize = 64;

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
fn cfg() -> PocketQuenchPdf {
    PocketQuenchPdf { s: 0.0625, n_bell: NB, n_quad: NQ, ..PocketQuenchPdf::default() }
}
fn qp() -> QuenchPdf {
    QuenchPdf { s: 0.0625, n_bell: 120, n_quad: 160, ..QuenchPdf::default() }
}
fn popts() -> PocketOpts {
    PocketOpts { n_bell: NB, quench_ngrid: NG, quench_nsteps: NSTEPS, super_eq_o: false }
}
fn j_opt(c: &PocketQuenchPdf) -> f64 {
    let x = c.c_opt * JetMixing::default().h / c.s;
    x * x
}

struct Dp {
    g: Gas,
    tt3: f64,
    tt4: f64,
    far: f64,
    p: f64,
    comp_p: Vec<(&'static str, f64)>,
    t_p: f64,
    alpha: f64,
    n0: f64,
    tab: Vec<QuenchPoint>,
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
    Dp { g, tt3, tt4, far, p, comp_p, t_p, alpha, n0, tab }
}

impl Dp {
    /// Term 1 — the rung-11 mean-field bulk quench, shared with rung 15 and UNCHANGED here.
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

    /// Term 2 — the per-pocket quench β-PDF integral at this jet. Returns `(excess, max_a)`.
    fn term2(&self, c: &PocketQuenchPdf, j: f64) -> (f64, f64) {
        let cc = c.c(&mix(j));
        pocket_quench_mean_ei(
            self.far,
            self.tt3,
            self.p,
            hf_fuel_default(),
            TAU,
            c.core_dwell(cc),
            c.segregation(cc),
            c.n_quad,
            popts(),
        )
    }

    fn ei16(&self, c: &PocketQuenchPdf, j: f64) -> f64 {
        self.floor(j) + self.term2(c, j).0
    }

    /// Rung-15's term 2 at the same jet, from production — the linear comparator.
    fn term2_15(&self, j: f64) -> f64 {
        self.g
            .zoned_nox(
                self.far, self.tt3, self.tt4, self.p, PHI_P,
                ZonedNoxOpts { mixing: Some(mix(j)), pdf_quench: Some(qp()), ..opts() },
            )
            .ei_no_pdf_excess
            .expect("rung 15")
    }

    fn ei15(&self, j: f64) -> f64 {
        self.g
            .zoned_nox(
                self.far, self.tt3, self.tt4, self.p, PHI_P,
                ZonedNoxOpts { mixing: Some(mix(j)), pdf_quench: Some(qp()), ..opts() },
            )
            .ei_no_pdf_quench
            .expect("rung 15")
    }
}

// ---------------------------------------------------------------------------------------
// GATE 1 — reduce: `pocket_quench = None` is rung 15; at `C_opt` it is the finite floor.
// ---------------------------------------------------------------------------------------

#[test]
fn reduce_no_pocket_quench_is_the_rung15_path() {
    let dp = design_point();
    for j in [9.0f64, 36.0] {
        let a = dp.g.zoned_nox(
            dp.far, dp.tt3, dp.tt4, dp.p, PHI_P,
            ZonedNoxOpts { mixing: Some(mix(j)), ..opts() },
        );
        let b = dp.g.zoned_nox(
            dp.far, dp.tt3, dp.tt4, dp.p, PHI_P,
            ZonedNoxOpts { mixing: Some(mix(j)), pocket_quench: None, ..opts() },
        );
        for s in [&a, &b] {
            assert!(s.pocket_quench.is_none());
            assert!(s.ei_no_pocket_quench.is_none() && s.ei_no_pocket_excess.is_none());
        }
        assert_eq!(a.ei_no_quenched.unwrap().to_bits(), b.ei_no_quenched.unwrap().to_bits());
        assert_eq!(a.max_a_quench.unwrap().to_bits(), b.max_a_quench.unwrap().to_bits());
    }
}

/// At `C_opt` (g→0) term 2 collapses to the single pocket AT the mean — a deep-lean pocket, so
/// ≈0 — and the total is the FINITE bulk quench NO. Same reduce as rung 15's, and the same thing
/// that separates both from rung-13's ≈0 optimum.
#[test]
fn reduce_at_c_opt_is_the_finite_bulk_quench_no() {
    let dp = design_point();
    let c = cfg();
    let s = dp.g.zoned_nox(
        dp.far, dp.tt3, dp.tt4, dp.p, PHI_P,
        ZonedNoxOpts { mixing: Some(mix(j_opt(&c))), pocket_quench: Some(c), ..opts() },
    );
    assert_eq!(s.g_seg.unwrap(), 0.0);
    assert!((s.c_holdeman.unwrap() - c.c_opt).abs() < 1e-12);
    let floor = s.ei_no_quenched.unwrap();
    assert!(floor > 0.3, "the bulk floor must be a FINITE non-trace value, got {floor}");
    let rel = (s.ei_no_pocket_quench.unwrap() - floor).abs() / floor;
    assert!(rel < 1e-3, "at C_opt ⟨EI⟩₁₆ must equal the finite bulk NO to <0.1 %, rel={rel:.2e}");
}

/// Production really is `term1 + term2`, with term 1 computed independently.
///
/// This replaces the Python's "helper matches production" pin, which in Rust would compare a
/// function to itself. Here term 1 comes from a separately-built bulk quench on the shared
/// trajectory, so the equality is a statement about the WIRING and not a tautology.
#[test]
fn production_is_the_bulk_floor_plus_the_pocket_excess() {
    let dp = design_point();
    let c = cfg();
    let j = 36.0;
    let s = dp.g.zoned_nox(
        dp.far, dp.tt3, dp.tt4, dp.p, PHI_P,
        ZonedNoxOpts { mixing: Some(mix(j)), pocket_quench: Some(c), ..opts() },
    );
    let (excess, _) = dp.term2(&c, j);
    let total = dp.floor(j) + excess;
    assert!(
        (s.ei_no_pocket_quench.unwrap() - total).abs() < 1e-9 * total.max(1e-12),
        "wiring: production {} vs term1+term2 {total}",
        s.ei_no_pocket_quench.unwrap()
    );
    assert!(
        (s.ei_no_pocket_excess.unwrap() - excess).abs() < 1e-9 * excess.max(1e-12),
        "the reported excess must BE term 2"
    );
}

// ---------------------------------------------------------------------------------------
// A gate Rust can write and the Python cannot: the bank is `g`-independent, EXACTLY.
// ---------------------------------------------------------------------------------------

/// The pocket bank depends on `tau_core` and NOT on `g_seg`, so ONE build serves every width.
///
/// That is the split production is built around — and it is only worth having if it is exact, so
/// this asserts the split integral is BIT-IDENTICAL to the monolithic call at three widths, and
/// that the same bank really does serve all three. The Python rebuilds 48 quenches per width and
/// therefore has no way to state this.
#[test]
fn the_pocket_bank_is_independent_of_the_segregation() {
    let dp = design_point();
    let hf = hf_fuel_default();
    let tau_core = 4.0e-3;
    let bank = pocket_quench_grid(dp.far, dp.tt3, dp.p, hf, TAU, tau_core, popts());
    for g_seg in [0.0f64, 0.05, 0.12] {
        let split = pocket_quench_integrate(&bank, dp.far, g_seg, NQ);
        let (mono, mono_max_a) = pocket_quench_mean_ei(
            dp.far, dp.tt3, dp.p, hf, TAU, tau_core, g_seg, NQ, popts(),
        );
        assert_eq!(
            split.to_bits(),
            mono.to_bits(),
            "g={g_seg}: the g-independence lever must be EXACT, not approximate"
        );
        assert_eq!(bank.max_a.to_bits(), mono_max_a.to_bits());
    }
}

// ---------------------------------------------------------------------------------------
// GATE 2 — the composition excess vanishes AT `C_opt`, both immediate flanks up.
// ---------------------------------------------------------------------------------------

/// The `C_opt` notch SURVIVES the per-pocket quench: term 2 → 0 there (g→0, the single lean
/// pocket at ξ̄), and BOTH immediate flanks lift above the floor.
#[test]
fn the_excess_vanishes_at_c_opt_with_both_flanks_up() {
    let dp = design_point();
    let c = cfg();
    let jo = j_opt(&c);
    let e_opt = dp.ei16(&c, jo);
    let (exc_opt, _) = dp.term2(&c, jo);
    assert!(exc_opt < 0.01 * dp.floor(jo), "term 2 must vanish AT C_opt, got {exc_opt}");
    let (under, over) = (dp.ei16(&c, jo / 1.7), dp.ei16(&c, jo * 1.7));
    assert!(
        under > e_opt && over > e_opt,
        "both immediate flanks must lift above the C_opt notch: {under} > {e_opt} < {over}"
    );
}

// ---------------------------------------------------------------------------------------
// GATE 3 — THE MECHANISM: per-pocket cooling makes term 2 SUBLINEAR in the dwell.
// ---------------------------------------------------------------------------------------

/// Rung-15's term 2 scales EXACTLY with the dwell factor — that IS the linearisation, made
/// visible. Rung 16 carries the dwell inside the cooling quench, so its term 2 grows more slowly.
///
/// The comparison is a RATIO of two values from the same sweep against another ratio from the
/// same sweep, so nothing here is an absolute number to be remembered.
#[test]
fn the_per_pocket_dwell_is_sublinear() {
    let dp = design_point();
    let c = cfg();
    let (lo, hi) = (144.0f64, 625.0);
    let ratio_15 = dp.term2_15(hi) / dp.term2_15(lo);
    let ratio_16 = dp.term2(&c, hi).0 / dp.term2(&c, lo).0;
    let q = qp();
    let d_ratio = q.dwell_factor(q.c(&mix(hi)), TAU) / q.dwell_factor(q.c(&mix(lo)), TAU);
    assert!(
        (ratio_15 - d_ratio).abs() < 0.02 * d_ratio,
        "rung-15 term 2 must scale LINEARLY with the dwell: ratio={ratio_15}, D_ratio={d_ratio}"
    );
    assert!(
        ratio_16 < 0.95 * ratio_15,
        "rung-16 term 2 must be SUBLINEAR (the pocket cools): {ratio_16} vs {ratio_15}"
    );
}

// ---------------------------------------------------------------------------------------
// GATE 4/5 — the far over-penetration flank ERODES, and its CLIMB FLATTENS.
// ---------------------------------------------------------------------------------------

#[test]
fn the_far_flank_erodes_against_rung15() {
    let dp = design_point();
    let c = cfg();
    for j in [144.0f64, 225.0, 400.0, 625.0] {
        let (e16, e15) = (dp.ei16(&c, j), dp.ei15(j));
        assert!(e16 < 0.93 * e15, "J={j}: rung 16 must erode the far flank: {e16} vs {e15}");
    }
}

/// The resolution-robust face of the erosion: rung-15's LINEAR dwell makes the far flank CLIMB,
/// while rung-16's sublinear per-pocket dwell FLATTENS it. Same two endpoints, opposite slope.
///
/// The assertion is the SLOPE CONTRAST and never which well is the global minimum — that sits
/// inside the quadrature / tail / `C_e` ambiguity, which is exactly why this file has no argmin.
#[test]
fn the_far_flank_climb_flattens_against_rung15() {
    let dp = design_point();
    let c = cfg();
    let (lo, hi) = (144.0f64, 625.0);
    let climb15 = dp.ei15(hi) / dp.ei15(lo) - 1.0;
    let climb16 = dp.ei16(&c, hi) / dp.ei16(&c, lo) - 1.0;
    assert!(climb15 > 0.10, "rung 15's far flank must CLIMB (linear dwell): {climb15:.3}");
    assert!(
        climb16 < 0.5 * climb15,
        "rung 16's must FLATTEN (sublinear cooling): {climb16:.3} vs {climb15:.3}"
    );
}

// ---------------------------------------------------------------------------------------
// GATE 6 — the clamp stays dormant over EVERY pocket.
// ---------------------------------------------------------------------------------------

/// The per-pocket integrator is CLAMP-FREE — a super-equilibrium pocket would roll over — but at
/// this design point it is DORMANT: `max_a < 1` across the sweep AND across every pocket. So the
/// rung-15↔16 difference is COOLING within the dwell, not a super-equilibrium rollover.
#[test]
fn the_clamp_stays_dormant_over_every_pocket() {
    let dp = design_point();
    let c = cfg();
    for j in [16.0f64, 144.0, 625.0] {
        let s = dp.g.zoned_nox(
            dp.far, dp.tt3, dp.tt4, dp.p, PHI_P,
            ZonedNoxOpts { mixing: Some(mix(j)), pocket_quench: Some(c), ..opts() },
        );
        let a = s.max_a_quench.unwrap();
        assert!(a < 1.0, "J={j}: the dropped clamp must stay dormant (max_a={a})");
    }
}

// ---------------------------------------------------------------------------------------
// GATE 7/8/9 — cycle untouched, the guards, and the kinks.
// ---------------------------------------------------------------------------------------

#[test]
fn the_cycle_is_untouched_by_a_pocket_quench_call() {
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
        ZonedNoxOpts { mixing: Some(mix(36.0)), pocket_quench: Some(cfg()), ..opts() },
    );
    assert_eq!(run().station("4").far.to_bits(), far.to_bits());
}

#[test]
fn the_pocket_quench_closure_requires_a_mixing_config() {
    let r = std::panic::catch_unwind(|| {
        let (g, tt3, tt4, far, p) = design_state();
        g.zoned_nox(
            far, tt3, tt4, p, PHI_P,
            ZonedNoxOpts { pocket_quench: Some(PocketQuenchPdf::default()), ..opts() },
        )
    });
    assert!(r.is_err(), "pocket_quench without mixing must be rejected");
}

#[test]
fn pocket_quench_is_exclusive_with_the_other_three_closures() {
    for which in 0..3 {
        let r = std::panic::catch_unwind(move || {
            let (g, tt3, tt4, far, p) = design_state();
            let mut o = ZonedNoxOpts {
                mixing: Some(mix(16.0)),
                pocket_quench: Some(PocketQuenchPdf::default()),
                ..opts()
            };
            match which {
                0 => o.pdf = Some(MixingPdf::default()),
                1 => o.unmixedness = Some(Unmixedness::default()),
                _ => o.pdf_quench = Some(QuenchPdf::default()),
            }
            g.zoned_nox(far, tt3, tt4, p, PHI_P, o)
        });
        assert!(r.is_err(), "pocket_quench + closure #{which} must be rejected");
    }
}

#[test]
fn pocket_quench_pdf_positivity_guards() {
    PocketQuenchPdf::default().validate();
    PocketQuenchPdf { k_g: 0.0, ..PocketQuenchPdf::default() }.validate();
    PocketQuenchPdf { b_u: 0.0, ..PocketQuenchPdf::default() }.validate();
    let bad = [
        PocketQuenchPdf { s: 0.0, ..PocketQuenchPdf::default() },
        PocketQuenchPdf { s: -0.1, ..PocketQuenchPdf::default() },
        PocketQuenchPdf { c_opt: 0.0, ..PocketQuenchPdf::default() },
        PocketQuenchPdf { tau_res: 0.0, ..PocketQuenchPdf::default() },
        PocketQuenchPdf { k_g: -0.1, ..PocketQuenchPdf::default() },
        PocketQuenchPdf { b_u: -0.1, ..PocketQuenchPdf::default() },
        PocketQuenchPdf { g_max: 0.0, ..PocketQuenchPdf::default() },
        PocketQuenchPdf { g_max: 1.0, ..PocketQuenchPdf::default() },
        PocketQuenchPdf { n_bell: 1, ..PocketQuenchPdf::default() },
        PocketQuenchPdf { n_quad: 0, ..PocketQuenchPdf::default() },
    ];
    for c in bad {
        let r = std::panic::catch_unwind(move || c.validate());
        assert!(r.is_err(), "PocketQuenchPdf {c:?} should be rejected");
    }
}

/// `τ_core` is the ABSOLUTE dwell (no `τ_ref` ratio) — that is the whole difference from rung
/// 15's `dwell_factor`, and it is what lets the penalty survive J→∞.
#[test]
fn the_kinks_are_zero_at_the_optimum_and_the_dwell_is_absolute() {
    let c = PocketQuenchPdf::default();
    assert_eq!(c.segregation(c.c_opt), 0.0);
    assert_eq!(c.u(c.c_opt), 0.0);
    assert!(c.segregation(c.c_opt / 1.3) > 0.0 && c.segregation(c.c_opt * 1.3) > 0.0);
    assert!((c.segregation(c.c_opt / 1.4) - c.segregation(c.c_opt * 1.4)).abs() < 1e-12);
    assert!(c.segregation(c.c_opt * 1.05) > 0.0);
    assert_eq!(c.segregation(c.c_opt * 1e6), c.g_max);
    assert!((c.core_dwell(c.c_opt) - c.tau_res).abs() < 1e-12, "τ_core = τ_res AT C_opt");
    assert!(c.core_dwell(c.c_opt * 1.3) > c.core_dwell(c.c_opt));
    assert!(c.core_dwell(c.c_opt / 1.3) > c.core_dwell(c.c_opt));
}
