// Probe: every below-idle point the FULL-SPEED start returns — is it the only root at its throttle?
// 2-D start grid; a distinct root with positive thrust would mean "refuse" must try the ladder instead.
use turbojet::atmosphere::standard;
use turbojet::customer_bleed::CustomerBleed;
use turbojet::engine::FlightCondition;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::{control_maps, CONTROL_SHAPES};
use turbojet::turbine_cooling::TurbineCooling;
use turbojet::bleed_transient::{build_scheduled_bleed, LeverArm};
use turbojet::stator_transient::ScheduledStatorTransient;

fn main() {
    let bs = BladeSettings::defaults();
    let mut flights: Vec<(String, FlightCondition)> = vec![("design".into(), bs.flight())];
    for m0 in [1.6, 2.0, 2.6] { let f = bs.flight(); flights.push((format!("designalt M{m0}"), FlightCondition::new(f.t0, f.p0, m0))); }
    for (alt, dt) in [(0.0, 0.0), (0.0, 40.0), (0.0, -30.0), (5000.0, 0.0), (11000.0, 0.0)] {
        let (t, p) = standard(alt);
        for m0 in [0.0, 0.3, 0.85, 1.6, 2.0, 2.6] {
            flights.push((format!("{alt}m dT{dt} M{m0}"), FlightCondition::new(t + dt, p, m0)));
        }
    }
    let engines = [("bare", bs.design()),
                   ("cust5", bs.design().with_customer_bleed(CustomerBleed::share(0.05))),
                   ("cool", bs.design().with_turbine_cooling(TurbineCooling::new(0.05, 0.03)))];
    let (mut neg, mut multi, mut pos_other) = (0, 0, 0);
    let skip = std::env::args().nth(1).unwrap_or_default();
    for (ename, design) in engines.iter().filter(|(n, _)| !skip.contains(n)) {
        for shape in CONTROL_SHAPES.iter() {
            let (ml, mh) = control_maps(shape).unwrap();
            let core = match build_scheduled_bleed(design.clone(), bs.flight(), 1.0, Some(ml), Some(mh), 1.0, &LeverArm::default()) { ScheduledStatorTransient::Full(c) => c, _ => unreachable!() };
            let tr = &core.fuel.inner;
            for (name, fl) in &flights {
                let mut tt4 = 500.0;
                while tt4 <= 1200.0 + 1e-9 {
                    if let Ok((i, _, _)) = tr.try_equilibrium(fl, tt4, None) {
                        let sp = i.sp_thrust_inlet.unwrap_or(i.sp_thrust);
                        if !(sp > 0.0) {
                            neg += 1;
                            let mut roots: Vec<(f64, f64, f64)> = vec![(i.nu_lp, i.nu_hp, sp)];
                            for il in 0..8 { for ih in 0..6 {
                                let st = (0.3 + 0.1 * il as f64, 0.5 + 0.1 * ih as f64);
                                if let Ok((j, _, _)) = tr.try_equilibrium(fl, tt4, Some(st)) {
                                    let r = (j.nu_lp, j.nu_hp, j.sp_thrust_inlet.unwrap_or(j.sp_thrust));
                                    if !roots.iter().any(|q| (q.0 - r.0).abs() < 1e-6 && (q.1 - r.1).abs() < 1e-6) { roots.push(r); }
                                }
                            }}
                            if roots.len() > 1 {
                                multi += 1;
                                if roots.iter().any(|r| r.2 > 0.0) { pos_other += 1; }
                                println!("MULTI {ename} {shape} {name} {tt4}: {roots:?}");
                            }
                            println!("NEG {ename} {shape} {name} {tt4} sp={sp:.3} nu=({:.4},{:.4})", i.nu_lp, i.nu_hp);
                        }
                    }
                    tt4 += 20.0;
                }
            }
        }
    }
    println!("below-idle full-speed points {neg}; with another root {multi}; another root that runs {pos_other}");
}
