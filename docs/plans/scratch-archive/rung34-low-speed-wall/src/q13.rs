// Against the repaired worktree: the afterburner instant's error at each low speed (unlit, held dry throat),
// Fly-it engine, 45 °C sea level, Mach 0.3, 700 K — which step refuses below the low-speed wall?
use turbojet::afterburner::{AbMode, FlyAfterburner};
use turbojet::sandbox::{fly_solver, FlySettings, GasModel};

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let s = FlySettings { gas: GasModel::ThermallyPerfect, t0: 318.15, p0: 101_325.0, m0: 0.3, tt4: 700.0, ab: Some(AbMode::Unlit), ..FlySettings::defaults() };
    let st = fly_solver(&s);
    let fl = s.flight();
    let cmap = s.map.map().with_phi_surge(s.phi_surge);
    let fa = FlyAfterburner::capture(&st, s.afterburner());
    for k in 0..16 {
        let nu = 0.30 + 0.02 * k as f64;
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fa.try_instant(&fl, nu, 700.0, &cmap, AbMode::Unlit, fa.a8_dry)));
        let txt = match r { Ok(Ok(p)) => format!("Phi {:+.5}", p.eq.phi), Ok(Err(e)) => format!("ERR {:.110}", e.0), Err(_) => "PANIC".into() };
        let plain = st.try_instant(&fl, nu, 700.0, Some(&cmap)).map(|i| format!("{:+.5}", i.phi)).unwrap_or_else(|e| format!("ERR {:.40}", e.0));
        let abl = fa.try_instant_below_choke(&fl, nu, 700.0, &cmap, AbMode::Unlit, fa.a8_dry).map(|p| format!("{:+.5}", p.eq.phi)).unwrap_or_else(|e| format!("ERR {:.90}", e.0));
        let low = st.try_instant_below_choke(&fl, nu, 700.0, Some(&cmap)).map(|i| format!("{:+.5}", i.phi)).unwrap_or_else(|e| format!("ERR {:.40}", e.0));
        println!("nu {nu:.2}: ab below choke {abl} | ab {txt:.30} | dry {plain} | dry below choke {low}");
    }
}
