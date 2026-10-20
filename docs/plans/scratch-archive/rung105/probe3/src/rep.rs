// Rung 105 — REPORTED, NOT SCORED (anchor § 3 "Reported", § 5): run after scoring, against the shipped code.
use turbojet::atmosphere::standard;
use turbojet::blade_speed::shape_maps;
use turbojet::components::{ram_recovery, try_sonic_throat};
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::hp_vane::HpVane;
use turbojet::matcher::OffDesignMatcher;
use turbojet::sandbox::{fly, fly_solver, FlySettings, GasModel, Settings};
use turbojet::sandbox_blades::{lever_json, BladeSettings};
use turbojet::sandbox_controls::{controls, ControlsSettings};
use turbojet::sandbox_cooling::{cooling_point, CoolingSettings};
use turbojet::sandbox_transient::{slam, SlamSettings, ThrottleMode};
use turbojet::turbine_law::TurbineLaw;
use turbojet::two_spool::{TwoSpoolMapCore, TwoSpoolMapMatcher};

fn flight(dt: f64, h: f64, m0: f64) -> FlightCondition { let (t, p) = standard(h); FlightCondition::new(t + dt, p, m0) }
fn rel(a: f64, b: f64) -> f64 { a / b - 1.0 }
fn full(m: TwoSpoolMapMatcher) -> TwoSpoolMapCore { match m { TwoSpoolMapMatcher::Full(c) => c, _ => unreachable!() } }
const R30: HpVane = HpVane::Rows { reaction: 0.3 };

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    // (1) H1's margin: how close the reaction-0.3 vane came to choking — its interstage pressure over the sonic one.
    println!("=== H1's margin at reaction 0.3: min p1/p* over S1 (Fly it) and S3 (Controls) ===");
    let mut m1 = f64::INFINITY;
    for gm in [GasModel::ThermallyPerfect, GasModel::Equilibrium] {
        for fl in [flight(20.0, 0.0, 0.60), flight(0.0, 2000.0, 0.50), flight(0.0, 5000.0, 0.70)] {
            for i in 0..6 {
                let tt4 = 900.0 + 100.0 * i as f64;
                let s = FlySettings { gas: gm, tt4, t0: fl.t0, p0: fl.p0, m0: fl.m0, hp_vane: R30, ..FlySettings::defaults() };
                let o = fly(&s).unwrap();
                let st = |k: &str| o.result.stations.iter().find(|(l, _)| *l == k).unwrap().1;
                let (s4, s5) = (st("4"), st("5"));
                let sv = fly_solver(&s);
                let mm = &sv.inner.inner;
                let owned = mm.try_working_gas(s4.far, s4.tt, s4.pt).unwrap();
                let g = owned.as_ref().unwrap_or(mm.gas());
                let cap = mm.hp_cap.get();
                let v = turbojet::hp_vane::Vane::at(g, s4.tt, s4.far).unwrap();
                let p1 = cap.p1_at(&v, g, s5.pt / s4.pt, s4.tt, s4.far, o.point.eta_t).unwrap();
                m1 = m1.min(p1 / v.p_star);
            }
        }
    }
    println!("S1: min p1/p* = {m1:.4}");
    let _ = try_sonic_throat;

    // (2) S3 thrust against both references, and the HP share's spread.
    println!("=== S3: thrust under the law against the old model and rung 104's model ===");
    let (mut fo_lo, mut fo_hi, mut fq_lo, mut fq_hi) = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
    for shape in ["steep", "flat-eta", "flow/press"] {
        let bs = BladeSettings::defaults();
        let (ml, mh) = shape_maps(shape).unwrap();
        let c = full(TwoSpoolMapMatcher::new(bs.design(), bs.flight(), 1.0, ml, mh));
        let run = |fl: &FlightCondition, tt4: f64, lp: TurbineLaw, hp: HpVane, old: bool| {
            c.base.turbine_law.set(lp); c.base.set_hp_vane(hp); c.base.pin_past_choke.set(old); c.base.below_choke.set(!old);
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| c.try_match_point(fl, tt4))).ok().and_then(|x| x.ok());
            c.base.pin_past_choke.set(false); c.base.below_choke.set(false); c.base.set_hp_vane(HpVane::Pin); r
        };
        for fl in [flight(20.0, 0.0, 0.60), flight(0.0, 2000.0, 0.50), flight(-10.0, 0.0, 0.40)] {
            for i in 0..6 {
                let tt4 = 575.0 + 50.0 * i as f64;
                let (Some(o), Some(q), Some(e)) = (run(&fl, tt4, TurbineLaw::Pin, HpVane::Pin, true),
                    run(&fl, tt4, TurbineLaw::Ellipse, HpVane::Pin, false), run(&fl, tt4, TurbineLaw::Ellipse, R30, false)) else { continue };
                if !(e.base.m9 <= 0.99 && e.base.thrust >= 50.0) { continue }
                let (a, b) = (rel(e.base.thrust, o.base.thrust), rel(e.base.thrust, q.base.thrust));
                fo_lo = fo_lo.min(a); fo_hi = fo_hi.max(a); fq_lo = fq_lo.min(b); fq_hi = fq_hi.max(b);
            }
        }
    }
    println!("S3 thrust vs the old model {:+.2} .. {:+.2} %, vs rung 104's model {:+.3} .. {:+.3} %",
             100.0 * fo_lo, 100.0 * fo_hi, 100.0 * fq_lo, 100.0 * fq_hi);

    // (3) One shaft below choke at S1's flights (the Fly-it page), 600-850 K.
    println!("=== one shaft below choke (Fly it, thermally perfect), reaction 0.3 against the pin ===");
    for (fname, fl) in [("SL+20 M0.60", flight(20.0, 0.0, 0.60)), ("2km M0.50", flight(0.0, 2000.0, 0.50)), ("5km M0.70", flight(0.0, 5000.0, 0.70))] {
        for i in 0..6 {
            let tt4 = 600.0 + 50.0 * i as f64;
            let f = |law| fly(&FlySettings { tt4, t0: fl.t0, p0: fl.p0, m0: fl.m0, hp_vane: law, ..FlySettings::defaults() }).ok();
            match (f(HpVane::Pin), f(R30)) {
                (Some(a), Some(b)) if !a.point.choked || !b.point.choked =>
                    println!("{fname} {tt4}: unchoked; pi_c {:+.3} %, mdot {:+.3} %, N {:+.3} %, F {:+.3} %", 100.0 * rel(b.point.pi_c, a.point.pi_c),
                             100.0 * rel(b.point.mdot_air, a.point.mdot_air), 100.0 * rel(b.point.nu, a.point.nu), 100.0 * rel(b.point.thrust, a.point.thrust)),
                (Some(_), Some(_)) => {}
                (a, b) => println!("{fname} {tt4}: pin {} law {}", a.is_some(), b.is_some()),
            }
        }
    }

    // (4) G4's double root under each law: rung 31/33's matcher at 318 K / sea level / Mach 0.30.
    println!("=== G4: rung 31/33 at 318 K / SL / M0.30 — which throttles solve, per law ===");
    let d = Settings::defaults();
    let gas = turbojet::sandbox_blades::BladeGas::Perfect.gas();
    let m = OffDesignMatcher::new(build_turbojet(gas, d.pi_c, d.tt4, 50_000.0, Losses { nozzle_convergent: true, p_exit: None, ..d.losses() }),
                                  FlightCondition::new(250.0, 50_000.0, 0.85), 1.0);
    let fl = FlightCondition::new(318.15, 101_325.0, 0.3);
    let _ = ram_recovery;
    for law in [HpVane::Pin, HpVane::Rows { reaction: 0.10 }, R30, HpVane::Rows { reaction: 0.5 }] {
        m.set_hp_vane(law);
        let ok: Vec<String> = (0..9).map(|i| 600.0 + 50.0 * i as f64)
            .map(|t| format!("{t}:{}", if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| m.match_point(&fl, t))).is_ok() { "ok" } else { "--" }))
            .collect();
        println!("{law:?}: {}", ok.join(" "));
    }

    // (5) The slams with the law on.
    println!("=== § 5: the slams, reaction 0.3 against the pin ===");
    let sd = SlamSettings::defaults();
    for (name, s) in [("Fly-it slam, temperature", sd), ("Fly-it slam, fuel", SlamSettings { mode: ThrottleMode::Fuel, ..sd })] {
        let a = slam(&s).unwrap();
        let b = slam(&SlamSettings { fly: FlySettings { hp_vane: R30, ..s.fly }, ..s }).unwrap();
        let pk = |o: &turbojet::sandbox_transient::SlamOutcome| o.points.iter().map(|p| p.tt4).fold(f64::NEG_INFINITY, f64::max);
        let (ea, eb) = (a.points.last().unwrap(), b.points.last().unwrap());
        println!("{name}: peak Tt4 {:.2} -> {:.2} K, end N {:.5} -> {:.5}, stop {} -> {}", pk(&a), pk(&b), ea.nu, eb.nu, a.stop.is_some(), b.stop.is_some());
    }
    for (name, s) in [("Controls default", ControlsSettings::defaults()),
                      ("Controls below choke", ControlsSettings { t0: 288.15 + 40.0, p0: 101_325.0, m0: 0.85, from: 700.0, to: 1400.0, ..ControlsSettings::defaults() })] {
        let (_, a) = controls(&s).unwrap();
        let (_, b) = controls(&ControlsSettings { hp_vane: R30, ..s }).unwrap();
        let pk = |o: &turbojet::sandbox_controls::ControlsOutcome| o.points.iter().map(|p| p.tt4).fold(f64::NEG_INFINITY, f64::max);
        let (ea, eb) = (a.points.last().unwrap(), b.points.last().unwrap());
        let mx = a.points.iter().zip(&b.points).map(|(x, y)| rel(y.nu_hp, x.nu_hp).abs().max(rel(y.nu_lp, x.nu_lp).abs())).fold(0.0, f64::max);
        println!("{name}: peak Tt4 {:.3} -> {:.3} K, end N_L {:.5} -> {:.5}, N_H {:.5} -> {:.5}, largest speed change {:.3} %",
                 pk(&a), pk(&b), ea.nu_lp, eb.nu_lp, ea.nu_hp, eb.nu_hp, 100.0 * mx);
    }

    // (6) A seeded crash map of each view's requests with the law on at random reactions.
    println!("=== § 5: crash map, the law on at random reactions ===");
    let mut seed: u64 = 105;
    let mut rnd = || { seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17; (seed >> 11) as f64 / (1u64 << 53) as f64 };
    // Each request on the pin and on the law: a panic the pin also has is the old model's own.
    let (mut runs, mut new_panics, mut old_panics, mut refused) = (0, 0, 0, 0);
    let mut tally = |pin: std::thread::Result<bool>, law: std::thread::Result<bool>| {
        runs += 1;
        match (pin.is_err(), law) { (false, Err(_)) => new_panics += 1, (true, Err(_)) => old_panics += 1, (_, Ok(false)) => refused += 1, _ => {} }
    };
    for _ in 0..80 {
        let law = HpVane::Rows { reaction: 0.05 + 0.55 * rnd() };
        let fl = FlySettings { tt4: 600.0 + 1200.0 * rnd(), t0: 220.0 + 100.0 * rnd(), p0: 20_000.0 + 85_000.0 * rnd(), m0: 0.05 + 1.5 * rnd(),
                               gas: if rnd() < 0.5 { GasModel::Perfect } else { GasModel::ThermallyPerfect }, ..FlySettings::defaults() };
        tally(std::panic::catch_unwind(|| fly(&fl).is_ok()), std::panic::catch_unwind(|| fly(&FlySettings { hp_vane: law, ..fl }).is_ok()));
        let c = ControlsSettings { from: 600.0 + 500.0 * rnd(), to: 900.0 + 600.0 * rnd(), t0: 230.0 + 90.0 * rnd(), p0: 30_000.0 + 75_000.0 * rnd(),
                                   m0: 0.1 + 0.9 * rnd(), cool_vane: if rnd() < 0.3 { 0.04 } else { 0.0 }, ..ControlsSettings::defaults() };
        tally(std::panic::catch_unwind(|| controls(&c).is_ok()), std::panic::catch_unwind(|| controls(&ControlsSettings { hp_vane: law, ..c }).is_ok()));
        let b = BladeSettings::defaults();
        let t = 600.0 + 900.0 * rnd();
        tally(std::panic::catch_unwind(|| lever_json(&b, t).is_ok()),
              std::panic::catch_unwind(|| lever_json(&BladeSettings { hp_vane: law, ..b }, t).is_ok()));
        let k = CoolingSettings::defaults();
        let t = 700.0 + 800.0 * rnd();
        tally(std::panic::catch_unwind(|| cooling_point(&k, t).is_ok()),
              std::panic::catch_unwind(|| cooling_point(&CoolingSettings { hp_vane: law, ..k }, t).is_ok()));
    }
    println!("crash map: {runs} requests, {new_panics} NEW panics, {old_panics} the old model's own, {refused} refused in words");
}
