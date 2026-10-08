use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Mutex;
use turbojet::blade_speed::{BladeKnobs, Droop, Lever, SHAPES};
use turbojet::engine::FlightCondition;
use turbojet::two_spool::Spool;

static MSG: Mutex<String> = Mutex::new(String::new());
extern "system" { fn GetCurrentThread() -> isize; fn QueryThreadCycleTime(h: isize, c: *mut u64) -> i32; }
fn cyc() -> u64 { let mut c = 0u64; unsafe { QueryThreadCycleTime(GetCurrentThread(), &mut c); } c }

struct Rng(u64);
impl Rng {
    fn u(&mut self) -> f64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; (self.0 >> 11) as f64 / (1u64 << 53) as f64 }
    fn r(&mut self, a: f64, b: f64) -> f64 { a + (b - a) * self.u() }
    fn i(&mut self, n: usize) -> usize { ((self.u() * n as f64) as usize).min(n - 1) }
}
fn knobs(g: &mut Rng) -> BladeKnobs {
    let lam = match g.i(10) { 0..=2 => 0.0, 3..=4 => 1.0, _ => g.r(0.0, 1.0) };
    BladeKnobs { h: g.r(0.3, 0.9), m_rel_lim: g.r(0.9, 1.7), sigma_over_rho: (g.r(3e4f64.ln(), 4.0e5f64.ln())).exp(),
        overspeed: g.r(1.0, 1.5), phi_d: g.r(0.3, 1.0), lambda: lam, droop: if g.i(2) == 0 { Droop::WithBladeSpeed } else { Droop::WithRowWork } }
}
fn main() {
    std::panic::set_hook(Box::new(|i| { *MSG.lock().unwrap() = format!("{}", i).lines().skip(1).collect::<Vec<_>>().join(" "); }));
    let args: Vec<String> = std::env::args().collect();
    let gasname = args[1].clone(); let n: usize = args[2].parse().unwrap(); let seed: u64 = args[3].parse().unwrap();
    let (c0, t0) = (cyc(), std::time::Instant::now());
    let mut x = 1.0f64; while t0.elapsed().as_millis() < 300 { x = (x * 1.0000001).sin() + 1.0; }
    let ghz = (cyc() - c0) as f64 / t0.elapsed().as_secs_f64() / 1e9; let _ = x;
    let ms = |c: u64| c as f64 / (ghz * 1e6);
    let fl = FlightCondition::new(250.0, 50_000.0, 0.85);
    let mut g = Rng(seed);
    let mut classes: BTreeMap<String, usize> = BTreeMap::new();
    let mut times: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut fails = Vec::new();
    let (mut unreached, mut ok) = (0, 0);
    for k in 0..n {
        let (pl, ph, tt4d) = (g.r(1.2, 8.0), g.r(1.5, 20.0), g.r(1000.0, 2000.0));
        let shape = SHAPES[g.i(5)];
        let kl = knobs(&mut g);
        let kh = if g.i(2) == 0 { kl } else { knobs(&mut g) };
        let (lever, ln) = [(Lever::Lumped, "lumped"), (Lever::AllRows, "all"), (Lever::FrontRow, "front")][g.i(3)];
        let (spool, sn) = [(Spool::Lp, "LP"), (Spool::Hp, "HP")][g.i(2)];
        let frac = g.r(0.6, 1.0);
        let tt4 = tt4d * frac;
        let kj = |k: &BladeKnobs| format!(r#"{{"h":{},"m_rel_lim":{},"sigma_over_rho":{},"overspeed":{},"phi_d":{},"lambda":{},"droop":"{}"}}"#,
            k.h, k.m_rel_lim, k.sigma_over_rho, k.overspeed, k.phi_d, k.lambda, if k.droop == Droop::WithBladeSpeed { "blade_speed" } else { "row_work" });
        let gk = if gasname == "tpg" { "thermally_perfect" } else { "perfect" };
        let lk = match lever { Lever::Lumped => "lumped", Lever::AllRows => "all_rows", Lever::FrontRow => "front_row" };
        let sk = if spool == Spool::Lp { "lp" } else { "hp" };
        let blades = format!(r#"{{"gas":"{gk}","pi_lpc":{pl},"pi_hpc":{ph},"Tt4":{tt4d},"shape":"{shape}","lever":"{lk}","spool":"{sk}","lp":{},"hp":{}}}"#, kj(&kl), kj(&kh));
        let desc = format!("#{k} {blades} Tt4 {tt4}");
        let c = cyc();
        let r = catch_unwind(AssertUnwindSafe(|| {
            let sz = turbojet::sandbox::call(&format!(r#"{{"op":"blade_size","blades":{blades}}}"#));
            if sz.starts_with(r#"{"ok":0"#) || args.get(4).map(|a| a == "size").unwrap_or(false) { return sz; }
            turbojet::sandbox::call(&format!(r#"{{"op":"blade_lever","Tt4":{tt4},"blades":{blades}}}"#))
        }));
        let t = ms(cyc() - c);
        let _ = &fl;
        match r {
            Ok(j) if j.starts_with(r#"{"ok":1"#) => { ok += 1; let un = j.contains(r#""reached":0"#); if un { unreached += 1; } times.entry(ln).or_default().push(t); times.entry(if un { "UNREACHED" } else { "REACHED" }).or_default().push(t); }
            Ok(j) => { let key: String = j.chars().filter(|c| !c.is_ascii_digit()).skip(20).take(110).collect(); *classes.entry(format!("REFUSED: {key}")).or_default() += 1; if !j.contains("unchokes") && !j.contains("chokes at design") { fails.push(format!("{desc}
   -> {j}")); } }
            Err(_) => {
                let m = MSG.lock().unwrap().clone();
                let key: String = m.chars().filter(|c| !c.is_ascii_digit()).take(90).collect();
                *classes.entry(format!("PANIC: {key}")).or_default() += 1;
                fails.push(format!("{desc}
   -> {m}"));
            }
        }
    }
    println!("gas {gasname} n {n}: ran {ok} (unreached {unreached})");
    for (k, v) in &classes { println!("  {v:5}  {k}"); }
    for (k, v) in times.iter_mut() { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); println!("  time {k}: n {} median {:.1} p90 {:.1} max {:.1} ms", v.len(), v[v.len()/2], v[v.len()*9/10], v[v.len()-1]); }
    std::fs::write(format!("W:/temp/claude/slice2/crash-{gasname}-{seed}.txt"), fails.join("\n")).unwrap();
}
