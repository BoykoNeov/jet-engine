// Rung 104 probe P17 — THE SORT, rung 102's two afterburner stops (the redline cutting toward the held lean wall; a Tt7
// too hot), under each setting: does the run stop, and in what words?
use turbojet::afterburner::AbMode;
use turbojet::sandbox_controls::{controls, ControlsSettings};
use turbojet::turbine_law::TurbineLaw;

fn main() {
    let d = ControlsSettings::defaults();
    let lit = |s: ControlsSettings| ControlsSettings { ab: Some(AbMode::Lit), ..s };
    for (name, s) in [("redline + late light-off", lit(ControlsSettings { light_at: 0.24, redline_on: true, ..d })),
                      ("Tt7 2400", lit(ControlsSettings { light_at: 0.24, tt7: 2400.0, ..d }))] {
        for (ln, law) in [("pin", TurbineLaw::Pin), ("1row", TurbineLaw::OneRow), ("2rows", TurbineLaw::TwoRows), ("ellipse", TurbineLaw::Ellipse)] {
            let (_, o) = controls(&ControlsSettings { turbine: law, ..s }).unwrap();
            let last = o.points.last().unwrap();
            match &o.stop {
                Some(st) => println!("{name:26} {ln:8} STOPS after {} points: {}", o.points.len(),
                                     st.failure.as_ref().map(|f| f.message.chars().take(110).collect::<String>()).unwrap_or_default()),
                None => println!("{name:26} {ln:8} runs to the end ({} points), end Tt4 {:.1} nuL {:.4} nuH {:.4}", o.points.len(),
                                 last.tt4, last.nu_lp, last.nu_hp),
            }
        }
    }
}
