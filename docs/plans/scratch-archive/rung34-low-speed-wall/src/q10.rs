// Against the trial worktree: can ANY fly request still reach the page's "No shaft speed gives a workable" words
// (a speed-search end with no workable speed at all)? A wide random sweep through the page's own fly().
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
    let n: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(3000);
    let mut tally = std::collections::BTreeMap::<String, usize>::new();
    for _ in 0..n {
        let mut f = FlySettings::defaults();
        f.gas = if g.u() < 0.7 { GasModel::ThermallyPerfect } else { GasModel::Perfect };
        f.map = MapShape::ALL[g.i(3)];
        let h = g.r(0.0, 12000.0);
        let (t, p) = standard(h);
        f.t0 = t + g.r(-30.0, 40.0);
        f.p0 = p;
        f.m0 = g.r(0.0, 2.0);
        f.tt4 = g.r(300.0, 900.0);
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fly(&f).map(|_| ())));
        let k = match r {
            Ok(Ok(())) => "runs".to_string(),
            Ok(Err(_)) => "precheck refuses".to_string(),
            Err(e) => {
                let m = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                let w = explain_fly(&m);
                if w.starts_with("No shaft speed") { println!("NO_SPEED  T0 {:.1} p0 {:.0} M0 {:.3} Tt4 {:.1} {:?} {:?}: {m}", f.t0, f.p0, f.m0, f.tt4, f.gas, f.map); }
                w.chars().take(50).collect()
            }
        };
        *tally.entry(k).or_default() += 1;
    }
    for (k, v) in tally { println!("{v:5}  {k}"); }
}
