//! PHASE 3D GATE — every rung-22/23/24 resolved-cross-plane value the Python oracle dumped,
//! recomputed in Rust.
//!
//! The sixth in the family (`gas_oracle.rs` → `cycle_oracle.rs` → `nox_oracle.rs` →
//! `quench_oracle.rs` → `pdf_oracle.rs` → here), and a separate file for the same reason the dump
//! is: each gate's cost stays proportional to what it certifies, and the earlier slices' TSVs
//! stay frozen as their own audit trail.
//!
//! WHAT THIS GATE IS BUILT TO CATCH:
//!
//! * **THE SUMMATION-SHAPE ASYMMETRY, which is the slice's headline.** Three functions compute
//!   the same `g` by three routes. Rung 23's is BIT-EQUAL to rung 22's; rung 24's is NOT, and the
//!   entire difference is that rung 24's mean is `sum(sum(r) for r in xi)` (hierarchical) while
//!   its mean-square two lines later is flat. The `d23/` keys are asserted to be EXACTLY `+0.0`
//!   and the `d24/` keys are asserted to be NON-zero and small — so a port that "tidies" the
//!   hierarchical sum into a flat one, which would be MORE accurate, FAILS here. The Python's own
//!   docstrings claim the opposite pairing and its own gate asserts `< 1e-9` on both, which is why
//!   the source cannot see this.
//! * **A 60-STEP BISECTION WHOSE BRACKET IS DECIDED BY AN ACCUMULATED MEAN.** Every field runs it,
//!   and a single last-bit difference in the accumulation can send one halving the other way. The
//!   `g22/` sweep spans three grids × five J × three geometries × two design points for exactly
//!   that reason.
//! * **DISCRETE OUTPUTS NO TOLERANCE CAN SEE.** The τ(ξ) binner keeps only NON-EMPTY bins, so its
//!   knot count is data-dependent (`knots/`); rung 24's `u < 1e-8` stagnant branch is taken by
//!   18–50 % of cells, so its census is a real integer (`stag/`). Both are dumped per grid and
//!   per J.
//! * **THE CIRCULARITY KILL TEST.** `u` carries an explicit `1/var` and rung 22 already mins `g`
//!   at `C_opt`, so "argmin F == argmin g" is a TELL. The `grad/` keys are `⟨|∇ξ|²⟩`, which
//!   carries no `g`, and they are rebuilt HERE from the field without variance normalisation —
//!   reading them off a production accessor would compare production to itself.
//!
//! The bars are not invented; they are the measured CPython↔PyPy spread on this dump. See
//! [`bar_for`] for the table.
//!
//! Regenerate the oracle with:
//!     py -3                     rust/oracle/dump_spatial.py rust/oracle/spatial_cpython.tsv
//!     .venv\Scripts\python.exe  rust/oracle/dump_spatial.py rust/oracle/spatial_pypy.tsv

use std::collections::HashMap;
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{self, Gas};
use turbojet::nox::{
    beta_pdf_nodes_weights, spatial_dwell_field, spatial_local_field, spatial_local_stagnant_cells,
    spatial_segregation, two_stream_ceiling, JetMixing, SpatialDwellPdf, SpatialLocalPdf,
    SpatialPdf, ZonedNoxOpts,
};

const ORACLE_CPYTHON: &str = include_str!("../oracle/spatial_cpython.tsv");
const ORACLE_PYPY: &str = include_str!("../oracle/spatial_pypy.tsv");

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

// --- the grids, transcribed from `dump_spatial.py` --------------------------------------------
const TAU: f64 = 3e-3;
const PHI_P: f64 = 1.5;
const CE: f64 = 0.20;
const S0: f64 = 0.0625;
const H0: f64 = 0.10;
const KP: f64 = 0.316;
const KY: f64 = 0.28;
const KZ: f64 = 0.28;
const NY_SWEEP: &[usize] = &[16, 32, 48];
const NT: usize = 24;
const NB16: usize = 20;
const NQ16: usize = 64;
const NGRID: usize = 17;
const NSTEPS: usize = 200;
const J_COARSE: &[f64] = &[4.0, 9.0, 16.0, 36.0, 64.0];
const TAU_MIX_REF: f64 = 1.0e-3;

fn mix(j: f64, h: f64) -> JetMixing {
    JetMixing { j, h, c_e: CE, shape_n: 2.0, ..Default::default() }
}

/// Python's `repr(float)` for the keys the dump builds by formatting a float.
fn jtag(v: f64) -> String {
    if v == v.trunc() && v.abs() < 1e16 {
        format!("{v:.1}")
    } else {
        format!("{v}")
    }
}

