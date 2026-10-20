// Rung 105 probe G1: is the HP turbine's nozzle guide vane (A4) choked on the running line?
// The vane is choked iff its exit kinetic energy reaches the sonic one: c1^2/2 >= c*^2/2, c* from the gas's own
// sonic throat at (Tt4, pt4) (loss-independent: total enthalpy is conserved in the vane, and M=1 is a static-T
// condition). With reaction R (static-enthalpy share of the rotor) and stage inlet/exit KE equal (repeating
// stage), c1^2/2 = KE4 + (1-R)*dh0. So choked iff R <= R* = 1 - (c*^2/2 - KE4)/dh0.
// dh0 = the first stage's total-enthalpy drop: the whole HP turbine (one stage) or half of it (two equal stages).
// KE4 = 0 (no inlet KE) and the burner exit at Mach 0.15.
use turbojet::components::try_sonic_throat;
use turbojet::engine::FlightCondition;
use turbojet::gas::Gas;
use turbojet::map::ComponentMap;
use turbojet::sandbox::{fly, FlySettings, GasModel, MapShape};
use turbojet::sandbox_blades::{rig_losses, BladeGas};
use turbojet::turbine_law::TurbineLaw;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolMapMatcher};

struct W { min1: f64, min2: f64, max1: f64 }

