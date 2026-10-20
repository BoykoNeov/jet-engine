use turbojet::hp_vane::HpVane;
use turbojet::sandbox::{fly, FlySettings};
fn main() {
    let s = FlySettings { hp_vane: HpVane::law(), ..FlySettings::defaults() };
    match std::panic::catch_unwind(|| fly(&s)) { Ok(Ok(o)) => println!("ok {}", o.point.pi_c), Ok(Err(e)) => println!("err {}", e.0), Err(_) => println!("panicked (message above)") }
}
