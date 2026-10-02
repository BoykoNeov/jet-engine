//! Rungs 7–14's panels — the NOx and mixing strand's first half (slice AL).
//!
//! Each function is the `main.py` function of the same name (less the `print_` prefix), line
//! for line. Every panel that needs the design point builds its own equilibrium engine, as the
//! Python does, and calls the diagnostics on THAT engine's gas (`e.gas`) — the object the burner
//! froze, which is the one the Python calls them on.

use super::{real_losses, Design, PI_C, TT4};
use crate::engine::{build_turbojet, Engine, EngineResult, Losses};
use crate::gas::{equilibrium_composition, f_stoich, hf_fuel_default, Gas};
use crate::nox::{
    beta_pdf_nodes_weights, bell_interpolator, equilibrium_no_fraction, primary_aft, quench_no,
    quench_trajectory, thermal_no, JetMixing, MixingPdf, QuenchOpts, QuenchPoint, ThermalNoxOpts,
    Unmixedness, ZonedNoxOpts,
};
use crate::pyf;
use crate::pyfmt::Printer;

/// `sum(comp.values())` — a left fold in the composition's own (Python dict) order.
pub(crate) fn total(comp: &[(&'static str, f64)]) -> f64 {
    comp.iter().fold(0.0, |a, &(_, v)| a + v)
}

/// `comp['X']` for a composition kept as Python's ordered dict.
pub(crate) fn at(comp: &[(&'static str, f64)], key: &str) -> f64 {
    comp.iter().find(|&&(s, _)| s == key).unwrap_or_else(|| panic!("no species {key:?}")).1
}

/// Python's `min(xs, key=k)`: the FIRST element whose key is smallest (a later tie never wins).
pub(crate) fn py_argmin(keys: &[f64]) -> usize {
    let mut best = 0;
    for i in 1..keys.len() {
        if keys[i] < keys[best] {
            best = i;
        }
    }
    best
}

/// Python's `max(xs)` over floats: the first maximum (`x > best` replaces).
pub(crate) fn py_max(xs: impl IntoIterator<Item = f64>) -> f64 {
    let mut it = xs.into_iter();
    let mut best = it.next().expect("max() of an empty sequence");
    for x in it {
        if x > best {
            best = x;
        }
    }
    best
}

/// `"-" * n`.
pub(crate) fn dashes(n: usize) -> String { "-".repeat(n) }

/// `eq = Gas.reacting_equilibrium(); real = build_turbojet(eq, …, **REAL_LOSSES).run(flight, 1.0)`
/// — the engine (whose gas is the object the Python goes on to call) and its run.
pub(crate) fn real_eq(d: &Design) -> (Engine, EngineResult) {
    let e = build_turbojet(Gas::reacting_equilibrium(), PI_C, TT4, d.flight.p0, real_losses());
    let r = e.run(&d.flight, 1.0);
    (e, r)
}

/// `Tt3, Tt4, far, p = st3.Tt, st4.Tt, st4.far, st4.pt`.
pub(crate) fn design_quad(r: &EngineResult) -> (f64, f64, f64, f64) {
    let (st3, st4) = (r.station("3"), r.station("4"));
    (st3.tt, st4.tt, st4.far, st4.pt)
}

/// `hf = eq.hf_fuel_molar if eq.hf_fuel_molar is not None else _HF_FUEL_DEFAULT`.
pub(crate) fn hf_of(g: &Gas) -> f64 {
    g.spec.hf_fuel_molar.unwrap_or_else(hf_fuel_default)
}

/// The "Design point: Tt3=…" line rungs 8–24 share (`φ` from `far/_F_STOICH`).
pub(crate) fn design_line(tt3: f64, tt4: f64, far: f64, p: f64, tail: &str) -> String {
    pyf!("  Design point: Tt3={:.0f} K, Tt4={:.0f} K, p={:.1f} bar, overall far={:.4f} (φ={:.2f}{})",
         tt3, tt4, p / 1e5, far, far / f_stoich(), tail)
}

/// The rich-primary (φ_p = 1.5) setup rungs 10 and 11 build once and reuse: `(T_p, comp_p,
/// alpha, n0, the rung-9 primary NoxState, tab)`.
pub(crate) struct RichPrimary {
    pub t_p: f64,
    pub comp_p: Vec<(&'static str, f64)>,
    pub alpha: f64,
    pub n0: f64,
    pub ei9: f64,
    pub tab: Vec<QuenchPoint>,
}

pub(crate) fn rich_primary(phi_p: f64, far: f64, tt3: f64, p: f64, hf: f64, ng: usize) -> RichPrimary {
    let far_p = phi_p * f_stoich();
    let alpha = far / far_p;
    let t_p = primary_aft(far_p, p, tt3, hf);
    let comp_p = equilibrium_composition(far_p, t_p, p);
    let nox = thermal_no(&comp_p, t_p, p, 3e-3, far_p, 4000, 1.0);
    let n0 = alpha * nox.x_no * total(&comp_p);
    let tab = quench_trajectory(&comp_p, t_p, alpha, far, tt3, p, ng);
    RichPrimary { t_p, comp_p, alpha, n0, ei9: nox.ei_no, tab }
}

/// `print_nox_table(flight)` — rung 7: thermal NOx, kinetically limited.
pub fn nox_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    let eq = Gas::reacting_equilibrium();
    let fst = f_stoich() * 0.999;
    let tau = 3e-3;
    let opts = |tau: f64| ThermalNoxOpts { tau, ..ThermalNoxOpts::default() };

    p.print("\nThermal NOx (rung 7): kinetically-limited NO — the lesson inverts rung 6");
    p.print(pyf!("  Stoich (CH2)n flame sweep, 1 atm, residence tau = {:.0f} ms:", tau * 1e3));
    p.print(pyf!("  {:>7} {:>9} {:>10} {:>11} {:>9} {:>10} {:>9}",
                 "T [K]", "x_O ppm", "NO_eq ppm", "NO_kin ppm", "kin/eq %", "tau_NO ms", "rate rel"));
    p.print(format!("  {}", dashes(70)));
    let rate_ref = eq.thermal_nox(fst, 2000.0, 101325.0, opts(tau)).initial_rate;
    for t in [1800.0, 2000.0, 2200.0, 2400.0] {
        let n = eq.thermal_nox(fst, t, 101325.0, opts(tau));
        let comp = equilibrium_composition(fst, t, 101325.0);
        let x_o = at(&comp, "O") / total(&comp) * 1e6;
        p.print(pyf!("  {:>7.0f} {:>9.1f} {:>10.0f} {:>11.2f} {:>9.2f} {:>10.1f} {:>9.3f}",
                     t, x_o, n.ppm_eq(), n.ppm(), 100.0 * n.fraction_of_equil(),
                     n.char_time * 1e3, n.initial_rate / rate_ref));
    }
    p.print("  NO is frozen at a few % of equilibrium (tau_NO >> ms residence); the initial");
    p.print("  rate explodes ~30x per 200 K — it is PEAK FLAME TEMPERATURE, not the capped");
    p.print("  mixed-out Tt4, that governs NOx (why NOx, not just blade metal, caps the flame).");

    p.print("\n  Kinetic NO climbs toward equilibrium as residence grows (stoich, 2300 K, 1 atm):");
    p.print(pyf!("  {:>9} {:>11} {:>11}", "tau [ms]", "NO_kin ppm", "% of equil"));
    for tau_ms in [0.5, 1.0, 3.0, 10.0, 100.0, 1000.0] {
        let n = eq.thermal_nox(fst, 2300.0, 101325.0, opts(tau_ms * 1e-3));
        p.print(pyf!("  {:>9.1f} {:>11.1f} {:>11.2f}", tau_ms, n.ppm(), 100.0 * n.fraction_of_equil()));
    }

    let e = build_turbojet(eq, PI_C, TT4, flight.p0, Losses::default());
    let re = e.run(flight, 1.0);
    let st4 = re.station("4");
    let n4 = e.gas.thermal_nox(st4.far, st4.tt, st4.pt, opts(tau));
    p.print(pyf!("\n  Station 4 (this design point: Tt4={:.0f} K, {:.1f} bar, lean far={:.4f}): equilibrium NO \
                  {:.0f} ppm, but kinetic NO only {:.2e} ppm\n  ({:.4f}% of equil, EI_NO={:.2e} \
                  g/kg) — too cool AND kinetically frozen. Real NOx is a hot primary-zone effect.",
                 st4.tt, st4.pt / 1e5, st4.far, n4.ppm_eq(), n4.ppm(), 100.0 * n4.fraction_of_equil(), n4.ei_no));

    p.print("\n  Contrast rung 6: equilibrium NO carries NO (p/p0) factor (Dnu=0). At a LEAN \
             f=0.030, 2000 K,");
    let xs: Vec<f64> = [1.0, 13.0].iter()
        .map(|&pa| equilibrium_no_fraction(&equilibrium_composition(0.030, 2000.0, pa * 101325.0), 2000.0))
        .collect();
    p.print(pyf!("  NO_eq is {:.0f} ppm at 1 atm vs {:.0f} ppm at 13 atm — high \
                  combustor pressure does NOT directly suppress NOx.", xs[0] * 1e6, xs[1] * 1e6));
}

/// `print_zoning_table(flight)` — rung 8: two-zone combustor.
pub fn zoning_table(p: &mut Printer, d: &Design) {
    let (e, real) = real_eq(d);
    let eq = &e.gas;
    let (tt3, tt4, far, pp) = design_quad(&real);
    let tau = 3e-3;
    let zo = |phi: f64| eq.zoned_nox(far, tt3, tt4, pp, phi, ZonedNoxOpts { tau, ..ZonedNoxOpts::default() });

    let n_mixed = eq.thermal_nox(far, tt4, pp, ThermalNoxOpts { tau, ..ThermalNoxOpts::default() });
    p.print("\nCombustor zoning (rung 8): the primary-zone NOx effect — completes rung 7");
    p.print(pyf!("  Design point: Tt3={:.0f} K, Tt4={:.0f} K, p={:.1f} bar, overall far={:.4f} (φ={:.2f}), τ={:.0f} ms",
                 tt3, tt4, pp / 1e5, far, far / f_stoich(), tau * 1e3));
    p.print(pyf!("  Mixed-out station 4 (rung 7): EI_NO = {:.2e} g/kg — essentially zero.", n_mixed.ei_no));
    p.print(pyf!("  {:>5} {:>7} {:>7} {:>7} {:>8} {:>9} {:>8} {:>7} {:>8}",
                 "φ_p", "α_air", "far_p", "AFT K", "NO_eq", "NO_kin", "EI_NO", "T_mix", "NO_mix"));
    p.print(pyf!("  {:>5} {:>7} {:>7} {:>7} {:>8} {:>9} {:>8} {:>7} {:>8}",
                 "", "", "", "(prim)", "ppm", "ppm", "g/kg", "K", "ppm"));
    p.print(format!("  {}", dashes(74)));
    for phi_p in [0.7, 0.8, 0.9, 1.0] {
        let z = zo(phi_p);
        p.print(pyf!("  {:>5.2f} {:>7.4f} {:>7.5f} {:>7.0f} {:>8.0f} {:>9.1f} {:>8.2f} {:>7.0f} {:>8.0f}",
                     phi_p, z.alpha, z.far_primary, z.t_primary, z.primary.ppm_eq(), z.ppm_primary(),
                     z.ei_no(), z.t_mix, z.ppm_mix()));
    }
    let z1 = zo(1.0);
    let z07 = zo(0.7);
    p.print(pyf!("  Primary φ_p 0.7→1.0 lifts the AFT {:.0f}→{:.0f} K (+{:.0f} K) and swings EI_NO {:.0f}× \
                  — the rung-7 exp-in-T rate showing through.",
                 z07.t_primary, z1.t_primary, z1.t_primary - z07.t_primary, z1.ei_no() / z07.ei_no()));
    p.print(pyf!("  At φ_p≈1 EI_NO ≈ {:.0f} g/kg (ICAO take-off band 18–64), vs the mixed-out \
                  {:.0e} — a ~{:.0f}-order lift.",
                 z1.ei_no(), n_mixed.ei_no, (z1.ei_no() / n_mixed.ei_no).log10()));
    p.print("  T_mix is IDENTICAL across the sweep and returns to ≈Tt4 (majors re-equilibrate on");
    p.print("  dilution); NO_mix < NO_kin (fraction diluted) but EI_NO is conserved (moles fixed).");
    p.print("  The capped mixed-out turbine inlet never made the NO — a hot zone you averaged");
    p.print("  away did. THIS is why real combustors fight peak flame T (lean-premixed, RQL).");
}

/// `print_rql_table(flight)` — rung 9: the rich flank of the bell.
pub fn rql_table(p: &mut Printer, d: &Design) {
    let (e, real) = real_eq(d);
    let eq = &e.gas;
    let (tt3, tt4, far, pp) = design_quad(&real);
    let tau = 3e-3;
    let zo = |phi: f64| eq.zoned_nox(far, tt3, tt4, pp, phi, ZonedNoxOpts { tau, ..ZonedNoxOpts::default() });

    p.print("\nRich primary / RQL (rung 9): the rich flank of the NOx bell — completes rung 7's");
    p.print("inversion on the OTHER side. EI_NO peaks near stoich, then falls as the primary goes rich.");
    p.print(pyf!("  Design point: Tt3={:.0f} K, Tt4={:.0f} K, p={:.1f} bar, overall far={:.4f} (φ={:.2f}), τ={:.0f} ms",
                 tt3, tt4, pp / 1e5, far, far / f_stoich(), tau * 1e3));
    p.print(pyf!("  {:>5} {:>7} {:>7} {:>7} {:>8} {:>9} {:>8} {:>7}",
                 "φ_p", "AFT K", "xCO", "xH2", "NO_eq", "NO_kin", "EI_NO", "T_mix"));
    p.print(pyf!("  {:>5} {:>7} {:>7} {:>7} {:>8} {:>9} {:>8} {:>7}",
                 "", "(prim)", "%", "%", "ppm", "ppm", "g/kg", "K"));
    p.print(format!("  {}", dashes(63)));
    let mut best = (0.0, 0.0);
    for phi_p in [0.8, 0.9, 1.0, 1.05, 1.1, 1.3, 1.5, 1.8, 2.0] {
        let z = zo(phi_p);
        let comp = equilibrium_composition(z.far_primary, z.t_primary, pp);
        let nt = total(&comp);
        if z.ei_no() > best.1 {
            best = (phi_p, z.ei_no());
        }
        let flank = if phi_p == 1.0 { "  <- peak" } else if phi_p >= 1.3 { "  rich, low-NOx" } else { "" };
        p.print(pyf!("  {:>5.2f} {:>7.0f} {:>7.2f} {:>7.2f} {:>8.0f} {:>9.1f} {:>8.3f} {:>7.0f}{}",
                     phi_p, z.t_primary, 100.0 * at(&comp, "CO") / nt, 100.0 * at(&comp, "H2") / nt,
                     z.primary.ppm_eq(), z.ppm_primary(), z.ei_no(), z.t_mix, flank));
    }
    let z_stoich = zo(1.0);
    let z_rich = zo(1.4);
    p.print(pyf!("  Peak EI_NO ≈ {:.0f} g/kg near φ_p≈{:.2f} (ICAO band); a RICH primary \
                  φ_p=1.4 cuts it to {:.2f} g/kg", best.1, best.0, z_rich.ei_no()));
    p.print(pyf!("  — {:.0f}× lower — even though it still burns ALL the fuel. The rich pool's",
                 z_stoich.ei_no() / z_rich.ei_no()));
    p.print("  CO/H2 (major, unoxidized) + rolled-over AFT starve the O/OH the Zeldovich rate needs.");
    p.print("  THAT is why RQL burns rich, then quick-quenches PAST stoich (the NO peak) to lean.");
    p.print("  T_mix still returns to ≈Tt4 for every φ_p — the CO/H2 oxidation energy releases on");
    p.print("  re-equilibration (mix-out is the ideal, infinitely-fast quench; NO frozen).");
}

/// `print_finite_quench_table(flight)` — rung 10: the finite-rate quench.
pub fn finite_quench_table(p: &mut Printer, d: &Design) {
    let (e, real) = real_eq(d);
    let eq = &e.gas;
    let (tt3, tt4, far, pp) = design_quad(&real);
    let hf = hf_of(eq);
    let ng = 80;

    p.print("\nFinite-rate quench (rung 10): the RQL hazard — NO re-made as the gas dwells at stoich");
    p.print("while the quench air mixes in. Rung 9's rich-flank collapse holds ONLY if the quench is fast.");
    p.print(design_line(tt3, tt4, far, pp, ""));

    let phi_p = 1.5;
    let rp = rich_primary(phi_p, far, tt3, pp, hf, ng);
    let t_peak = py_max(rp.tab.iter().map(|r| r.t));
    let ei9 = rp.ei9;
    p.print(pyf!("\n  A rich primary φ_p={}: AFT={:.0f} K RISES to a stoich peak of {:.0f} K", phi_p, rp.t_p, t_peak));
    p.print(pyf!("  as it quenches (rung-9 ideal-quench EI_NO = {:.4f} g/kg — NO frozen at the primary).", ei9));
    p.print(pyf!("  {:>8} {:>11} {:>9}   quench speed", "τ_q ms", "EI_NO g/kg", "×rung9"));
    p.print(format!("  {}", dashes(46)));
    for tau_q in [0.01e-3, 0.1e-3, 0.3e-3, 1e-3, 3e-3, 10e-3] {
        let q = quench_no(&rp.comp_p, rp.t_p, rp.alpha, far, tt3, pp, rp.n0, tau_q,
                          QuenchOpts { tab: Some(&rp.tab), ..QuenchOpts::default() });
        let tag = if tau_q <= 0.3e-3 { "fast (RQL target)" } else if tau_q >= 3e-3 { "slow — NO re-made" } else { "" };
        p.print(pyf!("  {:>8.3f} {:>11.4g} {:>8.0f}×   {}", tau_q * 1e3, q.ei, q.ei / ei9, tag));
    }

    p.print("\n  The bell re-filled — rung-9 ideal vs a 3 ms quench (the rich flank comes back):");
    p.print(pyf!("  {:>5} {:>10} {:>11}   note", "φ_p", "ideal EI", "quench 3ms"));
    p.print(format!("  {}", dashes(46)));
    let mut last = None;
    for phi in [0.9, 1.0, 1.1, 1.3, 1.5, 1.8] {
        let z = eq.zoned_nox(far, tt3, tt4, pp, phi,
                             ZonedNoxOpts { tau_q: Some(3e-3), quench_ngrid: ng, ..ZonedNoxOpts::default() });
        let note = if phi == 1.0 { "peak (already at stoich)" } else if phi >= 1.3 { "rich: collapsed → re-filled" } else { "" };
        p.print(pyf!("  {:>5.2f} {:>10.4g} {:>11.4g}   {}", phi, z.ei_no(), z.ei_no_quenched.unwrap(), note));
        last = Some(z);
    }
    let z = last.unwrap();
    p.print("  The ideal rich flank (φ_p≥1.3) collapses to ≈0; a finite quench fills it to a");
    p.print("  ~φ_p-independent ~3 g/kg floor — every rich mix passes the SAME stoich peak. The");
    p.print("  'quick' in quick-quench is the whole game: only a sub-ms quench keeps the rich win.");
    p.print(pyf!("  (Clamp dormant here: max [NO]/[NO]_e = {:.3f} < 1 — NO stays sub-equilibrium;",
                 z.max_a_quench.unwrap()));
    p.print("  the dropped clamp is correct-on-principle, dormant-on-numbers at this lean point.)");
}

/// `JetMixing(J=J, C_e=0.20, shape_n=n)`.
pub(crate) fn jet(j: f64, shape_n: f64) -> JetMixing {
    JetMixing { j, c_e: 0.20, shape_n, ..JetMixing::default() }
}

/// `print_jet_mixing_table(flight)` — rung 11: τ_q derived from the jet.
pub fn jet_mixing_table(p: &mut Printer, d: &Design) {
    let (e, real) = real_eq(d);
    let (tt3, tt4, far, pp) = design_quad(&real);
    let hf = hf_of(&e.gas);
    let ng = 80;

    p.print("\nPhysical mixing (rung 11): what SETS the quench rate — jets in crossflow. τ_q is no");
    p.print("longer a free knob; it is DERIVED from the jet momentum-flux ratio J. 'Quick' = strong jet.");
    p.print(design_line(tt3, tt4, far, pp, ""));

    let phi_p = 1.5;
    let rp = rich_primary(phi_p, far, tt3, pp, hf, ng);
    let run = |m: &JetMixing| {
        let sched = |t: f64| m.schedule(t);
        quench_no(&rp.comp_p, rp.t_p, rp.alpha, far, tt3, pp, rp.n0, m.tau_q(),
                  QuenchOpts { tab: Some(&rp.tab), schedule: Some(&sched), ..QuenchOpts::default() })
    };

    p.print(pyf!("\n  Rich primary φ_p={}: derive τ_q from J (H=0.10 m, U_c=75 m/s, C_e=0.20; \
                  decelerating n=2):", phi_p));
    p.print(pyf!("  {:>5} {:>8} {:>11}   jet", "J", "τ_q ms", "EI_NO g/kg"));
    p.print(format!("  {}", dashes(44)));
    for j in [4.0, 9.0, 16.0, 25.0, 49.0, 100.0] {
        let m = jet(j, 2.0);
        let q = run(&m);
        let tag = if j <= 4.0 { "weak — mixes slow" } else if j >= 49.0 { "strong (RQL target)" } else { "" };
        p.print(pyf!("  {:>5.0f} {:>8.3f} {:>11.4g}   {}", j, m.tau_q() * 1e3, q.ei, tag));
    }
    p.print("  EI_NO falls MONOTONICALLY as J rises — a strong jet quenches fast and escapes the");
    p.print("  stoich peak. No optimum: a mean-field model has no unmixedness (that is rung 12).");

    let j = 25.0;
    let tq = JetMixing { j, c_e: 0.20, ..JetMixing::default() }.tau_q();
    p.print(pyf!("\n  Schedule shape at J={:.0f} (τ_q={:.3f} ms fixed; stoich crossing is at LOW β):", j, tq * 1e3));
    p.print(pyf!("  {:>8} {:>11}   entrainment", "shape n", "EI_NO g/kg"));
    p.print(format!("  {}", dashes(44)));
    for n_shape in [0.5, 1.0, 2.0, 3.0] {
        let q = run(&jet(j, n_shape));
        let kind = if n_shape < 1.0 { "accelerating" } else if n_shape == 1.0 { "LINEAR (rung 10)" } else { "decelerating (real)" };
        p.print(pyf!("  {:>8.1f} {:>11.4g}   {}", n_shape, q.ei, kind));
    }
    p.print("  A DECELERATING entrainment (n>1: fast near the jet, slowing as the gradient collapses)");
    p.print("  clears the early stoich crossing faster → LESS NO than the linear schedule. So IF");
    p.print("  entrainment decelerates (as gradient-collapse suggests), rung 10's linear schedule");
    p.print("  over-predicted the spike by ~2× — within the shape uncertainty (n=0.5 would go the");
    p.print("  other way; the shape is a residual choice, so the SIGN of 'conservative' rides on it).");
}

/// `print_unmixedness_table(flight)` — rung 12: the two-stream variance layer.
pub fn unmixedness_table(p: &mut Printer, d: &Design) {
    let (e, real) = real_eq(d);
    let eq = &e.gas;
    let (tt3, tt4, far, pp) = design_quad(&real);
    let ng = 80;
    let zo = |j: f64, u: Unmixedness| eq.zoned_nox(far, tt3, tt4, pp, 1.5, ZonedNoxOpts {
        mixing: Some(jet(j, 2.0)), unmixedness: Some(u), quench_ngrid: ng, ..ZonedNoxOpts::default()
    });

    p.print("\nSpatial unmixedness (rung 12): the variance rung 11 missed. A mean field says 'stronger");
    p.print("jet → less NO' forever; a real jet has an OPTIMUM. Give the quench TWO streams and the NO-");
    p.print("vs-J curve turns back UP — the Holdeman dilution-jet optimum, recovered AT C_opt≈2.5.");
    p.print(design_line(tt3, tt4, far, pp, ""));

    let u = Unmixedness { s: 0.0625, ..Unmixedness::default() };
    let q = u.c_opt * JetMixing { j: 1.0, ..JetMixing::default() }.h / u.s;
    let j_opt = q * q;
    p.print(pyf!("\n  Rich primary φ_p=1.5; unmixedness S={} m (H=0.10 → uniformity J_opt={:.0f}), \
                  τ_res={:.1f} ms, C_opt={}:", u.s, j_opt, u.tau_res * 1e3, u.c_opt));
    p.print(pyf!("  {:>5} {:>6} {:>7} {:>8} {:>12}   note", "J", "C", "w_core", "EI bulk", "EI 2-stream"));
    p.print(format!("  {}", dashes(58)));
    let mut rows: Vec<(f64, f64)> = Vec::new();
    for j in [4.0, 9.0, 16.0, 25.0, 36.0, 49.0, 64.0, 100.0] {
        let s = zo(j, u);
        rows.push((j, s.ei_no_unmixed.unwrap()));
        let note = if j < j_opt { "under-penetrates" } else if (j - j_opt).abs() < 1e-9 { "OPTIMUM (w→0)" } else { "over-penetrates" };
        p.print(pyf!("  {:>5.0f} {:>6.2f} {:>7.3f} {:>8.4g} {:>12.4g}   {}",
                     j, s.c_holdeman.unwrap(), s.w_core.unwrap(), s.ei_no_quenched.unwrap(),
                     s.ei_no_unmixed.unwrap(), note));
    }
    let imin = py_argmin(&rows.iter().map(|r| r.1).collect::<Vec<_>>());
    p.print(pyf!("  EI_NO falls THEN rises — the minimum lands AT J={:.0f} (C=C_opt=2.5), the", rows[imin].0));
    p.print("  recovered Holdeman optimum. The mean-field 'EI bulk' (rung 11) is still falling at J=100");
    p.print("  — the un-mixed CORE (which misses the jet, quenches at an ABSOLUTE dwell so it survives");
    p.print("  strong jets, and lingers LONGER the further from C_opt) is what turns the TOTAL back up.");

    p.print("\n  The optimum sits AT the Holdeman group C=(S/H)√J=C_opt — shrink S and it moves ((H/S)²):");
    p.print(pyf!("  {:>7} {:>6} {:>12}", "S (m)", "J_opt", "EI-min at J"));
    p.print(format!("  {}", dashes(30)));
    for s_sp in [0.0625, 0.0500] {
        let uu = Unmixedness { s: s_sp, ..Unmixedness::default() };
        let eis: Vec<(f64, f64)> = [9.0, 16.0, 25.0, 36.0, 49.0, 64.0, 100.0].iter()
            .map(|&j| (j, zo(j, uu).ei_no_unmixed.unwrap()))
            .collect();
        let j_min = eis[py_argmin(&eis.iter().map(|e| e.1).collect::<Vec<_>>())].0;
        let qq = uu.c_opt * 0.10 / s_sp;
        let jopt = qq * qq;
        p.print(pyf!("  {:>7.4f} {:>6.0f} {:>12.0f}", s_sp, jopt, j_min));
    }
    p.print("  The EI-min lands ON J_opt for both spacings — the kinked unmixedness PINS it at C_opt, so");
    p.print("  the emissions optimum shifts as (H/S)² exactly (16→25). 'A stronger jet is better' holds");
    p.print("  ONLY up to the Holdeman optimum; past it, over-penetration strands a hot core and NO climbs.");
}

/// `print_mixing_pdf_table(flight)` — rung 13: the resolved β-PDF on the ideal bell.
pub fn mixing_pdf_table(p: &mut Printer, d: &Design) {
    let (e, real) = real_eq(d);
    let (tt3, tt4, far, pp) = design_quad(&real);
    let hf = hf_of(&e.gas);
    let xibar = far / (1.0 + far);

    p.print("\nResolved mixing PDF (rung 13): rung 12 used TWO hand-tuned streams; resolve the WHOLE");
    p.print("mixture-fraction distribution as a mean-preserving β-PDF instead. Integrating the ideal NO");
    p.print("bell over it, the emissions minimum pins AT the Holdeman optimum from a CONTINUOUS PDF — but");
    p.print("the over-penetration CLIMB is gone: that was rung-12's DWELL effect, which this drops.");
    p.print(pyf!("  Design point: Tt3={:.0f} K, Tt4={:.0f} K, p={:.1f} bar, overall far={:.4f} (φ={:.2f}, LEAN); ξ̄={:.4f}",
                 tt3, tt4, pp / 1e5, far, far / f_stoich(), xibar));

    let bell = bell_interpolator(pp, tt3, hf, 3e-3, 200, false);
    let mean_over = |xb: f64, g_seg: f64| -> f64 {
        let (nodes, w) = beta_pdf_nodes_weights(xb, g_seg, 200);
        w.iter().zip(&nodes).fold(0.0, |a, (wi, x)| a + wi * bell.at(*x))
    };
    let pdf_ei = |g_seg: f64| if g_seg <= 1e-9 { bell.at(xibar) } else { mean_over(xibar, g_seg) };

    let ei_mean_lean = bell.at(xibar);
    let xistoich = f_stoich() / (1.0 + f_stoich());
    let ei_mean_stoich = bell.at(xistoich);
    p.print("\n  The mechanism — a PEAKED bell × an OFF-stoich mean (NOT generic convexity):");
    p.print(pyf!("  {:>16} {:>15} {:>17}", "g (segregation)", "⟨EI⟩ lean mean", "⟨EI⟩ stoich mean"));
    p.print(format!("  {}", dashes(52)));
    for g_seg in [0.0, 0.05, 0.10, 0.20] {
        let ei_s = if g_seg <= 1e-9 { ei_mean_stoich } else { mean_over(xistoich, g_seg) };
        p.print(pyf!("  {:>16.2f} {:>15.4g} {:>17.4g}", g_seg, pdf_ei(g_seg), ei_s));
    }
    p.print(pyf!("  LEAN mean (EI(mean)={:.2e}): segregation RAISES ⟨EI⟩ by ~10⁴–10⁵× (the stoich-", ei_mean_lean));
    p.print(pyf!("  ward tail samples the bell peak). STOICH mean (EI(mean)={:.1f}): it LOWERS it", ei_mean_stoich));
    p.print("  (mass moves OFF the peak) — the sign reversal that proves it is 'peaked×off-mean', not Jensen.");

    let s_sp = 0.0625;
    let h = JetMixing { j: 1.0, ..JetMixing::default() }.h;
    let q = 2.5 * h / s_sp;
    let j_opt = q * q;
    p.print(pyf!("\n  J-sweep (S={} m → uniformity J_opt={:.0f}, C_opt=2.5); ⟨EI⟩ over the β-PDF:", s_sp, j_opt));
    p.print(pyf!("  {:>5} {:>6} {:>6} {:>12}   note", "J", "C", "g(C)", "⟨EI⟩ (g/kg)"));
    p.print(format!("  {}", dashes(60)));
    let pdf = MixingPdf { s: s_sp, ..MixingPdf::default() };
    for j in [4.0, 9.0, 16.0, 25.0, 36.0, 49.0, 64.0, 100.0] {
        let c = pdf.c(&JetMixing { j, ..JetMixing::default() });
        let g_seg = pdf.segregation(c);
        let ei = pdf_ei(g_seg);
        let note = if (j - j_opt).abs() < 1e-9 { "OPTIMUM (g→0, ≈0)" } else if j < j_opt { "under-penetrates" } else { "over-penetrates" };
        p.print(pyf!("  {:>5.0f} {:>6.2f} {:>6.3f} {:>12.4g}   {}", j, c, g_seg, ei, note));
    }
    p.print(pyf!("  The minimum is a sharp NOTCH pinned AT J={:.0f} (C=C_opt): perfect mixing → uniform lean", j_opt));
    p.print("  → ≈0 NO. Both immediate flanks lift by orders (segregation). But UNLIKE rung 12 the far");
    p.print("  over-penetration flank DESCENDS, not climbs — ⟨EI⟩(g) is humped (the β-PDF goes bimodal to");
    p.print("  pure-air + rich, both off the stoich peak). Composition variance pins the optimum LOCATION;");
    p.print("  rung-12's DWELL effect makes the climb; carrying the PDF through the quench (rung 15) unites");
    p.print("  them. (The ≈0 minimum here drops the bulk NO floor — that is the same rung-15 scope boundary.)");
}

/// `print_nozzle_flow_table(flight)` — rung 14: frozen vs shifting nozzle.
pub fn nozzle_flow_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nEquilibrium-vs-frozen nozzle flow (rung 14): the production nozzle FREEZES the station-4");
    p.print("mixture; a real nozzle lets it re-equilibrate as it cools — CO/H₂/OH/O/H recombine and give");
    p.print("back thrust. Frozen = a LOWER bound, equilibrium = an UPPER bound; the real nozzle sits between.");

    p.print("\n  Thrust bracket (Tt4 sweep, real-loss cycle) — V9 recovered by a shifting expansion:");
    p.print(pyf!("  {:>8} {:>12} {:>9} {:>9} {:>8} {:>8}", "Tt4 [K]", "CO/(CO+CO2)", "V9 froz", "V9 equil", "ΔV9 m/s", "ΔV9 %"));
    p.print(format!("  {}", dashes(62)));
    for tt4 in [1500.0, 1800.0, 2000.0, 2200.0] {
        let e = build_turbojet(Gas::reacting_equilibrium(), PI_C, tt4, flight.p0, real_losses());
        let r = e.run(flight, 1.0);
        let (st4, st9) = (r.station("4"), r.station("9"));
        let nf = e.gas.nozzle_flow(st4.far, st4.tt, st4.pt, st9.tt, st9.pt, r.p9, None);
        p.print(pyf!("  {:>8.0f} {:>12.2e} {:>9.2f} {:>9.2f} {:>8.3f} {:>7.4f}%",
                     tt4, nf.co_fraction_entry, nf.v9_frozen, nf.v9_equilibrium, nf.dv9(), nf.dv9_frac() * 100.0));
    }
    p.print("  At the metallurgically-capped design point (Tt4=1500 K, lean φ≈0.4) dissociation is ~5e-6");
    p.print("  and the bracket is DORMANT (~0.006%) — like the clamp, negligible HERE. A hot combustor");
    p.print("  dissociates ~1% of the carbon; recombination in the nozzle then buys ~0.4% more exhaust");
    p.print("  velocity. (ΔV9 is the nozzle quantity; the specific-THRUST gain is ~1.2–1.35× larger via");
    p.print("  the M0=0.85 ram term — ΔF/F = ΔV9/(V9−V0/(1+f)).) The real nozzle sits between the bounds.");

    let (e, real) = real_eq(d);
    let eq = &e.gas;
    let (st3, st4, st9) = (real.station("3"), real.station("4"), real.station("9"));
    let zn = eq.zoned_nox(st4.far, st3.tt, st4.tt, st4.pt, 1.0, ZonedNoxOpts { tau: 3e-3, ..ZonedNoxOpts::default() });
    let nf = eq.nozzle_flow(st4.far, st4.tt, st4.pt, st9.tt, st9.pt, real.p9, Some(zn.x_no_mix));
    let max_a = nf.max_a.unwrap();
    p.print(pyf!("\n  The dropped clamp earns its keep (design point: Tt9={:.0f} K → T9={:.0f} K):", st9.tt, nf.t9_frozen));
    p.print(pyf!("  equilibrium NO collapses {:.0f} → {:.2f} ppm on cooling ({:.0f}× — frozen-NO-independent).",
                 nf.x_no_e_entry * 1e6, nf.x_no_e_exit * 1e6, nf.no_collapse_ratio));
    p.print(pyf!("  A realistic zoned exhaust carries EI_NO≈{:.0f} g/kg ({:.0f} ppm), FROZEN", zn.ei_no(), zn.x_no_mix * 1e6));
    p.print(pyf!("  through the nozzle → it is {:.0f}× super-equilibrium at the exit (max_a={:.0f}).", max_a, max_a));
    p.print("  Rung 7's cNO≤cNOe clamp would DELETE that surplus — a plausible-but-wrong low number with");
    p.print("  every assert green. Rung 10 DROPPED the clamp and proved it DORMANT on the combustor quench");
    p.print("  (max_a=0.677<1); HERE, in the near-stoich exhaust-cooling rung 10 flagged, it FIRES. That is");
    p.print("  the whole reason the clamp was dropped 'on principle' — this nozzle is where it bites.");
}
