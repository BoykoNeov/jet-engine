// Rung 99 final review: the formula at the unchoked 700 K start (Mach 0.05): split F_true − F̂ at the calibrated end into
// the airflow guess V0·(ṁ̂ − ṁ_face) (ṁ̂ the table's airflow AT THE READING) and the rest (the jet's push), both readings.
use turbojet::burner_hold::Hold;
use turbojet::inferred_thrust::{face, lin, Reading};
use turbojet::mach_axis::SpeedCorr;
use turbojet::sandbox_controls::{build, ControlsSettings, ThrustBy, CONTROL_SHAPES};
fn main() {
    for shape in 0..4 {
        for (by, r) in [(ThrustBy::Epr, Reading::Epr), (ThrustBy::N1, Reading::LpSpeed)] {
            let s = ControlsSettings { shape, govern: true, hold: Hold::Thrust, thrust_by: by, speed_corr: SpeedCorr::Formula,
                                       from: 700.0, t0: 250.0, p0: 50_000.0, m0: 0.05, ..ControlsSettings::defaults() };
            let sv = build(&s).unwrap();
            let t = sv.governor.as_ref().unwrap().infer.as_ref().unwrap();
            let i = &sv.start;
            let v = t.read(r, i);
            let f_hat = t.inferred(r, v);
            let f_true = ControlsSolver_thrust(i);
            let x = |w: &turbojet::inferred_thrust::Row| if r == Reading::Epr { w.epr } else { w.nu_lp };
            let m_hat = lin(&t.rows, x, |w| w.face, v);
            let guess = sv.carry.unwrap().v0 * (m_hat - face(i));
            let rest = f_true - f_hat - guess;
            println!("{:10} {:?}: F_true−F̂ {:+.3}% of F = airflow guess {:+.3}% (ṁ̂/ṁ−1 {:+.3}%) + jet push {:+.3}%",
                     CONTROL_SHAPES[shape], r, 100.0 * (f_true - f_hat) / f_true, 100.0 * guess / f_true,
                     100.0 * (m_hat / face(i) - 1.0), 100.0 * rest / f_true);
        }
    }
}
#[allow(non_snake_case)]
fn ControlsSolver_thrust(i: &turbojet::two_spool_transient::Instant2) -> f64 { turbojet::governor::thrust(i) }
