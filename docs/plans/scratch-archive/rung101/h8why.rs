// H8's misses: is the start choked? (rung 99 G3: choked, the fan formula's only error is the airflow guess)
use turbojet::airflow_source::AirflowFrom;
use turbojet::burner_hold::Hold;
use turbojet::mach_axis::SpeedCorr;
use turbojet::matcher::Branch;
use turbojet::sandbox_controls::{build, ControlsSettings, ThrustBy};
fn main() {
    let d = ControlsSettings::defaults();
    for (t0, p0, m0, from, v) in [(250.0, 50_000.0, 1.2, 750.0, "plain"), (250.0, 50_000.0, 1.5, 900.0, "plain"), (250.0, 50_000.0, 1.5, 750.0, "plain"),
                                  (250.0, 50_000.0, 0.25, 900.0, "plain"), (250.0, 50_000.0, 0.25, 750.0, "plain"), (250.0, 50_000.0, 0.25, 750.0, "cool"),
                                  (268.65, 70_120.0, 1.3, 900.0, "plain"), (268.65, 70_120.0, 1.3, 750.0, "plain"), (318.15, 101_325.0, 0.2, 900.0, "plain")] {
        let base = ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_corr: SpeedCorr::Formula, from, t0, p0, m0,
                                      cool_vane: if v == "cool" { 0.05 } else { 0.0 }, cool_blade: if v == "cool" { 0.05 } else { 0.0 }, ..d };
        let sv = build(&base).unwrap();
        let g = build(&ControlsSettings { airflow: AirflowFrom::Grid, ..base }).unwrap();
        let miss = |sv: &turbojet::sandbox_controls::ControlsSolver| { let h = sv.hold_start.unwrap(); 100.0 * (h.achieved - h.target) / h.target };
        println!("{t0} M{m0} {from} {v:5}: start {} | rung100 {:+.3}% grid {:+.3}%", if sv.start.branch == Branch::Choked { "choked  " } else { "UNCHOKED" }, miss(&sv), miss(&g));
    }
}
