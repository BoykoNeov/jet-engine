//! Rung-22 verification: the RESOLVED cross-plane — the INVERSION of rung 18.
//!
//! Rung 18's load-bearing NEGATIVE was that a 0-D variance transport CANNOT DERIVE the Holdeman
//! optimum: with any mean-field `ω(J)` the residual `g(J)` is monotone, so the optimum had to be
//! IMPOSED through a coverage `ω(C)` — the spatial spacing `S` injected by hand. Rung 22 resolves
//! the y-z dilution cross-plane instead, and `C_opt` EMERGES as an OUTPUT.
//!
//! The delta over rung 18 is MINIMAL and that is the point: both feed the identical rung-13 ideal
//! bell, and only the SOURCE of `g` changes — imposed ODE → derived cross-plane.
//!
//! **THE SIGNATURE OF THE INVERSION is a MISSING FIELD.** Every rung-12..18 config takes
//! `C_opt = 2.5` as an input; [`SpatialPdf`] has no such field, and `c_opt()` is derived from the
//! penetration constant alone. The Python guards this with a test asserting that
//! `SpatialPDF(C_opt=2.5)` raises `TypeError`. **That test is deliberately NOT ported**: in Rust
//! an unknown struct field is a COMPILE error and the crate has no dependencies by decision, so
//! there is no `trybuild` and a runtime transcription would measure literally nothing. What is
//! gated instead is the DERIVATION — `c_opt()` tracking `1/(4k_p²)` across several `k_p`, and the
//! argmin FOLLOWING it. `docs/plans/todo-rust-port.md` § 4.7 records this as one of three cases
//! where the source's test guards what the target's type system already guarantees.
//!
//! Gates, priority order:
//!
//! 1. **reduce** — no `spatial` leaves the prior path untouched, and the cycle is untouched.
//! 2. **THE COLLAPSE** (headline) — `g`'s minimum VALUE is geometry-independent; only `J_opt`
//!    moves, exactly as `(H/S)²`.
//! 3. **`C_opt` IS AN OUTPUT** — the argmin lands on the closed form, and TRACKS it as `k_p` moves.
//! 4. **`g < g_ceiling` always** — the tie back to rung-18's DERIVED two-stream ceiling.
//! 5. **emissions**: the `C_opt` min is only LOCAL, and the honest global min is at max segregation.
//! 6. guards — mixing required, ≤1-of-EIGHT, the RQL geometry, positivity.

use std::panic::catch_unwind;
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::Gas;
use turbojet::nox::{
    spatial_segregation, two_stream_ceiling, JetMixing, MixingPdf, SpatialPdf, ZonedNoxOpts,
};

const TAU: f64 = 3e-3;
const PHI_P: f64 = 1.5;
const CE: f64 = 0.20;
const NB: usize = 32;
const NQ: usize = 64;
const NG: usize = 17;
const NSTEPS: usize = 200;
const NY: usize = 32;
const S0: f64 = 0.0625;
const H0: f64 = 0.10;
/// The COARSE location grid. `C_opt ≈ 2.5` lands exactly on the J=16 node and the neighbours sit
/// 1.8–2.25× away, so a last-bit difference cannot relocate the argmin. The Python's own helper
/// sweeps 49–81 log-spaced points over J ∈ [1,400], which puts neighbours 3.8–6.4 % apart in `C`
/// around a QUADRATIC minimum — the configuration slice C measured as unsafe for a location key.
const J_COARSE: &[f64] = &[4.0, 9.0, 16.0, 36.0, 64.0];

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
fn mix(j: f64, h: f64) -> JetMixing {
    JetMixing { j, h, c_e: CE, shape_n: 2.0, ..JetMixing::default() }
}
fn cfg() -> SpatialPdf {
    SpatialPdf { s: S0, ny: NY, nz: NY, n_bell: NB, n_quad: NQ, ..SpatialPdf::default() }
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

fn design_state() -> (Gas, f64, f64, f64, f64) {
    let g = Gas::reacting_equilibrium();
    let r = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, flight().p0, losses())
        .run(&flight(), 50.0);
    (g, r.station("3").tt, r.station("4").tt, r.station("4").far, r.station("4").pt)
}

