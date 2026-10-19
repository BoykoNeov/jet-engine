// Throwaway probe 2: a matcher-free fallback — retry the Newton from lower equal speeds
// (0.9 … 0.4) where the full-speed start fails. Compared to the steady matcher where it solves.
use std::collections::BTreeMap;
use turbojet::atmosphere::standard;
use turbojet::engine::FlightCondition;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::{control_maps, CONTROL_SHAPES};
use turbojet::two_spool_transient::TwoSpoolTransientCore;

#[test]
fn probe_ladder() {
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
                    if tr.try_equilibrium(fl, tt4, None).is_err() {
                        let steady = tr.inner.try_match_point(fl, tt4).ok().map(|m| (m.n_lp_ratio, m.n_hp_ratio));
                        let mut got = None;
                        for (k, s) in [0.9, 0.8, 0.7, 0.6, 0.5, 0.4].iter().enumerate() {
                            if let Ok((i, _, it)) = tr.try_equilibrium(fl, tt4, Some((*s, *s))) {
                                got = Some((k, *s, it, i.nu_lp, i.nu_hp)); println!("T {shape} below={below} {name} {tt4} sp_thrust={:.2} steady_ok={}", i.sp_thrust, steady.is_some());
                                break;
                            }
                        }
                        let key = match (got, steady) {
                            (Some((k, s, it, l, h)), Some(st)) => {
                                let d = (l - st.0).abs().max((h - st.1).abs());
                                println!("L {shape} below={below} {name} {tt4} start={s} tries={} it={it} d_vs_steady={d:.1e}", k + 1);
                                if d < 1e-10 { "ladder solves = steady" } else { "ladder solves != steady" }
                            }
                            (Some((_, s, _, l, h)), None) => {
                                println!("N {shape} below={below} {name} {tt4} start={s} nu=({l:.3},{h:.3}) steady refuses");
                                "ladder solves, steady refuses"
                            }
                            (None, Some(_)) => "ladder fails, steady solves",
                            (None, None) => "both fail",
                        };
                        *tally.entry(format!("below={below} {key}")).or_default() += 1;
                    }
                    tt4 += 20.0;
                }
            }
        }
    }
    for (k, v) in &tally { println!("TALLY {v:6} {k}"); }
}
