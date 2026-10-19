// Part B on the equilibrium gas (and reacting): cost, best of 3, opening flight, flow map, Tt7 1900.
use std::time::Instant as Clock;
use turbojet::afterburner::{AbMode, Afterburner, FlyAfterburner, Throat};
use turbojet::sandbox::{fly_solver, FlySettings, GasModel};

fn best<T>(mut f: impl FnMut() -> T) -> (f64, T) {
    let mut b = f64::INFINITY; let mut out = None;
    for _ in 0..3 { let t = Clock::now(); let r = f(); b = b.min(t.elapsed().as_secs_f64()); out = Some(r); }
    (b * 1e3, out.unwrap())
}

fn main() {
    for gas in [GasModel::Reacting, GasModel::Equilibrium] {
        let s = FlySettings { gas, ..FlySettings::defaults() };
        let fl = s.flight();
        let cmap = s.map.map().with_phi_surge(s.phi_surge);
        let st = fly_solver(&s);
        let (t_dry, dry) = best(|| st.equilibrium(&fl, s.tt4, Some(&cmap)));
        let fa = FlyAfterburner::capture(&st, Afterburner { tt7: 1900.0, ..Afterburner::default() });
        let (t_un, un) = best(|| fa.try_point(&fl, s.tt4, &cmap, AbMode::Unlit, Throat::Held(1.0)));
        let (t_h, h) = best(|| fa.try_point(&fl, s.tt4, &cmap, AbMode::Lit, Throat::Held(1.0)));
        let (t_m, m) = best(|| fa.try_point(&fl, s.tt4, &cmap, AbMode::Lit, Throat::Match));
        println!("{gas:?}: dry {t_dry:.0} ms nu {:.6}; unlit {t_un:.0} ms (x{:.2}) {:?}", dry.nu, t_un / t_dry, un.as_ref().map(|p| p.eq.nu));
        println!("   held {t_h:.0} ms (x{:.2}) {:?}", t_h / t_dry, h.as_ref().map(|p| (p.eq.nu, p.eq.thrust / dry.thrust, p.detail.passes)).map_err(|e| e.0.clone()));
        println!("   match {t_m:.0} ms (x{:.2}) {:?}", t_m / t_dry, m.as_ref().map(|p| (p.s, p.eq.thrust / dry.thrust, p.detail.passes)).map_err(|e| e.0.clone()));
    }
}
