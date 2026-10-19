//! Rung 102's panel — **born in Rust**, held to its own capture
//! (`rust/oracle/rust_owned/print_afterburner_table.txt`), a CHANGE detector only; its numbers are checked by
//! `tests/rung102.rs`. The afterburner on the design point: Mattingly § 8.4's reference engine through the model, then
//! the project's design lit to three temperatures at two flights, on the equilibrium gas; then (Part B) that hardware
//! flown, its nozzle held, opened partway and opened by the rule.

use super::{real_losses, Design, PI_C, TT4};
use crate::afterburner::{build_afterburning_turbojet, AbMode, Afterburner, FlyAfterburner, Throat};
use crate::components::{choked_mfp, ram_recovery};
use crate::engine::{FlightCondition, Losses};
use crate::gas::{FlowState, Gas, GasSpec};
use crate::pyfmt::Printer;
use crate::sandbox::{fly_solver, stall_reading, FlySettings, Stall};

/// `print_afterburner_table` — rung 102.
pub fn afterburner_table(p: &mut Printer, _d: &Design) {
    p.print("\nTHE AFTERBURNER (rung 102): a second burner behind the turbine, its duct losing pressure at the flame holders");
    p.print("and by heating a moving gas, the nozzle re-sized for the hotter jet.");
    // Mattingly § 8.4's reference point (sea level, maximum wet), the book's equations' fixed duct loss.
    let (gc, gt) = (1.4, 1.3);
    let (cpc, cpt) = (0.24 * 4186.8, 0.295 * 4186.8);
    let gas = Gas::new(GasSpec { gamma_c: gc, cp_c: cpc, r_c: (gc - 1.0) / gc * cpc, gamma_t: gt, cp_t: cpt,
                                 r_t: (gt - 1.0) / gt * cpt, hpr: 18_400.0 * 2326.0, ..GasSpec::default() });
    let l = Losses { pi_d: 0.98, eta_c: 1.0, e_c: Some(0.9), eta_b: 0.995, pi_b: 0.96, eta_t: 1.0, e_t: Some(0.9),
                     eta_m: 0.995, pi_n: 0.98, ..Losses::default() };
    let ab = Afterburner { tt7: 2000.0, eta_ab: 0.95, pi_fh: 0.94, m6: 0.0 };
    let r = build_afterburning_turbojet(gas, 20.0, 3200.0 / 1.8, 101_325.0, l, ab)
        .run(&FlightCondition::new(518.7 / 1.8, 101_325.0, 1e-3), 1.0, AbMode::Lit);
    p.print("\n  Mattingly's afterburning turbojet (Elements of Propulsion, 8.4), sea level, Tt4 1778 K, Tt7 2000 K:");
    p.print(format!("  {:<22}{:>10}{:>10}", "", "book", "model"));
    let f = r.station("4").far;
    for (name, book, model) in [("fuel/air, main", "0.0358", format!("{f:.4}")),
                                ("fuel/air, afterburner", "0.0195", format!("{:.4}", r.detail.f7 - f)),
                                ("F/m0, N.s/kg", "1350.3", format!("{:.1}", r.performance.specific_thrust)),
                                ("TSFC, mg/(N.s)", "41.00", format!("{:.2}", r.performance.tsfc * 1e6)),
                                ("exit Mach", "1.85", format!("{:.3}", r.m9))] {
        p.print(format!("  {name:<22}{book:>10}{model:>10}"));
    }

    // The project's design, lit, at its own flight and at Mach 2 (equilibrium gas, the defaults' duct).
    p.print("\n  The project's design (pressure ratio 10, Tt4 1500 K) on the equilibrium gas, lit over the same engine unlit:");
    p.print(format!("\n  {:<12}{:>8}{:>10}{:>10}{:>12}{:>10}{:>10}", "flight", "Tt7 K", "thrust x", "TSFC x", "throat x",
                    "duct pt", "f_AB"));
    for (name, fl) in [("M0.85", FlightCondition::new(250.0, 50_000.0, 0.85)),
                       ("M2 11 km", FlightCondition::new(216.65, 22_632.0, 2.0))] {
        let losses = Losses { pi_d: 0.97 * ram_recovery(fl.m0), ..real_losses() };
        for tt7 in [1700.0, 1900.0, 2100.0] {
            let e = build_afterburning_turbojet(Gas::reacting_equilibrium(), PI_C, TT4, fl.p0, losses,
                                                Afterburner { tt7, ..Afterburner::default() });
            let core = e.run_core(&fl, 1.0);
            let (wet, dry) = (e.try_continue(&core, AbMode::Lit).expect("runs"), e.try_continue(&core, AbMode::Unlit).expect("runs"));
            let g = &e.engine.gas;
            let area = |s: &FlowState| s.mdot * s.tt.sqrt() / (s.pt * choked_mfp(g, s.tt, s.far));
            p.print(format!("  {name:<12}{tt7:>8.0}{:>10.3}{:>10.3}{:>12.3}{:>10.4}{:>10.4}",
                            wet.performance.specific_thrust / dry.performance.specific_thrust,
                            wet.performance.tsfc / dry.performance.tsfc,
                            area(wet.station("7")) / area(dry.station("7")), wet.detail.pi_ab,
                            wet.detail.f7 - wet.detail.f5));
        }
    }
    let e = build_afterburning_turbojet(Gas::reacting_equilibrium(), PI_C, TT4, 50_000.0, real_losses(),
                                        Afterburner { tt7: 2600.0, ..Afterburner::default() });
    let msg = e.try_run(&FlightCondition::new(250.0, 50_000.0, 0.85), 1.0, AbMode::Lit).err().expect("refused").0;
    p.print(format!("\n  Asked for 2600 K: \"{msg}\""));
    for line in NARRATIVE {
        p.print(*line);
    }

    // Part B: the same hardware flown (the page's Fly it), its nozzle held, opened partway, and opened by the rule.
    let s = FlySettings::defaults();
    let (fl, cmap) = (s.flight(), s.map.map().with_phi_surge(s.phi_surge));
    let st = fly_solver(&s);
    let fa = FlyAfterburner::capture(&st, Afterburner { tt7: 1900.0, ..Afterburner::default() });
    let dry = fa.try_point(&fl, s.tt4, &cmap, AbMode::Unlit, Throat::Held(1.0)).expect("runs");
    let margin = |eq: &crate::spool::Instant| match stall_reading(&st, eq, &cmap) {
        Stall::Margin { sm_n, .. } => format!("{:.1} %", sm_n * 100.0),
        other => format!("{other:?}"),
    };
    p.print("\n  Flown (the page's Fly it): the opening design's hardware at its own flight, thermally-perfect gas, the");
    p.print("  afterburner to 1900 K:");
    p.print(format!("\n  {:<28}{:>11}{:>14}{:>15}", "nozzle throat", "thrust kN", "shaft speed", "stall margin"));
    let rows = [("unlit, dry size".to_string(), dry),
                ("lit, held".to_string(), fa.try_point_after_dry(&fl, s.tt4, &cmap, AbMode::Lit, Throat::Held(1.0), &dry).expect("runs")),
                ("lit, opened x1.15".to_string(), fa.try_point_after_dry(&fl, s.tt4, &cmap, AbMode::Lit, Throat::Held(1.15), &dry).expect("runs")),
                ("lit, by the rule".to_string(), fa.try_lit_matched(&fl, s.tt4, &cmap, &dry).expect("runs"))];
    for (name, pt) in rows {
        let name = if name == "lit, by the rule" { format!("{name} (x{:.3})", pt.s) } else { name };
        p.print(format!("  {name:<28}{:>11.2}{:>12.1} %{:>15}", pt.eq.thrust / 1e3, pt.eq.nu * 100.0, margin(&pt.eq)));
    }
    for line in NARRATIVE_B {
        p.print(*line);
    }
}

const NARRATIVE: &[&str] = &[
    "\n  The afterburner pays most flown fast: the same temperature buys more thrust at Mach 2 for less extra fuel per",
    "  thrust. The nozzle must open more than the square root of the temperature rise, because the hotter gas passes",
    "  less flow per area of throat. On the equilibrium gas the hot products split apart and soak up heat, so burning",
    "  all the oxygen left cannot reach much past 2200 K.",
];

const NARRATIVE_B: &[&str] = &[
    "\n  Lit behind the dry-size throat, the engine makes LESS thrust than unlit: the hotter gas needs a bigger throat, so",
    "  it backs up into the turbine, which then does less work; the shaft slows and the compressor climbs toward stall.",
    "  Opening the throat gives both back. At the rule's size the core runs exactly as it does unlit, and every bit of",
    "  the afterburner's heat goes into the jet; the rule opens it MORE than the square root of the temperature rise.",
];
