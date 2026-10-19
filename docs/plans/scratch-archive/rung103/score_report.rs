// Rung 103, anchor § 3 "reported, not scored", on H2's 174 scored cells: the HP shaft, both flow coefficients, thrust
// as a DIFFERENCE (branch − pin), per variant. Run after H1/H2 were scored.
use std::collections::BTreeMap;
use turbojet::atmosphere::standard;
use turbojet::blade_speed::shape_maps;
use turbojet::bleed::TwoSpoolBleedMatcher;
use turbojet::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
use turbojet::engine::FlightCondition;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::shaft_offtake::{Demand, ShaftOfftake, ShaftOfftakeMatcher};
use turbojet::turbine_cooling::{TurbineCooling, TurbineCoolingMatcher};
use turbojet::two_spool::{TwoSpoolMapCore, TwoSpoolMapMatcher};

fn flight(dt: f64, h: f64, m0: f64) -> FlightCondition { let (t, p) = standard(h); FlightCondition::new(t + dt, p, m0) }

#[derive(Default)]
struct Tally { n: usize, hp_slower: usize, phl_up: usize, phh_down: usize, f_more: usize, df_min: f64, df_max: f64,
               dnl_min: f64, dnl_max: f64, dnh_min: f64, dnh_max: f64 }

fn main() {
    let bs = BladeSettings::defaults();
    let fd = bs.flight();
    let fls = [flight(20.0, 0.0, 0.60), flight(0.0, 2000.0, 0.50), flight(-10.0, 0.0, 0.40)];
    let mut t: BTreeMap<&str, Tally> = BTreeMap::new();
    for shape in ["steep", "flat-eta"] {
        let (ml, mh) = shape_maps(shape).unwrap();
        let d = || bs.design();
        let vs: Vec<(&str, TwoSpoolMapCore)> = vec![
            ("plain", match TwoSpoolMapMatcher::new(d(), fd, 1.0, ml, mh) { TwoSpoolMapMatcher::Full(c) => c, _ => unreachable!() }),
            ("bleed", TwoSpoolBleedMatcher::new(d(), fd, 1.0, ml, mh, 0.08).core),
            ("customer", CustomerBleedMatcher::new(d().with_customer_bleed(CustomerBleed::share(0.03)), fd, 1.0, ml, mh, 0.0).core),
            ("cooling", TurbineCoolingMatcher::new(d().with_turbine_cooling(TurbineCooling::new(0.02, 0.04)), fd, 1.0, ml, mh, 0.0).core),
            ("gen-HP", ShaftOfftakeMatcher::new(d().with_shaft_offtake(ShaftOfftake::hp(Demand::Power(0.02))), fd, 1.0, ml, mh, 0.0).core),
            ("gen-LP", ShaftOfftakeMatcher::new(d().with_shaft_offtake(ShaftOfftake::lp(Demand::Power(0.02))), fd, 1.0, ml, mh, 0.0).core),
        ];
        for (vn, c) in vs.iter() {
            for fl in fls.iter() {
                for i in 0..6 {
                    let tt4 = 575.0 + 50.0 * i as f64;
                    let off = c.try_match_point(fl, tt4);
                    if !matches!(&off, Err(e) if e.0.contains("UNCHOKED")) { continue }
                    c.base.below_choke.set(true);
                    let on = c.try_match_point(fl, tt4);
                    c.base.below_choke.set(false);
                    c.base.pin_past_choke.set(true);
                    let pin = c.try_match_point(fl, tt4);
                    c.base.pin_past_choke.set(false);
                    let (Ok(o), Ok(p)) = (on, pin) else { continue };
                    let x = t.entry(vn).or_insert(Tally { df_min: f64::MAX, df_max: f64::MIN, dnl_min: f64::MAX, dnl_max: f64::MIN,
                                                          dnh_min: f64::MAX, dnh_max: f64::MIN, ..Default::default() });
                    x.n += 1;
                    if o.n_hp_ratio < p.n_hp_ratio { x.hp_slower += 1 }
                    if o.phi_lp > p.phi_lp { x.phl_up += 1 }
                    if o.phi_hp < p.phi_hp { x.phh_down += 1 }
                    let df = o.base.thrust - p.base.thrust;
                    if df > 0.0 { x.f_more += 1 }
                    x.df_min = x.df_min.min(df); x.df_max = x.df_max.max(df);
                    let dnl = o.n_lp_ratio / p.n_lp_ratio - 1.0;
                    let dnh = o.n_hp_ratio / p.n_hp_ratio - 1.0;
                    x.dnl_min = x.dnl_min.min(dnl); x.dnl_max = x.dnl_max.max(dnl);
                    x.dnh_min = x.dnh_min.min(dnh); x.dnh_max = x.dnh_max.max(dnh);
                }
            }
        }
    }
    let mut tot = (0, 0, 0, 0, 0);
    for (v, x) in t.iter() {
        println!("{v:9} n={:3} | LP speed {:+.2}%..{:+.2}% | HP slower {:3} ({:+.3}%..{:+.3}%) | LP phi up {:3} | HP phi down {:3} | thrust more {:3}, diff {:+.2}..{:+.2} N per kg/s design air",
                 x.n, 100.0 * x.dnl_min, 100.0 * x.dnl_max, x.hp_slower, 100.0 * x.dnh_min, 100.0 * x.dnh_max, x.phl_up, x.phh_down,
                 x.f_more, x.df_min, x.df_max);
        tot = (tot.0 + x.n, tot.1 + x.hp_slower, tot.2 + x.phl_up, tot.3 + x.phh_down, tot.4 + x.f_more);
    }
    println!("TOTAL n={} HP slower {} LP phi up {} HP phi down {} thrust more {}", tot.0, tot.1, tot.2, tot.3, tot.4);
}
