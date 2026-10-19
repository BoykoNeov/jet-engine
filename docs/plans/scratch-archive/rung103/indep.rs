// Rung 103 probe 2c: an INDEPENDENT root for every variant — the LP turbine HELD at a trial pressure ratio (no call to
// rung 97's helper), the variant's own shipped body run to its forward rebuild, and an outer bisection on the nozzle's
// mass balance read off the RESULT (station-5 flow vs A8·rho9·V9). Calibrated first on choked cells (root = the pin).
use turbojet::atmosphere::standard;
use turbojet::bleed::TwoSpoolBleedMatcher;
use turbojet::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
use turbojet::engine::FlightCondition;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::control_maps;
use turbojet::shaft_offtake::{Demand, ShaftOfftake, ShaftOfftakeMatcher};
use turbojet::turbine_cooling::{TurbineCooling, TurbineCoolingMatcher};
use turbojet::two_spool::{TwoSpoolMapCore, TwoSpoolMapMatcher};
use turbojet::unchoked_lp::PI_TOP;

fn resid(core: &TwoSpoolMapCore, fl: &FlightCondition, tt4: f64, pi: f64) -> Option<f64> {
    core.base.fixed_pi.set(Some(pi));
    let r = core.try_match_point(fl, tt4);
    core.base.fixed_pi.set(None);
    let r = r.ok()?;
    let s5 = *r.base.station("5");
    let rgas = core.gas().r_t_at(s5.far);
    let demand = core.base.a8 * r.base.p9 / (rgas * r.base.t9) * r.base.v9;
    Some((s5.mdot - demand) / s5.mdot)
}

fn main() {
    let bs = BladeSettings::defaults();
    let fd = bs.flight();
    let (t, p) = standard(0.0);
    let flights = [("SL+40 M0.85", FlightCondition::new(t + 40.0, p, 0.85)), ("SL std M0.30", FlightCondition::new(t, p, 0.30))];
    let (ml, mh) = control_maps("flow/press").unwrap();
    let d = || bs.design();
    let variants: Vec<(&str, TwoSpoolMapCore)> = vec![
        ("plain", match TwoSpoolMapMatcher::new(d(), fd, 1.0, ml, mh) { TwoSpoolMapMatcher::Full(c) => c, _ => unreachable!() }),
        ("bleed.05", TwoSpoolBleedMatcher::new(d(), fd, 1.0, ml, mh, 0.05).core),
        ("cust.05", CustomerBleedMatcher::new(d().with_customer_bleed(CustomerBleed::share(0.05)), fd, 1.0, ml, mh, 0.0).core),
        ("cool.03", TurbineCoolingMatcher::new(d().with_turbine_cooling(TurbineCooling::new(0.03, 0.03)), fd, 1.0, ml, mh, 0.0).core),
        ("genHP.05", ShaftOfftakeMatcher::new(d().with_shaft_offtake(ShaftOfftake::hp(Demand::Power(0.05))), fd, 1.0, ml, mh, 0.0).core),
        ("genLP.05", ShaftOfftakeMatcher::new(d().with_shaft_offtake(ShaftOfftake::lp(Demand::Power(0.05))), fd, 1.0, ml, mh, 0.0).core),
    ];
    for (vn, core) in variants.iter() {
        for (fname, fl) in flights.iter() {
            for tt4 in [650.0, 700.0, 750.0, 800.0, 1000.0] {
                let b = &core.base;
                b.guard_off.set(true);
                let pin = core.try_match_point(fl, tt4);
                b.guard_off.set(false);
                b.below_choke.set(true);
                let on = core.try_match_point(fl, tt4);
                b.below_choke.set(false);
                let (Ok(pin), Ok(on)) = (pin, on) else { println!("{vn} {fname} {tt4:.0}: pin or ON refused, skipped"); continue };
                let lo0 = pin.base.pi_lpt;
                let (Some(rlo), Some(rhi)) = (resid(core, fl, tt4, lo0), resid(core, fl, tt4, PI_TOP)) else {
                    println!("{vn} {fname} {tt4:.0}: bracket end not evaluable"); continue };
                let choked = on.base.m9 >= 1.0 - 1e-9;
                if rlo * rhi > 0.0 {
                    println!("{vn} {fname} {tt4:.0} {} : no sign change (rlo {rlo:+.2e}, rhi {rhi:+.2e}); ON pi {:.12}",
                             if choked { "CHOKED" } else { "unchoked" }, on.base.pi_lpt);
                    continue;
                }
                let (mut lo, mut hi) = (lo0, PI_TOP);
                for _ in 0..200 {
                    let mid = 0.5 * (lo + hi);
                    match resid(core, fl, tt4, mid) { Some(r) if r * rlo > 0.0 => lo = mid, Some(_) => hi = mid, None => { hi = mid } }
                    if hi - lo <= 1e-14 { break }
                }
                let root = 0.5 * (lo + hi);
                println!("{vn} {fname} {tt4:.0} {} : indep pi {root:.12} ON pi {:.12} diff {:+.2e} | pin {:.12} rlo {rlo:+.2e}",
                         if choked { "CHOKED" } else { "unchoked" }, on.base.pi_lpt, root - on.base.pi_lpt, lo0);
            }
        }
    }
}
