//! The design-point run and rungs 2b–6's panels — `main.py`'s first eight steps.
//!
//! Each function is the `main.py` function of the same name (less the `print_` prefix), line
//! for line; the docstrings that carry each panel's lesson are not repeated here — they are in
//! `main.py` at the `python-final` tag and in each rung's spec.

use super::nox::{at, total};
use super::{Design, PI_C, TT4};
use crate::engine::{build_turbojet, EngineResult, Losses};
use crate::gas::{
    air_mole_fractions, equilibrium_composition, f_stoich, h_molar_a, hf_fuel_default, m_air,
    powp, products_composition, Gas, GasSpec, M_CH2,
};
use crate::pyf;
use crate::pyfmt::{py_dict, PyFormat, Printer};

const LABELS: [&str; 6] = ["0", "2", "3", "4", "5", "9"];

/// `print_station_table(title, result)`.
fn station_table(p: &mut Printer, title: &str, result: &EngineResult) {
    p.print(pyf!("\n{}", title));
    p.print(pyf!("{:>8} {:>10} {:>10} {:>9}", "Station", "Tt [K]", "pt [kPa]", "far"));
    p.print("-".repeat(40));
    for (label, s) in &result.stations {
        p.print(pyf!("{:>8} {:>10.1f} {:>10.2f} {:>9.5f}", *label, s.tt, s.pt / 1000.0, s.far));
    }
    let perf = &result.performance;
    p.print(pyf!("V0 = {:7.1f} m/s    V9 = {:7.1f} m/s    M9 = {:.3f}", result.v0, result.v9, result.m9));
    p.print(pyf!("Specific thrust = {:.1f} N·s/kg    TSFC = {:.3e} kg/(N·s)", perf.specific_thrust, perf.tsfc));
    p.print(pyf!("eta_brayton = {:.4f}   eta_thermal = {:.4f}   eta_p = {:.4f}   eta_o = {:.4f}",
                 perf.eta_brayton, perf.eta_thermal, perf.eta_propulsive, perf.eta_overall));
}

pub fn station_table_ideal(p: &mut Printer, d: &Design) {
    station_table(p, "IDEAL turbojet (rung-1 validation case)", &d.ideal);
}

pub fn station_table_real(p: &mut Printer, d: &Design) {
    station_table(p, "REAL components (same design point, with losses)", &d.real);
}

/// The line `main()` prints itself between the station tables and the rung-2b panel.
pub fn losses_cost(p: &mut Printer, d: &Design) {
    let (ri, rr) = (&d.ideal.performance, &d.real.performance);
    let df = 100.0 * (rr.specific_thrust / ri.specific_thrust - 1.0);
    let ds = 100.0 * (rr.tsfc / ri.tsfc - 1.0);
    p.print(pyf!("\nLosses cost: specific thrust {:+.1f}%, TSFC {:+.1f}% (less thrust, burned harder).", df, ds));
}

/// `print_polytropic_table(gas, flight)` — rung 2b: `η_c < e < η_t`.
pub fn polytropic_table(p: &mut Printer, d: &Design) {
    let (gas, flight) = (&d.gas, &d.flight);
    let (e, gc) = (0.9, gas.g_c());
    let poly = |pi_c: f64| {
        let l = Losses { e_c: Some(e), e_t: Some(e), ..Losses::default() };
        build_turbojet(gas.clone(), pi_c, TT4, flight.p0, l).run(flight, 1.0)
    };
    p.print("\nPolytropic knob (rung 2b): implied isentropic eta at e_c = e_t = 0.90");
    p.print(pyf!("{:>6} {:>8} {:>6} {:>8}    (eta_c < e < eta_t)", "pi_c", "eta_c", "e", "eta_t"));
    p.print("-".repeat(48));
    for pi_c in [2.0, 10.0, 30.0] {
        let eta_c = (powp(pi_c, gc) - 1.0) / (powp(pi_c, gc / e) - 1.0);
        let tau_t = poly(pi_c).station("5").tt / TT4;
        let eta_t = (1.0 - tau_t) / (1.0 - powp(tau_t, 1.0 / e));
        p.print(pyf!("{:>6.0f} {:>8.4f} {:>6.2f} {:>8.4f}", pi_c, eta_c, e, eta_t));
    }

    let eta_c = (powp(PI_C, gc) - 1.0) / (powp(PI_C, gc / e) - 1.0);
    let rp = poly(PI_C);
    let tau_t = rp.station("5").tt / TT4;
    let eta_t = (1.0 - tau_t) / (1.0 - powp(tau_t, 1.0 / e));
    let iso = build_turbojet(gas.clone(), PI_C, TT4, flight.p0,
                             Losses { eta_c, eta_t, ..Losses::default() }).run(flight, 1.0);
    let df = (rp.performance.specific_thrust - iso.performance.specific_thrust).abs();
    p.print(pyf!("At pi_c={:.0f}, e=0.90 implies eta_c={:.4f}, eta_t={:.4f}; the converted-eta engine\n\
                  agrees on specific thrust to {:.0e} N·s/kg — one machine, two knobs.", PI_C, eta_c, eta_t, df));
}

