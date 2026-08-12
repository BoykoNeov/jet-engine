//! RUNG 5 — Fork B: formation-enthalpy bookkeeping (a DERIVED heat release).
//!
//! Ported from `tests/test_forkb.py`. Five gates (`docs/rung5-fork-b.md` § Verification gates),
//! in priority order:
//!
//! 1. REDUCE-TO-IDEAL + REDUCE-TO-RUNG-4 (load-bearing) — Fork B is a separate factory, so the
//!    CPG/frozen/reacting-Fork-A paths are untouched. The exact-equivalence THEOREM: a
//!    reacting-Fork-B cycle reproduces the reacting-Fork-A cycle's f/Tt5/thrust/TSFC to machine
//!    precision, because the released chemical energy is IDENTICALLY `f*LHV` for complete
//!    combustion.
//! 2. DERIVED LHV = MATTINGLY hPR — `hf_fuel = -34.99 kJ/mol` => `LHV = 42.8000 MJ/kg`.
//! 3. FORMATION SELF-CHECK — `H(298.15) = dHf` per species; elements land at h = 0 at 298.15 K.
//! 4. ABSOLUTE-BALANCE CLOSURE — the burner's `Σ N h(react) = Σ N h(prod) + loss` assert fires
//!    on every Fork-B run (checked here explicitly at the converged f), and the fuel enthalpy is
//!    a LIVE knob (a lower LHV needs more fuel: f rises).
//! 5. AFT PHYSICAL PLAUSIBILITY (test-only, no book digit) — no-dissociation flame temps are
//!    monotone in f and stoich ~2375 K; deliberately HIGH vs the real ~2250 K because
//!    dissociation is not modelled yet. That gap is rung 6.

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{
    air_mole_fractions, antideriv_h, f_stoich, hf298, products_composition, species,
    Gas, HF298, HPR_MATTINGLY, M_CH2_KG, RU, T_REF,
};

/// A real (lossy, supersonic) design point, so the gates exercise the whole cycle.
fn flight() -> FlightCondition { FlightCondition::new(216.7, 18_750.0, 2.0) }

fn design() -> Losses {
    Losses {
        pi_d: 0.95, eta_c: 0.90, eta_b: 0.98, pi_b: 0.95, eta_t: 0.90, eta_m: 0.99, pi_n: 0.97,
        ..Losses::default()
    }
}

fn close(a: f64, b: f64, rel: f64, abs_: f64) -> bool {
    (a - b).abs() <= rel * b.abs() + abs_
}

/// The pinned fuel ΔHf298 that makes the derived LHV land on Mattingly's assumed 42.8 MJ/kg.
fn hf_fuel_default() -> f64 { HPR_MATTINGLY * M_CH2_KG + hf298("CO2") + hf298("H2O") }

// --------------------------------------------------------------------------------------- //
// GATE 1 — reduce-to-rung-4: Fork B == reacting Fork A, EXACTLY (the theorem).
// --------------------------------------------------------------------------------------- //
#[test]
fn forkb_equals_forka_exactly() {
    let ga = Gas::reacting_with(0.0, 42.8e6);      // rung-4 Fork A, assumed hPR
    let gb = Gas::reacting_forkb();                // rung-5 Fork B, DERIVED LHV (-> 42.8)
    let ra = build_turbojet(ga, 10.0, 1800.0, 18_750.0, design()).run(&flight(), 50.0);
    let rb = build_turbojet(gb, 10.0, 1800.0, 18_750.0, design()).run(&flight(), 50.0);
    // Machine-precision agreement on the load-bearing outputs.
    assert!(close(rb.station("4").far, ra.station("4").far, 1e-12, 1e-15));
    assert!(close(rb.station("5").tt, ra.station("5").tt, 1e-12, 0.0));
    assert!(close(rb.performance.specific_thrust, ra.performance.specific_thrust, 1e-12, 0.0));
    assert!(close(rb.performance.tsfc, ra.performance.tsfc, 1e-12, 0.0));
}

// --------------------------------------------------------------------------------------- //
// GATE 2 — the derived LHV reproduces Mattingly's assumed hPR = 42.8 MJ/kg.
// --------------------------------------------------------------------------------------- //
#[test]
fn derived_lhv_matches_mattingly_hpr() {
    let g = Gas::reacting_forkb();
    assert!(close(g.lhv(), 42.8e6, 1e-6, 0.0), "derived LHV {} != 42.8 MJ/kg", g.lhv());
    assert!(close(g.hpr(), g.lhv(), 1e-12, 0.0), "Fork B hPR slot must hold the derived LHV");
    // The pinned fuel enthalpy is ~ -35 kJ/mol (the advisor's prediction, at rung 5).
    let hf = hf_fuel_default();
    assert!((-36_000.0..-34_000.0).contains(&hf), "{hf}");
}

// --------------------------------------------------------------------------------------- //
// GATE 3 — formation self-check: H(298.15) = dHf; elements h = 0 at 298.15 K.
// --------------------------------------------------------------------------------------- //
#[test]
fn formation_self_check() {
    for &(sp, hf) in HF298 {
        let a_low = species(sp).a_low;
        let a6 = hf / RU - antideriv_h(&a_low, T_REF);               // derived formation const
        let h_abs = RU * (antideriv_h(&a_low, T_REF) + a6);          // absolute molar enthalpy
        assert!(close(h_abs, hf, 1e-9, 1e-6), "{sp}: H(298.15)={h_abs} != {hf}");
    }
    // Elements sit at zero (the absolute datum); CO2/H2O carry their negative formation.
    assert!(hf298("N2") == 0.0 && hf298("O2") == 0.0 && hf298("Ar") == 0.0);
    assert!(hf298("CO2") < 0.0 && hf298("H2O") < 0.0);
}

