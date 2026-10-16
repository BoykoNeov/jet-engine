//! Reachability only: (a) rich clamps with NO leg in authority (would mean riding the wall); (b) the P-law: ran to end,
//! clamp counts, at gains 2/5/20/50, accel and decel, two shapes; whether the march runs at the step at all.
use jet_rung95_probe::*;
use turbojet::sandbox_controls::{build, ControlsSettings, CONTROL_SHAPES};
fn main() {
    let base = ControlsSettings::defaults();
    for (name, s) in [("accel +redline", ControlsSettings { redline_on: true, ..base }),
                      ("accel +sched", ControlsSettings { accel_on: true, ..base })] {
        for shape in [0usize, 3] {
            let s = ControlsSettings { shape, ..s };
            let sv = build(&s).unwrap();
            for g in [Gov::Thrust, Gov::SpeedRate(0.1)] {
                let (recs, _) = march(&sv, g, ds());
                let bad = recs.iter().filter(|r| r.clamp == Clamp::Rich && r.leg == 0).count();
                println!("{name:16} {:10} {g:?}: rich clamps with no leg cutting: {bad}", CONTROL_SHAPES[shape]);
            }
        }
    }
    for (name, s) in [("accel", base), ("decel", ControlsSettings { from: 1400.0, to: 1000.0, ..base })] {
        for shape in [0usize, 3] {
            let s = ControlsSettings { shape, ..s };
            let sv = build(&s).unwrap();
            let rows = table(&sv, 13);
            for k in [2.0, 5.0, 20.0, 50.0, 200.0] {
                let (recs, stop) = p_march(&sv, &rows, k, ds());
                let c = |x: Clamp| recs.iter().filter(|r| r.1 == x).count();
                println!("{name} {:10} P K={k}: {}/{} pts{}; root/rich/lean {}/{}/{}", CONTROL_SHAPES[shape], recs.len(),
                         s.expected_points(), stop.map(|m| format!(" STOP {m}")).unwrap_or_default(),
                         c(Clamp::Root), c(Clamp::Rich), c(Clamp::Lean));
            }
        }
    }
}
