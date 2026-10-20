// Rung 105 C0 (off costs nothing): the pages' own entries on the default (the pin), best of 7 (slams 3). The same file
// builds against the code before rung 105 (4ba69ee) and against today's, so it names no rung-105 setting.
use std::time::Instant as T;
use turbojet::sandbox::{fly, FlySettings};
use turbojet::sandbox_blades::{lever_json, BladeSettings};
use turbojet::sandbox_controls::{controls, ControlsSettings};
use turbojet::sandbox_cooling::{cooling_point, CoolingSettings};
use turbojet::sandbox_transient::{slam, SlamSettings, ThrottleMode};

fn best<R>(n: usize, mut f: impl FnMut() -> R) -> f64 {
    let mut b = f64::INFINITY;
    for _ in 0..n { let t = T::now(); let r = f(); b = b.min(t.elapsed().as_secs_f64()); std::hint::black_box(r); }
    b
}
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let tag = std::env::args().nth(1).unwrap_or_default();
    let row = |name: &str, t: f64| println!("{tag} {name:<22} {:9.2} ms", 1e3 * t);
    let fd = FlySettings::defaults();
    for (name, s) in [("fly design", fd), ("fly 1100", FlySettings { tt4: 1100.0, ..fd }), ("fly 800", FlySettings { tt4: 800.0, ..fd }),
                      ("fly hot M.3 1000", FlySettings { tt4: 1000.0, t0: 318.15, p0: 101_325.0, m0: 0.3, ..fd })] {
        row(name, best(7, || std::panic::catch_unwind(|| fly(&s).is_ok()).ok()));
    }
    let sd = SlamSettings::defaults();
    for (name, s) in [("slam default", sd), ("slam fuel", SlamSettings { mode: ThrottleMode::Fuel, ..sd }),
                      ("slam 700->1200", SlamSettings { from: 700.0, to: 1200.0, ..sd })] {
        row(name, best(3, || std::panic::catch_unwind(|| slam(&s).is_ok()).ok()));
    }
    let cd = ControlsSettings::defaults();
    for (name, s) in [("controls default", cd),
                      ("controls unchoked", ControlsSettings { t0: 288.15 + 40.0, p0: 101_325.0, m0: 0.85, from: 700.0, to: 1400.0, ..cd }),
                      ("controls cooled", ControlsSettings { cool_vane: 0.05, cool_blade: 0.03, ..cd })] {
        row(name, best(7, || std::panic::catch_unwind(|| controls(&s).is_ok()).ok()));
    }
    let bd = BladeSettings::defaults();
    for tt4 in [700.0, 1100.0] {
        row(&format!("blades point {tt4}"), best(7, || std::panic::catch_unwind(|| lever_json(&bd, tt4).is_ok()).ok()));
    }
    let kd = CoolingSettings::defaults();
    for tt4 in [700.0, 1100.0] {
        row(&format!("cooling point {tt4}"), best(7, || std::panic::catch_unwind(|| cooling_point(&kd, tt4).is_ok()).ok()));
    }
}
