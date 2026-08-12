//! RUNG 6 — high-temperature dissociation and the chemical-equilibrium solve.
//!
//! Ported from `tests/test_rung6.py`. Gates (`docs/rung6-spec.md` § Verification gates), in
//! priority order:
//!
//! 1. REDUCE-TO-LOWER-RUNG (load-bearing) — `Gas::reacting_equilibrium` is a separate factory,
//!    so rungs 1-5 stay green untouched. Here: the ANTI-SEAM / reduce-to-rung-5 cold-`Tt4`
//!    limit — the rung-6 CYCLE f equals the rung-5 Fork-B f to ~1e-6 as `Tt4` drops
//!    (dissociation -> 0). A CONSTANT ~1 % offset would betray scale-A enthalpy leaking into
//!    the scale-B energy balance; instead the delta SHRINKS with dissociation.
//! 2. THE Kp / EQUILIBRIUM PHYSICS ANCHOR — methane-air stoichiometric equilibrium AFT in the
//!    CEA band (2226 K; ours 2231.7), and the `(p/p0)^dnu` factor live (pressure suppresses
//!    dissociation).
//! 3. FORMATION + ENTROPY SELF-CHECKS — `h(298.15) = dHf`, `s(298.15) = S298` per species.
//! 4. EQUILIBRIUM-AFT DROP (test-only, scale A) — (CH2)n stoichiometric AFT drops into the real
//!    ~2250 K band, below rung 5's no-dissociation value, monotone in f.
//! 5. STATION-4 DELTA BOUNDED + the whole cycle runs (asserts pass); the burn-config guard.
//!
//! GATE 1 IS THE ONE PHASE 1 COULD NOT SHIP. `gas_spine.rs` carries its gas-layer shadow — the
//! composition itself reducing as it cools — because the CYCLE form needs `build_turbojet`,
//! which did not exist until phase 2. It lands here, as `todo-rust-port.md` § 5.1 said it would.

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{
    a6_of, a7_of, air_mole_fractions, equil_solve, equilibrium_composition, f_stoich, h_molar_a,
    hf298, m_air, s298, s_molar, sens_h, sens_phi, Gas, HF298, HPR_MATTINGLY, M_CH2, M_CH2_KG,
    RU, T_REF,
};

/// A real (lossy, supersonic) design point — the same one `rung5.rs` uses — so the gates
/// exercise the whole cycle at station-4 pressure (~13 atm), where dissociation is doubly
/// suppressed.
fn flight() -> FlightCondition { FlightCondition::new(216.7, 18_750.0, 2.0) }

fn design() -> Losses {
    Losses {
        pi_d: 0.95, eta_c: 0.90, eta_b: 0.98, pi_b: 0.95, eta_t: 0.90, eta_m: 0.99, pi_n: 0.97,
        ..Losses::default()
    }
}

/// 1 atm, for the CEA anchor and the AFT diagnostics.
const P1: f64 = 101325.0;

fn close(a: f64, b: f64, rel: f64, abs_: f64) -> bool {
    (a - b).abs() <= rel * b.abs() + abs_
}

fn hf_fuel_default() -> f64 { HPR_MATTINGLY * M_CH2_KG + hf298("CO2") + hf298("H2O") }

