//! PHASE 3C GATE — every rung-13/15/16/18/21 mixing-PDF value the Python oracle dumped,
//! recomputed in Rust.
//!
//! The fifth in the family (`gas_oracle.rs` → `cycle_oracle.rs` → `nox_oracle.rs` →
//! `quench_oracle.rs` → here), and a separate file for the same reason the dump is: each gate's
//! cost stays proportional to what it certifies, and the earlier slices' TSVs stay frozen.
//!
//! WHAT IS NEW, and therefore what this gate is built to catch:
//!
//! * `beta_pdf_nodes_weights` is REGIME-SWITCHING at `a = 1`, and the two regimes are different
//!   integration schemes — `powp` with a computed exponent on one side, a `sqrt`-sized window on
//!   the other. The `quad/` keys dump nodes, weights, the achieved mean and the achieved variance
//!   on BOTH sides of the switch, so a port that gets one branch right and the other wrong is
//!   named at the node rather than three sections later as "⟨EI⟩ differs".
//! * The bell's LEAN END IS A BRANCH. Python catches an AssertionError from `_primary_aft` to
//!   return 0; Rust splits the guard out as [`turbojet::nox::try_primary_aft`]. The
//!   `bell/*/first_burnable` keys measure that the two take the zero branch the same number of
//!   times, instead of assuming a `try/except` and an `Option` agree.
//! * `transport_variance` is `nsteps` REPEATED DIVISIONS whose closed form is an `exp`. The
//!   `ode/*/analytic` keys sit beside the integrated ones — ~1 % apart — so "simplifying" the
//!   loop fails here rather than making rung 18's basin quietly the wrong depth.
//! * The three grid formulas (`bell`, the pocket grid, the `a≥1` quadrature window) have three
//!   DIFFERENT shapes. `xi_max·(i+0.5)/n` is not `xi_max·((i+0.5)/n)`.
//!
//! The bars are not invented. The project ships on two interpreters, so whatever THEY disagree
//! by is a deviation it ALREADY tolerates. Measured on this dump: **70.55 %** of the 2448 values
//! are bit-identical between CPython and PyPy — higher than slice B's 58.3 % only because a
//! larger share of this slice is solver-free algebra, and the split is the same one every slice
//! has found. The worst disagreement anywhere is 4.9e-15.
//!
//! **All 48 shape/location keys are bit-identical across the two interpreters** — slice A's
//! finding reproduced a third time, and the reason `shape_location` gets a bar of exactly zero.
//!
//! Regenerate the oracle with:
//!     C:\Python314\python.exe rust/oracle/dump_pdf.py rust/oracle/pdf_cpython.tsv
//!     .venv\Scripts\python.exe  rust/oracle/dump_pdf.py rust/oracle/pdf_pypy.tsv

use std::collections::HashMap;
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{self, equilibrium_composition, Gas};
use turbojet::nox::{
    beta_pdf_nodes_weights, bell_interpolator, ideal_bell_ei, pdf_mean_ei, pdf_mean_ei_on_bell,
    pocket_quench_grid, pocket_quench_integrate, primary_aft, quench_no, super_eq_o_multiplier,
    thermal_no, transport_variance, two_stream_ceiling, xi_soot_bound, Bell, JetMixing,
    MixingPdf, PocketOpts, PocketQuenchPdf, QuenchOpts, QuenchPdf, TransportedPdf, ZonedNoxOpts,
};

const ORACLE_CPYTHON: &str = include_str!("../oracle/pdf_cpython.tsv");
const ORACLE_PYPY: &str = include_str!("../oracle/pdf_pypy.tsv");

/// Python's `repr(float)` for every value this dump keys on — the same rule `quench_oracle.rs`
/// derived and pinned. CPython formats shortest-round-trip digits plus a decimal-point position
/// `decpt`, switching to exponential when `decpt <= -4 || decpt > 16`, with a signed at-least-
/// two-digit exponent. Rust's `Display` never uses exponential and its `LowerExp` never pads.
fn py_repr(v: f64) -> String {
    if v == 0.0 {
        return if v.is_sign_negative() { "-0.0".into() } else { "0.0".into() };
    }
    let sci = format!("{v:e}");
    let (mantissa, exp) = sci.split_once('e').expect("LowerExp always emits an 'e'");
    let exp: i32 = exp.parse().expect("integer exponent");
    let decpt = exp + 1;
    if decpt <= -4 || decpt > 16 {
        let sign = if exp < 0 { '-' } else { '+' };
        format!("{mantissa}e{sign}{:02}", exp.abs())
    } else {
        let s = format!("{v}");
        if s.contains('.') {
            s
        } else {
            format!("{s}.0")
        }
    }
}

fn load_oracle(text: &str) -> HashMap<&str, f64> {
    let mut out = HashMap::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let mut it = line.split('\t');
        let key = it.next().expect("key");
        let bits: u64 = it.next().expect("bits").parse().expect("u64 bits");
        out.insert(key, f64::from_bits(bits));
    }
    out
}

/// The text of a caught panic.
///
/// `catch_unwind` catches ANY panic, so "it panicked" is a weaker statement than it reads —
/// an index error would satisfy it just as well as the guard under test. The Python's versions
/// of these gates check the message (`"RQL geometry" in str(e)`), so naming the panic here is
/// restoring fidelity rather than adding defensiveness.
fn panic_text(e: Box<dyn std::any::Any + Send>) -> String {
    e.downcast_ref::<String>()
        .cloned()
        .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_string()))
        .unwrap_or_default()
}

/// Python's `min(range(n), key=…)` / `max(…)` keep the FIRST extremum on ties, and so do these.
fn argmin(v: &[f64]) -> usize {
    let mut best = 0;
    for (i, &x) in v.iter().enumerate() {
        if x < v[best] {
            best = i;
        }
    }
    best
}

fn argmax(v: &[f64]) -> usize {
    let mut best = 0;
    for (i, &x) in v.iter().enumerate() {
        if x > v[best] {
            best = i;
        }
    }
    best
}

// --- the grids, transcribed from `dump_pdf.py` --------------------------------------------
const TAU: f64 = 3e-3;
const NB: usize = 32;
const NQ: usize = 160;
const NB16: usize = 24;
const NQ16: usize = 64;
const NG16: usize = 9;
const NS16: usize = 100;
const NGRID: usize = 17;
const NSTEPS: usize = 200;
const PHI_P: f64 = 1.5;
const CE: f64 = 0.20;

