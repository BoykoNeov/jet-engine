//! Reachability only: run length and stops, the governor's clamps, the applied fuel at the lean wall, which leg holds,
//! the lag's sign changes (release / re-attack), whether the clip is already asked for at s = 0, bit-equality with rung
//! 95 before the lagged legs first engage, cost against rung 95. No temperature, thrust, time or margin is printed.
use rung107_probe::*;
use std::time::Instant;
use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::AsymmetricLag;
use turbojet::governor::Clamp;
use turbojet::sandbox_controls::{build, ControlsSettings, SpeedLaw, ThrustBy, CONTROL_SHAPES, DS};

fn main() {
    let d = ControlsSettings { govern: true, ..ControlsSettings::defaults() };
    let laws: Vec<(&str, ControlsSettings)> = vec![
        ("Tt4", ControlsSettings { hold: Hold::BurnerTemp, ..d }),
        ("thrust", ControlsSettings { hold: Hold::Thrust, ..d }),
        ("HP rate .1", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Rate, gov_tau: 0.1, ..d }),
        ("HP prop 20", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 20.0, ..d }),
        ("EPR", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..d }),
        ("N1 rate .2", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Rate, gov_tau: 0.2, ..d }),
        ("N1 prop 5", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Prop, gov_gain: 5.0, ..d }),
    ];
    let cases: Vec<(&str, fn(ControlsSettings) -> ControlsSettings)> = vec![
        ("accel sched", |s| ControlsSettings { accel_on: true, ..s }),
        ("accel floor", |s| ControlsSettings { floor_on: true, ..s }),
        ("accel both+rl", |s| ControlsSettings { accel_on: true, floor_on: true, redline_on: true, ..s }),
        ("fast r.15 sched", |s| ControlsSettings { accel_on: true, ramp: 0.15, ..s }),
        ("decel floor", |s| ControlsSettings { floor_on: true, from: 1400.0, to: 1000.0, ..s }),
        ("to1600 sched+rl", |s| ControlsSettings { accel_on: true, redline_on: true, to: 1600.0, ..s }),
    ];
    for (cname, cf) in &cases {
        for (lname, ls) in &laws {
            for shape in 0..4 {
                let s = cf(ControlsSettings { shape, ..*ls });
                let sv = match build(&s) {
                    Ok(v) => v,
                    Err(e) => {
                        println!("{cname:15} {lname:10} {:10}: REFUSED {}", CONTROL_SHAPES[shape], e.0);
                        continue;
                    }
                };
                let t0 = Instant::now();
                let (base, _) = sv.march_marked();
                let t95 = t0.elapsed().as_secs_f64();
                let n_exp = s.expected_points();
                for (ta, tr) in [(0.02, 0.1), (0.02, 0.8), (0.1, 0.4), (0.4, 0.4)] {
                    let t0 = Instant::now();
                    let (recs, stop) = march(&sv, AsymmetricLag::new(ta, tr), false, DS);
                    let t = t0.elapsed().as_secs_f64();
                    let c = |k: Clamp| recs.iter().filter(|r| r.clamp == k).count();
                    let lean_app = recs.iter().filter(|r| r.lean_applied).count();
                    let l = |k: u8| recs.iter().filter(|r| r.leg == k).count();
                    let (rel, reatt) = crossings(&recs);
                    let req0 = recs.first().map(|r| r.req > 0.0).unwrap_or(false);
                    let first_eng = recs.iter().position(|r| r.engaged).unwrap_or(recs.len());
                    let same = (0..first_eng.min(base.len()))
                        .all(|k| recs[k].a == base[k].nu_lp && recs[k].b == base[k].nu_hp && recs[k].mf == base[k].mf);
                    println!("{cname:15} {lname:10} {:10} att {ta:4} rel {tr:4}: {}/{n_exp} pts (r95 {}){}; gov root/rich/lean {}/{}/{}; applied-lean {lean_app}; legs none/lag/rl/metal {}/{}/{}/{}; releases {rel} re-attacks {reatt}; asked at s=0 {req0}; pre-engage = r95 {same} ({first_eng} pts); {:.2}s vs r95 {:.2}s",
                             CONTROL_SHAPES[shape], recs.len(), base.len(), stop.map(|m| format!(" STOP {m}")).unwrap_or_default(),
                             c(Clamp::Root), c(Clamp::Rich), c(Clamp::Lean), l(0), l(1), l(2), l(3), t, t95);
                }
            }
        }
    }
}
