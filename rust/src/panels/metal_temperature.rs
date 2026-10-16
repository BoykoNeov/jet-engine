//! Rung 93's panel — **born in Rust**, held to its own capture
//! (`rust/oracle/rust_owned/print_metal_temperature_table.txt`), a CHANGE detector only; its numbers
//! are checked by `tests/rung93.rs`. Gauntner's correlation at one point, the knob form at full-
//! coverage film, and the two framings side by side at four burner temperatures — the scored grid is
//! the spec's.

use super::airflow::design13;
use super::twospool::{cpg13, ts_losses};
use super::Design;
use crate::map::ComponentMap;
use crate::metal_temperature::{self as mt, FixedEngine, MetalLimit};
use crate::pyfmt::Printer;
use crate::turbine_cooling::TurbineCooling;

/// `print_metal_temperature_table` — rung 93.
pub fn metal_temperature_table(p: &mut Printer, d: &Design) {
    p.print("\nMETAL TEMPERATURE (rung 93): what the turbine's metal allows, and what cooling buys. Gauntner's");
    p.print("correlation (NASA TM-81453, 1980): coolant/gas = FACTOR x 0.022 x (phi/(1-phi))^1.25, with his hot");
    p.print("spot, safety margin and 1985 metals (vane 1227.8 K, blade 1172.2 K). Two-spool engine, pi 3 x 6.");
    let flat = ComponentMap::default();
    let fe = FixedEngine::new(design13(d), d.flight, 1.0, flat, flat);

    p.print("\n  1. The knob: full-coverage film (FACTOR 1.0), 2 % blade air, vane air rising. The hottest");
    p.print("     burner temperature the metal allows on THIS engine, and which row sets it.");
    p.print(format!("\n  {:>8}{:>12}{:>9}{:>12}{:>12}", "vane %", "Tt4 max", "row", "vane K", "blade K"));
    let lim = MetalLimit::gauntner_1985(1.0);
    for e1 in [0.0, 0.02, 0.04, 0.08, 0.16] {
        match fe.try_max_tt4(TurbineCooling::new(e1, 0.02), &lim, &d.flight, 800.0, 2400.0) {
            Ok(m) => p.print(format!("  {:>8.1}{:>12.2}{:>9}{:>12.2}{:>12.2}", 100.0 * e1, m.tt4, m.binding.name(),
                                     m.reading.vane, m.reading.blade)),
            Err(e) => p.print(format!("  {:>8.1}  {}", 100.0 * e1, e.0)),
        }
    }

    p.print("\n  2. The cooling that holds BOTH metals at their limits: on this engine (B, its hardware sized");
    p.print("     uncooled at 1500 K) and on a fresh design at each temperature (A). Shares of the HP");
    p.print("     compressor's air; thrust per inlet air; fuel per unit thrust (mg/N.s).");
    p.print(format!("\n  {:>7}{:>6}{:>9}{:>9}{:>9}{:>9}{:>9}{:>9}{:>9}", "FACTOR", "Tt4", "B vane", "B blade",
                    "B F/m", "A vane", "A blade", "A F/m", "A TSFC"));
    for factor in [2.0, 1.0] {
        let lim = MetalLimit::gauntner_1985(factor);
        for tt4 in [1300.0, 1600.0, 1900.0, 2200.0] {
            let b = fe.try_limited(&lim, &d.flight, tt4).expect("B");
            let a = mt::try_limited_design(&cpg13(), 3.0, 6.0, tt4, d.flight.p0, ts_losses(), &lim, &d.flight, 1.0)
                .expect("A");
            p.print(format!("  {factor:>7.1}{tt4:>6.0}{:>9.4}{:>9.4}{:>9.1}{:>9.4}{:>9.4}{:>9.1}{:>9.3}", b.vane, b.blade,
                            b.point.booking.expect("cooled").st_inlet, a.vane, a.blade, a.run.st_inlet,
                            1e6 * a.run.tsfc_inlet));
        }
    }
    for line in NARRATIVE {
        p.print(*line);
    }
}

const NARRATIVE: &[&str] = &[
    "\n  Cooling never stops the engine: each percent of vane air buys about 100 K of burner temperature",
    "  and costs about half a percent of thrust, so thrust at the metal limit keeps rising - on this engine",
    "  and on a fresh design. On fixed hardware the cooling also cools ITS OWN coolant (it slows the HP",
    "  spool, so the compressor delivers colder air), and below a crossing - which rises as the cooling",
    "  technology worsens - the fixed engine needs less air than a fresh design. Vane air relieves the",
    "  blade too: it dilutes the gas the rotor sees. And on a fresh design the metal-limited engine burns",
    "  LESS fuel per unit thrust than the uncooled one at the same burner temperature: the burner gets",
    "  less air at the same fuel ratio, while the thrust falls only half as fast.",
];