/// `⟨|∇ξ|²⟩` of the terminal field, rebuilt with NO variance normalisation — the g-free witness.
///
/// Deliberately NOT a call into production. The whole point of this quantity is that it is
/// independent of the `1/var` coupling that makes "argmin F == argmin g" circular, so it is
/// transcribed from the dump's own helper, exactly as the Python's rung-24 suite transcribes it.
fn mean_grad_sq(far: f64, s_sp: f64, h: f64, j: f64, ny: usize, nz: usize) -> f64 {
    let xibar = far / (1.0 + far);
    let far_p = PHI_P * gas::f_stoich();
    let xi_p = far_p / (1.0 + far_p);
    let delta = KP * (s_sp * h).sqrt() * gas::powp(j, 0.25);
    let (sig_y, sig_z) = (KY * h, KZ * s_sp);
    let ys: Vec<f64> = (0..ny).map(|i| (i as f64 + 0.5) * h / ny as f64).collect();
    let zs: Vec<f64> = (0..nz).map(|k| (k as f64 + 0.5) * s_sp / nz as f64).collect();
    let ay: Vec<f64> = ys
        .iter()
        .map(|&y| {
            [-delta, delta, 2.0 * h - delta, 2.0 * h + delta]
                .iter()
                .map(|&c| (-((y - c) * (y - c)) / (2.0 * sig_y * sig_y)).exp())
                .sum::<f64>()
        })
        .collect();
    let az: Vec<f64> = zs
        .iter()
        .map(|&z| {
            [-1.0f64, 0.0, 1.0]
                .iter()
                .map(|&m| {
                    let d = z - s_sp / 2.0 - m * s_sp;
                    (-(d * d) / (2.0 * sig_z * sig_z)).exp()
                })
                .sum::<f64>()
        })
        .collect();
    let may = ay.iter().sum::<f64>() / ny as f64;
    let maz = az.iter().sum::<f64>() / nz as f64;
    let ayh: Vec<f64> = ay.iter().map(|&a| a / may).collect();
    let azh: Vec<f64> = az.iter().map(|&a| a / maz).collect();
    let beta_bar = (xi_p - xibar) / xi_p;
    // the dump's `mean_at` is a single flat generator sum over the outer product, y outer
    let mean_at = |s: f64| -> f64 {
        let mut t = 0.0f64;
        for &a in &ayh {
            for &b in &azh {
                t += xi_p * (1.0 - (s * beta_bar * a * b).clamp(0.0, 1.0));
            }
        }
        t / (ny * nz) as f64
    };
    let (mut lo, mut hi) = (0.0f64, 50.0f64);
    for _ in 0..60 {
        let s = 0.5 * (lo + hi);
        if mean_at(s) > xibar {
            lo = s;
        } else {
            hi = s;
        }
    }
    let s_star = 0.5 * (lo + hi);
    let xi: Vec<Vec<f64>> = ayh
        .iter()
        .map(|&a| {
            azh.iter()
                .map(|&b| xi_p * (1.0 - (s_star * beta_bar * a * b).clamp(0.0, 1.0)))
                .collect()
        })
        .collect();
    let (dy, dz) = (h / ny as f64, s_sp / nz as f64);
    let mut tot = 0.0f64;
    for i in 0..ny {
        let (im, ip) = (i.saturating_sub(1), (i + 1).min(ny - 1));
        for k in 0..nz {
            let km = (k + nz - 1) % nz;
            let kp = (k + 1) % nz;
            let gy = (xi[ip][k] - xi[im][k]) / ((ip - im) as f64 * dy);
            let gz = (xi[i][kp] - xi[i][km]) / (2.0 * dz);
            tot += gy * gy + gz * gz;
        }
    }
    tot / (ny * nz) as f64
}