fn get(comp: &[(&'static str, f64)], name: &str) -> f64 {
    comp.iter().find(|&&(s, _)| s == name).expect("species").1
}

/// The cycle's converged station-4 f at this `Tt4`, on whichever gas is passed.
fn far_at(gas: Gas, tt4: f64) -> f64 {
    build_turbojet(gas, 10.0, tt4, 18_750.0, design())
        .run(&flight(), 50.0)
        .station("4")
        .far
}

// --- AFT helpers (TEST-ONLY, SCALE A: the physically-correct formation datum that matches
//     CEA — the same two-context split rung 5 used) ---------------------------------------

/// Constant-p adiabatic flame temperature with dissociation, reactants at 298.15 K.
/// Air enthalpy at 298.15 is 0 on scale A (elements), so `H_react = n_fuel*hf_fuel`.
fn aft_equilibrium(b_c: f64, b_h: f64, n_o2: f64, hf_fuel: f64, p: f64) -> f64 {
    let x = air_mole_fractions();
    let xg = |n: &str| x.iter().find(|&&(s, _)| s == n).unwrap().1;
    let n_n2 = n_o2 * xg("N2") / xg("O2");
    let n_ar = n_o2 * xg("Ar") / xg("O2");
    let n_fuel = 1.0;
    let h_react = n_fuel * hf_fuel;
    let (mut lo, mut hi) = (1000.0f64, 3200.0f64);
    for _ in 0..100 {
        let t = 0.5 * (lo + hi);
        let comp = equil_solve(b_c, b_h, 2.0 * n_o2, n_n2 + n_ar, t, p);
        // Summed in SP_REACT order, then the inerts — the Python dict's iteration order.
        let mut h_prod = 0.0f64;
        for (j, &sp) in turbojet::gas::SP_REACT.iter().enumerate() {
            h_prod += comp[j] * h_molar_a(sp, t);
        }
        h_prod += n_n2 * h_molar_a("N2", t) + n_ar * h_molar_a("Ar", t);
        if h_prod > h_react { hi = t; } else { lo = t; }
    }
    0.5 * (lo + hi)
}

/// (CH2)n flame temperature per mol air. `dissociate = false` => complete combustion (rung 5).
///
/// NOTE THE SUM ORDER in the non-dissociating branch: the Python hand-builds the dict as
/// `{CO2, H2O, O2, N2, Ar}`, which is a DIFFERENT order from `products_composition`'s
/// `(N2, Ar, CO2, H2O, O2)`. Float addition is not associative, so the order is part of the
/// arithmetic and is transcribed as written rather than reused from the production helper.
fn aft_ch2(f: f64, p: f64, dissociate: bool) -> f64 {
    let x = air_mole_fractions();
    let xg = |n: &str| x.iter().find(|&&(s, _)| s == n).unwrap().1;
    let n_fuel = f * m_air() / M_CH2;
    let h_react = n_fuel * hf_fuel_default();
    let (mut lo, mut hi) = (800.0f64, 3200.0f64);
    for _ in 0..100 {
        let t = 0.5 * (lo + hi);
        let comp: Vec<(&'static str, f64)> = if dissociate {
            equilibrium_composition(f, t, p)
        } else {
            vec![("CO2", n_fuel), ("H2O", n_fuel), ("O2", xg("O2") - 1.5 * n_fuel),
                 ("N2", xg("N2")), ("Ar", xg("Ar"))]
        };
        let mut h_prod = 0.0f64;
        for &(s, n) in comp.iter() {
            h_prod += n * h_molar_a(s, t);
        }
        if h_prod > h_react { hi = t; } else { lo = t; }
    }
    0.5 * (lo + hi)
}

// --------------------------------------------------------------------------------------- //
// GATE 1 — anti-seam / reduce-to-rung-5 in the cold-Tt4 limit. THE CYCLE FORM.
// --------------------------------------------------------------------------------------- //
#[test]
fn reduce_to_rung5_cold_limit() {
    // Cold Tt4 -> dissociation ~ 0.
    let f_b = far_at(Gas::reacting_forkb(), 1000.0);
    let f_e = far_at(Gas::reacting_equilibrium(), 1000.0);
    assert!(close(f_e, f_b, 1e-6, 0.0), "cold-limit seam: equil {f_e} vs Fork B {f_b}");

    // And the delta SHRINKS with Tt4 — a scale leak would be a CONSTANT ~1 %. 1400 K > 1000 K.
    let f_b2 = far_at(Gas::reacting_forkb(), 1400.0);
    let f_e2 = far_at(Gas::reacting_equilibrium(), 1400.0);
    let (rel_cold, rel_warm) = ((f_e - f_b).abs() / f_b, (f_e2 - f_b2).abs() / f_b2);
    println!("seam delta: {rel_cold:.3e} at 1000 K, {rel_warm:.3e} at 1400 K");
    assert!(rel_warm > rel_cold, "seam delta must grow with Tt4, not be constant");
}

// --------------------------------------------------------------------------------------- //
// GATE 2 — methane-air stoichiometric equilibrium AFT vs CEA; pressure suppression.
// --------------------------------------------------------------------------------------- //
#[test]
fn methane_aft_equilibrium_anchor() {
    // CH4 + 2 O2 -> ... (C=1, H=4, nO2=2). CEA/Turns ~2226 K; ours ~2231.7 (NO/N deferred).
    let tf = aft_equilibrium(1.0, 4.0, 2.0, -74600.0, P1);
    assert!((2210.0..2245.0).contains(&tf), "CH4-air equilibrium AFT {tf} out of the CEA band");
}

#[test]
fn pressure_suppresses_dissociation() {
    // Stoichiometric (CH2)n at a fixed T = 2300 K: CO/(CO+CO2) must FALL as pressure rises,
    // which is the (p/p0)^dnu factor in Kp.
    let f = f_stoich() * 0.999;
    let fracs: Vec<f64> = [1.0, 5.0, 13.0].iter().map(|&p_atm| {
        let c = equilibrium_composition(f, 2300.0, p_atm * P1);
        get(&c, "CO") / (get(&c, "CO") + get(&c, "CO2"))
    }).collect();
    assert!(fracs[0] > fracs[1] && fracs[1] > fracs[2],
            "dissociation must fall with pressure: {fracs:?}");
    assert!(fracs[0] > 0.05,
            "1 atm stoich should show real CO dissociation, got {}", fracs[0]);
}

// --------------------------------------------------------------------------------------- //
// GATE 3 — formation + entropy self-checks (a6 from dHf, a7 from S298).
// --------------------------------------------------------------------------------------- //
#[test]
fn formation_and_entropy_self_check() {
    for &(sp, hf) in HF298 {
        // h(298.15) on scale A = dHf; s(298.15) = S298. Both exact by construction.
        let h_abs = RU * (sens_h(sp, T_REF) + a6_of(sp));
        let s_abs = s_molar(sp, T_REF);
        assert!(close(h_abs, hf, 1e-9, 1e-6), "{sp}: h(298)={h_abs} != {hf}");
        assert!(close(s_abs, s298(sp), 1e-9, 1e-6), "{sp}: s(298)={s_abs} != {}", s298(sp));
        // a7 is the entropy twin of a6, and is what makes the line above hold.
        let _ = (a7_of(sp), sens_phi(sp, T_REF));
    }
    // The five dissociation species carry the expected formation signs; H2 is an element.
    assert!(hf298("CO") < 0.0 && hf298("H2") == 0.0);
    assert!(hf298("OH") > 0.0 && hf298("O") > 0.0 && hf298("H") > 0.0);
}

// --------------------------------------------------------------------------------------- //
// GATE 4 — equilibrium-AFT drop: (CH2)n stoich into the real ~2250 K band.
// --------------------------------------------------------------------------------------- //
#[test]
fn equilibrium_aft_drop() {
    let fs = [0.020, 0.030, 0.050, f_stoich() * 0.999];
    let equil: Vec<f64> = fs.iter().map(|&f| aft_ch2(f, P1, true)).collect();
    let frozen: Vec<f64> = fs.iter().map(|&f| aft_ch2(f, P1, false)).collect();
    // Monotone in f.
    assert!(equil.windows(2).all(|w| w[1] > w[0]), "{equil:?}");
    // Stoich drops into the real band, strictly below rung 5's no-dissociation value.
    let (e_last, f_last) = (*equil.last().unwrap(), *frozen.last().unwrap());
    assert!((2250.0..2275.0).contains(&e_last), "stoich equilibrium AFT {e_last} out of band");
    assert!(e_last < f_last - 80.0,
            "dissociation must LOWER the stoich AFT: {e_last} vs {f_last}");
}

// --------------------------------------------------------------------------------------- //
// GATE 5 — station-4 delta bounded, the cycle runs, and the burn-config guard.
// --------------------------------------------------------------------------------------- //
#[test]
fn station4_delta_bounded_and_cycle_runs() {
    let r_b = build_turbojet(Gas::reacting_forkb(), 10.0, 1800.0, 18_750.0, design())
        .run(&flight(), 50.0);
    let r_e = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1800.0, 18_750.0, design())
        .run(&flight(), 50.0);
    let (f_b, f_e) = (r_b.station("4").far, r_e.station("4").far);
    // Bounded, not to the digit — Tt3 from the compressor plus eta_b = 0.98 drift it. The
    // equilibrium cycle needs slightly MORE fuel: dissociated products retain chemical enthalpy.
    let delta = (f_e - f_b) / f_b;
    assert!(delta > 0.0 && delta < 0.005,
            "station-4 delta {:.3} % out of bound", 100.0 * delta);
    // The frozen station-4 mixture actually dissociated a little (trace CO/OH present).
    let c = equilibrium_composition(f_e, 1800.0, r_e.station("4").pt);
    assert!(get(&c, "CO") > 0.0 && get(&c, "OH") > 0.0,
            "expected trace dissociation products at station 4");
}

/// Reusing ONE gas across two burn conditions with the SAME `far` but different `(Tt4, pt4)`
/// must trip the guard — no hidden state, a pure function of `far` per fixed burn config.
#[test]
#[should_panic(expected = "burn condition changed on a reused Gas")]
fn burn_config_guard() {
    let g = Gas::reacting_equilibrium();
    g.freeze_equilibrium(0.03, 1800.0, 1.3e6);
    g.freeze_equilibrium(0.03, 1700.0, 1.3e6);          // different Tt4
}

/// The other half of the guard: the SAME burn config may be re-frozen freely, and a second
/// `far` at that same config is fine. Without this, the `should_panic` above would also pass on
/// a guard that rejected every second call.
#[test]
fn burn_config_guard_permits_the_same_config() {
    let g = Gas::reacting_equilibrium();
    let a = g.freeze_equilibrium(0.03, 1800.0, 1.3e6);
    let b = g.freeze_equilibrium(0.03, 1800.0, 1.3e6);     // same config, same far: memoised
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x.1.to_bits(), y.1.to_bits(), "a memo hit must be bit-identical");
    }
    g.freeze_equilibrium(0.025, 1800.0, 1.3e6);           // same config, a different far
}
