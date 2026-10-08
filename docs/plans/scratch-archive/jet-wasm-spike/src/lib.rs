use std::sync::Mutex;
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::Gas;

fn losses() -> Losses {
    Losses { pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, eta_m: 0.99, pi_n: 0.98, ..Losses::default() }
}
fn gas(k: u32) -> Gas {
    match k { 0 => Gas::default(), 1 => Gas::thermally_perfect(), 2 => Gas::reacting(), _ => Gas::reacting_equilibrium() }
}

static MSG: Mutex<Vec<u8>> = Mutex::new(Vec::new());

/// Install a hook that keeps the panic text where JS can read it after the trap.
#[no_mangle]
pub extern "C" fn init() {
    std::panic::set_hook(Box::new(|info| {
        let s = info.to_string();
        if let Ok(mut m) = MSG.lock() { *m = s.into_bytes(); }
    }));
}
#[no_mangle]
pub extern "C" fn msg_ptr() -> *const u8 { MSG.lock().map(|m| m.as_ptr()).unwrap_or(std::ptr::null()) }
#[no_mangle]
pub extern "C" fn msg_len() -> usize { MSG.lock().map(|m| m.len()).unwrap_or(0) }

#[no_mangle]
pub extern "C" fn st(k: u32, pi_c: f64, tt4: f64) -> f64 {
    let fl = FlightCondition::new(250.0, 50_000.0, 0.85);
    build_turbojet(gas(k), pi_c, tt4, fl.p0, losses()).run(&fl, 1.0).performance.specific_thrust
}
#[no_mangle]
pub extern "C" fn tsfc(k: u32, pi_c: f64, tt4: f64) -> f64 {
    let fl = FlightCondition::new(250.0, 50_000.0, 0.85);
    build_turbojet(gas(k), pi_c, tt4, fl.p0, losses()).run(&fl, 1.0).performance.tsfc
}
