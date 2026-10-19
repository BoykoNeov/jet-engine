use turbojet::sandbox_controls::{build, ControlsSettings, GeneratorMode, LeverChoice, CONTROL_SHAPES};
fn main() {
    let flat = CONTROL_SHAPES.iter().position(|&k| k == "flat-lp").unwrap();
    let d = ControlsSettings::defaults();
    for from in [500.0, 510.0, 520.0] {
        for (name, s) in [("plain", ControlsSettings { shape: flat, from, ..d }),
                          ("to", ControlsSettings { shape: flat, to: from, ..d }),
                          ("bleed", ControlsSettings { shape: flat, from, lever: LeverChoice::Bleed, ..d }),
                          ("bleed+gen", ControlsSettings { shape: flat, from, lever: LeverChoice::Bleed, generator: GeneratorMode::Lp, generator_power: 0.005, ..d })] {
            let r = match build(&s) { Ok(_) => "RUNS".to_string(), Err(e) => e.0[..e.0.len().min(160)].to_string() };
            println!("{from} {name}: {r}");
        }
    }
}
