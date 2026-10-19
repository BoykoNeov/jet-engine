// Rung 103 probe 2 (against the throwaway worktree's branch): every steady variant, OFF / ON / PIN (guard skipped).
use std::time::Instant;
use turbojet::atmosphere::standard;
use turbojet::bleed::TwoSpoolBleedMatcher;
use turbojet::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
use turbojet::engine::FlightCondition;
use turbojet::gas::Abort;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::control_maps;
use turbojet::shaft_offtake::{Demand, ShaftOfftake, ShaftOfftakeMatcher};
use turbojet::turbine_cooling::{TurbineCooling, TurbineCoolingMatcher};
use turbojet::two_spool::{TwoSpoolMapCore, TwoSpoolMapMatcher, TwoSpoolMapResult};

fn words(e: &Abort) -> String {
    let m = &e.0;
    if m.contains("UNCHOKED") { "UNCHOKED".into() } else if m.contains("rung-103") { "SUB-IDLE".into() }
    else if m.contains("rung-97") { "NO-PASS".into() } else { format!("ERR:{}", &m[..m.len().min(40)]) }
}
fn line(r: &Result<TwoSpoolMapResult, Abort>) -> String {
    match r {
        Ok(x) => format!("F {:8.3} nL {:.6} nH {:.6} phL {:.5} phH {:.5} piLT {:.6} M9 {:.4}",
                         x.base.thrust, x.n_lp_ratio, x.n_hp_ratio, x.phi_lp, x.phi_hp, x.base.pi_lpt, x.base.m9),
        Err(e) => words(e),
    }
}
fn same(a: &TwoSpoolMapResult, b: &TwoSpoolMapResult) -> bool {
    a.base.thrust.to_bits() == b.base.thrust.to_bits() && a.n_lp_ratio.to_bits() == b.n_lp_ratio.to_bits()
        && a.phi_hp.to_bits() == b.phi_hp.to_bits() && a.base.pi_lpc.to_bits() == b.base.pi_lpc.to_bits()
}

fn main() {
    let bs = BladeSettings::defaults();
    let fd = bs.flight();
    let (t, p) = standard(0.0);
    let flights = [("SL+40 M0.85", FlightCondition::new(t + 40.0, p, 0.85)),
                   ("SL std M0.30", FlightCondition::new(t, p, 0.30)),
                   ("design", fd)];
    for shape in ["flow/press", "tilted"] {
        let (ml, mh) = control_maps(shape).unwrap();
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
                println!("\n=== {shape} | {vn} | {fname}");
                let mut tt4 = 550.0;
                while tt4 <= 900.0 + 1e-9 {
                    let b = &core.base;
                    let t0 = Instant::now();
                    let off = core.try_match_point(fl, tt4);
                    let t_off = t0.elapsed().as_secs_f64();
                    b.below_choke.set(true);
                    let t1 = Instant::now();
                    let on = core.try_match_point(fl, tt4);
                    let t_on = t1.elapsed().as_secs_f64();
                    b.retry_any.set(true);
                    let rt = core.try_match_point(fl, tt4);
                    b.retry_any.set(false);
                    let rtag = match (&on, &rt) { (Ok(a), Ok(c)) if same(a, c) => String::new(), (Err(a), Err(c)) if a.0 == c.0 => String::new(), _ => format!(" | RETRY {}", line(&rt)) };
                    b.below_choke.set(false);
                    assert!(b.lp_ctx.get().is_none());
                    b.guard_off.set(true);
                    let pin = core.try_match_point(fl, tt4);
                    b.guard_off.set(false);
                    let tag = match (&off, &on) {
                        (Ok(a), Ok(c)) => if same(a, c) { "ON==OFF bits" } else { "ON!=OFF ***" },
                        _ => "",
                    };
                    println!("Tt4 {tt4:4.0} OFF {:9} | ON {} | PIN {} | cost {:.1}x {tag}{rtag}",
                             match &off { Ok(_) => "ok".into(), Err(e) => words(e) }, line(&on), line(&pin), t_on / t_off);
                    tt4 += 50.0;
                }
            }
        }
    }
}
