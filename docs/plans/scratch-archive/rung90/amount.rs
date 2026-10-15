//! Is the amount-mode refusal at w = 0.3 real? Steady matcher on the sandbox's Controls engine.
use turbojet::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::{control_maps, ControlsSettings, CONTROL_SHAPES};

fn main() {
    let s = ControlsSettings::defaults();
    let fl = s.flight();
    let (ml, mh) = control_maps(CONTROL_SHAPES[0]).unwrap();
    for w in [0.1, 0.2, 0.22, 0.24, 0.26, 0.28, 0.3] {
        let de = BladeSettings::defaults().design().with_customer_bleed(CustomerBleed::amount(w));
        let m = CustomerBleedMatcher::new(de, fl, 1.0, ml, mh, 0.0);
        let mut line = format!("w={w}:");
        for tt4 in [1500.0, 1400.0, 1200.0, 1000.0] {
            line += &match m.try_match_point(&fl, tt4) {
                Ok(r) => { let k = r.booking.unwrap(); format!(" {tt4}: c={:.4} hpc={:.4} phiL={:.4}", k.customer, k.mdot_hpc, r.base.phi_lp) }
                Err(e) => format!(" {tt4}: ERR {}", e.0.chars().take(60).collect::<String>()),
            };
        }
        println!("{line}");
    }
}
