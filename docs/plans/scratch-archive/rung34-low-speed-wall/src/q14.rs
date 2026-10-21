// q10's sweep, one line per request (index, flight, outcome class) — to diff the shipped code against the repair.
// For a request that RUNS, also its thrust and nozzle state.
use turbojet::atmosphere::standard;
use turbojet::sandbox::{explain_fly, fly, FlySettings, GasModel, MapShape};

struct Lcg(u64);
impl Lcg {
    fn u(&mut self) -> f64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); (self.0 >> 11) as f64 / (1u64 << 53) as f64 }
    fn r(&mut self, a: f64, b: f64) -> f64 { a + (b - a) * self.u() }
    fn i(&mut self, n: usize) -> usize { ((self.u() * n as f64) as usize).min(n - 1) }
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let mut g = Lcg(10);
    for k in 0..3000 {
        let mut f = FlySettings::defaults();
        f.gas = if g.u() < 0.7 { GasModel::ThermallyPerfect } else { GasModel::Perfect };
        f.map = MapShape::ALL[g.i(3)];
        let h = g.r(0.0, 12000.0);
        let (t, p) = standard(h);
        f.t0 = t + g.r(-30.0, 40.0);
        f.p0 = p;
        f.m0 = g.r(0.0, 2.0);
        f.tt4 = g.r(300.0, 900.0);
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fly(&f).map(|o| (o.point.thrust, o.point.nu, o.point.choked))));
        let k2 = match r {
            Ok(Ok((t, nu, ch))) => format!("runs thrust {t:.1} nu {nu:.4} choked {ch}"),
            Ok(Err(_)) => "precheck".to_string(),
            Err(e) => {
                let m = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                format!("{} || {:.90}", explain_fly(&m).chars().take(30).collect::<String>(), m)
            }
        };
        println!("{k}\tT0 {:.1} p0 {:.0} M0 {:.3} Tt4 {:.1} {:?} {:?}\t{k2}", f.t0, f.p0, f.m0, f.tt4, f.gas, f.map);
    }
}
