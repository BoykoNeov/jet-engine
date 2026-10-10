use jet_combustor_timing::*;
use std::panic;
use std::sync::Mutex;
use std::time::Instant;

static MSG: Mutex<String> = Mutex::new(String::new());

fn try_<T>(f: impl FnOnce() -> T + panic::UnwindSafe) -> Result<T, String> {
    panic::catch_unwind(f).map_err(|_| MSG.lock().unwrap().clone())
}

const GASES: [&str; 5] = ["perfect", "therm_perf", "reacting", "fork_b", "equilibrium"];
const JS: [f64; 14] = [4.0, 6.0, 9.0, 12.0, 16.0, 20.0, 25.0, 36.0, 49.0, 64.0, 100.0, 144.0, 225.0, 400.0];
const PHIS: [f64; 17] = [0.6, 0.7, 0.8, 0.9, 0.95, 1.0, 1.05, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 2.0];

fn short(s: &str) -> String { s.chars().take(110).collect() }

fn main() {
    panic::set_hook(Box::new(|info| {
        let m = if let Some(s) = info.payload().downcast_ref::<&str>() { s.to_string() }
                else if let Some(s) = info.payload().downcast_ref::<String>() { s.clone() } else { "?".into() };
        *MSG.lock().unwrap() = m;
    }));
    let which = std::env::args().nth(1).unwrap_or_default();
    if which == "gas" {
        // Does the burner model accept a design run on each gas? t_mix/Tt4 and the 5 % gate.
        for (pi_c, tt4) in [(10.0, 1500.0), (4.0, 1000.0), (4.0, 1600.0), (20.0, 1300.0), (20.0, 1900.0),
                            (40.0, 1600.0), (40.0, 2100.0), (10.0, 2200.0), (30.0, 1200.0)] {
            println!("pi_c {pi_c} Tt4 {tt4}");
            for g in 0..5 {
                let inlet = match try_(|| burner_inlet(g, pi_c, tt4)) {
                    Ok(i) => i,
                    Err(m) => { println!("  {:12} design fails: {}", GASES[g], short(&m)); continue; }
                };
                let phi_ov = inlet.2 / turbojet::gas::f_stoich();
                let mut line = format!("  {:12} Tt3 {:7.1} far {:.5} phi_ov {:.3} |", GASES[g], inlet.0, inlet.2, phi_ov);
                for phi in [0.6, 1.0, 1.5, 2.0] {
                    match try_(|| zn(inlet, 0, phi, 25.0, 0)) {
                        Ok((ei, tm)) => line += &format!(" phi{phi}: EI {ei:8.3} Tmix/Tt4 {:.4} |", tm / tt4),
                        Err(m) => line += &format!(" phi{phi}: X[{}] |", short(&m)),
                    }
                }
                println!("{line}");
            }
        }
    }
    if which == "time" {
        let inlet = burner_inlet(4, 10.0, 1500.0);
        println!("inlet {inlet:?}");
        for level in [2usize, 1, 0] {
            for c in 0..12 {
                let phi = if c <= 1 { 1.0 } else { 1.5 };
                let t = Instant::now();
                let r = try_(|| zn(inlet, c, phi, 25.0, level));
                let ms = t.elapsed().as_secs_f64() * 1e3;
                match r {
                    Ok((ei, _)) => println!("L{level} {:18} point {ms:9.1} ms  EI {ei:.6}", CLOSURES[c]),
                    Err(m) => println!("L{level} {:18} point {ms:9.1} ms  X {}", CLOSURES[c], short(&m)),
                }
            }
        }
    }
    if which == "vals" { vals(); }
    if which == "why" { why(); }
    if which == "ref" { reference(); }
    if which == "nozzle" { nozzle(); }
    if which == "vals1" { vals1(); }
    if which == "shape" {
        // J sweep per closure per level: values, argmin, monotone? Then the phi bell (closure 0/1).
        let level_max: usize = std::env::args().nth(2).map(|s| s.parse().unwrap()).unwrap_or(1);
        let inlet = burner_inlet(4, 10.0, 1500.0);
        for c in 3..12 {
            for level in (level_max..=2).rev() {
                let t = Instant::now();
                let mut v = Vec::new();
                for j in JS {
                    v.push(try_(|| zn(inlet, c, 1.5, j, level)).map(|x| x.0).unwrap_or(f64::NAN));
                }
                let ms = t.elapsed().as_secs_f64() * 1e3;
                let am = (0..v.len()).min_by(|&a, &b| v[a].partial_cmp(&v[b]).unwrap_or(std::cmp::Ordering::Greater)).unwrap();
                println!("{:18} L{level} sweep {ms:8.0} ms argmin J={:5} vals {}", CLOSURES[c], JS[am],
                         v.iter().map(|x| format!("{x:.4}")).collect::<Vec<_>>().join(" "));
            }
        }
        for c in [0usize, 1, 2] {
            for level in (level_max..=2).rev() {
                let t = Instant::now();
                let v: Vec<f64> = PHIS.iter().map(|&p| try_(|| zn(inlet, c, p, 25.0, level)).map(|x| x.0).unwrap_or(f64::NAN)).collect();
                let ms = t.elapsed().as_secs_f64() * 1e3;
                println!("{:18} L{level} phi-bell {ms:8.0} ms vals {}", CLOSURES[c],
                         v.iter().map(|x| format!("{x:.4}")).collect::<Vec<_>>().join(" "));
            }
        }
    }
}