const C_FACTORS: &[f64] = &[0.01, 0.25, 0.5, 0.8, 1.0, 1.25, 2.0, 4.0, 100.0];
const J_GRID: &[f64] = &[1.0, 4.0, 9.0, 16.0, 25.0, 64.0, 100.0, 225.0, 625.0];
const S_GRID: &[f64] = &[0.0800, 0.0625, 0.0500, 0.0400];
const G_GRID: &[f64] = &[0.0005, 0.004, 0.01, 0.02, 0.0257, 0.026, 0.05, 0.12, 0.24, 0.40];
const HUMP_G: &[f64] = &[0.005, 0.01, 0.02, 0.05, 0.12, 0.30];
const CEIL_FAR: &[f64] = &[0.0150, 0.0271791907192821, 0.0350, 0.0450];
const TAU_CORES: &[f64] = &[2.5e-3, 4.0e-3, 6.0e-3, 1.0e-2];
const MF_J: &[f64] = &[4.0, 9.0, 16.0, 25.0, 49.0, 100.0, 225.0, 625.0];
/// The literal the dump uses for its lean reference mean — deliberately the SAME 16-digit
/// literal, not the design point's own `far`, so the quadrature sweep is reproducible on its own.
const FAR_LEAN_LITERAL: f64 = 0.0271791907192821;

fn mix(j: f64) -> JetMixing {
    JetMixing { j, c_e: CE, shape_n: 2.0, ..JetMixing::default() }
}

