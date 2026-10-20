// Why does the pin refuse one-shaft hot Mach 0.3 at 700/800 K? Print the panic words, worktree and pin both.
use turbojet::engine::{build_turbojet, FlightCondition};
use turbojet::hp_vane::HpVane;
use turbojet::matcher::OffDesignMatcher;
use turbojet::sandbox::Settings;
use turbojet::sandbox_blades::BladeGas;
fn main() {
    std::panic::set_hook(Box::new(|i| { eprintln!("PANIC: {}", i); }));
    let d = Settings::defaults();
    let fl = FlightCondition::new(318.15, 101_325.0, 0.3);
    let fd = FlightCondition::new(250.0, 50_000.0, 0.85);
    let e = build_turbojet(BladeGas::Perfect.gas(), d.pi_c, d.tt4, fd.p0, { let mut l = d.losses(); l.nozzle_convergent = true; l.p_exit = None; l });
    let m = OffDesignMatcher::new(e, fd, 1.0);
    for law in [HpVane::Pin, HpVane::Rows { reaction: 0.15 }, HpVane::Rows { reaction: 0.3 }] {
        m.set_hp_vane(law);
        for tt4 in [650.0, 700.0, 750.0, 800.0, 850.0, 900.0] {
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| m.match_point(&fl, tt4)));
            match r { Ok(x) => println!("{law:?} {tt4}: ok choked={} pi_t {:.5} pi_c {:.4} m9 {:.4}", x.nozzle_choked, x.pi_t, x.pi_c, x.m9),
                      Err(_) => println!("{law:?} {tt4}: panic (above)") }
        }
    }
}
