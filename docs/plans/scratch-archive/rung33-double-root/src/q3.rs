// Rung 33's double root, round 2 (the advisor's four checks):
//  A. both roots' physics (pi_c, M9, specific thrust) at chosen cells;
//  B. any cell where the OLD march succeeded while the grid shows 2+ roots — which root did it return?
//  C. the candidate on the Python oracle's grid (250 K / 50 kPa, M0 0.3–2.0, Tt4 400–1500, cpg/tpg/eq-free) —
//     does any stored abort become a point (thrust > 0 at the candidate root)?
//  D. the fold edge: where the two roots merge, bisected in Tt4.
use turbojet::components::{ram_recovery, Nozzle};
use turbojet::engine::{build_turbojet, FlightCondition};
use turbojet::matcher::{OffDesignMatcher, SubsonicOp};
use turbojet::sandbox::{FlySettings, GasModel};

fn thrust(m: &OffDesignMatcher, fl: &FlightCondition, tt4: f64, op: &SubsonicOp) -> f64 {
    // Specific thrust at this root: the subsonic nozzle is fully expanded (p9 = p0), so no pressure term.
    let _ = tt4;
    let owned = m.working_gas(op.f, tt4, op.pt4);
    let wgas = owned.as_ref().unwrap_or(m.gas());
    let s5 = turbojet::gas::FlowState { tt: op.tt5, pt: op.pi_t * op.pt4, mdot: 1.0, far: op.f };
    let ex = Nozzle::convergent(fl.p0, m.pi_n).try_apply(&s5, wgas).unwrap();
    let (_, v0) = m.freestream_for(fl);
    (1.0 + op.f) * ex.v9 - v0
}

fn roots(m: &OffDesignMatcher, tt4: f64, tt2: f64, pt2: f64, p0: f64, n: usize) -> Vec<f64> {
    let mut out = Vec::new();
    let mut prev: Option<(f64, f64)> = None;
    for i in 0..=n {
        let y = 0.15 + (0.9995 - 0.15) * i as f64 / n as f64;
        match m.try_subsonic_operating(tt4, tt2, pt2, y, p0) {
            Ok(o) => {
                if let Some((py, pv)) = prev {
                    if pv * o.resid < 0.0 {
                        // bisect the grid interval
                        let (mut a, mut fa, mut b) = (py, pv, y);
                        for _ in 0..100 {
                            let c = 0.5 * (a + b);
                            let fc = m.try_subsonic_operating(tt4, tt2, pt2, c, p0).map(|o| o.resid).unwrap_or(f64::NAN);
                            if fa * fc <= 0.0 { b = c; } else { a = c; fa = fc; }
                            if b - a < 1e-13 { break; }
                        }
                        out.push(0.5 * (a + b));
                    }
                }
                prev = Some((y, o.resid));
            }
            Err(_) => prev = None,
        }
    }
    out
}

fn inlet(m: &OffDesignMatcher, fl: &FlightCondition) -> (f64, f64) {
    let pi_d = m.pi_d_max * ram_recovery(fl.m0);
    let (s0, _) = m.freestream_for(fl);
    (s0.tt, pi_d * s0.pt)
}

fn describe(m: &OffDesignMatcher, fl: &FlightCondition, tt4: f64, y: f64) -> String {
    let (tt2, pt2) = inlet(m, fl);
    let op = m.try_subsonic_operating(tt4, tt2, pt2, y, fl.p0).unwrap();
    let ft = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| thrust(m, fl, tt4, &op)))
        .map(|t| format!("{t:.1}")).unwrap_or_else(|_| "panic".into());
    format!("pi_t {y:.4} pi_c {:.3} tau_t {:.4} M9 {:.3} pt9/p0 {:.4} F/mdot {ft} m4 {:.4}", op.pi_c, op.tau_t, op.m9, op.pt9 / fl.p0, op.mdot4_ngv)
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    // The Fly-it design, perfect and thermally perfect.
    let fly: Vec<(GasModel, OffDesignMatcher)> = [GasModel::ThermallyPerfect, GasModel::Perfect].into_iter().map(|g| {
        let s = FlySettings { gas: g, ..FlySettings::defaults() };
        let d = s.capture();
        (g, OffDesignMatcher::new(build_turbojet(d.gas.gas(), d.pi_c, d.tt4, d.p0, d.losses()), d.flight(), d.mdot))
    }).collect();
    let hot = FlightCondition::new(318.15, 101_325.0, 0.30);
    let isa0 = FlightCondition::new(288.15, 101_325.0, 0.0);

    println!("== A. both roots' physics");
    for (g, m) in &fly {
        for (name, fl) in [("hot M.3", hot), ("ISA static", isa0)] {
            for tt4 in [650.0, 675.0, 700.0, 800.0, 900.0] {
                let (tt2, pt2) = inlet(m, &fl);
                for (i, y) in roots(m, tt4, tt2, pt2, fl.p0, 1600).into_iter().enumerate() {
                    println!("{g:?} {name} {tt4} root{}: {}", i + 1, describe(m, &fl, tt4, y));
                }
            }
        }
    }

    println!("== D. the fold edge (lowest Tt4 with a root), bisected");
    for (g, m) in &fly {
        for (name, fl) in [("hot M.3", hot), ("ISA static", isa0)] {
            let (tt2, pt2) = inlet(m, &fl);
            let has = |t: f64| !roots(m, t, tt2, pt2, fl.p0, 1600).is_empty();
            let (mut lo, mut hi) = (450.0, 900.0);
            if has(lo) || !has(hi) { println!("{g:?} {name}: no clean fold in [450, 900]"); continue; }
            while hi - lo > 0.5 { let c = 0.5 * (lo + hi); if has(c) { hi = c; } else { lo = c; } }
            let (tt2, pt2) = inlet(m, &fl);
            let r = roots(m, hi, tt2, pt2, fl.p0, 1600);
            println!("{g:?} {name}: fold between {lo:.1} and {hi:.1} K; at {hi:.1}: roots {:?}; low root {}", r,
                     describe(m, &fl, hi, r[0]));
        }
    }
}