/// A named mean-field mixing frequency ω(J) — rung 18's three negative-result shapes.
type MeanFieldOmega = (&'static str, fn(f64) -> f64);

/// `J` where `C = (S/H)√J = C_opt`. Python writes `(C_opt·H/S) ** 2` with an INTEGER literal
/// exponent, which PyPy rewrites into a multiply — hence `x * x`, not `powp(x, 2.0)`.
fn j_opt(s: f64, c_opt: f64) -> f64 {
    let x = c_opt * JetMixing::default().h / s;
    x * x
}

/// The two design points, derived from REAL equilibrium-engine runs exactly as the oracle
/// derives them. Returns `(name, Tt3, Tt4, far, pt4)`.
fn design_points() -> Vec<(&'static str, f64, f64, f64, f64)> {
    let sub = FlightCondition::new(250.0, 50_000.0, 0.85);
    let sup = FlightCondition::new(216.7, 18_750.0, 2.0);
    let losses = Losses {
        pi_d: 0.97,
        eta_c: 0.88,
        eta_b: 0.99,
        pi_b: 0.96,
        eta_t: 0.90,
        eta_m: 0.99,
        pi_n: 0.98,
        ..Losses::default()
    };
    [("dp1", &sub, 10.0, 1500.0, 50.0), ("dp4", &sup, 12.0, 1800.0, 50.0)]
        .iter()
        .map(|&(name, flight, pi_c, tt4, mdot)| {
            let r = build_turbojet(Gas::reacting_equilibrium(), pi_c, tt4, flight.p0, losses)
                .run(flight, mdot);
            (name, r.station("3").tt, r.station("4").tt, r.station("4").far, r.station("4").pt)
        })
        .collect()
}

/// The oracle's `dump_quad` — the shape parameters, a fixed set of nodes and weights, and the
/// achieved mean/variance against the target. The index set is Python's `sorted({…})`.
fn dump_quad(
    out: &mut Vec<(String, f64)>,
    tag: &str,
    xibar: f64,
    g_seg: f64,
    nq: usize,
) {
    let (nodes, w) = beta_pdf_nodes_weights(xibar, g_seg, nq);
    let inv = 1.0 / g_seg - 1.0;
    out.push((format!("quad/{tag}/a"), xibar * inv));
    out.push((format!("quad/{tag}/b"), (1.0 - xibar) * inv));
    let mut idx = vec![0usize, 1, nq / 4, nq / 2, (3 * nq) / 4, nq - 2, nq - 1];
    idx.sort_unstable();
    idx.dedup();
    for i in idx {
        out.push((format!("quad/{tag}/n{i}"), nodes[i]));
        out.push((format!("quad/{tag}/w{i}"), w[i]));
    }
    let mean = w.iter().zip(nodes.iter()).fold(0.0, |acc, (&wi, &x)| acc + wi * x);
    let var = w.iter().zip(nodes.iter()).fold(0.0, |acc, (&wi, &x)| {
        let d = x - xibar;
        acc + wi * (d * d)
    });
    out.push((format!("quad/{tag}/mean"), mean));
    out.push((format!("quad/{tag}/var"), var));
    out.push((format!("quad/{tag}/vartgt"), g_seg * xibar * (1.0 - xibar)));
}

/// Recompute every key the oracle dumped. The section ORDER mirrors `dump_pdf.py` so the two
/// can be read side by side.
fn rust_values() -> Vec<(String, f64)> {
    let mut out: Vec<(String, f64)> = Vec::new();

    // === SECTION 1 — the SOLVER-FREE algebra ==============================================
    for &s in S_GRID {
        let pdf = MixingPdf { s, ..MixingPdf::default() };
        let qp = QuenchPdf { s, ..QuenchPdf::default() };
        let pq = PocketQuenchPdf { s, ..PocketQuenchPdf::default() };
        let tr = TransportedPdf { s, ..TransportedPdf::default() };
        for &j in J_GRID {
            let m = mix(j);
            let tag = format!("cfg/{}/{}", py_repr(s), py_repr(j));
            out.push((format!("{tag}/C_pdf"), pdf.c(&m)));
            out.push((format!("{tag}/C_qp"), qp.c(&m)));
            out.push((format!("{tag}/C_pq"), pq.c(&m)));
            out.push((format!("{tag}/C_tr"), tr.c(&m)));
        }
        for &f in C_FACTORS {
            let c = 2.5 * f;
            let tag = format!("cfg/{}/C{}", py_repr(s), py_repr(f));
            out.push((format!("{tag}/g_pdf"), pdf.segregation(c)));
            out.push((format!("{tag}/u_qp"), qp.u(c)));
            out.push((format!("{tag}/g_qp"), qp.segregation(c)));
            out.push((format!("{tag}/D_qp"), qp.dwell_factor(c, TAU)));
            out.push((format!("{tag}/g_pq"), pq.segregation(c)));
            out.push((format!("{tag}/tcore_pq"), pq.core_dwell(c)));
            out.push((format!("{tag}/omega_tr"), tr.coverage_omega(c)));
        }
    }
    for &(k_g, g_max) in &[(0.0f64, 0.3f64), (0.3, 0.05), (0.9, 0.3)] {
        let pdf = MixingPdf { k_g, g_max, ..MixingPdf::default() };
        for &f in &[0.25f64, 1.0, 4.0] {
            out.push((
                format!("cfg/kg{}gm{}/{}/g", py_repr(k_g), py_repr(g_max), py_repr(f)),
                pdf.segregation(2.5 * f),
            ));
        }
    }
    for &(tau_res, b_u) in &[(1.0e-3f64, 3.0f64), (2.5e-3, 0.0), (5.0e-3, 6.0)] {
        let qp = QuenchPdf { tau_res, b_u, ..QuenchPdf::default() };
        let pq = PocketQuenchPdf { tau_res, b_u, ..PocketQuenchPdf::default() };
        for &f in &[0.25f64, 1.0, 4.0] {
            let tag = format!("cfg/tr{}bu{}/{}", py_repr(tau_res), py_repr(b_u), py_repr(f));
            out.push((format!("{tag}/D"), qp.dwell_factor(2.5 * f, TAU)));
            out.push((format!("{tag}/tcore"), pq.core_dwell(2.5 * f)));
        }
    }
    for &(da_opt, w_cov, c_phi, tau_mix) in &[
        (2.0f64, 1.0f64, 2.0f64, 2.5e-3f64),
        (0.5, 0.5, 2.0, 2.5e-3),
        (6.0, 2.0, 3.0, 1.0e-3),
    ] {
        let tr = TransportedPdf { da_opt, w_cov, c_phi, tau_mix, ..TransportedPdf::default() };
        for &f in &[0.25f64, 1.0, 4.0] {
            out.push((
                format!(
                    "cfg/da{}w{}c{}t{}/{}/omega",
                    py_repr(da_opt),
                    py_repr(w_cov),
                    py_repr(c_phi),
                    py_repr(tau_mix),
                    py_repr(f)
                ),
                tr.coverage_omega(2.5 * f),
            ));
        }
    }

    for &far in CEIL_FAR {
        for &phi_p in &[0.8f64, 1.0, 1.2, 1.5, 1.8, 2.0] {
            if phi_p * gas::f_stoich() <= far {
                continue; // the RQL guard: the primary must be RICHER than the mean
            }
            out.push((
                format!("ceil/{}/{}", py_repr(far), py_repr(phi_p)),
                two_stream_ceiling(far, phi_p),
            ));
        }
    }

    for &gc in &[0.0675f64, 0.0200, 0.3000] {
        for &om in &[0.0f64, 50.0, 250.0, 1000.0] {
            for &tau in &[1.0e-3f64, 2.5e-3] {
                for &nst in &[50usize, 200, 400] {
                    out.push((
                        format!("ode/{}/{}/{}/{nst}", py_repr(gc), py_repr(om), py_repr(tau)),
                        transport_variance(gc, om, tau, 2.0, nst),
                    ));
                }
                out.push((
                    format!("ode/{}/{}/{}/analytic", py_repr(gc), py_repr(om), py_repr(tau)),
                    gc * (-2.0 * om * tau).exp(),
                ));
            }
        }
    }
    for &c_phi in &[1.0f64, 2.0, 3.5] {
        out.push((
            format!("ode/cphi{}", py_repr(c_phi)),
            transport_variance(0.0675, 250.0, 2.5e-3, c_phi, 400),
        ));
    }

    let xibar_lean = FAR_LEAN_LITERAL / (1.0 + FAR_LEAN_LITERAL);
    let xibar_st = gas::f_stoich() / (1.0 + gas::f_stoich());
    for &gs in G_GRID {
        dump_quad(&mut out, &format!("lean/{}", py_repr(gs)), xibar_lean, gs, NQ);
        dump_quad(&mut out, &format!("stoich/{}", py_repr(gs)), xibar_st, gs, NQ);
    }
    for &nq in &[40usize, 64, 160, 200] {
        dump_quad(&mut out, &format!("nq{nq}/sing"), xibar_lean, 0.12, nq);
        dump_quad(&mut out, &format!("nq{nq}/delta"), xibar_lean, 0.004, nq);
    }
    for &nq in &[112usize, 128, 160, 200] {
        for &gs in &[0.026f64, 0.40] {
            dump_quad(&mut out, &format!("floor/{}/nq{nq}", py_repr(gs)), xibar_lean, gs, nq);
        }
    }
    dump_quad(&mut out, "clamp/hi", 0.97, 0.0005, NQ);

    // === SECTION 2 — the design points =====================================================
    let dps = design_points();
    for &(name, tt3, tt4, far, pt4) in &dps {
        out.push((format!("dp/{name}/Tt3"), tt3));
        out.push((format!("dp/{name}/Tt4"), tt4));
        out.push((format!("dp/{name}/far"), far));
        out.push((format!("dp/{name}/pt4"), pt4));
        let xb = far / (1.0 + far);
        out.push((format!("dp/{name}/xibar"), xb));
        out.push((format!("dp/{name}/g_bmax"), (1.0 - xb) / (2.0 - xb)));
    }
    let find = |n: &str| *dps.iter().find(|d| d.0 == n).expect("design point");
    let hf = gas::hf_fuel_default();

    // === SECTION 3 — the IDEAL BELL ========================================================
    let xi_max = xi_soot_bound();
    out.push(("bell/xi_max".to_string(), xi_max));

    // Four bells carry sections 3-7: the bell depends on NEITHER g NOR J.
    let bells: Vec<((&str, bool), Bell)> = [("dp1", false), ("dp1", true), ("dp4", false),
                                            ("dp4", true)]
        .iter()
        .map(|&(dp, su)| {
            let (_, tt3, _, _, p) = find(dp);
            ((dp, su), bell_interpolator(p, tt3, hf, TAU, NB, su))
        })
        .collect();
    let bell_of = |dp: &str, su: bool| {
        &bells.iter().find(|((d, s), _)| *d == dp && *s == su).expect("bell").1
    };

    for dp in ["dp1", "dp4"] {
        let (_, tt3, _, far, p) = find(dp);
        for su in [false, true] {
            let tag = format!("{dp}/{}", if su { "su" } else { "eq" });
            let b = bell_of(dp, su);
            let xi_ref: Vec<f64> =
                (0..NB).map(|i| xi_max * (i as f64 + 0.5) / NB as f64).collect();
            let vals: Vec<f64> = xi_ref.iter().map(|&x| b.at(x)).collect();
            for (i, (&x, &v)) in xi_ref.iter().zip(vals.iter()).enumerate() {
                out.push((format!("bell/{tag}/node{i}/xi"), x));
                out.push((format!("bell/{tag}/node{i}/ei"), v));
            }
            let first_burn =
                vals.iter().position(|&v| v > 0.0).unwrap_or(vals.len());
            out.push((format!("bell/{tag}/first_burnable"), first_burn as f64));
            out.push((format!("bell/{tag}/argmax"), argmax(&vals) as f64));
            out.push((format!("bell/{tag}/lo_edge"), b.at(0.0)));
            out.push((format!("bell/{tag}/lo_edge2"), b.at(xi_ref[0] * 0.5)));
            out.push((format!("bell/{tag}/hi_edge"), b.at(xi_max)));
            out.push((format!("bell/{tag}/hi_edge2"), b.at(0.5)));
            for &t in &[0.25f64, 0.5, 0.75] {
                for i in [3usize, NB / 2, NB - 2] {
                    let x = xi_ref[i] + t * (xi_ref[i + 1] - xi_ref[i]);
                    out.push((format!("bell/{tag}/lerp{i}_{}", py_repr(t)), b.at(x)));
                }
            }
            for &fl in
                &[0.0f64, 0.002, 0.010, far, 0.0500, gas::f_stoich(), 0.0900, 0.1359, 0.1400,
                  0.2000]
            {
                out.push((
                    format!("bellpt/{tag}/{}", py_repr(fl)),
                    ideal_bell_ei(fl, p, tt3, hf, TAU, su),
                ));
            }
            for &t_res in &[1.0e-3f64, 5.0e-3] {
                out.push((
                    format!("bellpt/{tag}/tau{}", py_repr(t_res)),
                    ideal_bell_ei(gas::f_stoich(), p, tt3, hf, t_res, su),
                ));
            }
        }
    }

    // === SECTION 4 — ⟨EI⟩ over the β-PDF ===================================================
    for dp in ["dp1", "dp4"] {
        let (_, _, _, far, _) = find(dp);
        let xibar = far / (1.0 + far);
        for su in [false, true] {
            let tag = format!("{dp}/{}", if su { "su" } else { "eq" });
            let b = bell_of(dp, su);
            let mut gs_all: Vec<f64> = G_GRID.to_vec();
            gs_all.push(0.0);
            for &gs in &gs_all {
                out.push((
                    format!("pdfei/{tag}/lean/{}", py_repr(gs)),
                    pdf_mean_ei_on_bell(b, xibar, gs, NQ),
                ));
                out.push((
                    format!("pdfei/{tag}/stoich/{}", py_repr(gs)),
                    pdf_mean_ei_on_bell(b, xibar_st, gs, NQ),
                ));
            }
            for &gs in &[0.0f64, 1e-12, 1e-9, 1.0000001e-9, 1e-8] {
                out.push((
                    format!("pdfei/{tag}/delta/{}", py_repr(gs)),
                    pdf_mean_ei_on_bell(b, xibar, gs, NQ),
                ));
            }
            for &nq in &[40usize, 160] {
                out.push((
                    format!("pdfei/{tag}/nq{nq}"),
                    pdf_mean_ei_on_bell(b, xibar, 0.12, nq),
                ));
            }
        }
    }
    for dp in ["dp1", "dp4"] {
        let (_, _, _, far, _) = find(dp);
        let xibar = far / (1.0 + far);
        for su in [false, true] {
            let tag = format!("{dp}/{}", if su { "su" } else { "eq" });
            let b = bell_of(dp, su);
            let hv: Vec<f64> =
                HUMP_G.iter().map(|&gs| pdf_mean_ei_on_bell(b, xibar, gs, NQ)).collect();
            for (&gs, &v) in HUMP_G.iter().zip(hv.iter()) {
                out.push((format!("hump/{tag}/{}", py_repr(gs)), v));
            }
            let im = argmax(&hv);
            out.push((format!("hump/{tag}/argmax"), im as f64));
            out.push((format!("hump/{tag}/margin_lo"), hv[im] / hv[im - 1]));
            out.push((format!("hump/{tag}/margin_hi"), hv[im] / hv[im + 1]));
        }
    }
    let (_, tt3_1, _tt4_1, far_1, p_1) = find("dp1");
    let xib_1 = far_1 / (1.0 + far_1);
    for su in [false, true] {
        for &gs in &[0.0f64, 0.02, 0.12, 0.30] {
            out.push((
                format!("pdfmean/dp1/{}/{}", if su { "su" } else { "eq" }, py_repr(gs)),
                pdf_mean_ei(far_1, tt3_1, p_1, hf, TAU, gs, NB, NQ, su),
            ));
        }
    }

    // === SECTION 5 — rung 13's optimum LOCATION, rung 21's shape key =======================
    for dp in ["dp1", "dp4"] {
        let (_, _, _, far, _) = find(dp);
        let xibar = far / (1.0 + far);
        for &s in S_GRID {
            let pdf = MixingPdf { s, ..MixingPdf::default() };
            let jo = j_opt(s, pdf.c_opt);
            let js = [jo / 4.0, jo / 2.0, jo, 2.0 * jo, 4.0 * jo];
            out.push((format!("jsweep/{dp}/{}/J_opt", py_repr(s)), jo));
            let mut loc = [0usize; 2];
            for (ai, su) in [false, true].into_iter().enumerate() {
                let arm = if su { "su" } else { "eq" };
                let b = bell_of(dp, su);
                let mut eis = Vec::with_capacity(js.len());
                for &j in &js {
                    let gs = pdf.segregation(pdf.c(&mix(j)));
                    let v = pdf_mean_ei_on_bell(b, xibar, gs, NQ);
                    eis.push(v);
                    let tag = format!("jsweep/{dp}/{}/{arm}/{}", py_repr(s), py_repr(j));
                    out.push((format!("{tag}/g"), gs));
                    out.push((format!("{tag}/ei"), v));
                }
                loc[ai] = argmin(&eis);
                let im = loc[ai];
                out.push((format!("jsweep/{dp}/{}/{arm}/argmin", py_repr(s)), im as f64));
                out.push((
                    format!("jsweep/{dp}/{}/{arm}/lift_lo", py_repr(s)),
                    eis[im - 1] / eis[im].max(1e-300),
                ));
                out.push((
                    format!("jsweep/{dp}/{}/{arm}/lift_hi", py_repr(s)),
                    eis[im + 1] / eis[im].max(1e-300),
                ));
            }
            out.push((
                format!("jsweep/{dp}/{}/loc_agree", py_repr(s)),
                f64::from(u8::from(loc[0] == loc[1])),
            ));
        }
    }
    for &gs in &[0.0f64, 0.02, 0.12, 0.30] {
        out.push((
            format!("r21/lift/dp1/{}", py_repr(gs)),
            pdf_mean_ei_on_bell(bell_of("dp1", true), xib_1, gs, NQ)
                / pdf_mean_ei_on_bell(bell_of("dp1", false), xib_1, gs, NQ),
        ));
    }
    out.push((
        "r21/lift/primary_m".to_string(),
        super_eq_o_multiplier(primary_aft(PHI_P * gas::f_stoich(), p_1, tt3_1, hf)),
    ));
    out.push((
        "r21/lift/peak_m".to_string(),
        super_eq_o_multiplier(primary_aft(gas::f_stoich(), p_1, tt3_1, hf)),
    ));

    // === SECTION 6 — rung 18 ===============================================================
    let tr = TransportedPdf { s: 0.0625, n_bell: NB, n_quad: NQ, n_ode: 200,
                              ..TransportedPdf::default() };
    for dp in ["dp1", "dp4"] {
        let (_, _, _, far, _) = find(dp);
        let xibar = far / (1.0 + far);
        let b = bell_of(dp, false);
        let gc = two_stream_ceiling(far, PHI_P);
        out.push((format!("tr/{dp}/g_ceiling"), gc));
        let mut gs_list = Vec::with_capacity(J_GRID.len());
        for &j in J_GRID {
            let c = tr.c(&mix(j));
            let (g_seg, g_ceil) = tr.segregation(c, far, PHI_P);
            gs_list.push(g_seg);
            let tag = format!("tr/{dp}/{}", py_repr(j));
            out.push((format!("{tag}/C"), c));
            out.push((format!("{tag}/omega"), tr.coverage_omega(c)));
            out.push((format!("{tag}/g"), g_seg));
            out.push((format!("{tag}/gceil"), g_ceil));
            out.push((format!("{tag}/ei"), pdf_mean_ei_on_bell(b, xibar, g_seg, NQ)));
        }
        out.push((format!("tr/{dp}/argmin_g"), argmin(&gs_list) as f64));
        let eis: Vec<f64> =
            gs_list.iter().map(|&g| pdf_mean_ei_on_bell(b, xibar, g, NQ)).collect();
        out.push((format!("tr/{dp}/argmin_ei"), argmin(&eis) as f64));
        let (g_opt, _) = tr.segregation(tr.c(&mix(16.0)), far, PHI_P);
        out.push((format!("tr/{dp}/floor_g"), g_opt));
        out.push((format!("tr/{dp}/floor_ratio"), g_opt / (gc * (-tr.da_opt).exp())));
        out.push((
            format!("tr/{dp}/elevation"),
            pdf_mean_ei_on_bell(b, xibar, g_opt, NQ)
                / pdf_mean_ei_on_bell(b, xibar, 0.0, NQ).max(1e-300),
        ));
    }
    let mf_forms: [MeanFieldOmega; 3] = [
        ("const", |_j| 250.0),
        ("sqrtJ", |j| 250.0 * (j / 16.0).sqrt()),
        ("linJ", |j| 250.0 * (j / 16.0)),
    ];
    for (name, om) in mf_forms {
        let vals: Vec<f64> =
            MF_J.iter().map(|&j| transport_variance(0.0675, om(j), 2.5e-3, 2.0, 400)).collect();
        for (&j, &v) in MF_J.iter().zip(vals.iter()) {
            out.push((format!("mf/{name}/{}", py_repr(j)), v));
        }
        out.push((format!("mf/{name}/argmin"), argmin(&vals) as f64));
        let hi = vals.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let lo = vals.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        out.push((format!("mf/{name}/spread"), (hi - lo) / hi));
    }
    let mut sp = Vec::with_capacity(MF_J.len());
    for &j in MF_J {
        let c = (tr.s / JetMixing::default().h) * j.sqrt();
        let v = transport_variance(0.0675, tr.coverage_omega(c), tr.tau_mix, tr.c_phi, 400);
        sp.push(v);
        out.push((format!("mf/spatial/{}", py_repr(j)), v));
    }
    out.push(("mf/spatial/argmin".to_string(), argmin(&sp) as f64));

    const EPS: f64 = 1e-5;
    let c0 = tr.c_opt;
    let g_tr = |c: f64| transport_variance(0.0675, tr.coverage_omega(c), tr.tau_mix, tr.c_phi, 400);
    out.push(("smooth/tr/slope_r".to_string(), (g_tr(c0 * (1.0 + EPS)) - g_tr(c0)) / (EPS * c0)));
    out.push(("smooth/tr/slope_l".to_string(), (g_tr(c0) - g_tr(c0 * (1.0 - EPS))) / (EPS * c0)));
    let kink = MixingPdf { s: tr.s, c_opt: c0, ..MixingPdf::default() };
    out.push((
        "smooth/kink/slope_r".to_string(),
        (kink.segregation(c0 * (1.0 + EPS)) - kink.segregation(c0)) / (EPS * c0),
    ));
    out.push((
        "smooth/kink/slope_l".to_string(),
        (kink.segregation(c0) - kink.segregation(c0 * (1.0 - EPS))) / (EPS * c0),
    ));

    // === SECTION 7 — rung 15's dwell factor × the bell integral ============================
    let qp = QuenchPdf { s: 0.0625, n_bell: NB, n_quad: NQ, ..QuenchPdf::default() };
    for dp in ["dp1", "dp4"] {
        let (_, _, _, far, _) = find(dp);
        let xibar = far / (1.0 + far);
        for su in [false, true] {
            let arm = if su { "su" } else { "eq" };
            let b = bell_of(dp, su);
            let mut t2 = Vec::with_capacity(J_GRID.len());
            for &j in J_GRID {
                let c = qp.c(&mix(j));
                let g_seg = qp.segregation(c);
                let d = qp.dwell_factor(c, TAU);
                let v = d * pdf_mean_ei_on_bell(b, xibar, g_seg, NQ);
                t2.push(v);
                let tag = format!("r15/{dp}/{arm}/{}", py_repr(j));
                out.push((format!("{tag}/D"), d));
                out.push((format!("{tag}/g"), g_seg));
                out.push((format!("{tag}/term2"), v));
            }
            out.push((format!("r15/{dp}/{arm}/argmin_t2"), argmin(&t2) as f64));
        }
    }

    // === SECTION 8 — rung 16's PER-POCKET integral =========================================
    // The lever the Python cannot use: the bank depends on `tau_core` and NOT on `g_seg`, so one
    // build serves every width. Python rebuilds 24 quenches per (τ_core, g); this builds one per
    // τ_core and integrates twice. Bit-identical, half the work.
    let popts = PocketOpts {
        n_bell: NB16,
        quench_ngrid: NG16,
        quench_nsteps: NS16,
        super_eq_o: false,
    };
    let mut banks: Vec<(f64, turbojet::nox::PocketGrid)> = Vec::new();
    for &tc in TAU_CORES {
        let grid = pocket_quench_grid(far_1, tt3_1, p_1, hf, TAU, tc, popts);
        for &gs in &[0.05f64, 0.12] {
            let tag = format!("r16/dp1/{}/{}", py_repr(tc), py_repr(gs));
            out.push((
                format!("{tag}/ei"),
                pocket_quench_integrate(&grid, far_1, gs, NQ16),
            ));
            out.push((format!("{tag}/max_a"), grid.max_a));
        }
        banks.push((tc, grid));
    }
    let bank_of = |tc: f64| &banks.iter().find(|(t, _)| *t == tc).expect("pocket bank").1;
    for &gs in &[0.0f64, 1e-12] {
        let grid = bank_of(2.5e-3);
        out.push((
            format!("r16/dp1/delta/{}/ei", py_repr(gs)),
            pocket_quench_integrate(grid, far_1, gs, NQ16),
        ));
        out.push((format!("r16/dp1/delta/{}/max_a", py_repr(gs)), grid.max_a));
    }
    let grid_su = pocket_quench_grid(
        far_1,
        tt3_1,
        p_1,
        hf,
        TAU,
        4.0e-3,
        PocketOpts { super_eq_o: true, ..popts },
    );
    out.push((
        "r16/dp1/su/ei".to_string(),
        pocket_quench_integrate(&grid_su, far_1, 0.12, NQ16),
    ));
    out.push(("r16/dp1/su/max_a".to_string(), grid_su.max_a));
    let lo = pocket_quench_integrate(bank_of(2.5e-3), far_1, 0.12, NQ16);
    let hi = pocket_quench_integrate(bank_of(6.0e-3), far_1, 0.12, NQ16);
    out.push(("r16/sublinear/ratio16".to_string(), hi / lo));
    out.push(("r16/sublinear/ratio_dwell".to_string(), 6.0e-3 / 2.5e-3));

    let xi_grid16: Vec<f64> =
        (0..NB16).map(|i| xi_max * (i as f64 + 0.5) / NB16 as f64).collect();
    for i in [8usize, 12, 16, 20] {
        let xi = xi_grid16[i];
        let far_local = xi / (1.0 - xi);
        out.push((format!("r16/pocket{i}/xi"), xi));
        out.push((format!("r16/pocket{i}/far_local"), far_local));
        let t_p = primary_aft(far_local, p_1, tt3_1, hf);
        let alpha = far_1 / far_local;
        let comp = equilibrium_composition(far_local, t_p, p_1);
        let ntot: f64 = comp.iter().map(|&(_, v)| v).sum();
        let n0 = alpha * thermal_no(&comp, t_p, p_1, TAU, far_local, 4000, 1.0).x_no * ntot;
        out.push((format!("r16/pocket{i}/T_p"), t_p));
        out.push((format!("r16/pocket{i}/alpha"), alpha));
        out.push((format!("r16/pocket{i}/n0"), n0));
        let q = quench_no(
            &comp,
            t_p,
            alpha,
            far_1,
            tt3_1,
            p_1,
            n0,
            4.0e-3,
            QuenchOpts { nsteps: NS16, ngrid: NG16, ..QuenchOpts::default() },
        );
        out.push((format!("r16/pocket{i}/ei"), q.ei));
        out.push((format!("r16/pocket{i}/T_peak"), q.t_peak));
        out.push((format!("r16/pocket{i}/max_a"), q.max_a));
    }

    // === SECTION 9 — the PUBLIC entry point ================================================
    let g = Gas::reacting_equilibrium();
    let base = ZonedNoxOpts {
        tau: TAU,
        quench_ngrid: NGRID,
        quench_nsteps: NSTEPS,
        ..ZonedNoxOpts::default()
    };
    let pdf_s = MixingPdf { s: 0.0625, n_bell: NB, n_quad: NQ, ..MixingPdf::default() };
    let qp_s = QuenchPdf { s: 0.0625, n_bell: NB, n_quad: NQ, ..QuenchPdf::default() };
    let pq_s =
        PocketQuenchPdf { s: 0.0625, n_bell: NB16, n_quad: NQ16, ..PocketQuenchPdf::default() };
    let tr_s =
        TransportedPdf { s: 0.0625, n_bell: NB, n_quad: NQ, n_ode: 200, ..TransportedPdf::default() };
    let cases: [(&str, &str, ZonedNoxOpts); 16] = [
        ("r13/J9", "dp1", ZonedNoxOpts { mixing: Some(mix(9.0)), pdf: Some(pdf_s), ..base }),
        ("r13/J16", "dp1", ZonedNoxOpts { mixing: Some(mix(16.0)), pdf: Some(pdf_s), ..base }),
        ("r13/J36", "dp1", ZonedNoxOpts { mixing: Some(mix(36.0)), pdf: Some(pdf_s), ..base }),
        ("r13/J16/su", "dp1", ZonedNoxOpts {
            mixing: Some(mix(16.0)), pdf: Some(pdf_s), super_eq_o: true, ..base }),
        ("r13/J36/su", "dp1", ZonedNoxOpts {
            mixing: Some(mix(36.0)), pdf: Some(pdf_s), super_eq_o: true, ..base }),
        ("r15/J9", "dp1", ZonedNoxOpts {
            mixing: Some(mix(9.0)), pdf_quench: Some(qp_s), ..base }),
        ("r15/J16", "dp1", ZonedNoxOpts {
            mixing: Some(mix(16.0)), pdf_quench: Some(qp_s), ..base }),
        ("r15/J64", "dp1", ZonedNoxOpts {
            mixing: Some(mix(64.0)), pdf_quench: Some(qp_s), ..base }),
        ("r15/J64/su", "dp1", ZonedNoxOpts {
            mixing: Some(mix(64.0)), pdf_quench: Some(qp_s), super_eq_o: true, ..base }),
        ("r16/J16", "dp1", ZonedNoxOpts {
            mixing: Some(mix(16.0)), pocket_quench: Some(pq_s), ..base }),
        ("r16/J64", "dp1", ZonedNoxOpts {
            mixing: Some(mix(64.0)), pocket_quench: Some(pq_s), ..base }),
        ("r18/J9", "dp1", ZonedNoxOpts {
            mixing: Some(mix(9.0)), transported: Some(tr_s), ..base }),
        ("r18/J16", "dp1", ZonedNoxOpts {
            mixing: Some(mix(16.0)), transported: Some(tr_s), ..base }),
        ("r18/J25", "dp1", ZonedNoxOpts {
            mixing: Some(mix(25.0)), transported: Some(tr_s), ..base }),
        ("r18/J16/su", "dp1", ZonedNoxOpts {
            mixing: Some(mix(16.0)), transported: Some(tr_s), super_eq_o: true, ..base }),
        ("r13/dp4", "dp4", ZonedNoxOpts { mixing: Some(mix(16.0)), pdf: Some(pdf_s), ..base }),
    ];
    for (tag, dp, opts) in cases {
        let (_, tt3, tt4, far, p) = find(dp);
        let z = g.zoned_nox(far, tt3, tt4, p, PHI_P, opts);
        out.push((format!("zn/{tag}/ei_no"), z.ei_no()));
        out.push((format!("zn/{tag}/ei_quenched"), z.ei_no_quenched.expect("finite quench")));
        out.push((format!("zn/{tag}/max_a"), z.max_a_quench.expect("finite quench")));
        out.push((format!("zn/{tag}/C_holdeman"), z.c_holdeman.expect("a closure")));
        out.push((format!("zn/{tag}/g_seg"), z.g_seg.expect("a closure")));
        if let Some(v) = z.ei_no_pdf {
            out.push((format!("zn/{tag}/ei_pdf"), v));
        }
        if let Some(v) = z.ei_no_pdf_quench {
            out.push((format!("zn/{tag}/ei_pdf_excess"), z.ei_no_pdf_excess.expect("rung 15")));
            out.push((format!("zn/{tag}/ei_pdf_quench"), v));
        }
        if let Some(v) = z.ei_no_pocket_quench {
            out.push((
                format!("zn/{tag}/ei_pocket_excess"),
                z.ei_no_pocket_excess.expect("rung 16"),
            ));
            out.push((format!("zn/{tag}/ei_pocket_quench"), v));
        }
        if let Some(v) = z.ei_no_transported {
            out.push((format!("zn/{tag}/g_ceiling"), z.g_ceiling.expect("rung 18")));
            out.push((format!("zn/{tag}/g_transported"), z.g_transported.expect("rung 18")));
            out.push((format!("zn/{tag}/ei_transported"), v));
        }
    }
    out
}

/// Which class a key belongs to. Extends the slice-A/B split with the three things this slice
/// adds: the β-PDF quadrature, the bell, and the integrals built on them.
fn quant_of(key: &str) -> &'static str {
    let head = key.split('/').next().unwrap_or("");
    let last = key.rsplit('/').next().unwrap_or("");
    if matches!(
        last,
        "argmax" | "argmin" | "first_burnable" | "loc_agree" | "argmin_g" | "argmin_ei"
            | "argmin_t2"
    ) {
        return "shape_location";
    }
    match head {
        // Closed forms and finite loops over them: no solver, no composition, no integrator.
        "cfg" | "ceil" | "ode" | "mf" | "smooth" => "mixing_algebra",
        "dp" => "design_point",
        "quad" => "quadrature",
        "bell" | "bellpt" => "bell",
        "pdfei" | "pdfmean" | "hump" | "jsweep" | "r21" | "tr" | "r15" => "pdf_integral",
        _ => "kinetic", // r16 (per-pocket quench) and zn (the public wiring)
    }
}

