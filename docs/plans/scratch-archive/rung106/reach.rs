//! Reachability only: does the prototype lagged-governed march run to the end; how often the governor sits on its
//! root / rich / lean wall; how often the APPLIED fuel is clamped at the lean wall; which leg holds; cost against rung
//! 95's governed march. No temperature, thrust, time or stall margin a prediction is about is printed.
use rung106_probe::*;
use std::time::Instant;
use turbojet::burner_hold::Hold;
use turbojet::governor::Clamp;
use turbojet::sandbox_controls::{build, ControlsSettings, SpeedLaw, ThrustBy, CONTROL_SHAPES, DS};

fn main() {
    let d = ControlsSettings { govern: true, redline_on: true, ..ControlsSettings::defaults() };
    let laws: Vec<(&str, ControlsSettings)> = vec![
        ("Tt4", ControlsSettings { hold: Hold::BurnerTemp, ..d }),
        ("thrust", ControlsSettings { hold: Hold::Thrust, ..d }),
        ("HP rate .1", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Rate, gov_tau: 0.1, ..d }),
        ("HP prop 20", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 20.0, ..d }),
        ("EPR", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..d }),
        ("N1 rate", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Rate, gov_tau: 0.2, ..d }),
        ("N1 prop", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Prop, gov_gain: 5.0, ..d }),
    ];
    let cases: Vec<(&str, fn(ControlsSettings) -> ControlsSettings)> = vec![
        ("accel", |s| s),
        ("accel+sched", |s| ControlsSettings { accel_on: true, ..s }),
        ("accel+floor", |s| ControlsSettings { floor_on: true, ..s }),
        ("fast accel r.15 rl1440", |s| ControlsSettings { ramp: 0.15, redline: 1440.0, ..s }),
        ("decel", |s| ControlsSettings { from: 1400.0, to: 1000.0, ..s }),
        ("accel to 1600 rl1480", |s| ControlsSettings { to: 1600.0, ..s }),
    ];
    for (cname, cf) in &cases {
        for (lname, ls) in &laws {
            for shape in [0usize, 3] {
                let s = cf(ControlsSettings { shape, ..*ls });
                let sv = match build(&s) { Ok(v) => v, Err(e) => { println!("{cname:22} {lname:10} {:10}: REFUSED {}", CONTROL_SHAPES[shape], e.0); continue } };
                let t0 = Instant::now();
                let (base, marks) = sv.march_marked();
                let t95 = t0.elapsed().as_secs_f64();
                let n_exp = s.expected_points();
                let rich95 = marks.iter().filter(|m| m.clamp == Clamp::Rich).count();
                for tau in [0.02, 0.05, 0.2, 0.8] {
                    let t0 = Instant::now();
                    let (recs, stop) = march(&sv, s.redline, tau, DS);
                    let t = t0.elapsed().as_secs_f64();
                    let c = |k: Clamp| recs.iter().filter(|r| r.clamp == k).count();
                    let lean_app = recs.iter().filter(|r| r.lean_applied).count();
                    let l = |k: u8| recs.iter().filter(|r| r.leg == k).count();
                    // the reduce at the start: identical to rung 95 until the clip engages (g == 0 exactly)
                    let first_g = recs.iter().position(|r| r.g != 0.0).unwrap_or(recs.len());
                    let same = (0..first_g.min(base.len())).all(|k| recs[k].a == base[k].nu_lp && recs[k].b == base[k].nu_hp && recs[k].mf == base[k].mf);
                    println!("{cname:22} {lname:10} {:10} tau {tau:4}: {}/{n_exp} pts{}; gov root/rich/lean {}/{}/{} (r95 rich {rich95}); applied-lean {lean_app}; legs none/lag/sched/floor/metal {}/{}/{}/{}/{}; pre-engage bit-equal r95 {same} ({first_g} pts); {:.2}s vs r95 {:.2}s",
                             CONTROL_SHAPES[shape], recs.len(), stop.map(|m| format!(" STOP {m}")).unwrap_or_default(),
                             c(Clamp::Root), c(Clamp::Rich), c(Clamp::Lean), l(0), l(1), l(2), l(3), l(4), t, t95);
                }
            }
        }
    }
}
