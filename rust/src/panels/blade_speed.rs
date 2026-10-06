//! Rung 85's panel — **born in Rust**, so it has no `main.py` ancestor and no Python segment. It
//! is held to its own capture (`rust/oracle/rust_owned/print_blade_speed_table.txt`), which is a
//! CHANGE detector only; its numbers are checked by `tests/rung85.rs`. Formatted with `format!`
//! (no Python format spec to honour).

use super::airflow::{design13, hp_map, lp_map};
use super::Design;
use crate::blade_speed::{build, BladeKnobs, Droop, Lever, LeverRead, Machine, Verdict};
use crate::pyfmt::Printer;
use crate::two_spool::Spool;

/// § 6.2's six (`h`, `M_rel,lim`) cells.
const CELLS: [(f64, f64); 6] = [(0.5, 1.3), (0.5, 1.4), (0.5, 1.5), (0.7, 1.3), (0.7, 1.4), (0.7, 1.5)];

fn machine(d: &Design, h: f64, m: f64, lambda: f64, droop: Droop) -> Machine {
    let k = BladeKnobs { h, m_rel_lim: m, lambda, droop, ..BladeKnobs::default() };
    build(&design13(d), d.flight, lp_map(), hp_map(), k, k)
        .unwrap_or_else(|e| panic!("rung 85 panel: h={h} M={m} λ={lambda}: {e}"))
}

fn verdict_str(v: Verdict) -> &'static str {
    match v {
        Verdict::Crosses => "CROSSES",
        Verdict::Under => "under",
        Verdict::Void => "void",
    }
}

fn lever_row(p: &mut Printer, z: &Machine, label: &str, r: &LeverRead) {
    let (n, red) = match r.spool {
        Spool::Lp => (r.n_lp, z.lp.redline),
        Spool::Hp => (r.n_hp, z.hp.redline),
    };
    // The lumped plant is rung 53's one block per spool, whatever K the walls sized.
    let ks = if r.lever == Lever::Lumped { "1/1".to_string() } else { format!("{}/{}", z.lp.k, z.hp.k) };
    p.print(format!("  {label:<31}{ks:>6}{:>9}{:>8.4}{:>9.4}{:>8.3}  {}",
                    if r.reached { "yes" } else { "NO (V1)" }, r.vsv_star, n, red,
                    verdict_str(r.verdict(z, r.spool))));
}

