// The held lit shaft speed over the unlit one, at the 32 scored cells (H7/H8's grid), and the flame holders' thrust cost.
use turbojet::afterburner::{AbMode, Afterburner, FlyAfterburner, Throat};
use turbojet::atmosphere::Ambient;
use turbojet::sandbox::{fly_solver, FlySettings, MapShape};
fn main() {
    let a3 = Ambient::from_altitude(3000.0, 0.0);
    let a11 = Ambient::from_altitude(11000.0, 0.0);
    let (mut lo, mut hi, mut clo, mut chi) = (9.0f64, 0.0f64, 9.0f64, 0.0f64);
    for map in [MapShape::Flow, MapShape::Pressure] {
        for (t0, p0, m0) in [(250.0, 50_000.0, 0.5), (a3.t0, a3.p0, 0.3), (a11.t0, a11.p0, 1.4), (288.15, 101_325.0, 0.1)] {
            for tt4 in [1300.0, 1500.0] {
                let s = FlySettings { t0, p0, m0, tt4, map, ..FlySettings::defaults() };
                let (fl, cmap) = (s.flight(), s.map.map().with_phi_surge(s.phi_surge));
                let st = fly_solver(&s);
                let none = st.equilibrium(&fl, tt4, Some(&cmap));
                for tt7 in [1700.0, 1900.0] {
                    let fa = FlyAfterburner::capture(&st, Afterburner { tt7, ..Afterburner::default() });
                    let dry = fa.try_point(&fl, tt4, &cmap, AbMode::Unlit, Throat::Held(1.0)).unwrap();
                    let h = fa.try_point_after_dry(&fl, tt4, &cmap, AbMode::Lit, Throat::Held(1.0), &dry).unwrap();
                    let r = h.eq.nu / dry.eq.nu;
                    let c = 1.0 - dry.eq.thrust / none.thrust;
                    lo = lo.min(r); hi = hi.max(r); clo = clo.min(c); chi = chi.max(c);
                }
            }
        }
    }
    println!("held lit speed / unlit speed: {lo:.4} .. {hi:.4};  flame holders' thrust cost (unlit vs none): {clo:.5} .. {chi:.5}");
}
