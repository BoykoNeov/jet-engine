//! Rungs 25–30's panels — the nozzle and turbine marches (slice AM).
//!
//! Each function is the `main.py` function of the same name (less the `print_` prefix), line for
//! line. As in `nox.rs`, a panel builds its own equilibrium engine per `Tt4` and calls the
//! diagnostic on THAT engine's gas (`e.gas`).

use super::nox::dashes;
use super::{real_losses, Design, PI_C, TT4};
use crate::engine::{build_turbojet, Engine, EngineResult, Losses};
use crate::gas::{equilibrium_composition, powp, Gas, RU};
use crate::march::{
    tau_chem_recomb, tau_no_destroy, CoupledNoFreezeOut, FiniteRate, FreezeOut, NoFreezeOut,
};
use crate::pyf;
use crate::pyfmt::{PyFormat, Printer};

/// `eq = Gas.reacting_equilibrium(); r = build_turbojet(eq, PI_C, Tt4, …, **REAL_LOSSES).run(flight, 1.0)`.
fn eq_run(d: &Design, tt4: f64) -> (Engine, EngineResult) {
    let e = build_turbojet(Gas::reacting_equilibrium(), PI_C, tt4, d.flight.p0, real_losses());
    let r = e.run(&d.flight, 1.0);
    (e, r)
}

/// `print_finite_rate_nozzle_table(flight)` — rung 25.
pub fn finite_rate_nozzle_table(p: &mut Printer, d: &Design) {
    p.print("\nFinite-rate nozzle chemistry (rung 25): the real Damköhler flow BETWEEN rung-14's bounds.");
    p.print("It reveals a THREE-state picture — the super-equilibrium frozen entry re-equilibrates");
    p.print("IRREVERSIBLY, so even infinitely-fast chemistry (I) falls SHORT of the reversible ceiling (R).");

    p.print("\n  Three-state ceilings (Tt4 sweep, real-loss cycle):");
    p.print(pyf!("  {:>8} {:>9} {:>9} {:>9} {:>11} {:>12}",
                 "Tt4 [K]", "V9 (F)", "V9 (I)", "V9 (R)", "attain I-F", "unreach R-I"));
    p.print(format!("  {}", dashes(66)));
    for tt4 in [1500.0, 1800.0, 2000.0, 2200.0] {
        let (e, r) = eq_run(d, tt4);
        let (st4, st9) = (r.station("4"), r.station("9"));
        let fr = e.gas.finite_rate_nozzle(st4.far, st4.tt, st4.pt, st9.tt, st9.pt, r.p9,
                                          FiniteRate { da: 3.0, ..FiniteRate::default() });
        p.print(pyf!("  {:>8.0f} {:>9.2f} {:>9.2f} {:>9.2f} {:>10.4f}% {:>11.4f}%",
                     tt4, fr.v9_frozen, fr.v9_irrev_fast, fr.v9_reversible,
                     fr.attainable_gap() / fr.v9_frozen * 100.0, fr.unreachable_gap() / fr.v9_frozen * 100.0));
    }
    p.print("  Both gaps COLLAPSE at the cool lean design point (dissociation ~5e-6, no entry non-equilibrium)");
    p.print("  and EARN THEIR KEEP hot — rung-14's own arc. The unreachable (R−I) gap is the entry");
    p.print("  re-equilibration irreversibility: a genuine sliver lean, ~7% of the bracket at 2200 K.");

    p.print("\n  The finite-rate flow filling the attainable [F, I] bracket (Tt4=2200 K):");
    p.print(pyf!("  {:>7} {:>10} {:>13} {:>9} {:>10}", "Da", "V9 finite", "attain-filled", "dS≥0", "CO exit"));
    p.print(format!("  {}", dashes(54)));
    let (e, r) = eq_run(d, 2200.0);
    let (st4, st9) = (r.station("4"), r.station("9"));
    for da in [0.3, 1.0, 3.0, 10.0, 30.0] {
        let fr = e.gas.finite_rate_nozzle(st4.far, st4.tt, st4.pt, st9.tt, st9.pt, r.p9,
                                          FiniteRate { da, ..FiniteRate::default() });
        p.print(pyf!("  {:>7.1f} {:>10.3f} {:>12.3f}  {:>9.3e} {:>10.2e}",
                     da, fr.v9_finite, fr.finite_filled(), fr.ds_finite, fr.co_fraction_finite_exit));
    }
    p.print("  V9(Da) climbs monotonically toward the ATTAINABLE ceiling (I) — never the reversible (R).");
    p.print("  Da is a CARTOON knob (a normalized Damköhler, not an Arrhenius τ_chem): the interior curve");
    p.print("  rides on it; the three-state picture and the (R−I) gap do NOT. Entropy production dS≥0 (2nd");
    p.print("  law) peaks at intermediate Da; freeze-out (τ_chem(T) quenching the recombination) is the");
    p.print("  deferred seam. The entry irreversibility is a consequence of the FROZEN turbine — a shifting");
    p.print("  turbine would shrink it (the rung-26 seam).");
}

