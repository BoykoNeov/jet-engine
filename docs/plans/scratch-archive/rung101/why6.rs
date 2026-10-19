// H6/H7's refusals: the message, and where in the run (start build or the march) — fan speed + sensor.
use turbojet::airflow_source::AirflowFrom;
use turbojet::burner_hold::Hold;
use turbojet::mach_axis::SpeedCorr;
use turbojet::sandbox_controls::{build, stop_reason, ControlsSettings, ThrustBy};
fn main() {
    for (t0, p0, m0, from) in [(268.65, 70_120.0, 0.2, 900.0), (250.0, 50_000.0, 0.25, 900.0), (318.15, 101_325.0, 0.2, 900.0), (268.65, 70_120.0, 0.2, 750.0)] {
        let s = ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_corr: SpeedCorr::Formula,
                                   airflow: AirflowFrom::Exhaust, from, t0, p0, m0, ..ControlsSettings::defaults() };
        match build(&s) {
            Err(e) => println!("{t0} M{m0} {from}: BUILD refused: {}", e.0),
            Ok(sv) => {
                let (pts, _) = sv.march_marked();
                println!("{t0} M{m0} {from}: {} of {} points", pts.len(), s.expected_points());
                if let Some(st) = stop_reason(&sv, &pts) {
                    println!("   stop at s {:.3}: {:?}", st.s_last, st.failure.map(|f| (f.message, f.call, f.mf, f.s)));
                }
                // the start's own rows: fan rows nu_x, f_hat; the start's belief
                let t = sv.governor.as_ref().unwrap().infer.as_ref().unwrap();
                println!("   start belief {:?}  start thrust {:.2}  table rows x {:.0}..{:.0}", t.try_belief(&sv.core.fuel, turbojet::inferred_thrust::Reading::LpSpeed, &sv.start, None).map_err(|e| e.0),
                         sv.hold_start.unwrap().achieved, 0.0, 0.0);
            }
        }
    }
}
