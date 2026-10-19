// Rung 103 PRE-FLIGHT of the anchor's scored cells (none probed before). Prints ONLY the pin side (guard skipped) and
// the class of the ON outcome — never an ON value.
use turbojet::atmosphere::standard;
use turbojet::bleed::TwoSpoolBleedMatcher;
use turbojet::blade_speed::shape_maps;
use turbojet::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
use turbojet::engine::FlightCondition;
use turbojet::gas::{Abort, Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::shaft_offtake::{Demand, ShaftOfftake, ShaftOfftakeMatcher};
use turbojet::turbine_cooling::{TurbineCooling, TurbineCoolingMatcher};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolLosses, TwoSpoolMapCore, TwoSpoolMapMatcher, TwoSpoolMapResult};

fn class(r: &Result<TwoSpoolMapResult, Abort>) -> &'static str {
    match r { Ok(_) => "SOLVES", Err(e) if e.0.contains("rung-103") => "SUB-IDLE", Err(e) if e.0.contains("rung-97") => "NO-PASS",
              Err(e) if e.0.contains("UNCHOKED") => "UNCHOKED", Err(_) => "OTHER" }
}
fn cell(core: &TwoSpoolMapCore, fl: &FlightCondition, tt4: f64) -> (&'static str, String, bool) {
    let b = &core.base;
    let off = core.try_match_point(fl, tt4);
    b.guard_off.set(true);
    let pin = core.try_match_point(fl, tt4);
    b.guard_off.set(false);
    b.below_choke.set(true);
    let on = core.try_match_point(fl, tt4);
    b.below_choke.set(false);
    let ps = match &pin { Ok(x) => format!("pin nL {:.5} nH {:.5} F {:.2}", x.n_lp_ratio, x.n_hp_ratio, x.base.thrust), Err(_) => "pin ERR".into() };
    let scored = class(&off) == "UNCHOKED" && class(&on) == "SOLVES" && pin.is_ok();
    (class(&off), format!("{ps} | ON {}", class(&on)), scored)
}

fn main() {
    let bs = BladeSettings::defaults();
    let fd = bs.flight();
    let flights = |dt: f64, h: f64, m: f64| { let (t, p) = standard(h); FlightCondition::new(t + dt, p, m) };
    let fls = [("SL+20 M0.60", flights(20.0, 0.0, 0.60)), ("2km std M0.50", flights(0.0, 2000.0, 0.50)),
               ("SL-10 M0.40", flights(-10.0, 0.0, 0.40))];
    let mut n_scored = 0;
    // (a) H2: the Controls engine, every variant, two unprobed map shapes.
    for shape in ["steep", "flat-eta"] {
        let (ml, mh) = shape_maps(shape).unwrap();
        let d = || bs.design();
        let variants: Vec<(&str, TwoSpoolMapCore)> = vec![
            ("plain", match TwoSpoolMapMatcher::new(d(), fd, 1.0, ml, mh) { TwoSpoolMapMatcher::Full(c) => c, _ => unreachable!() }),
            ("bleed.08", TwoSpoolBleedMatcher::new(d(), fd, 1.0, ml, mh, 0.08).core),
            ("cust.03", CustomerBleedMatcher::new(d().with_customer_bleed(CustomerBleed::share(0.03)), fd, 1.0, ml, mh, 0.0).core),
            ("cool.02/.04", TurbineCoolingMatcher::new(d().with_turbine_cooling(TurbineCooling::new(0.02, 0.04)), fd, 1.0, ml, mh, 0.0).core),
            ("genHP.02", ShaftOfftakeMatcher::new(d().with_shaft_offtake(ShaftOfftake::hp(Demand::Power(0.02))), fd, 1.0, ml, mh, 0.0).core),
            ("genLP.02", ShaftOfftakeMatcher::new(d().with_shaft_offtake(ShaftOfftake::lp(Demand::Power(0.02))), fd, 1.0, ml, mh, 0.0).core),
        ];
        for (vn, core) in variants.iter() {
            for (fname, fl) in fls.iter() {
                let mut tt4 = 575.0;
                while tt4 <= 825.0 + 1e-9 {
                    let (off, s, sc) = cell(core, fl, tt4);
                    if sc { n_scored += 1; }
                    println!("H2 {shape} {vn} {fname} {tt4:.0}: OFF {off} | {s}{}", if sc { "  [SCORED]" } else { "" });
                    tt4 += 50.0;
                }
            }
        }
    }
    // (b) H1: rung 39's own engine, CPG and the equilibrium gas, three map shapes, two flights.
    let real = TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92, eta_lpt: 0.90,
                                eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true };
    let cpg = Gas::new(GasSpec { gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9, gamma_t: 1.3, cp_t: 1239.0,
                                 r_t: (1.3 - 1.0) / 1.3 * 1239.0, hpr: 42.8e6, ..GasSpec::default() });
    let f = ComponentMap::flat();
    let shapes = [("flat", f, f),
                  ("mixed", ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f }, ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f }),
                  ("steep", shape_maps("steep").unwrap().0, shape_maps("steep").unwrap().1)];
    let fd39 = FlightCondition::new(250.0, 50_000.0, 0.85);
    let fl39 = [("250K/50kPa M0.85", fd39), ("SL std M0.50", flights(0.0, 0.0, 0.50))];
    let mut n1 = 0;
    for (gn, gas) in [("CPG", cpg), ("eq", Gas::reacting_equilibrium())] {
        for (sn, ml, mh) in shapes.iter() {
            let m = TwoSpoolMapMatcher::new(build_two_spool_turbojet(gas.clone(), 3.0, 6.0, 1500.0, 50_000.0, real), fd39, 1.0, *ml, *mh);
            for (fname, fl) in fl39.iter() {
                for tt4 in [450.0, 500.0, 550.0, 600.0, 650.0, 700.0, 750.0] {
                    let (off, s, sc) = cell(m.core(), fl, tt4);
                    if sc { n1 += 1; }
                    println!("H1 {gn} {sn} {fname} {tt4:.0}: OFF {off} | {s}{}", if sc { "  [SCORED]" } else { "" });
                }
            }
        }
    }
    println!("\nH2 scored cells: {n_scored}; H1 scored cells: {n1}");
}
