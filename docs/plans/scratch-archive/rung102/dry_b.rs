// Part B pre-flight, DRY ONLY: the 16 scored (flight, Tt4) points on each map shape — does the dry engine run,
// is its nozzle choked, what is its stall margin. Panics are caught per point.
use turbojet::atmosphere::Ambient;
use turbojet::sandbox::{fly_solver, stall_reading, FlySettings, MapShape, Stall};
use turbojet::matcher::Branch;

fn main() {
    let a3 = Ambient::from_altitude(3000.0, 0.0);
    let a11 = Ambient::from_altitude(11000.0, 0.0);
    let flights = [("250K 50kPa M0.5", 250.0, 50_000.0, 0.5), ("3 km M0.3", a3.t0, a3.p0, 0.3),
                   ("11 km M1.4", a11.t0, a11.p0, 1.4), ("SL M0.1", 288.15, 101_325.0, 0.1)];
    std::panic::set_hook(Box::new(|_| {}));
    for map in MapShape::ALL {
        println!("== map {}", map.key());
        for &(name, t0, p0, m0) in &flights {
            for tt4 in [1300.0, 1500.0] {
                let s = FlySettings { t0, p0, m0, tt4, map, ..FlySettings::defaults() };
                let r = std::panic::catch_unwind(|| {
                    let st = fly_solver(&s);
                    let cmap = s.map.map().with_phi_surge(s.phi_surge);
                    let eq = st.equilibrium(&s.flight(), s.tt4, Some(&cmap));
                    let sm = stall_reading(&st, &eq, &cmap);
                    (eq.nu, eq.branch == Branch::Choked, eq.thrust, eq.m9, sm)
                });
                match r {
                    Ok((nu, ch, f, m9, sm)) => {
                        let sm = match sm { Stall::Margin { sm_n, .. } => format!("{sm_n:.4}"), o => format!("{o:?}") };
                        println!("  {name:<16} Tt4 {tt4:.0}: nu {nu:.4} choked {ch} M9 {m9:.4} thrust {f:.1} N  sm {sm}");
                    }
                    Err(_) => println!("  {name:<16} Tt4 {tt4:.0}: DOES NOT RUN (panic)"),
                }
            }
        }
    }
}
