//! Rung-12 verification: spatial unmixedness — the two-stream variance layer that turns the
//! NO-vs-J curve BACK UP and recovers the Holdeman dilution-jet optimum AT `C_opt`.
//!
//! Rung 11 was MEAN-FIELD (one well-mixed core diluting on a mean β(t)), so its J-sweep is
//! monotone: a stronger jet only ever re-makes LESS NO. Real dilution jets have an OPTIMUM at
//! the Holdeman group `C = (S/H)√J ≈ 2.5` — UNDER-penetration and OVER-penetration BOTH leave a
//! hot near-stoich core that misses the fast jet mixing and lingers. Rung 12 adds that core as
//! a second stream:
//!
//! ```text
//! EI_total = (1−w)·EI(τ_mean) + w·EI(τ_core)
//! ```
//!
//! with a BULK quenched at the rung-11 jet time `τ_mean(J) ∝ 1/√J` (the still-falling reference)
//! and an under-mixed CORE whose fraction and dwell both grow off-optimum (the dwell ABSOLUTE,
//! so it survives J→∞). The unmixedness `u(C) = |ln(C/C_opt)|` is KINKED at `C_opt`, and that
//! non-zero slope PINS the EI-min AT `C_opt`. Still a pure diagnostic: bit-for-bit rung 6.
//!
//! Gates (`docs/rung12-spec.md`), priority order:
//!
//! 1. **reduce-to-rung-11 (LOAD-BEARING, exact)** — no `unmixedness` leaves every rung-12 field
//!    `None`; and `k_u = 0` is bit-for-bit the mean-field bulk at every J.
//! 2. **the TURN-UP (THE lesson)** — `ei_no_unmixed` is NON-monotone in J: it FALLS then RISES.
//! 3. **the optimum is AT the Holdeman group** — `J_min == J_opt = (C_opt·H/S)²`, shifting as
//!    `(H/S)²` with the spacing. **See [`the_pin_at_c_opt_has_a_spacing_limit`]: this holds
//!    over a BAND of S, not for all S, and the port is what measured the edge.**
//! 4. **at `C_opt` the two-stream total == the mean-field bulk** (`w = 0` there — the seam).
//! 5. **the core penalty survives J→∞** and grows off-optimum.
//! 6. **`w(C)` is the unmixedness** — 0 at `C_opt`, rising on BOTH flanks, kinked, symmetric.
//! 7. **cycle untouched**; 8. **dormancy + require-mixing + positivity guards**.

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{equilibrium_composition, f_stoich, hf_fuel_default, Gas};
use turbojet::nox::{
    primary_aft, quench_no, quench_trajectory, thermal_no, JetMixing, QuenchOpts, QuenchPoint,
    Unmixedness, ZonedNoxOpts,
};

const TAU: f64 = 3e-3;
const NG: usize = 33;

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
fn jet(j: f64, c_e: f64) -> JetMixing {
    JetMixing { j, c_e, shape_n: 2.0, ..JetMixing::default() }
}
fn spacing(s: f64) -> Unmixedness {
    Unmixedness { s, ..Unmixedness::default() }
}

/// The uniformity optimum `J_opt` where `C = (S/H)√J_opt = C_opt`.
fn j_opt(u: &Unmixedness) -> f64 {
    (u.c_opt * JetMixing::default().h / u.s).powi(2)
}

fn design_point() -> (Gas, f64, f64, f64, f64) {
    let g = Gas::reacting_equilibrium();
    let r = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, flight().p0, losses())
        .run(&flight(), 50.0);
    (g, r.station("3").tt, r.station("4").tt, r.station("4").far, r.station("4").pt)
}

