// Part B census: trials and instants per point (thermally-perfect gas).
use turbojet::afterburner::{take_census, AbMode, Afterburner, FlyAfterburner, Throat};
use turbojet::sandbox::{fly_solver, FlySettings, MapShape};

fn main() {
    for (fname, t0, p0, m0) in [("M0.85", 250.0, 50_000.0, 0.85), ("11km M1.4", 216.65, 22_632.1, 1.4)] {
        let s = FlySettings { t0, p0, m0, map: MapShape::Flow, ..FlySettings::defaults() };
        let fl = s.flight();
        let cmap = s.map.map().with_phi_surge(s.phi_surge);
        let st = fly_solver(&s);
        for tt7 in [1700.0, 1900.0] {
            let fa = FlyAfterburner::capture(&st, Afterburner { tt7, ..Afterburner::default() });
            for (lbl, mode, th) in [("unlit", AbMode::Unlit, Throat::Held(1.0)), ("held", AbMode::Lit, Throat::Held(1.0)), ("match", AbMode::Lit, Throat::Match)] {
                take_census();
                let _ = fa.try_point(&fl, s.tt4, &cmap, mode, th);
                let (t, i) = take_census();
                println!("{fname} Tt7 {tt7} {lbl:<6}: {i} instants, {t} trials ({:.1} per instant)", t as f64 / i as f64);
            }
        }
    }
}
