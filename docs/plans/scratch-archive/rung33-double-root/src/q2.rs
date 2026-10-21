// Rung 33's double root: the candidate fallback (march from the low end's first evaluable point UP in 0.02 steps to the
// first sign change, then rung 33's own bisection) against rung 34's flat-map equilibrium (the reduce rung 34 claims),
// and the root count on a 400-point grid. One shaft, the Fly-it design, several flights.
use turbojet::components::ram_recovery;
use turbojet::engine::{build_turbojet, FlightCondition};
use turbojet::map::ComponentMap;
use turbojet::matcher::OffDesignMatcher;
use turbojet::sandbox::{FlySettings, GasModel};
use turbojet::spool::SpoolTransient;

fn msg(e: Box<dyn std::any::Any + Send>) -> String {
    e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default()
}

/// The march as shipped, then the candidate where it does not straddle. Returns (old ok?, candidate root, roots on grid).
fn candidate(m: &OffDesignMatcher, tt4: f64, tt2: f64, pt2: f64, p0: f64) -> (bool, Option<f64>, Vec<f64>) {
    let r = |y: f64| m.try_subsonic_operating(tt4, tt2, pt2, y, p0).map(|o| o.resid);
    let (mut lo, mut rlo) = (f64::NAN, f64::NAN);
    let mut pt = 0.15;
    while pt < 0.95 {
        if let Ok(v) = r(pt) { lo = pt; rlo = v; break; }
        pt += 0.02;
    }
    let (mut hi, mut rhi) = (f64::NAN, f64::NAN);
    let mut pt = 0.9995;
    while !lo.is_nan() && pt > lo {
        if let Ok(v) = r(pt) { hi = pt; rhi = v; break; }
        pt -= 0.02;
    }
    let old_ok = !lo.is_nan() && !hi.is_nan() && rlo * rhi < 0.0;
    let mut grid = Vec::new();
    let mut prev: Option<(f64, f64)> = None;
    for i in 0..=800 {
        let y = 0.15 + (0.9995 - 0.15) * i as f64 / 800.0;
        match r(y) {
            Ok(v) => { if let Some((py, pv)) = prev { if pv * v < 0.0 { grid.push(0.5 * (py + y)); } } prev = Some((y, v)); }
            Err(_) => prev = None,
        }
    }
    if old_ok || lo.is_nan() || hi.is_nan() { return (old_ok, None, grid); }
    // Candidate: up from lo in 0.02 steps to the first sign change.
    let (mut a, mut ra) = (lo, rlo);
    let mut b = lo + 0.02;
    let mut found = None;
    while b < hi {
        match r(b) {
            Ok(rb) if ra * rb < 0.0 => { found = Some((a, ra, b)); break; }
            Ok(rb) => { a = b; ra = rb; }
            Err(_) => break,
        }
        b += 0.02;
    }
    let Some((mut lo, mut rlo, mut hi)) = found else { return (false, None, grid) };
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        let rm = r(mid).unwrap();
        if rlo * rm <= 0.0 { hi = mid; } else { lo = mid; rlo = rm; }
        if hi - lo <= 1e-13 { break; }
    }
    (false, Some(0.5 * (lo + hi)), grid)
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let flights = [
        ("hot SL M.30", 318.15, 101_325.0, 0.30),
        ("hot SL M.10", 318.15, 101_325.0, 0.10),
        ("ISA SL M.00", 288.15, 101_325.0, 0.0),
        ("ISA SL M.30", 288.15, 101_325.0, 0.30),
        ("cold SL M.3", 250.0, 101_325.0, 0.30),
        ("design", 250.0, 50_000.0, 0.85),
    ];
    for gas in [GasModel::ThermallyPerfect, GasModel::Perfect] {
        let s = FlySettings { gas, ..FlySettings::defaults() };
        let d = s.capture();
        let mk = || build_turbojet(d.gas.gas(), d.pi_c, d.tt4, d.p0, d.losses());
        let m = OffDesignMatcher::new(mk(), d.flight(), d.mdot);
        let st = SpoolTransient::new(mk(), d.flight(), d.mdot, ComponentMap::flat());
        for (name, t0, p0, m0) in flights {
            let fl = FlightCondition::new(t0, p0, m0);
            let pi_d = m.pi_d_max * ram_recovery(fl.m0);
            let (s0, _) = m.freestream_for(&fl);
            let (tt2, pt2) = (s0.tt, pi_d * s0.pt);
            for k in 0..=18 {
                let tt4 = 550.0 + 25.0 * k as f64;
                let old = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| m.match_point(&fl, tt4)));
                let old_s = match &old {
                    Ok(od) => format!("{:?} pi_t {:.6} pi_c {:.5}", od.branch, od.pi_t, od.pi_c),
                    Err(_) => "FAIL".into(),
                };
                let (march_ok, cand, grid) = candidate(&m, tt4, tt2, pt2, fl.p0);
                let cand_s = match cand {
                    Some(y) => {
                        let op = m.try_subsonic_operating(tt4, tt2, pt2, y, fl.p0).unwrap();
                        format!("CAND pi_t {:.6} pi_c {:.5} m9 {:.3}", y, op.pi_c, op.m9)
                    }
                    None => String::new(),
                };
                let r34 = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| st.equilibrium(&fl, tt4, None))) {
                    Ok(i) => format!("{:?} pi_t {:.6} pi_c {:.5} nu {:.4}", i.branch, i.pi_t, i.pi_c, i.nu),
                    Err(e) => format!("FAIL {}", msg(e).chars().take(60).collect::<String>()),
                };
                let g: Vec<String> = grid.iter().map(|y| format!("{y:.3}")).collect();
                println!("{gas:?} {name} Tt4 {tt4:>5}: R33 {old_s} | march {} {cand_s} | roots {:?} | R34flat {r34}",
                         if march_ok { "ok" } else { "NO" }, g);
            }
        }
    }
}