/// The bar for each class — CPYTHON arm only; the PyPy arm is held to BIT-EQUALITY.
///
/// Every number is a MEASUREMENT of the CPython↔PyPy spread on this dump, with headroom — not a
/// guess. Slice B set one class by analogy instead of by measurement and it failed inside the
/// hour, so the measured table is reproduced here in full:
///
/// ```text
///   mixing_algebra  0.00e0   <- EXACTLY equal, all 594 keys (cfg/ceil/ode/mf/smooth)
///   shape_location  0.00e0   <- all 48 locations identical (the VALUES at them are not)
///   jsweep          6.76e-16
///   quadrature      9.74e-16
///   r15             1.09e-15
///   tr              1.30e-15
///   hump            1.30e-15
///   design_point    1.36e-15
///   r16             1.54e-15
///   pdfei / zn      1.66e-15 / 1.85e-15
///   bell            4.89e-15   <- the worst anywhere on the dump
/// ```
///
/// The SPLIT reproduces every earlier slice's: the classes that are EXACTLY equal across
/// interpreters are the ones with no accumulated iterate. `mixing_algebra` here is a wider class
/// than slice B's — it now includes the variance-decay ODE (400 repeated divisions) and a finite
/// difference of two of them, both bit-identical on both interpreters, which is worth noting
/// because a loop of divisions is exactly the shape one would expect to accumulate.
///
/// `shape_location` gets ZERO, and that is the point of dumping locations at all: they are small
/// integers read off a deliberately coarse grid, so ANY movement is a real relocation rather
/// than last-bit noise.
fn bar_for(quant: &str) -> f64 {
    match quant {
        "shape_location" => 0.0,
        "mixing_algebra" => 1.0e-15,
        _ => 1.0e-12,
    }
}