/// The two-run, side-by-side station rows rungs 3–6 print (`{label:>8} {Tt…} {Tt…}  {pt…} {pt…}`),
/// with the column widths each panel uses.
fn paired_rows(p: &mut Printer, a: &EngineResult, b: &EngineResult, wide: bool) {
    for label in LABELS {
        let (c, t) = (a.station(label), b.station(label));
        p.print(if wide {
            pyf!("{:>8} {:>10.1f} {:>9.1f}  {:>10.2f} {:>9.2f}", label, c.tt, t.tt, c.pt / 1000.0, t.pt / 1000.0)
        } else {
            pyf!("{:>8} {:>9.1f} {:>9.1f}  {:>9.2f} {:>9.2f}", label, c.tt, t.tt, c.pt / 1000.0, t.pt / 1000.0)
        });
    }
}

/// `print_variable_cp_table(flight)` — rung 3: frozen cp vs `cp(T)`.
pub fn variable_cp_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    let frozen_gas = Gas::new(GasSpec { gamma_t: 1.3, cp_t: 1239.0, r_t: 285.9, ..GasSpec::default() });
    let vary_gas = Gas::thermally_perfect();
    let frozen = build_turbojet(frozen_gas, PI_C, TT4, flight.p0, Losses::default()).run(flight, 1.0);
    let ev = build_turbojet(vary_gas, PI_C, TT4, flight.p0, Losses::default());
    let vary = ev.run(flight, 1.0);
    let vary_gas = &ev.gas;

    p.print("\nVariable cp(T) (rung 3): rung-2 frozen cp vs thermally-perfect cp(T), same design point");
    p.print(pyf!("{:>8} {:>10} {:>9}  {:>10} {:>9}", "Station", "Tt frozen", "Tt cp(T)", "pt frozen", "pt cp(T)"));
    p.print("-".repeat(52));
    paired_rows(p, &frozen, &vary, true);
    let (cp_c3, cp_c8) = (vary_gas.cp_c_at(300.0), vary_gas.cp_c_at(800.0));
    let (cp_t3, cp_t15) = (vary_gas.cp_t_at(300.0, 0.0), vary_gas.cp_t_at(1500.0, 0.0));
    let avg_cp_t = vary_gas.h_t(TT4, 0.0) / TT4;
    p.print(pyf!("cp(T) varies (rung 1-2 froze it): cold air {:.0f}->{:.0f} over 300->800 K; \
                  hot products {:.0f}->{:.0f} over 300->1500 K.", cp_c3, cp_c8, cp_t3, cp_t15));
    p.print(pyf!("Gas-table effect at pi_c={:.0f}: Tt3 {:.1f} -> {:.1f} K (cooler).",
                 PI_C, frozen.station("3").tt, vary.station("3").tt));
    p.print(pyf!("Frozen cp_t=1239 vs true burner-average {:.0f} J/(kg·K) -> far {:.5f} -> {:.5f} (less fuel), \
                  F/mdot {:.1f} -> {:.1f} N·s/kg.",
                 avg_cp_t, frozen.station("4").far, vary.station("4").far,
                 frozen.performance.specific_thrust, vary.performance.specific_thrust));
}

