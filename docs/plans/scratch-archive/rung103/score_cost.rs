// Rung 103 anchor § 5 cost trigger: a page request at an unchoked throttle against the same request at the nearest
// choked one (best of 9, interleaved), on the page's own path (`sandbox::call`), plus the Controls generator check's
// steady matcher called directly.
use std::time::Instant;
use turbojet::atmosphere::standard;
use turbojet::engine::FlightCondition;
use turbojet::sandbox::call;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::control_maps;
use turbojet::shaft_offtake::{Demand, ShaftOfftake, ShaftOfftakeMatcher};

fn best(f: &dyn Fn()) -> f64 {
    (0..9).map(|_| { let t = Instant::now(); f(); t.elapsed().as_secs_f64() }).fold(f64::MAX, f64::min)
}

fn main() {
    let pairs = [
        ("Blades lever", r#"{"op":"blade_lever","Tt4":600,"blades":{}}"#, r#"{"op":"blade_lever","Tt4":650,"blades":{}}"#),
        ("Blades lever", r#"{"op":"blade_lever","Tt4":625,"blades":{}}"#, r#"{"op":"blade_lever","Tt4":650,"blades":{}}"#),
        ("Cooling point", r#"{"op":"cooling_point","cooling":{},"Tt4":600}"#, r#"{"op":"cooling_point","cooling":{},"Tt4":650}"#),
        ("Cooling knob heavy", r#"{"op":"cooling_knob","cooling":{"vane":0.135,"blade":0.185}}"#, r#"{"op":"cooling_knob","cooling":{}}"#),
    ];
    for (name, u, c) in pairs {
        let (mut tu, mut tc) = (f64::MAX, f64::MAX);
        for _ in 0..3 { tu = tu.min(best(&|| { call(u); })); tc = tc.min(best(&|| { call(c); })); }
        println!("{name:20} unchoked {:8.2} ms | choked {:8.2} ms | x{:.2}   [{}]", 1e3 * tu, 1e3 * tc, tu / tc,
                 &call(u)[..60.min(call(u).len())]);
    }
    let bs = BladeSettings::defaults();
    let (ml, mh) = control_maps("flow/press").unwrap();
    let (t, p) = standard(0.0);
    let fl = FlightCondition::new(t + 40.0, p, 0.85);
    let m = ShaftOfftakeMatcher::new(bs.design().with_shaft_offtake(ShaftOfftake::hp(Demand::Power(0.02))), bs.flight(), 1.0, ml, mh, 0.0);
    m.core.base.below_choke.set(true);
    for (tu, tc) in [(700.0, 900.0), (750.0, 900.0)] {
        let ok = m.try_match_point(&fl, tu).is_ok();
        let (mut a, mut b) = (f64::MAX, f64::MAX);
        for _ in 0..3 { a = a.min(best(&|| { let _ = m.try_match_point(&fl, tu); })); b = b.min(best(&|| { let _ = m.try_match_point(&fl, tc); })); }
        println!("generator check {tu:.0}/{tc:.0} K  unchoked {:8.3} ms | choked {:8.3} ms | x{:.2}  (unchoked solves: {ok})",
                 1e3 * a, 1e3 * b, a / b);
    }
}
