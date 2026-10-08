use std::panic::{catch_unwind, AssertUnwindSafe};
use turbojet::blade_speed::{build, BladeKnobs, Droop, Lever};
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

extern "system" { fn GetCurrentThread() -> isize; fn QueryThreadCycleTime(h: isize, c: *mut u64) -> i32; }
fn cyc() -> u64 { let mut c = 0u64; unsafe { QueryThreadCycleTime(GetCurrentThread(), &mut c); } c }

fn design(plpc: f64, phpc: f64, tt4: f64) -> TwoSpoolEngine {
    let (gc, cc, gt, ct) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64);
    let gas = Gas::new(GasSpec { gamma_c: gc, cp_c: cc, r_c: (gc - 1.0) / gc * cc,
        gamma_t: gt, cp_t: ct, r_t: (gt - 1.0) / gt * ct, hpr: 42.8e6, ..GasSpec::default() });
    let losses = TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
        eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true };
    build_two_spool_turbojet(gas, plpc, phpc, tt4, 50_000.0, losses)
}
fn maps(name: &str) -> (ComponentMap, ComponentMap) {
    let f = ComponentMap::flat();
    let (l, h) = match name {
        "flow/press" => (ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f }, ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f }),
        "press/flow" => (ComponentMap { a: 0.05, b: 0.20, sigma: 0.1, l: 1.0, ..f }, ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f }),
        "tilted" => (ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..f }, ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..f }),
        "steep" => (ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..f }, ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..f }),
        _ => (ComponentMap { sigma: 0.1, l: 0.7, ..f }, ComponentMap { sigma: 0.1, l: 1.0, ..f }),
    };
    (l.with_phi_surge(0.55), h.with_phi_surge(0.55))
}
fn tryit<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    catch_unwind(AssertUnwindSafe(f)).map_err(|e| e.downcast_ref::<String>().cloned()
        .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default())
}
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let (c0, t0) = (cyc(), std::time::Instant::now());
    let mut x = 1.0f64; while t0.elapsed().as_millis() < 300 { x = (x * 1.0000001).sin() + 1.0; }
    let ghz = (cyc() - c0) as f64 / t0.elapsed().as_secs_f64() / 1e9;
    println!("calibration {ghz:.2} GHz (x={x:.2})");
    let ms = |c: u64| c as f64 / (ghz * 1e6);
    let fl = FlightCondition::new(250.0, 50_000.0, 0.85);
    // 1. build alone
    let (ml, mh) = maps("flow/press");
    let c = cyc();
    for _ in 0..20 { let _ = build(&design(3.0, 6.0, 1500.0), fl, ml, mh, BladeKnobs::default(), BladeKnobs::default()); }
    println!("build (sizing only): {:.3} ms", ms(cyc() - c) / 20.0);
    // 2. one schedule point per lever/spool, per shape, per lambda
    for shape in ["flow/press", "press/flow", "tilted", "steep", "flat-eta"] {
        let (ml, mh) = maps(shape);
        for lam in [0.0, 1.0] {
            let k = BladeKnobs { lambda: lam, droop: Droop::WithBladeSpeed, ..BladeKnobs::default() };
            let z = build(&design(3.0, 6.0, 1500.0), fl, ml, mh, k, k).unwrap();
            let mut line = format!("{shape:>10} lam={lam}:");
            for (lever, ln) in [(Lever::Lumped, "lump"), (Lever::AllRows, "all"), (Lever::FrontRow, "front")] {
                for (sp, sn) in [(Spool::Lp, "L"), (Spool::Hp, "H")] {
                    for tt4 in [1500.0, 1300.0, 1100.0, 1000.0] {
                        let c = cyc();
                        let r = tryit(|| z.schedule(lever, sp, &[tt4])[0]);
                        let t = ms(cyc() - c);
                        let tag = match r { Ok(r) => if r.reached { "" } else { "u" }, Err(_) => "X" };
                        line += &format!(" {ln}{sn}{tt4:.0}{tag}:{t:.0}");
                    }
                }
            }
            println!("{line}");
        }
    }
    // 2b. other gases
    for (gn, g) in [("tpg", Gas::thermally_perfect()), ("reacting", Gas::reacting()), ("equil", Gas::reacting_equilibrium())] {
        let losses = TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
            eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true };
        let (ml, mh) = maps("flow/press");
        let c = cyc();
        let r = tryit(|| build(&build_two_spool_turbojet(g.clone(), 3.0, 6.0, 1500.0, 50_000.0, losses), fl, ml, mh, BladeKnobs::default(), BladeKnobs::default()));
        let tb = ms(cyc() - c);
        match r {
            Ok(Ok(z)) => {
                let c = cyc();
                let s = tryit(|| z.schedule(Lever::Lumped, Spool::Lp, &[1000.0])[0]);
                println!("gas {gn}: build {tb:.1} ms K {}/{} R {:.3}/{:.3}; lumped LP: {} {:.0} ms", z.lp.k, z.hp.k, z.lp.redline, z.hp.redline,
                    match s { Ok(r) => format!("n_L {:.4} reached {}", r.n_lp, r.reached), Err(e) => format!("PANIC {e}") }, ms(cyc() - c));
            }
            Ok(Err(e)) => println!("gas {gn}: sizing error {e}"),
            Err(e) => println!("gas {gn}: build PANIC {e}"),
        }
    }
    // 3. design knobs: pi split and Tt4_design
    for (pl, ph, tt) in [(2.0, 6.0, 1500.0), (4.0, 6.0, 1500.0), (3.0, 4.0, 1500.0), (3.0, 10.0, 1500.0), (3.0, 6.0, 1300.0), (3.0, 6.0, 1700.0), (1.5, 15.0, 1500.0)] {
        let (ml, mh) = maps("flow/press");
        let c = cyc();
        let r = tryit(|| build(&design(pl, ph, tt), fl, ml, mh, BladeKnobs::default(), BladeKnobs::default()));
        let tb = ms(cyc() - c);
        match r {
            Ok(Ok(z)) => {
                let c = cyc();
                let s = tryit(|| z.schedule(Lever::Lumped, Spool::Lp, &[tt * 2.0 / 3.0])[0]);
                println!("pi {pl}/{ph} Tt4d {tt}: build {tb:.2} ms K {}/{} R {:.3}/{:.3}; lumped LP @ {:.0}: {} {:.0} ms", z.lp.k, z.hp.k, z.lp.redline, z.hp.redline, tt*2.0/3.0,
                    match s { Ok(r) => format!("n_L {:.4} reached {}", r.n_lp, r.reached), Err(e) => format!("PANIC {e}") }, ms(cyc() - c));
            }
            Ok(Err(e)) => println!("pi {pl}/{ph} Tt4d {tt}: sizing error {e}"),
            Err(e) => println!("pi {pl}/{ph} Tt4d {tt}: build PANIC {e}"),
        }
    }
}
