// Probe: at below-idle ends the page now keeps, does the DISPLAYED thrust (governor::thrust) share the sign of the
// thrust per inlet air the rule judges? Variants x flights x low throttles.
use turbojet::sandbox_controls::{build, ControlsSolver, ControlsSettings, CustomerMode, LeverChoice, CONTROL_SHAPES};
fn main() {
    let d = ControlsSettings::defaults();
    let (t_sl, p_sl) = turbojet::atmosphere::standard(0.0);
    let (mut n, mut disagree) = (0, 0);
    for variant in ["plain", "bleed lever", "customer", "customer amount", "cooling", "customer+cooling"] {
        for (name, t0, p0, m0) in [("design", d.t0, d.p0, d.m0), ("M2", d.t0, d.p0, 2.0), ("M2.6", d.t0, d.p0, 2.6), ("SL", t_sl, p_sl, 0.85)] {
            for (k, shape) in CONTROL_SHAPES.iter().enumerate() {
                let mut s = ControlsSettings { shape: k, t0, p0, m0, ..d };
                match variant {
                    "bleed lever" => s.lever = LeverChoice::Bleed,
                    "customer" => s.customer = CustomerMode::Share,
                    "customer amount" => { s.customer = CustomerMode::Amount; s.customer_amount = 0.05; }
                    "cooling" => { s.cool_vane = 0.06; s.cool_blade = 0.04; }
                    "customer+cooling" => { s.customer = CustomerMode::Share; s.cool_vane = 0.06; s.cool_blade = 0.04; }
                    _ => {}
                }
                let mut tt4 = 400.0;
                while tt4 <= 1100.0 {
                    if let Ok(sv) = build(&ControlsSettings { from: tt4, ..s }) {
                        let i = &sv.start;
                        let sp = i.sp_thrust_inlet.unwrap_or(i.sp_thrust);
                        let shown = ControlsSolver::thrust(i);
                        if sp <= 0.0 || shown <= 0.0 {
                            n += 1;
                            if (sp > 0.0) != (shown > 0.0) {
                                disagree += 1;
                                println!("DISAGREE {variant} | {name} | {shape} {tt4}: per-inlet {sp:.3}, shown {shown:.4} N");
                            }
                        }
                    }
                    tt4 += 20.0;
                }
            }
        }
    }
    println!("starts with either sign <= 0: {n}; signs disagree: {disagree}");
}