struct Dp {
    g: Gas,
    tt3: f64,
    tt4: f64,
    far: f64,
    p: f64,
}

fn design_point() -> Dp {
    let (g, tt3, tt4, far, p) = design_state();
    Dp { g, tt3, tt4, far, p }
}

impl Dp {
    fn run(&self, j: f64, c: SpatialPdf) -> turbojet::nox::ZonedNoxState {
        self.g.zoned_nox(
            self.far, self.tt3, self.tt4, self.p, PHI_P,
            ZonedNoxOpts { mixing: Some(mix(j, H0)), spatial: Some(c), ..opts() },
        )
    }
}

/// `(g_min, J_opt, C_at_opt)` over the coarse grid — the helper-level collapse probe.
fn coarse_argmin(far: f64, s_sp: f64, h: f64, k_p: f64, grid: &[f64]) -> (f64, f64, f64, Vec<f64>) {
    let gs: Vec<f64> = grid
        .iter()
        .map(|&j| spatial_segregation(far, PHI_P, s_sp, h, j, k_p, 0.28, 0.28, NY, NY))
        .collect();
    let i = argmin(&gs);
    (gs[i], grid[i], (s_sp / h) * grid[i].sqrt(), gs)
}

// ------------------------------------------------------------------------------------------
// GATE 1 — reduce.
// ------------------------------------------------------------------------------------------

#[test]
fn reduce_none_leaves_the_prior_path_untouched() {
    let dp = design_point();
    let base = dp.g.zoned_nox(
        dp.far, dp.tt3, dp.tt4, dp.p, PHI_P,
        ZonedNoxOpts { mixing: Some(mix(16.0, H0)), ..opts() },
    );
    let with_none = dp.g.zoned_nox(
        dp.far, dp.tt3, dp.tt4, dp.p, PHI_P,
        ZonedNoxOpts { mixing: Some(mix(16.0, H0)), spatial: None, ..opts() },
    );
    assert_eq!(base.ei_no().to_bits(), with_none.ei_no().to_bits());
    assert_eq!(
        base.ei_no_quenched.unwrap().to_bits(),
        with_none.ei_no_quenched.unwrap().to_bits(),
        "spatial=None must be the EXACT prior path, bit for bit"
    );
    assert!(base.g_spatial.is_none() && base.ei_no_spatial.is_none());
}

#[test]
fn the_cycle_is_untouched_a_pure_diagnostic() {
    // NO and N never enter `_equil_solve`, so the design run stays bit-for-bit rung 6.
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
    assert_eq!(a.performance.tsfc.to_bits(), b.performance.tsfc.to_bits());
    assert_eq!(a.station("4").tt.to_bits(), b.station("4").tt.to_bits());
}

// ------------------------------------------------------------------------------------------
// GATE 2 — THE COLLAPSE (headline): the minimum's DEPTH is geometry-independent, its LOCATION
// moves exactly as (H/S)².
// ------------------------------------------------------------------------------------------

#[test]
fn the_minimum_value_is_geometry_independent() {
    let dp = design_point();
    // Five geometries varying S and H INDEPENDENTLY by 2×. Each gets a grid scaled so its own
    // optimum lands on a node — the collapse is about the DEPTH, and reading the depth off a
    // grid whose optimum falls between nodes would measure the grid instead.
    let cases: [(f64, f64, f64); 5] = [
        (S0, H0, 1.0),
        (S0 / 2.0, H0, 4.0),
        (S0 * 2.0, H0, 0.25),
        (S0, H0 * 2.0, 4.0),
        (S0 * 2.0, H0 * 2.0, 1.0),
    ];
    let mut mins = Vec::new();
    for (s_sp, h, scale) in cases {
        let grid: Vec<f64> = J_COARSE.iter().map(|&j| j * scale).collect();
        let (gmin, _, _, _) = coarse_argmin(dp.far, s_sp, h, 0.316, &grid);
        mins.push(gmin);
    }
    let g0 = mins[0];
    for (k, &gm) in mins.iter().enumerate() {
        let rel = (gm - g0).abs() / g0;
        assert!(
            rel < 0.03,
            "case {k}: g_min must be geometry-independent (THE COLLAPSE) — {gm:.6e} vs {g0:.6e} \
             ({rel:.1e} apart). Only J_opt is allowed to move."
        );
    }
}

