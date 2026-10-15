use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::shaft_offtake::{Demand, ShaftOfftake, ShaftOfftakeMatcher};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolLosses};
fn main() {
    let real = TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
        eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true };
    let g = Gas::new(GasSpec { gamma_c: 1.4, cp_c: 1004.0, r_c: 0.4 / 1.4 * 1004.0, gamma_t: 1.3,
        cp_t: 1239.0, r_t: 0.3 / 1.3 * 1239.0, hpr: 42.8e6, ..GasSpec::default() });
    let fl = FlightCondition::new(250.0, 50_000.0, 0.85);
    if std::env::args().nth(1).as_deref() == Some("650") { for shaft in ["HP","LP"] { for w in [0.0, 0.01, 0.02] { let so = if shaft == "HP" { ShaftOfftake::hp(Demand::power(w)) } else { ShaftOfftake::lp(Demand::power(w)) }; let de = build_two_spool_turbojet(g.clone(), 3.0, 6.0, 1500.0, 50_000.0, real).with_shaft_offtake(so); let m = ShaftOfftakeMatcher::new(de, fl, 1.0, ComponentMap::flat(), ComponentMap::flat(), 0.0); for tt4 in [700.0, 650.0] { match m.try_match_point(&fl, tt4) { Ok(r) => println!("{shaft} w={w} {tt4}: OK p9 {:.0} share {:?}", r.base.base.p9, r.booking.map(|k| (k.share_hp, k.share_lp))), Err(e) => println!("{shaft} w={w} {tt4}: ERR {}", e.0) } } } } return; }
    for shaft in ["HP", "LP"] {
        for w in [0.05, 0.1, 0.12, 0.14, 0.16, 0.18, 0.2, 0.25, 0.3, 0.35, 0.4] {
            let so = if shaft == "HP" { ShaftOfftake::hp(Demand::power(w)) } else { ShaftOfftake::lp(Demand::power(w)) };
            let de = build_two_spool_turbojet(g.clone(), 3.0, 6.0, 1500.0, 50_000.0, real).with_shaft_offtake(so);
            let m = ShaftOfftakeMatcher::new(de, fl, 1.0, ComponentMap::flat(), ComponentMap::flat(), 0.0);
            for tt4 in [1500.0, 1000.0] {
                match m.try_match_point(&fl, tt4) {
                    Ok(r) => { let k = r.booking.unwrap(); println!("{shaft} w={w} Tt4={tt4}: OK share hp {:.3} lp {:.3}", k.share_hp, k.share_lp) }
                    Err(e) => println!("{shaft} w={w} Tt4={tt4}: ERR {}", &e.0[..e.0.len().min(90)]),
                }
            }
        }
    }
}
// (appended) 650 K power-mode failure reasons
#[allow(dead_code)]
fn unused() {}
