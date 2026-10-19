// Throwaway probe 3: (a) does the full-speed path already return below-idle points? (b) on cells
// the full-speed start refuses, do ALL lower starts that solve reach the same root?
use std::collections::BTreeMap;
use turbojet::atmosphere::standard;
use turbojet::engine::FlightCondition;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::{control_maps, CONTROL_SHAPES};
use turbojet::two_spool_transient::TwoSpoolTransientCore;

#[test]
fn probe_unique() {
    let bs = BladeSettings::defaults();
    let mut flights: Vec<(String, FlightCondition)> = vec![("design".into(), bs.flight())];
    for (alt, dt) in [(0.0, 0.0), (0.0, 40.0), (0.0, -30.0), (5000.0, 0.0), (11000.0, 0.0)] {
        let (t, p) = standard(alt);
        for m0 in [0.0, 0.3, 0.85, 1.6, 2.0] {
            flights.push((format!("{alt}m dT{dt} M{m0}"), FlightCondition::new(t + dt, p, m0)));
        }
    }
    let mut tally: BTreeMap<String, usize> = BTreeMap::new();
    let mut worst_spread = 0.0f64;
    let mut min_full_sp = f64::INFINITY;
    for shape in CONTROL_SHAPES.iter() {
        let (ml, mh) = control_maps(shape).unwrap();
        let tr = TwoSpoolTransientCore::new(bs.design(), bs.flight(), 1.0, ml, mh, 1.0);
        tr.inner.base.below_choke.set(true);
        for (name, fl) in &flights {
            let mut tt4 = 500.0;
            while tt4 <= 1600.0 + 1e-9 {
                match tr.try_equilibrium(fl, tt4, None) {
                    Ok((i, _, _)) => {
                        min_full_sp = min_full_sp.min(i.sp_thrust);
                        if !(i.sp_thrust > 0.0) {
                            println!("FULLNEG {shape} {name} {tt4} sp={:.3}", i.sp_thrust);
                            *tally.entry("full-speed solve below idle".into()).or_default() += 1;
                        }
                    }
                    Err(_) => {
                        let roots: Vec<(f64, f64, f64)> = [0.9, 0.8, 0.7, 0.6, 0.5, 0.4].iter()
                            .filter_map(|&s| tr.try_equilibrium(fl, tt4, Some((s, s))).ok())
                            .map(|(i, _, _)| (i.nu_lp, i.nu_hp, i.sp_thrust)).collect();
                        if roots.is_empty() { tt4 += 20.0; continue; }
                        let spread = roots.iter().map(|r| (r.0 - roots[0].0).abs().max((r.1 - roots[0].1).abs()))
                            .fold(0.0, f64::max);
                        worst_spread = worst_spread.max(spread);
                        let pos = roots[0].2 > 0.0;
                        if spread > 1e-9 {
                            println!("SPREAD {shape} {name} {tt4} spread={spread:.2e} n={} pos={pos} {:?}", roots.len(), roots);
                        }
                        *tally.entry(format!("rescue-candidate pos={pos} starts_solving={}", roots.len())).or_default() += 1;
                    }
                }
                tt4 += 20.0;
            }
        }
    }
    for (k, v) in &tally { println!("TALLY {v:6} {k}"); }
    println!("worst spread across starts {worst_spread:.2e}; min full-speed sp_thrust {min_full_sp:.3}");
}
