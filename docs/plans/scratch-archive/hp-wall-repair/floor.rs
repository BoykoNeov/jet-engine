// Re-measure the Controls refusal text's numbers: the lowest Tt4 (10 K steps) where rung 40's equilibrium solves, per map,
// at the design flight's air and Mach 0.85 / 1.6 / 2.0 / 2.6; and the highest at 11 km (the text's "top of the range").
use turbojet::atmosphere::standard;
use turbojet::engine::FlightCondition;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::{control_maps, CONTROL_SHAPES};
use turbojet::two_spool_transient::TwoSpoolTransientCore;

fn main() {
    let bs = BladeSettings::defaults();
    let fd = bs.flight();
    for shape in CONTROL_SHAPES {
        let (ml, mh) = control_maps(shape).unwrap();
        let tr = TwoSpoolTransientCore::new(bs.design(), fd, 1.0, ml, mh, 1.0);
        let mut line = format!("{shape:10}");
        for m0 in [0.85, 1.6, 2.0, 2.6] {
            let fl = FlightCondition::new(fd.t0, fd.p0, m0);
            let lo = (0..=150).map(|k| 500.0 + 10.0 * k as f64).find(|&t| tr.try_equilibrium(&fl, t, None).is_ok());
            line += &format!(" | M{m0}: lowest {}", lo.map(|t| format!("{t:.0}")).unwrap_or("none".into()));
        }
        let (t11, p11) = standard(11000.0);
        let fl = FlightCondition::new(t11, p11, 0.85);
        let ok: Vec<f64> = (0..=150).map(|k| 500.0 + 10.0 * k as f64).filter(|&t| tr.try_equilibrium(&fl, t, None).is_ok()).collect();
        line += &format!(" | 11km M0.85: {}..{}", ok.first().map_or(f64::NAN, |v| *v), ok.last().map_or(f64::NAN, |v| *v));
        println!("{line}");
    }
}
