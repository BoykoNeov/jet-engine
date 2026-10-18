// Post-score (reported): H1's 28 misses are the LP generator on the fan-speed reading. Same cells with the tables made
// at THIS flight (no speed axis at all), and with the speed grid; and the pressure ratio beside it.
use turbojet::atmosphere::Ambient;
use turbojet::burner_hold::Hold;
use turbojet::corrected_tables::TableAt;
use turbojet::sandbox_controls::{build, ControlsSettings, GeneratorBy, GeneratorMode, ThrustBy, CONTROL_SHAPES};
fn m(s: &ControlsSettings) -> String { match build(s) { Ok(sv) => { let a = sv.hold_start.unwrap(); format!("{:+.3}%", 100.0 * (a.achieved / a.target - 1.0)) } Err(e) => format!("refused ({})", &e.0[..40.min(e.0.len())]) } }
fn main() {
    let a3 = Ambient::from_altitude(3000.0, 0.0);
    for &(t0, p0, m0) in &[(318.15, 101_325.0, 0.08), (a3.t0, a3.p0, 0.12), (250.0, 50_000.0, 0.5), (250.0, 50_000.0, 0.85)] {
        for by in [ThrustBy::N1, ThrustBy::Epr] {
            let s = ControlsSettings { shape: 0, govern: true, hold: Hold::Thrust, thrust_by: by, from: 800.0, t0, p0, m0,
                                       generator: GeneratorMode::Lp, generator_by: GeneratorBy::Share, ..ControlsSettings::defaults() };
            println!("{} {t0:.1}K M{m0} {by:?}: grid {} | tables made at this flight {} | loaded table (table_clean off) {}",
                     CONTROL_SHAPES[0], m(&s), m(&ControlsSettings { table_at: TableAt::Flight, ..s }), m(&ControlsSettings { table_clean: false, ..s }));
        }
    }
}
