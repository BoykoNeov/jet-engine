// Rung 104 probe P18 — rung 103's R6 on the COOLED variant: at the unchoke boundary, which solve fails, and why?
use turbojet::atmosphere::standard;
use turbojet::blade_speed::shape_maps;
use turbojet::engine::FlightCondition;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::turbine_cooling::{TurbineCooling, TurbineCoolingMatcher};

fn main() {
    let bs = BladeSettings::defaults();
    let (ml, mh) = shape_maps("steep").unwrap();
    let c = TurbineCoolingMatcher::new(bs.design().with_turbine_cooling(TurbineCooling::new(0.02, 0.04)), bs.flight(), 1.0, ml, mh, 0.0).core;
    let (t, p) = standard(0.0);
    let (t2, p2) = standard(2000.0);
    for fl in [FlightCondition::new(t + 20.0, p, 0.60), FlightCondition::new(t2, p2, 0.50)] {
        let off = |tt4: f64| c.try_match_point(&fl, tt4);
        let unch = |tt4: f64| matches!(off(tt4), Err(e) if e.0.contains("UNCHOKED"));
        let (mut lo, mut hi) = (575.0, 1200.0);
        if !unch(lo) || off(hi).is_err() { println!("skip"); continue }
        for _ in 0..60 { let m = 0.5 * (lo + hi); if unch(m) { lo = m } else { hi = m } }
        c.base.below_choke.set(true);
        let on = c.try_match_point(&fl, lo);
        c.base.below_choke.set(false);
        c.base.pin_past_choke.set(true);
        let pin = c.try_match_point(&fl, lo);
        c.base.pin_past_choke.set(false);
        println!("boundary {lo:.9}: on {:?}", on.as_ref().map(|r| r.base.pi_lpt).map_err(|e| e.0.chars().take(200).collect::<String>()));
        println!("                pin {:?}", pin.as_ref().map(|r| r.base.pi_lpt).map_err(|e| e.0.chars().take(200).collect::<String>()));
        c.base.law_shortcut.set(false);
        c.base.below_choke.set(true);
        let on2 = c.try_match_point(&fl, lo);
        c.base.below_choke.set(false);
        c.base.law_shortcut.set(true);
        println!("   on, shortcut off {:?}", on2.as_ref().map(|r| r.base.pi_lpt).map_err(|e| e.0.chars().take(200).collect::<String>()));
    }
}
