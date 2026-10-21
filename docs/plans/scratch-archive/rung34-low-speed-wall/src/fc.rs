// Fly-it crash map (rung 34 low-speed wall repair): seeded random Fly-it and slam requests through the page's own
// `sandbox::call`, a panic caught. Built against the repo (before) and against the trial worktree (after).
use turbojet::afterburner::AbMode;
use turbojet::atmosphere::standard;
use turbojet::sandbox::{call, FlySettings, GasModel, MapShape};
use turbojet::sandbox_transient::{SlamSettings, ThrottleMode};

struct Lcg(u64);
impl Lcg {
    fn u(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
    fn r(&mut self, a: f64, b: f64) -> f64 { a + (b - a) * self.u() }
    fn i(&mut self, n: usize) -> usize { ((self.u() * n as f64) as usize).min(n - 1) }
}

fn main() {
    let mut g = Lcg(34);
    let n: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(800);
    std::panic::set_hook(Box::new(|_| {}));
    for i in 0..n {
        let mut f = FlySettings::defaults();
        f.gas = if g.u() < 0.8 { GasModel::ThermallyPerfect } else { GasModel::Perfect };
        f.map = MapShape::ALL[g.i(3)];
        // half the flights near the ground and slow (where the wall lives), the rest anywhere
        let (h, m0) = if g.u() < 0.5 { (g.r(0.0, 2000.0), g.r(0.0, 0.5)) } else { (g.r(0.0, 11000.0), g.r(0.0, 1.6)) };
        let (t, p) = standard(h);
        f.t0 = t + g.r(-20.0, 30.0);
        f.p0 = p;
        f.m0 = m0;
        f.tt4 = g.r(550.0, 1500.0);
        if g.u() < 0.1 { f.ab = Some(if g.u() < 0.5 { AbMode::Unlit } else { AbMode::Lit }); }
        let kind = g.u();
        let q = if kind < 0.70 {
            let op = if g.u() < 0.8 { "fly" } else { "running_point" };
            format!(r#"{{"op":"{op}","fly":{}}}"#, f.to_json().dump_compact())
        } else {
            let mut s = SlamSettings::defaults();
            s.fly = f;
            s.from = g.r(550.0, 1500.0);
            s.to = g.r(550.0, 1500.0);
            s.ramp = (g.r(0.0, 1.0) / 0.05).round() * 0.05;
            s.settle = g.r(0.5, 3.0);
            s.mode = ThrottleMode::ALL[g.i(2)];
            format!(r#"{{"op":"slam","slam":{}}}"#, s.to_json().dump_compact())
        };
        let t = std::time::Instant::now();
        let r = std::panic::catch_unwind(|| call(&q));
        let ms = t.elapsed().as_millis();
        match r {
            Ok(a) => println!("{i}\t{ms}\t{q}\t{a}"),
            Err(e) => println!("{i}\t{ms}\t{q}\tPANIC {}", e.downcast_ref::<String>().cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default()),
        }
    }
}
