//! Rung-9 verification: rich primary / RQL — the rich side of the NOx bell.
//!
//! Rung 8 resolved a hot, near-stoichiometric primary and lifted EI_NO into the ICAO band, but
//! held the primary LEAN-to-stoich (φ_p ≤ 1) — it could only see the lean flank of the NO-vs-φ
//! bell. Rung 9 lets the primary run RICH (φ_p up to 2.0): the 8-species equilibrium pool
//! (CO/H₂ already unknowns; reactions 1+2 span the water-gas shift) now carries MAJOR CO/H₂,
//! set by a branched seed in the solve. No new species, reactions, or datum — the same
//! extended-Zeldovich integrator on a rich pool. The payoff, which is RQL's whole reason to
//! exist: EI_NO forms a BELL that peaks near stoichiometric and FALLS steeply on the rich
//! flank, so a rich primary is a low-NOx regime. Mix-out here is the IDEAL (infinitely-fast)
//! quench — NO frozen at the primary value; the finite-rate quench is rung 10's seam.
//!
//! Gates (`docs/rung9-spec.md`), priority order:
//!
//! 1. **reduce-to-rung-8 (LOAD-BEARING)** — at φ_p ≤ 1 the rich branch is never taken; the lean
//!    seed is byte-identical, so rungs 1–8 are bit-for-bit and the rung-8 exact same-`T_p`
//!    identity still holds. Running `zoned_nox` (rich or lean) never touches the cycle `far`.
//! 2. **rich equilibrium is right** — methane φ=1.05 AFT in the CEA band (~2231 K), the AFT peak
//!    sits slightly rich, CO/H₂ are MAJOR and grow with φ, and the rich pool satisfies the
//!    water-gas-shift identity (a thermodynamic self-check on the branched solve).
//! 3. **the EI_NO BELL** — peaks near stoich (φ_p ≈ 0.95–1.0) and falls steeply rich (EI(1.3) is
//!    <10 % of the peak). THE rung-9 lesson: why RQL burns rich.
//! 4. **rich mix-out still returns to Tt4**, split-independent across rich φ_p (the
//!    re-equilibration gate, now releasing CO/H₂ oxidation energy too).
//! 5. **soot-bound guard** — φ_p ≤ 2.0 accepted, above rejected (the 5-species / no-C(s) basis).
//! 6. **the K-check + trace guard still bind** at the (lower) rich primary T.

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{
    air_mole_fractions, equil_solve, equilibrium_composition, f_stoich, g_molar, h_molar_a, m_air,
    Gas, M_CH2, RU, SP_REACT,
};
use turbojet::nox::{kcheck_ratio, thermal_no, ZonedNoxOpts};

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}
fn losses(eta_b: f64) -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.88, eta_b, pi_b: 0.96, eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
        ..Losses::default()
    }
}
const TAU: f64 = 3e-3;

fn opts() -> ZonedNoxOpts {
    ZonedNoxOpts { tau: TAU, ..ZonedNoxOpts::default() }
}

fn close(a: f64, b: f64, rel: f64) -> bool {
    (a - b).abs() <= rel * b.abs()
}

fn xg(name: &str) -> f64 {
    air_mole_fractions().iter().find(|&&(s, _)| s == name).expect("air species").1
}

/// Build the equilibrium engine and read the (derived) station-3/4 state. Same helper as
/// `rung8.rs` — NO is trace, so the cycle is bit-for-bit rung 6.
fn design_point(eta_b: f64) -> (Gas, f64, f64, f64, f64) {
    let r = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, 50_000.0, losses(eta_b))
        .run(&flight(), 50.0);
    let (st3, st4) = (r.station("3"), r.station("4"));
    (Gas::reacting_equilibrium(), st3.tt, st4.tt, st4.far, st4.pt)
}

// ------------------------------------------------------------------------------------- //
// GATE 1 — reduce-to-rung-8: lean branch byte-identical; cycle untouched.                 //
// ------------------------------------------------------------------------------------- //
#[test]
fn reduce_lean_branch_unchanged() {
    // At φ_p ≤ 1 the primary far ≤ f_stoich, so bO ≥ 2bC + bH/2 (the full-oxidation O demand)
    // and the solve takes the LEAN branch — byte-identical to rung 6/8. Confirm the branch
    // predicate holds across the whole lean range: that predicate IS the bit-for-bit guarantee.
    for phi in [0.4, 0.7, 0.9, 1.0] {
        let n_fuel = phi * f_stoich() * m_air() / M_CH2;
        let (b_c, b_h, b_o) = (n_fuel, 2.0 * n_fuel, 2.0 * xg("O2"));
        assert!(b_o >= 2.0 * b_c + b_h / 2.0 - 1e-15,
                "φ={phi} should be lean-branch (bit-for-bit)");
    }
}

