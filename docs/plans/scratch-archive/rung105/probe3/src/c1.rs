// Rung 105 COST (anchor § COST, shipped code): the pages' own entries, the law (0.3) against the pin, best of 5 (slams 3).
use std::time::Instant as T;
use turbojet::hp_vane::{census, HpVane};
use turbojet::sandbox::{fly, FlySettings};
use turbojet::sandbox_blades::{lever_json, BladeSettings};
use turbojet::sandbox_controls::{controls, ControlsSettings};
use turbojet::sandbox_cooling::{cooling_point, CoolingSettings};
use turbojet::sandbox_transient::{slam, SlamSettings, ThrottleMode};

fn best<R>(n: usize, mut f: impl FnMut() -> R) -> (f64, R) {
    let mut b = f64::INFINITY;
    let mut out = None;
    for _ in 0..n { let t = T::now(); let r = f(); b = b.min(t.elapsed().as_secs_f64()); out = Some(r); }
    (b, out.unwrap())
}
fn ms(t: f64) -> String { format!("{:9.1} ms", 1e3 * t) }
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let law = HpVane::law();
    let row = |name: &str, tp: f64, tl: f64, ok: &str, c: [u64; 4]| println!("{name:<22} pin {} law {} ratio {:6.1}x  {ok}  census {:?}", ms(tp), ms(tl), tl / tp, c);
    let fd = FlySettings::defaults();
    for (name, s) in [("fly design", fd), ("fly 1100", FlySettings { tt4: 1100.0, ..fd }), ("fly 800", FlySettings { tt4: 800.0, ..fd }),
                      ("fly hot M.3 1000", FlySettings { tt4: 1000.0, t0: 318.15, p0: 101_325.0, m0: 0.3, ..fd })] {
        let (tp, rp) = best(5, || std::panic::catch_unwind(|| fly(&s).map(|o| o.point.pi_c)).ok());
        let _ = census::take();
        let (tl, rl) = best(5, || std::panic::catch_unwind(|| fly(&FlySettings { hp_vane: law, ..s }).map(|o| o.point.pi_c)).ok());
        let c = census::take();
        row(name, tp, tl, &format!("{rp:?} -> {rl:?}"), c);
    }
    let sd = SlamSettings::defaults();
    for (name, s) in [("slam default", sd), ("slam fuel", SlamSettings { mode: ThrottleMode::Fuel, ..sd }),
                      ("slam 700->1200", SlamSettings { from: 700.0, to: 1200.0, ..sd })] {
        let (tp, rp) = best(3, || std::panic::catch_unwind(|| slam(&s).map(|o| o.points.len())).ok());
        let _ = census::take();
        let ls = SlamSettings { fly: FlySettings { hp_vane: law, ..s.fly }, ..s };
        let (tl, rl) = best(3, || std::panic::catch_unwind(|| slam(&ls).map(|o| o.points.len())).ok());
        row(name, tp, tl, &format!("{rp:?} -> {rl:?}"), census::take());
    }
    let cd = ControlsSettings::defaults();
    for (name, s) in [("controls default", cd),
                      ("controls unchoked", ControlsSettings { t0: 288.15 + 40.0, p0: 101_325.0, m0: 0.85, from: 700.0, to: 1400.0, ..cd }),
                      ("controls cooled", ControlsSettings { cool_vane: 0.05, cool_blade: 0.03, ..cd })] {
        let (tp, rp) = best(5, || std::panic::catch_unwind(|| controls(&s).map(|(_, o)| o.points.len())).ok());
        let _ = census::take();
        let (tl, rl) = best(5, || std::panic::catch_unwind(|| controls(&ControlsSettings { hp_vane: law, ..s }).map(|(_, o)| o.points.len())).ok());
        row(name, tp, tl, &format!("{rp:?} -> {rl:?}"), census::take());
    }
    let bd = BladeSettings::defaults();
    for tt4 in [700.0, 1100.0] {
        let (tp, rp) = best(5, || std::panic::catch_unwind(|| lever_json(&bd, tt4).is_ok()).ok());
        let _ = census::take();
        let (tl, rl) = best(5, || std::panic::catch_unwind(|| lever_json(&BladeSettings { hp_vane: law, ..bd }, tt4).is_ok()).ok());
        row(&format!("blades point {tt4}"), tp, tl, &format!("{rp:?} -> {rl:?}"), census::take());
    }
    let kd = CoolingSettings::defaults();
    for tt4 in [700.0, 1100.0] {
        let (tp, rp) = best(5, || std::panic::catch_unwind(|| cooling_point(&kd, tt4).is_ok()).ok());
        let _ = census::take();
        let (tl, rl) = best(5, || std::panic::catch_unwind(|| cooling_point(&CoolingSettings { hp_vane: law, ..kd }, tt4).is_ok()).ok());
        row(&format!("cooling point {tt4}"), tp, tl, &format!("{rp:?} -> {rl:?}"), census::take());
    }
}