/// `print_freeze_out_nozzle_table(flight)` — rung 26.
pub fn freeze_out_nozzle_table(p: &mut Printer, d: &Design) {
    p.print("\nFreeze-out (rung 26): an ANCHORED recombination clock over rung-25's exact integrator.");
    p.print("Rung-25's Da is uniform (no freeze-out); here Da_local(T,p)=τ_res/τ_chem falls through 1");
    p.print("PARTWAY down the nozzle — and the freeze point MOVES with Tt4 (the physics a constant Da");
    p.print("cannot express). Chemistry is GRI-Mech 3.0 verbatim; the only knob left is geometric (L).");

    p.print("\n  The freeze point walking downstream with Tt4 (real self-quenching integrator, L=0.5 m):");
    p.print(pyf!("  {:>8} {:>9} {:>9} {:>9} {:>10} {:>10} {:>13}",
                 "Tt4 [K]", "Da_entry", "Da_exit", "s_freeze", "CO_entry", "CO_exit", "frozen@entry"));
    p.print(format!("  {}", dashes(74)));
    for tt4 in [1500.0, 1650.0, 1800.0, 2000.0, 2200.0] {
        let (e, r) = eq_run(d, tt4);
        let (st4, st9) = (r.station("4"), r.station("9"));
        let fz = e.gas.freeze_out_nozzle(st4.far, st4.tt, st4.pt, st9.tt, st9.pt, r.p9, FreezeOut::default());
        p.print(pyf!("  {:>8.0f} {:>9.3e} {:>9.3e} {:>9.3f} {:>10.3e} {:>10.3e} {:>13}",
                     tt4, fz.da_entry, fz.da_exit, fz.s_freeze, fz.co_fraction_entry,
                     fz.co_fraction_freeze_exit, fz.frozen_from_entry().py_str()));
    }
    p.print("  Lean (≤1650 K): Da_local<1 throughout ⇒ FROZEN FROM ENTRY (s=0) — an independent derivation");
    p.print("  of the production frozen nozzle. Hot: the crossing walks downstream (0.118→0.288→0.378) and");
    p.print("  more CO recombines. The MOTION is the certified rung; the s-VALUES are not (they ride on L).");

    p.print("\n  Kill test (Tt4=2200 K, standalone clock, x_OH pinned at frozen entry):");
    let (_e, r) = eq_run(d, 2200.0);
    let (st4, st9) = (r.station("4"), r.station("9"));
    let ce = equilibrium_composition(st4.far, st4.tt, st4.pt);
    let (tt9, pt9, p9) = (st9.tt, st9.pt, r.p9);
    let t_ex = tt9 * powp(p9 / pt9, (1.30 - 1.0) / 1.30);
    let tau_res = 0.5 / (0.6 * r.v9);
    let da = |tau: f64| tau_res / tau;
    let da_entry = da(tau_chem_recomb(&ce, tt9, pt9, None, None));
    let da_real = da(tau_chem_recomb(&ce, t_ex, p9, None, None));
    let da_kill_t = da(tau_chem_recomb(&ce, t_ex, p9, Some(tt9), None));
    let c_m_in = pt9 / (RU * tt9) / 1.0e6;
    let da_kill_p = da(tau_chem_recomb(&ce, t_ex, p9, None, Some(c_m_in)));
    p.print(pyf!("    entry Da={:6.3f}  →  real exit Da={:6.3f}  (T {:.0f}→{:.0f} K, p {:.0f}→{:.0f} kPa)",
                 da_entry, da_real, tt9, t_ex, pt9 / 1e3, p9 / 1e3));
    p.print(pyf!("    kill T (k pinned, density alone): Da={:6.3f}  → STILL FREEZES  (density did it)", da_kill_t));
    p.print(pyf!("    kill p ([M] pinned, T alone)    : Da={:6.3f}  → NO FREEZE, Da RISES  (k accelerates)", da_kill_p));
    p.print("  Density drives freeze-out DESPITE an opposing T effect — the OPPOSITE sign to Arrhenius");
    p.print("  intuition (Ea=0 ⇒ no thermal barrier), which is what refutes rung-25's 'unanchored Arrhenius");
    p.print("  trap' framing: the rate is anchored (GRI-Mech), and the mechanism runs the other way.");
}

