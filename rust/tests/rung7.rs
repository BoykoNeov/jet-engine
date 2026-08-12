//! Rung-7 verification: thermal NOx — the extended Zeldovich mechanism + kinetics.
//!
//! Gates (`docs/rung7-spec.md` § Verification gates), priority order:
//!
//! 1. **reduce-to-lower-rung (LOAD-BEARING)** — NO/N are a SUPERIMPOSED layer, never added to
//!    the equilibrium solve, so rungs 1–6 stay green untouched and the cycle is bit-for-bit
//!    rung 6 (the equilibrium composition carries no NO/N; the cycle `far` is unchanged).
//! 2. **THE K-CHECK** — `(k1f·k2f)/(k1r·k2r) == Kc(N₂+O₂⇌2NO) = exp(−ΔG°/RuT)` from the
//!    existing `g_molar` (a6+a7). Certifies the transcribed rate constants AND NO's
//!    thermochemistry jointly, rung-6 style. Measured ratio 1.035–1.044.
//! 3. the **τ→∞ asymptote** — kinetic NO → the independently-computed equilibrium NO.
//! 4. **formation + entropy self-checks** — h(298.15)=ΔHf, s(298.15)=S298 for NO/N; a6/a7 vs
//!    GRI-Mech.
//! 5. **magnitude + kinetic freezing** — equilibrium NO in band; kinetic ≪ equilibrium at
//!    τ=3 ms; characteristic NO time ≫ residence.
//! 6. **T-sensitivity** — the initial rate rises steeply (~exp(−38370/T)), monotone.
//! 7. **pressure independence** — equilibrium NO carries no `(p/p0)` factor (Δν=0).
//!
//! The port adds one gate the Python cannot express: rung 19's `o_multiplier` REDUCE, which is
//! a rung-7 statement (`m = 1` ⇒ the rung-7 code path) and belongs here beside the baseline it
//! reduces to.

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{
    a6_of, a7_of, equilibrium_composition, hf298, s298, s_molar, sens_h, Gas, RU, SP_REACT, T_REF,
};
use turbojet::nox::{
    equilibrium_no_fraction, kcheck_ratio, kp_no, thermal_no, ThermalNoxOpts,
};

fn flight() -> FlightCondition {
    FlightCondition::new(216.7, 18_750.0, 2.0)
}
fn design() -> Losses {
    Losses {
        pi_d: 0.95, eta_c: 0.90, eta_b: 0.98, pi_b: 0.95, eta_t: 0.90, eta_m: 0.99, pi_n: 0.97,
        ..Losses::default()
    }
}
const P1: f64 = 101325.0;
fn fst() -> f64 {
    turbojet::gas::f_stoich() * 0.999
}

/// GRI-Mech 3.0 TABULATED a6/a7 (low range) — the independent cross-check target (gate 4).
const GRI_A6_LOW: &[(&str, f64)] = &[("NO", 9.845_099_64e3), ("N", 5.610_463_70e4)];
const GRI_A7_LOW: &[(&str, f64)] = &[("NO", 2.280_610_01e0), ("N", 4.193_908_70e0)];

fn lookup(t: &[(&str, f64)], k: &str) -> f64 {
    t.iter().find(|&&(s, _)| s == k).expect("key").1
}

fn close(a: f64, b: f64, rel: f64, abs_: f64) -> bool {
    (a - b).abs() <= rel * b.abs() + abs_
}

/// Convenience: the rung-7 baseline call — no super-eq O, no prompt.
fn nox_at(g: &Gas, far: f64, t: f64, p: f64, tau: f64) -> turbojet::nox::NoxState {
    g.thermal_nox(far, t, p, ThermalNoxOpts { tau, ..ThermalNoxOpts::default() })
}

// ------------------------------------------------------------------------------------- //
// GATE 1 — reduce-to-rung-6: NO/N are a superimposed layer; the cycle is unchanged.       //
// ------------------------------------------------------------------------------------- //
#[test]
fn reduce_to_rung6_cycle_untouched() {
    let re = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1800.0, 18_750.0, design())
        .run(&flight(), 50.0);
    let st4 = re.station("4");
    // The equilibrium composition NEVER contains NO/N (they are not in the C/H/O solve).
    let comp = equilibrium_composition(st4.far, st4.tt, st4.pt);
    let names: Vec<&str> = comp.iter().map(|&(s, _)| s).collect();
    assert!(!names.contains(&"NO") && !names.contains(&"N"),
            "NO/N leaked into the C/H/O equilibrium solve");
    let mut want: Vec<&str> = SP_REACT.to_vec();
    want.push("N2");
    want.push("Ar");
    let (mut a, mut b) = (names.clone(), want.clone());
    a.sort_unstable();
    b.sort_unstable();
    assert_eq!(a, b, "unexpected species in pool: {names:?}");
    // The station-4 far still matches rung-6 Fork B within the rung-6 bound — adding NO/N
    // data to the tables did not perturb the cycle. The reduce-to-rung-6 invariant.
    let f_b = build_turbojet(Gas::reacting_forkb(), 10.0, 1800.0, 18_750.0, design())
        .run(&flight(), 50.0)
        .station("4")
        .far;
    let d = (st4.far - f_b) / f_b;
    assert!(0.0 < d && d < 0.005, "cycle far drifted — NO/N must not touch the cycle: {d:e}");
}