#[test]
fn reduce_exact_same_tp_still_holds() {
    // The rung-8 exact reduce (α→1: zoned EI == the integrator at the same primary AFT T_p)
    // must survive rung 9 unchanged — the rich work must not perturb the lean path.
    for eta_b in [0.99, 1.0] {
        let (g, tt3, tt4, far, p) = design_point(eta_b);
        let z = g.zoned_nox(far, tt3, tt4, p, far / f_stoich(), opts()); // φ_p = φ_overall (α→1)
        let direct = thermal_no(&equilibrium_composition(far, z.t_primary, p), z.t_primary, p,
                                TAU, far, 4000, 1.0);
        assert!(close(z.ei_no(), direct.ei_no, 1e-9),
                "α→1 same-T_p identity broke: zoned {} vs direct {}", z.ei_no(), direct.ei_no);
    }
}

#[test]
fn cycle_untouched_by_rich_zoning() {
    // Running `zoned_nox` at RICH φ_p is a pure diagnostic — it must not mutate the gas. Prove
    // it by re-running the cycle after the rich zoned calls and demanding a BIT-FOR-BIT
    // identical station-4 far. In Rust `build_turbojet` consumes the gas, so each run gets its
    // own — which makes the claim STRONGER than the Python's, not weaker: the second run cannot
    // inherit a mutated cache because there is no shared cache to mutate. The diagnostic gas is
    // separate and is the one the rich calls run on.
    let run = || {
        build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, 50_000.0, losses(0.99))
            .run(&flight(), 50.0)
    };
    let r1 = run();
    let (tt3, tt4, far1, p) = (r1.station("3").tt, r1.station("4").tt, r1.station("4").far,
                               r1.station("4").pt);
    let g = Gas::reacting_equilibrium();
    for phi_p in [0.9, 1.2, 1.6, 2.0] {
        g.zoned_nox(far1, tt3, tt4, p, phi_p, opts());
    }
    let far2 = run().station("4").far;
    assert_eq!(far2.to_bits(), far1.to_bits(),
               "rich zoning perturbed the cycle far ({far2} vs {far1}) — must stay rung-6");
}

// ------------------------------------------------------------------------------------- //
// GATE 2 — rich equilibrium is correct: the CEA methane anchor + the WGS self-check.      //
// ------------------------------------------------------------------------------------- //
/// Methane-air AFT with dissociation (scale A). CH₄: bC=1, bH=4, stoich 2 O₂.
fn methane_aft(phi: f64, p: f64) -> f64 {
    const HF_CH4: f64 = -74600.0; // J/mol (JANAF)
    let n_o2 = 2.0 / phi;
    let n_n2 = n_o2 * xg("N2") / xg("O2");
    let n_ar = n_o2 * xg("Ar") / xg("O2");
    let h_react = HF_CH4;
    let (mut lo, mut hi) = (1000.0f64, 3200.0f64);
    for _ in 0..100 {
        let t = 0.5 * (lo + hi);
        let comp = equil_solve(1.0, 4.0, 2.0 * n_o2, n_n2 + n_ar, t, p);
        // Summed in SP_REACT order, then the two inerts — the order Python's dict holds.
        let h_prod: f64 = SP_REACT.iter().zip(comp.iter()).map(|(&s, &n)| n * h_molar_a(s, t))
            .sum::<f64>()
            + n_n2 * h_molar_a("N2", t)
            + n_ar * h_molar_a("Ar", t);
        if h_prod > h_react {
            hi = t;
        } else {
            lo = t;
        }
    }
    0.5 * (lo + hi)
}

