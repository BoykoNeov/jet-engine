//! Rung 109's panel — **born in Rust**, held to its own capture
//! (`rust/oracle/rust_owned/print_turbofan_table.txt`), a CHANGE detector only; its numbers are checked by
//! `tests/rung109.rs`. The turbofan on the design point: Mattingly's Example 7-6 through the model, then the project's
//! design with a fan, bypass ratio 0 to 12, at the design flight and standing still, each at its best fan pressure ratio.

use super::{real_losses, Design, PI_C, TT4};
use crate::engine::{FlightCondition, Losses};
use crate::gas::{Gas, GasSpec};
use crate::pyfmt::Printer;
use crate::turbofan::{best_pi_f, build_turbofan, Fan};

/// `print_turbofan_table` — rung 109.
pub fn turbofan_table(p: &mut Printer, _d: &Design) {
    p.print("\nTHE TURBOFAN (rung 109): a fan, driven by the same turbine, pushes a second stream of air around the core");
    p.print("and out of its own nozzle: more air, more slowly.");
    // Mattingly, Elements of Gas Turbine Propulsion (1996), Example 7-6.
    let (gc, gt) = (1.4, 1.33);
    let (cpc, cpt) = (0.240 * 4186.8, 0.276 * 4186.8);
    let gas = Gas::new(GasSpec { gamma_c: gc, cp_c: cpc, r_c: (gc - 1.0) / gc * cpc, gamma_t: gt, cp_t: cpt,
                                 r_t: (gt - 1.0) / gt * cpt, hpr: 18_400.0 * 2326.0, ..GasSpec::default() });
    let p0 = 20_000.0;
    let l = Losses { pi_d: 0.99, eta_c: 1.0, e_c: Some(0.90), eta_b: 0.99, pi_b: 0.96, eta_t: 1.0, e_t: Some(0.89),
                     eta_m: 0.99, pi_n: 0.99, p_exit: Some(p0 / 0.9), nozzle_convergent: false };
    let fan = Fan { alpha: 8.0, pi_f: 1.7, eta_f: 1.0, e_f: Some(0.89), pi_fn: 0.99 };
    let t0 = 390.0 / 1.8;
    let r = build_turbofan(gas, 36.0, 3000.0 / 1.8, p0, l, fan).run(&FlightCondition::new(t0, p0, 0.8), 1.0);
    let fs = r.fan.expect("a fan");
    let a0 = (gc * (gc - 1.0) / gc * cpc * t0).sqrt();
    p.print("\n  Mattingly's turbofan (Elements of Gas Turbine Propulsion, Example 7-6), Mach 0.8, bypass ratio 8:");
    p.print(format!("  {:<28}{:>10}{:>10}", "", "book", "model"));
    for (name, book, model) in [
        ("fuel/air", "0.02868", format!("{:.5}", r.f())),
        ("core jet speed / a0", "2.427", format!("{:.3}", r.core.v9 / a0)),
        ("fan jet speed / a0", "1.186", format!("{:.3}", fs.exit.v9 / a0)),
        ("F/m0, lbf/(lbm/s)", "18.02", format!("{:.2}", r.core.performance.specific_thrust / 9.80665)),
        ("TSFC, (lbm/h)/lbf", "0.6366", format!("{:.4}", r.core.performance.tsfc / 2.832_545e-5)),
        ("thermal efficiency", "0.4098", format!("{:.4}", r.core.performance.eta_thermal)),
        ("overall (from thrust)", "0.3060", format!("{:.4}", r.core.performance.eta_overall)),
    ] {
        p.print(format!("  {name:<28}{book:>10}{model:>10}"));
    }

    // The project's design with a fan (equilibrium gas, fan efficiency 0.88, duct 0.98), each bypass ratio at its best fan.
    p.print("\n  The project's design (pressure ratio 10, Tt4 1500 K) on the equilibrium gas, each bypass ratio at the fan");
    p.print("  pressure ratio with the lowest fuel per thrust:");
    for (name, fl) in [("Mach 0.85, 250 K, 50 kPa", FlightCondition::new(250.0, 50_000.0, 0.85)),
                       ("standing, sea level", FlightCondition::new(288.15, 101_325.0, 0.0))] {
        p.print(format!("\n  {name}"));
        p.print(format!("  {:>6}{:>10}{:>14}{:>16}{:>10}", "alpha", "best fan", "TSFC g/kN.s", "thrust/core air", "V19/V9"));
        let tf = build_turbofan(Gas::reacting_equilibrium(), PI_C, TT4, fl.p0, real_losses(), Fan::default());
        let jet = tf.run(&fl, 1.0);
        p.print(format!("  {:>6.0}{:>10}{:>14.3}{:>16.1}{:>10}", 0.0, "-", jet.core.performance.tsfc * 1e6,
                        jet.thrust_per_core(), "-"));
        let core = tf.core_to_4(&fl, 1.0);
        for alpha in [1.0, 2.0, 4.0, 8.0, 12.0] {
            let fan = Fan { alpha, ..Fan::default() };
            match best_pi_f(&tf, &core, fan) {
                Some(b) => {
                    let r = tf.try_finish(&core, Fan { pi_f: b.pi_f, ..fan }).expect("the best runs");
                    let mark = if b.at_top { "+" } else if b.at_wall { "w" } else { " " };
                    p.print(format!("  {alpha:>6.0}{:>9.3}{mark}{:>14.3}{:>16.1}{:>10.3}", b.pi_f, b.tsfc * 1e6,
                                    r.thrust_per_core(), r.fan.expect("a fan").exit.v9 / r.core.v9));
                }
                None => p.print(format!("  {alpha:>6.0}  no fan pressure ratio runs")),
            }
        }
    }
    p.print("\n  (+: the best fan is at the knob's top, 4.0, or above it; w: against the turbine's limit.)");
}