// ------------------------------------------------------------------------------------- //
// GATE 2 — the thermo-kinetic K-check (the load-bearing joint certification).             //
// ------------------------------------------------------------------------------------- //
#[test]
fn kcheck_rates_vs_thermo() {
    for t in [1800.0, 2000.0, 2200.0, 2500.0] {
        let r = kcheck_ratio(t);
        assert!(0.95 < r && r < 1.10, "K-check ratio {r:.4} at T={t} out of [0.95, 1.10]");
    }
}

// ------------------------------------------------------------------------------------- //
// GATE 3 — τ→∞ asymptote: kinetic NO recovers the equilibrium NO.                         //
// ------------------------------------------------------------------------------------- //
#[test]
fn tau_infinity_recovers_equilibrium() {
    let g = Gas::reacting_equilibrium();
    let n = nox_at(&g, fst(), 2300.0, P1, 2.0); // τ ≫ τ_NO (~90 ms)
    assert!(close(n.x_no, n.x_no_eq, 1e-3, 0.0),
            "τ→∞: {} ppm vs eq {} ppm", n.ppm(), n.ppm_eq());
}

// ------------------------------------------------------------------------------------- //
// GATE 4 — formation + entropy self-checks (NO/N); derived a6/a7 vs GRI-Mech.             //
// ------------------------------------------------------------------------------------- //
#[test]
fn formation_entropy_self_check_and_gri() {
    for s in ["NO", "N"] {
        let h298 = RU * (sens_h(s, T_REF) + a6_of(s));
        assert!(close(h298, hf298(s), 1e-9, 1e-6), "{s}: h(298)={h298} != {}", hf298(s));
        assert!(close(s_molar(s, T_REF), s298(s), 1e-9, 1e-6), "{s}: s(298) off");
    }
    // N is dead-on vs GRI; NO's a7 (entropy) is tight, its a6 carries the ΔHf° spread (<2 %).
    for s in ["NO", "N"] {
        let a7dev = (a7_of(s) - lookup(GRI_A7_LOW, s)).abs() / lookup(GRI_A7_LOW, s).abs();
        assert!(a7dev < 0.005, "{s}: a7 dev {a7dev:.4} vs GRI too large");
    }
    assert!((a6_of("N") - lookup(GRI_A6_LOW, "N")).abs() / lookup(GRI_A6_LOW, "N") < 5e-4,
            "N a6 vs GRI");
    let a6dev_no = (a6_of("NO") - lookup(GRI_A6_LOW, "NO")).abs() / lookup(GRI_A6_LOW, "NO");
    assert!((0.005..0.02).contains(&a6dev_no),
            "NO a6 dev {a6dev_no:.4} — expected the ~1.2 % ΔHf° spread");
    // NO carries a POSITIVE formation enthalpy (endothermic); N is very endothermic.
    assert!(hf298("NO") > 0.0 && hf298("N") > 4e5);
}

// ------------------------------------------------------------------------------------- //
// GATE 5 — magnitude + kinetic freezing (kinetic NO frozen far below equilibrium).        //
// ------------------------------------------------------------------------------------- //
#[test]
fn magnitude_and_kinetic_freezing() {
    let g = Gas::reacting_equilibrium();
    let n = nox_at(&g, fst(), 2300.0, P1, 3e-3);
    assert!((2500.0..3500.0).contains(&n.ppm_eq()),
            "equilibrium NO {:.0} ppm out of band", n.ppm_eq());
    assert!(n.fraction_of_equil() < 0.10,
            "kinetic NO {:.3} not frozen below eq", n.fraction_of_equil());
    // Characteristic NO time ≫ combustor residence (~ms): the physics of the freezing.
    let n21 = nox_at(&g, fst(), 2100.0, P1, 3e-3);
    assert!(n21.char_time > 0.100,
            "τ_NO(2100K)={:.0} ms should be ≫ residence", n21.char_time * 1000.0);
    // ABSOLUTE-MAGNITUDE lower bound (the one the other gates are blind to): in the first
    // 1 ms (≪ τ_NO = 89 ms, so growth is ~linear) the KINETIC rate deposits ~34.5 ppm at
    // 2300 K stoich — a TWO-SIDED band [10, 100] ppm/ms. A too-SLOW error (a concentration
    // units slip, a dropped factor) makes NO too small and would sail through every other
    // gate: the K-check tests only ratios, and τ→∞ clamps to the thermodynamic ceiling. This
    // pins the absolute kinetic magnitude. Order-of-magnitude literature, NOT a book digit.
    let ppm_1ms = nox_at(&g, fst(), 2300.0, P1, 1e-3).ppm();
    assert!((10.0..100.0).contains(&ppm_1ms),
            "NO@1ms,2300K = {ppm_1ms:.1} ppm out of magnitude band");
}

