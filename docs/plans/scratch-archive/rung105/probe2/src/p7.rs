// Rung 105 probe P7 — the floors before freezing R2/R3:
//  R3: at the DESIGN flight and Tt4, how far each engine lands from its own design run — the pin (the old model's own
//      floor) and the law (reactions 0.15/0.3/0.5), perfect / thermally perfect / equilibrium gas.
//  R2: perfect gas, nozzle choked, the shortcut OFF: the law's own search against the pin, worst relative difference.
use turbojet::engine::{build_turbojet, FlightCondition};
use turbojet::gas::Gas;
use turbojet::hp_vane::HpVane;
use turbojet::map::ComponentMap;
use turbojet::matcher::OffDesignMatcher;
use turbojet::sandbox::Settings;
use turbojet::sandbox_blades::{rig_losses, BladeGas};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolMapMatcher};

fn laws() -> [HpVane; 4] {
    [HpVane::Pin, HpVane::Rows { reaction: 0.10 }, HpVane::Rows { reaction: 0.3 }, HpVane::Rows { reaction: 0.5 }]
}
fn rel(a: f64, b: f64) -> f64 { (a / b - 1.0).abs() }

fn main() {
    let fd = FlightCondition::new(250.0, 50_000.0, 0.85);
    let d = Settings::defaults();
    println!("=== R3 one shaft (pi_c 10, Tt4 1500): |pi_t - design|, |pi_c - 10|, |mdot - 1| ===");
    for (gn, gas) in [("CPG", BladeGas::Perfect.gas()), ("TPG", Gas::thermally_perfect()), ("EQ", Gas::reacting_equilibrium())] {
        let e = build_turbojet(gas, d.pi_c, d.tt4, fd.p0, { let mut l = d.losses(); l.nozzle_convergent = true; l.p_exit = None; l });
        let m = OffDesignMatcher::new(e, fd, 1.0);
        let s4 = *m.reference.station("4");
        let s5 = *m.reference.station("5");
        let yd = s5.pt / s4.pt;
        for law in laws() {
            m.set_hp_vane(law);
            let r = m.match_point(&fd, d.tt4);
            println!("1sh {gn:<3} {law:?}: pi_t {:.2e}  pi_c {:.2e}  mdot {:.2e}", rel(r.pi_t, yd), rel(r.pi_c, d.pi_c), rel(r.mdot_air, 1.0));
        }
    }
    println!("=== R3 two shafts (rig 3/6/1500, flat maps): |pi_hpt - design|, |pi_hpc - 6|, |pi_lpc - 3|, |mdot - 1| ===");
    let f = ComponentMap::flat();
    for (gn, gas) in [("CPG", BladeGas::Perfect.gas()), ("TPG", Gas::thermally_perfect()), ("EQ", Gas::reacting_equilibrium())] {
        let design = build_two_spool_turbojet(gas, 3.0, 6.0, 1500.0, fd.p0, rig_losses());
        let m = TwoSpoolMapMatcher::new(design, fd, 1.0, f, f);
        let c = m.core();
        let s4 = *c.base.reference.station("4");
        let s45 = *c.base.reference.station("45");
        let yd = s45.pt / s4.pt;
        for law in laws() {
            c.base.set_hp_vane(law);
            let r = c.try_match_point(&fd, 1500.0).unwrap().base;
            println!("2sh {gn:<3} {law:?}: pi_hpt {:.2e}  pi_hpc {:.2e}  pi_lpc {:.2e}  mdot {:.2e}", rel(r.pi_hpt, yd),
                     rel(r.pi_hpc, 6.0), rel(r.pi_lpc, 3.0), rel(r.mdot_air, 1.0));
        }
    }
    println!("=== R2 perfect gas, choked, shortcut OFF: worst |law - pin| over Tt4 700..1600, two flights ===");
    let flights = [fd, FlightCondition::new(288.15, 101_325.0, 0.8)];
    {
        let e = build_turbojet(BladeGas::Perfect.gas(), d.pi_c, d.tt4, fd.p0, { let mut l = d.losses(); l.nozzle_convergent = true; l.p_exit = None; l });
        let m = OffDesignMatcher::new(e, fd, 1.0);
        for law in &laws()[1..] {
            let mut worst: f64 = 0.0;
            for fl in flights.iter() {
                for i in 0..10 {
                    let tt4 = 700.0 + 100.0 * i as f64;
                    m.set_hp_vane(HpVane::Pin);
                    let p = m.match_point(fl, tt4);
                    if !p.nozzle_choked { continue }
                    m.set_hp_vane(*law);
                    m.hp_shortcut.set(false);
                    let q = m.match_point(fl, tt4);
                    m.hp_shortcut.set(true);
                    worst = worst.max(rel(q.pi_t, p.pi_t)).max(rel(q.pi_c, p.pi_c)).max(rel(q.mdot_air, p.mdot_air));
                }
            }
            println!("1sh CPG {law:?}: worst {worst:.2e}");
        }
    }
    {
        let design = build_two_spool_turbojet(BladeGas::Perfect.gas(), 3.0, 6.0, 1500.0, fd.p0, rig_losses());
        let m = TwoSpoolMapMatcher::new(design, fd, 1.0, f, f);
        let c = m.core();
        for law in &laws()[1..] {
            let mut worst: f64 = 0.0;
            for fl in flights.iter() {
                for i in 0..10 {
                    let tt4 = 700.0 + 100.0 * i as f64;
                    c.base.set_hp_vane(HpVane::Pin);
                    let Ok(p) = c.try_match_point(fl, tt4) else { continue };
                    if p.base.m9 < 0.999 { continue }
                    c.base.set_hp_vane(*law);
                    c.base.law_shortcut.set(false);
                    let q = c.try_match_point(fl, tt4).unwrap();
                    c.base.law_shortcut.set(true);
                    worst = worst.max(rel(q.base.pi_hpt, p.base.pi_hpt)).max(rel(q.base.pi_hpc, p.base.pi_hpc))
                        .max(rel(q.base.mdot_air, p.base.mdot_air));
                }
            }
            println!("2sh CPG {law:?}: worst {worst:.2e}");
        }
    }
}
