// Rung 104 probe P6 — time Controls runs (the page's own entry) under the law in R104_LAW.
use turbojet::burner_hold::Hold;
use turbojet::sandbox_controls::{controls, ControlsSettings, CustomerMode, GeneratorMode};
use turbojet::turbine_law::census;
fn main() {
    let d = ControlsSettings::defaults();
    let cases: Vec<(&str, ControlsSettings)> = vec![
        ("default 1000->1400", d),
        ("SL+40 M0.85 700->1400", ControlsSettings { t0: 288.15 + 40.0, p0: 101_325.0, m0: 0.85, from: 700.0, to: 1400.0, ..d }),
        ("SL+40 M0.85 1400->700", ControlsSettings { t0: 288.15 + 40.0, p0: 101_325.0, m0: 0.85, from: 1400.0, to: 700.0, ..d }),
        ("governed thrust", ControlsSettings { govern: true, hold: Hold::Thrust, ..d }),
        ("cooling 0.05/0.03", ControlsSettings { cool_vane: 0.05, cool_blade: 0.03, ..d }),
        ("generator HP", ControlsSettings { generator: GeneratorMode::Hp, ..d }),
        ("customer share", ControlsSettings { customer: CustomerMode::Share, ..d }),
    ];
    let law = std::env::var("R104_LAW").unwrap_or_default();
    for (name, s) in cases {
        let _ = controls(&s);
        census::take();
        let t0 = std::time::Instant::now();
        for _ in 0..9 { let _ = controls(&s); }
        let r = controls(&s);
        let dt = t0.elapsed().as_secs_f64() / 10.0;
        let c = census::take().map(|v| v / 10);
        print!("[pairs {} pair-evals {} w45 {} interstage {}] ", c[0], c[1], c[2], c[3]);
        match r {
            Ok((_, o)) => println!("{law:8} {name:24} {dt:8.4} s  points {:4} stop {:?}", o.points.len(),
                                   o.stop.as_ref().map(|x| format!("{x:?}").chars().take(90).collect::<String>())),
            Err(e) => println!("{law:8} {name:24} {dt:8.4} s  DOES NOT RUN {}", format!("{e:?}").chars().take(120).collect::<String>()),
        }
    }
}