#[test]
fn j_opt_shifts_exactly_as_h_over_s_squared() {
    let dp = design_point();
    let quad: Vec<f64> = J_COARSE.iter().map(|&j| j * 4.0).collect();
    let (_, j_base, c_base, _) = coarse_argmin(dp.far, S0, H0, 0.316, J_COARSE);
    let (_, j_half_s, c_half, _) = coarse_argmin(dp.far, S0 / 2.0, H0, 0.316, &quad);
    let (_, j_dbl_h, c_dbl, _) = coarse_argmin(dp.far, S0, H0 * 2.0, 0.316, &quad);
    let (_, j_both, c_both, _) = coarse_argmin(dp.far, S0 * 2.0, H0 * 2.0, 0.316, J_COARSE);
    assert_eq!(j_half_s / j_base, 4.0, "halve S ⇒ J_opt ×4 (an EXACT grid ratio, not a tolerance)");
    assert_eq!(j_dbl_h / j_base, 4.0, "double H ⇒ J_opt ×4");
    assert_eq!(j_both, j_base, "S/H fixed ⇒ J_opt unchanged");
    // ...and all four land on the SAME Holdeman group, which is what "collapse" means.
    for c in [c_half, c_dbl, c_both] {
        assert!((c - c_base).abs() < 1e-12, "C at the optimum moved: {c} vs {c_base}");
    }
}

// ------------------------------------------------------------------------------------------
// GATE 3 — C_opt IS AN OUTPUT.
// ------------------------------------------------------------------------------------------

#[test]
fn c_opt_is_the_derived_closed_form_and_the_argmin_tracks_it() {
    let dp = design_point();
    // THE REPLACEMENT for the Python's `SpatialPDF(C_opt=2.5)` TypeError test, which in Rust
    // would be a compile error and therefore un-portable as a runtime check (see the module
    // docs). This asserts the DERIVATION instead — strictly more than the absence of a field.
    for k_p in [0.25f64, 0.316, 0.40] {
        let c = SpatialPdf { k_p, ..cfg() };
        let closed = 1.0 / (4.0 * k_p * k_p);
        assert_eq!(c.c_opt().to_bits(), closed.to_bits(), "c_opt() must BE 1/(4k_p²)");
    }
    // k_p=0.316 ⇒ C_opt ≈ 2.504, Holdeman's ≈2.5
    assert!((cfg().c_opt() - 2.5).abs() < 0.02);
    // and the measured argmin lands there
    let (_, _, c_at_min, _) = coarse_argmin(dp.far, S0, H0, 0.316, J_COARSE);
    assert!(
        (c_at_min - cfg().c_opt()).abs() / cfg().c_opt() < 0.08,
        "the argmin C={c_at_min:.3} must land at the DERIVED C_opt={:.3}",
        cfg().c_opt()
    );
    // The Holdeman group itself is the config's, not the test's, arithmetic.
    let m = mix(16.0, H0);
    assert_eq!(cfg().c(&m).to_bits(), ((S0 / H0) * 16.0f64.sqrt()).to_bits());
}

#[test]
fn a_larger_k_p_moves_c_opt_down_and_the_argmin_follows() {
    // The DIRECTION is the derivation's content: deeper penetration ⇒ the half-height fill
    // happens at a smaller group. If the argmin did not move with k_p, C_opt would be an input
    // in disguise.
    let dp = design_point();
    let mut c_opts = Vec::new();
    let mut argmins = Vec::new();
    for k_p in [0.25f64, 0.316, 0.40] {
        // each k_p gets a grid centred on ITS OWN predicted optimum, so the grid cannot be what
        // pins the answer
        let j_star = (1.0 / (4.0 * k_p * k_p) * H0 / S0).powi(2);
        let grid: Vec<f64> = [0.25f64, 0.5625, 1.0, 2.25, 4.0].iter().map(|f| f * j_star).collect();
        let (_, _, c_at_min, _) = coarse_argmin(dp.far, S0, H0, k_p, &grid);
        c_opts.push(1.0 / (4.0 * k_p * k_p));
        argmins.push(c_at_min);
    }
    assert!(c_opts[0] > c_opts[1] && c_opts[1] > c_opts[2], "larger k_p ⇒ smaller C_opt");
    for (k, (&pred, &got)) in c_opts.iter().zip(argmins.iter()).enumerate() {
        assert!(
            (got - pred).abs() / pred < 0.12,
            "k_p case {k}: the argmin C={got:.3} must TRACK the derived C_opt={pred:.3}"
        );
    }
}

