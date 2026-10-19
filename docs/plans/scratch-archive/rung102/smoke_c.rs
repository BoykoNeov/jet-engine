// Part C smoke: the shipped Controls view with the afterburner.
use turbojet::afterburner::AbMode;
use turbojet::burner_hold::Hold;
use turbojet::sandbox_controls::{classify, controls, controls_json, ControlsSettings};

fn show(label: &str, s: &ControlsSettings) {
    let t = std::time::Instant::now();
    match controls(s) {
        Err(e) => println!("{label}: REFUSED {}", e.0),
        Ok((sv, o)) => {
            let j = controls_json(&sv, &o);
            let ms = t.elapsed().as_secs_f64() * 1e3;
            let p = o.points.last().unwrap();
            let r = sv.read(p).unwrap();
            println!("{label}: {} pts ({ms:.0} ms), end nu ({:.5},{:.5}) Tt4 {:.1} thrust {:.2} ab {:?}", o.points.len(), p.nu_lp, p.nu_hp, p.tt4, r.thrust,
                     r.ab.map(|a| (a.w, a.s, a.detail.f7 - a.detail.f5, a.s7.tt)));
            if let Some(st) = &o.stop { println!("   STOP {:?}: {:?}", classify(&sv, st, o.points.last()), st.failure.as_ref().map(|f| &f.message)); }
            let _ = j;
        }
    }
}

fn main() {
    let d = ControlsSettings::defaults();
    show("not fitted", &d);
    show("fitted unlit", &ControlsSettings { ab: Some(AbMode::Unlit), ..d });
    show("lit held @0.5", &ControlsSettings { ab: Some(AbMode::Lit), ..d });
    show("lit x1.15 @0.5", &ControlsSettings { ab: Some(AbMode::Lit), a8_ratio: 1.15, ..d });
    show("lit match @0.5", &ControlsSettings { ab: Some(AbMode::Lit), a8_match: true, ..d });
    show("lit held @0.25 (off grid)", &ControlsSettings { ab: Some(AbMode::Lit), light_at: 0.25, ..d });
    show("lit held redline @0.24", &ControlsSettings { ab: Some(AbMode::Lit), light_at: 0.24, redline_on: true, ..d });
    show("lit hold thrust", &ControlsSettings { ab: Some(AbMode::Lit), govern: true, hold: Hold::Thrust, ..d });
    show("lit hold HP speed", &ControlsSettings { ab: Some(AbMode::Lit), govern: true, hold: Hold::HpSpeed, ..d });
    show("lit Tt7 2400 @0.24", &ControlsSettings { ab: Some(AbMode::Lit), light_at: 0.24, tt7: 2400.0, ..d });
}
