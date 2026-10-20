// Rung 105 PRE-FLIGHT (anchor § 3): each scored cell's CLASS only — solves / choked / near idle — never a scored number.
use std::time::Instant as T;
use turbojet::atmosphere::standard;
use turbojet::blade_speed::shape_maps;
use turbojet::engine::FlightCondition;
use turbojet::gas::Gas;
use turbojet::hp_vane::HpVane;
use turbojet::sandbox::{fly, FlySettings, GasModel};
use turbojet::sandbox_blades::{rig_losses, BladeSettings};
use turbojet::turbine_law::TurbineLaw;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolMapCore, TwoSpoolMapMatcher};

fn flight(dt: f64, h: f64, m0: f64) -> FlightCondition { let (t, p) = standard(h); FlightCondition::new(t + dt, p, m0) }
fn laws() -> [HpVane; 4] { [HpVane::Pin, HpVane::Rows { reaction: 0.10 }, HpVane::law(), HpVane::Rows { reaction: 0.5 }] }
fn full(m: TwoSpoolMapMatcher) -> TwoSpoolMapCore { match m { TwoSpoolMapMatcher::Full(c) => c, _ => unreachable!() } }

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let s1_flights = [("SL+20 M0.60", flight(20.0, 0.0, 0.60)), ("2km M0.50", flight(0.0, 2000.0, 0.50)), ("5km M0.70", flight(0.0, 5000.0, 0.70))];
    println!("=== S1 (Fly-it page, map flow): class per law [pin, 0.10, 0.3, 0.5]: C choked / U unchoked / R refused ===");
    for (gn, gm) in [("TPG", GasModel::ThermallyPerfect), ("EQ", GasModel::Equilibrium)] {
        for (fname, fl) in s1_flights.iter() {
            for i in 0..6 {
                let tt4 = 900.0 + 100.0 * i as f64;
                let t = T::now();
                let cls: Vec<&str> = laws().iter().map(|&law| {
                    let s = FlySettings { gas: gm, tt4, t0: fl.t0, p0: fl.p0, m0: fl.m0, hp_vane: law, ..FlySettings::defaults() };
                    match std::panic::catch_unwind(|| fly(&s)) { Ok(Ok(o)) => if o.point.choked { "C" } else { "U" }, _ => "R" }
                }).collect();
                println!("S1 {gn} {fname} {tt4}: {:?}  ({:.1} s)", cls, t.elapsed().as_secs_f64());
            }
        }
    }
    println!("=== S2 (rung 39 rig, equilibrium gas): class per law ===");
    let s2_flights = [("2km M0.80", flight(0.0, 2000.0, 0.80)), ("5km M0.70", flight(0.0, 5000.0, 0.70))];
    for shape in ["steep", "flow/press"] {
        let (ml, mh) = shape_maps(shape).unwrap();
        let c = full(TwoSpoolMapMatcher::new(build_two_spool_turbojet(Gas::reacting_equilibrium(), 3.0, 6.0, 1500.0, 50_000.0, rig_losses()),
                                             FlightCondition::new(250.0, 50_000.0, 0.85), 1.0, ml, mh));
        for (fname, fl) in s2_flights.iter() {
            for i in 0..7 {
                let tt4 = 800.0 + 100.0 * i as f64;
                let t = T::now();
                let cls: Vec<&str> = laws().iter().map(|&law| {
                    c.base.set_hp_vane(law);
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| c.try_match_point(fl, tt4))) {
                        Ok(Ok(r)) => if r.base.m9 >= 0.999 { "C" } else { "U" }, _ => "R" }
                }).collect();
                println!("S2 {shape} {fname} {tt4}: {:?}  ({:.1} s)", cls, t.elapsed().as_secs_f64());
            }
        }
    }
    println!("=== S3 (Controls engine, perfect gas): old / r104 / law 0.3 solve? law unchoked (M9<=0.99)? near idle (<50 N per kg/s)? ===");
    let s3_flights = [("SL+20 M0.60", flight(20.0, 0.0, 0.60)), ("2km M0.50", flight(0.0, 2000.0, 0.50)), ("SL-10 M0.40", flight(-10.0, 0.0, 0.40))];
    let (mut scored, mut total) = (0, 0);
    for shape in ["steep", "flat-eta", "flow/press"] {
        let bs = BladeSettings::defaults();
        let (ml, mh) = shape_maps(shape).unwrap();
        let c = full(TwoSpoolMapMatcher::new(bs.design(), bs.flight(), 1.0, ml, mh));
        for (fname, fl) in s3_flights.iter() {
            for i in 0..6 {
                let tt4 = 575.0 + 50.0 * i as f64;
                let run = |lp: TurbineLaw, hp: HpVane, old: bool| {
                    c.base.turbine_law.set(lp); c.base.set_hp_vane(hp); c.base.pin_past_choke.set(old); c.base.below_choke.set(!old);
                    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| c.try_match_point(fl, tt4))).ok().and_then(|x| x.ok());
                    c.base.pin_past_choke.set(false); c.base.below_choke.set(false); r
                };
                let o = run(TurbineLaw::Pin, HpVane::Pin, true);
                let q = run(TurbineLaw::Ellipse, HpVane::Pin, false);
                let e = run(TurbineLaw::Ellipse, HpVane::law(), false);
                let unch = e.as_ref().is_some_and(|e| e.base.m9 <= 0.99);
                let idle = e.as_ref().is_some_and(|e| e.base.thrust < 50.0 * c.base.mdot_air_design);
                let ok = o.is_some() && q.is_some() && e.is_some() && unch && !idle;
                total += 1;
                if ok { scored += 1; }
                println!("S3 {shape} {fname} {tt4}: old {} r104 {} law {} unchoked {} near-idle {} => {}", o.is_some(), q.is_some(), e.is_some(), unch, idle, if ok { "SCORED" } else { "-" });
            }
        }
    }
    println!("S3: {scored} scored of {total}");
}
