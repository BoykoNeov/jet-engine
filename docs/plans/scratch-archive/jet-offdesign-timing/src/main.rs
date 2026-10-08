use std::panic::{catch_unwind, AssertUnwindSafe};
use turbojet::atmosphere::Ambient;
use turbojet::engine::{build_turbojet, FlightCondition};
use turbojet::map::ComponentMap;
use turbojet::matcher::OffDesignMatcher;
use turbojet::sandbox::{GasModel, NozzleMode, Settings};
use turbojet::spool::SpoolTransient;

extern "system" { fn GetCurrentThread() -> isize; fn QueryThreadCycleTime(h: isize, c: *mut u64) -> i32; }
fn cyc() -> u64 { let mut c = 0u64; unsafe { QueryThreadCycleTime(GetCurrentThread(), &mut c); } c }

fn tryit<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    catch_unwind(AssertUnwindSafe(f)).map_err(|e| {
        e.downcast_ref::<String>().cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default()
    })
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    // calibrate cycles -> ms on this machine
    let (c0, t0) = (cyc(), std::time::Instant::now());
    let mut x = 1.0f64; while t0.elapsed().as_millis() < 300 { x = (x * 1.0000001).sin() + 1.0; }
    let ghz = (cyc() - c0) as f64 / t0.elapsed().as_secs_f64() / 1e9;
    println!("calibration: {ghz:.2} GHz-equivalent (x={x:.3})");
    let ms = |c: u64| c as f64 / (ghz * 1e6);
    let mk = |z: f64, m0: f64| { let a = Ambient::from_altitude(z, 0.0); FlightCondition::new(a.t0, a.p0, m0) };
    let flights = vec![("design", Settings::defaults().flight()), ("SL M0.3", mk(0.0, 0.3)), ("11km M0.85", mk(11000.0, 0.85))];
    let tt4s = [700.0, 800.0, 900.0, 1000.0, 1100.0, 1200.0, 1300.0, 1400.0, 1500.0, 1600.0, 1700.0, 1800.0];
    let cm = ComponentMap::surge_flow().with_phi_surge(0.65);
    for g in [GasModel::ThermallyPerfect, GasModel::Equilibrium, GasModel::Reacting, GasModel::ForkB, GasModel::Perfect] {
        let s = Settings { gas: g, nozzle: NozzleMode::Convergent, ..Settings::defaults() };
        println!("\n===== gas {} =====", g.key());
        for (name, fl) in &flights {
            let mut line = format!("  spool+stall {name:>11}:");
            let mut tot = Vec::new();
            for &tt4 in &tt4s {
                let c = cyc();
                // FRESH solver per point, as the page would do
                let r = tryit(|| {
                    let st = SpoolTransient::new(build_turbojet(g.gas(), s.pi_c, s.tt4, s.p0, s.losses()), s.flight(), s.mdot, ComponentMap::surge_flow());
                    let eq = st.equilibrium(fl, tt4, Some(&cm));
                    let sm = if eq.branch.label() == "choked" { Some(st.surge_margin(fl, tt4, Some(&cm)).sm_n) } else { None };
                    (eq.nu, eq.branch.label(), sm)
                });
                let t = ms(cyc() - c);
                tot.push(t);
                match r {
                    Ok((nu, b, _)) => line += &format!(" {tt4:.0}:{}{t:.0}", &b[..1].to_uppercase()),
                    Err(_) => line += &format!(" {tt4:.0}:X{t:.0}"),
                }
                let _ = nu_unused();
            }
            tot.sort_by(|a, b| a.partial_cmp(b).unwrap());
            println!("{line}   [median {:.0} ms, max {:.0} ms]", tot[tot.len() / 2], tot[tot.len() - 1]);
        }
        // rung 31 alone, fresh per point, for reference
        let mut tot = Vec::new();
        for (_, fl) in &flights { for &tt4 in &tt4s {
            let c = cyc();
            let _ = tryit(|| OffDesignMatcher::new(build_turbojet(g.gas(), s.pi_c, s.tt4, s.p0, s.losses()), s.flight(), s.mdot).match_point(fl, tt4).pi_c);
            tot.push(ms(cyc() - c));
        }}
        tot.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!("  r31 alone (36 points): median {:.1} ms, max {:.0} ms", tot[tot.len() / 2], tot[tot.len() - 1]);
    }
}
fn nu_unused() {}
