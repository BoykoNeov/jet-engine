// Rung 104 PRE-FLIGHT of the anchor's § 3 cells: prints only each cell's CLASS (which of old / 97 / ellipse solve, and
// whether the ellipse's nozzle is unchoked), never a scored quantity. Counts scored cells per variant / map / flight.
use turbojet::atmosphere::standard;
use turbojet::blade_speed::shape_maps;
use turbojet::bleed::TwoSpoolBleedMatcher;
use turbojet::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
use turbojet::engine::FlightCondition;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::shaft_offtake::{Demand, ShaftOfftake, ShaftOfftakeMatcher};
use turbojet::turbine_cooling::{TurbineCooling, TurbineCoolingMatcher};
use turbojet::turbine_law::TurbineLaw;
use turbojet::two_spool::{TwoSpoolMapCore, TwoSpoolMapMatcher};

fn flight(dt: f64, h: f64, m0: f64) -> FlightCondition { let (t, p) = standard(h); FlightCondition::new(t + dt, p, m0) }

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

fn main() {
    let flights = [("SL+20 M0.60", flight(20.0, 0.0, 0.60)), ("2km M0.50", flight(0.0, 2000.0, 0.50)),
                   ("SL-10 M0.40", flight(-10.0, 0.0, 0.40))];
    let mut total = 0;
    for shape in ["steep", "flat-eta", "flow/press"] {
        for (vn, c) in variants(shape) {
            let b = &c.base;
            let mut line = format!("{shape:10} {vn:9}");
            let mut n = 0;
            for (fname, fl) in &flights {
                let mut cls = String::new();
                for i in 0..6 {
                    let tt4 = 575.0 + 50.0 * i as f64;
                    b.turbine_law.set(TurbineLaw::Pin);
                    b.pin_past_choke.set(true);
                    let old = c.try_match_point(fl, tt4).is_ok();
                    b.pin_past_choke.set(false);
                    b.below_choke.set(true);
                    let r97 = c.try_match_point(fl, tt4).is_ok();
                    b.turbine_law.set(TurbineLaw::Ellipse);
                    let ell = c.try_match_point(fl, tt4);
                    b.below_choke.set(false);
                    let ch = match (&ell, old, r97) {
                        (Ok(r), true, true) if r.base.m9 <= 0.99 => { n += 1; 'S' }
                        (Ok(r), _, _) if r.base.m9 > 0.99 => 'c',
                        (Ok(_), _, _) => 'p',
                        (Err(e), _, _) => { eprintln!("x {shape} {vn} {fname} {tt4} old {old} 97 {r97}: {}", &e.0[..e.0.len().min(110)]); 'x' }
                    };
                    cls.push(ch);
                }
                line += &format!("  {fname} {cls}");
            }
            total += n;
            println!("{line}  scored {n}");
        }
    }
    println!("S scored, c choked, p a reference refused, x the ellipse refused; total scored {total}");
}
