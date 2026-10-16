//! The LP laws' step check BEYOND the page's limits (the precheck refuses them, so the governor's law is swapped on a
//! built run): 0.02 against 0.005, max |dphi| over the run, accel and decel, all four maps.
use turbojet::burner_hold::Hold;
use turbojet::governor::{self as gv, Law};
use turbojet::sandbox_controls::{build, ControlsSettings, SpeedLaw, ThrustBy, CONTROL_SHAPES, DS};

fn main() {
    let n1 = ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::N1, ..ControlsSettings::defaults() };
    for (dir, from, to) in [("accel", 1000.0, 1400.0), ("decel", 1400.0, 1000.0)] {
        for law in [Law::LpRate { tau: 0.02 }, Law::LpRate { tau: 0.03 }, Law::LpRate { tau: 0.04 }, Law::LpRate { tau: 0.06 },
                    Law::LpProp { gain: 50.0 }, Law::LpProp { gain: 100.0 }, Law::LpProp { gain: 200.0 }] {
            let mut row = Vec::new();
            for shape in 0..4 {
                let sp = if matches!(law, Law::LpProp { .. }) { SpeedLaw::Prop } else { SpeedLaw::Rate };
                let s = ControlsSettings { shape, from, to, speed_law: sp, ..n1 };
                let sv = build(&s).unwrap();
                let mut g = sv.governor.clone().unwrap();
                g.law = law;
                let nu0 = (sv.start.nu_lp, sv.start.nu_hp);
                let lim = sv.limiters();
                let (a, ma) = gv::march(&sv.core.fuel, &sv.flight, &g, &lim, None, nu0, s.s_end(), DS);
                let (b, _) = gv::march(&sv.core.fuel, &sv.flight, &g, &lim, None, nu0, s.s_end(), DS / 4.0);
                let mut dmax = 0.0f64;
                for (k, x) in a.iter().enumerate() {
                    if let Some(y) = b.get(4 * k) { dmax = dmax.max((x.phi_lp - y.phi_lp).abs().max((x.phi_hp - y.phi_hp).abs())) }
                }
                let lean = ma.iter().filter(|m| m.clamp == gv::Clamp::Lean).count();
                row.push(format!("{} {dmax:.1e} ({}/{} pts, lean {lean})", CONTROL_SHAPES[shape], a.len(), s.expected_points()));
            }
            println!("{dir} {law:?}: {}", row.join(" | "));
        }
    }
}
