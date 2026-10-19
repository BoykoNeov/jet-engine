// Probe: can the one-shaft Fly page show a running point with negative net thrust?
use turbojet::sandbox::{fly, FlySettings};
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let d = FlySettings::defaults();
    println!("defaults: T0 {} p0 {} M0 {} Tt4 {}", d.t0, d.p0, d.m0, d.tt4);
    let (mut ok, mut neg, mut err, mut panics) = (0, 0, 0, 0);
    for t0 in [d.t0, 288.15, 328.15] {
        for m0 in [0.3, 0.8, 1.2, 1.6, 2.0, 2.4, 2.8] {
            for k in 0..=30 {
                let tt4 = 500.0 + 25.0 * k as f64;
                let s = FlySettings { t0, m0, tt4, ..d };
                match std::panic::catch_unwind(|| fly(&s)) {
                    Ok(Ok(o)) => {
                        ok += 1;
                        if !(o.point.thrust > 0.0) {
                            neg += 1;
                            println!("NEG T0 {t0} M {m0} Tt4 {tt4}: thrust {:.3} choked {} sp {:.2}", o.point.thrust, o.point.choked,
                                     o.result.performance.specific_thrust);
                        }
                    }
                    Ok(Err(_)) => err += 1,
                    Err(_) => { panics += 1; println!("PANIC T0 {t0} M {m0} Tt4 {tt4}"); }
                }
            }
        }
    }
    println!("ok {ok} (negative thrust {neg}), refused {err}, panics {panics}");
}
