// Check B: where the OLD march returned a subsonic point, does the grid show 2 roots — and which did it return?
use turbojet::components::ram_recovery;
use turbojet::engine::{build_turbojet, FlightCondition};
use turbojet::matcher::{Branch, OffDesignMatcher};
use turbojet::sandbox::{FlySettings, GasModel};

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let mut flights = vec![(318.15, 101_325.0, 0.30), (318.15, 101_325.0, 0.10), (288.15, 101_325.0, 0.0),
                           (288.15, 101_325.0, 0.30), (250.0, 101_325.0, 0.30), (288.15, 101_325.0, 0.6)];
    for m0 in [0.3, 0.5, 0.85, 1.2, 1.6, 2.0] { flights.push((250.0, 50_000.0, m0)); }
    let (mut n_sub, mut n_two, mut n_wrong) = (0, 0, 0);
    for g in [GasModel::ThermallyPerfect, GasModel::Perfect] {
        let s = FlySettings { gas: g, ..FlySettings::defaults() };
        let d = s.capture();
        let m = OffDesignMatcher::new(build_turbojet(d.gas.gas(), d.pi_c, d.tt4, d.p0, d.losses()), d.flight(), d.mdot);
        for &(t0, p0, m0) in &flights {
            let fl = FlightCondition::new(t0, p0, m0);
            let pi_d = m.pi_d_max * ram_recovery(fl.m0);
            let (s0, _) = m.freestream_for(&fl);
            let (tt2, pt2) = (s0.tt, pi_d * s0.pt);
            for k in 0..=24 {
                let tt4 = 400.0 + 25.0 * k as f64;
                let Ok(od) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| m.match_point(&fl, tt4))) else { continue };
                if od.branch != Branch::Subsonic { continue; }
                n_sub += 1;
                let mut roots = Vec::new();
                let mut prev: Option<(f64, f64)> = None;
                for i in 0..=1600 {
                    let y = 0.15 + (0.9995 - 0.15) * i as f64 / 1600.0;
                    match m.try_subsonic_operating(tt4, tt2, pt2, y, fl.p0) {
                        Ok(o) => { if let Some((py, pv)) = prev { if pv * o.resid < 0.0 { roots.push(0.5 * (py + y)); } } prev = Some((y, o.resid)); }
                        Err(_) => prev = None,
                    }
                }
                if roots.len() >= 2 {
                    n_two += 1;
                    let wrong = (od.pi_t - roots[0]).abs() > 1e-3;
                    if wrong { n_wrong += 1; }
                    println!("{g:?} ({t0},{p0},{m0}) Tt4 {tt4}: old pi_t {:.5}, roots {:?}{}", od.pi_t,
                             roots.iter().map(|r| format!("{r:.4}")).collect::<Vec<_>>(), if wrong { "  <-- NOT THE LOW ROOT" } else { "" });
                }
            }
        }
    }
    println!("subsonic points the old code returned: {n_sub}; with 2+ roots on the grid: {n_two}; returned a root other than the low one: {n_wrong}");
}
