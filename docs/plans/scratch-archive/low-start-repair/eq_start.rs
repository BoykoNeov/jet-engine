// Throwaway probe: where does rung 40's equilibrium fail from its full-speed start, and does a
// start from the steady matcher's speeds rescue it?
use std::collections::BTreeMap;
use turbojet::atmosphere::standard;
use turbojet::engine::FlightCondition;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::{control_maps, CONTROL_SHAPES};
use turbojet::two_spool_transient::TwoSpoolTransientCore;

fn kind(m: &str) -> String {
    for k in ["does not bracket", "did not converge", "off the", "below idle", "cannot pass", "efficiency"] {
        if m.contains(k) { return k.to_string(); }
    }
    m.chars().take(70).collect()
}

#[test]
fn probe() {
    let bs = BladeSettings::defaults();
    let mut flights: Vec<(String, FlightCondition)> = vec![("design".into(), bs.flight())];
    for (alt, dt) in [(0.0, 0.0), (0.0, 40.0), (0.0, -30.0), (5000.0, 0.0), (11000.0, 0.0)] {
        let (t, p) = standard(alt);
        for m0 in [0.0, 0.3, 0.85, 1.6, 2.0] {
            flights.push((format!("{alt}m dT{dt} M{m0}"), FlightCondition::new(t + dt, p, m0)));
        }
    }
    let mut tally: BTreeMap<String, usize> = BTreeMap::new();
    for shape in CONTROL_SHAPES.iter() {
        for below in [true, false] {
            let (ml, mh) = control_maps(shape).unwrap();
            let tr = TwoSpoolTransientCore::new(bs.design(), bs.flight(), 1.0, ml, mh, 1.0);
            tr.inner.base.below_choke.set(below);
            for (name, fl) in &flights {
                let mut tt4 = 500.0;
                while tt4 <= 1600.0 + 1e-9 {
                    let key;
                    match tr.try_equilibrium(fl, tt4, None) {
                        Ok(_) => key = "A solves from full speed".to_string(),
                        Err(e) => {
                            let k0 = kind(&e.0);
                            match tr.inner.try_match_point(fl, tt4) {
                                Err(e2) => key = format!("D full:[{k0}] steady refuses:[{}]", kind(&e2.0)),
                                Ok(m) => {
                                    let s = (m.n_lp_ratio, m.n_hp_ratio);
                                    match tr.try_equilibrium(fl, tt4, Some(s)) {
                                        Ok((i, ex, it)) => {
                                            let d = (i.nu_lp - s.0).abs().max((i.nu_hp - s.1).abs());
                                            let mut pert = String::new();
                                            for f in [0.97, 1.03] {
                                                match tr.try_equilibrium(fl, tt4, Some((s.0 * f, s.1 * f))) {
                                                    Ok((j, _, _)) => {
                                                        let dj = (j.nu_lp - s.0).abs().max((j.nu_hp - s.1).abs());
                                                        pert += &format!(" x{f}:{dj:.1e}");
                                                    }
                                                    Err(e3) => pert += &format!(" x{f}:ERR[{}]", kind(&e3.0)),
                                                }
                                            }
                                            println!("B {shape} below={below} {name} {tt4} full:[{k0}] seeded ok {ex:?} it={it} d={d:.1e}{pert} nu=({:.3},{:.3})", s.0, s.1);
                                            key = format!("B full:[{k0}] rescued");
                                        }
                                        Err(e3) => {
                                            println!("C {shape} below={below} {name} {tt4} full:[{k0}] seeded:[{}]", kind(&e3.0));
                                            key = format!("C full:[{k0}] seeded fails:[{}]", kind(&e3.0));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    *tally.entry(format!("below={below} {key}")).or_default() += 1;
                    tt4 += 20.0;
                }
            }
        }
    }
    for (k, v) in &tally { println!("TALLY {v:6} {k}"); }
}
