// Rung 105 probe P8 — where does the CAPTURE's vane flip from choked to unchoked (bisect the reaction), against G1's
// R* = 1 - (c*^2/2)/dh0 (KE4 = 0) at the design point, both engines, two gases.
use turbojet::components::try_sonic_throat;
use turbojet::engine::{build_turbojet, FlightCondition};
use turbojet::gas::Gas;
use turbojet::hp_vane::{HpCapacity, HpVane};
use turbojet::matcher::OffDesignMatcher;
use turbojet::sandbox::Settings;
use turbojet::sandbox_blades::{rig_losses, BladeGas};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolCore};
fn flip(gas: &Gas, f: f64, tt4: f64, yd: f64, eta: f64, a4: f64) -> f64 {
    let (mut lo, mut hi) = (0.0001, 0.9);
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if HpCapacity::capture(gas, f, tt4, yd, eta, a4, HpVane::Rows { reaction: mid }).choked_d { lo = mid } else { hi = mid }
    }
    0.5 * (lo + hi)
}
fn g1(gas: &Gas, f: f64, tt4: f64, tt_out: f64) -> f64 {
    let (_, _, vs) = try_sonic_throat(gas, tt4, 1.0, f).unwrap();
    1.0 - 0.5 * vs * vs / (gas.h_t(tt4, f) - gas.h_t(tt_out, f))
}
fn main() {
    let fd = FlightCondition::new(250.0, 50_000.0, 0.85);
    let d = Settings::defaults();
    for (gn, gas) in [("CPG", BladeGas::Perfect.gas()), ("TPG", Gas::thermally_perfect())] {
        let e = build_turbojet(gas.clone(), d.pi_c, d.tt4, fd.p0, { let mut l = d.losses(); l.nozzle_convergent = true; l.p_exit = None; l });
        let m = OffDesignMatcher::new(e, fd, 1.0);
        let (s4, s5) = (*m.reference.station("4"), *m.reference.station("5"));
        println!("1sh {gn}: eta {:.3} capture flip R = {:.4}   G1 R* = {:.4}", m.eta_t,
                 flip(m.gas(), s4.far, s4.tt, s5.pt / s4.pt, m.eta_t, m.a4), g1(m.gas(), s4.far, s4.tt, s5.tt));
        let c = TwoSpoolCore::new(build_two_spool_turbojet(gas, 3.0, 6.0, 1500.0, fd.p0, rig_losses()), fd, 1.0);
        let (s4, s45) = (*c.reference.station("4"), *c.reference.station("45"));
        println!("2sh {gn}: eta {:.3} capture flip R = {:.4}   G1 R* = {:.4}", c.eta_hpt,
                 flip(c.gas(), s4.far, s4.tt, s45.pt / s4.pt, c.eta_hpt, c.a4), g1(c.gas(), s4.far, s4.tt, s45.tt));
    }
}