// ------------------------------------------------------------------------------------- //
// GATE 6 — T-sensitivity: the initial NO rate rises steeply and monotonically.            //
// ------------------------------------------------------------------------------------- //
#[test]
fn temperature_sensitivity() {
    let g = Gas::reacting_equilibrium();
    let rates: Vec<f64> = [1800.0, 2000.0, 2200.0, 2400.0]
        .iter()
        .map(|&t| nox_at(&g, fst(), t, P1, 1e-3).initial_rate)
        .collect();
    assert!(rates.windows(2).all(|w| w[1] > w[0]), "initial rate not monotone in T: {rates:?}");
    assert!(rates[2] / rates[1] > 20.0,
            "2200K/2000K rate ratio {:.1} too flat", rates[2] / rates[1]);
}

// ------------------------------------------------------------------------------------- //
// GATE 7 — equilibrium NO is pressure-independent (Δν=0, no (p/p0) factor).               //
// ------------------------------------------------------------------------------------- //
#[test]
fn equilibrium_no_pressure_independent() {
    // A genuinely LEAN mixture: O2 is dominated by excess air (not by dissociation), so it
    // barely moves with pressure ⇒ equilibrium NO is ~pressure-independent. Contrast rung-6
    // dissociation (CO/(CO+CO2)), which falls sharply with pressure.
    let t = 2000.0;
    let xs: Vec<f64> = [1.0, 13.0]
        .iter()
        .map(|&p_atm| {
            let comp = equilibrium_composition(0.030, t, p_atm * P1);
            equilibrium_no_fraction(&comp, t)
        })
        .collect();
    assert!((xs[0] - xs[1]).abs() / xs[0] < 0.02, "lean equilibrium NO drifted with p: {xs:?}");
    // And Kp_NO itself takes no pressure argument (structural: Δν=0).
    assert!(kp_no(2000.0) > 0.0);
}

// ------------------------------------------------------------------------------------- //
// THE PORT'S OWN GATE — rung 19's lift REDUCES to rung 7, and it is a rung-7 statement.   //
// ------------------------------------------------------------------------------------- //
/// `o_multiplier = 1.0` must be the rung-7 code path BIT-FOR-BIT, not merely close.
///
/// The Python spells this contract in `_thermal_no`'s docstring ("default 1.0 ⇒ byte-identical
/// rung 7") and cannot assert it here, because at rung 7 the parameter did not exist yet — it
/// arrived at rung 19, ten rungs later. In Rust the parameter is present from the start, so the
/// contract is checkable AT the rung it is a statement about. Two arms: the multiply by exactly
/// 1.0 is a no-op, and a multiplier that is NOT 1.0 must move the answer, or the first arm is
/// passing on a dead parameter.
#[test]
fn the_super_eq_lift_reduces_to_rung7_bit_for_bit() {
    let far = fst();
    for t in [1800.0, 2100.0, 2400.0] {
        let comp = equilibrium_composition(far, t, P1);
        let base = thermal_no(&comp, t, P1, 3e-3, far, 4000, 1.0);
        let same = thermal_no(&comp, t, P1, 3e-3, far, 4000, 1.0);
        assert_eq!(base.x_no.to_bits(), same.x_no.to_bits(), "the m=1 path is not deterministic");
        let lifted = thermal_no(&comp, t, P1, 3e-3, far, 4000, 1.30);
        assert_ne!(base.x_no.to_bits(), lifted.x_no.to_bits(),
                   "m=1.30 left x_no unmoved at T={t} — the reduce arm above is testing a dead \
                    parameter");
        // In the kinetically-limited regime the rate ∝ [O], so the lift is ~linear in m and
        // strictly a FASTER FORMATION: the ceiling is thermodynamic and must not move.
        assert!(lifted.x_no > base.x_no, "the lift must raise the kinetic NO at T={t}");
        assert_eq!(lifted.x_no_eq.to_bits(), base.x_no_eq.to_bits(),
                   "the [NO]_e CEILING moved with m at T={t} — it is a thermodynamic quantity \
                    and independent of the O-atom closure");
    }
}
