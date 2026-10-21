// The Fly-it running-line chart (running_point_json over running_line_grid): points per flight, and how many have
// no positive thrust (which Fly it refuses below idle) — shipped code vs the repair.
use turbojet::atmosphere::standard;
use turbojet::sandbox::{running_line_grid, running_point_json, FlySettings, GasModel, MapShape};
use turbojet::visuals::Json;

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let flights = [("45C SL M0.3", 0.0, 45.0, 0.3), ("SL M0.6", 0.0, 0.0, 0.6), ("SL M1.0", 0.0, 0.0, 1.0),
                   ("9km M1.4", 9000.0, 0.0, 1.4), ("11km M1.8", 11000.0, 0.0, 1.8), ("11km M2.0", 11000.0, 0.0, 2.0)];
    let (mut tot, mut neg) = (0, 0);
    for (name, h, dt, m0) in flights {
        for gas in [GasModel::ThermallyPerfect, GasModel::Perfect] {
            for map in MapShape::ALL {
                let (t0, p0) = standard(h);
                let base = FlySettings { gas, map, t0: t0 + dt, p0, m0, ..FlySettings::defaults() };
                let mut row = String::new();
                for tt4 in running_line_grid(&base) {
                    let s = FlySettings { tt4, ..base };
                    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| running_point_json(&s)));
                    row += &match r {
                        Ok(j) => match j.get("thrust") { Some(Json::Float(t)) => { tot += 1; if !(*t > 0.0) { neg += 1; format!(" {tt4:.0}:NEG{t:.0}") } else { format!(" {tt4:.0}:+") } }, _ => " ?".into() },
                        Err(_) => format!(" {tt4:.0}:x"),
                    };
                }
                println!("{name} {gas:?} {map:?}:{row}");
            }
        }
    }
    println!("points {tot}, no positive thrust {neg}");
}
