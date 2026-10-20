// Rung 104 probe P7 (advisor 2026-10-20, item 3):
// (A) a HAND-WRITTEN perfect-gas solve of the whole steady point under the ELLIPSE (closed-form turbines, the ellipse
//     formula for W45, a sonic A4, the subsonic nozzle, an outer bisection on x = pi_LPT with the HP ratio solved from
//     the A4 = A45 mass balance at each x) against the shipped rung-38 matcher and rung-39's on flat maps; with a count
//     of the nozzle residual's sign changes over x (uniqueness of the whole system on this engine).
// (B) the SHIPPED joint residual over the HP ratio at the converged outer state (pt4, f held), the Controls engine on
//     map "tilted", at P5's below-choke cells: sign changes on a 71-point grid over [0.2, 0.9], and where the root sits.
use turbojet::atmosphere::standard;
use turbojet::blade_speed::shape_maps;
use turbojet::components::{ram_recovery, try_choked_mfp};
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::turbine_law::TurbineLaw;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolCore, TwoSpoolLosses, TwoSpoolMapMatcher, TwoSpoolMatcher};
use turbojet::unchoked_lp::{try_lp_turbine, PI_TOP};

fn flight(dt: f64, h: f64, m0: f64) -> FlightCondition { let (t, p) = standard(h); FlightCondition::new(t + dt, p, m0) }
fn real() -> TwoSpoolLosses {
    TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92, eta_lpt: 0.90,
                     eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true }
}
fn cpg() -> Gas {
    Gas::new(GasSpec { gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9, gamma_t: 1.3, cp_t: 1239.0,
                       r_t: (1.3 - 1.0) / 1.3 * 1239.0, hpr: 42.8e6, ..GasSpec::default() })
}
fn bis(mut lo: f64, mut hi: f64, f: impl Fn(f64) -> f64) -> f64 {
    let flo = f(lo);
    for _ in 0..200 { let m = 0.5 * (lo + hi); if f(m) * flo > 0.0 { lo = m } else { hi = m } }
    0.5 * (lo + hi)
}
/// (pi_HPT, pi_LPT, pi_LPC, pi_HPC, sign changes)
fn hand(c: &TwoSpoolCore, fl: &FlightCondition, tt4: f64) -> Option<(f64, f64, f64, f64, usize)> {
    let (gc, cpc, gt, cpt, rt, hpr) = (1.4, 1004.0, 1.3, 1239.0, (1.3 - 1.0) / 1.3 * 1239.0, 42.8e6);
    let (ec, et) = ((gc - 1.0) / gc, (gt - 1.0) / gt);
    let tt2 = fl.t0 * (1.0 + 0.5 * (gc - 1.0) * fl.m0 * fl.m0);
    let pt2 = c.pi_d_max * ram_recovery(fl.m0) * fl.p0 * (tt2 / fl.t0).powf(1.0 / ec);
    let tt_out = |tin: f64, pi: f64, eta: f64| tin * (1.0 - eta * (1.0 - pi.powf(et)));
    let xd = c.lp_capacity.xd;
    let e = |x: f64| (1.0 - x * x).max(0.0).sqrt() / (1.0 - xd * xd).sqrt();
    let k = c.a4 / c.a45_pin();
    let mfp = (gt / rt).sqrt() * (2.0 / (gt + 1.0)).powf((gt + 1.0) / (2.0 * (gt - 1.0)));
    // HP ratio from A4 = A45 at this x: pi*sqrt(Tt4/Tt45) = k/e(x); None if the vane cannot pass A4's flow.
    let pih = |x: f64| -> Option<f64> {
        let tgt = k / e(x);
        if !(tgt < 1.0) { return None }
        Some(bis(1e-4, 1.0, |p| p * (tt4 / tt_out(tt4, p, c.eta_hpt)).sqrt() - tgt))
    };
    let point = |x: f64| -> Option<(f64, f64, f64, f64)> {
        let ph = pih(x)?;
        let tt45 = tt_out(tt4, ph, c.eta_hpt);
        let tt5 = tt_out(tt45, x, c.eta_lpt);
        let (mut f, mut pt4, mut out) = (c.f_design, 0.0, (0.0, 0.0));
        for _ in 0..400 {
            let tt25 = tt2 + c.eta_m * (1.0 + f) * cpt * (tt45 - tt5) / cpc;
            let tt3 = tt25 + c.eta_m * (1.0 + f) * cpt * (tt4 - tt45) / cpc;
            let pi_lpc = (1.0 + c.eta_lpc * (tt25 / tt2 - 1.0)).powf(1.0 / ec);
            let pi_hpc = (1.0 + c.eta_hpc * (tt3 / tt25 - 1.0)).powf(1.0 / ec);
            let pt4n = c.pi_b * pi_hpc * pi_lpc * pt2;
            let fnew = (cpt * tt4 - cpc * tt3) / (c.eta_b * hpr - cpt * tt4);
            let done = (fnew - f).abs() <= 1e-15 && (pt4n - pt4).abs() <= 1e-12 * pt4n;
            f = fnew;
            pt4 = pt4n;
            out = (pi_lpc, pi_hpc);
            if done { break }
        }
        let pt45 = ph * pt4;
        let supply = c.a45_pin() * mfp * e(x) * pt45 / tt45.sqrt();
        let pt9 = c.pi_n * x * pt45;
        let m9 = (((pt9 / fl.p0).powf(et) - 1.0) / (0.5 * (gt - 1.0))).max(0.0).sqrt();
        let t9 = tt5 / (1.0 + 0.5 * (gt - 1.0) * m9 * m9);
        let demand = c.a8 * fl.p0 / (rt * t9) * m9 * (gt * rt * t9).sqrt();
        Some(((supply - demand) / supply.max(1e-300), ph, out.0, out.1))
    };
    // the valid x range: e(x) > k  <=>  x < sqrt(1 - k^2 (1 - xd^2))
    let xmax = (1.0 - k * k * (1.0 - xd * xd)).sqrt().min(PI_TOP);
    let n = 400;
    let grid: Vec<(f64, f64)> = (0..n)
        .filter_map(|i| {
            let x = 0.3 + (xmax - 0.3) * (i as f64 + 0.5) / n as f64;
            point(x).map(|p| (x, p.0))
        })
        .collect();
    if std::env::var("P7_DUMP").is_ok() {
        println!("  xmax {xmax:.6}  k {k:.6}  xd {xd:.6}");
        for (i, (x, r)) in grid.iter().enumerate() {
            if i % 10 == 0 || grid.get(i + 1).map_or(false, |n| n.1 * r < 0.0) {
                let p = point(*x).unwrap();
                println!("  x {x:.5} resid {r:+.4e} piH {:.5}", p.1);
            }
        }
    }
    let changes = grid.windows(2).filter(|w| w[0].1 * w[1].1 < 0.0).count();
    let w = grid.windows(2).find(|w| w[0].1 * w[1].1 < 0.0)?;
    let (mut lo, mut hi) = (w[0].0, w[1].0);
    let rlo = w[0].1;
    for _ in 0..200 {
        let m = 0.5 * (lo + hi);
        if point(m)?.0 * rlo > 0.0 { lo = m } else { hi = m }
    }
    let x = 0.5 * (lo + hi);
    let p = point(x)?;
    Some((p.1, x, p.2, p.3, changes))
}