fn rstar(gas: &Gas, tt4: f64, pt4: f64, f: f64, tt_out: f64, w: &mut W, tag: &str, extra: &str) {
    let (_ts, _ps, vs) = try_sonic_throat(gas, tt4, pt4, f).expect("sonic");
    let ke_s = 0.5 * vs * vs;
    let dh0 = gas.h_t(tt4, f) - gas.h_t(tt_out, f);
    let g = gas.gamma_t_at(tt4, f);
    let m = 0.15f64;
    let t4s = tt4 / (1.0 + 0.5 * (g - 1.0) * m * m);
    let ke4 = 0.5 * m * m * g * gas.r_t_at(f) * t4s;
    let r1 = 1.0 - ke_s / dh0;
    let r1m = 1.0 - (ke_s - ke4) / dh0;
    let r2 = 1.0 - ke_s / (0.5 * dh0);
    let r2m = 1.0 - (ke_s - ke4) / (0.5 * dh0);
    w.min1 = w.min1.min(r1);
    w.max1 = w.max1.max(r1m);
    w.min2 = w.min2.min(r2);
    println!("{tag:<38} Tt4 {tt4:6.0} {extra}  dh0/KE* {:.3}  R*(1 stage) {r1:+.3} (M4 .15: {r1m:+.3})  R*(2 stages) {r2:+.3} ({r2m:+.3})",
             dh0 / ke_s);
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let flights = [("design M.85 50kPa", FlightCondition::new(250.0, 50_000.0, 0.85)),
                   ("SL M.05", FlightCondition::new(288.15, 101_325.0, 0.05)),
                   ("hot M.3", FlightCondition::new(318.15, 101_325.0, 0.3)),
                   ("M2 11km", FlightCondition::new(216.65, 22_632.0, 2.0))];
    let tt4s = [500.0, 550.0, 600.0, 650.0, 700.0, 800.0, 900.0, 1000.0, 1100.0, 1200.0, 1300.0, 1400.0, 1500.0, 1600.0, 1700.0];

    println!("=== ONE SHAFT (Fly it: the opening design, pi_c 10, Tt4 1500) ===");
    let mut w = W { min1: 9.0, min2: 9.0, max1: -9.0 };
    for (gn, gm) in [("perfect", GasModel::Perfect), ("TPG", GasModel::ThermallyPerfect), ("equilibrium", GasModel::Equilibrium)] {
        for map in [MapShape::Flow, MapShape::Pressure] {
            for (fname, fl) in flights.iter() {
                for &tt4 in tt4s.iter() {
                    let mut s = FlySettings::defaults();
                    s.gas = gm; s.map = map; s.tt4 = tt4; s.t0 = fl.t0; s.p0 = fl.p0; s.m0 = fl.m0;
                    let tag = format!("1sh {gn} {map:?} {fname}");
                    match std::panic::catch_unwind(|| fly(&s)).unwrap_or_else(|p| Err(turbojet::sandbox::DoesNotRun(format!("PANIC {}", p.downcast_ref::<String>().cloned().unwrap_or_default())))) {
                        Ok(o) => {
                            let st = |k: &str| o.result.stations.iter().find(|(l, _)| *l == k).unwrap().1;
                            let (s4, s5) = (st("4"), st("5"));
                            let gas = s.gas.gas();
                            if gas.is_equilibrium() { gas.freeze_equilibrium(s4.far, s4.tt, s4.pt); }
                            let extra = format!("{} pi_t {:.3}", if o.point.choked { "CH" } else { "UN" }, s4.pt / s5.pt);
                            rstar(&gas, s4.tt, s4.pt, s4.far, s5.tt, &mut w, &tag, &extra);
                        }
                        Err(e) => println!("{tag:<38} Tt4 {tt4:6.0} refused: {}", &e.0[..e.0.len().min(90)]),
                    }
                }
            }
        }
    }
    println!("ONE SHAFT: lowest R*(1 stage) {:+.3}, highest R*(1 stage, M4 .15) {:+.3}, lowest R*(2 stages) {:+.3}", w.min1, w.max1, w.min2);

    println!("=== TWO SHAFTS (Blades/Controls rig: pi_lpc 3, pi_hpc 6, Tt4 1500), below choke ON ===");
    let mut w = W { min1: 9.0, min2: 9.0, max1: -9.0 };
    let f = ComponentMap::flat();
    let maps = [("flat", f, f),
                ("mixed", ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f },
                          ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f })];
    for (gn, gas) in [("perfect", BladeGas::Perfect.gas()), ("TPG", Gas::thermally_perfect()), ("equilibrium", Gas::reacting_equilibrium())] {
        for law in [TurbineLaw::Ellipse, TurbineLaw::Pin] {
            for (mn, ml, mh) in maps.iter() {
                let design = build_two_spool_turbojet(gas.clone(), 3.0, 6.0, 1500.0, flights[0].1.p0, rig_losses()).with_turbine_law(law);
                let m = TwoSpoolMapMatcher::new(design, flights[0].1, 1.0, *ml, *mh);
                m.core().base.below_choke.set(true);
                for (fname, fl) in flights.iter() {
                    for &tt4 in tt4s.iter() {
                        let tag = format!("2sh {gn} {} {mn} {fname}", law.key());
                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| m.core().try_match_point(fl, tt4))).unwrap_or_else(|p| Err(turbojet::gas::Abort(format!("PANIC {}", p.downcast_ref::<String>().cloned().unwrap_or_default())))) {
                            Ok(p) => {
                                let r = &p.base;
                                let (s4, s45, s5) = (*r.station("4"), *r.station("45"), *r.station("5"));
                                let g = m.core().base.gas();
                                if g.is_equilibrium() { g.freeze_equilibrium(s4.far, s4.tt, s4.pt); }
                                let p9 = s5.pt; let _ = p9;
                                let extra = format!("pi_HPT {:.3} pi_LPT {:.3}", s4.pt / s45.pt, s45.pt / s5.pt);
                                rstar(g, s4.tt, s4.pt, s4.far, s45.tt, &mut w, &tag, &extra);
                            }
                            Err(e) => println!("{tag:<38} Tt4 {tt4:6.0} refused: {}", &e.0[..e.0.len().min(90)]),
                        }
                    }
                }
            }
        }
    }
    println!("TWO SHAFTS: lowest R*(1 stage) {:+.3}, highest R*(1 stage, M4 .15) {:+.3}, lowest R*(2 stages) {:+.3}", w.min1, w.max1, w.min2);
}
