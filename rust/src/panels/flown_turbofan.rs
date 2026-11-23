//! Rung 110's panel — **born in Rust**, held to its own capture
//! (`rust/oracle/rust_owned/print_flown_turbofan_table.txt`), a CHANGE detector only; its numbers are checked by
//! `tests/rung110.rs` (the same engine is design (6, 1.6) on the equilibrium gas in § 3's grid). The project's
//! turbofan — bypass ratio 6, fan pressure ratio 1.6, designed at Mach 0.85 — frozen and flown over the throttle at its
//! design flight and standing still.

use super::{real_losses, Design, PI_C, TT4};
use crate::engine::FlightCondition;
use crate::flown_turbofan::fly_rung109;
use crate::gas::Gas;
use crate::pyfmt::Printer;
use crate::turbofan::{build_turbofan, Fan};

/// `print_flown_turbofan_table` — rung 110.
pub fn flown_turbofan_table(p: &mut Printer, _d: &Design) {
    p.print("\nTHE TURBOFAN FLOWN (rung 110): the fan's hardware frozen at its design, then the throttle moved. Two fixed");
    p.print("nozzles share the fan's air, so the bypass ratio is now an OUTPUT, not a knob.");
    let design = FlightCondition::new(250.0, 50_000.0, 0.85);
    let mut l = real_losses();
    l.nozzle_convergent = true;
    let tf = build_turbofan(Gas::reacting_equilibrium(), PI_C, TT4, design.p0, l,
                            Fan { alpha: 6.0, ..Fan::default() });
    // 20 kg/s of core air: the engine § 3 scores (`tests/rung110.rs`, MDOT), so every number here is one it checked.
    let ft = fly_rung109(&tf, &design, 20.0).expect("the panel's turbofan flies");
    for (name, fl) in [("Mach 0.85, 250 K, 50 kPa (the design flight)", design),
                       ("standing, sea level", FlightCondition::new(288.15, 101_325.0, 0.0))] {
        p.print(format!("\n  {name}"));
        p.print(format!("  {:>6}{:>8}{:>7}{:>8}{:>7}{:>7}{:>8}{:>8}{:>10}{:>10}{:>13}", "Tt4 K", "bypass", "fan",
                        "HP comp", "N fan", "N HP", "core", "fan", "core F/m", "fan F/m", "TSFC g/kN.s"));
        for tt4 in [1500.0, 1400.0, 1300.0, 1200.0, 1100.0, 1000.0, 900.0] {
            match ft.try_match(&fl, tt4) {
                Ok(r) => {
                    let fan = r.fan.expect("a fan");
                    let state = |choked: bool| if choked { "choked" } else { "open" };
                    p.print(format!("  {tt4:>6.0}{:>8.3}{:>7.3}{:>8.3}{:>7.3}{:>7.3}{:>8}{:>8}{:>10.1}{:>10.1}{:>13.3}",
                                    r.alpha, r.pi_f, r.pi_hpc, r.n_fan, r.n_hp, state(r.core_choked(fl.p0)),
                                    state(r.fan_choked(fl.p0) == Some(true)), r.core_thrust, fan.fan_thrust,
                                    r.performance.tsfc * 1e6));
                }
                Err(e) => p.print(format!("  {tt4:>6.0}  no operating point: {}", e.0)),
            }
        }
    }
    p.print("\n  (fan, HP comp: pressure ratios. N: shaft speed / design, from the work each compressor does. core F/m,");
    p.print("  fan F/m: each jet's net thrust per kilogram of its own air, N.s/kg; negative = slower than the flight.)");
}
