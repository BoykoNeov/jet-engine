//! Rung-8 verification: combustor zoning — the primary-zone NOx effect.
//!
//! Gates (`docs/rung8-spec.md` § Verification gates), priority order:
//!
//! 1. **reduce-to-rung-7 (LOAD-BEARING)** — at α→1 (all air in the primary) the two-zone
//!    diagnostic collapses to rung-7's single mixed-out pool. TWO parts:
//!    - **EXACT**: zoned EI == `thermal_nox(far, T_p, p)` at the SAME primary AFT `T_p` (machine
//!      precision) — confirms `far_p`/α/the freeze scaling and that `T_mix` collapses to `T_p`.
//!    - **PHYSICAL**: `T_p ≈ Tt4` and the zoned EI is within a small factor of the rung-7
//!      mixed-out `thermal_nox(far, Tt4, p)`. The residual is a ~8 K scale-A/scale-B DATUM
//!      offset (formation vs 0K-sensible+HF298 — it does NOT cancel across combustion because
//!      moles change, and it SURVIVES η_b=1) PLUS a ~9 K η_b piece (more fuel → hotter true
//!      AFT). Both are dwarfed by NO's exp-in-T sensitivity at ~1500 K, so the EI ratio is O(1),
//!      not 1e-6.
//! 2. **EI_NO lands in the ICAO band** — primary φ_p = 0.9–1.0 gives single-digit-to-tens g/kg,
//!    ~6 orders above the mixed-out ~zero.
//! 3. **mix-out T is split-independent and returns to Tt4** (the re-equilibration gate). A
//!    frozen-majors mix-out traps the dissociation energy and misses Tt4 — the discriminator.
//! 4. **NO-mole conservation through dilution** — the mole FRACTION falls but EI (per kg fuel)
//!    is set in the primary and unchanged (concentration ≠ emission index).
//! 5. **T-sensitivity** — EI_NO rises monotonically and >10× over φ_p 0.7 → 1.0.
//! 6. **φ_p scope guard**; the K-check binds at the hotter primary T (asserted every call).
//!
//! NOTE on gate 6: rung 8 held φ_p ≤ 1 (lean-stoich); rung 9 widened the guard to φ_p ≤ 2 (a
//! rich RQL primary, below soot onset). So 1.2/1.5 are now VALID and the guard rejects only
//! above the soot bound. The rich payoff gates live in `rung9.rs`.

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{
    air_mole_fractions, equilibrium_composition, h_molar_a, m_air, Gas, M_CH2, M_CH2_KG,
};
use turbojet::nox::{h_air_molar_a, kcheck_ratio, m_no, ThermalNoxOpts, ZonedNoxOpts};

/// Design point = `main.py`'s (subsonic cruise) — the one the anchor's worked example uses;
/// its Tt3 ≈ 583 K makes the near-stoich primary land in the ICAO band. Derived from a REAL
/// equilibrium-engine run (never hardcoded): NO is trace, so the cycle is bit-for-bit rung 6.
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
fn t_opts() -> ThermalNoxOpts {
    ThermalNoxOpts { tau: TAU, ..ThermalNoxOpts::default() }
}

/// Build the equilibrium engine and read the (derived) station-3/4 state.
///
/// The returned `Gas` is a FRESH one rather than the engine's: `Gas`'s equilibrium section
/// caches its burn condition behind a `RefCell` and its `Clone` deliberately resets that, so
/// handing back a clone would be handing back exactly this. The diagnostic reads only the
/// spec, so the two are interchangeable — and a fresh gas makes that independence explicit.
fn design_point(eta_b: f64) -> (Gas, f64, f64, f64, f64) {
    let r = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, 50_000.0, losses(eta_b))
        .run(&flight(), 50.0);
    let (st3, st4) = (r.station("3"), r.station("4"));
    (Gas::reacting_equilibrium(), st3.tt, st4.tt, st4.far, st4.pt)
}

fn close(a: f64, b: f64, rel: f64, abs_: f64) -> bool {
    (a - b).abs() <= rel * b.abs() + abs_
}

