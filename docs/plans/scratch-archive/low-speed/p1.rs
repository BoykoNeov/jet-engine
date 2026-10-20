// Probe 1: at the design flight, flat-lp, starts 400..560 K — the start's fuel-air ratio against the closure's
// lean wall (0.004), how many points the slam gets, and the stop message; plus the f of the last recorded point.
use turbojet::sandbox_controls::{controls, ControlsSettings, CONTROL_SHAPES};
fn main() {
    let d = ControlsSettings::defaults();
    for (k, shape) in CONTROL_SHAPES.iter().enumerate() {
        let mut tt4 = 400.0;
        while tt4 <= 600.0 + 1e-9 {
            match controls(&ControlsSettings { shape: k, from: tt4, ..d }) {
                Err(e) => println!("{shape} {tt4}: refused {e:?}"),
                Ok((sv, o)) => {
                    let st = &sv.start;
                    let fmin = o.points.iter().map(|p| p.f).fold(f64::INFINITY, f64::min);
                    let last = o.points.last().map(|p| (p.f, p.mf)).unwrap_or((f64::NAN, f64::NAN));
                    let msg = o.stop.as_ref().and_then(|s| s.failure.as_ref()).map(|f| f.message.clone()).unwrap_or_default();
                    println!("{shape} {tt4}: start f={:.5} m_lp={:.4} nu=({:.4},{:.4}) mf_lo={:.5} | pts {}/{} fmin={:.5} last f={:.5} mf={:.5} | {}",
                        st.close.f, st.close.m_lp, st.nu_lp, st.nu_hp, sv.mf_lo, o.points.len(), sv.settings.expected_points(),
                        fmin, last.0, last.1, &msg[..msg.len().min(110)]);
                }
            }
            tt4 += 20.0;
        }
    }
}