fn rust_values() -> HashMap<String, f64> {
    let mut m: HashMap<String, f64> = HashMap::new();
    let mut put = |k: String, v: f64| {
        assert!(v.is_finite(), "{k} is not finite: {v}");
        assert!(m.insert(k.clone(), v).is_none(), "duplicate key {k}");
    };

    // --- SECTION 1: solver-free algebra -------------------------------------------------
    put("alg/f_stoich".into(), gas::f_stoich());
    put(
        "alg/xi_soot".into(),
        (2.0 * gas::f_stoich()) / (1.0 + 2.0 * gas::f_stoich()),
    );
    for kp in [0.25f64, 0.316, 0.40] {
        put(format!("alg/c_opt/kp{kp}"), 1.0 / (4.0 * kp * kp));
    }
    for j in [1.0f64, 4.0, 16.0, 100.0, 400.0] {
        put(
            format!("alg/delta/J{}", jtag(j)),
            KP * (S0 * H0).sqrt() * gas::powp(j, 0.25),
        );
        put(format!("alg/quarter/J{}", jtag(j)), gas::powp(j, 0.25));
        put(format!("alg/C/J{}", jtag(j)), (S0 / H0) * j.sqrt());
    }
    for frac in [1.0 / 24.0, 0.5, 23.0 / 24.0, 1.0] {
        put(format!("alg/cbrt/{frac:.6}"), gas::powp(frac, 1.0 / 3.0));
        put(format!("alg/sqrtf/{frac:.6}"), frac.sqrt());
    }

    let flight_sub = FlightCondition { t0: 250.0, p0: 50_000.0, m0: 0.85 };
    let flight_sup = FlightCondition { t0: 216.7, p0: 18_750.0, m0: 2.0 };
    let losses = Losses {
        pi_d: 0.97,
        eta_c: 0.88,
        eta_b: 0.99,
        pi_b: 0.96,
        eta_t: 0.90,
        eta_m: 0.99,
        pi_n: 0.98,
        ..Default::default()
    };
    let mut design: HashMap<&str, (f64, f64, f64, f64)> = HashMap::new();
    for (name, flight, pi_c, tt4, mdot) in [
        ("dp1", flight_sub, 10.0, 1500.0, 50.0),
        ("dp4", flight_sup, 12.0, 1800.0, 50.0),
    ] {
        let gas_eq = Gas::reacting_equilibrium();
        let r = build_turbojet(gas_eq, pi_c, tt4, flight.p0, losses).run(&flight, mdot);
        let st3 = r.station("3");
        let st4 = r.station("4");
        design.insert(name, (st3.tt, st4.tt, st4.far, st4.pt));
        put(format!("dp/{name}/Tt3"), st3.tt);
        put(format!("dp/{name}/Tt4"), st4.tt);
        put(format!("dp/{name}/far"), st4.far);
        put(format!("dp/{name}/pt4"), st4.pt);
        put(format!("dp/{name}/xibar"), st4.far / (1.0 + st4.far));
    }
    let far1 = design["dp1"].2;
    let far4 = design["dp4"].2;
    for (name, far) in [("dp1", far1), ("dp4", far4)] {
        put(format!("ceil/{name}"), two_stream_ceiling(far, PHI_P));
    }

    // --- SECTION 2: the resolved width, and the three routes to it ----------------------
    for &ny in NY_SWEEP {
        for j in [1.0f64, 4.0, 16.0, 100.0, 400.0] {
            let g22 = spatial_segregation(far1, PHI_P, S0, H0, j, KP, KY, KZ, ny, ny);
            let (g23, _) =
                spatial_dwell_field(far1, PHI_P, S0, H0, j, TAU_MIX_REF, KP, KY, KZ, ny, ny, NT);
            let (g24, _, _) =
                spatial_local_field(far1, PHI_P, S0, H0, j, TAU_MIX_REF, KP, KY, KZ, ny, ny);
            let tag = format!("n{ny}/J{}", jtag(j));
            put(format!("g22/{tag}"), g22);
            put(format!("g23/{tag}"), g23);
            put(format!("g24/{tag}"), g24);
            put(format!("d23/{tag}"), g23 - g22);
            put(format!("d24/{tag}"), g24 - g22);
        }
    }
    for (label, s_sp, h) in
        [("halfS", S0 / 2.0, H0), ("dblH", S0, 2.0 * H0), ("both", 2.0 * S0, 2.0 * H0)]
    {
        for j in [4.0f64, 16.0, 64.0, 256.0] {
            put(
                format!("g22/{label}/J{}", jtag(j)),
                spatial_segregation(far1, PHI_P, s_sp, h, j, KP, KY, KZ, 32, 32),
            );
        }
    }
    for j in [4.0f64, 16.0, 64.0] {
        put(
            format!("g22/dp4/J{}", jtag(j)),
            spatial_segregation(far4, PHI_P, S0, H0, j, KP, KY, KZ, 32, 32),
        );
    }

    // --- SECTION 3: the location keys, on the COARSE grid -------------------------------
    for (label, s_sp, h, kp) in [
        ("base", S0, H0, KP),
        ("halfS", S0 / 2.0, H0, KP),
        ("dblH", S0, 2.0 * H0, KP),
        ("kp25", S0, H0, 0.25),
        ("kp40", S0, H0, 0.40),
    ] {
        let grid: Vec<f64> = if label == "halfS" || label == "dblH" {
            J_COARSE.iter().map(|&j| j * 4.0).collect()
        } else {
            J_COARSE.to_vec()
        };
        let gs: Vec<f64> = grid
            .iter()
            .map(|&j| spatial_segregation(far1, PHI_P, s_sp, h, j, kp, KY, KZ, 32, 32))
            .collect();
        let i = argmin(&gs);
        put(format!("loc/g/{label}/idx"), i as f64);
        put(format!("loc/g/{label}/J"), grid[i]);
        put(format!("loc/g/{label}/C"), (s_sp / h) * grid[i].sqrt());
        put(format!("loc/g/{label}/gmin"), gs[i]);
        if i > 0 && i < gs.len() - 1 {
            put(format!("loc/g/{label}/clear_lo"), gs[i - 1] / gs[i]);
            put(format!("loc/g/{label}/clear_hi"), gs[i + 1] / gs[i]);
        }
        for (k, &gv) in gs.iter().enumerate() {
            put(format!("loc/g/{label}/v{k}"), gv);
        }
    }
    let j_fine: Vec<f64> = (0..49).map(|i| gas::powp(400.0, i as f64 / 48.0)).collect();
    for k in [0usize, 12, 24, 30, 36, 48] {
        put(format!("fine/J{k}"), j_fine[k]);
        put(
            format!("fine/g{k}"),
            spatial_segregation(far1, PHI_P, S0, H0, j_fine[k], KP, KY, KZ, 32, 32),
        );
    }

    // --- SECTION 4: F(C), the τ_mix cancellation, and the g-free witness ----------------
    let mut f_coarse = Vec::new();
    for &j in J_COARSE {
        let (_, _, f) =
            spatial_local_field(far1, PHI_P, S0, H0, j, TAU_MIX_REF, KP, KY, KZ, 32, 32);
        f_coarse.push(f);
        put(format!("F/base/J{}", jtag(j)), f);
    }
    let i_f = argmin(&f_coarse);
    put("loc/F/base/idx".into(), i_f as f64);
    put("loc/F/base/J".into(), J_COARSE[i_f]);
    put("loc/F/base/C".into(), (S0 / H0) * J_COARSE[i_f].sqrt());
    put("loc/F/base/clear_lo".into(), f_coarse[i_f - 1] / f_coarse[i_f]);
    put("loc/F/base/clear_hi".into(), f_coarse[i_f + 1] / f_coarse[i_f]);
    let fmax = f_coarse.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let fmin = f_coarse.iter().cloned().fold(f64::INFINITY, f64::min);
    put("F/base/depth".into(), fmax / fmin);
    for j in [4.0f64, 16.0, 64.0] {
        let (_, _, fa) = spatial_local_field(far1, PHI_P, S0, H0, j, 1.0e-4, KP, KY, KZ, 32, 32);
        let (_, _, fb) = spatial_local_field(far1, PHI_P, S0, H0, j, 1.0e-1, KP, KY, KZ, 32, 32);
        put(format!("F/cancel/J{}/a", jtag(j)), fa);
        put(format!("F/cancel/J{}/b", jtag(j)), fb);
        put(format!("F/cancel/J{}/d", jtag(j)), fb - fa);
    }
    let grads: Vec<f64> =
        J_COARSE.iter().map(|&j| mean_grad_sq(far1, S0, H0, j, 32, 32)).collect();
    for (&j, &gv) in J_COARSE.iter().zip(grads.iter()) {
        put(format!("grad/base/J{}", jtag(j)), gv);
    }
    let i_g = argmax(&grads);
    put("loc/grad/base/idx".into(), i_g as f64);
    put("loc/grad/base/J".into(), J_COARSE[i_g]);
    put("loc/grad/base/clear_lo".into(), grads[i_g] / grads[i_g - 1]);
    put("loc/grad/base/clear_hi".into(), grads[i_g] / grads[i_g + 1]);
    for (label, s_sp, grid) in [
        ("halfS", S0 / 2.0, J_COARSE.iter().map(|&j| j * 4.0).collect::<Vec<_>>()),
        ("base", S0, J_COARSE.to_vec()),
    ] {
        let fs: Vec<f64> = grid
            .iter()
            .map(|&j| {
                spatial_local_field(far1, PHI_P, s_sp, H0, j, TAU_MIX_REF, KP, KY, KZ, 32, 32).2
            })
            .collect();
        let i = argmin(&fs);
        put(format!("loc/Fshift/{label}/idx"), i as f64);
        put(format!("loc/Fshift/{label}/J"), grid[i]);
        put(format!("loc/Fshift/{label}/C"), (s_sp / H0) * grid[i].sqrt());
    }

    // --- SECTION 5: the DISCRETE keys ---------------------------------------------------
    let stag: Vec<usize> = J_COARSE
        .iter()
        .map(|&j| spatial_local_stagnant_cells(far1, PHI_P, S0, H0, j, KP, KY, KZ, 32, 32))
        .collect();
    for (&j, &c) in J_COARSE.iter().zip(stag.iter()) {
        put(format!("stag/base/J{}", jtag(j)), c as f64);
    }
    let stag_f: Vec<f64> = stag.iter().map(|&c| c as f64).collect();
    put("loc/stag/base/idx".into(), argmin(&stag_f) as f64);
    put("loc/stag/base/J".into(), J_COARSE[argmin(&stag_f)]);
    put(
        "stag/base/frac_min".into(),
        *stag.iter().min().unwrap() as f64 / (32.0 * 32.0),
    );
    put(
        "stag/base/frac_max".into(),
        *stag.iter().max().unwrap() as f64 / (32.0 * 32.0),
    );
    for &ny in NY_SWEEP {
        put(
            format!("stag/n{ny}/J16"),
            spatial_local_stagnant_cells(far1, PHI_P, S0, H0, 16.0, KP, KY, KZ, ny, ny) as f64,
        );
    }
    for &ny in NY_SWEEP {
        for j in [4.0f64, 16.0, 100.0] {
            let (_, t23) =
                spatial_dwell_field(far1, PHI_P, S0, H0, j, TAU_MIX_REF, KP, KY, KZ, ny, ny, NT);
            let (_, t24, _) =
                spatial_local_field(far1, PHI_P, S0, H0, j, TAU_MIX_REF, KP, KY, KZ, ny, ny);
            put(format!("knots/r23/n{ny}/J{}", jtag(j)), t23.n_knots() as f64);
            put(format!("knots/r24/n{ny}/J{}", jtag(j)), t24.n_knots() as f64);
        }
    }

    // --- SECTION 6: the τ(ξ) spectra ----------------------------------------------------
    let xi_max = (2.0 * gas::f_stoich()) / (1.0 + 2.0 * gas::f_stoich());
    let xibar1 = far1 / (1.0 + far1);
    for j in [4.0f64, 16.0, 64.0] {
        let (g23, t23) =
            spatial_dwell_field(far1, PHI_P, S0, H0, j, TAU_MIX_REF, KP, KY, KZ, 32, 32, NT);
        let (g24, t24, _) =
            spatial_local_field(far1, PHI_P, S0, H0, j, TAU_MIX_REF, KP, KY, KZ, 32, 32);
        for k in 0..9 {
            let xi = xi_max * k as f64 / 8.0;
            put(format!("tau23/J{}/x{k}", jtag(j)), t23.at(xi));
            put(format!("tau24/J{}/x{k}", jtag(j)), t24.at(xi));
        }
        for (tag, tf, gseg) in [("r23", &t23, g23), ("r24", &t24, g24)] {
            let (nodes, wts) = beta_pdf_nodes_weights(xibar1, gseg, NQ16);
            let tm = wts.iter().zip(nodes.iter()).fold(0.0, |acc, (&wi, &x)| acc + wi * tf.at(x));
            put(format!("taumean/{tag}/J{}", jtag(j)), tm);
        }
    }

    // --- SECTION 7: end to end through `zoned_nox` --------------------------------------
    let gas_eq = Gas::reacting_equilibrium();
    let sp_s = SpatialPdf { s: S0, ny: 24, nz: 24, n_bell: NB16, n_quad: NQ16, ..Default::default() };
    let sd_s = SpatialDwellPdf {
        s: S0,
        ny: 20,
        nz: 20,
        nt: 16,
        n_bell: NB16,
        n_quad: NQ16,
        ..Default::default()
    };
    let sl_s =
        SpatialLocalPdf { s: S0, ny: 20, nz: 20, n_bell: NB16, n_quad: NQ16, ..Default::default() };

    let mut dump_zoned = |tag: String, dp: &str, o: ZonedNoxOpts, put: &mut dyn FnMut(String, f64)| {
        let (tt3, tt4, far, p) = design[dp];
        let z = gas_eq.zoned_nox(far, tt3, tt4, p, PHI_P, o);
        put(format!("zn/{tag}/ei_no"), z.ei_no());
        put(format!("zn/{tag}/ei_quenched"), z.ei_no_quenched.expect("finite quench"));
        put(format!("zn/{tag}/max_a"), z.max_a_quench.expect("finite quench"));
        put(format!("zn/{tag}/C_holdeman"), z.c_holdeman.expect("mixing"));
        put(format!("zn/{tag}/g_seg"), z.g_seg.expect("a closure"));
        put(format!("zn/{tag}/g_ceiling"), z.g_ceiling.expect("a spatial closure"));
        if let Some(v) = z.ei_no_spatial {
            put(format!("zn/{tag}/g_spatial"), z.g_spatial.unwrap());
            put(format!("zn/{tag}/ei_spatial"), v);
        }
        if let Some(v) = z.ei_no_spatial_dwell {
            put(format!("zn/{tag}/g_dwell"), z.g_spatial_dwell.unwrap());
            put(format!("zn/{tag}/tau_mean"), z.tau_mean_dwell.unwrap());
            put(format!("zn/{tag}/ei_excess"), z.ei_no_spatial_dwell_excess.unwrap());
            put(format!("zn/{tag}/ei_dwell"), v);
            put(format!("zn/{tag}/ei_meanfield"), z.ei_no_spatial_dwell_meanfield.unwrap());
            put(format!("zn/{tag}/corr_ratio"), z.corr_ratio.unwrap());
        }
        if let Some(v) = z.ei_no_spatial_local {
            put(format!("zn/{tag}/g_local"), z.g_spatial_local.unwrap());
            put(format!("zn/{tag}/f_shape"), z.f_shape.unwrap());
            put(format!("zn/{tag}/tau_mean"), z.tau_mean_local.unwrap());
            put(format!("zn/{tag}/ei_excess"), z.ei_no_spatial_local_excess.unwrap());
            put(format!("zn/{tag}/ei_local"), v);
            put(format!("zn/{tag}/ei_meanfield"), z.ei_no_spatial_local_meanfield.unwrap());
            put(format!("zn/{tag}/corr_ratio"), z.corr_ratio_local.unwrap());
        }
    };

    let base = |j: f64| ZonedNoxOpts {
        tau: TAU,
        mixing: Some(mix(j, H0)),
        quench_ngrid: NGRID,
        quench_nsteps: NSTEPS,
        ..Default::default()
    };
    for j in [4.0f64, 16.0, 64.0] {
        dump_zoned(
            format!("r22/J{}", jtag(j)),
            "dp1",
            ZonedNoxOpts { spatial: Some(sp_s), ..base(j) },
            &mut put,
        );
    }
    dump_zoned(
        "r22/J16/su".into(),
        "dp1",
        ZonedNoxOpts { spatial: Some(sp_s), super_eq_o: true, ..base(16.0) },
        &mut put,
    );
    dump_zoned(
        "r22/dp4".into(),
        "dp4",
        ZonedNoxOpts { spatial: Some(sp_s), ..base(16.0) },
        &mut put,
    );
    for j in [4.0f64, 16.0, 64.0] {
        dump_zoned(
            format!("r23/J{}", jtag(j)),
            "dp1",
            ZonedNoxOpts { spatial_dwell: Some(sd_s), ..base(j) },
            &mut put,
        );
    }
    dump_zoned(
        "r23/J16/su".into(),
        "dp1",
        ZonedNoxOpts { spatial_dwell: Some(sd_s), super_eq_o: true, ..base(16.0) },
        &mut put,
    );
    for j in [4.0f64, 16.0, 64.0] {
        dump_zoned(
            format!("r24/J{}", jtag(j)),
            "dp1",
            ZonedNoxOpts { spatial_local: Some(sl_s), ..base(j) },
            &mut put,
        );
    }
    dump_zoned(
        "r24/J16/su".into(),
        "dp1",
        ZonedNoxOpts { spatial_local: Some(sl_s), super_eq_o: true, ..base(16.0) },
        &mut put,
    );

    m
}