struct Traj {
    comp: Vec<(&'static str, f64)>,
    t_p: f64,
    alpha: f64,
    n0: f64,
    tab: Vec<QuenchPoint>,
    tt3: f64,
    far: f64,
    p: f64,
}

fn reusable_traj(phi_p: f64) -> Traj {
    let (_g, tt3, _tt4, far, p) = design_point();
    let far_p = phi_p * f_stoich();
    let alpha = far / far_p;
    let t_p = primary_aft(far_p, p, tt3, hf_fuel_default());
    let comp = equilibrium_composition(far_p, t_p, p);
    let nox = thermal_no(&comp, t_p, p, TAU, far_p, 4000, 1.0);
    let ntot: f64 = comp.iter().map(|&(_, v)| v).sum();
    let n0 = alpha * nox.x_no * ntot;
    let tab = quench_trajectory(&comp, t_p, alpha, far, tt3, p, NG);
    Traj { comp, t_p, alpha, n0, tab, tt3, far, p }
}

/// One two-stream evaluation, mirroring `zoned_nox`'s rung-12 math on a SHARED trajectory —
/// which is the whole reason a J sweep is affordable. [`zoned_nox_matches_two_stream_helper`]
/// pins it to the production path at one point, so the sweeps below exercise the same
/// arithmetic the shipped code does.
struct TwoStream {
    ei: f64,
    ei_bulk: f64,
    ei_core: f64,
    c: f64,
    w: f64,
    max_a: f64,
}

impl Traj {
    fn two_stream(&self, m: &JetMixing, u: &Unmixedness) -> TwoStream {
        let c = u.c(m);
        let w = u.core_fraction(c);
        let sched = |x: f64| m.schedule(x);
        let run = |tau_q: f64| {
            quench_no(
                &self.comp, self.t_p, self.alpha, self.far, self.tt3, self.p, self.n0, tau_q,
                QuenchOpts {
                    ngrid: NG, tab: Some(&self.tab), schedule: Some(&sched),
                    ..QuenchOpts::default()
                },
            )
        };
        let qb = run(m.tau_q());
        let qc = run(u.core_dwell(c));
        TwoStream {
            ei: (1.0 - w) * qb.ei + w * qc.ei,
            ei_bulk: qb.ei,
            ei_core: qc.ei,
            c,
            w,
            max_a: qb.max_a.max(qc.max_a),
        }
    }
    fn sweep(&self, u: &Unmixedness, js: &[f64]) -> Vec<TwoStream> {
        js.iter().map(|&j| self.two_stream(&jet(j, 0.20), u)).collect()
    }
}

fn argmin(v: &[f64]) -> usize {
    let mut i = 0;
    for (k, &x) in v.iter().enumerate() {
        if x < v[i] {
            i = k;
        }
    }
    i
}

// --------------------------------------------------------------------------------------
// GATE 1 — reduce-to-rung-11.
// --------------------------------------------------------------------------------------

#[test]
fn reduce_unmixedness_none_is_rung11_meanfield() {
    let (g, tt3, tt4, far, p) = design_point();
    for j in [9.0, 25.0, 64.0] {
        let m = JetMixing { j, ..JetMixing::default() };
        let a = g.zoned_nox(far, tt3, tt4, p, 1.5, ZonedNoxOpts { mixing: Some(m), ..opts() });
        let b = g.zoned_nox(far, tt3, tt4, p, 1.5,
                            ZonedNoxOpts { mixing: Some(m), unmixedness: None, ..opts() });
        for s in [&a, &b] {
            assert!(s.unmixedness.is_none() && s.ei_no_unmixed.is_none() && s.w_core.is_none());
            assert!(s.c_holdeman.is_none() && s.ei_no_core.is_none());
        }
        assert_eq!(a.ei_no_quenched.unwrap().to_bits(), b.ei_no_quenched.unwrap().to_bits());
        assert_eq!(a.max_a_quench.unwrap().to_bits(), b.max_a_quench.unwrap().to_bits());
    }
}

/// `k_u = 0` ⇒ `w ≡ 0` ⇒ the two-stream total collapses onto the mean-field BULK at EVERY J,
/// bit-for-bit — variance switched off recovers rung 11.
///
/// Note this is NOT a short-circuit: the core integration still RUNS at `τ_core` and must not
/// trip an assert on the way. That is what makes it a second-level reduce rather than a
/// re-statement of gate 1.
#[test]
fn reduce_k_u_zero_is_bit_for_bit_meanfield_bulk() {
    let (g, tt3, tt4, far, p) = design_point();
    for j in [4.0, 25.0, 100.0] {
        let s = g.zoned_nox(far, tt3, tt4, p, 1.5, ZonedNoxOpts {
            mixing: Some(JetMixing { j, shape_n: 2.0, ..JetMixing::default() }),
            unmixedness: Some(Unmixedness { k_u: 0.0, ..Unmixedness::default() }),
            ..opts()
        });
        assert_eq!(s.w_core.unwrap(), 0.0);
        assert_eq!(s.ei_no_unmixed.unwrap().to_bits(), s.ei_no_quenched.unwrap().to_bits(),
                   "J={j}: k_u=0 must be bit-for-bit the mean-field bulk");
    }
}

/// Pin the fast sweep helper to the PRODUCTION `zoned_nox` path at one point, so every sweep
/// gate below is exercising the shipped arithmetic and not a parallel re-implementation.
#[test]
fn zoned_nox_matches_two_stream_helper() {
    let (g, tt3, tt4, far, p) = design_point();
    let t = reusable_traj(1.5);
    let (m, u) = (jet(36.0, 0.20), Unmixedness::default());
    let h = t.two_stream(&m, &u);
    let s = g.zoned_nox(far, tt3, tt4, p, 1.5,
                        ZonedNoxOpts { mixing: Some(m), unmixedness: Some(u), ..opts() });
    assert!((s.ei_no_unmixed.unwrap() - h.ei).abs() < 1e-12 * h.ei);
    assert!((s.c_holdeman.unwrap() - h.c).abs() < 1e-12);
    assert_eq!(s.w_core.unwrap().to_bits(), h.w.to_bits());
    assert!((s.ei_no_core.unwrap() - h.ei_core).abs() < 1e-12 * h.ei_core);
}

// --------------------------------------------------------------------------------------
// GATE 2 — the TURN-UP.
// --------------------------------------------------------------------------------------

/// THE rung-12 lesson: unmixedness breaks rung 11's monotone fall. EI falls as the jet
/// strengthens (mean-field win) THEN rises as over-penetration strands an un-mixed core — an
/// interior minimum. The last assertion is the discriminator: the mean-field bulk ALONE is
/// still falling at the far end, so the variance is what turns the total up.
#[test]
fn j_sweep_turns_back_up_interior_minimum() {
    let t = reusable_traj(1.5);
    let js = [4.0, 9.0, 16.0, 25.0, 36.0, 49.0, 64.0, 100.0];
    let rows = t.sweep(&Unmixedness::default(), &js);
    let eis: Vec<f64> = rows.iter().map(|r| r.ei).collect();
    let imin = argmin(&eis);
    assert!(0 < imin && imin < eis.len() - 1,
            "minimum must be INTERIOR (the turn-up), got imin={imin}: {eis:?}");
    for i in 0..imin {
        assert!(eis[i] > eis[i + 1], "must FALL before the min: {eis:?}");
    }
    for i in imin..eis.len() - 1 {
        assert!(eis[i] < eis[i + 1], "must RISE after the min: {eis:?}");
    }
    assert!(eis[eis.len() - 1] > 1.5 * eis[imin],
            "far-flank turn-up should be material: min={:.3} J=100={:.3}",
            eis[imin], eis[eis.len() - 1]);
    let bulks: Vec<f64> = rows.iter().map(|r| r.ei_bulk).collect();
    assert!(bulks[bulks.len() - 1] < bulks[imin],
            "the mean-field bulk is still monotone-falling (rung 11): {bulks:?}");
}

// --------------------------------------------------------------------------------------
// GATE 3 — the optimum is AT the Holdeman group, and where that stops being true.
// --------------------------------------------------------------------------------------

/// The recovered optimum sits AT the Holdeman uniformity group, so changing the jet spacing
/// moves the EI-min as `(H/S)²`. Both spacings here are the ones the Python's own gate uses.
#[test]
fn optimum_is_at_holdeman_c_opt_and_shifts_as_h_over_s_squared() {
    let t = reusable_traj(1.5);
    for s in [0.0625, 0.0500] {
        let u = spacing(s);
        let jo = j_opt(&u); // 16 (S=.0625), 25 (S=.05)
        let js = [jo / 4.0, jo / 2.0, jo, 2.0 * jo, 4.0 * jo]; // C = C_opt·{.5,.707,1,1.41,2}
        let rows = t.sweep(&u, &js);
        let imin = argmin(&rows.iter().map(|r| r.ei).collect::<Vec<_>>());
        assert_eq!(js[imin].to_bits(), jo.to_bits(),
                   "S={s}: EI-min must sit AT J_opt={jo} (C=C_opt), got J={}", js[imin]);
        assert!((rows[imin].c - u.c_opt).abs() < 1e-9,
                "S={s}: the min's C must equal C_opt={}", u.c_opt);
    }
    // the shift itself, as a relationship between two LOCATIONS: J_opt scales as (H/S)².
    let (a, b) = (j_opt(&spacing(0.0625)), j_opt(&spacing(0.05)));
    assert!((b / a - (0.0625f64 / 0.05).powi(2)).abs() < 1e-12,
            "J_opt must shift EXACTLY as (H/S)²: {a} → {b}");
}

/// **THE PIN HAS A SPACING LIMIT, and the port is what found it.**
///
/// `Unmixedness`'s own docstring says the EI-min pins at `C_opt` "for ALL S". It does not. The
/// Python's gate 3 only ever tests `S ∈ {0.0625, 0.05}`, both comfortably inside the band, so
/// nothing there could have seen it; this slice's oracle dumped the argmin as a shape key over
/// a wider spread and it moved.
///
/// The mechanism is the docstring's OWN pin condition,
/// `k_u·[EI(τ_core) − EI(τ_mean)] > EI(τ_mean)` at `C_opt` (where `w = 0`). At the optimum
/// `τ_mean = S/(C_e·C_opt·U_c)` GROWS with the spacing, so a wide enough spacing makes the
/// mean-field bulk quench SLOWER than the "lingering" core — the model's premise inverts, the
/// core becomes a RELIEF rather than a penalty, and the minimum slides to a stronger jet.
///
/// The boundary has a CLOSED FORM, and it is the model's own knobs with nothing added. At the
/// optimum `√J_opt = C_opt·H/S`, so
///
/// ```text
/// τ_mean(J_opt) = H/(C_e·√J_opt·U_c) = S/(C_e·C_opt·U_c)
/// ```
///
/// and the core stops lingering — stops being a penalty at all — once that reaches `τ_res`. Call
/// the spacing where they cross `S_x = τ_res·C_e·C_opt·U_c`. **MEASURED over 16 points spanning
/// two entrainment constants (`C_e` = 0.15 and 0.20, which move `S_x` from 0.0703 m to
/// 0.0938 m): the pin holds iff `S/S_x ≲ 1.2`, in BOTH sweeps.** That collapse is the evidence
/// that `S_x` is the right group — a spacing limit that moved with `C_e` in absolute metres but
/// not in this ratio.
///
/// The excess over 1 is real and has a reason. The docstring's inequality assumes `EI ∝ τ` (that
/// is what turns `dE/dlnJ` into `−E/2`); EI is SUBLINEAR in dwell at these times, so the bulk
/// falls more slowly than the algebra predicts and the pin survives ~20 % past the crossing.
/// **1.2 is therefore a MEASURED coefficient, not a derived one** — the derived value is 1.0.
/// The transition is bracketed, not resolved: pinned at 1.17, broken at 1.28, and 1.2 is a bar
/// inside that gap rather than a threshold anyone has located.
///
/// The shipped default (`S` = 0.0625, `C_e` = 0.15) sits at `S/S_x` = 0.89 — inside the band,
/// but only by about 1.3×. That is the useful thing to know and was invisible before this sweep.
#[test]
fn the_pin_at_c_opt_has_a_spacing_limit() {
    let t = reusable_traj(1.5);
    // (C_e, S) pairs spanning the boundary from both sides at two entrainment constants. The
    // expectation is computed from the RATIO, not listed per row, so the test states the law
    // rather than a table of answers.
    let cases = [
        (0.15, 0.05), (0.15, 0.0625), (0.15, 0.08), (0.15, 0.09), (0.15, 0.10),
        (0.15, 0.11), (0.15, 0.125), (0.15, 0.15),
        (0.20, 0.05), (0.20, 0.0625), (0.20, 0.08), (0.20, 0.09), (0.20, 0.10),
        (0.20, 0.11), (0.20, 0.125), (0.20, 0.15),
    ];
    let mut n_pinned = 0usize;
    for (c_e, s) in cases {
        let u = spacing(s);
        let base = JetMixing { c_e, ..JetMixing::default() };
        // the crossing spacing: where τ_mean(J_opt) == τ_res
        let s_x = u.tau_res * c_e * u.c_opt * base.u_c;
        let ratio = s / s_x;
        let jo = j_opt(&u);
        let js = [jo / 4.0, jo / 2.0, jo, 2.0 * jo, 4.0 * jo];
        let rows: Vec<TwoStream> =
            js.iter().map(|&j| t.two_stream(&jet(j, c_e), &u)).collect();
        let imin = argmin(&rows.iter().map(|r| r.ei).collect::<Vec<_>>());
        let pinned = imin == 2;
        assert_eq!(pinned, ratio < 1.2,
                   "C_e={c_e}, S={s}: S/S_x={ratio:.2}, so the pin should be {} — but the \
                    minimum sat at index {imin} (J={:.4} vs J_opt={jo:.4}). This is the \
                    MEASURED band of rung 12's 'min at C_opt for ALL S' claim.",
                   if ratio < 1.2 { "HELD" } else { "BROKEN" }, js[imin]);
        // The physical reading of the same thing: past the crossing the mean-field bulk at the
        // optimum is SLOWER than the core's absolute dwell, so the "lingering" core is really a
        // relief. The pin survives a little past that, which is the sublinearity.
        let tau_mean_opt = JetMixing { j: jo, c_e, ..JetMixing::default() }.tau_q();
        assert_eq!(tau_mean_opt > u.tau_res, ratio > 1.0,
                   "C_e={c_e}, S={s}: S_x must be exactly where τ_mean(J_opt) crosses τ_res");
        if pinned {
            n_pinned += 1;
        }
    }
    // Guard against the whole sweep landing on one side, which would make the law vacuous.
    assert!(n_pinned > 0 && n_pinned < cases.len(),
            "the spacing sweep must straddle the boundary, got {n_pinned}/{} pinned",
            cases.len());
}

// --------------------------------------------------------------------------------------
// GATE 4 — at C_opt the two-stream total == the mean-field bulk (the seam).
// --------------------------------------------------------------------------------------

#[test]
fn at_c_opt_total_equals_meanfield_bulk() {
    let (g, tt3, tt4, far, p) = design_point();
    let u = spacing(0.0625);
    let jo = j_opt(&u); // C = (S/H)√J_opt = C_opt → J_opt = 16
    let s = g.zoned_nox(far, tt3, tt4, p, 1.5, ZonedNoxOpts {
        mixing: Some(JetMixing { j: jo, shape_n: 2.0, ..JetMixing::default() }),
        unmixedness: Some(u),
        ..opts()
    });
    assert!((s.c_holdeman.unwrap() - u.c_opt).abs() < 1e-12);
    assert_eq!(s.w_core.unwrap(), 0.0);
    assert_eq!(s.ei_no_unmixed.unwrap().to_bits(), s.ei_no_quenched.unwrap().to_bits(),
               "at C_opt the two-stream total must equal the mean-field bulk");
}

// --------------------------------------------------------------------------------------
// GATE 5 — the core penalty survives J→∞ and grows off-optimum.
// --------------------------------------------------------------------------------------

/// The core quenches at an ABSOLUTE dwell that does NOT ride the vanishing jet time, so at a
/// STRONG jet it still out-emits the fast bulk many-fold — a J-scaled core would vanish and the
/// curve would stay monotone, which is exactly the rung-11 ceiling. And the dwell is MINIMISED
/// at `C_opt`, so both flanks emit more.
#[test]
fn core_penalty_survives_strong_jets_and_grows_off_optimum() {
    let t = reusable_traj(1.5);
    let u = spacing(0.0625);
    let jo = j_opt(&u);
    let r_opt = t.two_stream(&jet(jo, 0.20), &u);
    let r_over = t.two_stream(&jet(4.0 * jo, 0.20), &u); // C = 2·C_opt
    let r_under = t.two_stream(&jet(jo / 4.0, 0.20), &u); // C = C_opt/2
    assert!(r_over.ei_core > 2.0 * r_over.ei_bulk,
            "core must out-emit the fast bulk at strong jets: core={:.3} bulk={:.3}",
            r_over.ei_core, r_over.ei_bulk);
    assert!(r_over.ei_core > r_opt.ei_core && r_under.ei_core > r_opt.ei_core,
            "EI_core must be MINIMISED at C_opt: opt={:.3} under={:.3} over={:.3}",
            r_opt.ei_core, r_under.ei_core, r_over.ei_core);
}

// --------------------------------------------------------------------------------------
// GATE 6 — w(C) is the unmixedness: 0 at C_opt, kinked, symmetric in ln C.
// --------------------------------------------------------------------------------------

#[test]
fn core_fraction_is_kinked_zero_at_optimum() {
    let u = Unmixedness::default();
    assert_eq!(u.core_fraction(u.c_opt), 0.0,
               "w must be exactly 0 at C_opt (perfect tiling — no core)");
    let (lo, hi) = (u.core_fraction(u.c_opt / 1.3), u.core_fraction(u.c_opt * 1.3));
    assert!(lo > 0.0 && hi > 0.0, "w must rise on BOTH flanks: {lo} / {hi}");
    // symmetric in ln C (an L1 |ln| distance) — equal factors either side give equal w.
    assert!((u.core_fraction(u.c_opt / 1.4) - u.core_fraction(u.c_opt * 1.4)).abs() < 1e-12);
    // KINKED: a non-zero slope AT C_opt — this is what pins the min there rather than letting
    // a smooth parabola's ~0 slope drift it right.
    assert!(u.core_fraction(u.c_opt * 1.05) > 0.02,
            "kink: w must lift with non-zero slope just off C_opt");
    assert_eq!(u.core_fraction(u.c_opt * 100.0), u.w_max, "w must saturate at w_max");
}

// --------------------------------------------------------------------------------------
// GATE 7/8 — cycle untouched; dormancy, require-mixing, positivity.
// --------------------------------------------------------------------------------------

#[test]
fn cycle_untouched_by_unmixedness_quench() {
    let run = || {
        build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, flight().p0, losses())
            .run(&flight(), 50.0)
    };
    let r1 = run();
    let (tt3, tt4, far1, p) =
        (r1.station("3").tt, r1.station("4").tt, r1.station("4").far, r1.station("4").pt);
    let g = Gas::reacting_equilibrium();
    g.zoned_nox(far1, tt3, tt4, p, 1.5, ZonedNoxOpts {
        mixing: Some(jet(25.0, 0.15)), unmixedness: Some(Unmixedness::default()), ..opts()
    });
    assert_eq!(run().station("4").far.to_bits(), far1.to_bits(),
               "unmixedness quench perturbed the cycle far — must stay rung-6");
}