/// The browser-agreement grid, full precision (the same as vals.mjs prints).
pub fn vals() {
    let i = burner_inlet(4, 10.0, 1500.0);
    println!("inlet {:e} {:e} {:e} {:e}", i.0, i.1, i.2, i.3);
    for c in 0..2 { for p in PHIS { println!("c{c} phi{p} {:e}", zn(i, c, p, 25.0, 2).0); } }
    for c in 2..10 { for j in [9.0, 25.0, 100.0] { println!("c{c} J{j} {:e}", zn(i, c, 1.5, j, 2).0); } }
    for c in 10..12 { println!("c{c} J25 {:e}", zn(i, c, 1.5, 25.0, 2).0); }
}

pub fn why() {
    let i = burner_inlet(4, 10.0, 1500.0);
    for (c, j) in [(9usize, 6.0), (9, 36.0), (10, 6.0)] {
        for level in [2usize, 1] {
            let r = std::panic::catch_unwind(|| zn(i, c, 1.5, j, level));
            match r { Ok(x) => println!("c{c} J{j} L{level} ok {}", x.0),
                      Err(_) => println!("c{c} J{j} L{level} X {}", MSG.lock().unwrap()) }
        }
    }
}

/// The model-default (L0) J sweeps, the reference the coarse grids are judged against.
pub fn reference() {
    let i = burner_inlet(4, 10.0, 1500.0);
    for c in [3usize, 4, 5, 6, 8, 9, 7] {
        let t = Instant::now();
        let v: Vec<f64> = JS.iter().map(|&j| try_(|| zn(i, c, 1.5, j, 0)).map(|x| x.0).unwrap_or(f64::NAN)).collect();
        println!("{:18} L0 sweep {:8.0} ms vals {}", CLOSURES[c], t.elapsed().as_secs_f64() * 1e3,
                 v.iter().map(|x| format!("{x:.4}")).collect::<Vec<_>>().join(" "));
    }
    for c in [2usize] {
        let t = Instant::now();
        let v: Vec<f64> = PHIS.iter().map(|&p| try_(|| zn(i, c, p, 25.0, 0)).map(|x| x.0).unwrap_or(f64::NAN)).collect();
        println!("{:18} L0 phi-bell {:8.0} ms vals {}", CLOSURES[c], t.elapsed().as_secs_f64() * 1e3,
                 v.iter().map(|x| format!("{x:.4}")).collect::<Vec<_>>().join(" "));
    }
}

pub fn nozzle() {
    use turbojet::nox::*;
    use turbojet::sandbox::{GasModel, Settings};
    let s = Settings { gas: GasModel::Equilibrium, ..Settings::defaults() };
    let r = turbojet::engine::build_turbojet(s.gas.gas(), s.pi_c, s.tt4, s.p0, s.losses()).run(&s.flight(), s.mdot);
    let (s3, s4, s5) = (r.station("3"), r.station("4"), r.station("5"));
    let (tt9, pt9, p9) = (s5.tt, s.pi_n * s5.pt, s.p0);
    let eq = turbojet::gas::Gas::reacting_equilibrium();
    for phi in [1.0, 1.5] {
        let z = eq.zoned_nox(s4.far, s3.tt, s4.tt, s4.pt, phi, ZonedNoxOpts::default());
        let t = Instant::now();
        let nf = eq.nozzle_flow(s4.far, s4.tt, s4.pt, tt9, pt9, p9, Some(z.x_no_mix));
        println!("nozzle_flow phi{phi} {:.1} ms  T9 froz {:.2} eq {:.2} dV9 {:.4} max_a {:?}", t.elapsed().as_secs_f64()*1e3,
                 nf.t9_frozen, nf.t9_equilibrium, nf.dv9(), nf.max_a);
    }
    let mix = JetMixing { j: 225.0, c_e: 0.20, shape_n: 2.0, ..JetMixing::default() };
    let pq = PocketQuenchPdf { s: 0.0625, n_bell: 40, n_quad: 120, ..PocketQuenchPdf::default() };
    let t = Instant::now();
    let c = try_(move || eq.exhaust_no_clamp(s4.far, s3.tt, s4.tt, s4.pt, tt9, pt9, p9, 1.5, mix, pq,
        ExhaustClampOpts { tau: 3e-3, super_eq_o: false, quench_ngrid: 60, quench_nsteps: 400 }));
    match c { Ok(c) => println!("exhaust_no_clamp {:.0} ms a_mixed {:.3e} a_bulk {:.3e} a_pocket {:.3e}",
                                t.elapsed().as_secs_f64()*1e3, c.a_mixed_out, c.a_bulk_quench, c.a_pocket),
              Err(m) => println!("exhaust_no_clamp X {m}") }
}

/// Browser agreement at L1 over several designs (vals1.mjs prints the same lines).
pub fn vals1() {
    for (pi_c, tt4) in DESIGNS {
        let i = burner_inlet(4, pi_c, tt4);
        println!("d{pi_c}/{tt4} inlet {:e} {:e} {:e} {:e}", i.0, i.1, i.2, i.3);
        for c in [0usize, 1] { for p in [0.8, 1.0, 1.5] { println!("d{pi_c}/{tt4} c{c} phi{p} {:e}", zn(i, c, p, 25.0, 1).0); } }
        for c in [2usize, 3, 4, 5, 6, 8, 9] { println!("d{pi_c}/{tt4} c{c} J25 {:e}", zn(i, c, 1.5, 25.0, 1).0); }
    }
}
pub const DESIGNS: [(f64, f64); 4] = [(10.0, 1500.0), (4.0, 1000.0), (20.0, 1900.0), (40.0, 1600.0)];
