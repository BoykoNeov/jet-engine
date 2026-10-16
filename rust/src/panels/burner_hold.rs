//! Rung 94's panel — **born in Rust**, held to its own capture
//! (`rust/oracle/rust_owned/print_burner_hold_table.txt`), a CHANGE detector only; its numbers are checked by
//! `tests/rung94.rs`. Both holds at three throttles on each shaft, and what a hold carries at the 700 K thrust — the
//! scored grid is the spec's.

use super::airflow::design13;
use super::Design;
use crate::burner_hold::{self as bh, Hold};
use crate::map::ComponentMap;
use crate::pyfmt::Printer;
use crate::shaft_offtake::{Demand, ShaftOfftake, ShaftOfftakeMatcher};

/// `print_burner_hold_table` — rung 94.
pub fn burner_hold_table(p: &mut Printer, d: &Design) {
    p.print("\nA BURNER THAT CARRIES THE LOAD (rung 94): each throttle holds the thrust - or the HP speed - this engine");
    p.print("gives there with the generator off, and the burner temperature rises to carry the generator. Two-spool");
    p.print("engine, pi 3 x 6, flat maps; the generator a fixed power (x the design HP compressor power).");
    let flat = ComponentMap::default();
    let mk = |so: Option<ShaftOfftake>| {
        let e = match so { Some(o) => design13(d).with_shaft_offtake(o), None => design13(d) };
        ShaftOfftakeMatcher::new(e, d.flight, 1.0, flat, flat, 0.0)
    };
    let bare = mk(None);
    p.print("\n  1. The burner's rise, and each compressor's flow coefficient against the same load at a fixed burner");
    p.print("     temperature (+ is away from stall). Thrust and speed held against a fixed temperature.");
    p.print(format!("\n  {:>6}{:>7}{:>7}{:>9}{:>10}{:>10}{:>9}{:>10}", "shaft", "Tt4", "load", "dT thr", "dlnphiL", "fixed", "dT spd",
                    "dlnF spd"));
    for (name, hp) in [("HP", true), ("LP", false)] {
        for (t0, w) in [(1500.0, 0.05), (1000.0, 0.05), (700.0, 0.02)] {
            let so = if hp { ShaftOfftake::hp(Demand::power(w)) } else { ShaftOfftake::lp(Demand::power(w)) };
            let m = mk(Some(so));
            let b = bare.match_point(&d.flight, t0);
            let fixed = m.match_point(&d.flight, t0);
            let row = |h: Hold| bh::try_hold_point(&m, &bare, &d.flight, t0, h, 600.0, 2200.0);
            match (row(Hold::Thrust), row(Hold::HpSpeed)) {
                (Ok(th), Ok(sp)) => p.print(format!("  {name:>6}{t0:>7.0}{w:>7.2}{:>+9.2}{:>+10.4}{:>+10.4}{:>+9.2}{:>+10.4}",
                    th.tt4 - t0, (th.point.base.phi_lp / b.base.phi_lp).ln(), (fixed.base.phi_lp / b.base.phi_lp).ln(),
                    sp.tt4 - t0, (sp.point.base.base.thrust / b.base.base.thrust).ln())),
                (a, b2) => p.print(format!("  {name:>6}{t0:>7.0}{w:>7.2}  {}", a.err().or(b2.err()).unwrap_or_default())),
            }
        }
    }
    p.print("\n  2. Holding the 700 K thrust: what a load needs, and what it cannot have.");
    for (name, so) in [("HP 0.10", ShaftOfftake::hp(Demand::power(0.10))), ("LP 0.05", ShaftOfftake::lp(Demand::power(0.05)))] {
        match bh::try_hold_point(&mk(Some(so)), &bare, &d.flight, 700.0, Hold::Thrust, 600.0, 2200.0) {
            Ok(h) => p.print(format!("  {name}: the burner at {:.2} K (the least it can run at: {:.2} K)", h.tt4, h.t_floor)),
            Err(e) => p.print(format!("  {name}: {e}")),
        }
    }
    for line in NARRATIVE {
        p.print(*line);
    }
}

const NARRATIVE: &[&str] = &[
    "\n  Carrying the generator, the hotter burner pushes more air: the LP compressor sits further from stall than at a",
    "  fixed burner temperature at every power, and an LP-shaft generator leaves BOTH compressors further from stall",
    "  than no generator at all, above idle. Holding HP speed is not holding thrust: on the HP shaft it burns hotter and",
    "  gives more thrust; on the LP shaft it can run the burner cooler. A load near a throttle's fold can still be held",
    "  if the burner may rise - but an LP-shaft load lifts the thrust the engine can run at, and below it no hold exists.",
];
