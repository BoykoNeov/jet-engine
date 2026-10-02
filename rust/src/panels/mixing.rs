//! Rungs 15–24's panels — the NOx and mixing strand's second half (slice AL).
//!
//! Each function is the `main.py` function of the same name (less the `print_` prefix), line
//! for line, plus `main.py`'s two local helpers `_mean_grad_sq` and `_j_opt_from`. Python's
//! `min(…, key=…)`, `max(…)`, `x ** 0.5` (a real `pow`, NOT `sqrt`), `x ** 2` (a multiply), `//`
//! and `%` on ints are spelled at each site; the byte gate does not see a last-bit slip that
//! printing rounds away (plan § 8.2), so those rules are applied by reading.

use super::nox::{dashes, design_quad, hf_of, jet, py_argmin, py_max, real_eq, rich_primary, total};
use super::Design;
use crate::gas::{equilibrium_composition, f_stoich, powp};
use crate::nox::{
    beta_pdf_nodes_weights, bell_interpolator, ideal_bell_ei, pdf_mean_ei, primary_aft, quench_no,
    quench_trajectory, spatial_dwell_field, spatial_local_field, spatial_segregation,
    super_eq_o_multiplier, thermal_no, transport_variance, try_primary_aft, two_stream_ceiling,
    Bell, ExhaustClampOpts, JetMixing, MixingPdf, PocketQuenchPdf, PromptNo, QuenchOpts,
    QuenchPdf, QuenchPoint, SpatialDwellPdf, SpatialLocalPdf, SpatialPdf, ThermalNoxOpts,
    TransportedPdf, ZonedNoxOpts,
};
use crate::pyf;
use crate::pyfmt::Printer;

/// Python's `min(xs)` over floats: the first minimum (`x < best` replaces).
pub(crate) fn py_min(xs: &[f64]) -> f64 {
    let mut best = xs[0];
    for &x in &xs[1..] {
        if x < best {
            best = x;
        }
    }
    best
}

/// `max(a, b)` of two floats: `a` unless `b > a`.
fn py_max2(a: f64, b: f64) -> f64 { if b > a { b } else { a } }
/// `min(a, b)` of two floats: `a` unless `b < a`.
fn py_min2(a: f64, b: f64) -> f64 { if b < a { b } else { a } }

/// `_j_opt_from(cfg)` — `(cfg.C_opt * JetMixing(J=1.0).H / cfg.S) ** 2`.
fn j_opt_from(c_opt: f64, s: f64) -> f64 {
    let q = c_opt * JetMixing { j: 1.0, ..JetMixing::default() }.h / s;
    q * q
}

/// `sum(wi * f(x) for x, wi in zip(nodes, w))` over a β-PDF's quadrature — a left fold, each term
/// `wi * f(x)` exactly as written.
fn pdf_sum(nodes: &[f64], w: &[f64], f: impl Fn(f64) -> f64) -> f64 {
    nodes.iter().zip(w).fold(0.0, |a, (&x, &wi)| a + wi * f(x))
}

/// `⟨EI_bell⟩` at mean `xb`, segregation `g`: the bell's point value at `g ≤ 1e-9`, else the
/// β-PDF integral.
fn bell_pdf_at(bell: &Bell, xb: f64, g_seg: f64, n_quad: usize) -> f64 {
    if g_seg <= 1e-9 {
        return bell.at(xb);
    }
    let (nodes, w) = beta_pdf_nodes_weights(xb, g_seg, n_quad);
    pdf_sum(&nodes, &w, |x| bell.at(x))
}

fn lean_line(tt3: f64, tt4: f64, far: f64, p: f64, xibar: f64) -> String {
    pyf!("  Design point: Tt3={:.0f} K, Tt4={:.0f} K, p={:.1f} bar, overall far={:.4f} (φ={:.2f}, LEAN); ξ̄={:.4f}",
         tt3, tt4, p / 1e5, far, far / f_stoich(), xibar)
}

/// `print_pdf_quench_table(flight)` — rung 15: the PDF through the finite quench.
pub fn pdf_quench_table(p: &mut Printer, d: &Design) {
    let (e, real) = real_eq(d);
    let (tt3, tt4, far, pp) = design_quad(&real);
    let hf = hf_of(&e.gas);
    let xibar = far / (1.0 + far);
    let ng = 80;

    p.print("\nPDF through the finite quench (rung 15): rungs 12–13 isolated the two mixing mechanisms —");
    p.print("the DWELL (rung 12: the far flank climbs) and the resolved COMPOSITION β-PDF (rung 13: the");
    p.print("optimum pinned AT C_opt, but ≈0 because it dropped the quench). Rung 15 COMBINES them: the ≈0");
    p.print("floor becomes the finite bulk NO, and the descending rung-13 far flank climbs again.");
    p.print(lean_line(tt3, tt4, far, pp, xibar));

    let phi_p = 1.5;
    let rp = rich_primary(phi_p, far, tt3, pp, hf, ng);
    let bell = bell_interpolator(pp, tt3, hf, 3e-3, 200, false);
    let floor_ei = |j: f64| {
        let m = jet(j, 2.0);
        let sched = |t: f64| m.schedule(t);
        quench_no(&rp.comp_p, rp.t_p, rp.alpha, far, tt3, pp, rp.n0, m.tau_q(),
                  QuenchOpts { tab: Some(&rp.tab), schedule: Some(&sched), ..QuenchOpts::default() }).ei
    };

    let qp = QuenchPdf { s: 0.0625, ..QuenchPdf::default() };
    let j_opt = j_opt_from(qp.c_opt, qp.s);
    p.print(pyf!("\n  J-sweep (S={} m → uniformity J_opt={:.0f}, C_opt={}); rich primary φ_p={}:", qp.s, j_opt, qp.c_opt, phi_p));
    p.print(pyf!("  {:>5} {:>6} {:>6} {:>9} {:>8} {:>8}   note", "J", "C", "g", "EI floor", "⟨EI⟩13", "⟨EI⟩15"));
    p.print(format!("  {}", dashes(60)));
    let mut rows: Vec<(f64, f64)> = Vec::new();
    for j in [4.0, 9.0, 16.0, 25.0, 36.0, 49.0, 100.0, 225.0, 400.0] {
        let c = qp.c(&JetMixing { j, ..JetMixing::default() });
        let g_seg = qp.segregation(c);
        let floor = floor_ei(j);
        let ei13 = bell_pdf_at(&bell, xibar, g_seg, 200);
        let ei15 = floor + qp.dwell_factor(c, 3e-3) * ei13;
        rows.push((j, ei15));
        let note = if (j - j_opt).abs() < 1e-9 { "OPTIMUM (g→0)" } else if j < j_opt { "under" } else { "over" };
        p.print(pyf!("  {:>5.0f} {:>6.2f} {:>6.3f} {:>9.4g} {:>8.4g} {:>8.4g}   {}", j, c, g_seg, floor, ei13, ei15, note));
    }
    let imin = py_argmin(&rows.iter().map(|r| r.1).collect::<Vec<_>>());
    p.print(pyf!("  The rung-15 minimum is a FINITE floor ({:.3g} g/kg) pinned AT J={:.0f}", rows[imin].1, rows[imin].0));
    p.print("  (C=C_opt) — NOT rung-13's ≈0. Both immediate flanks lift; and the far over-penetration flank");
    p.print("  CLIMBS again (J=100→400: the dwell restored, surviving strong jets) where rung-13's ⟨EI⟩13");
    p.print("  DESCENDS (bimodal PDF). The over-flank is non-monotone: the composition convexity jump near");
    p.print("  C_opt hands off to the dwell climb far out — BOTH parents' fingerprints, in one curve.");

    let xistoich = f_stoich() / (1.0 + f_stoich());
    p.print("\n  Why it is NOT rung 12 in disguise — the STOICH-MEAN SIGN REVERSAL (term 2's nonlinear bell):");
    p.print(pyf!("  {:>6} {:>15} {:>17}", "g", "⟨EI_bell⟩ lean", "⟨EI_bell⟩ stoich"));
    p.print(format!("  {}", dashes(40)));
    for g_seg in [0.0, 0.05, 0.10, 0.20] {
        p.print(pyf!("  {:>6.2f} {:>15.4g} {:>17.4g}", g_seg,
                     bell_pdf_at(&bell, xibar, g_seg, 200), bell_pdf_at(&bell, xistoich, g_seg, 200)));
    }
    p.print("  Segregation RAISES the bell integral at a LEAN mean but LOWERS it at a STOICH mean (mass off");
    p.print("  the peak). A dwell-only 'PDF through the quench' rides the ~linear EI_quench, so its variance");
    p.print("  has the WRONG sign and cannot reverse — this reversal certifies term 2 is genuine composition");
    p.print("  work. (Carrying the FULL per-pocket trajectory, not the bell×dwell-ratio, is the rung-16 seam.)");
}