/// `print_no_freeze_out_table(flight)` — rung 27.
pub fn no_freeze_out_table(p: &mut Printer, d: &Design) {
    p.print("\nNO freeze-out (rung 27): is the frozen-NO ASSUMPTION (rung 7 → rung 14/17's clamp) EARNED?");
    p.print("Rung 26 showed the MAJOR pool freezes only PARTWAY down. The SAME machinery on a NO clock from");
    p.print("rung 7's OWN Zeldovich reverse rates (zero new constants): exhaust NO is FROZEN FROM ENTRY at");
    p.print("EVERY Tt4 (Da_NO≪1) — the assumption is DERIVED, on an upper bound (radical-rich frozen pool).");

    p.print("\n  Frozen from entry at every Tt4 — the Da_NO-vs-Da_recomb separation NARROWS hot (L=0.5 m):");
    p.print(pyf!("  {:>8} {:>9} {:>9} {:>13} {:>11} {:>7} {:>9}",
                 "Tt4 [K]", "Da_NO@in", "Da_NO@ex", "Da_recomb@in", "separation", "max_a", "==frozen"));
    p.print(format!("  {}", dashes(78)));
    for tt4 in [1500.0, 1650.0, 1800.0, 2000.0, 2200.0] {
        let (e, r) = eq_run(d, tt4);
        let (st3, st4, st9) = (r.station("3"), r.station("4"), r.station("9"));
        let s = e.gas.no_freeze_out_nozzle(st4.far, st3.tt, st4.tt, st4.pt, st9.tt, st9.pt, r.p9, 1.0,
                                           NoFreezeOut::default());
        let fz = e.gas.freeze_out_nozzle(st4.far, st4.tt, st4.pt, st9.tt, st9.pt, r.p9, FreezeOut::default());
        let sep = fz.da_entry / s.da_entry;
        let matches = (s.max_a - s.max_a_frozen).abs() / s.max_a_frozen < 1e-2;
        p.print(pyf!("  {:>8.0f} {:>9.3e} {:>9.3e} {:>13.3e} {:>11.2e} {:>7.1f} {:>9}",
                     tt4, s.da_entry, s.da_exit, fz.da_entry, sep, s.max_a, matches.py_str()));
    }
    p.print("  Da_NO<1 EVERYWHERE (3–9 orders clear) at every Tt4 — the frozen-NO assumption HOLDS, unlike");
    p.print("  rung 26's major pool (frozen only lean, relaxes hot). The clamp max_a == its rung-14/17");
    p.print("  frozen value to the ≪1 margin: the firing is EARNED. Separation collapses hot (steeply");
    p.print("  Arrhenius NO vs Ea=0 recombination) but never crosses — no moving freeze point is claimed.");

    p.print("\n  Kill test (Tt4=2200 K, standalone NO clock on the frozen pool) — the INVERSION of rung 26:");
    let (e, r) = eq_run(d, 2200.0);
    let (st3, st4, st9) = (r.station("3"), r.station("4"), r.station("9"));
    let s = e.gas.no_freeze_out_nozzle(st4.far, st3.tt, st4.tt, st4.pt, st9.tt, st9.pt, r.p9, 1.0,
                                       NoFreezeOut::default());
    let ce = equilibrium_composition(st4.far, st4.tt, st4.pt);
    let (tt9, pt9, p9, t9) = (st9.tt, st9.pt, r.p9, s.t9_frozen);
    let t_in = tau_no_destroy(&ce, tt9, pt9, None, None);
    let t_kill_t = tau_no_destroy(&ce, t9, p9, Some(tt9), None);
    let t_kill_c = tau_no_destroy(&ce, t9, p9, None, Some(pt9 / (RU * tt9)));
    p.print(pyf!("    net τ_NO growth (T {:.0f}→{:.0f} K, p {:.0f}→{:.0f} kPa): ×{:.2e}  → Da_NO {:.2e}→{:.2e}",
                 tt9, t9, pt9 / 1e3, p9 / 1e3, tau_no_destroy(&ce, t9, p9, None, None) / t_in,
                 s.da_entry, s.da_exit));
    p.print(pyf!("    kill T (k pinned, density alone): τ ×{:6.2f}  → DRIVES freezing", t_kill_t / t_in));
    p.print(pyf!("    kill p (c_tot pinned, T alone)  : τ ×{:6.2e}  → DRIVES freezing (Arrhenius k craters)",
                 t_kill_c / t_in));
    p.print("  BOTH terms AGREE — both drive. Rung 26 had them OPPOSE (density won DESPITE a k that rose on");
    p.print("  cooling, Ea=0). Here the NO reverse rates carry a large barrier (θ≈20820/24560 K), so k joins");
    p.print("  density: same nozzle, two anchored clocks, OPPOSITE mechanism structure. NO freezes BECAUSE");
    p.print("  of temperature; the majors freeze DESPITE it.");
}

