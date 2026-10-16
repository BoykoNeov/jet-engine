//! The final review's unmeasured claims, measured: (a) at the calibrated ends, the fan-speed control's HP speed and Tt4
//! against the generator-off engine's ends; (b) the pressure-ratio run's fuel against the true-thrust run's over the ramp.
use turbojet::burner_hold::Hold;
use turbojet::sandbox_controls::{build, controls, ControlsSettings, GeneratorMode, ThrustBy, CONTROL_SHAPES};
fn main() {
    let d = ControlsSettings { govern: true, hold: Hold::Thrust, ..ControlsSettings::defaults() };
    for shape in [0usize, 3] {
        let clean = build(&ControlsSettings { shape, thrust_by: ThrustBy::N1, ..d }).unwrap();
        for g in [GeneratorMode::Hp, GeneratorMode::Lp] {
            let sv = build(&ControlsSettings { shape, thrust_by: ThrustBy::N1, generator: g, ..d }).unwrap();
            for (end, a, b) in [("1000", &sv.start, &clean.start), ("1400", &sv.end, &clean.end)] {
                println!("{:10} {g:?} {end}: nu_L {:.5} vs {:.5}  nu_H {:.5} vs {:.5} ({:+.4})  Tt4 {:.1} vs {:.1}", CONTROL_SHAPES[shape],
                         a.nu_lp, b.nu_lp, a.nu_hp, b.nu_hp, a.nu_hp - b.nu_hp, a.tt4, b.tt4);
            }
        }
    }
    for shape in 0..4 {
        let (_, e) = controls(&ControlsSettings { shape, thrust_by: ThrustBy::Epr, ..d }).unwrap();
        let (_, t) = controls(&ControlsSettings { shape, ..d }).unwrap();
        let r: Vec<f64> = (1..=25).map(|k| e.points[k].mf / t.points[k].mf - 1.0).collect();
        let (lo, hi) = r.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(l, h), &x| (l.min(x), h.max(x)));
        let all: Vec<f64> = e.points.iter().zip(&t.points).map(|(a, b)| a.mf / b.mf - 1.0).collect();
        let neg = all.iter().filter(|&&x| x < 0.0).count();
        println!("{:10} EPR/true fuel - 1 over the ramp {lo:+.4}..{hi:+.4}; points below over the whole run {neg}/{}", CONTROL_SHAPES[shape], all.len());
    }
}