// --------------------------------------------------------------------------------------- //
// GATE 4 — absolute-balance closure + the fuel enthalpy is a LIVE knob.
// --------------------------------------------------------------------------------------- //
/// NB on what this assert can and cannot catch: the exact-equivalence theorem makes the
/// absolute balance ALGEBRAICALLY equal to the solver's Fork-A form (with `hPR := LHV`), so it
/// can never expose a SOLVER error. It guards only the absolute-interface plumbing (`h_t_abs`,
/// `hf_fuel_mass`, `formation_products_mass`) against sign/constant slips. That is its job; it
/// is not an independent energy check on the converged f.
#[test]
fn absolute_balance_closes_and_fuel_is_live_knob() {
    let g = Gas::reacting_forkb();
    // Re-close Σ N h(react) = Σ N h(prod) + loss at a hand set of states, mirroring the
    // burner's standing assert (eta_b = 1 => no loss term).
    let (tt3, tt4) = (800.0, 1600.0);
    let mut f = 0.0f64;
    for _ in 0..100 {                                                // rung-4 contraction
        let h4 = g.h_t(tt4, f);
        f = (h4 - g.h_c(tt3)) / (g.hpr() - h4);
    }
    let react = g.h_c_abs(tt3) + f * g.hf_fuel_mass();               // per kg air
    let prod = (1.0 + f) * g.h_t_abs(tt4, f);
    // Normalise by the SENSIBLE product enthalpy (the burner's own tolerance basis): the
    // absolute enthalpies are small — formation cancels most of the sensible part — so the
    // identity's ~1e-6 rounding lands relative to the ~1.9 MJ sensible scale.
    let scale = (1.0 + f) * g.h_t(tt4, f);
    assert!((react - prod).abs() <= 1e-6 * scale, "absolute balance open: {react} vs {prod}");

    // A LOWER-LHV fuel (a more negative formation enthalpy) needs MORE fuel for the same Tt4,
    // so f rises. This proves the derived heat release actually drives the burner.
    let g_lean = Gas::reacting_forkb_with(-50_000.0, 0.0);           // LHV ~ 41.7 MJ/kg
    assert!(g_lean.lhv() < Gas::reacting_forkb().lhv());
    let r_hi = build_turbojet(Gas::reacting_forkb(), 10.0, 1800.0, 18_750.0, design())
        .run(&flight(), 50.0);
    let r_lo = build_turbojet(g_lean, 10.0, 1800.0, 18_750.0, design())
        .run(&flight(), 50.0);
    assert!(r_lo.station("4").far > r_hi.station("4").far, "lower LHV must need more fuel");
}

// --------------------------------------------------------------------------------------- //
// GATE 5 — adiabatic flame temperature: physical plausibility (TEST-ONLY).
//
// The engine takes Tt4 as an INPUT; AFT is computed here only as the physical sanity anchor and
// the rung-6 motivation. It uses absolute enthalpies across the 1000 K join — the burner never
// needs the join, so this lives in the test.
// --------------------------------------------------------------------------------------- //
fn h_molar_abs(sp: &str, t: f64) -> f64 {
    let s = species(sp);
    let a6 = hf298(sp) / RU - antideriv_h(&s.a_low, T_REF);
    let sens = if t <= 1000.0 {
        antideriv_h(&s.a_low, t)
    } else {
        antideriv_h(&s.a_low, 1000.0) + antideriv_h(&s.a_high, t) - antideriv_h(&s.a_high, 1000.0)
    };
    RU * (sens + a6)
}

fn flame_temp(f: f64) -> f64 {
    let comp = products_composition(f);
    let x = air_mole_fractions();
    // `M_CH2_KG * 1000.0` is NOT bit-identical to `M_CH2`, so it is transcribed as written
    // rather than "simplified" — the same rule that governs every power in the port.
    let n_fuel = f * air_mass() / (M_CH2_KG * 1000.0);
    let mut h_react = 0.0f64;
    for &(s, xi) in x.iter() {
        h_react += xi * h_molar_abs(s, T_REF);
    }
    h_react += n_fuel * hf_fuel_default();
    let (mut lo, mut hi) = (300.0f64, 4000.0f64);
    for _ in 0..200 {
        let t = 0.5 * (lo + hi);
        let mut h_prod = 0.0f64;
        for &(s, n) in comp.iter() {
            h_prod += n * h_molar_abs(s, t);
        }
        if h_prod > h_react { hi = t; } else { lo = t; }
    }
    0.5 * (lo + hi)
}

/// Mean molar mass of dry air, g/mol — `gas.rs`'s `m_air()`, re-derived here so the test model
/// stays independent of the production one.
fn air_mass() -> f64 {
    air_mole_fractions().iter().map(|&(s, x)| x * species(s).m).sum()
}

#[test]
fn adiabatic_flame_temp_plausible() {
    let temps: Vec<f64> =
        [0.020, 0.030, 0.050, f_stoich() * 0.999].iter().map(|&f| flame_temp(f)).collect();
    // Monotone increasing with f.
    assert!(temps.windows(2).all(|w| w[1] > w[0]), "{temps:?}");
    // The stoichiometric no-dissociation value, in the right (deliberately high) band.
    let t_stoich = flame_temp(f_stoich() * 0.999);
    assert!((2300.0..2450.0).contains(&t_stoich), "stoich AFT {t_stoich} out of band");
    // It is HIGHER than the real dissociation-capped ~2250 K — that gap is rung 6.
    assert!(t_stoich > 2250.0);
}
