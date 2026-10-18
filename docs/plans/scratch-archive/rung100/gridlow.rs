// Rung 100 pre-anchor (user: "try grid slice low"): the calibrated ends' thrust miss under the speed grid (and, on the
// shipped build only, rung 99's formula), at low flight speeds. Built twice: against the shipped crate (grid 0.3…3.0)
// and against a worktree whose grid starts with a Mach-0.05 slice. Output: one line per case, key then miss.
#[path = "common.rs"]
mod common;
use common::*;
use turbojet::burner_hold::Hold;
use turbojet::mach_axis::{SpeedCorr, GRID};
use turbojet::sandbox_controls::{build, ControlsSettings, ThrustBy};
fn main() {
    let shipped = GRID[0] == 0.3;
    let v = variants();
    let flights = [(250.0, 50_000.0, 0.05), (250.0, 50_000.0, 0.1), (250.0, 50_000.0, 0.15), (250.0, 50_000.0, 0.2),
                   (250.0, 50_000.0, 0.25), (250.0, 50_000.0, 0.45), (288.15, 101_325.0, 0.05), (288.15, 101_325.0, 0.15),
                   (288.15, 101_325.0, 0.25), (255.7, 54_048.0, 0.1)];
    for (name, s) in [("plain", v[0].1), ("cool", v[6].1), ("cust", v[4].1)] {
        for shape in 0..4 {
            for &(t0, p0, m0) in &flights {
                for from in [700.0, 1000.0] {
                    for by in [ThrustBy::Epr, ThrustBy::N1] {
                        let corrs: &[SpeedCorr] = if shipped { &[SpeedCorr::Grid, SpeedCorr::Formula] } else { &[SpeedCorr::Grid] };
                        for &corr in corrs {
                            let st = ControlsSettings { shape, govern: true, hold: Hold::Thrust, thrust_by: by, speed_corr: corr,
                                                        from, to: 1400.0, t0, p0, m0, ..s };
                            let key = format!("{name} {shape} {t0} {m0} {from} {:?} {}", by, corr.key());
                            match build(&st) {
                                Ok(sv) => {
                                    let (Some(a), Some(b)) = (sv.hold_start, sv.hold_end) else { println!("{key} | AT-SLICE (tables made here)"); continue };
                                    println!("{key} | {:+.4} {:+.4}", 100.0 * (a.achieved / a.target - 1.0), 100.0 * (b.achieved / b.target - 1.0));
                                }
                                Err(e) => println!("{key} | REFUSED {}", e.0.chars().take(90).collect::<String>()),
                            }
                        }
                    }
                }
            }
        }
    }
}