/// Which measured class a key belongs to.
fn quant_of(key: &str) -> &'static str {
    let head = key.split('/').next().unwrap_or("");
    let last = key.rsplit('/').next().unwrap_or("");
    if last == "idx" {
        return "shape_location";
    }
    // RESIDUALS — differences of two nearly-equal quantities, so a RELATIVE bar is the wrong
    // currency: `d24` is ~1e-17 against operands of ~1e-2, and two interpreters that differ in
    // the operands' last bits can differ by O(1) *relatively* in the difference. Measured: the
    // worst relative disagreement on `d24/n16/J1.0` is 1.60, and on `F/cancel/J4.0/d` it is 0.44
    // — both of which are last-bit noise reported in a unit that cannot see it. These keys are
    // compared ABSOLUTELY instead. Same lesson as the golden fingerprint gate's slice 5.
    if head == "d23" || head == "d24" || (head == "F" && last == "d") {
        return "residual";
    }
    match head {
        // discrete integers read off the field: knot counts and the stagnant-branch census
        "knots" | "stag" => "discrete",
        // closed forms and finite loops over them; no solver, no composition, no integrator
        "alg" | "ceil" => "cross_plane_algebra",
        "dp" => "design_point",
        // the resolved fields and everything read straight off them
        "g22" | "g23" | "g24" | "loc" | "fine" | "F" | "grad" => "field",
        "tau23" | "tau24" | "taumean" => "spectrum",
        _ => "kinetic", // zn — the public wiring, through the real chemistry
    }
}

