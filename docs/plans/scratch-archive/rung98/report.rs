// Rung 98 — the anchor's reported (not scored) quantities: fixed-amount variants at thin/dense, and the thrust gap
// through the move (F_true − F̂ over the thrust change, ramp points) with the tables made at the design flight vs here.
use turbojet::burner_hold::Hold;
use turbojet::corrected_tables::TableAt;
use turbojet::sandbox_controls::{build, controls, ControlsSettings, CustomerMode, GeneratorMode, ThrustBy, CONTROL_SHAPES};
fn main() {
    let g = ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..ControlsSettings::defaults() };
    for (vn, v) in [("gen HP power .02", ControlsSettings { generator: GeneratorMode::Hp, table_clean: false, ..g }),
                    ("cust amount .05", ControlsSettings { customer: CustomerMode::Amount, ..g })] {
        for (fname, p0) in [("thin", 26_500.0), ("dense", 80_000.0)] {
            for shape in [0, 3] {
                let s = ControlsSettings { shape, p0, ..v };
                let r = match build(&s) { Ok(sv) => { let (a, b) = (sv.hold_start.unwrap(), sv.hold_end.unwrap());
                    format!("{:+.2} % / {:+.2} %", 100.0 * (a.achieved - a.target) / a.target, 100.0 * (b.achieved - b.target) / b.target) }
                    Err(e) => format!("refused: {}", e.0.chars().take(90).collect::<String>()) };
                println!("FIXED {vn:17} {fname:5} {:10}: {r}", CONTROL_SHAPES[shape]);
            }
        }
    }
    for by in [ThrustBy::Epr, ThrustBy::N1] {
        for (fname, t0, m0) in [("hot", 290.0, 0.85), ("cold", 216.65, 0.85), ("M0.5", 250.0, 0.5), ("M1.4", 250.0, 1.4)] {
            for shape in [0, 3] {
                let mut out = String::new();
                for at in [TableAt::Design, TableAt::Flight] {
                    let s = ControlsSettings { shape, t0, m0, thrust_by: by, table_at: at, ..g };
                    let (sv, o) = controls(&s).unwrap();
                    let d = (ControlsSettingsThrust::end(&sv) - ControlsSettingsThrust::start(&sv)).abs();
                    let gaps: Vec<f64> = o.points.iter().filter(|p| p.s > 0.0 && p.s <= s.ramp + 1e-9).map(|p| {
                        let r = sv.read(p).unwrap(); 100.0 * (r.thrust - r.inferred.unwrap()) / d }).collect();
                    out += &format!("  {:?}: {:+.1} / {:+.1}", at, gaps.iter().cloned().fold(f64::MIN, f64::max), gaps.iter().cloned().fold(f64::MAX, f64::min));
                }
                println!("GAP {by:?} {fname:4} {:10}:{out}", CONTROL_SHAPES[shape]);
            }
        }
    }
}
struct ControlsSettingsThrust;
impl ControlsSettingsThrust {
    fn start(sv: &turbojet::sandbox_controls::ControlsSolver) -> f64 { turbojet::governor::thrust(&sv.start) }
    fn end(sv: &turbojet::sandbox_controls::ControlsSolver) -> f64 { turbojet::governor::thrust(&sv.end) }
}
