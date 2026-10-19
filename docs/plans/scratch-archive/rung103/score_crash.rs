// Rung 103 anchor § 5 crash map: seeded random Blades-lever and Cooling requests at LOW throttles, each replied through
// the page's own `sandbox::call`, a panic caught and printed. The same binary source is built against the code before
// rung 103 (a worktree of 8bf63b9) and against rung 103; the two outputs are compared line by line.
use turbojet::blade_speed::{Lever, SHAPES};
use turbojet::sandbox::call;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_cooling::CoolingSettings;
use turbojet::two_spool::Spool;

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
    let mut g = Lcg(103);
    let mut reqs = Vec::new();
    for _ in 0..600 {
        let d = BladeSettings::defaults();
        let s = BladeSettings {
            pi_lpc: g.r(2.0, 5.0), pi_hpc: g.r(4.0, 12.0), tt4: g.r(1150.0, 1700.0), shape: g.i(SHAPES.len()),
            lever: [Lever::Lumped, Lever::AllRows, Lever::FrontRow][g.i(3)], spool: [Spool::Lp, Spool::Hp][g.i(2)], ..d
        };
        let mut j = s.to_json().dump_compact();
        if g.u() < 0.2 { j = j.replacen("{", r#"{"gas":"thermally_perfect","#, 1).replacen(r#""gas":"perfect","#, "", 1) }
        let t = s.tt4 * g.r(0.35, 0.75);
        reqs.push(format!(r#"{{"op":"blade_lever","Tt4":{t:.2},"blades":{j}}}"#));
    }
    for k in 0..600 {
        let d = CoolingSettings::defaults();
        let s = CoolingSettings { vane: g.r(0.0, 0.2), blade: g.r(0.0, 0.2), shape: g.i(4), tech: g.i(3),
                                  year: g.r(1960.0, 2020.0), ..d };
        let j = s.to_json().dump_compact();
        if k % 3 == 0 {
            reqs.push(format!(r#"{{"op":"cooling_knob","cooling":{j}}}"#));
        } else {
            reqs.push(format!(r#"{{"op":"cooling_point","cooling":{j},"Tt4":{:.1}}}"#, g.r(550.0, 1300.0)));
        }
    }
    std::panic::set_hook(Box::new(|_| {}));
    for (i, q) in reqs.iter().enumerate() {
        let r = std::panic::catch_unwind(|| call(q));
        match r {
            Ok(a) => println!("{i}\t{q}\t{a}"),
            Err(e) => println!("{i}\t{q}\tPANIC {}", e.downcast_ref::<String>().cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default()),
        }
    }
}