/// Dormancy now spans BOTH streams — the core lingers longer than the bulk, so it is the one
/// that could reach super-equilibrium first, and `max_a_quench` is the max over the pair.
#[test]
fn clamp_dormancy_persists_over_both_streams() {
    let t = reusable_traj(1.5);
    let mut overall = 0.0f64;
    for j in [4.0, 25.0, 100.0] {
        overall = overall.max(t.two_stream(&jet(j, 0.20), &Unmixedness::default()).max_a);
    }
    assert!(overall < 1.0,
            "max_a={overall:.3} ≥ 1 over the two streams — the dropped clamp is now load-bearing");
}

#[test]
fn unmixedness_requires_mixing() {
    let r = std::panic::catch_unwind(|| {
        let (g, tt3, tt4, far, p) = design_point();
        g.zoned_nox(far, tt3, tt4, p, 1.5, ZonedNoxOpts {
            tau_q: Some(1e-3), unmixedness: Some(Unmixedness::default()), ..opts()
        })
    });
    assert!(r.is_err(),
            "unmixedness without mixing must be rejected — it needs the jet's J and duct H");
}

#[test]
fn unmixedness_positivity_guards() {
    Unmixedness::default().validate();
    let bad: [Unmixedness; 6] = [
        Unmixedness { s: 0.0, ..Unmixedness::default() },
        Unmixedness { s: -0.1, ..Unmixedness::default() },
        Unmixedness { c_opt: 0.0, ..Unmixedness::default() },
        Unmixedness { tau_res: 0.0, ..Unmixedness::default() },
        Unmixedness { k_u: -1.0, ..Unmixedness::default() },
        Unmixedness { w_max: 1.5, ..Unmixedness::default() },
    ];
    for u in bad {
        let r = std::panic::catch_unwind(move || u.validate());
        assert!(r.is_err(), "Unmixedness {u:?} should be rejected");
    }
}
