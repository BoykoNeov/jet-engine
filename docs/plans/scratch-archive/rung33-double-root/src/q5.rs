// Why does rung 34's speed search refuse hot Mach 0.3, 650–725 K? What fails below nu 0.6, and where is rung 33's root in nu?
use turbojet::engine::{build_turbojet, FlightCondition};
use turbojet::map::ComponentMap;
use turbojet::sandbox::{FlySettings, GasModel};
use turbojet::spool::SpoolTransient;

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let s = FlySettings { gas: GasModel::ThermallyPerfect, ..FlySettings::defaults() };
    let d = s.capture();
    let fl = FlightCondition::new(318.15, 101_325.0, 0.30);
    for (mname, map) in [("flat", ComponentMap::flat()), ("page default", s.map.map())] {
        let st = SpoolTransient::new(build_turbojet(d.gas.gas(), d.pi_c, d.tt4, d.p0, d.losses()), d.flight(), d.mdot, map);
        for tt4 in [700.0, 750.0] {
            println!("-- {mname} Tt4 {tt4}");
            for k in 0..=20 {
                let nu = 0.30 + 0.02 * k as f64;
                match st.try_instant(&fl, nu, tt4, None) {
                    Ok(i) => println!("nu {nu:.2}: Phi {:+.4} {:?} pi_c {:.3} pi_t {:.4}", i.phi, i.branch, i.pi_c, i.pi_t),
                    Err(e) => println!("nu {nu:.2}: ERR {}", e.0.chars().take(150).collect::<String>()),
                }
            }
        }
    }
}
