// The requests where the repair turned a refusal into the efficiency check's failure: the operating point the second
// pass finds there (thrust, jet speed vs flight speed) — near-zero thrust, or below zero?
use turbojet::atmosphere::standard;
use turbojet::sandbox::{fly_solver, FlySettings, GasModel, MapShape};

struct Lcg(u64);
impl Lcg {
    fn u(&mut self) -> f64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); (self.0 >> 11) as f64 / (1u64 << 53) as f64 }
    fn r(&mut self, a: f64, b: f64) -> f64 { a + (b - a) * self.u() }
    fn i(&mut self, n: usize) -> usize { ((self.u() * n as f64) as usize).min(n - 1) }
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let want: Vec<usize> = std::fs::read_to_string(r"W:\temp\claude\rung33-dr\eff-new.txt").unwrap()
        .lines().map(|l| l.split('\t').next().unwrap().parse().unwrap()).collect();
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
        if !want.contains(&k) { continue }
        let st = fly_solver(&f);
        let cmap = f.map.map().with_phi_surge(f.phi_surge);
        let e = st.equilibrium(&f.flight(), f.tt4, Some(&cmap));
        println!("{k}: M0 {:.2} Tt4 {:.0}: nu {:.3} pi_c {:.3} thrust {:+.1} N  F/mdot {:+.2}  M9 {:.3} v0 {:.0} {:?}", f.m0, f.tt4, e.nu, e.pi_c, e.thrust, e.sp_thrust, e.m9, e.v0, e.branch);
    }
}