fn compare_against(oracle_text: &str, label: &str, require_bit_exact: bool) {
    let oracle = load_oracle(oracle_text);
    let ours = rust_values();
    println!("\n=== Rust vs {label} ===");

    assert_eq!(
        ours.len(),
        oracle.len(),
        "key COUNT differs: rust {} vs oracle {} — the dump and the test have drifted apart, \
         so a missing key would otherwise read as a pass",
        ours.len(),
        oracle.len()
    );

    let mut missing: Vec<&str> = Vec::new();
    let mut per: HashMap<&str, (usize, usize, f64, String)> = HashMap::new();
    let mut failures: Vec<String> = Vec::new();

    for (key, got) in &ours {
        let Some(&want) = oracle.get(key.as_str()) else {
            missing.push(key);
            continue;
        };
        let q = quant_of(key);
        let e = per.entry(q).or_insert((0, 0, 0.0, String::new()));
        e.0 += 1;
        if got.to_bits() == want.to_bits() {
            e.1 += 1;
            continue;
        }
        let scale = got.abs().max(want.abs());
        let rel = if scale > 0.0 { (got - want).abs() / scale } else { (got - want).abs() };
        if rel > e.2 {
            e.2 = rel;
            e.3 = key.clone();
        }
        if rel > bar_for(q) {
            failures.push(format!(
                "  {key:<52} rust {got:.17e}  oracle {want:.17e}  rel {rel:.2e}"
            ));
        }
    }

    let mut rows: Vec<_> = per.iter().collect();
    rows.sort_by(|a, b| b.1 .2.partial_cmp(&a.1 .2).unwrap());
    println!(
        "\n{:<16} {:>6} {:>11} {:>12} {:>12}",
        "quantity", "keys", "bit-exact", "worst rel", "bar"
    );
    println!("{}", "-".repeat(62));
    for (q, (n, exact, worst, _)) in &rows {
        println!("{:<16} {:>6} {:>11} {:>12.2e} {:>12.0e}", q, n, exact, worst, bar_for(q));
    }
    let total: usize = rows.iter().map(|r| r.1 .0).sum();
    let exact: usize = rows.iter().map(|r| r.1 .1).sum();
    println!(
        "\n{exact} / {total} bit-identical to {label} ({:.2}%)",
        100.0 * exact as f64 / total as f64
    );
    for (q, (_, _, worst, key)) in &rows {
        if *worst > 0.0 {
            println!("  worst {q:<16} {worst:.2e}  at {key}");
        }
    }

    assert!(missing.is_empty(), "keys computed by Rust but absent from the oracle: {missing:?}");
    assert!(
        failures.is_empty(),
        "{} value(s) outside the measured bar:\n{}",
        failures.len(),
        failures.join("\n")
    );
    if require_bit_exact {
        let drifted: Vec<&String> =
            rows.iter().filter(|(_, (_, _, w, _))| *w > 0.0).map(|(_, (_, _, _, k))| k).collect();
        assert_eq!(
            exact, total,
            "phase 3C measured {total}/{total} BIT-IDENTICAL to {label}; this run got {exact}. \
             A drop is either a real arithmetic regression or a toolchain/libm change — find out \
             WHICH before loosening this to a tolerance. Phase 1 ran its own arm at 98.89 % and \
             the missing 1.11 % was a transcription bug in a polynomial's power spelling. First \
             drifted keys: {drifted:?}"
        );
    }
}