// ------------------------------------------------------------------------------------- //
// GATE 1 — reduce-to-rung-7: exact (same T_p) + physical (≈ Tt4, O(1) factor).             //
// ------------------------------------------------------------------------------------- //
#[test]
fn reduce_exact_same_tp() {
    // At α→1 the primary far == the overall far, and the zoned NO is computed on that pool at
    // the primary AFT T_p. `thermal_nox(far, T_p, p)` is the IDENTICAL computation, so the two
    // EIs match to machine precision — regardless of η_b. This certifies far_p, α and the
    // mole-freeze scaling; it is not a tautology, because a bug in any of them breaks it.
    for eta_b in [0.99, 1.0] {
        let (g, tt3, tt4, far, p) = design_point(eta_b);
        let z = g.zoned_nox(far, tt3, tt4, p, far / turbojet::gas::f_stoich(), opts()); // α = 1
        assert!(close(z.alpha, 1.0, 1e-9, 0.0), "α should be 1 at φ_p=φ_overall, got {}", z.alpha);
        let r = g.thermal_nox(far, z.t_primary, p, t_opts());
        assert!(close(z.ei_no(), r.ei_no, 1e-9, 0.0),
                "exact reduce broke: zoned {} != rung-7@T_p {} (η_b={eta_b})", z.ei_no(), r.ei_no);
    }
}

#[test]
fn reduce_physical_to_mixed_out() {
    // The PHYSICAL reduce: at α→1 the primary really is ~ the cycle's station-4 state. T_p sits
    // a few K above Tt4 (datum ~8 K + η_b ~9 K), and the mixed-out rung-7 EI is within an O(1)
    // factor — this is reduce-to-rung-7, not reduce-to-itself.
    let (g, tt3, tt4, far, p) = design_point(1.0);
    let z = g.zoned_nox(far, tt3, tt4, p, far / turbojet::gas::f_stoich(), opts());
    assert!(0.0 < z.t_primary - tt4 && z.t_primary - tt4 < 20.0,
            "T_p {:.1} not just above Tt4 {tt4}", z.t_primary);
    let ei_mixed = g.thermal_nox(far, tt4, p, t_opts()).ei_no;
    let ratio = z.ei_no() / ei_mixed;
    assert!(0.5 < ratio && ratio < 3.0, "α→1 EI ratio {ratio:.2} vs mixed-out out of O(1) band");
}

#[test]
fn cycle_untouched_by_zoning() {
    // The cycle is bit-for-bit rung 6: `zoned_nox` is a pure diagnostic — the equilibrium pool
    // never carries NO/N, and calling it does not perturb the station-4 far.
    let (g, tt3, tt4, far, p) = design_point(0.99);
    let names: Vec<&str> = equilibrium_composition(far, tt4, p).iter().map(|&(s, _)| s).collect();
    assert!(!names.contains(&"NO") && !names.contains(&"N"),
            "NO/N leaked into the equilibrium pool");
    g.zoned_nox(far, tt3, tt4, p, 0.9, opts());
    let far2 = build_turbojet(g, 10.0, 1500.0, 50_000.0, losses(0.99))
        .run(&flight(), 50.0)
        .station("4")
        .far;
    assert!(close(far, far2, 1e-12, 0.0), "running zoned_nox perturbed the cycle far");
}

// ------------------------------------------------------------------------------------- //
// GATE 2 — EI_NO climbs into the ICAO band; mixed-out is ~6 orders lower.                 //
// ------------------------------------------------------------------------------------- //
#[test]
fn ei_no_in_icao_band() {
    let (g, tt3, tt4, far, p) = design_point(0.99);
    for phi_p in [0.9, 1.0] {
        let ei = g.zoned_nox(far, tt3, tt4, p, phi_p, opts()).ei_no();
        assert!(5.0 < ei && ei < 80.0,
                "φ_p={phi_p}: EI_NO {ei:.2} g/kg outside single-digit-to-tens");
    }
    // The mixed-out station-4 number is ~zero, and the primary lifts it ~6 orders of magnitude.
    let ei_primary = g.zoned_nox(far, tt3, tt4, p, 1.0, opts()).ei_no();
    let ei_mixed = g.thermal_nox(far, tt4, p, t_opts()).ei_no;
    assert!(ei_mixed < 1e-3, "mixed-out EI_NO {ei_mixed:.2e} not ~zero");
    assert!(ei_primary / ei_mixed > 1e4,
            "primary lift only {:.1e}× (< 1e4)", ei_primary / ei_mixed);
}

