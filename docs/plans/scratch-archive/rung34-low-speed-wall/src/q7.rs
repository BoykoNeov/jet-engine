// Rung 34's low-speed wall: at nu below ~0.6 the CHOKED-star nozzle (the turbine at the choke pi_t) cannot expand to
// p0 and its `?` ends the instant BEFORE the subsonic turbine solve runs. What does the subsonic solve give there?
// Replicates try_instant_tail with that one `?` treated as "subsonic".
use turbojet::components::{ram_recovery, Nozzle};
use turbojet::engine::{build_turbojet, FlightCondition};
use turbojet::gas::FlowState;
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
        let mm = &st.inner.inner;
        for tt4 in [700.0, 750.0] {
            println!("-- {mname} Tt4 {tt4}");
            let pi_d = mm.pi_d_max * ram_recovery(fl.m0);
            let (s0, v0) = mm.freestream_for(&fl);
            let (tt2, pt2) = (s0.tt, pi_d * s0.pt);
            for k in 0..=20 {
                let nu = 0.30 + 0.02 * k as f64;
                let n = nu * (st.inner.tt2_d / tt2).powf(0.5);
                let comp = match st.try_close_compressor(tt4, tt2, pt2, &map, n) {
                    Ok(c) => c,
                    Err(e) => { println!("nu {nu:.2}: closure ERR {}", e.0.chars().take(90).collect::<String>()); continue }
                };
                let wgas = comp.gas(mm);
                let nu_t = nu * (st.inner.tt4_d / tt4).powf(0.5);
                let eta_t = map.eta_t_at(mm.eta_t, nu_t);
                let (pi_t_c, _, tt5_c) = mm.try_solve_turbine(wgas, tt4, comp.f, Some(eta_t)).unwrap();
                let s5 = FlowState { tt: tt5_c, pt: pi_t_c * comp.pt4, mdot: comp.mdot_air, far: comp.f };
                let star = Nozzle::convergent(fl.p0, mm.pi_n).try_apply(&s5, wgas);
                let star_s = match &star { Ok(e) => format!("star M9 {:.3}", e.m9), Err(_) => format!("star ERR (pt9 {:.0})", mm.pi_n * pi_t_c * comp.pt4) };
                match st.try_turbine_subsonic(wgas, tt4, comp.f, comp.pt4, comp.mdot4, eta_t, fl.p0) {
                    Ok((pi_t, _tau, tt5, ex)) => {
                        let pt_spec = mm.eta_m * (1.0 + comp.f) * (wgas.h_t(tt4, comp.f) - wgas.h_t(tt5, comp.f));
                        let pc_spec = wgas.h_c(comp.tt3) - wgas.h_c(tt2);
                        let phi = comp.mdot_air * (pt_spec - pc_spec) / (st.p_ref * nu);
                        let fsp = (1.0 + comp.f) * ex.v9 - v0;
                        println!("nu {nu:.2}: pi_c {:.3} choke pi_t {pi_t_c:.4} {star_s} | subsonic pi_t {pi_t:.4} M9 {:.3} Phi {phi:+.4} F/mdot {fsp:.1}", comp.pi_c, ex.m9);
                    }
                    Err(e) => println!("nu {nu:.2}: pi_c {:.3} choke pi_t {pi_t_c:.4} {star_s} | subsonic ERR {}", comp.pi_c, e.0.chars().take(70).collect::<String>()),
                }
            }
        }
    }
}
