// Rung 104 probe P19 — THE SORT's witness hunt. Older tests whose WITNESS (a case a crash map or a sweep found) moved
// under the default law: does the behaviour still occur, and where?
//  (a) low_start_repair: per map at the design flight, the throttle edge between "below idle" and "runs";
//  (b) the "shafts too slow" first-step stop (lean_wall, sandbox_controls);
//  (c) an endpoint the customer bleed removes, named so (sandbox_controls);
//  (d) the nozzle that cannot push the gas out at the starting throttle (sandbox_altitude);
//  (e) a thermally-perfect blade-lever point colder than the gas tables (sandbox_blades);
//  (f) rung 100's fan-speed formula with no working state below the table.
use turbojet::atmosphere::standard;
use turbojet::burner_hold::Hold;
use turbojet::mach_axis::SpeedCorr;
use turbojet::sandbox::call;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::{build, classify, control_maps, controls, lean_wall, ControlsSettings, CustomerMode, StopKind,
                                 ThrustBy, CONTROL_SHAPES};
use turbojet::two_spool_transient::TwoSpoolTransientCore;

fn main() {
    let which = std::env::args().nth(1).unwrap_or_default();
    let d = ControlsSettings::defaults();
    let (t_sl, p_sl) = standard(0.0);
    if which.is_empty() || which == "a" {
        let bs = BladeSettings::defaults();
        let fl = bs.flight();
        for shape in CONTROL_SHAPES {
            let (ml, mh) = control_maps(shape).unwrap();
            let tr = TwoSpoolTransientCore::new(bs.design(), bs.flight(), 1.0, ml, mh, 1.0);
            tr.inner.base.below_choke.set(true);
            let mut line = format!("(a) {shape:10}");
            for i in 0..14 {
                let tt4 = 460.0 + 10.0 * i as f64;
                let c = match tr.try_equilibrium(&fl, tt4, None) {
                    Ok(_) => 'R',
                    Err(e) if e.0.contains("; and below idle") => 'i',
                    Err(_) => 'x',
                };
                line += &format!(" {tt4:.0}{c}");
            }
            println!("{line}");
        }
    }
    if which.is_empty() || which == "b" {
        for shape in 0..CONTROL_SHAPES.len() {
            for (fname, t0, p0, m0) in [("SL+40 M0.85", t_sl + 40.0, p_sl, 0.85), ("SL M0.30", t_sl, p_sl, 0.30), ("SL+40 M0.30", t_sl + 40.0, p_sl, 0.30)] {
                for cust in [CustomerMode::Off, CustomerMode::Share, CustomerMode::Amount] {
                    for i in 0..10 {
                        let from = 400.0 + 10.0 * i as f64;
                        let s = ControlsSettings { shape, t0, p0, m0, customer: cust, from, ..d };
                        let Ok((sv, o)) = controls(&s) else { continue };
                        let Some(st) = o.stop.as_ref() else { continue };
                        if o.points.is_empty() && classify(&sv, st, None) == StopKind::LowSpeed {
                            println!("(b) {} {fname} {cust:?} from {from}: LowSpeed at once; start nuL {:.3}, f above the wall {}",
                                     CONTROL_SHAPES[shape], sv.start.nu_lp, sv.start.close.f > lean_wall(&sv));
                        }
                    }
                }
            }
        }
    }
    if which.is_empty() || which == "c" {
        let words = "With the customer bleed off this throttle does have a steady point, so the air the aircraft takes is what removes it: ask for less, or raise the throttle.";
        for shape in 0..CONTROL_SHAPES.len() {
            for (fname, t0, p0, m0) in [("alt", 282.44, 53737.0, 0.2925), ("design", d.t0, d.p0, d.m0), ("SL M0.30", t_sl, p_sl, 0.30)] {
                for k in 0..8 {
                    let share = 0.03 + 0.01 * k as f64;
                    for j in 0..10 {
                        let to = 500.0 + 15.0 * j as f64;
                        let s = ControlsSettings { shape, rho: 2.1287, from: 1008.0, to, ramp: 0.48, customer: CustomerMode::Share,
                                                   customer_share: share, t0, p0, m0, ..d };
                        if let Err(e) = controls(&s) {
                            if e.0.contains("does not bracket") && e.0.ends_with(words)
                                && controls(&ControlsSettings { customer: CustomerMode::Off, ..s }).is_ok() {
                                println!("(c) {} {fname} share {share:.2} to {to}", CONTROL_SHAPES[shape]);
                            }
                        }
                    }
                }
            }
        }
    }
    if which.is_empty() || which == "d" {
        for from in [500.0, 520.0, 540.0, 560.0, 580.0, 600.0, 620.0] {
            for amount in [0.0, 0.05, 0.10, 0.13, 0.16] {
                for gen in [0.0, 0.05] {
                    let g = if gen > 0.0 { format!(r#","generator":"hp","generator_power":{gen}"#) } else { String::new() };
                    let c = if amount > 0.0 { format!(r#","customer":"amount","customer_amount":{amount}"#) } else { String::new() };
                    let req = format!(r#"{{"op":"controls","controls":{{"T0":281.025575242416,"p0":110666.3721625535,"shape":"flow/press","redline_on":1{g}{c},"cool_vane":0.04,"cool_blade":0.12,"from":{from},"to":870,"ramp":0.02,"settle":3.3,"rho":2.3}}}}"#);
                    let r = call(&req);
                    if r.contains("pushed out against the air") {
                        println!("(d) from {from} amount {amount} generator {gen}: {}", r.chars().take(160).collect::<String>());
                    }
                }
            }
        }
    }
    if which.is_empty() || which == "e" {
        for (lpc, hpc, t) in [(4.3068, 13.6148, 1100.77), (4.0, 12.0, 1100.0), (4.5, 14.0, 1100.0), (5.0, 15.0, 1150.0), (3.5, 12.0, 1050.0)] {
            for k in 0..12 {
                let tt4 = 700.0 + 15.0 * k as f64;
                let r = call(&format!(r#"{{"op":"blade_lever","Tt4":{tt4},"blades":{{"gas":"thermally_perfect","pi_lpc":{lpc},"pi_hpc":{hpc},"Tt4":{t},"shape":"steep"}}}}"#));
                if r.contains("colder than the gas tables reach") {
                    println!("(e) {lpc}/{hpc}/{t} at {tt4}: {}", r.chars().take(120).collect::<String>());
                }
            }
        }
    }
    if which.is_empty() || which == "f" {
        for m0 in [0.0, 0.05, 0.1, 0.2] {
            for k in 0..8 {
                let from = 450.0 + 20.0 * k as f64;
                let s = ControlsSettings { shape: 0, from, t0: 288.15, p0: 101_325.0, m0, govern: true, hold: Hold::Thrust,
                                           thrust_by: ThrustBy::N1, speed_corr: SpeedCorr::Formula, ..d };
                match build(&s) {
                    Err(e) if e.0.contains("fan-speed formula") && e.0.contains("no working state") => println!("(f) M{m0} from {from}: refused"),
                    Err(e) => println!("(f) M{m0} from {from}: other: {}", e.0.chars().take(100).collect::<String>()),
                    Ok(_) => {}
                }
            }
        }
    }
}
