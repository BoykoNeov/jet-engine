// HP-wall repair crash map: seeded random Controls requests through the page's own `sandbox::call`, a panic caught.
// The same source is built against the code before the repair (worktree of 070b729) and after; outputs compared.
use turbojet::atmosphere::standard;
use turbojet::sandbox::call;
use turbojet::sandbox_controls::{ControlsSettings, CustomerMode, GeneratorMode, LeverChoice, CONTROL_SHAPES};

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
    let mut g = Lcg(36);
    let n: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1200);
    std::panic::set_hook(Box::new(|_| {}));
    for i in 0..n {
        let mut s = ControlsSettings::defaults();
        // flat-lp over-weighted: it is the map the repair reaches
        s.shape = if g.u() < 0.5 { CONTROL_SHAPES.len() - 1 } else { g.i(CONTROL_SHAPES.len()) };
        s.from = g.r(550.0, 1500.0);
        s.to = g.r(550.0, 1500.0);
        s.ramp = (g.r(0.1, 1.0) / 0.02).round() * 0.02;
        s.settle = g.r(0.5, 2.0);
        s.rho = g.r(0.5, 3.0);
        s.lever = LeverChoice::ALL[g.i(4)];
        if g.u() < 0.25 { s.customer = CustomerMode::Share; s.customer_share = g.r(0.0, 0.08); }
        if g.u() < 0.25 { s.cool_vane = g.r(0.0, 0.08); s.cool_blade = g.r(0.0, 0.08); }
        if g.u() < 0.2 { s.generator = [GeneratorMode::Hp, GeneratorMode::Lp][g.i(2)]; }
        if g.u() < 0.5 {
            let h = g.r(0.0, 11000.0);
            let (t, p) = standard(h);
            s.t0 = t + g.r(-20.0, 30.0);
            s.p0 = p;
            s.m0 = g.r(0.2, 1.6);
        }
        let j = s.to_json().dump_compact();
        let q = format!(r#"{{"op":"controls","controls":{j}}}"#);
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
