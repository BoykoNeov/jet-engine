// Rung 105 probe P3 — rung 104's PROBE cells (Controls engine, map `tilted`, its three probe flights): does rung 104's
// headline ("below choke the back-pressure slows BOTH shafts", against the old model) survive the HP vane's law?
// old = both vanes pinned, the pin pushed past choke; r104 = HP pin, LP ellipse; r105 = HP rows (0.15/0.3/0.5), LP ellipse.
use turbojet::atmosphere::standard;
use turbojet::blade_speed::shape_maps;
use turbojet::engine::FlightCondition;
use turbojet::hp_vane::HpVane;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::turbine_law::TurbineLaw;
use turbojet::two_spool::{TwoSpoolMapCore, TwoSpoolMapMatcher, TwoSpoolMapResult};

fn flight(dt: f64, h: f64, m0: f64) -> FlightCondition { let (t, p) = standard(h); FlightCondition::new(t + dt, p, m0) }
fn run(c: &TwoSpoolMapCore, fl: &FlightCondition, tt4: f64, lp: TurbineLaw, hp: HpVane, old: bool) -> Option<TwoSpoolMapResult> {
    c.base.turbine_law.set(lp);
    c.base.set_hp_vane(hp);
    c.base.pin_past_choke.set(old);
    c.base.below_choke.set(!old);
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| c.try_match_point(fl, tt4))).ok().and_then(|x| x.ok());
    c.base.pin_past_choke.set(false);
    c.base.below_choke.set(false);
    r
}
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let bs = BladeSettings::defaults();
    let (ml, mh) = shape_maps("tilted").unwrap();
    let c = match TwoSpoolMapMatcher::new(bs.design(), bs.flight(), 1.0, ml, mh) { TwoSpoolMapMatcher::Full(c) => c, _ => unreachable!() };
    let flights = [("SL+40 M0.85", flight(40.0, 0.0, 0.85)), ("SL M0.30", flight(0.0, 0.0, 0.30)), ("ctl design", bs.flight())];
    for (fname, fl) in flights.iter() {
        for i in 0..12 {
            let tt4 = 550.0 + 25.0 * i as f64;
            let Some(o) = run(&c, fl, tt4, TurbineLaw::Pin, HpVane::Pin, true) else { println!("{fname} {tt4}: old refuses"); continue };
            let mut line = format!("{fname:<11} {tt4:5.0} m9old {:.3}", o.base.m9);
            for (tag, hp) in [("r104", HpVane::Pin), ("R.10", HpVane::Rows { reaction: 0.10 }), ("R.30", HpVane::Rows { reaction: 0.3 }), ("R.50", HpVane::Rows { reaction: 0.5 })] {
                match run(&c, fl, tt4, TurbineLaw::Ellipse, hp, false) {
                    Some(e) => {
                        let dh = e.n_hp_ratio / o.n_hp_ratio - 1.0;
                        let dl = e.n_lp_ratio / o.n_lp_ratio - 1.0;
                        line += &format!(" | {tag} m9 {:.3} dNH {:+.3}% dNL {:+.3}% s {:+.3} dF {:+.2}%", e.base.m9, 100.0 * dh, 100.0 * dl, dh / dl,
                                         100.0 * (e.base.thrust / o.base.thrust - 1.0));
                    }
                    None => line += &format!(" | {tag} refuses"),
                }
            }
            println!("{line}");
        }
    }
}
