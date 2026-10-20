// Rung 104 R1b — the OLD MODEL's record, captured from the repo BEFORE any rung-104 code (`33250a2` + docs only).
// Rung 104's test computes the SAME keys with every engine on the pin setting and requires them bit for bit, so the pin
// stays proven after the Python-held two-shaft oracles are retired. One line per key: `key value` (`{:?}`, round-trip
// exact) or `key ERR <first 60 chars of the refusal>`.
use turbojet::afterburner::AbMode;
use turbojet::atmosphere::standard;
use turbojet::blade_speed::shape_maps;
use turbojet::bleed::TwoSpoolBleedMatcher;
use turbojet::burner_hold::Hold;
use turbojet::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
use turbojet::engine::FlightCondition;
use turbojet::gas::Gas;
use turbojet::map::ComponentMap;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::{controls, ControlsSettings, CustomerMode, GeneratorMode};
use turbojet::shaft_offtake::{Demand, ShaftOfftake, ShaftOfftakeMatcher};
use turbojet::turbine_cooling::{TurbineCooling, TurbineCoolingMatcher};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolLosses, TwoSpoolMapCore, TwoSpoolMapMatcher, TwoSpoolMatcher};

fn flight(dt: f64, h: f64, m0: f64) -> FlightCondition { let (t, p) = standard(h); FlightCondition::new(t + dt, p, m0) }
fn real() -> TwoSpoolLosses {
    TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92, eta_lpt: 0.90,
                     eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true }
}
fn err(k: &str, e: &str) { println!("{k} ERR {}", e.chars().take(60).collect::<String>()); }

fn variants(shape: &str) -> Vec<(&'static str, TwoSpoolMapCore)> {
    let bs = BladeSettings::defaults();
    let fd = bs.flight();
    let (ml, mh) = shape_maps(shape).unwrap();
    let d = || bs.design();
    vec![
        ("plain", match TwoSpoolMapMatcher::new(d(), fd, 1.0, ml, mh) { TwoSpoolMapMatcher::Full(c) => c, _ => unreachable!() }),
        ("bleed", TwoSpoolBleedMatcher::new(d(), fd, 1.0, ml, mh, 0.08).core),
        ("customer", CustomerBleedMatcher::new(d().with_customer_bleed(CustomerBleed::share(0.03)), fd, 1.0, ml, mh, 0.0).core),
        ("cooling", TurbineCoolingMatcher::new(d().with_turbine_cooling(TurbineCooling::new(0.02, 0.04)), fd, 1.0, ml, mh, 0.0).core),
        ("gen-HP", ShaftOfftakeMatcher::new(d().with_shaft_offtake(ShaftOfftake::hp(Demand::Power(0.02))), fd, 1.0, ml, mh, 0.0).core),
        ("gen-LP", ShaftOfftakeMatcher::new(d().with_shaft_offtake(ShaftOfftake::lp(Demand::Power(0.02))), fd, 1.0, ml, mh, 0.0).core),
    ]
}

fn map_point(k: &str, c: &TwoSpoolMapCore, fl: &FlightCondition, tt4: f64) {
    match c.try_match_point(fl, tt4) {
        Ok(r) => {
            for (n, v) in [("piHPT", r.base.pi_hpt), ("piLPT", r.base.pi_lpt), ("piLPC", r.base.pi_lpc), ("piHPC", r.base.pi_hpc),
                           ("NL", r.n_lp_ratio), ("NH", r.n_hp_ratio), ("F", r.base.thrust)] {
                println!("{k}.{n} {v:?}");
            }
        }
        Err(e) => err(k, &e.0),
    }
}