/// One entry of rung 16's per-pocket bank: a bell value, or a pocket's own quench inputs.
enum Pocket {
    Bell(f64),
    Quench { comp: Vec<(&'static str, f64)>, t_pk: f64, al: f64, n0k: f64, tab: Vec<QuenchPoint> },
}

/// `print_pocket_quench_table(flight)` — rung 16: the PDF through the quench, per pocket.
pub fn pocket_quench_table(p: &mut Printer, d: &Design) {
    let (e, real) = real_eq(d);
    let (tt3, tt4, far, pp) = design_quad(&real);
    let hf = hf_of(&e.gas);
    let xibar = far / (1.0 + far);
    let (ng, tau) = (32usize, 3e-3);
    let fs = f_stoich();

    p.print("\nPDF through the finite quench, PER POCKET (rung 16): rung 15 combined composition + dwell,");
    p.print("but LINEARISED the dwell — it scaled a CONSTANT-T bell by D(u)=τ_core/τ_ref (exact only while");
    p.print("EI ∝ τ). Rung 16 carries EACH pocket through its OWN quench, so the dwell acts INSIDE the");
    p.print("cooling chemistry: term 2 goes SUBLINEAR and the far over-penetration flank ERODES.");
    p.print(lean_line(tt3, tt4, far, pp, xibar));

    let phi_p = 1.5;
    let far_p = phi_p * fs;
    let alpha = far / far_p;
    let t_p = primary_aft(far_p, pp, tt3, hf);
    let comp_p = equilibrium_composition(far_p, t_p, pp);
    let n0 = alpha * thermal_no(&comp_p, t_p, pp, tau, far_p, 4000, 1.0).x_no * total(&comp_p);
    let tab = quench_trajectory(&comp_p, t_p, alpha, far, tt3, pp, ng);
    let floor_ei = |j: f64| {
        let m = jet(j, 2.0);
        let sched = |t: f64| m.schedule(t);
        quench_no(&comp_p, t_p, alpha, far, tt3, pp, n0, m.tau_q(),
                  QuenchOpts { tab: Some(&tab), schedule: Some(&sched), ..QuenchOpts::default() }).ei
    };

    let bell = bell_interpolator(pp, tt3, hf, tau, 120, false);
    let term2_15 = |qp: &QuenchPdf, c: f64| {
        let g_seg = qp.segregation(c);
        let mb = bell_pdf_at(&bell, xibar, g_seg, 120);
        qp.dwell_factor(c, tau) * mb
    };

    const NB: usize = 48;
    const NQ: usize = 80;
    let xi_max = (2.0 * fs) / (1.0 + 2.0 * fs);
    let xi_grid: Vec<f64> = (0..NB).map(|i| xi_max * (i as f64 + 0.5) / NB as f64).collect();
    let mut bank: Vec<Pocket> = Vec::new();
    for &xi in &xi_grid {
        let fl = xi / (1.0 - xi);
        if fl < far || fl / fs > 2.0 + 1e-9 || fl <= 0.0 {
            bank.push(Pocket::Bell(ideal_bell_ei(fl, pp, tt3, hf, tau, false)));
            continue;
        }
        // Python: `try: _primary_aft(...) except AssertionError: bank.append(("b", 0.0))`.
        let t_pk = match try_primary_aft(fl, pp, tt3, hf) {
            Some(t) => t,
            None => {
                bank.push(Pocket::Bell(0.0));
                continue;
            }
        };
        let al = far / fl;
        let cp = equilibrium_composition(fl, t_pk, pp);
        let n0k = al * thermal_no(&cp, t_pk, pp, tau, fl, 4000, 1.0).x_no * total(&cp);
        let tabk = quench_trajectory(&cp, t_pk, al, far, tt3, pp, ng);
        bank.push(Pocket::Quench { comp: cp, t_pk, al, n0k, tab: tabk });
    }

    let term2_16 = |pqp: &PocketQuenchPdf, c: f64| {
        let g_seg = pqp.segregation(c);
        let tau_core = pqp.core_dwell(c);
        let vals: Vec<f64> = bank.iter().map(|b| match b {
            Pocket::Quench { comp, t_pk, al, n0k, tab } => quench_no(
                comp, *t_pk, *al, far, tt3, pp, *n0k, tau_core,
                QuenchOpts { tab: Some(tab), ..QuenchOpts::default() }).ei,
            Pocket::Bell(v) => *v,
        }).collect();
        let qb = |x: f64| -> f64 {
            if x <= xi_grid[0] {
                return vals[0];
            }
            if x >= xi_grid[NB - 1] {
                return 0.0;
            }
            let (mut lo, mut hi) = (0usize, NB - 1);
            while hi - lo > 1 {
                let mid = (lo + hi) / 2;
                if xi_grid[mid] <= x { lo = mid } else { hi = mid }
            }
            let t = (x - xi_grid[lo]) / (xi_grid[hi] - xi_grid[lo]);
            vals[lo] + t * (vals[hi] - vals[lo])
        };
        if g_seg <= 1e-9 {
            return qb(xibar);
        }
        let (nodes, w) = beta_pdf_nodes_weights(xibar, g_seg, NQ);
        pdf_sum(&nodes, &w, qb)
    };

    let qp = QuenchPdf { s: 0.0625, ..QuenchPdf::default() };
    let pqp = PocketQuenchPdf { s: 0.0625, ..PocketQuenchPdf::default() };
    let j_opt = j_opt_from(pqp.c_opt, pqp.s);
    p.print(pyf!("\n  J-sweep (S={} m → J_opt={:.0f}, C_opt={}); rich primary φ_p={}, C_e=0.20:", pqp.s, j_opt, pqp.c_opt, phi_p));
    p.print(pyf!("  {:>5} {:>6} {:>6} {:>9} {:>8} {:>8} {:>8}   note", "J", "C", "g", "EI floor", "⟨EI⟩15", "⟨EI⟩16", "erosion"));
    p.print(format!("  {}", dashes(66)));
    let cj = |j: f64| pqp.c(&JetMixing { j, ..JetMixing::default() });
    for j in [16.0, 36.0, 64.0, 144.0, 225.0, 400.0, 625.0] {
        let c = cj(j);
        let g_seg = pqp.segregation(c);
        let floor = floor_ei(j);
        let ei15 = floor + term2_15(&qp, c);
        let ei16 = floor + term2_16(&pqp, c);
        let ero = if j <= j_opt + 1e-9 { String::new() } else { pyf!("{:+.0f}%", (ei15 - ei16) / ei15 * 100.0) };
        let note = if (j - j_opt).abs() < 1e-9 { "OPTIMUM (g→0)" } else if j < j_opt { "under" } else { "over" };
        p.print(pyf!("  {:>5.0f} {:>6.2f} {:>6.3f} {:>9.4g} {:>8.4g} {:>8.4g} {:>8}   {}",
                     j, c, g_seg, floor, ei15, ei16, ero, note));
    }

    let (c_lo, c_hi) = (cj(144.0), cj(625.0));
    let r15 = term2_15(&qp, c_hi) / term2_15(&qp, c_lo);
    let r16 = term2_16(&pqp, c_hi) / term2_16(&pqp, c_lo);
    let (e15_lo, e15_hi) = (floor_ei(144.0) + term2_15(&qp, c_lo), floor_ei(625.0) + term2_15(&qp, c_hi));
    let (e16_lo, e16_hi) = (floor_ei(144.0) + term2_16(&pqp, c_lo), floor_ei(625.0) + term2_16(&pqp, c_hi));
    p.print(pyf!("\n  THE MECHANISM — term 2 vs dwell (J=144→625): rung-15 ×{:.2f} (LINEAR, = the dwell ratio)", r15));
    p.print(pyf!("  vs rung-16 ×{:.2f} (SUBLINEAR — each pocket COOLS through its quench). That cooling erodes", r16));
    p.print(pyf!("  the far flank: rung-15 CLIMBS ({:.3g}→{:.3g}, +{:.0f}%) but", e15_lo, e15_hi, (e15_hi / e15_lo - 1.0) * 100.0));
    p.print(pyf!("  rung-16 is FLAT ({:.3g}→{:.3g}, {:+.0f}%) — the over-penetration", e16_lo, e16_hi, (e16_hi / e16_lo - 1.0) * 100.0));
    p.print("  basin erodes into near-degeneracy with the C_opt notch (which SURVIVES: term 2 → 0 at C_opt).");
    p.print("  HONEST SCOPE: which of the two near-degenerate wells is the GLOBAL min is NOT claimed — it flips");
    p.print("  sign across the β-PDF quadrature (~5%), the φ>2 tail, and the C_e regime (2%→21% over 0.20→0.15).");
    p.print("  Rung 16 quantifies rung-15's linearisation error; it does not relocate the optimum. (Clamp");
    p.print("  DORMANT here, max_a<1 — the difference is cooling, not super-eq rollover.)");
}

/// `print_exhaust_clamp_table(flight)` — rung 17: the clamp through the mixing-fidelity ladder.
pub fn exhaust_clamp_table(p: &mut Printer, d: &Design) {
    let (e, real) = real_eq(d);
    let eq = &e.gas;
    let (st3, st4, st9) = (real.station("3"), real.station("4"), real.station("9"));
    let (far, tt3, tt4, pp) = (st4.far, st3.tt, st4.tt, st4.pt);
    let (j, phi_p) = (225.0, 1.5);
    let (ng, ns) = (32usize, 400usize);
    let pq = PocketQuenchPdf { s: 0.0625, n_bell: 40, n_quad: 120, ..PocketQuenchPdf::default() };
    let clamp = |c_e: f64| eq.exhaust_no_clamp(
        far, tt3, tt4, pp, st9.tt, st9.pt, real.p9, phi_p,
        JetMixing { j, c_e, shape_n: 2.0, ..JetMixing::default() }, pq,
        ExhaustClampOpts { quench_ngrid: ng, quench_nsteps: ns, ..ExhaustClampOpts::default() });

    p.print("\nExhaust-NO clamp through the mixing-fidelity ladder (rung 17): rung 14 fired the dropped");
    p.print("clamp on the φ_p=1.0 MIXED-OUT exhaust NO. At the RICH φ_p=1.5 RQL primary the mixed-out NO is");
    p.print("deceptively LOW — so the crude shortcut reads the clamp DORMANT — but the dilution re-making");
    p.print("(rung 11) and the near-stoich β-PDF pockets (rung 16) put NO back, frozen super-eq in the nozzle.");

    let s = clamp(0.20);
    p.print(pyf!("\n  Design point (rich φ_p={}, J={:.0f}): exhaust cools Tt9={:.0f} K → T9={:.0f} K,", phi_p, j, st9.tt, s.t9));
    p.print(pyf!("  equilibrium NO collapses {:.0f}× → the common clamp denominator x_no_e(T9).", s.no_collapse_ratio));
    p.print(pyf!("  {:>26} {:>13} {:>18}  verdict", "mixing-fidelity model", "exhaust x_no", "a=[NO]/[NO]_e(T9)"));
    p.print(format!("  {}", dashes(70)));
    p.print(pyf!("  {:>26} {:>13.3e} {:>18.3f}  {}", "MIXED-OUT (rung 8)", s.x_no_mixed_out, s.a_mixed_out,
                 if s.a_mixed_out < 1.0 { "DORMANT — hides the NO" } else { "fires" }));
    p.print(pyf!("  {:>26} {:>13.3e} {:>18.3f}  {}", "BULK QUENCH (rung 11)", s.x_no_bulk_quench, s.a_bulk_quench,
                 if s.a_bulk_quench > 1.0 { "FIRES — re-making" } else { "dormant" }));
    p.print(pyf!("  {:>26} {:>13.3e} {:>18.3f}  {}", "PER-POCKET (rung 16)", s.x_no_pocket, s.a_pocket,
                 if s.a_pocket > 1.0 { "FIRES — segregation lifts the mean" } else { "dormant" }));
    p.print(pyf!("  The ladder is MONOTONE in fidelity: a_mixed<1<a_bulk<a_pocket. The pocket/bulk ratio {:.2f} =",
                 s.gap_pocket_over_bulk));
    p.print("  rung-16's station-4 gap EXACTLY (the nozzle denominator cancels — algebra, a witnessed no-op).");

    let a_mixed = |phi: f64| {
        let zn = eq.zoned_nox(far, tt3, tt4, pp, phi, ZonedNoxOpts { tau: 3e-3, ..ZonedNoxOpts::default() });
        eq.nozzle_flow(far, tt4, pp, st9.tt, st9.pt, real.p9, Some(zn.x_no_mix)).max_a.unwrap()
    };
    let (a10, a15) = (a_mixed(1.0), a_mixed(1.5));
    p.print(pyf!("\n  The rung-14 contrast (same mixed-out-through-the-nozzle construction): φ_p=1.0 → a={:.0f}", a10));
    p.print(pyf!("  (FIRES, rung 14) but φ_p=1.5 → a={:.3f} (DORMANT). The RICH primary hides the NO the", a15));
    p.print("  mixed-out model never made — the same dropped-clamp lesson from the other side.");

    p.print("\n  Scale-sensitivity (C_e sweep, J=225) — the ORDERING is structural; magnitudes + the gap move:");
    p.print(pyf!("  {:>6} {:>8} {:>8} {:>9} {:>6}   ladder holds?", "C_e", "a_mixed", "a_bulk", "a_pocket", "gap"));
    for c_e in [0.15, 0.20] {
        let other;
        let sc = if c_e == 0.20 { &s } else { other = clamp(c_e); &other };
        let ok = if sc.ladder_monotone() && sc.a_mixed_out < 1.0 { "YES" } else { "NO" };
        p.print(pyf!("  {:>6.2f} {:>8.3f} {:>8.2f} {:>9.2f} {:>6.2f}   {}",
                     c_e, sc.a_mixed_out, sc.a_bulk_quench, sc.a_pocket, sc.gap_pocket_over_bulk, ok));
    }
    p.print("  HONEST SCOPE: the ORDERING (structural) + mixed-out dormancy are the certified claim. The");
    p.print("  FIRING (a>1) is IN-BAND, not universal — a fast quench (J→∞) drives a_bulk→a_mixed<1 (the");
    p.print("  rung-10 τ_q→0 reduce). a_bulk, a_pocket AND the gap ride on un-pinned scales (C_e, τ_res, H,");
    p.print(pyf!("  J); the clamp is DORMANT at station 4 (max_a={:.2f}) — the super-equilibrium is a", s.max_a_quench));
    p.print("  NOZZLE effect. Rung 17 is a synthesis of rungs 11/16/14, not new physics: it shows the crude");
    p.print("  mixed-out shortcut is UNCONSERVATIVE across the rich RQL operating band.");
}

/// `print_transported_variance_table(flight)` — rung 18: what a 0-D variance equation can derive.
pub fn transported_variance_table(p: &mut Printer, d: &Design) {
    let (e, real) = real_eq(d);
    let (tt3, tt4, far, pp) = design_quad(&real);
    let hf = hf_of(&e.gas);
    let (tau, phi_p) = (3e-3, 1.5);
    const NB: usize = 48;
    const NQ: usize = 80;
    let cfg = TransportedPdf { s: 0.0625, n_bell: NB, n_quad: NQ, ..TransportedPdf::default() };
    let kink = MixingPdf { s: 0.0625, n_bell: NB, n_quad: NQ, ..MixingPdf::default() };
    let j_opt = j_opt_from(cfg.c_opt, cfg.s);

    p.print("\nTransported-variance closure (rung 18): the deferred 'transported PDF' seam — derive g(C)");
    p.print("from a mixture-fraction variance equation instead of imposing the kink. The HONEST result is");
    p.print("a LIMIT: a 0-D transport CANNOT derive the C_opt optimum (it is irreducibly SPATIAL — the");
    p.print("rung-11/12 variance seam). What it CAN add: a DERIVED ceiling, a RESIDUAL floor, a smooth basin.");

    let g_ceiling = two_stream_ceiling(far, phi_p);
    let point = ideal_bell_ei(far, pp, tt3, hf, tau, false);
    p.print(pyf!("\n  Design point: Tt3={:.0f} K, Tt4={:.0f} K, overall far={:.4f} (φ={:.2f}, LEAN); rich primary φ_p={}",
                 tt3, tt4, far, far / f_stoich(), phi_p));
    p.print(pyf!("  DERIVED two-stream ceiling g_ceiling=(ξ_p−ξ̄)/(1−ξ̄)={:.4f} — set by φ_p, NOT a knob;", g_ceiling));
    p.print(pyf!("  rung-13's free g_max=0.30 is {:.1f}× larger (which is what let rung-13's ⟨EI⟩(g)", 0.30 / g_ceiling));
    p.print(pyf!("  reach its humped/descending regime). Residual floor g(C_opt)=g_ceiling·exp(−Da_opt)={:.4f} > 0.",
                 g_ceiling * (-cfg.da_opt).exp()));

    p.print("\n  (1) THE NEGATIVE RESULT — integrate the REAL variance ODE, read g at the combustor exit:");
    p.print(pyf!("  {:>5} {:>6} {:>10} {:>10} {:>10} {:>13}", "J", "C", "ω const", "ω∝√J", "ω∝J", "ω(C) spatial"));
    let js: [f64; 8] = [4.0, 9.0, 16.0, 25.0, 49.0, 100.0, 225.0, 625.0];
    let h = JetMixing { j: 1.0, ..JetMixing::default() }.h;
    let cols: [fn(f64) -> f64; 3] = [|_| 250.0, |j| 250.0 * (j / 16.0).sqrt(), |j| 250.0 * (j / 16.0)];
    let mut grids: [Vec<f64>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    let mut gcov: Vec<f64> = Vec::new();
    for &j in &js {
        let c = (cfg.s / h) * j.sqrt();
        for (k, om) in cols.iter().enumerate() {
            grids[k].push(transport_variance(g_ceiling, om(j), cfg.tau_mix, 2.0, 400));
        }
        gcov.push(transport_variance(g_ceiling, cfg.coverage_omega(c), cfg.tau_mix, cfg.c_phi, 400));
        p.print(pyf!("  {:>5.0f} {:>6.2f} {:>10.4f} {:>10.4f} {:>10.4f} {:>13.4f}",
                     j, c, grids[0].last().unwrap(), grids[1].last().unwrap(), grids[2].last().unwrap(), gcov.last().unwrap()));
    }
    let argmin_note = |vals: &[f64]| -> String {
        let i = py_argmin(vals);
        let (mx, mn) = (py_max(vals.iter().copied()), py_min(vals));
        if mx - mn <= 1e-4 * mx {
            return "FLAT — no optimum".to_string();
        }
        if 0 < i && i < vals.len() - 1 {
            pyf!("interior min J={:.0f}", js[i])
        } else {
            pyf!("monotone (min J={:.0f}) — no optimum", js[i])
        }
    };
    p.print(pyf!("  mean-field: const → {}; √J → {}; J → {}",
                 argmin_note(&grids[0]), argmin_note(&grids[1]), argmin_note(&grids[2])));
    p.print(pyf!("  spatial ω(C): {} — the optimum appears ONLY once S enters via C=(S/H)√J.", argmin_note(&gcov)));

    p.print(pyf!("\n  (2) THE SHAPE (S={} m → J_opt={:.0f}); transported g vs the imposed kink, through the ideal bell:",
                 cfg.s, j_opt));
    p.print(pyf!("  {:>5} {:>6} {:>7} {:>7} {:>9} {:>8}   note", "J", "C", "g_kink", "g_tr", "⟨EI⟩kink", "⟨EI⟩tr"));
    p.print(format!("  {}", dashes(64)));
    for j in [9.0, 16.0, 25.0, 49.0, 100.0, 225.0] {
        let c = cfg.c(&JetMixing { j, ..JetMixing::default() });
        let gk = kink.segregation(c);
        let (gt, _) = cfg.segregation(c, far, phi_p);
        let eik = pdf_mean_ei(far, tt3, pp, hf, tau, py_max2(gk, 1e-12), NB, NQ, false);
        let eit = pdf_mean_ei(far, tt3, pp, hf, tau, gt, NB, NQ, false);
        let note = if (j - j_opt).abs() < 1.0 { "← C_opt: kink DIVES to floor; transp ELEVATED" } else { "" };
        p.print(pyf!("  {:>5.0f} {:>6.2f} {:>7.4f} {:>7.4f} {:>9.5f} {:>8.5f}   {}", j, c, gk, gt, eik, eit, note));
    }
    p.print(pyf!("  The kink TOUCHES the well-mixed floor (≈{:.1e}) at C_opt (g→0); the transported basin", point));
    p.print("  sits ELEVATED (residual unmixedness). Both minima are AT C_opt — but the LOCATION is IMPOSED");
    p.print("  via ω(C) (proven in panel 1), while the SHARPNESS was the kink's artifact (any mixing rate");
    p.print("  rounds a corner). Transport tightens the closure (derived ceiling + residual floor) without");
    p.print("  over-claiming the one thing 0-D cannot reach: the spatial/CFD PDF stays the deferred ceiling.");
}

/// `print_super_eq_prompt_table(flight)` — rung 19: super-eq O and prompt NO.
pub fn super_eq_prompt_table(p: &mut Printer, d: &Design) {
    let (e, real) = real_eq(d);
    let eq = &e.gas;
    let (tt3, tt4, far, pp) = design_quad(&real);
    let tau = 3e-3;
    let prompt = PromptNo::default();

    p.print("\nSuper-equilibrium O & prompt NO (rung 19): every NO number since rung 7 read the rung-6");
    p.print("EQUILIBRIUM [O], so it is a LOWER BOUND. Two lifts — and BOTH refute 'the rich primary");
    p.print("explodes': super-eq O is T-driven (weakest when rich); prompt SURVIVES where thermal dies.");
    p.print(pyf!("\n  Design point: Tt3={:.0f} K, Tt4={:.0f} K, overall far={:.4f} (φ={:.2f}, LEAN).", tt3, tt4, far, far / f_stoich()));

    p.print("\n  (1) φ_p sweep — the two lifts on the primary (EI in g NO/kg fuel):");
    p.print(pyf!("  {:>5} {:>7} {:>9} {:>7} {:>10} {:>10} {:>9} {:>8}",
                 "φ_p", "T_p", "EI_therm", "m(T_p)", "EI_superO", "EI_prompt", "EI_total", "p/therm"));
    p.print(format!("  {}", dashes(74)));
    for phi_p in [0.8, 1.0, 1.1, 1.2, 1.3, 1.5] {
        let base = eq.zoned_nox(far, tt3, tt4, pp, phi_p, ZonedNoxOpts { tau, ..ZonedNoxOpts::default() });
        let lift = eq.zoned_nox(far, tt3, tt4, pp, phi_p, ZonedNoxOpts { tau, super_eq_o: true, ..ZonedNoxOpts::default() });
        let prm = eq.zoned_nox(far, tt3, tt4, pp, phi_p, ZonedNoxOpts { tau, prompt: Some(prompt), ..ZonedNoxOpts::default() });
        let ratio = if base.ei_no() > 0.0 { prm.ei_no_prompt / base.ei_no() } else { f64::INFINITY };
        let tot = lift.ei_no() + prm.ei_no_prompt;
        p.print(pyf!("  {:>5.2f} {:>7.0f} {:>9.4f} {:>7.3f} {:>10.4f} {:>10.4f} {:>9.4f} {:>8.2f}",
                     phi_p, base.t_primary, base.ei_no(), lift.o_multiplier, lift.ei_no(), prm.ei_no_prompt, tot, ratio));
    }
    p.print("  super-eq O: a MODEST T-driven lift (m falls as T rises), and its ABSOLUTE size COLLAPSES on");
    p.print("  the rich flank WITH thermal — it does NOT rescue the rich primary. prompt/thermal RISES");
    p.print("  monotonically rich (thermal dies, prompt persists): prompt is the rich-specific lift.");

    p.print("\n  (2) super-eq O multiplier m(T)=(C2/C1)·T·exp((θ1−θ2)/T) — Westenberg partial-eq/eq O:");
    let ts = [1800.0, 2000.0, 2200.0, 2400.0, 3000.0];
    let row_t: Vec<String> = ts.iter().map(|t| pyf!("{:>7.0f}", t)).collect();
    let row_m: Vec<String> = ts.iter().map(|&t| pyf!("{:>7.3f}", super_eq_o_multiplier(t))).collect();
    p.print(pyf!("  {:>7} ", "T (K)") + &row_t.join(" "));
    p.print(pyf!("  {:>7} ", "m(T)") + &row_m.join(" "));
    p.print("  DIMENSIONLESS (the shared [O2]^0.5 cancels) ⇒ a pure function of T, IDENTICAL across φ;");
    p.print("  DECREASING in T (→1 as T→∞). The lift is T-driven, NOT rich-driven — the intuition's first fail.");

    let far_s = f_stoich();
    let tn = |t: f64| eq.thermal_nox(far_s, t, pp, ThermalNoxOpts { tau, ..ThermalNoxOpts::default() }).ei_no;
    let t_rise = tn(2400.0) / tn(2000.0);
    let p_rise = prompt.ei_prompt(1.0, 2400.0) / prompt.ei_prompt(1.0, 2000.0);
    p.print(pyf!("\n  (3) T-sensitivity 2000→2400 K at stoich: thermal ×{:.0f} (DOUBLE exp: k1f·[O]_eq) vs prompt ×{:.0f} (SINGLE exp)",
                 t_rise, p_rise));
    p.print(pyf!("  ⇒ prompt is ~{:.0f}× MILDER — the quantitative face of 'survives where thermal dies'.", t_rise / p_rise));

    p.print("\n  TWO honest concessions (stated loudly; docs/rung19-spec.md § the concessions):");
    p.print("  • prompt MAGNITUDE is IMPOSED — a 0-D burnt pool has no flame structure to derive it; the");
    p.print(pyf!("    scale is back-solved from a REFERENCE EI≈{:.0f} g/kg at (φ={}, T={:.0f} K), so the delivered peak lands near ~{:.0f} g/kg — but a",
                 prompt.peak_ei, prompt.phi_ref, prompt.t_ref, prompt.peak_ei));
    p.print("    hotter primary (T_p>T_ref) nudges it up (single exp). Only the φ-SHAPE + the directional");
    p.print("    prompt/thermal ratio are certified. De Soete valid φ≤1.6 — the deep-rich flank is flagged.");
    p.print("  • super-eq O RATIO is semi-empirical — a full-equilibrium pool cannot self-yield super-eq O;");
    p.print("    the lift lives in Westenberg's fitted constants (cross-validated to ~5%, the units gate).");
}

/// `print_super_eq_quench_table(flight)` — rung 20: super-eq O through the quench.
pub fn super_eq_quench_table(p: &mut Printer, d: &Design) {
    let (e, real) = real_eq(d);
    let eq = &e.gas;
    let (st3, st4, st9) = (real.station("3"), real.station("4"), real.station("9"));
    let (far, tt3, tt4, pp) = (st4.far, st3.tt, st4.tt, st4.pt);
    let (j, phi_p, tau) = (225.0, 1.5, 3e-3);
    let (ng, ns) = (24usize, 200usize);
    let mix = jet(j, 2.0);
    let pq = PocketQuenchPdf { s: 0.0625, n_bell: 24, n_quad: 96, ..PocketQuenchPdf::default() };
    let zo = |o: ZonedNoxOpts| eq.zoned_nox(far, tt3, tt4, pp, phi_p, o);
    let q = |super_eq_o: bool, pocket: bool| ZonedNoxOpts {
        tau, mixing: Some(mix), super_eq_o, pocket_quench: if pocket { Some(pq) } else { None },
        quench_ngrid: ng, quench_nsteps: ns, ..ZonedNoxOpts::default()
    };

    p.print("\nSuper-equilibrium O THROUGH the quench (rung 20): rung 19 lifted the equilibrium-O lower");
    p.print("bound only on the PRIMARY. The finite-quench fields RE-MADE NO on equilibrium O — still lower");
    p.print("bounds. Rung 20 threads the m(T) lift INSIDE the _quench_no re-making, closing that seam.");
    p.print(pyf!("\n  Design point (rich φ_p={}, J={:.0f}): quench cools through the stoich crossing.", phi_p, j));

    let b0 = zo(q(false, false));
    let b_l = zo(q(true, false));
    let p0 = zo(q(false, true));
    let p_l = zo(q(true, true));
    let z0 = zo(ZonedNoxOpts { tau, ..ZonedNoxOpts::default() });
    let z_l = zo(ZonedNoxOpts { tau, super_eq_o: true, ..ZonedNoxOpts::default() });
    let (bq0, bql) = (b0.ei_no_quenched.unwrap(), b_l.ei_no_quenched.unwrap());
    let (pq0, pql) = (p0.ei_no_pocket_quench.unwrap(), p_l.ei_no_pocket_quench.unwrap());
    p.print("\n  (1) the effective lift (EI in g NO/kg fuel):");
    p.print(pyf!("  {:>26} {:>9} {:>11} {:>7}", "field", "eq-O", "super-eq-O", "factor"));
    p.print(format!("  {}", dashes(56)));
    p.print(pyf!("  {:>26} {:>9.4f} {:>11.4f} {:>7.3f}", "ei_no_quenched (bulk)", bq0, bql, bql / bq0));
    p.print(pyf!("  {:>26} {:>9.4f} {:>11.4f} {:>7.3f}", "ei_no_pocket_quench (r16)", pq0, pql, pql / pq0));
    p.print(pyf!("  {:>26} {:>9.4f} {:>11.4f} {:>7.3f}", "primary ei_no (rung 19)", z0.ei_no(), z_l.ei_no(), z_l.ei_no() / z0.ei_no()));

    let t_peak = b0.t_peak.unwrap();
    p.print(pyf!("\n  (2) the quench lift ({:.3f}) is SMALLER than the primary lift ({:.3f}) because the",
                 bql / bq0, z_l.ei_no() / z0.ei_no()));
    p.print(pyf!("  Zeldovich re-making peaks at the HOTTEST crossing T_peak={:.0f} K (hotter than the", t_peak));
    p.print(pyf!("  flame T_p={:.0f} K), and m(T) is SMALLEST where hottest: m(T_peak)={:.3f} vs m(T_p)={:.3f}.",
                 z0.t_primary, super_eq_o_multiplier(t_peak), super_eq_o_multiplier(z0.t_primary)));
    p.print(pyf!("  The cool tail carries large m (m({:.0f} K)={:.3f}) but makes NEGLIGIBLE NO — the lift is PEAK-concentrated.",
                 tt4, super_eq_o_multiplier(tt4)));

    let clamp = |super_eq_o: bool| eq.exhaust_no_clamp(
        far, tt3, tt4, pp, st9.tt, st9.pt, real.p9, phi_p, mix, pq,
        ExhaustClampOpts { super_eq_o, quench_ngrid: ng, quench_nsteps: ns, ..ExhaustClampOpts::default() });
    let c0 = clamp(false);
    let c_l = clamp(true);
    p.print("\n  (3) the CERTIFIED SPINE — the rung-17 clamp margins a=[NO]/[NO]_e(T9) rise, denominator fixed:");
    p.print(pyf!("  {:>22} {:>9} {:>11}", "margin", "eq-O", "super-eq-O"));
    p.print(pyf!("  {:>22} {:>9.4f} {:>11.4f}   (primary lift; stays ≪1)", "a_mixed_out", c0.a_mixed_out, c_l.a_mixed_out));
    p.print(pyf!("  {:>22} {:>9.3f} {:>11.3f}", "a_bulk_quench", c0.a_bulk_quench, c_l.a_bulk_quench));
    p.print(pyf!("  {:>22} {:>9.3f} {:>11.3f}", "a_pocket", c0.a_pocket, c_l.a_pocket));
    p.print(pyf!("  {:>22} {:>9.3e} {:>11.3e}   {}", "denom x_no_e(T9)", c0.x_no_e_exit, c_l.x_no_e_exit,
                 if c0.x_no_e_exit == c_l.x_no_e_exit { "← BIT-IDENTICAL" } else { "← MOVED (bug)" }));
    p.print("  The numerators lift (kinetic NO); the denominator is a THERMODYNAMIC ceiling Kp_NO·√(x_N2·x_O2)");
    p.print("  — NOT set by the O-atom closure — so every a rises by a BOUNDED factor. rung 17's a were lower");
    p.print(pyf!("  bounds. But the clamp still does NOT fire at station 4: max_a={:.2f}<1 — super-eq O", c_l.max_a_quench));
    p.print("  speeds FORMATION, not the [NO]_e collapse, so it is not the burner-clamp lever (a slow freeze is).");

    p.print("\n  HONEST SCOPE: the super-eq RATIO stays semi-empirical (rung 19) ⇒ the lifted a is");
    p.print("  better-justified but NOT pinned. Prompt rides the quench as an INVARIANT per-kg-fuel EI");
    p.print("  (kept OUT of a — imposed magnitude). The ideal-bell PDF integrals (rung 13/15/18) stay");
    p.print("  equilibrium-O here — RUNG 21 lifts them consistently (see the next panel).");
}

/// `print_ideal_bell_lift_table(flight)` — rung 21: super-eq O through the ideal-bell integrals.
pub fn ideal_bell_lift_table(p: &mut Printer, d: &Design) {
    let (e, real) = real_eq(d);
    let eq = &e.gas;
    let (tt3, tt4, far, pp) = design_quad(&real);
    let hf = hf_of(eq);
    let (j, phi_p, tau) = (36.0, 1.5, 3e-3);
    let (ng, nb, nq) = (24usize, 80usize, 100usize);
    let mix = JetMixing { j, ..JetMixing::default() };
    let fs = f_stoich();

    p.print("\nSuper-equilibrium O through the IDEAL-BELL PDF integrals (rung 21): rung 20 lifted everything");
    p.print("through _quench_no but LEFT the composition integrals (pdf r13 / pdf_quench-term2 r15 /");
    p.print("transported r18) on equilibrium O — a FORBIDDEN combination (a half-lifted hybrid). Rung 21");
    p.print("threads m(T) through the ideal bell too, so BOTH pdf_quench terms lift and the hybrid dissolves.");
    p.print(pyf!("\n  Design point (lean mean φ≈{:.2f}, rich primary φ_p={}, J={:.0f}, g>0).", far / fs, phi_p, j));

    let run = |o: ZonedNoxOpts| eq.zoned_nox(far, tt3, tt4, pp, phi_p,
                                            ZonedNoxOpts { tau, mixing: Some(mix), quench_ngrid: ng, ..o });
    let pd = ZonedNoxOpts { pdf: Some(MixingPdf { s: 0.0625, n_bell: nb, n_quad: nq, ..MixingPdf::default() }), ..ZonedNoxOpts::default() };
    let qd = ZonedNoxOpts { pdf_quench: Some(QuenchPdf { s: 0.0625, n_bell: nb, n_quad: nq, ..QuenchPdf::default() }), ..ZonedNoxOpts::default() };
    let td = ZonedNoxOpts { transported: Some(TransportedPdf { s: 0.0625, n_bell: nb, n_quad: nq, ..TransportedPdf::default() }), ..ZonedNoxOpts::default() };
    let lifted = |o: ZonedNoxOpts| ZonedNoxOpts { super_eq_o: true, ..o };
    let (p0, p_l) = (run(pd), run(lifted(pd)));
    let (q0, q_l) = (run(qd), run(lifted(qd)));
    let (t0, t_l) = (run(td), run(lifted(td)));
    let (b0, b_l) = (run(ZonedNoxOpts::default()), run(lifted(ZonedNoxOpts::default())));
    let (pdf0, pdfl) = (p0.ei_no_pdf.unwrap(), p_l.ei_no_pdf.unwrap());
    let (pq0, pql) = (q0.ei_no_pdf_quench.unwrap(), q_l.ei_no_pdf_quench.unwrap());
    let (tr0, trl) = (t0.ei_no_transported.unwrap(), t_l.ei_no_transported.unwrap());

    p.print("\n  (1) the effective ideal-bell lift (EI in g NO/kg fuel):");
    p.print(pyf!("  {:>28} {:>9} {:>11} {:>7}", "field", "eq-O", "super-eq-O", "factor"));
    p.print(format!("  {}", dashes(58)));
    p.print(pyf!("  {:>28} {:>9.4f} {:>11.4f} {:>7.3f}", "ei_no_pdf (rung 13)", pdf0, pdfl, pdfl / pdf0));
    p.print(pyf!("  {:>28} {:>9.4f} {:>11.4f} {:>7.3f}", "ei_no_pdf_quench (rung 15)", pq0, pql, pql / pq0));
    p.print(pyf!("  {:>28} {:>9.4f} {:>11.4f} {:>7.3f}", "ei_no_transported (rung 18)", tr0, trl, trl / tr0));

    let lift_point = ideal_bell_ei(far, pp, tt3, hf, tau, true) / ideal_bell_ei(far, pp, tt3, hf, tau, false);
    let fl_p = phi_p * fs;
    let lift_primary = ideal_bell_ei(fl_p, pp, tt3, hf, tau, true) / ideal_bell_ei(fl_p, pp, tt3, hf, tau, false);
    let lift_pdf = pdfl / pdf0;
    p.print("\n  (2) WHY it is PEAK-CONCENTRATED — the bell EI is peaked near stoich (hottest ⇒ m smallest):");
    let l1 = format!("deep-lean point value (g→0, φ≈{})", pyf!("{:.2f}", far / fs));
    let l2 = format!("primary flame (φ_p={})", pyf!("{:.1f}", phi_p));
    p.print(pyf!("  {:>40}  lift ×{:.2f}   (cool flame ⇒ m LARGE, but EI≈0)", l1, lift_point));
    p.print(pyf!("  {:>40}  lift ×{:.2f}   (rung 19)", l2, lift_primary));
    p.print(pyf!("  {:>40}  lift ×{:.2f}   (SMALLEST — onto the stoich peak)", "EI-weighted ⟨EI⟩_pdf (the number that counts)", lift_pdf));
    p.print("  The naive 'the bell spans cool deep-lean pockets where m→1.9' is WRONG: those carry ≈0 EI.");

    let lift_composite = pql / pq0;
    let lift_bulk = b_l.ei_no_quenched.unwrap() / b0.ei_no_quenched.unwrap();
    p.print(pyf!("\n  (3) the HYBRID RESOLVED — pdf_quench composite ×{:.3f} sits BETWEEN term1", lift_composite));
    p.print(pyf!("  (bulk quench ×{:.3f}, rung 20) and term2 (ideal bell ×{:.3f}, rung 21) —", lift_bulk, lift_pdf));
    p.print("  the measured proof BOTH terms now carry m(T). rung 20's forbidden combination is now VALID.");

    p.print("\n  HONEST SCOPE: a shape-preserving CONSISTENCY lift — the optimum LOCATION (pinned AT C_opt),");
    p.print("  the (H/S)² shift and the stoich-mean sign reversal are UNMOVED; only the magnitude lifts ≈×1.15.");
    p.print("  The super-eq RATIO stays semi-empirical (rung 19); prompt stays a primary-only invariant EI.");
}

/// `JetMixing(J=float(J), H=0.10)` — rung 22's jet (default `C_e`).
fn jet_h(j: f64) -> JetMixing { JetMixing { j, h: 0.10, ..JetMixing::default() } }
/// `JetMixing(J=float(J), C_e=0.20, U_c=75.0, H=0.10)` — rungs 23–24's jet.
fn jet_full(j: f64) -> JetMixing { JetMixing { j, c_e: 0.20, u_c: 75.0, h: 0.10, ..JetMixing::default() } }

/// `print_spatial_pdf_table(flight)` — rung 22: the resolved cross-plane.
pub fn spatial_pdf_table(p: &mut Printer, d: &Design) {
    let (e, real) = real_eq(d);
    let eq = &e.gas;
    let (tt3, tt4, far, pp) = design_quad(&real);
    let (phi_p, tau) = (1.5, 3e-3);
    let (ng, nb, nq) = (24usize, 64usize, 160usize);
    let k_p = SpatialPdf::default().k_p;
    let gceil = two_stream_ceiling(far, phi_p);

    p.print("\nResolved cross-plane / spatial PDF (rung 22): the INVERSION of rung 18. Rung 18 proved a 0-D");
    p.print("variance transport CANNOT derive the Holdeman C_opt (mean-field ⇒ monotone g(J)); it had to");
    p.print("IMPOSE the coverage ω(C) — the spacing S by hand. Rung 22 RESOLVES the y-z cross-plane and C_opt");
    p.print("comes out as an OUTPUT: δ=k_p·√(S·H)·J^(1/4) couples S in; δ fills half-height ⇒ (S/H)√J=1/(4k_p²).");
    p.print("\n  (1) THE COLLAPSE — vary S and H INDEPENDENTLY; g_min is geometry-independent, J_opt ∝ (H/S)²,");
    p.print(pyf!("      C_opt=1/(4·k_p²)={:.3f} an OUTPUT (Holdeman's ≈2.5). No C_opt is fed in.", 1.0 / (4.0 * (k_p * k_p))));
    p.print(pyf!("  {:>8} {:>6} {:>7} {:>11} {:>8}  note", "S", "H", "J_opt", "C_opt(OUT)", "g_min"));
    p.print(format!("  {}", dashes(62)));
    let js: Vec<f64> = (0..61).map(|i| 1.0 * powp(400.0, i as f64 / 60.0)).collect();
    let notes = [((0.0625, 0.10), "baseline"), ((0.03125, 0.10), "halve S ⇒ J_opt ×4"),
                 ((0.125, 0.10), "double S ⇒ J_opt ÷4"), ((0.0625, 0.20), "double H ⇒ J_opt ×4"),
                 ((0.125, 0.20), "S/H fixed ⇒ unchanged")];
    for ((s_sp, h), note) in notes {
        let gs: Vec<f64> = js.iter()
            .map(|&j| spatial_segregation(far, phi_p, s_sp, h, j, 0.316, 0.28, 0.28, 48, 48))
            .collect();
        let i = py_argmin(&gs);
        let (gmin, jo) = (gs[i], js[i]);
        p.print(pyf!("  {:8.4f} {:6.2f} {:7.2f} {:11.3f} {:8.4f}  {}", s_sp, h, jo, (s_sp / h) * powp(jo, 0.5), gmin, note));
    }
    p.print("  ⇒ the g_min VALUE is the same everywhere (the collapse); only the LOCATION J_opt shifts as (H/S)².");

    p.print(pyf!("\n  (2) g_spatial < g_ceiling={:.4f} (rung-18 two-stream ceiling) — a partial-mix field is", gceil));
    p.print("      LESS segregated than the two-δ extreme (the one thing rung 18 derived bounds the resolved g):");
    let sp = SpatialPdf { s: 0.0625, n_bell: nb, n_quad: nq, ..SpatialPdf::default() };
    let zs = |j: i64| eq.zoned_nox(far, tt3, tt4, pp, phi_p, ZonedNoxOpts {
        tau, mixing: Some(jet_h(j as f64)), spatial: Some(sp), quench_ngrid: ng, ..ZonedNoxOpts::default()
    });
    for j in [1i64, 16, 400] {
        let st = zs(j);
        p.print(pyf!("      J={:>4}  g_spatial={:.4f}  (< {:.4f})", j, st.g_spatial.unwrap(), gceil));
    }

    p.print("\n  (3) EMISSIONS (honest): through the ideal bell, C_opt (J_opt=16) is only a LOCAL ⟨EI⟩ min;");
    p.print("      the GLOBAL min is at max segregation (rung-13's descending far flank, spatialized):");
    p.print(pyf!("  {:>6} {:>6} {:>10} {:>14}  flank", "J", "C", "g_spatial", "ei_no_spatial"));
    p.print(format!("  {}", dashes(52)));
    let mut ei: Vec<(i64, f64)> = Vec::new();
    let mut g_floor = None;
    for j in [9i64, 16, 25, 100, 400] {
        let st = zs(j);
        let (gsp, eis) = (st.g_spatial.unwrap(), st.ei_no_spatial.unwrap());
        ei.push((j, eis));
        if j == 16 {
            g_floor = Some(gsp);
        }
        let c = (0.0625 / 0.10) * powp(j as f64, 0.5);
        let flank = if j == 16 { "C_opt (local min)" } else if j == 9 || j == 25 { "immediate flank ↑" } else { "far over-pen ↓" };
        p.print(pyf!("  {:6d} {:6.2f} {:10.4f} {:14.4f}  {}", j, c, gsp, eis, flank));
    }
    let amin = ei[py_argmin(&ei.iter().map(|e| e.1).collect::<Vec<_>>())].0;
    p.print(pyf!("  ⇒ local min AT J=16 (both immediate flanks up), but the GLOBAL min is at J={} (an ENDPOINT):", amin));
    p.print(pyf!("     the derived floor g(C_opt)≈{:.3f} sits JUST below the ideal-bell hump peak", g_floor.unwrap()));
    p.print("     (≈0.021), so the C_opt basin is NARROW — which is WHY UNIFORMITY (g), not emissions, is the");
    p.print("     headline. rung 18 was NOT wrong: it reported the real LOCAL behaviour.");

    p.print("\n  HONEST SCOPE: the VALUE C_opt≈2.5 rides on the semi-empirical k_p (only the COLLAPSE + the");
    p.print("  (H/S)² shift are derived); rung 22 derives the WIDTH g(C), NOT the DWELL (rung-16 kink imported);");
    p.print("  the field is a Gaussian-plume cartoon feeding the β-PDF closure — a real PDF-transport/CFD");
    p.print("  cross-plane (the full shape + the dwell spectrum + rung-17's firing MAGNITUDE) stays the ceiling.");
}

/// `print_dwell_spectrum_table(flight)` — rung 23: the derived dwell spectrum.
pub fn dwell_spectrum_table(p: &mut Printer, d: &Design) {
    let (e, real) = real_eq(d);
    let eq = &e.gas;
    let (tt3, tt4, far, pp) = design_quad(&real);
    let (phi_p, tau) = (1.5, 3e-3);
    let (ng, nb, nq) = (24usize, 40usize, 120usize);
    let cfg = SpatialDwellPdf { s: 0.0625, ny: 32, nz: 32, nt: 24, n_bell: nb, n_quad: nq, ..SpatialDwellPdf::default() };

    p.print("\nDerived dwell spectrum (rung 23): completing rung-22's PARTIAL closure. Rung 22 derived the");
    p.print("cross-plane WIDTH g(C) but imported rung-16's KINKED scalar dwell (which bakes C_opt in). Rung 23");
    p.print("develops the SAME cross-plane in TIME (over rung-11's τ_mix) so each pocket carries its OWN dwell");
    p.print("τ(ξ) — rich pockets arrive LATE ⇒ dwell LONG. NO C_opt, NO τ_res, NO b_u; the scale is rung-11's τ_mix.");

    p.print("\n  (1) THE ξ–τ CORRELATION (the certified positive) — the MATCHED-MEAN twin isolates it: term2 with");
    p.print("      the correlated τ(ξ) vs term2 with a scalar ⟨τ⟩_PDF (same g, same mean dwell). corr_ratio>1 ⇒");
    p.print("      rich pockets dwell long ⇒ ADD NO. Concentrated under-penetration, fading toward C_opt:");
    p.print(pyf!("  {:>5} {:>6} {:>7} {:>10} {:>9} {:>9} {:>9} {:>9} {:>6}",
                 "J", "C", "g", "τ_mix(ms)", "⟨τ⟩(ms)", "ei_corr", "ei_mean", "corr/mean", "max_a"));
    p.print(format!("  {}", dashes(84)));
    for j in [4i64, 9, 16, 64, 400] {
        let st = eq.zoned_nox(far, tt3, tt4, pp, phi_p, ZonedNoxOpts {
            tau, mixing: Some(jet_full(j as f64)), spatial_dwell: Some(cfg), quench_ngrid: ng, ..ZonedNoxOpts::default()
        });
        let c = (0.0625 / 0.10) * powp(j as f64, 0.5);
        let taumix = jet_full(j as f64).tau_q();
        let note = if j < 16 { "under-pen" } else if j == 16 { "C_opt" } else { "over-pen" };
        p.print(pyf!("  {:5d} {:6.2f} {:7.4f} {:10.3f} {:9.4f} {:9.4f} {:9.4f} {:9.4f} {:6.3f}  {}",
                     j, c, st.g_spatial_dwell.unwrap(), taumix * 1e3, st.tau_mean_dwell.unwrap() * 1e3,
                     st.ei_no_spatial_dwell.unwrap(), st.ei_no_spatial_dwell_meanfield.unwrap(),
                     st.corr_ratio.unwrap(), st.max_a_quench.unwrap(), note));
    }
    p.print("  ⇒ corr/mean > 1 EVERYWHERE (the correlation adds NO), largest under-penetration; max_a<1 (formation-");
    p.print("    limited, so the sign never flips). This is the physics rung-16's SCALAR τ_core cannot express.");

    p.print("\n  (2) THE DIVERGENCE (honest) — rung-16's imposed τ_core GROWS off-optimum (|ln(C/C_opt)|); the");
    p.print("      derived ⟨τ⟩ FALLS ∝1/√J (rung-11 mixing time). ~3×–70× apart, opposite-trending, BOTH un-anchored:");
    let cfg16 = PocketQuenchPdf { s: 0.0625, ..PocketQuenchPdf::default() };
    p.print(pyf!("  {:>5} {:>6} {:>17} {:>15} {:>7}", "J", "C", "rung16 τ_core(ms)", "rung23 ⟨τ⟩(ms)", "ratio"));
    p.print(format!("  {}", dashes(54)));
    for j in [4i64, 16, 64, 400] {
        let m = jet_full(j as f64);
        let c = cfg16.c(&m);
        let t16 = cfg16.core_dwell(c);
        let (g_s, tau_of) = spatial_dwell_field(far, phi_p, cfg.s, m.h, m.j, m.tau_q(),
                                                cfg.k_p, cfg.k_y, cfg.k_z, 32, 32, 24);
        let xibar = far / (1.0 + far);
        let (nodes, wts) = beta_pdf_nodes_weights(xibar, g_s, nq);
        let t23 = wts.iter().zip(&nodes).fold(0.0, |a, (&wi, &x)| a + wi * tau_of.at(x));
        p.print(pyf!("  {:5d} {:6.2f} {:17.4f} {:15.4f} {:7.1f}", j, c, t16 * 1e3, t23 * 1e3, t16 / t23));
    }

    p.print("\n  HONEST SCOPE: rung 23 derives the correlation's SHAPE (sign + under-penetration concentration);");
    p.print("  the absolute MAGNITUDE/TREND rides on rung-11's un-anchored τ_mix (as rung-22's C_opt≈2.5 rides on");
    p.print("  k_p). The emissions C_opt pin is NOT recovered (the derived τ falls off-optimum) — but rung 16");
    p.print("  ALREADY declined the global-min location. A LOCALLY-resolved mixing time (each cell its own rate)");
    p.print("  — plus the full CFD-PDF shape, which would let rung 17 claim a firing MAGNITUDE — stays the ceiling.");
}

/// `_mean_grad_sq(far_overall, phi_primary, S, H, J)` — `main.py`'s own ⟨|∇ξ|²⟩ of the rung-22
/// terminal field (rung 24's g-free kill-test witness), with its defaults `k_p=0.316, k_y=k_z=0.28,
/// ny=nz=32`. Every operation is in the Python's order: `(y - c) ** 2` and `sig ** 2` as
/// multiplies, `J ** 0.25` a real `pow`, each `sum` a left fold, `max(0.0, ·)`/`min(1.0, ·)` with
/// Python's argument order, and `(j - 1) % nz` Python's non-negative modulo.
fn mean_grad_sq(far_overall: f64, phi_primary: f64, s: f64, h: f64, j: f64) -> f64 {
    let (k_p, k_y, k_z) = (0.316, 0.28, 0.28);
    const NY: usize = 32;
    const NZ: usize = 32;
    let xibar = far_overall / (1.0 + far_overall);
    let far_p = phi_primary * f_stoich();
    let xi_p = far_p / (1.0 + far_p);
    let delta = k_p * (s * h).sqrt() * powp(j, 0.25);
    let (sig_y, sig_z) = (k_y * h, k_z * s);
    let ys: Vec<f64> = (0..NY).map(|i| (i as f64 + 0.5) * h / NY as f64).collect();
    let zs: Vec<f64> = (0..NZ).map(|k| (k as f64 + 0.5) * s / NZ as f64).collect();
    let gauss = |d: f64, sig: f64| (-(d * d) / (2.0 * (sig * sig))).exp();
    let ay: Vec<f64> = ys.iter()
        .map(|&y| [-delta, delta, 2.0 * h - delta, 2.0 * h + delta].iter().fold(0.0, |a, &c| a + gauss(y - c, sig_y)))
        .collect();
    let az: Vec<f64> = zs.iter()
        .map(|&z| [-1.0, 0.0, 1.0].iter().fold(0.0, |a, &m| a + gauss(z - s / 2.0 - m * s, sig_z)))
        .collect();
    let may = ay.iter().fold(0.0, |a, &v| a + v) / NY as f64;
    let maz = az.iter().fold(0.0, |a, &v| a + v) / NZ as f64;
    let ayh: Vec<f64> = ay.iter().map(|&a| a / may).collect();
    let azh: Vec<f64> = az.iter().map(|&a| a / maz).collect();
    let beta_bar = (xi_p - xibar) / xi_p;
    let cell = |sc: f64, a: f64, b: f64| xi_p * (1.0 - py_min2(1.0, py_max2(0.0, sc * beta_bar * a * b)));
    let (mut lo, mut hi) = (0.0, 50.0);
    for _ in 0..60 {
        let sc = 0.5 * (lo + hi);
        let mut acc = 0.0;
        for &a in &ayh {
            for &b in &azh {
                acc += cell(sc, a, b);
            }
        }
        let m_ = acc / (NY * NZ) as f64;
        if m_ > xibar { lo = sc } else { hi = sc }
    }
    let s_star = 0.5 * (lo + hi);
    let xi: Vec<Vec<f64>> = ayh.iter().map(|&a| azh.iter().map(|&b| cell(s_star, a, b)).collect()).collect();
    let (dy, dz) = (h / NY as f64, s / NZ as f64);
    let mut tot = 0.0;
    for i in 0..NY {
        let (im, ip) = (i.saturating_sub(1), (i + 1).min(NY - 1));
        for k in 0..NZ {
            let (km, kp) = ((k + NZ - 1) % NZ, (k + 1) % NZ);
            let gy = (xi[ip][k] - xi[im][k]) / ((ip - im) as f64 * dy);
            let gz = (xi[i][kp] - xi[i][km]) / (2.0 * dz);
            tot += gy * gy + gz * gz;
        }
    }
    tot / (NY * NZ) as f64
}

/// `print_local_mixing_table(flight)` — rung 24: the locally-resolved mixing time.
pub fn local_mixing_table(p: &mut Printer, d: &Design) {
    let (e, real) = real_eq(d);
    let eq = &e.gas;
    let (tt3, tt4, far, pp) = design_quad(&real);
    let (phi_p, tau) = (1.5, 3e-3);
    let (ng, nb, nq) = (24usize, 40usize, 160usize);
    let cfg = SpatialLocalPdf { s: 0.0625, ny: 32, nz: 32, n_bell: nb, n_quad: nq, ..SpatialLocalPdf::default() };

    p.print("\nLocally-resolved mixing time (rung 24): the ceiling rungs 11–23 all deferred BY NAME. Every one of");
    p.print("them ran ONE GLOBAL τ_mix; rung 23's §9 asked whether giving each cell its OWN rate would restore an");
    p.print("off-optimum dwell GROWTH and pin the emissions optimum non-circularly. Rung 24 builds it and ASKS.");
    p.print("ω = D_t·|∇ξ|²/var with D_t = σ²/(2τ_mix) — REUSED, so NO new constant, NO C_opt, NO τ_res, NO b_u.");

    p.print("\n  (1) THE FACTORIZATION — τ_mix CANCELS out of u=ω·τ_mix=σ²|∇ξ|²/(2var), so the shape is a PURE");
    p.print("      FIELD FUNCTIONAL and ⟨τ⟩(J) = τ_mix(J)·F(C) EXACTLY. Scale × shape, cleanly separated:");
    p.print(pyf!("  {:>5} {:>6} {:>10} {:>10} {:>12} {:>13}", "J", "C", "g (==r22)", "τ_mix(ms)", "F_cell", "⟨τ⟩_cell(ms)"));
    p.print(format!("  {}", dashes(60)));
    for j in [1i64, 4, 9, 16, 36, 64, 144, 400] {
        let m = jet_full(j as f64);
        let (g_s, _tau_of, f) = spatial_local_field(far, phi_p, cfg.s, m.h, m.j, m.tau_q(),
                                                   cfg.k_p, cfg.k_y, cfg.k_z, 32, 32);
        let c = (cfg.s / m.h) * powp(j as f64, 0.5);
        let note = if j == 16 { "  ← C_opt (F MINIMAL)" } else { "" };
        p.print(pyf!("  {:5d} {:6.2f} {:10.5f} {:10.3f} {:12.4f} {:13.4f}{}", j, c, g_s, m.tau_q() * 1e3, f, m.tau_q() * f * 1e3, note));
    }
    p.print("  ⇒ F is U-SHAPED with its min AT C_opt — the off-optimum dwell GROWTH rung 16 IMPOSED as");
    p.print("    τ_res·(1+b_u|ln(C/C_opt)|), here DERIVED from the plume's own gradients. But ⟨τ⟩=τ_mix·F still");
    p.print("    FALLS monotonically: F's ~1.4× U loses to τ_mix's 20× 1/√J swing. THE SCALE SWAMPS THE SHAPE.");

    p.print("\n  (2) THE KILL TEST (why the U is NOT circular) — ω carries an explicit 1/g (var=g·ξ̄(1−ξ̄)) and");
    p.print("      rung 22 ALREADY mins g at C_opt, so 'argmin F == argmin g' is a TELL, not a confirmation.");
    p.print("      ⟨|∇ξ|²⟩ carries NO g algebraically — and IT is maximal at C_opt. The gradients place it:");
    p.print(pyf!("  {:>5} {:>6} {:>9} {:>17}", "J", "C", "g", "⟨|∇ξ|²⟩ (g-free)"));
    p.print(format!("  {}", dashes(42)));
    for j in [4i64, 9, 16, 36, 64] {
        let m = jet_full(j as f64);
        let (g_s, _, _) = spatial_local_field(far, phi_p, cfg.s, m.h, m.j, m.tau_q(), 0.316, 0.28, 0.28, 32, 32);
        let gsq = mean_grad_sq(far, phi_p, cfg.s, m.h, m.j);
        let note = if j == 16 { "  ← C_opt (STEEPEST)" } else { "" };
        p.print(pyf!("  {:5d} {:6.2f} {:9.5f} {:17.4f}{}", j, (cfg.s / m.h) * powp(j as f64, 0.5), g_s, gsq, note));
    }
    p.print("  ⇒ At C_opt the residual structure sits at the plume's OWN scale σ (fine ⇒ steep ⇒ fast ⇒ SHORT");
    p.print("    dwell); off-optimum the air piles into WALL-SCALE slabs (coarse ⇒ shallow ⇒ slow ⇒ LONG dwell).");
    p.print("    HONEST: that fine-vs-coarse behaviour is a property of the FIXED-σ plume CARTOON, not a general law.");

    p.print("\n  (3) THE NEGATIVE HEADLINE — on the REAL per-pocket chemistry (NOT inferred from ⟨τ⟩): ⟨EI⟩ stays");
    p.print("      MONOTONE, so the emissions C_opt pin is STILL NOT recovered:");
    p.print(pyf!("  {:>5} {:>6} {:>7} {:>12} {:>9} {:>10} {:>6} {:>6}",
                 "J", "C", "F_cell", "⟨τ⟩_PDF(ms)", "EI_local", "EI_meanfld", "corr", "max_a"));
    p.print(format!("  {}", dashes(68)));
    for j in [4i64, 9, 16, 36, 64] {
        let st = eq.zoned_nox(far, tt3, tt4, pp, phi_p, ZonedNoxOpts {
            tau, mixing: Some(jet_full(j as f64)), spatial_local: Some(cfg), quench_ngrid: ng, ..ZonedNoxOpts::default()
        });
        let c = (cfg.s / 0.10) * powp(j as f64, 0.5);
        let note = if j < 16 { "under-pen" } else if j == 16 { "C_opt" } else { "over-pen" };
        p.print(pyf!("  {:5d} {:6.2f} {:7.4f} {:12.4f} {:9.4f} {:10.4f} {:6.3f} {:6.3f}  {}",
                     j, c, st.f_shape.unwrap(), st.tau_mean_local.unwrap() * 1e3, st.ei_no_spatial_local.unwrap(),
                     st.ei_no_spatial_local_meanfield.unwrap(), st.corr_ratio_local.unwrap(), st.max_a_quench.unwrap(), note));
    }
    p.print("  ⇒ ⟨EI⟩ falls MONOTONICALLY through C_opt — no emissions optimum, even with the rate localized.");
    p.print("    (corr>1 throughout ALSO re-derives rung-23's ξ–τ correlation from INDEPENDENT physics: gradient");
    p.print("     structure, not arrival time. Rich pockets dwell longest either way.)");

    p.print("\n  THE ADJUDICATION: rung 23 left an explicit fork — does the off-optimum dwell GROW (rung-16,");
    p.print("  imposed) or FALL (rung-23, derived)? 'Neither is pinned from data.' It resolves BOTH WAYS, in");
    p.print("  DIFFERENT FACTORS: the SHAPE grows (rung 16 vindicated, and now DERIVED), the PRODUCT still falls");
    p.print("  (rung 23 vindicated). Rung-16's kink is NOT an artifact — it is real and MIS-SCALED.");
    p.print("  HONEST SCOPE: rung 24 localizes the RATE, not the SCALE — ⟨τ⟩'s magnitude still rides on rung-11's");
    p.print("  un-anchored τ_mix and F's on rung-22's k_p. So rung-23 §9's hope that this lets rung 17 claim a");
    p.print("  firing MAGNITUDE is NOT delivered (it buys a sharper DIRECTION only) — CORRECTED, not inherited.");
}
