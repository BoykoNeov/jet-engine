//! Rung-24 verification: the LOCALLY-RESOLVED MIXING TIME — and the answer is a SPLIT.
//!
//! Rungs 11–23 all ran on ONE GLOBAL `τ_mix`. Rung 23 § 9 named this successor by hand and
//! hypothesised that a per-cell rate "could restore an off-optimum dwell GROWTH that pins the
//! emissions optimum non-circularly". Rung 24 ASKS that question. Each cell relaxes at its own
//! gradient-derived `ω = D_t|∇ξ|²/var` — rung-18's own form made local in the numerator, with
//! `D_t` REUSED, so no new constant and no new knob.
//!
//! * **THE POSITIVE.** `τ_mix` cancels out of `u`, so `⟨τ⟩(J) = τ_mix(J)·F(C)` and `F(C)` is a
//!   PURE field functional — U-shaped with its minimum AT `C_opt`. That is the off-optimum dwell
//!   growth rung 16 IMPOSED, here DERIVED from the plume's own gradients.
//! * **THE NEGATIVE, AND THE HEADLINE.** F's U is worth ~39 % while `τ_mix` swings ~20× over the
//!   same range, so `⟨EI⟩(J)` stays MONOTONE and the emissions `C_opt` pin is STILL not
//!   recovered. **THE SCALE SWAMPS THE SHAPE** — rung 24 localises the RATE, not the SCALE.
//!
//! **THE SPLIT IS GATED AS A SPLIT.** Asserting only the U, or only the monotone `⟨EI⟩`, passes
//! while measuring half the rung — and a wiring bug that fed one quantity into both slots would
//! satisfy either alone. So both halves are read off the SAME sweep and the two argmins are
//! asserted to land at DIFFERENT `J`: `F` interior, `⟨EI⟩` at an end.
//!
//! **TWO "EXACT" CLAIMS THAT ARE EXACT ONLY IN ALGEBRA.** The `g` reduce and the `τ_mix`
//! cancellation are both stated as exact by the Python and are both true to ROUNDING only,
//! because production applies an operation INSIDE an accumulation and removes it OUTSIDE — a
//! hierarchical mean where rung 22 runs a flat one, and `Σ(τ_mix·X)` divided by `τ_mix`. Gated
//! from both sides, so a port that "tidies" either into the exact form FAILS.
//! `docs/plans/todo-rust-port.md` § 4.7.

use std::panic::catch_unwind;
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{self, Gas};
use turbojet::nox::{
    spatial_local_field, spatial_local_stagnant_cells, spatial_segregation, JetMixing,
    SpatialDwellPdf, SpatialLocalPdf, ZonedNoxOpts,
};

