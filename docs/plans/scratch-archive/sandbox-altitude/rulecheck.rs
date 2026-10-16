//! The density rule checked AT the limits it sets: gain = 50/max(σ,1), τ = 0.04/min(σ,1), σ = (p0/p0d)(T0d/T0).
use turbojet::burner_hold::Hold;
use turbojet::governor as gv;
use turbojet::sandbox_controls::*;
use turbojet::atmosphere::standard;

fn err(sv: &ControlsSolver) -> (f64, usize) {
    let nu0 = (sv.start.nu_lp, sv.start.nu_hp);
    let g = sv.governor.as_ref().unwrap();
    let run = |ds| gv::march(&sv.core.fuel, &sv.flight, g, &sv.limiters(), None, nu0, sv.settings.s_end(), ds);
    let (a, m) = run(0.02); let (b, _) = run(0.005);
    let n = a.len().min((b.len() + 3) / 4);
    let mut d = 0.0f64;
    for k in 0..n { d = d.max((a[k].phi_lp - b[4 * k].phi_lp).abs()).max((a[k].phi_hp - b[4 * k].phi_hp).abs()); }
    (d, m.iter().filter(|x| x.clamp.key() != "root").count())
}
fn main() {
    let d = ControlsSettings::defaults();
    let mut worst: Vec<(f64, String)> = Vec::new();
    for z in [0.0, 2500.0, 5000.0, 7500.0, 10000.0, 12500.0, 15000.0, 17500.0, 20000.0] {
        for dt in [-40.0, -20.0, 0.0, 20.0, 40.0] {
            let (t, p) = standard(z);
            let (t0, p0) = (t + dt, p);
            let sigma = (p0 / d.p0) * (d.t0 / t0);
            let gain = 50.0 / sigma.max(1.0);
            let tau = 0.04 / sigma.min(1.0);
            for shape in [0usize, 1, 2, 3] {
                let base = ControlsSettings { t0, p0, shape, govern: true, ..d };
                let cases = [
                    ("HP prop", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: gain, ..base }),
                    ("HP rate", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Rate, gov_tau: tau, ..base }),
                    ("LP prop", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Prop, gov_gain: gain, ..base }),
                    ("LP rate", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Rate, gov_tau: tau, ..base }),
                ];
                for (lab, s) in cases {
                    let key = format!("{lab} z {z:>5} dT {dt:+3} {:10} σ {sigma:.3} gain {gain:.1} τ {tau:.3}", CONTROL_SHAPES[shape]);
                    match build(&s) {
                        Ok(sv) => { let (e, c) = err(&sv); worst.push((e, format!("{key}: {e:.2e} (clamped {c})"))); }
                        Err(e) => println!("{key}: REFUSED {}", &e.0[..e.0.len().min(80)]),
                    }
                }
            }
        }
    }
    worst.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    println!("{} runs; worst 25:", worst.len());
    for w in worst.iter().take(25) { println!("  {}", w.1); }
    println!("unclamped over 1e-4:"); for w in worst.iter().filter(|w| w.0 > 1e-4 && w.1.contains("(clamped 0)")) { println!("  {}", w.1); }
    let over = worst.iter().filter(|w| w.0 > 1e-4).count();
    println!("over 1e-4: {over}; over 2e-4: {}", worst.iter().filter(|w| w.0 > 2e-4).count());
}
