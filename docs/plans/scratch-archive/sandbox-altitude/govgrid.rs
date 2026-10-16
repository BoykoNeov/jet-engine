//! The two governor knobs' step error over the flight box: prop gain vs flight, rate tau vs flight; with the
//! end point's HP turbine power over the design (the shaft's response-rate factor) printed beside.
use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::FuelPoint;
use turbojet::governor as gv;
use turbojet::sandbox_controls::*;
use turbojet::atmosphere::standard;

fn march(sv: &ControlsSolver, ds: f64) -> (Vec<FuelPoint>, usize, usize) {
    let nu0 = (sv.start.nu_lp, sv.start.nu_hp);
    let g = sv.governor.as_ref().unwrap();
    let (p, m) = gv::march(&sv.core.fuel, &sv.flight, g, &sv.limiters(), None, nu0, sv.settings.s_end(), ds);
    let rich = m.iter().filter(|x| x.clamp.key() == "rich").count();
    let lean = m.iter().filter(|x| x.clamp.key() == "lean").count();
    (p, rich, lean)
}
fn err(sv: &ControlsSolver) -> (f64, usize, usize) {
    let (a, r, l) = march(sv, 0.02);
    let (b, _, _) = march(sv, 0.005);
    let n = a.len().min((b.len() + 3) / 4);
    let mut d = 0.0f64;
    for k in 0..n { d = d.max((a[k].phi_lp - b[4 * k].phi_lp).abs()).max((a[k].phi_hp - b[4 * k].phi_hp).abs()); }
    (d, r, l)
}
fn main() {
    let d = ControlsSettings::defaults();
    let mut flights = vec![("design".to_string(), d.t0, d.p0)];
    for z in [0.0, 5000.0, 10000.0, 15000.0] { for dt in [-40.0, 0.0, 40.0] {
        let (t, p) = standard(z); flights.push((format!("{z:.0}m{dt:+.0}"), t + dt, p)); } }
    let ref_p = build(&d).unwrap().end.close.mdot_air;
    for (fname, t0, p0) in &flights {
        let base = ControlsSettings { t0: *t0, p0: *p0, govern: true, ..d };
        let sv = match build(&ControlsSettings { hold: Hold::HpSpeed, ..base }) { Ok(s) => s, Err(e) => { println!("{fname}: {}", e.0); continue; } };
        print!("{fname:>10} mdot_end/design {:.2} |", sv.end.close.mdot_air / ref_p);
        for (lab, by) in [("HP", ThrustBy::True), ("LP", ThrustBy::N1)] {
            for g in [10.0, 20.0, 30.0, 50.0] {
                let s = if lab == "HP" { ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: g, ..base } }
                        else { ControlsSettings { hold: Hold::Thrust, thrust_by: by, speed_law: SpeedLaw::Prop, gov_gain: g, ..base } };
                match build(&s) { Ok(sv) => { let (e, r, l) = err(&sv); print!(" {lab}g{g}:{e:.1e}({r}/{l})"); } Err(_) => print!(" {lab}g{g}:REF") }
            }
            for t in [0.04, 0.06, 0.1, 0.2] {
                let s = if lab == "HP" { ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Rate, gov_tau: t, ..base } }
                        else { ControlsSettings { hold: Hold::Thrust, thrust_by: by, speed_law: SpeedLaw::Rate, gov_tau: t, ..base } };
                match build(&s) { Ok(sv) => { let (e, r, l) = err(&sv); print!(" {lab}t{t}:{e:.1e}({r}/{l})"); } Err(_) => print!(" {lab}t{t}:REF") }
            }
        }
        println!();
    }
}
