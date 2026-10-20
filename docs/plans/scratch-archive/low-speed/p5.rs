// Probe 5: the one first-step stop left (customer bleed share, SL +40 K, tilted, 440 K): its message, start, wall,
// classification.
use turbojet::sandbox_controls::{controls, classify, lean_wall, run_mixture, ControlsSettings, CustomerMode, CONTROL_SHAPES};
fn main() {
    let d = ControlsSettings::defaults();
    let (t_sl, p_sl) = turbojet::atmosphere::standard(0.0);
    let k = CONTROL_SHAPES.iter().position(|&s| s == "tilted").unwrap();
    for tt4 in [440.0, 445.0, 450.0] {
        let s = ControlsSettings { shape: k, t0: t_sl + 40.0, p0: p_sl, m0: 0.85, customer: CustomerMode::Share, from: tt4, ..d };
        let (sv, o) = controls(&s).unwrap();
        let st = o.stop.as_ref();
        let f = st.and_then(|s| s.failure.as_ref());
        println!("{tt4}: start f={:.5} end f={:.5} wall={:.5} pts={} kind={:?} mixture={:?}\n   {:?}", sv.start.close.f, sv.end.close.f,
            lean_wall(&sv), o.points.len(), st.map(|s| classify(&sv, s, o.points.last())),
            f.and_then(|f| run_mixture(&sv, f, o.points.last())), f.map(|f| f.message.clone()));
    }
}