/// `print_blade_speed_table` — rung 85.
pub fn blade_speed_table(p: &mut Printer, d: &Design) {
    p.print("\nTHE BLADE-SPEED WALLS (rung 85): the stack gets a physical blade speed, and the stage");
    p.print("count K, the capacity C and a REDLINE R = N_red/N_d become OUTPUTS. Two walls per spool:");
    p.print("an AIRFLOW level (front-row relative tip Mach -- an efficiency target that SIZES the design)");
    p.print("and a STRENGTH wall (blade-root pull, Ti-6Al-4V AMS 4911 min 827.4 MPa / 4429 kg/m^3,");
    p.print("with 14 CFR 33.27's 120 % overspeed) -- the only OFF-design wall. Design flow coefficient");
    p.print("Phi_d = 0.537 (NASA Rotor 37's mean line).");
    p.print("\n  THE ROUNDING KNOB lambda: K must be whole, so K = ceil(K*) leaves a gap below the wall.");
    p.print("  lambda = 0 slows the blades to close it (rungs 55/56's machine); lambda = 1 keeps them AT");
    p.print("  the wall and loads every row lighter (r = its work vs the unswirled row) through a design");
    p.print("  pre-swirl -- exactly a reshaped map: l' = (1+l)/r - 1, sigma' = sigma/r (droop switch A).");

    p.print("\n  THE SIZED MACHINES (shape flow/press; the airflow level binds every cell; m/s, degrees):");
    p.print(format!("  {:<5}{:>4}{:>6} |{:>6}{:>3} | {:>7}{:>7}{:>7}{:>7} | {:>7}{:>7}{:>7}{:>7}{:>7}",
                    "spool", "h", "M_lim", "K*", "K", "U_tip", "M_rel", "C", "R",
                    "U_tip", "r", "swirl", "M_rel", "R"));
    p.print(format!("  {:<15} |{:9} | {:<28} | {}", "", "", "lambda = 0", "lambda = 1"));
    for (spool, name) in [(Spool::Lp, "LP"), (Spool::Hp, "HP")] {
        for &(h, m) in &CELLS {
            let z0 = machine(d, h, m, 0.0, Droop::WithBladeSpeed);
            let z1 = machine(d, h, m, 1.0, Droop::WithBladeSpeed);
            let (a, b) = match spool { Spool::Lp => (z0.lp, z1.lp), Spool::Hp => (z0.hp, z1.hp) };
            p.print(format!("  {name:<5}{h:>4.1}{m:>6.2} |{:>6.2}{:>3} | {:>7.1}{:>7.3}{:>7.3}{:>7.3} | {:>7.1}{:>7.3}{:>7.1}{:>7.3}{:>7.3}",
                            a.k_star, a.k, a.u_tip, a.m_rel_d, a.capacity, a.redline,
                            b.u_tip, b.r, b.alpha_d_deg, b.m_rel_d, b.redline));
        }
    }
    p.print("  The staircase: at lambda = 0 several levels build ONE machine (same K, same R); at");
    p.print("  lambda = 1 R moves smoothly with the level and only K steps.");

    p.print("\n  THE LEVERS AGAINST THE REDLINE (constant-incidence stator schedules, Tt4 = 1000; N/N_d is");
    p.print("  PHYSICAL shaft speed vs design; a schedule that never reaches its target is read at the");
    p.print("  last setting its scan reached -- V1):");
    p.print(format!("  {:<31}{:>6}{:>9}{:>8}{:>9}{:>8}  {}", "lever (spool), cell", "K L/H", "reached", "v*",
                    "N/N_d", "R", "verdict"));
    let t = [1000.0];
    let z = machine(d, 0.5, 1.4, 0.0, Droop::WithBladeSpeed);
    lever_row(p, &z, "lumped (LP) h.5 M1.4", &z.schedule(Lever::Lumped, Spool::Lp, &t)[0]);
    lever_row(p, &z, "lumped (HP) h.5 M1.4", &z.schedule(Lever::Lumped, Spool::Hp, &t)[0]);
    lever_row(p, &z, "all rows (LP) h.5 M1.4", &z.schedule(Lever::AllRows, Spool::Lp, &t)[0]);
    lever_row(p, &z, "front row (LP) h.5 M1.4", &z.schedule(Lever::FrontRow, Spool::Lp, &t)[0]);
    let z = machine(d, 0.5, 1.5, 0.0, Droop::WithBladeSpeed);
    lever_row(p, &z, "all rows (HP) h.5 M1.5", &z.schedule(Lever::AllRows, Spool::Hp, &t)[0]);
    lever_row(p, &z, "front row (HP) h.5 M1.5", &z.schedule(Lever::FrontRow, Spool::Hp, &t)[0]);
    for (lambda, droop, tag) in [(1.0, Droop::WithBladeSpeed, "lam=1 A"), (1.0, Droop::WithRowWork, "lam=1 B")] {
        for &(h, m) in &[(0.5, 1.5), (0.7, 1.5)] {
            let z = machine(d, h, m, lambda, droop);
            let label = format!("lumped (LP) h{h} M{m} {tag}");
            lever_row(p, &z, &label, &z.schedule(Lever::Lumped, Spool::Lp, &t)[0]);
        }
    }

    p.print("\n  WHAT IT SAYS:");
    p.print("  * Holding the design incidence pins 1/phi - v = 1, and then psi = 1 - sigma u^2 - l u - v(1+l)phi");
    p.print("    collapses to 1 + u - sigma u^2: on the lumped machine the held speed carries NO map slope l.");
    p.print("    So lambda reaches a held schedule only through the droop sigma -- switch A (droop tied to");
    p.print("    blade speed) raises it, switch B (droop shrinks with row work) leaves it within 0.1 %.");
    p.print("  * lambda = 1 lowers every redline (faster blades); on switch A the h=0.5 M=1.5 cell's lumped");
    p.print("    LP schedule then crosses its redline, by under 0.1 %.");
    p.print("  * The all-rows LP schedule never reaches its target at Tt4 = 1100/1000 and passes the redline");
    p.print("    on the way (V1); the HP all-rows one stays 1.6 % under the tightest HP redline.");
    p.print("  * The front-row lever raises N_L but LOWERS the front row's tip Mach (a reading, never a wall).");
}