// ------------------------------------------------------------------------------------------
// GATE 4 — the tie back to rung-18's DERIVED ceiling.
// ------------------------------------------------------------------------------------------

#[test]
fn the_resolved_width_stays_below_the_two_stream_ceiling() {
    let dp = design_point();
    let ceiling = two_stream_ceiling(dp.far, PHI_P);
    for &j in &[1.0f64, 4.0, 16.0, 64.0, 400.0] {
        let g = spatial_segregation(dp.far, PHI_P, S0, H0, j, 0.316, 0.28, 0.28, NY, NY);
        assert!(
            g < ceiling,
            "a PARTIAL-mix cross-plane must be LESS segregated than the two-δ extreme: \
             g={g:.4e} vs ceiling={ceiling:.4e} at J={j}"
        );
        assert!(g > 0.0, "the resolved width must be positive at J={j}");
    }
    // and production reports the same pair
    let s = dp.run(16.0, cfg());
    assert_eq!(s.g_ceiling.unwrap().to_bits(), ceiling.to_bits());
    assert!(s.g_spatial.unwrap() < s.g_ceiling.unwrap());
}

// ------------------------------------------------------------------------------------------
// GATE 5 — emissions: the C_opt min is LOCAL, and rung 22 says so out loud.
// ------------------------------------------------------------------------------------------

#[test]
fn the_emissions_minimum_at_c_opt_is_only_local() {
    // The honest half of the rung. The derived floor at C_opt sits just BELOW the ideal-bell
    // hump peak, so the basin is NARROW: both IMMEDIATE flanks rise, but a wide-enough sweep
    // finds a LOWER ⟨EI⟩ out on rung-13's descending far flank. This is why the clean headline
    // is UNIFORMITY (g collapses) and not emissions.
    let dp = design_point();
    // `n_quad = 160`, NOT this file's default 64 — and that is slice C's finding reaching into
    // slice D rather than a taste. `beta_pdf_nodes_weights` asserts its quadrature integrates at
    // the specified mean to 1 %, and that bar has a MEASURED `n_quad` FLOOR: it rejects the top
    // of the `g` range for every `n_quad ≤ 100` and accepts from 112 up. This gate is the one
    // place in rung 22 that sweeps `J` far enough for the resolved width to reach that range, so
    // it is the one place the floor binds. Running it at 64 panics inside the guard — which is
    // the guard working, and is why the number here is 160.
    let wide = SpatialPdf { n_quad: 160, ..cfg() };
    // The flank probes are J = 9 and 25, NOT the coarse location grid's 9 and 36 — and the
    // difference is the rung's own finding, not a fudge. The WIDTH's minimum is broad, which is
    // why a ~2× coarse grid locates it safely; the EMISSIONS minimum is NARROW, because the
    // derived floor sits just BELOW the ideal-bell hump peak. Probed at J=36 the descending far
    // flank has already taken over and the reading is monotone (measured: 1.1860 / 1.1767 /
    // 1.1639), which is a true statement about a different question. Two quantities, two grids.
    let eis: Vec<f64> = [9.0f64, 16.0, 25.0]
        .iter()
        .map(|&j| dp.run(j, wide).ei_no_spatial.unwrap())
        .collect();
    assert!(
        eis[1] < eis[0] && eis[1] < eis[2],
        "C_opt must be a LOCAL emissions min over its NARROW basin: {eis:?}"
    );
}