#[test]
fn pdf_matches_the_cpython_oracle() {
    compare_against(ORACLE_CPYTHON, "CPython 3.14.3", false);
}

/// The same comparison against the interpreter the test gate actually runs on — and here the bar
/// is BIT-EQUALITY, not a tolerance.
///
/// Not redundant with the CPython arm; it is the DISCRIMINATOR. Either Rust has its own drift
/// that coincidentally matches PyPy's, or Rust and PyPy are computing the same function. The
/// CPython arm's ~29 % disagreement is what makes the coincidence implausible.
#[test]
fn pdf_matches_the_pypy_oracle_to_the_bit() {
    compare_against(ORACLE_PYPY, "PyPy 3.11.15", true);
}

/// The β-PDF quadrature's own mean-preservation guard must FIRE where the Python's does.
///
/// This is the half of a guard that a value gate can never see. The bar is `n_quad`-sensitive:
/// at a lean mean it REJECTS `g = 0.026` (the first grid point past the `a = 1` branch switch)
/// and `g = 0.40` for every `n_quad ≤ 100`, and accepts both from 112 up. Measured under both
/// interpreters before the dump was written, because the dump's first run crashed on exactly
/// this and the temptation was to lower the sweep and move on.
///
/// A port that widened the tolerance, or that took the wrong quadrature branch and happened to
/// integrate more accurately, would pass every `quad/` value key and fail here.
#[test]
fn the_quadrature_guard_fires_below_its_measured_n_quad_floor() {
    let xibar = FAR_LEAN_LITERAL / (1.0 + FAR_LEAN_LITERAL);
    for &g_seg in &[0.026f64, 0.40] {
        for &nq in &[64usize, 100] {
            let r = std::panic::catch_unwind(|| beta_pdf_nodes_weights(xibar, g_seg, nq));
            let Err(e) = r else {
                panic!(
                    "g={g_seg} at n_quad={nq} must FAIL the 1 % mean-preservation guard — \
                     measured in both interpreters. If this now passes, the quadrature changed."
                );
            };
            // It must fail for THAT reason. The `b ≥ 1` shape assert and the variance assert
            // live in the same function and would satisfy a bare `is_err()`.
            let msg = panic_text(e);
            assert!(
                msg.contains("drifted the mean"),
                "g={g_seg} at n_quad={nq} panicked, but not on the mean-preservation guard: {msg}"
            );
        }
        for &nq in &[112usize, 160] {
            let (nodes, w) = beta_pdf_nodes_weights(xibar, g_seg, nq);
            let mean = w.iter().zip(nodes.iter()).fold(0.0, |a, (&wi, &x)| a + wi * x);
            assert!(
                (mean - xibar).abs() <= 0.01 * xibar,
                "g={g_seg} at n_quad={nq} must PASS: ⟨ξ⟩={mean}"
            );
        }
    }
}