/// `print_reacting_table(flight)` — rung 4: composition tracks `f`.
pub fn reacting_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    let rf = build_turbojet(Gas::thermally_perfect(), PI_C, TT4, flight.p0, Losses::default()).run(flight, 1.0);
    let er = build_turbojet(Gas::reacting(), PI_C, TT4, flight.p0, Losses::default());
    let rr = er.run(flight, 1.0);
    let react = &er.gas;

    p.print("\nReacting products (rung 4): rung-3 frozen composition vs composition(f), same design point");
    p.print(pyf!("{:>8} {:>10} {:>9}  {:>10} {:>9}", "Station", "Tt frozen", "Tt react", "pt frozen", "pt react"));
    p.print("-".repeat(52));
    paired_rows(p, &rf, &rr, true);
    p.print(pyf!("far: frozen-composition {:.5f} -> reacting {:.5f}; F/mdot {:.1f} -> {:.1f} N·s/kg.",
                 rf.station("4").far, rr.station("4").far,
                 rf.performance.specific_thrust, rr.performance.specific_thrust));

    p.print("\nf-sweep (rung 4): composition and cp_t track the fuel/air ratio (Tt4 drives f)");
    p.print(pyf!("{:>8} {:>8} {:>9} {:>7} {:>7} {:>7} {:>7} {:>8}",
                 "Tt4 [K]", "far", "cp_t@Tt4", "CO2 %", "H2O %", "O2 %", "R_t", "F/mdot"));
    p.print("-".repeat(68));
    for tt4 in [1200.0, 1400.0, 1600.0, 1800.0] {
        let r = build_turbojet(react.clone(), PI_C, tt4, flight.p0, Losses::default()).run(flight, 1.0);
        let f = r.station("4").far;
        let comp = products_composition(f);
        let tot = total(&comp);
        p.print(pyf!("{:>8.0f} {:>8.5f} {:>9.1f} {:>7.3f} {:>7.3f} {:>7.3f} {:>7.2f} {:>8.1f}",
                     tt4, f, react.cp_t_at(tt4, f),
                     100.0 * at(&comp, "CO2") / tot, 100.0 * at(&comp, "H2O") / tot,
                     100.0 * at(&comp, "O2") / tot, react.r_t_at(f),
                     r.performance.specific_thrust));
    }
}

/// `print_forkb_table(flight)` — rung 5: the heat release DERIVED.
pub fn forkb_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    let ea = build_turbojet(Gas::reacting_with(0.0, 42.8e6), PI_C, TT4, flight.p0, Losses::default());
    let ra = ea.run(flight, 1.0);
    let eb = build_turbojet(Gas::reacting_forkb(), PI_C, TT4, flight.p0, Losses::default());
    let rb = eb.run(flight, 1.0);
    let (fa, fb) = (&ea.gas, &eb.gas);

    p.print("\nFork B (rung 5): assumed hPR (Fork A) vs DERIVED heat release, same design point");
    p.print(pyf!("  hPR: Fork A assumes {:.4f} MJ/kg  ->  Fork B DERIVES {:.4f} MJ/kg from formation enthalpies \
                  (fuel ΔHf = {:.2f} kJ/mol)", fa.hpr() / 1e6, fb.lhv() / 1e6, fb.hf_fuel_molar() / 1000.0));
    p.print(pyf!("{:>8} {:>9} {:>9}  {:>9} {:>9}", "Station", "Tt A", "Tt B", "pt A", "pt B"));
    p.print("-".repeat(50));
    paired_rows(p, &ra, &rb, false);
    let df = (ra.station("4").far - rb.station("4").far).abs();
    p.print(pyf!("far: Fork A {:.6f} vs Fork B {:.6f} (|Δ| = {:.1e} — EXACT: released energy ≡ f·LHV for complete combustion).",
                 ra.station("4").far, rb.station("4").far, df));
    p.print("  Fork B buys structure, not digits: absolute-enthalpy scale for rung-6 \
             dissociation, and heat release that will track composition. Products carry");
    let f4 = rb.station("4").far;
    p.print(pyf!("  formation enthalpy {:.3f} MJ/kg (vs air 0.000), so absolute h_t(Tt4) = {:.3f} \
                  MJ/kg sits below sensible {:.3f} MJ/kg.",
                 fb.hf_products_mass(f4) / 1e6, fb.h_t_abs(TT4, f4) / 1e6, fb.h_t(TT4, f4) / 1e6));
}