#[test]
fn the_emissions_global_min_is_at_max_segregation_not_at_c_opt() {
    // THE HONEST HALF, and the reason rung 22's headline is UNIFORMITY rather than emissions.
    // Over a wide sweep the global ⟨EI⟩ min sits at an ENDPOINT — rung-13's descending far
    // flank, spatialised: segregation at a lean mean moves mass OFF the stoich peak and lowers
    // mean NO below the narrow C_opt basin.
    let dp = design_point();
    let wide = SpatialPdf { n_quad: 160, ..cfg() };
    let js = [1.0f64, 4.0, 16.0, 64.0, 256.0];
    let eis: Vec<f64> = js.iter().map(|&j| dp.run(j, wide).ei_no_spatial.unwrap()).collect();
    let i = argmin(&eis);
    assert!(
        i == 0 || i == js.len() - 1,
        "the global emissions min must be at an ENDPOINT, got J={} ({eis:?})",
        js[i]
    );
    assert_ne!(js[i], 16.0, "the global min is NOT at C_opt — that is only the LOCAL min");
    assert!(
        eis[i] < eis[2],
        "the endpoint must BEAT the C_opt floor: {:.4} < {:.4}",
        eis[i],
        eis[2]
    );
}

#[test]
fn super_eq_o_lifts_through_the_shared_bell() {
    // Rung 21's lift threads the SAME ideal bell rung 22 feeds, so `super_eq_o` must raise
    // ⟨EI⟩ without touching the width — the width is a field property and carries no chemistry.
    let dp = design_point();
    let base = dp.run(9.0, cfg());
    let lifted = dp.g.zoned_nox(
        dp.far, dp.tt3, dp.tt4, dp.p, PHI_P,
        ZonedNoxOpts {
            mixing: Some(mix(9.0, H0)),
            spatial: Some(cfg()),
            super_eq_o: true,
            ..opts()
        },
    );
    assert_eq!(
        base.g_spatial.unwrap().to_bits(),
        lifted.g_spatial.unwrap().to_bits(),
        "the resolved WIDTH is a field property — super_eq_o must not move it by one bit"
    );
    assert!(
        lifted.ei_no_spatial.unwrap() > base.ei_no_spatial.unwrap(),
        "rung 21's lift must raise ⟨EI⟩ through the shared bell"
    );
}

// ------------------------------------------------------------------------------------------
// GATE 6 — guards.
// ------------------------------------------------------------------------------------------

#[test]
fn spatial_requires_a_mixing_config() {
    let e = catch_unwind(|| {
        let (g, tt3, tt4, far, p) = design_state();
        g.zoned_nox(
            far, tt3, tt4, p, PHI_P,
            ZonedNoxOpts { spatial: Some(cfg()), tau_q: Some(1e-3), ..opts() },
        )
    })
    .expect_err("spatial without mixing must panic");
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
                mixing: Some(mix(16.0, H0)),
                spatial: Some(cfg()),
                pdf: Some(MixingPdf { n_bell: NB, n_quad: NQ, ..MixingPdf::default() }),
                ..opts()
            },
        )
    })
    .expect_err("two closures of the same variance physics must panic");
    let msg = e
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_string()))
        .unwrap_or_default();
    assert!(msg.contains("AT MOST ONE"), "wrong panic: {msg}");
}

#[test]
fn a_lean_primary_has_no_rql_geometry() {
    // The jet must dilute DOWN to the mean, so the primary has to be RICHER than it. A primary
    // at the overall mean is not a numerical edge case — it is a different combustor.
    let e = catch_unwind(|| spatial_segregation(0.0295, 0.3, S0, H0, 16.0, 0.316, 0.28, 0.28, 8, 8))
        .expect_err("a lean primary must panic");
    let msg = e
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_string()))
        .unwrap_or_default();
    assert!(msg.contains("RQL geometry"), "wrong panic: {msg}");
}

#[test]
fn the_config_rejects_non_positive_geometry() {
    for bad in [
        SpatialPdf { s: 0.0, ..cfg() },
        SpatialPdf { k_p: -0.1, ..cfg() },
        SpatialPdf { k_y: 0.0, ..cfg() },
        SpatialPdf { k_z: -1.0, ..cfg() },
    ] {
        assert!(catch_unwind(move || bad.validate()).is_err(), "{bad:?} must be rejected");
    }
    for bad in [SpatialPdf { ny: 1, ..cfg() }, SpatialPdf { n_quad: 1, ..cfg() }] {
        assert!(catch_unwind(move || bad.validate()).is_err(), "{bad:?} must be rejected");
    }
    cfg().validate(); // the shipped one is valid
}