/// Classes compared by ABSOLUTE difference rather than relative — see [`quant_of`].
fn is_absolute(quant: &str) -> bool {
    quant == "residual"
}

/// The bar for each class — CPYTHON arm only; the PyPy arm is held to BIT-EQUALITY.
///
/// Every number is a MEASUREMENT of the CPython↔PyPy spread on this dump, with headroom — not a
/// guess. Slice B set one class by analogy instead of by measurement and it failed inside the
/// hour, so the measured table is printed by the CPython arm on every run.
///
/// `discrete` and `shape_location` get EXACTLY zero, and that is the point of dumping them: they
/// are small integers — a knot count, a cell census, a coarse-grid index — so ANY movement is a
/// real relocation rather than last-bit noise.
/// ```text
///   cross_plane_algebra  0.00e0    <- EXACTLY equal, all 30 keys (alg/ceil)
///   shape_location       0.00e0    <- all 10 indices identical (the VALUES at them are not)
///   discrete             0.00e0    <- all 28 knot counts and stagnant censuses identical
///   residual (ABS)       2.19e-16  <- absolute; a relative bar reads 1.60 here and means nothing
///   dp                   1.36e-15
///   grad                 2.07e-15
///   zn                   4.39e-15
///   g22 / g23            1.34e-15
///   g24                  1.11e-14
///   loc                  3.97e-13
///   taumean              4.12e-13
///   tau24                1.04e-12  <- the worst anywhere on the dump
/// ```
fn bar_for(quant: &str) -> f64 {
    match quant {
        "shape_location" | "discrete" => 0.0,
        "cross_plane_algebra" => 1.0e-15,
        "residual" => 1.0e-14,       // ABSOLUTE; measured worst 2.19e-16
        "field" | "spectrum" => 1.0e-11, // measured worst 3.97e-13 / 1.04e-12
        _ => 1.0e-12,                // kinetic 4.39e-15, design_point 1.36e-15
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
        let rel = if is_absolute(q) {
            (got - want).abs()
        } else if scale > 0.0 {
            (got - want).abs() / scale
        } else {
            (got - want).abs()
        };
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
    println!("\n{:<22} {:>6} {:>11} {:>12} {:>12}", "quantity", "keys", "bit-exact", "worst rel", "bar");
    println!("{}", "-".repeat(68));
    for (q, (n, exact, worst, _)) in &rows {
        println!("{:<22} {:>6} {:>11} {:>12.2e} {:>12.0e}", q, n, exact, worst, bar_for(q));
    }
    let total: usize = rows.iter().map(|r| r.1 .0).sum();
    let exact: usize = rows.iter().map(|r| r.1 .1).sum();
    println!(
        "\n{exact} / {total} bit-identical to {label} ({:.2}%)",
        100.0 * exact as f64 / total as f64
    );
    for (q, (_, _, worst, key)) in &rows {
        if *worst > 0.0 {
            println!("  worst {q:<22} {worst:.2e}  at {key}");
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
            "phase 3D measured {total}/{total} BIT-IDENTICAL to {label}; this run got {exact}. \
             A drop is either a real arithmetic regression or a toolchain/libm change — find out \
             WHICH before loosening this to a tolerance. First drifted keys: {drifted:?}"
        );
    }
}

#[test]
fn spatial_matches_the_cpython_oracle() {
    compare_against(ORACLE_CPYTHON, "CPython 3.14.3", false);
}

/// The same comparison against the interpreter the gate actually runs on — and here the bar is
/// BIT-EQUALITY, not a tolerance.
///
/// Not redundant with the CPython arm; it is the DISCRIMINATOR. Either Rust has its own drift
/// that coincidentally matches PyPy's, or Rust and PyPy compute the same function.
#[test]
fn spatial_matches_the_pypy_oracle_to_the_bit() {
    compare_against(ORACLE_PYPY, "PyPy 3.11.15", true);
}

/// THE SUMMATION-SHAPE ASYMMETRY, asserted from BOTH sides — the slice's headline.
///
/// This is the half no value key can carry, because it is a statement about which of two reduces
/// is EXACT. Rung 23 reaches rung 22's terminal field through a time development whose `frac=1`
/// scalings are exactly 1, and accumulates FLAT — so its `g` is bit-equal. Rung 24 accumulates
/// its mean HIERARCHICALLY while its mean-square is FLAT — so its `g` is not, by ~1e-17.
///
/// A port that "tidied" the hierarchical sum into a flat one would be MORE accurate than the
/// source and would fail the second half of this test. That is deliberate: the port's contract is
/// to reproduce the Python, and § 4.2 of the plan is the record of a real transcription defect
/// that only bit-equality caught.
#[test]
fn the_two_reduces_differ_in_KIND_not_just_in_size() {
    let far = 0.0295;
    let (mut exact23, mut inexact24, mut n) = (0, 0, 0);
    let mut worst_abs = 0.0f64;
    // WIDER than the grid this test was first written on. The first version swept
    // 3 J × 3 grids and found rung 24 inexact 9/9, which made "never bit-equal" look like a law.
    // The oracle's wider sweep (adding ny=16 and J ∈ {1, 400}) found TWO points where the
    // hierarchical and flat sums round to the SAME double — so the honest claim is a MAJORITY,
    // not a universal, and the gate says which. Fifth consecutive slice where sweeping past the
    // first gate written changed what could be claimed.
    for j in [1.0f64, 4.0, 16.0, 100.0, 400.0] {
        for ny in [16usize, 32, 40, 48] {
            let g22 = spatial_segregation(far, PHI_P, S0, H0, j, KP, KY, KZ, ny, ny);
            let (g23, _) =
                spatial_dwell_field(far, PHI_P, S0, H0, j, 1e-3, KP, KY, KZ, ny, ny, 24);
            let (g24, _, _) = spatial_local_field(far, PHI_P, S0, H0, j, 1e-3, KP, KY, KZ, ny, ny);
            n += 1;
            if g23.to_bits() == g22.to_bits() {
                exact23 += 1;
            }
            if g24.to_bits() != g22.to_bits() {
                inexact24 += 1;
            }
            worst_abs = worst_abs.max((g24 - g22).abs());
        }
    }
    // Rung 23 is EXACT — universally, and that is the tighter half of the finding. Its terminal
    // field goes through `_plume(1.0)`, whose `1.0^(1/3)` and `sqrt(1.0)` are exactly 1, and
    // every accumulator on the way is FLAT like rung 22's.
    assert_eq!(
        exact23, n,
        "rung 23's terminal field must reproduce rung 22's BIT-EXACTLY at a matched grid — this \
         is TIGHTER than the Python's own `< 1e-9` bar and it held at every point measured. \
         {exact23}/{n} matched."
    );
    // Rung 24 is inexact at MOST points, never at all of them — the two exceptions on the
    // oracle's grid are the coarsest field at the largest J, where the two summation orders
    // happen to round together.
    assert!(
        inexact24 * 2 > n && inexact24 < n,
        "rung 24's g must differ from rung 22's at MOST points but not ALL: production takes \
         `sum(sum(r) for r in xi)` for the mean and a FLAT sum for the mean-square, and that \
         asymmetry usually — not always — changes the last bits. Got {inexact24}/{n}. If this is \
         now 0, someone flattened the hierarchical sum, which is MORE accurate than the source \
         and is therefore a PORT DEFECT."
    );
    // ...and when it differs it differs only by rounding. Bar is ABSOLUTE and measured (1.7e-16
    // worst on the oracle dump); a relative bar against `g` reads 3.8e-15 and against the
    // residual itself reads 1.6, which is why the oracle compares this class absolutely.
    assert!(
        worst_abs < 1.0e-14,
        "rung-24 g drifted {worst_abs:.2e} in ABSOLUTE terms — that is a DEFECT, not the \
         hierarchical-sum rounding (measured worst 1.7e-16)"
    );
}

/// `τ_mix` cancels out of `F` ALGEBRAICALLY but NOT BIT-EXACTLY — and both of rung 24's "exact"
/// claims fail for the SAME reason.
///
/// The docstring says "`τ_mix` CANCELS out of `u`, so `⟨τ⟩(J) = τ_mix(J)·F(C)` EXACTLY". The
/// algebra is right; the arithmetic is not, because production forms `tsum = Σ(τ_mix·X_i)` and
/// then divides the mean by `τ_mix`, rather than accumulating `ΣX_i` and scaling once. Multiply
/// inside the sum, divide outside, and the scale does not come back out bit-for-bit.
///
/// That is the SAME shape as the `g` reduce this slice measured: rung 24 applies an operation
/// INSIDE an accumulation and removes it OUTSIDE, and in both cases the source's docstring says
/// "exactly" where the floating point says "to rounding". Measured across three decades of
/// `τ_mix`: `F` moves by ≤2.4e-14 relative. So the gate asserts the cancellation to a MEASURED
/// bar and asserts that it is NOT bit-exact — because if it ever became bit-exact, the port
/// would have hoisted the scale out of the loop and stopped reproducing the source.
#[test]
fn tau_mix_cancels_out_of_F_algebraically_but_not_to_the_bit() {
    let far = 0.0295;
    let mut any_inexact = false;
    for j in [4.0f64, 16.0, 64.0] {
        let (_, _, fa) = spatial_local_field(far, PHI_P, S0, H0, j, 1.0e-4, KP, KY, KZ, 32, 32);
        let (_, _, fb) = spatial_local_field(far, PHI_P, S0, H0, j, 1.0e-1, KP, KY, KZ, 32, 32);
        let rel = (fb - fa).abs() / fa.abs();
        assert!(
            rel < 1.0e-13,
            "F moved {rel:.2e} across three decades of τ_mix at J={j} — the cancellation is \
             algebraically exact, so anything above rounding means τ_mix leaked into the shape"
        );
        if fa.to_bits() != fb.to_bits() {
            any_inexact = true;
        }
    }
    assert!(
        any_inexact,
        "F was BIT-identical across three decades of τ_mix. That is more exact than the Python, \
         which forms Σ(τ_mix·X) and divides by τ_mix afterwards — so this passing means the port \
         hoisted the scale out of the accumulation and is no longer reproducing the source."
    );
}

/// Rung 24's `u < 1e-8` stagnant branch is HEAVILY TAKEN, not dormant.
///
/// The inverse of rung 20's flame-band floor, which never binds at the shipped design point and
/// needed a second, cooler point before its gate meant anything. Here the branch is taken by a
/// large minority of cells at every J, because the β-clip creates exactly-flat plateaus where
/// `|∇ξ|²` is precisely zero. Asserted as a BAND so the gate fails if the branch ever goes
/// dormant (which would silently make `F` a different functional) or ever swallows the field.
#[test]
fn the_stagnant_branch_is_live_at_every_J() {
    let far = 0.0295;
    let counts: Vec<usize> = J_COARSE
        .iter()
        .map(|&j| spatial_local_stagnant_cells(far, PHI_P, S0, H0, j, KP, KY, KZ, 32, 32))
        .collect();
    for (&j, &c) in J_COARSE.iter().zip(counts.iter()) {
        let frac = c as f64 / 1024.0;
        assert!(
            (0.10..0.60).contains(&frac),
            "the stagnant branch took {:.1}% of cells at J={j} — outside the measured \
             18–50 % band, so either it went dormant (a gate on a dead branch) or it swallowed \
             the field",
            100.0 * frac
        );
    }
    // ...and its census is U-shaped with the minimum at C_opt (J=16), which CORROBORATES F's U.
    // NOT a second kill test: `u` carries the same explicit 1/var that makes argmin-F circular.
    let cf: Vec<f64> = counts.iter().map(|&c| c as f64).collect();
    assert_eq!(
        argmin(&cf),
        2,
        "the stagnant census should bottom at J=16 (C_opt), got J={}",
        J_COARSE[argmin(&cf)]
    );
}
