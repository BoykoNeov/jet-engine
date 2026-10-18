// Post-build (reported): Mach 1.4 from 700 K, EPR formula — split the new start's miss into the airflow guess and the
// rest; the table's airflow at the reading vs the engine's; rung 99's formula at the same reading.
use turbojet::burner_hold::Hold;
use turbojet::governor::thrust;
use turbojet::inferred_thrust::{epr, face, Reading};
use turbojet::mach_axis::SpeedCorr;
use turbojet::sandbox_controls::{build, ControlsSettings, ThrustBy, CONTROL_SHAPES};
fn main() {
    for shape in 0..3 { for m0 in [1.4, 1.2, 1.6] {
        let s = ControlsSettings { shape, govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr, speed_corr: SpeedCorr::Formula,
                                   from: 700.0, t0: 250.0, p0: 50_000.0, m0, ..ControlsSettings::defaults() };
        let Ok(sv) = build(&s) else { println!("{} M{m0}: refused", CONTROL_SHAPES[shape]); continue };
        let t = sv.governor.as_ref().unwrap().infer.as_ref().unwrap();
        let i = &sv.start; let e = epr(i, t.pt2);
        let f = thrust(i); let fh = t.try_inferred(&sv.core.fuel, Reading::Epr, e).unwrap();
        let guess = t.v0 * (t.face_at_epr(e) - face(i));
        let mut t99 = t.clone(); t99.formula = None;
        let f99 = t99.inferred(Reading::Epr, e);
        let dt = sv.design_table.as_ref().unwrap();
        println!("{:10} M{m0}: start Tt4 {:.0} {:?} | F {f:.1} F̂ {fh:.1} | guess {:+.3}% rest {:+.1e} | ṁ̂/ṁ−1 {:+.3}% V0 {:.0} | rung99 F̂ at same e {f99:.1} ({:+.3}% of F) | design rows epr {:.3}..{:.3}, reading {e:.3}",
                 CONTROL_SHAPES[shape], i.tt4, i.branch, 100.0 * guess / f, f - fh - guess, 100.0 * (t.face_at_epr(e) / face(i) - 1.0), t.v0,
                 100.0 * (f - f99) / f, dt.rows[0].epr, dt.rows.last().unwrap().epr);
    } }
}