/// `_aft_ch2(f, p, dissociate)` — the (CH2)n constant-p adiabatic flame temperature per mol air,
/// SCALE A, by 100 bisection steps on `[800, 3200]` K.
fn aft_ch2(f: f64, p: f64, dissociate: bool) -> f64 {
    let x = air_mole_fractions();
    let xg = |name: &str| x.iter().find(|&&(s, _)| s == name).unwrap().1;
    let n_fuel = f * m_air() / M_CH2;
    let h_react = n_fuel * hf_fuel_default();
    let (mut lo, mut hi) = (800.0, 3200.0);
    for _ in 0..100 {
        let t = 0.5 * (lo + hi);
        let comp: Vec<(&'static str, f64)> = if dissociate {
            equilibrium_composition(f, t, p)
        } else {
            vec![("CO2", n_fuel), ("H2O", n_fuel), ("O2", xg("O2") - 1.5 * n_fuel),
                 ("N2", xg("N2")), ("Ar", xg("Ar"))]
        };
        let h_prod = comp.iter().fold(0.0, |a, &(s, n)| a + n * h_molar_a(s, t));
        if h_prod > h_react { hi = t } else { lo = t }
    }
    0.5 * (lo + hi)
}

/// `print_equilibrium_table(flight)` — rung 6: dissociation.
pub fn equilibrium_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    let rb = build_turbojet(Gas::reacting_forkb(), PI_C, TT4, flight.p0, Losses::default()).run(flight, 1.0);
    let re = build_turbojet(Gas::reacting_equilibrium(), PI_C, TT4, flight.p0, Losses::default()).run(flight, 1.0);

    p.print("\nEquilibrium (rung 6): Fork B (complete) vs dissociating products, same design point");
    p.print(pyf!("{:>8} {:>9} {:>9}  {:>9} {:>9}", "Station", "Tt B", "Tt eq", "pt B", "pt eq"));
    p.print("-".repeat(50));
    paired_rows(p, &rb, &re, false);
    let (f_b, f_e) = (rb.station("4").far, re.station("4").far);
    let pt4 = re.station("4").pt;
    p.print(pyf!("far: Fork B {:.6f} -> equilibrium {:.6f} (+{:.3f}%) — a tiny \
                  correction: at pt4={:.1f} bar, lean, dissociation is doubly suppressed.",
                 f_b, f_e, 100.0 * (f_e - f_b) / f_b, pt4 / 1e5));
    let comp = equilibrium_composition(f_e, TT4, pt4);
    let tot = total(&comp);
    let keys = ["CO", "OH", "O", "H", "H2"];
    let vals: Vec<String> = keys.iter().map(|s| pyf!("{:.4f}%", 100.0 * at(&comp, s) / tot)).collect();
    let items: Vec<(&dyn PyFormat, &dyn PyFormat)> =
        keys.iter().zip(&vals).map(|(k, v)| (k as &dyn PyFormat, v as &dyn PyFormat)).collect();
    p.print(pyf!("  station-4 dissociation products (frozen downstream): {}", py_dict(&items)));

    p.print("\n  Adiabatic flame temperature (the diagnostic that finally drops), 1 atm:");
    p.print(pyf!("  {:>8} {:>18} {:>20} {:>8}", "f", "no-dissoc (rung 5)", "equilibrium (rung 6)", "drop K"));
    for f in [0.030, 0.050, f_stoich() * 0.999] {
        let tf = aft_ch2(f, 101325.0, false);
        let te = aft_ch2(f, 101325.0, true);
        let tag = if f > 0.06 { "  (≈stoich)" } else { "" };
        p.print(pyf!("  {:>8.4f} {:>18.1f} {:>20.1f} {:>8.1f}{}", f, tf, te, tf - te, tag));
    }
    p.print("  Stoich ~2259 K lands in the real kerosene-air band; the ~115 K gap IS \
             dissociation (endothermic), the rung-5 AFT overshoot explained.");

    p.print("\n  The Kp (p/p0)^Δν factor, live — dissociation falls as pressure rises \
             (stoich, T=2300 K):");
    let f = f_stoich() * 0.999;
    for p_atm in [1.0, 5.0, 13.0] {
        let c = equilibrium_composition(f, 2300.0, p_atm * 101325.0);
        p.print(pyf!("    p={:>5.1f} atm:  CO/(CO+CO2) = {:.4f}", p_atm, at(&c, "CO") / (at(&c, "CO") + at(&c, "CO2"))));
    }
}
