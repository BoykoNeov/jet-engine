// Census of the page path: dry (unlit, full search) then lit held from the dry speed.
use turbojet::afterburner::{take_census, AbMode, Afterburner, FlyAfterburner, Throat};
use turbojet::atmosphere::Ambient;
use turbojet::sandbox::{fly_solver, FlySettings, MapShape};

fn main() {
    let a11 = Ambient::from_altitude(11000.0, 0.0);
    for (fname, t0, p0, m0) in [("M0.5", 250.0, 50_000.0, 0.5), ("11km M1.4", a11.t0, a11.p0, 1.4), ("SL M0.1", 288.15, 101_325.0, 0.1)] {
        for tt4 in [1300.0, 1500.0] {
            let s = FlySettings { t0, p0, m0, tt4, map: MapShape::Pressure, ..FlySettings::defaults() };
            let fl = s.flight();
            let cmap = s.map.map().with_phi_surge(s.phi_surge);
            let st = fly_solver(&s);
            let fa = FlyAfterburner::capture(&st, Afterburner { tt7: 1900.0, ..Afterburner::default() });
            take_census();
            let dry = fa.try_point(&fl, tt4, &cmap, AbMode::Unlit, Throat::Held(1.0)).unwrap();
            let (td, idr) = take_census();
            let lit = fa.try_point_after_dry(&fl, tt4, &cmap, AbMode::Lit, Throat::Held(1.0), &dry).unwrap();
            let (tl, il) = take_census();
            println!("{fname} Tt4 {tt4}: dry {idr} instants {td} trials (nu {:.3}); lit held {il} instants {tl} trials (nu {:.3})", dry.eq.nu, lit.eq.nu);
        }
    }
}