/// `print_coupled_no_march_table(flight)` — rung 28.
pub fn coupled_no_march_table(p: &mut Printer, d: &Design) {
    p.print("\nCoupled NO march (rung 28): rung 27's VERDICT confirmed — both its REASONS corrected.");
    p.print("Rung 27 said the coupling 'can ONLY slow NO further'. But rung-26 recombination is EXOTHERMIC,");
    p.print("so it also LIFTS T — and an Arrhenius NO clock SPEEDS UP. Two opposing channels, decomposed");
    p.print("by running the same clock on the two hybrid trajectories (frozen/coupled T × frozen/coupled comp).");

    let band = [1500.0, 1650.0, 1800.0, 2000.0, 2200.0, 2400.0];
    let mut states = Vec::new();
    for tt4 in band {
        let (e, r) = eq_run(d, tt4);
        let (st3, st4, st9) = (r.station("3"), r.station("4"), r.station("9"));
        states.push((tt4, e.gas.coupled_no_freeze_out_nozzle(
            st4.far, st3.tt, st4.tt, st4.pt, st9.tt, st9.pt, r.p9, 1.0, CoupledNoFreezeOut::default(), true)));
    }

    p.print("\n  The two channels at the nozzle exit (Da_NO relative to rung 27's), anchored, L=0.5 m:");
    p.print(pyf!("  {:>8} {:>11} {:>8} {:>9} {:>9} {:>8} {:>10} {:>7}",
                 "Tt4 [K]", "s_frz pool", "dT_exit", "ch1 depl", "ch2 heat", "net", "|ln2/ln1|", "deeper"));
    p.print(format!("  {}", dashes(78)));
    for (tt4, s) in &states {
        p.print(pyf!("  {:>8.0f} {:>11.3f} {:>8.1f} {:>9.4f} {:>9.4f} {:>8.4f} {:>10.3f} {:>7}",
                     tt4, s.s_freeze_pool, s.t9_pool - s.t9_frozen, s.depletion_factor(),
                     s.heat_release_factor(), s.net_factor(), s.channel_ratio(), s.deeper_frozen().py_str()));
    }
    p.print("  net<1 EVERYWHERE ⇒ rung 27's CONCLUSION (deeper into frozen) is CONFIRMED. But ch2>1 always,");
    p.print("  and |ln ch2/ln ch1| rises MONOTONICALLY 0.003→0.48: at the hot edge the opposing channel");
    p.print("  cancels ~HALF the depletion. 'Can ONLY slow NO further' is wrong as a MECHANISM. The net");
    p.print("  even turns non-monotone (deepest ~2200–2300 K) — that turnaround rides on L and is NOT claimed.");
    p.print("  INTERLOCK: the coupling bites exactly where rung 26's pool is alive (s_freeze 0→0.39).");

    p.print("\n  Why depletion wins — drive the pool rate up (Tt4=2200 K): ch1 runs away, ch2 hits a wall:");
    let (e, r) = eq_run(d, 2200.0);
    let (st3, st4, st9) = (r.station("3"), r.station("4"), r.station("9"));
    p.print(pyf!("  {:>10} {:>10} {:>10} {:>10}", "pool rate", "ch1 depl", "ch2 heat", "net"));
    p.print(format!("  {}", dashes(44)));
    for rs in [1.0, 1e1, 1e2, 1e3, 1e4, 1e6] {
        let s = e.gas.coupled_no_freeze_out_nozzle(
            st4.far, st3.tt, st4.tt, st4.pt, st9.tt, st9.pt, r.p9, 1.0,
            CoupledNoFreezeOut { pool_rate_scale: rs, ..CoupledNoFreezeOut::default() }, true);
        p.print(pyf!("  {:>10.0e} {:>10.4f} {:>10.4f} {:>10.4f}",
                     rs, s.depletion_factor(), s.heat_release_factor(), s.net_factor()));
    }
    p.print("  ch1 → 0 with NO floor (τ_NO∝1/[O],[H], and equilibrium radicals crater on cooling);");
    p.print("  ch2 SATURATES (heat release is capped by the FINITE frozen-in chemical enthalpy). So at any");
    p.print("  chemistry faster than anchored, depletion wins by orders — the verdict is STRUCTURAL.");

    p.print("\n  The β repair — rung 27's clock is an a≫1 limit, but NO arrives SUB-equilibrium:");
    p.print(pyf!("  {:>8} {:>9} {:>9} {:>10} {:>8} {:>10} {:>9}",
                 "Tt4 [K]", "a @entry", "a @exit", "sub-eq in", "β max", "τex/τsurr", "bound OK"));
    p.print(format!("  {}", dashes(70)));
    for (tt4, s) in &states {
        p.print(pyf!("  {:>8.0f} {:>9.3f} {:>9.2f} {:>10} {:>8.3f} {:>10.4f} {:>9}",
                     tt4, s.a_entry, s.a_exit, s.sub_equilibrium_entry().py_str(), s.beta_max,
                     s.tau_ratio_min, s.surrogate_bounds_rate().py_str()));
    }
    p.print("  Rung 27 justified its a≫1 clock with 'exhaust NO arrives SUPER-equilibrium'. At the ENTRY it");
    p.print("  does NOT (a=0.31–0.61 hot — NO is BELOW the ceiling and tries to FORM); it goes super-eq only");
    p.print("  as the gas COOLS, at the exit where the clamp is read. Since freeze-FROM-ENTRY is decided at");
    p.print("  the entry, the premise fails where it is needed. What holds instead is β=R1/(R2+R3) < 1:");
    p.print("    τ_exact/τ_surr = (1+u)²/[(1+u)²−(1−β²)] > 1  for ALL a,  u=βa");
    p.print("  ⇒ the surrogate is an UPPER bound on the RATE in BOTH regimes (formation and destruction).");
    p.print("  Rung 27's NUMBERS are unaffected — only its reasoning is repaired. HONEST MARGIN: β rises to");
    p.print("  ~0.51 hot, HALF the β=1 threshold — a factor 2, not orders (the weak point, disclosed).");
}

