// Rung 105 probe P1 (prototype): what the HP vane's law does on the steady running lines, against the pin.
// One shaft: rung 31/33's matcher on the Fly-it design (pi_c 10, Tt4 1500). Two shafts: rung 39's map matcher on the
// rig (3, 6, 1500), below choke ON, rung 104's ellipse on the LP vane. Reactions 0.15 / 0.3 / 0.5.
use turbojet::engine::{build_turbojet, FlightCondition};
use turbojet::gas::Gas;
use turbojet::hp_vane::HpVane;
use turbojet::map::ComponentMap;
use turbojet::matcher::OffDesignMatcher;
use turbojet::sandbox::Settings;
use turbojet::sandbox_blades::{rig_losses, BladeGas};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolMapMatcher};

fn laws() -> Vec<(&'static str, HpVane)> {
    vec![("pin", HpVane::Pin), ("R.10", HpVane::Rows { reaction: 0.10 }), ("R.30", HpVane::Rows { reaction: 0.30 }),
         ("R.50", HpVane::Rows { reaction: 0.50 })]
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let flights = [("design", FlightCondition::new(250.0, 50_000.0, 0.85)),
                   ("hot M.3", FlightCondition::new(318.15, 101_325.0, 0.3))];
    let d = Settings::defaults();
    println!("=== ONE SHAFT (rung 31/33 matcher, pi_c 10, Tt4 1500 design) — rel. change vs pin ===");
    for (gn, gas) in [("CPG", BladeGas::Perfect.gas()), ("TPG", Gas::thermally_perfect())] {
        let mk = || {
            let e = build_turbojet(gas.clone(), d.pi_c, d.tt4, flights[0].1.p0, {
                let mut l = d.losses(); l.nozzle_convergent = true; l.p_exit = None; l });
            OffDesignMatcher::new(e, flights[0].1, 1.0)
        };
        let m = mk();
        for (fname, fl) in flights.iter() {
            for tt4 in [600.0, 700.0, 800.0, 1000.0, 1200.0, 1500.0] {
                let mut base: Option<(f64, f64, f64, f64)> = None;
                let mut line = format!("1sh {gn} {fname:<8} Tt4 {tt4:5.0}");
                for (ln, law) in laws() {
                    m.set_hp_vane(law);
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| m.match_point(fl, tt4))) {
                        Ok(r) => {
                            let v = (r.pi_c, r.mdot_air, r.pi_t, r.thrust);
                            let tag = if r.nozzle_choked { "C" } else { "U" };
                            match base {
                                None => { base = Some(v); line += &format!(" | {ln}{tag}(REF) pic {:.4} mdot {:.4} pit {:.4} F {:.1}", v.0, v.1, v.2, v.3); }
                                Some(b) => line += &format!(" | {ln}{tag} pic {:+.3}% mdot {:+.3}% pit {:+.3}% F {:+.3}%",
                                    100.0 * (v.0 / b.0 - 1.0), 100.0 * (v.1 / b.1 - 1.0), 100.0 * (v.2 / b.2 - 1.0), 100.0 * (v.3 / b.3 - 1.0)),
                            }
                        }
                        Err(_) => line += &format!(" | {ln} REFUSED"),
                    }
                }
                println!("{line}");
            }
        }
    }
    println!("=== TWO SHAFTS (rung 39 map matcher, rig 3/6/1500, below choke ON) — rel. change vs pin ===");
    let f = ComponentMap::flat();
    let mixed = (ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f }, ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f });
    for (gn, gas) in [("CPG", BladeGas::Perfect.gas()), ("TPG", Gas::thermally_perfect()), ("EQ", Gas::reacting_equilibrium())] {
        for (mn, ml, mh) in [("flat", f, f), ("mixed", mixed.0, mixed.1)] {
            let design = build_two_spool_turbojet(gas.clone(), 3.0, 6.0, 1500.0, flights[0].1.p0, rig_losses());
            let m = TwoSpoolMapMatcher::new(design, flights[0].1, 1.0, ml, mh);
            m.core().base.below_choke.set(true);
            for (fname, fl) in flights.iter() {
                for tt4 in [600.0, 700.0, 800.0, 1000.0, 1200.0, 1500.0] {
                    let mut base: Option<[f64; 6]> = None;
                    let mut line = format!("2sh {gn} {mn} {fname:<8} Tt4 {tt4:5.0}");
                    for (ln, law) in laws() {
                        m.core().base.set_hp_vane(law);
                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| m.core().try_match_point(fl, tt4))) {
                            Ok(Ok(p)) => {
                                let r = &p.base;
                                let v = [r.pi_lpc, r.pi_hpc, r.mdot_air, p.n_lp_ratio, p.n_hp_ratio, r.thrust];
                                let tag = if r.m9 >= 0.999 { "C" } else { "U" };
                                match base {
                                    None => { base = Some(v); line += &format!(" | {ln}{tag}(REF) plc {:.4} phc {:.4} mdot {:.4} NL {:.4} NH {:.4} F {:.1}", v[0], v[1], v[2], v[3], v[4], v[5]); }
                                    Some(b) => {
                                        line += &format!(" | {ln}{tag}");
                                        for (k, nm) in ["plc", "phc", "mdot", "NL", "NH", "F"].iter().enumerate() {
                                            line += &format!(" {nm} {:+.3}%", 100.0 * (v[k] / b[k] - 1.0));
                                        }
                                    }
                                }
                            }
                            Ok(Err(e)) => line += &format!(" | {ln} refused: {}", &e.0[..e.0.len().min(50)]),
                            Err(_) => line += &format!(" | {ln} PANIC"),
                        }
                    }
                    println!("{line}");
                }
            }
        }
    }
}
