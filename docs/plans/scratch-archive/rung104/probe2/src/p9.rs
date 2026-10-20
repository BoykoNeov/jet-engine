// Rung 104 probe P9 — the three-way comparison below choke, Controls engine on map "tilted" (P5's probe cells):
// (old) rung 38's pin pushed past choke (rung 103's `pin_past_choke`, rung 97's reference "the old model"),
// (97)  rung 97/103's branch with the pin (the LP turbine free, the vane choked),
// (law) the branch under each law.
// Prints both shaft speeds, both turbine ratios and thrust, and each against (old) and against (97).
use turbojet::atmosphere::standard;
use turbojet::blade_speed::shape_maps;
use turbojet::engine::FlightCondition;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::turbine_law::TurbineLaw;
use turbojet::two_spool::{TwoSpoolMapMatcher, TwoSpoolMapResult};

fn flight(dt: f64, h: f64, m0: f64) -> FlightCondition { let (t, p) = standard(h); FlightCondition::new(t + dt, p, m0) }
fn pct(a: f64, b: f64) -> f64 { 100.0 * (a / b - 1.0) }

fn main() {
    let bs = BladeSettings::defaults();
    let (ml, mh) = shape_maps("tilted").unwrap();
    let m = match TwoSpoolMapMatcher::new(bs.design(), bs.flight(), 1.0, ml, mh) {
        TwoSpoolMapMatcher::Full(c) => c,
        _ => unreachable!(),
    };
    let b = &m.base;
    for (fname, fl) in [("SL+40 M0.85", flight(40.0, 0.0, 0.85)), ("SL M0.30", flight(0.0, 0.0, 0.30)), ("design", bs.flight())] {
        for i in 0..7 {
            let tt4 = 550.0 + 50.0 * i as f64;
            b.turbine_law.set(TurbineLaw::Pin);
            b.pin_past_choke.set(true);
            let old = m.try_match_point(&fl, tt4);
            b.pin_past_choke.set(false);
            b.below_choke.set(true);
            let r97 = m.try_match_point(&fl, tt4);
            let line = |n: &str, r: &TwoSpoolMapResult| {
                print!("{fname:12} {tt4:5.0} {n:8} M9 {:.3} NL {:.5} NH {:.5} piH {:.5} piL {:.5} F {:8.2}", r.base.m9,
                       r.n_lp_ratio, r.n_hp_ratio, r.base.pi_hpt, r.base.pi_lpt, r.base.thrust);
                if let Ok(o) = &old {
                    print!(" | vs old: NL {:+.3}% NH {:+.3}% F {:+.3}%", pct(r.n_lp_ratio, o.n_lp_ratio), pct(r.n_hp_ratio, o.n_hp_ratio),
                           pct(r.base.thrust, o.base.thrust));
                }
                if let Ok(q) = &r97 {
                    print!(" | vs 97: NL {:+.3}% NH {:+.3}% F {:+.3}%", pct(r.n_lp_ratio, q.n_lp_ratio), pct(r.n_hp_ratio, q.n_hp_ratio),
                           pct(r.base.thrust, q.base.thrust));
                }
                println!();
            };
            match &old { Ok(r) => line("old", r), Err(e) => println!("{fname:12} {tt4:5.0} old      REFUSED {}", &e.0[..e.0.len().min(90)]) }
            match &r97 { Ok(r) => line("97", r), Err(e) => println!("{fname:12} {tt4:5.0} 97       REFUSED {}", &e.0[..e.0.len().min(90)]) }
            for (n, law) in [("ellipse", TurbineLaw::Ellipse), ("2rows", TurbineLaw::TwoRows), ("1row", TurbineLaw::OneRow)] {
                b.turbine_law.set(law);
                match m.try_match_point(&fl, tt4) {
                    Ok(r) => line(n, &r),
                    Err(e) => println!("{fname:12} {tt4:5.0} {n:8} REFUSED {}", &e.0[..e.0.len().min(90)]),
                }
            }
            b.below_choke.set(false);
        }
    }
}
