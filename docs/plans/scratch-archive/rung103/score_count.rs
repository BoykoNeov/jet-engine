// Rung 103: the fired cost trigger (generator check, 5.01x) — count the work: turbine evaluations (tau_calls) and
// joint-loop passes, pinned run vs the branch run, at the unchoked throttle and at the choked one.
use turbojet::atmosphere::standard;
use turbojet::engine::FlightCondition;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::control_maps;
use turbojet::shaft_offtake::{Demand, ShaftOfftake, ShaftOfftakeMatcher};
fn main() {
    let bs = BladeSettings::defaults();
    let (ml, mh) = control_maps("flow/press").unwrap();
    let (t, p) = standard(0.0);
    let fl = FlightCondition::new(t + 40.0, p, 0.85);
    let m = ShaftOfftakeMatcher::new(bs.design().with_shaft_offtake(ShaftOfftake::hp(Demand::Power(0.02))), bs.flight(), 1.0, ml, mh, 0.0);
    let b = &m.core.base;
    for (label, tt4, on) in [("choked 900 K", 900.0, false), ("unchoked 700 K, switch off (the refusal)", 700.0, false),
                             ("unchoked 700 K, switch on (refusal + branch)", 700.0, true)] {
        b.below_choke.set(on);
        b.tau_calls.set(0);
        let r = m.try_match_point(&fl, tt4);
        println!("{label:48} tau_of calls {:6}  ok {}", b.tau_calls.get(), r.is_ok());
    }
}
