//! The lowest throttle with a steady point (start end), per map and flight; and with a fixed generator / customer amount.
use turbojet::atmosphere::standard;
use turbojet::sandbox_controls::*;
fn lowest(s: ControlsSettings) -> (f64, String) {
    let mut last = String::new();
    for k in 0..=120 {
        let t = 500.0 + 10.0 * k as f64;
        match build(&ControlsSettings { from: t, to: t.max(1400.0), ..s }) {
            Ok(_) => return (t, last),
            Err(e) => {
                last = if e.0.contains("nozzle back-pressure") { "nozzle".into() } else if e.0.contains("maps the model covers") { "off-map".into() }
                       else if e.0.contains("generator") { "generator".into() } else if e.0.contains("customer") { "customer".into() }
                       else { e.0.chars().take(60).collect() };
            }
        }
    }
    (f64::NAN, last)
}
fn main() {
    let d = ControlsSettings::defaults();
    let mut fl = vec![("design".to_string(), d.t0, d.p0)];
    for (z, dt) in [(0.0, -40.0), (0.0, 0.0), (0.0, 40.0), (11000.0, -40.0), (11000.0, 0.0), (11000.0, 40.0), (20000.0, 0.0)] {
        let (t, p) = standard(z); fl.push((format!("{z:.0} m {dt:+}"), t + dt, p));
    }
    for (name, t0, p0) in &fl {
        print!("{name:>12} T0 {t0:.0}:");
        for shape in 0..4 {
            let (t, why) = lowest(ControlsSettings { t0: *t0, p0: *p0, shape, ..d });
            print!("  {} {t} (below: {why})", CONTROL_SHAPES[shape]);
        }
        let (tg, wg) = lowest(ControlsSettings { t0: *t0, p0: *p0, generator: GeneratorMode::Hp, generator_power: 0.05, ..d });
        let (tc, wc) = lowest(ControlsSettings { t0: *t0, p0: *p0, customer: CustomerMode::Amount, customer_amount: 0.1, ..d });
        println!("  | HP gen 0.05: {tg} ({wg}) | customer 0.1: {tc} ({wc})");
    }
}