/// The `b ≥ 1` guard, which is what caps every `g` sweep in this slice.
///
/// `b = (1−ξ̄)(1/g − 1) ≥ 1` ⟺ `g ≤ (1−ξ̄)/(2−ξ̄)`. The dump sweeps to 0.40 and the cap is 0.4933
/// at the lean mean; this pins the boundary itself so the sweep's headroom is a measured fact
/// rather than a comment.
#[test]
fn the_shape_guard_caps_the_segregation_sweep() {
    let xibar = FAR_LEAN_LITERAL / (1.0 + FAR_LEAN_LITERAL);
    let cap = (1.0 - xibar) / (2.0 - xibar);
    assert!(
        (0.49..0.50).contains(&cap),
        "the b≥1 cap moved: {cap} (the dump's 0.40 sweep ceiling assumes ~0.493)"
    );
    let over = std::panic::catch_unwind(move || beta_pdf_nodes_weights(xibar, cap * 1.02, 200));
    let Err(e) = over else { panic!("g just past (1−ξ̄)/(2−ξ̄) must fail the b≥1 shape guard") };
    let msg = panic_text(e);
    assert!(
        msg.contains("outside a>0,b≥1"),
        "it panicked, but not on the SHAPE guard — the mean-preservation assert is in the same \
         function and would satisfy a bare is_err(): {msg}"
    );
}