const TAU: f64 = 3e-3;
const PHI_P: f64 = 1.5;
const CE: f64 = 0.20;
const NB: usize = 20;
const NQ: usize = 64;
const NG: usize = 9;
const NSTEPS: usize = 100;
const NY: usize = 20;
const S0: f64 = 0.0625;
const H0: f64 = 0.10;
/// The coarse house grid — `C_opt ≈ 2.5` sits exactly on J=16 and the neighbours are 1.8–2.25×
/// away, so no last-bit difference can relocate an argmin read off it.
const J_COARSE: &[f64] = &[4.0, 9.0, 16.0, 36.0, 64.0];
const TAU_MIX_REF: f64 = 1.0e-3;

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
fn cfg() -> SpatialLocalPdf {
    SpatialLocalPdf {
        s: S0, ny: NY, nz: NY, n_bell: NB, n_quad: NQ, ..SpatialLocalPdf::default()
    }
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
fn argmax(v: &[f64]) -> usize {
    let mut b = 0;
    for (i, &x) in v.iter().enumerate() {
        if x > v[b] {
            b = i;
        }
    }
    b
}

/// `F(C)` on a given geometry, at a fixed grid.
fn f_at(far: f64, s_sp: f64, j: f64, ny: usize) -> f64 {
    spatial_local_field(far, PHI_P, s_sp, H0, j, TAU_MIX_REF, 0.316, 0.28, 0.28, ny, ny).2
}

/// `⟨|∇ξ|²⟩` of the terminal field, WITHOUT any variance normalisation — the G-FREE WITNESS.
///
/// Deliberately transcribed here rather than exposed from production. `ω` carries an explicit
/// `1/var` and rung 22 ALREADY mins `g` at `C_opt`, so "argmin F == argmin g" is a TELL, not a
/// confirmation. The witness only breaks that circularity if it is built independently — reading
/// it off a production accessor would compare production to itself, which is exactly the vacuity
/// trap slices C and D each hit once.
fn mean_grad_sq(far: f64, s_sp: f64, j: f64, ny: usize) -> f64 {
    let nz = ny;
    let xibar = far / (1.0 + far);
    let far_p = PHI_P * gas::f_stoich();
    let xi_p = far_p / (1.0 + far_p);
    let delta = 0.316 * (s_sp * H0).sqrt() * gas::powp(j, 0.25);
    let (sig_y, sig_z) = (0.28 * H0, 0.28 * s_sp);
    let ys: Vec<f64> = (0..ny).map(|i| (i as f64 + 0.5) * H0 / ny as f64).collect();
    let zs: Vec<f64> = (0..nz).map(|k| (k as f64 + 0.5) * s_sp / nz as f64).collect();
    let ay: Vec<f64> = ys
        .iter()
        .map(|&y| {
            [-delta, delta, 2.0 * H0 - delta, 2.0 * H0 + delta]
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
    let (dy, dz) = (H0 / ny as f64, s_sp / nz as f64);
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
    fn run(&self, j: f64, c: SpatialLocalPdf) -> turbojet::nox::ZonedNoxState {
        self.g.zoned_nox(
            self.far, self.tt3, self.tt4, self.p, PHI_P,
            ZonedNoxOpts { mixing: Some(mix(j)), spatial_local: Some(c), ..opts() },
        )
    }
}

// ------------------------------------------------------------------------------------------
// GATE 1 — reduce, and the "identical by construction" claim CORRECTED.
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
        ZonedNoxOpts { mixing: Some(mix(16.0)), spatial_local: None, ..opts() },
    );
    assert_eq!(base.ei_no_quenched.unwrap().to_bits(), none.ei_no_quenched.unwrap().to_bits());
    assert!(none.f_shape.is_none() && none.ei_no_spatial_local.is_none());
}

#[test]
fn the_width_equals_rung_22s_to_ROUNDING_and_not_to_the_bit() {
    // THE SOURCE'S CLAIM, CORRECTED BY MEASUREMENT. The Python says rung 24's `g` is "IDENTICAL
    // BY CONSTRUCTION — not to a tolerance (contrast rung-23's, which matches to <1%)". Measured,
    // it is the OTHER WAY ROUND: rung 23 is exact and rung 24 is not, and the entire ~1e-17 is
    // one line — production takes `sum(sum(r) for r in xi)` for the mean while its mean-square
    // two lines later is FLAT. Both halves are asserted, so a port that flattens the sum (which
    // would be MORE accurate than the source) fails here.
    let dp = design_point();
    let mut differed = 0;
    let mut total = 0;
    for j in [4.0f64, 16.0, 100.0] {
        for ny in [16usize, 20, 32] {
            let g24 = spatial_local_field(
                dp.far, PHI_P, S0, H0, j, TAU_MIX_REF, 0.316, 0.28, 0.28, ny, ny,
            )
            .0;
            let g22 = spatial_segregation(dp.far, PHI_P, S0, H0, j, 0.316, 0.28, 0.28, ny, ny);
            total += 1;
            if g24.to_bits() != g22.to_bits() {
                differed += 1;
            }
            let rel = (g24 - g22).abs() / g22;
            assert!(
                rel < 1e-13,
                "rung-24 g must equal rung-22's to ROUNDING: {rel:.2e} at J={j} ny={ny}"
            );
        }
    }
    assert!(
        differed > 0,
        "rung-24 g was bit-equal to rung-22's at ALL {total} points. That is more exact than the \
         Python, whose mean is hierarchical and whose mean-square is flat — so this passing means \
         the port flattened the sum and is no longer reproducing the source."
    );
}

// ------------------------------------------------------------------------------------------
// GATE 2 — THE FACTORISATION: τ_mix cancels (algebraically), so F is a PURE field functional.
// ------------------------------------------------------------------------------------------

#[test]
fn f_is_independent_of_tau_mix_to_rounding_but_not_to_the_bit() {
    // `u = σ²|∇ξ|²/(2var)` carries no `τ_mix`, so `F` cannot depend on it — ALGEBRAICALLY.
    // Arithmetically production forms `Σ(τ_mix·X_i)` and divides the mean by `τ_mix` afterwards
    // rather than scaling once, so the scale does not come back out bit-for-bit. Same shape as
    // GATE 1's finding: an operation applied inside an accumulation and removed outside.
    let dp = design_point();
    let mut any_inexact = false;
    for j in [4.0f64, 16.0, 64.0] {
        let fa =
            spatial_local_field(dp.far, PHI_P, S0, H0, j, 1e-4, 0.316, 0.28, 0.28, NY, NY).2;
        let fb =
            spatial_local_field(dp.far, PHI_P, S0, H0, j, 1e-1, 0.316, 0.28, 0.28, NY, NY).2;
        let rel = (fb - fa).abs() / fa;
        assert!(rel < 1e-13, "F moved {rel:.2e} across three decades of τ_mix at J={j}");
        if fa.to_bits() != fb.to_bits() {
            any_inexact = true;
        }
    }
    assert!(
        any_inexact,
        "F was BIT-identical across three decades of τ_mix — more exact than the Python, so the \
         port hoisted the scale out of the accumulation"
    );
}

#[test]
fn the_dwell_scales_linearly_in_tau_mix() {
    // The other half of `⟨τ⟩ = τ_mix·F`: with `F` fixed, the MEAN dwell must track the scale.
    // Production reports `⟨τ⟩` over the β-PDF, so this reads the two through the public wiring.
    let dp = design_point();
    let a = dp.g.zoned_nox(
        dp.far, dp.tt3, dp.tt4, dp.p, PHI_P,
        ZonedNoxOpts {
            mixing: Some(JetMixing { j: 16.0, h: H0, c_e: 0.20, shape_n: 2.0, ..Default::default() }),
            spatial_local: Some(cfg()),
            ..opts()
        },
    );
    let b = dp.g.zoned_nox(
        dp.far, dp.tt3, dp.tt4, dp.p, PHI_P,
        ZonedNoxOpts {
            // C_e halved ⇒ τ_mix DOUBLED (τ_q = H/(C_e√J·U_c)), geometry untouched
            mixing: Some(JetMixing { j: 16.0, h: H0, c_e: 0.10, shape_n: 2.0, ..Default::default() }),
            spatial_local: Some(cfg()),
            ..opts()
        },
    );
    assert!(
        (a.f_shape.unwrap() - b.f_shape.unwrap()).abs() / a.f_shape.unwrap() < 1e-13,
        "F is the SHAPE and must not move with the scale"
    );
    let ratio = b.tau_mean_local.unwrap() / a.tau_mean_local.unwrap();
    assert!(
        (ratio - 2.0).abs() < 1e-9,
        "⟨τ⟩ must scale LINEARLY in τ_mix: got ×{ratio:.6} for a doubled τ_mix"
    );
}

// ------------------------------------------------------------------------------------------
// GATE 3 — THE POSITIVE: F(C) is U-shaped with its minimum AT C_opt, and it is NOT the 1/g factor.
// ------------------------------------------------------------------------------------------

#[test]
fn f_is_u_shaped_with_its_minimum_at_c_opt() {
    let dp = design_point();
    let fs: Vec<f64> = J_COARSE.iter().map(|&j| f_at(dp.far, S0, j, 32)).collect();
    let i = argmin(&fs);
    assert_eq!(
        J_COARSE[i], 16.0,
        "argmin F must sit at J=16 (C_opt): got J={} ({fs:?})",
        J_COARSE[i]
    );
    // BOTH flanks rise — a U, not a monotone trend. This is the whole disagreement with rung 23,
    // whose one shared schedule gives F ≈ const.
    assert!(fs[0] > fs[2] && fs[1] > fs[2], "the under-penetration flank must rise: {fs:?}");
    assert!(fs[3] > fs[2] && fs[4] > fs[2], "the over-penetration flank must rise: {fs:?}");
    assert!((cfg().c_opt() - 2.5).abs() < 0.02, "and C_opt is DERIVED, not a knob");
}

#[test]
fn the_gradients_locate_c_opt_the_g_free_witness() {
    // THE CIRCULARITY KILL TEST. `ω` carries an explicit `1/g` and rung 22 already mins `g` at
    // `C_opt`, so "argmin F == argmin g" is a TELL. `⟨|∇ξ|²⟩` carries NO `g` algebraically, and
    // it is MAXIMAL at `C_opt` — the gradients place the optimum, and the `1/g` coupling only
    // amplifies it.
    //
    // Physically: at `C_opt` the jet fills to mid-height so the residual structure sits at the
    // plume's OWN scale σ — fine, therefore steep, therefore fast, therefore short-dwell; off
    // optimum the air piles into WALL-SCALE slabs — coarse, shallow, slow, long-dwell. That
    // fine-vs-coarse behaviour is a property of the FIXED-σ Gaussian-plume CARTOON, not a general
    // turbulent-mixing law.
    // A COARSER GRID THAN THE REST OF THIS FILE — 4× apart, not 1.8×, and that is measured, not
    // stylistic. On the five-point house grid the gradient peak clears its LOW neighbour by only
    // 1.4 % (4.1138 at J=16 against 4.0582 at J=9), which is the thinnest margin any location key
    // in this slice carries and far too thin to call a location. Widening to {4, 16, 64} lifts the
    // clearances to 19.5 % and 47.8 %. Slice C's rule, applied a second time: when a location key
    // sits close to its neighbours the fix is a coarser grid, never a looser bar.
    let dp = design_point();
    let grid = [4.0f64, 16.0, 64.0];
    let grads: Vec<f64> = grid.iter().map(|&j| mean_grad_sq(dp.far, S0, j, 32)).collect();
    let i = argmax(&grads);
    assert_eq!(
        grid[i], 16.0,
        "⟨|∇ξ|²⟩ is maximal at J={}, not at C_opt (J=16) — the kill test FAILS and F's minimum \
         would be the 1/g factor's doing, not the gradients'. ({grads:?})",
        grid[i]
    );
    assert!(
        grads[i] / grads[i - 1] > 1.15 && grads[i] / grads[i + 1] > 1.15,
        "the g-free peak must clear both neighbours to be a LOCATION and not a coin flip: {grads:?}"
    );
}

#[test]
fn the_f_optimum_shifts_as_h_over_s_squared() {
    // Rung-22's SIGNATURE, inherited by the DWELL: the optimum is a function of the Holdeman
    // group ALONE, so HALVING the spacing must QUADRUPLE J_opt with C_opt fixed.
    let dp = design_point();
    for (s_sp, want, grid) in [
        (S0 / 2.0, 64.0, vec![16.0f64, 36.0, 64.0, 144.0, 256.0]),
        (S0, 16.0, J_COARSE.to_vec()),
    ] {
        let fs: Vec<f64> = grid.iter().map(|&j| f_at(dp.far, s_sp, j, 32)).collect();
        let got = grid[argmin(&fs)];
        assert_eq!(got, want, "S={s_sp}: argmin F at J={got}, expected {want}");
        assert!(((s_sp / H0) * got.sqrt() - 2.5).abs() < 0.02, "...and it IS C=2.5 at both");
    }
}

#[test]
fn the_stagnant_branch_is_live_and_bottoms_at_c_opt() {
    // Rung 24's `u < 1e-8` analytic limit. NOT dormant — 18–50 % of cells take it, because the
    // β-clip creates exactly-flat plateaus where `|∇ξ|²` is precisely zero. The inverse of rung
    // 20's flame-band floor, which never binds at the shipped point and needed a second, cooler
    // one before its gate meant anything.
    let dp = design_point();
    let counts: Vec<f64> = J_COARSE
        .iter()
        .map(|&j| {
            spatial_local_stagnant_cells(dp.far, PHI_P, S0, H0, j, 0.316, 0.28, 0.28, 32, 32) as f64
        })
        .collect();
    for (&j, &c) in J_COARSE.iter().zip(counts.iter()) {
        let frac = c / 1024.0;
        assert!(
            (0.10..0.60).contains(&frac),
            "the stagnant branch took {:.1}% of cells at J={j} — outside the measured band, so \
             either it went dormant or it swallowed the field",
            100.0 * frac
        );
    }
    // Its census bottoms at C_opt too — CORROBORATION of F's U, and explicitly NOT a second kill
    // test: `u` carries the same explicit 1/var that makes argmin-F circular in the first place.
    assert_eq!(J_COARSE[argmin(&counts)], 16.0, "the stagnant census should bottom at C_opt");
}

// ------------------------------------------------------------------------------------------
// GATE 4 — THE NEGATIVE HEADLINE, and the SPLIT asserted AS a split.
// ------------------------------------------------------------------------------------------

#[test]
fn the_split_F_turns_but_the_emissions_do_not() {
    // BOTH HALVES OFF ONE SWEEP. Asserting only the U, or only the monotone ⟨EI⟩, passes while
    // measuring half the rung — and a wiring bug that fed one quantity into both slots would
    // satisfy either alone. The load-bearing extra assertion is that the two argmins land at
    // DIFFERENT J: F interior at C_opt, ⟨EI⟩ at an END of the same grid.
    let dp = design_point();
    // `n_quad = 160`, not this file's 64 — slice C's measured floor on the quadrature's own
    // mean-preservation guard, which binds at the wide end of this sweep exactly as it does in
    // rung 22's emissions gate. Running at 64 panics inside the guard, which is the guard working.
    let wide = SpatialLocalPdf { n_quad: 160, ..cfg() };
    let mut fs = Vec::new();
    let mut eis = Vec::new();
    for &j in J_COARSE {
        let s = dp.run(j, wide);
        fs.push(s.f_shape.unwrap());
        eis.push(s.ei_no_spatial_local.unwrap());
    }
    let i_f = argmin(&fs);
    let i_ei = argmin(&eis);
    assert_eq!(J_COARSE[i_f], 16.0, "F must min AT C_opt: {fs:?}");
    assert!(
        i_ei == 0 || i_ei == J_COARSE.len() - 1,
        "⟨EI⟩ must stay MONOTONE over the same sweep, so its min sits at an END: {eis:?}"
    );
    assert_ne!(
        i_f, i_ei,
        "THE SPLIT IS THE RUNG: the shape's optimum and the emissions' optimum must be at \
         DIFFERENT J. If they coincide, either the emissions pin was recovered (a new result) or \
         one quantity is wired into both slots (a bug) — find out which."
    );
    // ⟨EI⟩ monotone DECREASING across the whole grid, not merely min-at-an-end
    for w in eis.windows(2) {
        assert!(w[1] < w[0], "⟨EI⟩(J) must fall monotonically: {eis:?}");
    }
}

#[test]
fn the_scale_swamps_the_shape_quantified() {
    // The headline as a NUMBER: F's U is worth tens of percent while τ_mix swings by ~20× over
    // the same range, which is WHY the product still falls. Both are read off the same sweep so
    // the comparison cannot drift apart.
    let dp = design_point();
    let js = [1.0f64, 4.0, 16.0, 64.0, 400.0];
    let fs: Vec<f64> = js.iter().map(|&j| f_at(dp.far, S0, j, 32)).collect();
    let taus: Vec<f64> = js.iter().map(|&j| mix(j).tau_q()).collect();
    let f_swing = fs.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
        / fs.iter().cloned().fold(f64::INFINITY, f64::min);
    let t_swing = taus.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
        / taus.iter().cloned().fold(f64::INFINITY, f64::min);
    assert!(f_swing < 2.0, "F's U should be a tens-of-percent effect, got ×{f_swing:.2}");
    assert!(t_swing > 10.0, "τ_mix should swing an order of magnitude, got ×{t_swing:.2}");
    assert!(
        t_swing > 5.0 * f_swing,
        "THE SCALE SWAMPS THE SHAPE: τ_mix ×{t_swing:.1} against F ×{f_swing:.2}"
    );
}

// ------------------------------------------------------------------------------------------
// GATE 5 — what rung 24 DELIBERATELY does not claim, and the guards.
// ------------------------------------------------------------------------------------------

#[test]
fn the_correlation_still_adds_no_through_the_local_spectrum() {
    // Rung 23's instrument, on rung 24's spectrum: the matched-mean twin isolates the ξ–τ
    // correlation and the sign survives the change of dwell source.
    let dp = design_point();
    let s = dp.run(16.0, cfg());
    let r = s.corr_ratio_local.unwrap();
    assert!(r > 1.0, "corr_ratio_local={r} must exceed 1");
    assert!((r - 1.0).abs() > 1e-6, "corr_ratio_local is indistinguishable from 1 — dead twin");
    assert!(
        s.ei_no_spatial_local.unwrap() > s.ei_no_spatial_local_meanfield.unwrap(),
        "the correlated run must exceed its matched-mean twin"
    );
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

#[test]
fn spatial_local_requires_a_mixing_config() {
    let e = catch_unwind(|| {
        let (g, tt3, tt4, far, p) = design_state();
        g.zoned_nox(
            far, tt3, tt4, p, PHI_P,
            ZonedNoxOpts { spatial_local: Some(cfg()), tau_q: Some(1e-3), ..opts() },
        )
    })
    .expect_err("spatial_local without mixing must panic");
    let msg = e
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_string()))
        .unwrap_or_default();
    assert!(msg.contains("REQUIRES a `mixing` config"), "wrong panic: {msg}");
}

