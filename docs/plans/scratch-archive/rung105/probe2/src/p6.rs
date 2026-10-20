// Rung 105 probe P6 — the law's call census on one Fly-it point and one slam (prototype).
use turbojet::hp_vane::census;
use turbojet::sandbox::{fly, FlySettings};
use turbojet::sandbox_transient::{slam, SlamSettings};
fn main() {
    let _ = census::take();
    let t = std::time::Instant::now();
    let o = fly(&FlySettings { tt4: 1100.0, ..FlySettings::defaults() }).unwrap();
    println!("fly 1100: {:.1} ms nu {:.6}; census [vane, w4, interstage, closure-y] = {:?}", 1e3 * t.elapsed().as_secs_f64(), o.point.nu, census::take());
    let t = std::time::Instant::now();
    let o = slam(&SlamSettings::defaults()).unwrap();
    println!("slam: {:.1} ms points {}; census = {:?}", 1e3 * t.elapsed().as_secs_f64(), o.points.len(), census::take());
}