/// `print_shifting_turbine_table(flight)` — rung 29.
pub fn shifting_turbine_table(p: &mut Printer, d: &Design) {
    p.print("\nThe shifting turbine (rung 29): is FREEZING the turbine — assumed since rung 6 — EARNED?");
    p.print("Bracket it like rung 14 bracketed the nozzle: frozen vs fully-shifting, same shaft work.");
    p.print("The endpoint is WORK-limited, not pressure-limited: the shaft fixes delta_h (compressor + f");
    p.print("only), so a shifting turbine reopens NO shaft fixed point — it moves where the flow ENDS UP.");

    let band = [1500.0, 1800.0, 2100.0, 2400.0];
    let mut states = Vec::new();
    for tt4 in band {
        let (e, r) = eq_run(d, tt4);
        let (st2, st3, st4) = (r.station("2"), r.station("3"), r.station("4"));
        let eq = &e.gas;
        let delta_h = (eq.h_c(st3.tt) - eq.h_c(st2.tt)) / (real_losses().eta_m * (1.0 + st4.far));
        states.push((tt4, eq.shifting_turbine(st4.far, st4.tt, st4.pt, delta_h)));
    }

    p.print("\n  The bound (instant chemistry, reversible — nothing real can exceed it):");
    p.print(pyf!("  {:>8} {:>10} {:>10} {:>9} {:>10} {:>10} {:>7}",
                 "Tt4 [K]", "T5 frozen", "T5 shift", "dT5 [K]", "dT5/T5", "dp5/p5", "earned"));
    p.print(format!("  {}", dashes(70)));
    for (tt4, s) in &states {
        p.print(pyf!("  {:>8.0f} {:>10.2f} {:>10.2f} {:>9.2f} {:>9.4f}% {:>9.4f}% {:>7}",
                     tt4, s.t5_frozen, s.t5_shifting, s.dt5(), s.dt5_fraction() * 100.0,
                     s.dp5_fraction() * 100.0, s.frozen_turbine_earned().py_str()));
    }
    p.print("  At the design point the freeze is EARNED OUTRIGHT: the maximum conceivable shift moves Tt5");
    p.print("  by 0.011% — an order BELOW the cycle's own modelling error (eta_t, pi_b are quoted to ~1%).");
    p.print("  Hot it BITES: 1.9% in Tt5 and 0.47% in pt5 by 2400 K, a 174× growth. So 'the turbine is");
    p.print("  frozen' is a DESIGN-POINT fact, not a structural one — every rung from 6 up inherits that.");

    p.print("\n  Why we expected the opposite — RATIO ≠ ENERGY:");
    p.print(pyf!("  {:>8} {:>15} {:>18} {:>10}", "Tt4 [K]", "super-eq ratio", "radical inventory", "dT5/T5"));
    p.print(format!("  {}", dashes(55)));
    for (tt4, s) in &states {
        p.print(pyf!("  {:>8.0f} {:>14.1f}× {:>18.3e} {:>9.4f}%",
                     tt4, s.super_eq_ratio_max, s.radical_inventory, s.dt5_fraction() * 100.0));
    }
    let (r0, r1) = (&states[0].1, &states[states.len() - 1].1);
    p.print(pyf!("  Across the band the RATIO falls ÷{:.0f} while the INVENTORY rises ×{:.0f} and the SHIFT rises ×{:.0f}.",
                 r0.super_eq_ratio_max / r1.super_eq_ratio_max, r1.radical_inventory / r0.radical_inventory,
                 r1.dt5_fraction() / r0.dt5_fraction()));
    p.print("  Rungs 25–28 justify the super-equilibrium entry with a RATIO (x_frozen/x_eq, [NO]/[NO]_e —");
    p.print("  10×, 100×, '3–9 orders'). That ratio is CORRECT for what it measures — KINETIC distance from");
    p.print("  equilibrium, which is what a RATE question needs. But it is NOT a proxy for exploitable");
    p.print("  ENTHALPY, which scales with the ABSOLUTE radical INVENTORY (x·n) — and the two ANTI-correlate.");
    p.print("  109× of almost nothing is still almost nothing: at the lean design point the radicals are");
    p.print("  ~3e-5 in mole fraction, so complete recombination releases essentially no heat. The ratio is");
    p.print("  LOUDEST exactly where the shift is most NEGLIGIBLE. A cross-rung correction, not a local one.");
    p.print("\n  NOT a finding: that a fully-shifted entry collapses rung-25's (R−I) gap to zero. That is");
    p.print("  STRUCTURAL — an entry pinned at equilibrium has no super-equilibrium left to relax");
    p.print("  irreversibly, so (R−I)→0 is a tautology. What is worth carrying is the SIZE of the move");
    p.print("  needed to get there (1.9% in Tt5) and that the design point sits ~170× short of needing it.");
}

