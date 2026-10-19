// Throwaway: thrust at and just above each fast-flight floor (design-flight air). "r" = rescued by the low start.
use turbojet::engine::FlightCondition;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::{control_maps, ControlsSettings, CONTROL_SHAPES};
use turbojet::two_spool_transient::{take_low_start_rescues, TwoSpoolTransientCore};
fn main() {
    let d = ControlsSettings::defaults();
    let bs = BladeSettings::defaults();
    for shape in CONTROL_SHAPES.iter() {
        let (ml, mh) = control_maps(shape).unwrap();
        let tr = TwoSpoolTransientCore::new(bs.design(), bs.flight(), 1.0, ml, mh, 1.0);
        tr.inner.base.below_choke.set(true);
        for (m0, lo) in [(0.85, 470.0), (1.6, 570.0), (2.0, 660.0), (2.6, 790.0)] {
            let fl = FlightCondition::new(d.t0, d.p0, m0);
            let mut line = format!("{shape} M{m0}:");
            for k in 0..5 {
                let tt4 = lo + 10.0 * k as f64;
                take_low_start_rescues();
                let r = tr.try_equilibrium(&fl, tt4, None);
                let resc = take_low_start_rescues();
                line += &match r { Ok((i, _, _)) => format!(" {tt4}:{:.2}{}", i.sp_thrust, if resc > 0 { "r" } else { "" }), Err(_) => format!(" {tt4}:ERR") };
            }
            println!("{line}");
        }
    }
}
