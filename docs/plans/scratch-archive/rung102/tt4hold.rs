// The review's discriminating run: Controls with the burner temperature HELD all along the move (governed, hold Tt4),
// a held / x1.15 / matched light-off at 0.5, against the same request fitted-unlit. Does a held nozzle cost thrust
// on two shafts when Tt4 is held (Part B's throttle)?
use turbojet::afterburner::AbMode;
use turbojet::burner_hold::Hold;
use turbojet::sandbox_controls::{classify, controls, ControlsSettings};

fn main() {
    let d = ControlsSettings::defaults();
    for (lab, base) in [("default 1000->1400, settle 2", d), ("settle 6", ControlsSettings { settle: 6.0, ..d }),
                        ("900->1350 settle 6, tilted", ControlsSettings { shape: 2, from: 900.0, to: 1350.0, settle: 6.0, ..d })] {
        let g = ControlsSettings { govern: true, hold: Hold::BurnerTemp, ..base };
        let run = |s: ControlsSettings| {
            let (sv, o) = controls(&s).map_err(|e| e.0)?;
            let p = o.points.last().unwrap();
            let r = sv.read(p).unwrap();
            let stop = o.stop.as_ref().map(|st| format!("{:?}: {}", classify(&sv, st, o.points.last()), st.failure.as_ref().map_or("?".into(), |f| f.message.clone())));
            Ok::<_, String>((p.s, p.nu_lp, p.nu_hp, p.tt4, r.thrust, stop))
        };
        let u = run(ControlsSettings { ab: Some(AbMode::Unlit), ..g });
        println!("== {lab}: unlit {:?}", u);
        for (tl, s) in [("held", ControlsSettings { ab: Some(AbMode::Lit), ..g }),
                        ("x1.15", ControlsSettings { ab: Some(AbMode::Lit), a8_ratio: 1.15, ..g }),
                        ("match", ControlsSettings { ab: Some(AbMode::Lit), a8_match: true, ..g })] {
            let l = run(s);
            let ratio = match (&u, &l) { (Ok(a), Ok(b)) if b.5.is_none() => format!("x{:.4}", b.4 / a.4), _ => "-".into() };
            println!("   {tl:6} {ratio}  {:?}", l);
        }
    }
}