// ------------------------------------------------------------------------------------- //
// GATE 3 — mix-out T split-independent, returns to Tt4; frozen-majors misses.             //
// ------------------------------------------------------------------------------------- //
#[test]
fn mixout_split_independent_returns_to_tt4() {
    let (g, tt3, tt4, far, p) = design_point(0.99);
    let t_mix: Vec<f64> = [0.7, 0.8, 0.9, 1.0]
        .iter()
        .map(|&phi_p| g.zoned_nox(far, tt3, tt4, p, phi_p, opts()).t_mix)
        .collect();
    // α cancels analytically in the enthalpy balance ⇒ T_mix is the SAME for every split, to
    // the bisection tolerance (a wrong basis / frozen composition would break this). The
    // TIGHTEST true bar is one bisection quantum, which `nox_oracle.rs` measures and asserts;
    // 1e-3 K is the Python's bar and is kept here so the two suites say the same thing.
    for &t in &t_mix[1..] {
        assert!(close(t, t_mix[0], 0.0, 1e-3), "T_mix not split-independent: {t_mix:?}");
    }
    // And it returns to ≈ Tt4 (within the ~8 K datum + η_b gap): the re-equilibration gate.
    assert!(0.0 < t_mix[0] - tt4 && t_mix[0] - tt4 < 30.0,
            "T_mix {:.1} did not return to Tt4 {tt4}", t_mix[0]);
}

#[test]
fn frozen_majors_mixout_misses_tt4() {
    // DISCRIMINATING check (anchor § 4): re-equilibrating the majors on mix-out releases the
    // stored dissociation energy so T_mix returns to Tt4. FREEZING the dissociated primary
    // composition (no recombination) traps that energy and lands at a DIFFERENT temperature,
    // missing Tt4 by ≫ the split-independence tolerance. This proves the re-equilibration is
    // real, not cosmetic.
    let (g, tt3, tt4, far, p) = design_point(0.99);
    let phi_p = 1.0;
    let far_p = phi_p * turbojet::gas::f_stoich();
    let alpha = far / far_p;
    let z = g.zoned_nox(far, tt3, tt4, p, phi_p, opts());
    let comp_p = equilibrium_composition(far_p, z.t_primary, p);
    let h_mix: f64 = alpha * comp_p.iter().map(|&(s, n)| n * h_molar_a(s, z.t_primary)).sum::<f64>()
        + (1.0 - alpha) * h_air_molar_a(tt3);
    // Frozen composite: primary dissociated products (× α) + dilution air, NO recombination.
    // Built as an ORDERED list, in the order Python's dict ends up holding — the primary's
    // species first, then any air species not already present. All three air species ARE
    // already there (O2 closes the reacting eight; N2 and Ar are appended after it), so the
    // second loop only accumulates and never appends.
    let mut frozen: Vec<(&str, f64)> = comp_p.iter().map(|&(s, n)| (s, alpha * n)).collect();
    for &(s, x) in air_mole_fractions().iter() {
        match frozen.iter_mut().find(|e| e.0 == s) {
            Some(e) => e.1 += (1.0 - alpha) * x,
            None => frozen.push((s, (1.0 - alpha) * x)),
        }
    }
    let (mut lo, mut hi) = (500.0f64, 3200.0f64);
    for _ in 0..100 {
        let t = 0.5 * (lo + hi);
        let hp: f64 = frozen.iter().map(|&(s, n)| n * h_molar_a(s, t)).sum();
        if hp > h_mix {
            hi = t;
        } else {
            lo = t;
        }
    }
    let t_frozen = 0.5 * (lo + hi);
    // Freezing traps the recombination energy in dissociated bonds ⇒ substantially COOLER than
    // the re-equilibrated mix-out, which releases it into sensible heat.
    assert!(t_frozen < z.t_mix - 30.0,
            "frozen mix-out {t_frozen:.1} not substantially cooler than re-eq {:.1}", z.t_mix);
    // And the re-equilibrated mix-out returns MUCH closer to Tt4 than the frozen one does —
    // only re-equilibration recovers the station-4 the cycle computed.
    assert!((z.t_mix - tt4).abs() < (t_frozen - tt4).abs(),
            "re-eq ({:.1}) should be closer to Tt4 {tt4} than frozen ({t_frozen:.1})", z.t_mix);
}