/// The ≤1-of-EIGHT guard — which the PYTHON'S rung-24 suite does not have.
///
/// Rungs 22 and 23 each carry a `test_at_most_one_closure`; rung 24's file has none. Gating it
/// here is slice D's instance of sweeping past the source's own gate, which has now paid in four
/// consecutive slices.
#[test]
fn at_most_one_closure_of_the_eight() {
    let e = catch_unwind(|| {
        let (g, tt3, tt4, far, p) = design_state();
        g.zoned_nox(
            far, tt3, tt4, p, PHI_P,
            ZonedNoxOpts {
                mixing: Some(mix(16.0)),
                spatial_local: Some(cfg()),
                spatial_dwell: Some(SpatialDwellPdf {
                    ny: NY, nz: NY, ..SpatialDwellPdf::default()
                }),
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
fn the_config_rejects_bad_input_and_has_no_nt() {
    for bad in [
        SpatialLocalPdf { s: -1.0, ..cfg() },
        SpatialLocalPdf { k_p: 0.0, ..cfg() },
        SpatialLocalPdf { nz: 1, ..cfg() },
    ] {
        assert!(catch_unwind(move || bad.validate()).is_err(), "{bad:?} must be rejected");
    }
    let e = catch_unwind(|| {
        spatial_local_field(0.0295, PHI_P, S0, H0, 16.0, 0.0, 0.316, 0.28, 0.28, 8, 8)
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