fn rel(pairs: &[(f64, f64)]) -> f64 { pairs.iter().map(|(a, b)| ((a - b) / b).abs()).fold(0.0, f64::max) }

fn main() {
    println!("== (A) hand-written perfect-gas solve, ELLIPSE, rung 38's engine (no map) and rung 39's on flat maps");
    let fd = FlightCondition::new(250.0, 50_000.0, 0.85);
    let m38 = TwoSpoolMatcher::new(build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, real()), fd, 1.0);
    let c = m38.core();
    c.below_choke.set(true);
    c.turbine_law.set(TurbineLaw::Ellipse);
    let m39 = match TwoSpoolMapMatcher::new(build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, real()), fd, 1.0,
                                            ComponentMap::flat(), ComponentMap::flat()) {
        TwoSpoolMapMatcher::Full(c) => c,
        _ => unreachable!(),
    };
    m39.base.below_choke.set(true);
    m39.base.turbine_law.set(TurbineLaw::Ellipse);
    if std::env::var("P7_DUMP").is_ok() {
        for tt4 in [500.0, 600.0] {
            println!("SL M0.30 {tt4}");
            hand(c, &flight(0.0, 0.0, 0.30), tt4);
        }
        return;
    }
    let mut worst: f64 = 0.0;
    for (fname, fl) in [("SL M0.30", flight(0.0, 0.0, 0.30)), ("SL+40 M0.85", flight(40.0, 0.0, 0.85)), ("design", fd)] {
        for i in 0..8 {
            let tt4 = 450.0 + 50.0 * i as f64;
            let r38 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| c.match_point(&fl, tt4)));
            let h = hand(c, &fl, tt4);
            match (r38, h) {
                (Ok(r), Some((ph, pl, plc, phc, n))) => {
                    let d = rel(&[(r.pi_hpt, ph), (r.pi_lpt, pl), (r.pi_lpc, plc), (r.pi_hpc, phc)]);
                    let d39 = match m39.try_match_point(&fl, tt4) {
                        Ok(q) => rel(&[(q.base.pi_hpt, ph), (q.base.pi_lpt, pl), (q.base.pi_hpc, phc)]),
                        Err(_) => f64::NAN,
                    };
                    if r.m9 < 1.0 - 1e-9 {
                        worst = worst.max(d).max(if d39.is_nan() { 0.0 } else { d39 });
                    }
                    println!("{fname:12} {tt4:5.0} M9 {:.3} piH {:.5} piL {:.5} | hand piH {ph:.5} piL {pl:.5} | rel diff 38 {d:.2e} 39 {d39:.2e} | sign changes {n}",
                             r.m9, r.pi_hpt, r.pi_lpt);
                }
                (Ok(r), None) => println!("{fname:12} {tt4:5.0} shipped M9 {:.3} piH {:.5} piL {:.5} | hand: NO ROOT", r.m9, r.pi_hpt, r.pi_lpt),
                (Err(_), Some((ph, pl, _, _, n))) => println!("{fname:12} {tt4:5.0} shipped PANIC | hand piH {ph:.5} piL {pl:.5} sign changes {n}"),
                (Err(_), None) => println!("{fname:12} {tt4:5.0} shipped PANIC | hand: NO ROOT"),
            }
        }
    }
    println!("worst unchoked relative difference: {worst:.2e}");

    println!("== (B) the shipped joint residual over pi_HPT at the converged outer state, Controls engine, map tilted");
    let bs = BladeSettings::defaults();
    let (ml, mh) = shape_maps("tilted").unwrap();
    let m = match TwoSpoolMapMatcher::new(bs.design(), bs.flight(), 1.0, ml, mh) {
        TwoSpoolMapMatcher::Full(c) => c,
        _ => unreachable!(),
    };
    m.base.below_choke.set(true);
    for law in [TurbineLaw::Ellipse, TurbineLaw::TwoRows, TurbineLaw::OneRow] {
        m.base.turbine_law.set(law);
        for (fname, fl) in [("SL+40 M0.85", flight(40.0, 0.0, 0.85)), ("SL M0.30", flight(0.0, 0.0, 0.30)), ("design", bs.flight())] {
            for i in 0..7 {
                let tt4 = 550.0 + 50.0 * i as f64;
                let Ok(r) = m.try_match_point(&fl, tt4) else { continue };
                if r.base.m9 >= 1.0 - 1e-9 { continue }
                let b = &m.base;
                let gas = b.gas();
                let s4 = r.base.station("4");
                let (f, pt4) = (s4.far, s4.pt);
                let a4m = b.a4 * try_choked_mfp(gas, tt4, f).unwrap();
                let resid = |ph: f64| -> f64 {
                    let (_, tt45) = b.try_tau_of(gas, tt4, f, ph, r.eta_hpt).unwrap();
                    let hp = a4m * (tt45 / tt4).sqrt() / ph;
                    match try_lp_turbine(b, gas, tt45, f, r.eta_lpt, ph * pt4, fl.p0) {
                        Ok((x, _, _)) => hp - b.w45(gas, x, tt45, f, r.eta_lpt).unwrap(),
                        Err(_) => hp,
                    }
                };
                let g: Vec<f64> = (0..71).map(|i| resid(0.2 + 0.01 * i as f64)).collect();
                let ch = g.windows(2).filter(|w| w[0] * w[1] < 0.0).count();
                println!("{law:?} {fname:12} {tt4:5.0} M9 {:.3} piH {:.5} sign changes {ch} (ends {:+.3e} {:+.3e}) resid at root {:+.2e}",
                         r.base.m9, r.base.pi_hpt, g[0], g[70], resid(r.base.pi_hpt));
            }
        }
    }
}