// ------------------------------------------------------------------------------------- //
// GATE 4 — NO-mole conservation through dilution (index ≠ concentration).                 //
// ------------------------------------------------------------------------------------- //
#[test]
fn no_mole_conservation() {
    let (g, tt3, tt4, far, p) = design_point(0.99);
    let z = g.zoned_nox(far, tt3, tt4, p, 0.9, opts());
    // Dilution drops the NO mole FRACTION (primary ppm → mixed ppm)...
    assert!(z.ppm_mix() < z.ppm_primary(),
            "dilution should lower NO fraction: {} vs {}", z.ppm_mix(), z.ppm_primary());
    // ...but conserves NO MOLES, so EI (per kg fuel) computed from the DILUTED state equals the
    // primary EI. NO moles per mol total air = x_no_mix·ntot_mix; fuel mass per mol air =
    // far·(M_AIR/M_CH2)·M_CH2_KG. The clean concentration-vs-index separation.
    let ntot_mix: f64 = equilibrium_composition(far, z.t_mix, p).iter().map(|&(_, v)| v).sum();
    let n_no_total = z.x_no_mix * ntot_mix;
    let fuel_mass = far * m_air() / M_CH2 * M_CH2_KG;
    let ei_from_diluted = 1000.0 * (n_no_total * m_no()) / fuel_mass;
    assert!(close(ei_from_diluted, z.ei_no(), 1e-9, 0.0),
            "EI not conserved through dilution: {ei_from_diluted} vs {}", z.ei_no());
}

// ------------------------------------------------------------------------------------- //
// GATE 5 — T-sensitivity: EI_NO rises steeply and monotonically with φ_p.                 //
// ------------------------------------------------------------------------------------- //
#[test]
fn temperature_sensitivity() {
    let (g, tt3, tt4, far, p) = design_point(0.99);
    let zs: Vec<_> = [0.7, 0.8, 0.9, 1.0]
        .iter()
        .map(|&phi_p| g.zoned_nox(far, tt3, tt4, p, phi_p, opts()))
        .collect();
    let afts: Vec<f64> = zs.iter().map(|z| z.t_primary).collect();
    let eis: Vec<f64> = zs.iter().map(|z| z.ei_no()).collect();
    assert!(afts.windows(2).all(|w| w[1] > w[0]), "primary AFT not monotone in φ_p: {afts:?}");
    assert!(eis.windows(2).all(|w| w[1] > w[0]), "EI_NO not monotone in φ_p: {eis:?}");
    assert!(eis[3] / eis[0] > 10.0,
            "EI_NO φ_p 0.7→1.0 rise {:.1}× too weak (< 10×)", eis[3] / eis[0]);
}

// ------------------------------------------------------------------------------------- //
// GATE 6 — φ_p scope guard; the K-check binds at the primary T.                           //
// ------------------------------------------------------------------------------------- //
#[test]
#[should_panic(expected = "outside (0, 2]")]
fn phi_primary_guard_rejects_25() {
    let (g, tt3, tt4, far, p) = design_point(0.99);
    g.zoned_nox(far, tt3, tt4, p, 2.5, opts());
}

#[test]
#[should_panic(expected = "outside (0, 2]")]
fn phi_primary_guard_rejects_30() {
    let (g, tt3, tt4, far, p) = design_point(0.99);
    g.zoned_nox(far, tt3, tt4, p, 3.0, opts());
}

#[test]
fn kcheck_binds_at_primary_t() {
    // The primary AFT (~2400 K) is inside the rung-7 K-check band; the thermo-kinetic ratio
    // (rate constants vs the a6/a7 thermo) still binds there. `thermal_no` asserts it on every
    // zoned call; check the constant directly at a representative primary temperature.
    for t in [2200.0, 2350.0, 2450.0] {
        let r = kcheck_ratio(t);
        assert!(0.90 < r && r < 1.15, "K-check ratio {r:.4} at primary T={t} out of band");
    }
}
