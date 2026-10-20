// Rung 104 probe P16 — THE SORT, rung 102 Part C (the afterburner lit in the slam, the throat HELD): at the end of the
// move, lit against fitted-unlit, under each setting — the LP shaft's change, the HP shaft's, the main burner's Tt4 and
// thrust. Rung 102's claims (C2–C5): LP −8…−50 %, HP +2…+20 %, Tt4 hotter, thrust above unlit.
use turbojet::afterburner::AbMode;
use turbojet::atmosphere::standard;
use turbojet::sandbox_controls::{controls, ControlsSettings, CONTROL_SHAPES};
use turbojet::turbine_law::TurbineLaw;

fn main() {
    let d = ControlsSettings::defaults();
    let (t5, p5) = standard(5000.0);
    let flights = [("design", d), ("5km M0.6", ControlsSettings { t0: t5, p0: p5, m0: 0.6, ..d })];
    let laws = [("pin", TurbineLaw::Pin), ("1row", TurbineLaw::OneRow), ("2rows", TurbineLaw::TwoRows), ("ellipse", TurbineLaw::Ellipse)];
    for shape in 0..CONTROL_SHAPES.len().min(4) {
        for (fname, f) in flights {
            for (ln, law) in laws {
                let s = ControlsSettings { shape, from: 900.0, to: 1350.0, turbine: law, ..f };
                let u = controls(&ControlsSettings { ab: Some(AbMode::Unlit), ..s });
                let h = controls(&ControlsSettings { ab: Some(AbMode::Lit), light_at: 0.30, ..s });
                match (u, h) {
                    (Ok((_, u)), Ok((_, h))) if h.stop.is_none() && u.stop.is_none() => {
                        let (ue, he) = (u.points.last().unwrap(), h.points.last().unwrap());
                        let fu = ue.sp_thrust * ue.mdot_air;
                        let fh = he.sp_thrust * he.mdot_air;
                        println!("{:10} {fname:9} {ln:8} LP {:+6.1}%  HP {:+6.2}%  Tt4 {:+6.1} K  thrust {:+6.1}%",
                                 CONTROL_SHAPES[shape], 100.0 * (he.nu_lp / ue.nu_lp - 1.0), 100.0 * (he.nu_hp / ue.nu_hp - 1.0),
                                 he.tt4 - ue.tt4, 100.0 * (fh / fu - 1.0));
                    }
                    (Ok((_, u)), Ok((_, h))) => println!("{:10} {fname:9} {ln:8} stopped: unlit {:?} lit {:?}", CONTROL_SHAPES[shape],
                                                          u.stop.map(|x| format!("{x:?}").chars().take(60).collect::<String>()),
                                                          h.stop.map(|x| format!("{x:?}").chars().take(60).collect::<String>())),
                    (a, b) => println!("{:10} {fname:9} {ln:8} refused: {:?} / {:?}", CONTROL_SHAPES[shape], a.err().map(|e| format!("{e:?}").chars().take(60).collect::<String>()),
                                       b.err().map(|e| format!("{e:?}").chars().take(60).collect::<String>())),
                }
            }
        }
    }
    // The failing test's own case: the defaults, light-off at its default.
    for (ln, law) in laws {
        let s = ControlsSettings { turbine: law, ..d };
        let (_, u) = controls(&ControlsSettings { ab: Some(AbMode::Unlit), ..s }).unwrap();
        let (_, h) = controls(&ControlsSettings { ab: Some(AbMode::Lit), ..s }).unwrap();
        let (ue, he) = (u.points.last().unwrap(), h.points.last().unwrap());
        println!("TEST CASE {ln:8} stop {:?} LP {:+.1}%  HP {:+.2}%  Tt4 {:+.1} K", h.stop.is_some(),
                 100.0 * (he.nu_lp / ue.nu_lp - 1.0), 100.0 * (he.nu_hp / ue.nu_hp - 1.0), he.tt4 - ue.tt4);
    }
}