/// The quadrature rests on TWO integration schemes, not one — asserted as a count.
///
/// "The quadrature reproduces 703/703" would be worth much less if every one of those keys sat
/// on the same branch. `a = ξ̄(1/g − 1)` crosses 1 at `g = ξ̄/(1+ξ̄) ≈ 0.0258` for the lean mean,
/// and the `G_GRID` straddles it deliberately: this asserts that both branches are actually
/// exercised, and that the node sets they produce are structurally different (the singular
/// branch starts near zero; the windowed one starts near the mean).
#[test]
fn both_quadrature_branches_are_exercised() {
    let xibar = FAR_LEAN_LITERAL / (1.0 + FAR_LEAN_LITERAL);
    let switch = xibar / (1.0 + xibar);
    let (mut singular, mut windowed) = (0usize, 0usize);
    for &g in G_GRID {
        let a = xibar * (1.0 / g - 1.0);
        let (nodes, _) = beta_pdf_nodes_weights(xibar, g, NQ);
        // The discriminator is the node SPAN, not the first node. (A first draft asserted
        // `nodes[0] < 0.1·ξ̄`, which is a made-up threshold and duly failed at g = 0.026, where
        // `a = 0.991` is only just singular and the substitution is nearly the identity.) The
        // structural difference is real and large: `u = ξ^a` spreads nodes over essentially the
        // whole unit interval, while the `a ≥ 1` window is a ±8σ band around a lean mean.
        let last = nodes[nodes.len() - 1];
        if a < 1.0 {
            singular += 1;
            assert!(
                last > 0.9,
                "the singular (u = ξ^a) branch must span to ξ≈1, got last node {last}"
            );
        } else {
            windowed += 1;
            assert!(
                nodes[0] > 0.0 && last > xibar && last < 0.5,
                "the windowed (a ≥ 1) branch must be a narrow band BRACKETING the mean, got \
                 [{}, {last}] around ξ̄={xibar}",
                nodes[0]
            );
        }
    }
    println!("branch switch at g={switch:.6}: {windowed} windowed, {singular} singular");
    assert!(
        windowed >= 3 && singular >= 4,
        "the g grid must exercise BOTH schemes with room to spare, got {windowed} windowed / \
         {singular} singular"
    );
}

/// Rung 21's location key, stated as its own test rather than left inside the value sweep.
///
/// The rung's claim is that the super-equilibrium-O lift is SHAPE-PRESERVING: it moves every EI
/// but not where the optimum sits. Pre-registered to CONFIRM, at four spacings and two design
/// points — deliberately wider than the Python's own gate, which samples two spacings, because
/// slice B measured a location key REFUTING the claim it was dumped to confirm and the reason it
/// could was that the source's gate only sampled where the claim was true.
#[test]
fn the_super_eq_lift_moves_the_values_and_not_the_optimum() {
    let dps = design_points();
    let hf = gas::hf_fuel_default();
    for &(name, tt3, _tt4, far, p) in &dps {
        let xibar = far / (1.0 + far);
        let eq = bell_interpolator(p, tt3, hf, TAU, NB, false);
        let su = bell_interpolator(p, tt3, hf, TAU, NB, true);
        for &s in S_GRID {
            let pdf = MixingPdf { s, ..MixingPdf::default() };
            let jo = j_opt(s, pdf.c_opt);
            let js = [jo / 4.0, jo / 2.0, jo, 2.0 * jo, 4.0 * jo];
            let curve = |b: &Bell| -> Vec<f64> {
                js.iter()
                    .map(|&j| pdf_mean_ei_on_bell(b, xibar, pdf.segregation(pdf.c(&mix(j))), NQ))
                    .collect()
            };
            let (a, b) = (curve(&eq), curve(&su));
            assert_eq!(
                argmin(&a),
                argmin(&b),
                "{name}/S={s}: the lift RELOCATED the optimum ({} → {}) — that is a finding, \
                 not a failure: rung 21 claims the lift is shape-preserving",
                argmin(&a),
                argmin(&b)
            );
            assert_eq!(argmin(&a), 2, "{name}/S={s}: the optimum must sit AT J_opt (index 2)");
            // and it really is a LIFT: every value moves, so the equal argmin is not a tautology
            for (i, (&x, &y)) in a.iter().zip(b.iter()).enumerate() {
                assert!(
                    y > x,
                    "{name}/S={s}: the super-eq arm must lift EVERY point; index {i} did not \
                     ({y} vs {x})"
                );
            }
        }
    }
}