fn main() {
    // 1. the six steady variants, the Controls engine (perfect gas), choked and below choke (rung 103's branch on)
    for shape in ["tilted", "steep"] {
        for (vn, c) in variants(shape) {
            c.base.below_choke.set(true);
            for (fname, fl) in [("SL+40M0.85", flight(40.0, 0.0, 0.85)), ("SLM0.30", flight(0.0, 0.0, 0.30)),
                                ("design", BladeSettings::defaults().flight())] {
                for i in 0..5 {
                    let tt4 = 600.0 + 100.0 * i as f64;
                    map_point(&format!("steady.{shape}.{vn}.{fname}.{tt4}"), &c, &fl, tt4);
                }
            }
        }
    }
    // 2. rung 39's engine on the equilibrium gas, flat and steep maps, choked
    let fd = FlightCondition::new(250.0, 50_000.0, 0.85);
    for (mname, (ml, mh)) in [("flat", (ComponentMap::flat(), ComponentMap::flat())), ("steep", shape_maps("steep").unwrap())] {
        let m = match TwoSpoolMapMatcher::new(build_two_spool_turbojet(Gas::reacting_equilibrium(), 3.0, 6.0, 1500.0, 50_000.0, real()),
                                              fd, 1.0, ml, mh) {
            TwoSpoolMapMatcher::Full(c) => c,
            _ => unreachable!(),
        };
        for i in 0..5 {
            let tt4 = 700.0 + 200.0 * i as f64;
            map_point(&format!("r39eq.{mname}.{tt4}"), &m, &fd, tt4);
        }
    }
    // 3. rung 38's matcher (no map), equilibrium gas, choked
    let m38 = TwoSpoolMatcher::new(build_two_spool_turbojet(Gas::reacting_equilibrium(), 3.0, 6.0, 1500.0, 50_000.0, real()), fd, 1.0);
    for i in 0..5 {
        let tt4 = 700.0 + 200.0 * i as f64;
        let r = m38.match_point(&fd, tt4).two();
        for (n, v) in [("piHPT", r.pi_hpt), ("piLPT", r.pi_lpt), ("piLPC", r.pi_lpc), ("piHPC", r.pi_hpc), ("F", r.thrust)] {
            println!("r38eq.{tt4}.{n} {v:?}");
        }
    }
    // 4. Controls runs (the transient family through the page's own entry)
    let d = ControlsSettings::defaults();
    let lit = |s: ControlsSettings| ControlsSettings { ab: Some(AbMode::Lit), ..s };
    let runs: Vec<(&str, ControlsSettings)> = vec![
        ("default", d),
        ("unchoked-up", ControlsSettings { t0: 288.15 + 40.0, p0: 101_325.0, m0: 0.85, from: 700.0, to: 1400.0, ..d }),
        ("unchoked-down", ControlsSettings { t0: 288.15 + 40.0, p0: 101_325.0, m0: 0.85, from: 1400.0, to: 700.0, ..d }),
        ("governed-thrust", ControlsSettings { govern: true, hold: Hold::Thrust, ..d }),
        ("cooling", ControlsSettings { cool_vane: 0.05, cool_blade: 0.03, ..d }),
        ("gen-HP", ControlsSettings { generator: GeneratorMode::Hp, ..d }),
        ("customer", ControlsSettings { customer: CustomerMode::Share, ..d }),
        ("ab-lit", lit(d)),
        ("ab-lit-wide", lit(ControlsSettings { a8_ratio: 1.15, tt7: 1900.0, ..d })),
    ];
    for (rn, s) in runs {
        match controls(&s) {
            Ok((_, o)) => {
                println!("ctl.{rn}.points {}", o.points.len());
                for (i, p) in o.points.iter().enumerate() {
                    for (n, v) in [("nuL", p.nu_lp), ("nuH", p.nu_hp), ("Tt4", p.tt4), ("spF", p.sp_thrust), ("mdot", p.mdot_air)] {
                        println!("ctl.{rn}.{i}.{n} {v:?}");
                    }
                }
            }
            Err(e) => err(&format!("ctl.{rn}"), &format!("{e:?}")),
        }
    }
}
