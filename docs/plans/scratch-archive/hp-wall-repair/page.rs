// The Controls page's own path: build() + march() on flat-lp at low starting throttles, with each closure copy reached.
use turbojet::sandbox_controls::{build, ControlsSettings, CONTROL_SHAPES, CustomerMode, LeverChoice};
use turbojet::atmosphere::standard;

fn main() {
    let flat = CONTROL_SHAPES.iter().position(|&k| k == "flat-lp").unwrap();
    let (t_sl, p_sl) = standard(0.0);
    for (fname, t0, p0) in [("design", ControlsSettings::defaults().t0, ControlsSettings::defaults().p0), ("SL std", t_sl, p_sl)] {
        for variant in ["plain", "bleed lever", "customer", "cooling"] {
            for from in [600.0, 650.0, 700.0, 750.0, 800.0] {
                let mut s = ControlsSettings::defaults();
                s.shape = flat; s.from = from; s.to = 1300.0; s.t0 = t0; s.p0 = p0;
                match variant {
                    "bleed lever" => s.lever = LeverChoice::Bleed,
                    "customer" => s.customer = CustomerMode::Share,
                    "cooling" => { s.cool_vane = 0.03; s.cool_blade = 0.02; }
                    _ => {}
                }
                let r = match build(&s) {
                    Ok(sv) => { let p = sv.march(); format!("RUNS: {} points, last s={:.2}", p.len(), p.last().map(|q| q.s).unwrap_or(f64::NAN)) }
                    Err(e) => { let m = format!("{:?}", e); format!("REFUSED: {}", &m[..m.len().min(150)]) }
                };
                println!("{fname:7} {variant:12} from {from:4.0}: {r}");
            }
        }
    }
}