#[test]
fn methane_rich_aft_cea_anchor() {
    // CEA equilibrium methane-air AFT: ~2224 K stoich, ~2231 K at φ=1.05, peak slightly rich
    // (Marzouk 2024, ETASR/arXiv 2503.11826). Ours is ~7 K high (NO/N + 5-species deferred, the
    // same offset rung 6 noted). Anchor the rich point AND the rollover location.
    let tf_105 = methane_aft(1.05, 101325.0);
    assert!(2225.0 < tf_105 && tf_105 < 2248.0, "methane φ=1.05 AFT {tf_105:.1} out of CEA band");
    let phis = [0.95, 1.0, 1.05, 1.10, 1.30];
    let ts: Vec<f64> = phis.iter().map(|&phi| methane_aft(phi, 101325.0)).collect();
    let peak_i = (0..ts.len()).max_by(|&a, &b| ts[a].partial_cmp(&ts[b]).unwrap()).unwrap();
    let peak_phi = phis[peak_i];
    assert!((1.0..=1.08).contains(&peak_phi),
            "AFT peak at φ={peak_phi} — should be slightly rich");
    assert!(ts[4] < ts[1], "rich flank must fall below stoich (AFT rollover)");
}

#[test]
fn rich_pool_is_co_h2_major_and_wgs_consistent() {
    // Rich equilibrium of the (CH2)n fuel: CO/H₂ become MAJOR and grow with φ; and the pool
    // satisfies the water-gas shift CO + H₂O ⇌ CO₂ + H₂ (Δν=0) with Kp from the SAME g0 the
    // solve uses — a thermodynamic self-check that the branched rich solve landed on the real
    // equilibrium, not merely on an atom-balanced point.
    let (p, t) = (802_664.8f64, 2200.0f64);
    let get = |c: &Vec<(&str, f64)>, k: &str| c.iter().find(|&&(s, _)| s == k).unwrap().1;
    let mut prev_co = -1.0f64;
    for phi in [1.1, 1.4, 1.7] {
        let comp = equilibrium_composition(phi * f_stoich(), t, p);
        let nt: f64 = comp.iter().map(|&(_, v)| v).sum();
        assert!(get(&comp, "CO") / nt > 0.02,
                "φ={phi}: CO not major ({:.4})", get(&comp, "CO") / nt);
        assert!(get(&comp, "H2") / nt > 0.005,
                "φ={phi}: H2 not major ({:.4})", get(&comp, "H2") / nt);
        assert!(get(&comp, "CO") > prev_co, "CO must grow with φ (richer)");
        prev_co = get(&comp, "CO");
        let d_g = g_molar("CO2", t) + g_molar("H2", t) - g_molar("CO", t) - g_molar("H2O", t);
        let kp = (-d_g / (RU * t)).exp();
        let ratio = (get(&comp, "CO2") * get(&comp, "H2")) / (get(&comp, "CO") * get(&comp, "H2O"));
        assert!(close(ratio, kp, 1e-6), "φ={phi}: WGS off — {ratio:.4} vs Kp {kp:.4}");
    }
}

// ------------------------------------------------------------------------------------- //
// GATE 3 — the EI_NO bell: peaks near stoich, FALLS on the rich flank.                    //
// ------------------------------------------------------------------------------------- //
#[test]
fn ei_no_bell_falls_on_rich_flank() {
    let (g, tt3, tt4, far, p) = design_point(0.99);
    let phis = [0.7, 0.9, 0.95, 1.0, 1.05, 1.1, 1.3, 1.5];
    let ei: Vec<f64> = phis.iter().map(|&phi| g.zoned_nox(far, tt3, tt4, p, phi, opts()).ei_no())
        .collect();
    let peak_i = (0..ei.len()).max_by(|&a, &b| ei[a].partial_cmp(&ei[b]).unwrap()).unwrap();
    let peak_phi = phis[peak_i];
    assert!((0.9..=1.05).contains(&peak_phi), "EI_NO should peak near stoich, got φ={peak_phi}");
    // The rich flank collapses — a rich primary is low-NOx (RQL's reason to exist):
    assert!(ei[6] < 0.10 * ei[peak_i],
            "rich flank must collapse: EI(1.3)={:.3} vs peak {:.3}", ei[6], ei[peak_i]);
    // monotone falling once past the peak:
    for (a, b) in [(4usize, 5usize), (5, 6), (6, 7)] {
        assert!(ei[a] > ei[b],
                "EI_NO must fall monotonically rich: EI({}) !> EI({})", phis[a], phis[b]);
    }
    // and the peak still lands in the ICAO band (single-digit-to-tens g/kg):
    assert!(5.0 < ei[peak_i] && ei[peak_i] < 60.0,
            "peak EI_NO {:.2} outside ICAO band", ei[peak_i]);
}