/// `**REAL_LOSSES` with `nozzle_convergent=True` — the rung-30 fixed convergent nozzle.
pub(crate) fn conv_losses() -> Losses {
    Losses { nozzle_convergent: true, ..real_losses() }
}

/// `print_choked_nozzle_table(flight)` — rung 30.
pub fn choked_nozzle_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nThe choked convergent nozzle (rung 30): is FULL EXPANSION — assumed since rung 2 — EARNED?");
    p.print("The shipped nozzle expands fully to p0 (M9 = 1.86, SUPERSONIC — physically a C-D nozzle).");
    p.print("A fixed convergent nozzle chokes at M9 = 1 and leaves the jet UNDEREXPANDED (p9 = p* > p0).");

    let ideal = build_turbojet(Gas::reacting_equilibrium(), PI_C, TT4, flight.p0, real_losses()).run(flight, 1.0);
    let e = build_turbojet(Gas::reacting_equilibrium(), PI_C, TT4, flight.p0, conv_losses());
    let conv = e.run(flight, 1.0);
    let f = conv.station("4").far;
    let r_gas = e.gas.r_t_at(f);
    let p0 = flight.p0;

    let mom_i = (1.0 + f) * ideal.v9 - ideal.v0;
    let mom_c = (1.0 + f) * conv.v9 - conv.v0;
    let p_thrust = (1.0 + f) * r_gas * conv.t9 * (1.0 - p0 / conv.p9) / conv.v9;

    p.print(pyf!("\n  Design point Tt4={:.0f} K, pi_c={:.0f}, M0={}: nozzle entry pt9/p0 = {:.2f} (critical ~1.85 -> CHOKED)",
                 TT4, PI_C, flight.m0, conv.station("9").pt / p0));
    p.print(pyf!("  {:>26} {:>18} {:>18}", "quantity", "ideal (full exp)", "choked convergent"));
    p.print(format!("  {}", dashes(64)));
    let (pi, pc) = (&ideal.performance, &conv.performance);
    p.print(pyf!("  {:>26} {:>18.2f} {:>18.2f}", "exit pressure p9 [kPa]", ideal.p9 / 1000.0, conv.p9 / 1000.0));
    p.print(pyf!("  {:>26} {:>18.4f} {:>18.4f}", "exit Mach M9", ideal.m9, conv.m9));
    p.print(pyf!("  {:>26} {:>18.2f} {:>18.2f}", "exit velocity V9 [m/s]", ideal.v9, conv.v9));
    p.print(pyf!("  {:>26} {:>18.2f} {:>18.2f}", "momentum thrust [N·s/kg]", mom_i, mom_c));
    p.print(pyf!("  {:>26} {:>18.2f} {:>18.2f}", "pressure thrust [N·s/kg]", 0.0, p_thrust));
    p.print(pyf!("  {:>26} {:>18.2f} {:>18.2f}", "specific thrust [N·s/kg]", pi.specific_thrust, pc.specific_thrust));
    p.print(pyf!("  {:>26} {:>18.4e} {:>18.4e}", "TSFC [kg/(N·s)]", pi.tsfc, pc.tsfc));

    let drop = pi.specific_thrust - pc.specific_thrust;
    let recov = p_thrust / (mom_i - mom_c);
    p.print(pyf!("\n  FULL EXPANSION IS NOT EARNED here: specific thrust falls {:.1f} N·s/kg ({:.1f}%), TSFC rises {:.1f}%.",
                 drop, 100.0 * drop / pi.specific_thrust, 100.0 * (pc.tsfc / pi.tsfc - 1.0)));
    p.print(pyf!("  THE FINDING — the pressure term rescues most of it: V9 drops {:.0f}% and momentum thrust {:.0f}%, but",
                 100.0 * (ideal.v9 - conv.v9) / ideal.v9, 100.0 * (mom_i - mom_c) / mom_i));
    p.print(pyf!("  exhausting into p0 < p* turns the static-pressure excess into +{:.0f} N·s/kg of", p_thrust));
    p.print(pyf!("  DIRECT pressure thrust — recovering {:.0f}% of the momentum deficit. That gap between", 100.0 * recov));
    p.print("  '51% loss' and '6.6% loss' is why high-PR engines fit C-D / variable nozzles, and it is the");
    p.print("  pressure-thrust term the cycle has carried honestly since rung 2. (Production stays on the");
    p.print("  ideal nozzle -> cycle unmoved; rung 31 uses this choke to PIN the fixed-throat off-design flow.)");
}
