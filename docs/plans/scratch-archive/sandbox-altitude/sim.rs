//! (1) Where the design flight sits; (2) similarity through build: same T0, p0 x 0.5 / 2; (3) default run over altitude.
use turbojet::atmosphere::{Ambient, standard};
use turbojet::sandbox_controls::{build, controls, ControlsSettings, ControlsSolver};

fn main() {
    let a = Ambient::from_ambient(250.0, 50_000.0);
    println!("design flight: pressure altitude {:.1} m, deviation {:+.3} K", a.altitude, a.delta_t);
    let d = ControlsSettings::defaults();
    let b0 = build(&d).unwrap();
    for k in [0.5, 2.0] {
        let b = build(&ControlsSettings { p0: d.p0 * k, ..d }).unwrap();
        for (n, x, y) in [("start", &b0.start, &b.start), ("end", &b0.end, &b.end)] {
            println!("p0 x{k} {n}: dnuL {:.2e} dnuH {:.2e} mdot/k {:.2e} fuel/k {:.2e} thrust/k {:.2e}",
                (x.nu_lp - y.nu_lp).abs(), (x.nu_hp - y.nu_hp).abs(),
                y.close.mdot_air / (k * x.close.mdot_air) - 1.0,
                (y.close.f * y.close.mdot_air) / (k * x.close.f * x.close.mdot_air) - 1.0,
                ControlsSolver::thrust(y) / (k * ControlsSolver::thrust(x)) - 1.0);
        }
    }
    for z in [-1000.0, 0.0, 2000.0, 4000.0, 5574.0, 8000.0, 11000.0, 13000.0, 15000.0, 18000.0, 20000.0] {
        for dt in [0.0] {
            let (t, p) = standard(z);
            let s = ControlsSettings { t0: t + dt, p0: p, ..d };
            let t1 = std::time::Instant::now();
            match controls(&s) {
                Err(e) => println!("z {z:>6} dT {dt:+}: REFUSED {}", e.0),
                Ok((sv, o)) => {
                    let l = o.points.last().unwrap();
                    println!("z {z:>6}: T0 {:.1} p0 {:.0}  start nuH {:.4} end nuH {:.4} thrust {:.1}->{:.1}  pts {}/{} peakTt4 {:.1}  last nuH {:.4} (gap {:.2e})  {:.0} ms",
                        t + dt, p, sv.start.nu_hp, sv.end.nu_hp, ControlsSolver::thrust(&sv.start), ControlsSolver::thrust(&sv.end),
                        o.points.len(), s.expected_points(), o.points.iter().map(|x| x.tt4).fold(0.0, f64::max), l.nu_hp,
                        (l.nu_hp - sv.end.nu_hp).abs(), t1.elapsed().as_secs_f64() * 1e3);
                }
            }
        }
    }
}
