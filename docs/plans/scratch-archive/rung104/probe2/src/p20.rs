// Rung 104 probe P20 — anchor § 5: RUNG 97's numbers under each setting.
// (1) The 700 K start at sea level +40 K, Mach 0.85 (the steady matcher = the slam's start, rung 103 R4): each setting
//     against the OLD model (rung 38's pin pushed past choke) — rung 97 published LP −1.6…−1.8 %, HP −0.02…−0.03 %,
//     thrust +3.6…+4.8 %.
// (2) The 700 → 1400 K slam: each law against rung 97 as built (the pin) — peak Tt4 and the LP/HP speeds mid-slam.
use turbojet::atmosphere::standard;
use turbojet::engine::FlightCondition;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::{control_maps, controls, ControlsSettings, CONTROL_SHAPES};
use turbojet::turbine_law::TurbineLaw;
use turbojet::two_spool::TwoSpoolMapMatcher;

fn main() {
    let (t, p) = standard(0.0);
    let fl = FlightCondition::new(t + 40.0, p, 0.85);
    let laws = [("rung 97", TurbineLaw::Pin), ("1row", TurbineLaw::OneRow), ("2rows", TurbineLaw::TwoRows), ("ellipse", TurbineLaw::Ellipse)];
    for shape in 0..3 {
        let (ml, mh) = control_maps(CONTROL_SHAPES[shape]).unwrap();
        let bs = BladeSettings::defaults();
        let c = match TwoSpoolMapMatcher::new(bs.design(), bs.flight(), 1.0, ml, mh) { TwoSpoolMapMatcher::Full(c) => c, _ => unreachable!() };
        c.base.turbine_law.set(TurbineLaw::Pin);
        c.base.pin_past_choke.set(true);
        let old = c.try_match_point(&fl, 700.0).unwrap();
        c.base.pin_past_choke.set(false);
        for (ln, law) in laws {
            c.base.turbine_law.set(law);
            c.base.below_choke.set(true);
            let r = c.try_match_point(&fl, 700.0).unwrap();
            c.base.below_choke.set(false);
            println!("(1) {:10} {ln:8} M9 {:.3}  LP {:+.2}%  HP {:+.3}%  thrust {:+.2}%  vs the old model", CONTROL_SHAPES[shape], r.base.m9,
                     100.0 * (r.n_lp_ratio / old.n_lp_ratio - 1.0), 100.0 * (r.n_hp_ratio / old.n_hp_ratio - 1.0),
                     100.0 * (r.base.thrust / old.base.thrust - 1.0));
        }
        let s = ControlsSettings { shape, t0: t + 40.0, p0: p, m0: 0.85, from: 700.0, to: 1400.0, ..ControlsSettings::defaults() };
        let (_, a) = controls(&ControlsSettings { turbine: TurbineLaw::Pin, ..s }).unwrap();
        let peak = |v: &[turbojet::fuel_transient::FuelPoint]| v.iter().map(|q| q.tt4).fold(f64::MIN, f64::max);
        for (ln, law) in &laws[1..] {
            let (_, b) = controls(&ControlsSettings { turbine: *law, ..s }).unwrap();
            let mid = a.points.len() / 3;
            println!("(2) {:10} {ln:8} slam vs rung 97: peak Tt4 {:+.1} K; at a third of the run LP {:+.2}% HP {:+.2}%; end LP {:+.3}% HP {:+.3}%",
                     CONTROL_SHAPES[shape], peak(&b.points) - peak(&a.points),
                     100.0 * (b.points[mid].nu_lp / a.points[mid].nu_lp - 1.0), 100.0 * (b.points[mid].nu_hp / a.points[mid].nu_hp - 1.0),
                     100.0 * (b.points.last().unwrap().nu_lp / a.points.last().unwrap().nu_lp - 1.0),
                     100.0 * (b.points.last().unwrap().nu_hp / a.points.last().unwrap().nu_hp - 1.0));
        }
    }
}
