// Throwaway: the Controls page's design-flight flat-LP low end, old vs new.
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::{build, control_maps, ControlsSettings, CONTROL_SHAPES};
use turbojet::two_spool_transient::TwoSpoolTransientCore;
fn main() {
    let d = ControlsSettings::defaults();
    let flat = CONTROL_SHAPES.iter().position(|&k| k == "flat-lp").unwrap();
    let bs = BladeSettings::defaults();
    let (ml, mh) = control_maps("flat-lp").unwrap();
    let tr = TwoSpoolTransientCore::new(bs.design(), bs.flight(), 1.0, ml, mh, 1.0);
    tr.inner.base.below_choke.set(true);
    let fl = d.flight();
    let mut tt4 = 440.0;
    while tt4 <= 540.0 + 1e-9 {
        let st = tr.inner.try_match_point(&fl, tt4).map(|m| format!("({:.4},{:.4}) F={:.2} N", m.n_lp_ratio, m.n_hp_ratio, m.base.thrust)).map_err(|e| e.0.chars().take(60).collect::<String>());
        let eq = tr.try_equilibrium(&fl, tt4, None).map(|(i, _, _)| format!("({:.4},{:.4}) sp_in={:?} sp={:.2}", i.nu_lp, i.nu_hp, i.sp_thrust_inlet, i.sp_thrust)).map_err(|e| e.0.chars().take(60).collect::<String>());
        let b = build(&ControlsSettings { shape: flat, from: tt4, ..d }).map(|_| "runs".to_string()).map_err(|e| e.0.chars().take(90).collect::<String>());
        println!("{tt4} K\n  steady {st:?}\n  eq     {eq:?}\n  page   {b:?}");
        tt4 += 10.0;
    }
}
