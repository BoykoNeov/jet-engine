// The lag's order: worst true-thrust gap to the instant sensor's run at τ 0.05, 0.1, 0.2, both readings, Mach 0.5 / 1.4.
use turbojet::airflow_source::AirflowFrom;
use turbojet::burner_hold::Hold;
use turbojet::mach_axis::SpeedCorr;
use turbojet::sandbox_controls::{build, ControlsSettings, ThrustBy};
fn main() {
    for m0 in [0.5, 1.4] { for by in [ThrustBy::Epr, ThrustBy::N1] {
        let s = ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: by, speed_corr: SpeedCorr::Formula, airflow: AirflowFrom::Exhaust,
                                   t0: 250.0, p0: 50_000.0, m0, ..ControlsSettings::defaults() };
        let th = |s: &ControlsSettings| { let sv = build(s).unwrap(); sv.march_marked().0.iter().map(|p| sv.read(p).unwrap().thrust).collect::<Vec<f64>>() };
        let base = th(&s);
        let g = |tau: f64| th(&ControlsSettings { exhaust_lag: true, exhaust_tau: tau, ..s }).iter().zip(&base).map(|(x, y)| (x - y).abs()).fold(0.0f64, f64::max);
        let (a, b, c) = (g(0.05), g(0.1), g(0.2));
        println!("M{m0} {by:?}: gaps {a:.4} {b:.4} {c:.4}  ratios 0.1/0.05 {:.4}  0.2/0.1 {:.4}", b / a, c / b);
    } }
}