// ------------------------------------------------------------------------------------- //
// GATE 4 — rich mix-out returns to Tt4, split-independent.                                //
// ------------------------------------------------------------------------------------- //
#[test]
fn rich_mixout_returns_to_tt4_split_independent() {
    let (g, tt3, tt4, far, p) = design_point(0.99);
    let t_mix: Vec<f64> = [0.9, 1.2, 1.5, 1.8, 2.0]
        .iter()
        .map(|&phi| g.zoned_nox(far, tt3, tt4, p, phi, opts()).t_mix)
        .collect();
    for &tm in &t_mix {
        assert!((tm - t_mix[0]).abs() < 1e-3,
                "T_mix must be split-independent across rich φ_p: {t_mix:?}");
        assert!((tm - tt4).abs() < 0.02 * tt4, "rich mix-out {tm:.1} did not return to Tt4");
    }
}

#[test]
fn rich_dilution_drops_fraction_not_index() {
    // NO-mole conservation through the rich dilution: the mole FRACTION drops but the NO MOLES
    // are conserved. Assert the freeze arithmetic DIRECTLY — the NO moles per mol total air out
    // of the mix (x_no_mix·ntot_mix) equal those set in the primary and scaled by α — rather
    // than the `ei_no == primary.ei_no` property, which is a tautology.
    let (g, tt3, tt4, far, p) = design_point(0.99);
    let z = g.zoned_nox(far, tt3, tt4, p, 1.1, opts());
    assert!(z.ppm_mix() < z.ppm_primary(), "dilution must drop the NO mole fraction");
    let ntot_prim: f64 =
        equilibrium_composition(z.far_primary, z.t_primary, p).iter().map(|&(_, v)| v).sum();
    let ntot_mix: f64 = equilibrium_composition(far, z.t_mix, p).iter().map(|&(_, v)| v).sum();
    let n_no_mix = z.x_no_mix * ntot_mix;
    let n_no_prim = z.alpha * z.primary.x_no * ntot_prim;
    assert!(close(n_no_mix, n_no_prim, 1e-9),
            "NO moles not conserved through dilution: {n_no_mix} vs {n_no_prim}");
}

// ------------------------------------------------------------------------------------- //
// GATE 5 — the soot-bound scope guard (φ_p ≤ 2.0).                                        //
// ------------------------------------------------------------------------------------- //
#[test]
fn soot_bound_accepts_exactly_two() {
    let (g, tt3, tt4, far, p) = design_point(0.99);
    g.zoned_nox(far, tt3, tt4, p, 2.0, opts()); // at the bound: accepted
}

#[test]
#[should_panic(expected = "outside (0, 2]")]
fn soot_bound_rejects_22() {
    let (g, tt3, tt4, far, p) = design_point(0.99);
    g.zoned_nox(far, tt3, tt4, p, 2.2, opts());
}

#[test]
#[should_panic(expected = "outside (0, 2]")]
fn soot_bound_rejects_30() {
    let (g, tt3, tt4, far, p) = design_point(0.99);
    g.zoned_nox(far, tt3, tt4, p, 3.0, opts());
}

// ------------------------------------------------------------------------------------- //
// GATE 6 — the K-check + trace guard bind at the (lower) rich primary T.                  //
// ------------------------------------------------------------------------------------- //
#[test]
fn kcheck_and_trace_hold_at_rich_primary() {
    let (g, tt3, tt4, far, p) = design_point(0.99);
    // A rich primary is COOLER (the AFT rolls over) — down to ~1715 K at φ_p=2. `thermal_no`
    // asserts both the K-check and the trace guard on every zoned call, so a passing rich sweep
    // IS the gate; also check the K-check constant directly across the rich primary band.
    for phi_p in [1.2, 1.6, 2.0] {
        let z = g.zoned_nox(far, tt3, tt4, p, phi_p, opts());
        let r = kcheck_ratio(z.t_primary);
        assert!(0.90 < r && r < 1.15,
                "K-check {r:.4} at rich primary T={:.0} out of band", z.t_primary);
        assert!(z.primary.x_no_eq < 0.02,
                "NO must stay trace (decoupling) in the rich primary");
    }
}
