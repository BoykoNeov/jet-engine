// Rung 98 pre-anchor: do the planned scoring flights' own ends solve, choked? Prints y/n and the branch only.
#[path = "common.rs"]
mod common;
use common::*;
use turbojet::engine::FlightCondition;
use turbojet::sandbox_controls::{ControlsSettings, CONTROL_SHAPES};

fn main() {
    let flights = [("hot 290K", 290.0, 50_000.0, 0.85), ("cold 216.65K", 216.65, 50_000.0, 0.85),
                   ("thin 26.5kPa", 250.0, 26_500.0, 0.85), ("dense 80kPa", 250.0, 80_000.0, 0.85),
                   ("M0.5", 250.0, 50_000.0, 0.5), ("M1.4", 250.0, 50_000.0, 1.4)];
    for shape in 0..4 {
        let s = ControlsSettings { shape, ..ControlsSettings::defaults() };
        let sv = machine(&s);
        let d2 = st2(&sv, &s.design_flight());
        for (name, t0, p0, m) in flights {
            let fl = FlightCondition::new(t0, p0, m);
            let th = st2(&sv, &fl).0 / d2.0;
            let r: Vec<String> = [1000.0, 1400.0].iter().map(|&t| match sv.core.fuel.inner.try_equilibrium(&fl, t, None) {
                Ok((i, _, _)) => format!("{t}:ok{}", if choked(&i) { "" } else { ",UNCHOKED" }),
                Err(_) => format!("{t}:REFUSED"),
            }).collect();
            println!("{:10} {:13} θ {:.4} δ {:.4}: {}", CONTROL_SHAPES[shape], name, th, st2(&sv, &fl).1 / d2.1, r.join(" "));
        }
    }
}
