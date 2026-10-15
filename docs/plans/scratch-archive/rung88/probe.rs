// Rung 88 pre-anchor probes (scratch, not in the repo). Rig = rung 87's (= rung 86's).
use std::time::Instant;
use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{FuelPoint, PointExtra};
use turbojet::gas::{Gas, GasSpec};
use turbojet::lagged_bleed::build_lagged_bleed;
use turbojet::limited_bleed::{build_limited_bleed, BleedLimiter};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorArm, StatorLeg, StatorSchedule};
use turbojet::three_levers::march;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::valve_reference::{read, sibling};

const REAL: TwoSpoolLosses = TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true };
fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }
fn cpg() -> Gas { Gas::new(GasSpec { gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
    gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0, hpr: 42.8e6, ..GasSpec::default() }) }
fn lp_map() -> ComponentMap { ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }.with_phi_surge(0.55) }
fn hp_map() -> ComponentMap { ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }.with_phi_surge(0.55) }
fn tilt_map() -> ComponentMap { ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..ComponentMap::flat() }.with_phi_surge(0.55) }
fn design() -> TwoSpoolEngine { build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL) }
fn bare65(lp: ComponentMap, hp: ComponentMap) -> ScheduledStatorCore {
    match build_lagged_bleed(design(), flight(), 1.0, Some(lp), Some(hp), 1.0, &LeverArm::default()) {
        ScheduledStatorTransient::Full(c) => c, _ => unreachable!() } }
fn bare64(lp: ComponentMap, hp: ComponentMap) -> ScheduledStatorCore {
    match build_limited_bleed(design(), flight(), 1.0, Some(lp), Some(hp), 1.0, &LeverArm::default()) {
        ScheduledStatorTransient::Full(c) => c, _ => unreachable!() } }

const BMAX: f64 = 0.10;
fn stat() -> LeverArm { LeverArm::stator(StatorArm::scheduled_lp(StatorSchedule::new(0.20, 0.65))) }
fn phi_of(sm: f64) -> f64 { 0.55 * (1.0 + sm) }
fn valve(sm: f64, inc: bool, tau: Option<f64>) -> BleedLimiter {
    let v = BleedLimiter::with_tau(phi_of(sm), BMAX, tau);
    if inc { v.on_incidence() } else { v }
}

fn key(p: &FuelPoint) -> Vec<u64> {
    let mut k = vec![p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.phi_lp.to_bits(), p.phi_hp.to_bits(),
                     p.mf.to_bits(), p.tt4.to_bits(), p.sp_thrust.to_bits(), p.mdot_air.to_bits()];
    if let PointExtra::Valve { b, b_cmd } = p.extra { k.push(b.to_bits()); k.push(b_cmd.to_bits()); }
    k
}
fn same(a: &[FuelPoint], b: &[FuelPoint]) -> String {
    if a.len() != b.len() { return format!("LEN {} vs {}", a.len(), b.len()); }
    let d = a.iter().zip(b).filter(|(x, y)| key(x) != key(y)).count();
    format!("n={} differing points={d}", a.len())
}

fn main() {
    let which: String = std::env::args().nth(1).unwrap_or_default();
    let ds: f64 = std::env::args().nth(2).map(|x| x.parse().unwrap()).unwrap_or(0.005);
    let fl = flight();
    let shapes = [("shaped", lp_map(), hp_map()), ("tilted", tilt_map(), tilt_map())];
    // ---------------- P0: reduces and entry points
    if which == "p0" {
        let m = bare65(lp_map(), hp_map());
        let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r: 0.5, s_settle: 1.2, ds };
        let t0 = Instant::now();
        let a = march(&sibling(&m, None, Some(valve(0.42, false, Some(0.05)))), &fl, &ramp, None).0;
        println!("one lagged march: {:.1}s", t0.elapsed().as_secs_f64());
        let b = march(&sibling(&m, None, Some(valve(0.42, true, Some(0.05)))), &fl, &ramp, None).0;
        println!("P0a S0 lagged phi vs lagged inc (tau .05): {}", same(&a, &b));
        let ms = sibling(&m, Some(&stat()), Some(valve(0.42, false, Some(0.05))));
        let c = march(&ms, &fl, &ramp, None).0;
        let d = ms.stator_march(&fl, &ramp, None, &StatorLeg::default()).0;
        println!("P0b S1 lagged phi: three_levers::march vs stator_march: {}", same(&c, &d));
        let mi = sibling(&m, Some(&stat()), Some(valve(0.42, true, Some(0.05))));
        let e = march(&mi, &fl, &ramp, None).0;
        let f = mi.stator_march(&fl, &ramp, None, &StatorLeg::default()).0;
        println!("P0b' S1 lagged inc: march vs stator_march: {}", same(&e, &f));
        let m64 = bare64(lp_map(), hp_map());
        for inc in [false, true] {
            let g = march(&sibling(&m, Some(&stat()), Some(valve(0.42, inc, None))), &fl, &ramp, None).0;
            let h = march(&sibling(&m64, Some(&stat()), Some(valve(0.42, inc, None))), &fl, &ramp, None).0;
            println!("P0c S1 instantaneous inc={inc}: rung-65 builder vs rung-64 builder: {}", same(&g, &h));
        }
        let l = valve(0.42, true, None).lagged(0.05);
        println!("P0d .on_incidence().lagged(.05) == with_tau(.., .05).on_incidence(): {}", l == valve(0.42, true, Some(0.05)));
        // b(0) of a lagged march is the instantaneous valve's re-solved b at s=0
        let r0 = read(&mi, &fl, &e);
        let inst = sibling(&m, Some(&stat()), Some(valve(0.42, true, None)));
        let ti = march(&inst, &fl, &ramp, None).0;
        println!("P0e b(0) lagged inc = {:?}, instantaneous re-solved b(0) = {:.6e}, lagged npts={} inst npts={}",
                 e[0].extra, inst.b_at_point(&fl, &ti[0]), r0.npts, ti.len());
    }
    // ---------------- P1: the instantaneous valves: start position, opening/closing slopes
    if which == "p1" {
        for (shape, lp, hp) in shapes.clone() {
            let m = bare65(lp, hp);
            let (tt2, _, _) = m.fuel.inner.inlet(&fl);
            let t_c = m.design_map(Spool::Lp).tan_beta1_crit();
            for r in [0.25, 0.5, 1.0] {
                let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds };
                for sm in [0.30, 0.34, 0.38, 0.42, 0.46, 0.50, 0.54, 0.58] {
                    for (face, st) in [("S0", None), ("S1", Some(stat()))] {
                        for inc in [false, true] {
                            if face == "S0" && inc { continue; } // S0: inc == phi (P0a)
                            let mm = sibling(&m, st.as_ref(), Some(valve(sm, inc, None)));
                            let (traj, _) = march(&mm, &fl, &ramp, None);
                            let rr = read(&mm, &fl, &traj);
                            let b: Vec<f64> = traj.iter().map(|p| mm.b_at_point(&fl, p)).collect();
                            let v: Vec<f64> = traj.iter().map(|p| mm.v_of(Spool::Lp, p.nu_lp, p.nu_hp, Some(tt2))).collect();
                            let n = b.len();
                            let first_open = (0..n).find(|&i| b[i] > 0.0).map(|i| traj[i].s);
                            let (mut ipk, mut bpk) = (0, 0.0f64);
                            for i in 0..n { if b[i] > bpk { bpk = b[i]; ipk = i; } }
                            let close = (ipk..n).find(|&i| b[i] <= 0.0).map(|i| traj[i].s);
                            let (mut up, mut dn) = (0.0f64, 0.0f64);
                            for i in 1..n { let g = (b[i] - b[i - 1]) / (traj[i].s - traj[i - 1].s); up = up.max(g); dn = dn.min(g); }
                            // slope at the M_i minimum
                            let mut jm = 0; let mut mim = f64::INFINITY;
                            for i in 0..n { let x = t_c - 1.0 / traj[i].phi_lp + v[i]; if x < mim { mim = x; jm = i; } }
                            let gm = if jm > 0 && jm + 1 < n { (b[jm + 1] - b[jm - 1]) / (traj[jm + 1].s - traj[jm - 1].s) } else { f64::NAN };
                            println!("P1 {shape} r={r} sm={sm:.2} {face} {} n={n} b0={:.4e} open@{:?} pk={bpk:.4}@{:.3} close@{:?} up={up:.4} dn={dn:.4} Mi={mim:.5}@{:.3} db/ds@Mi={gm:.4} v0={:.4} vMi={:.4} bint={:.4e} Fint={:.3} sat={}",
                                     if inc { "inc" } else { "phi" }, b[0], first_open, traj[ipk].s, close, traj[jm].s, v[0], v[jm], rr.b_int, rr.thrust_int, rr.saturated);
                        }
                    }
                }
            }
        }
    }
    // ---------------- P2: the lagged phi valve beside the stators (it runs today)
    if which == "p2" {
        for (shape, lp, hp) in shapes.clone() {
            let m = bare65(lp, hp);
            for (r, sm) in [(0.5, 0.42), (0.5, 0.46), (1.0, 0.46)] {
                let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds };
                for (face, st) in [("S0", None), ("S1", Some(stat()))] {
                    for tau in [None, Some(0.02), Some(0.05), Some(0.1), Some(0.2)] {
                        let t0 = Instant::now();
                        let mm = sibling(&m, st.as_ref(), Some(valve(sm, false, tau)));
                        let (traj, _) = march(&mm, &fl, &ramp, None);
                        let rr = read(&mm, &fl, &traj);
                        println!("P2 {shape} r={r} sm={sm} {face} tau={tau:?} n={} minphi={:.6} Mi={:.6}@{:.3} bint={:.4e} bpk={:.4} Fint={:.3} sat={} [{:.1}s]",
                                 rr.npts, rr.min_phi, rr.m_i, rr.s_at_m_i, rr.b_int, rr.b_peak, rr.thrust_int, rr.saturated, t0.elapsed().as_secs_f64());
                    }
                }
            }
        }
    }

    // ---------------- P3: the 21 tabulated cells of rung 87 (20 scored + 0 band): starts, binding slopes, the phi lag identity
    if which == "p3" {
        let cells: [(&str, f64, f64, f64); 21] = [
            ("shaped",0.25,0.34,0.2609),("shaped",0.25,0.38,0.3038),("shaped",0.25,0.42,0.3464),("shaped",0.5,0.38,0.2876),
            ("shaped",0.5,0.42,0.3447),("shaped",0.5,0.46,0.3896),("shaped",0.5,0.50,0.4344),("shaped",1.0,0.42,0.3223),
            ("shaped",1.0,0.46,0.3753),("shaped",1.0,0.50,0.4265),("shaped",1.0,0.54,0.4798),("tilted",0.25,0.34,0.2574),
            ("tilted",0.25,0.38,0.2993),("tilted",0.25,0.42,0.3424),("tilted",0.5,0.42,0.3391),("tilted",0.5,0.46,0.3848),
            ("tilted",0.5,0.50,0.4289),("tilted",1.0,0.42,0.3126),("tilted",1.0,0.46,0.3683),("tilted",1.0,0.50,0.4191),("tilted",1.0,0.54,0.4728)];
        let ms = bare65(lp_map(), hp_map());
        let mt = bare65(tilt_map(), tilt_map());
        for (shape, r, sm, smx) in cells {
            let m = if shape == "shaped" { &ms } else { &mt };
            let (tt2, _, _) = m.fuel.inner.inlet(&fl);
            let t_c = m.design_map(Spool::Lp).tan_beta1_crit();
            let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds };
            for (name, smv, inc) in [("inc", sm, true), ("phi*", smx, false)] {
                let mm = sibling(m, Some(&stat()), Some(valve(smv, inc, None)));
                let (traj, _) = march(&mm, &fl, &ramp, None);
                let b: Vec<f64> = traj.iter().map(|p| mm.b_at_point(&fl, p)).collect();
                let v: Vec<f64> = traj.iter().map(|p| mm.v_of(Spool::Lp, p.nu_lp, p.nu_hp, Some(tt2))).collect();
                let n = b.len();
                let mut jm = 0; let mut mim = f64::INFINITY;
                for i in 0..n { let x = t_c - 1.0 / traj[i].phi_lp + v[i]; if x < mim { mim = x; jm = i; } }
                let gm = if jm > 0 && jm + 1 < n { (b[jm + 1] - b[jm - 1]) / (traj[jm + 1].s - traj[jm - 1].s) } else { f64::NAN };
                let rr = read(&mm, &fl, &traj);
                print!("P3 {shape} r={r} sm={sm:.2} {name}(sm {smv:.4}) bpk={:.4e} b0={:.4e} Mi={mim:.6}@{:.3} bdot@Mi={gm:.4} b@Mi={:.4} ride/sat={}/{} bint={:.4e} |", rr.b_peak, b[0], traj[jm].s, b[jm], rr.riding, rr.saturated, rr.b_int);
                if !inc {
                    for tau in [0.01, 0.02, 0.05, 0.1, 0.2] {
                        let ml = sibling(m, Some(&stat()), Some(valve(smv, false, Some(tau))));
                        let (tl, _) = march(&ml, &fl, &ramp, None);
                        let rl = read(&ml, &fl, &tl);
                        let (mut ib, mut ic) = (0.0, 0.0);
                        let bc: Vec<(f64, f64)> = tl.iter().map(|p| match p.extra { PointExtra::Valve { b, b_cmd } => (b, b_cmd), _ => panic!() }).collect();
                        for i in 1..tl.len() { let h = tl[i].s - tl[i-1].s; ib += 0.5*h*(bc[i].0+bc[i-1].0); ic += 0.5*h*(bc[i].1+bc[i-1].1); }
                        let ident = tau * (bc[0].0 - bc[bc.len()-1].0);
                        print!(" t{tau}: n={} Mi={:.6}@{:.3} dbint={:.3e} tau*b0={:.3e} (int b - int bcmd)={:.3e} vs tau(b0-bend)={:.3e} drift={:.3e} |",
                               rl.npts, rl.m_i, rl.s_at_m_i, rl.b_int - rr.b_int, tau*b[0], ib-ic, ident, ic - rr.b_int);
                    }
                }
                println!();
            }
        }
    }

    // ---------------- P4: rung 65's own case; clamp/b_end on the phi* lagged runs; M_i(phi_lim) of the lagged phi valve
    if which == "p4" {
        use turbojet::lagged_bleed::Census65;
        let m = bare65(lp_map(), hp_map());
        let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r: 0.5, s_settle: 1.2, ds };
        let inst = sibling(&m, None, Some(BleedLimiter::new(0.80, BMAX)));
        let ti = march(&inst, &fl, &ramp, None).0;
        let ri = read(&inst, &fl, &ti);
        let b0 = inst.b_at_point(&fl, &ti[0]);
        println!("P4a rung65 case inst: n={} bint={:.6} b0={b0:.6} minphi={:.9}", ri.npts, ri.b_int, ri.min_phi);
        for tau in [0.4, 0.2, 0.1, 0.05, 0.02, 0.01] {
            Census65::reset();
            let ml = sibling(&m, None, Some(BleedLimiter::with_tau(0.80, BMAX, Some(tau))));
            let tl = march(&ml, &fl, &ramp, None).0;
            let c = Census65::take();
            let rl = read(&ml, &fl, &tl);
            let bc: Vec<(f64, f64)> = tl.iter().map(|p| match p.extra { PointExtra::Valve { b, b_cmd } => (b, b_cmd), _ => panic!() }).collect();
            let mut ic = 0.0; for i in 1..tl.len() { ic += 0.5*(tl[i].s-tl[i-1].s)*(bc[i].1+bc[i-1].1); }
            println!("P4a tau={tau}: n={} bint={:.6} dbint={:.4e} tau*b0={:.4e} drift={:.3e} bend={:.2e} clamp={} minphi={:.9}",
                     rl.npts, rl.b_int, rl.b_int-ri.b_int, tau*b0, ic-ri.b_int, bc[bc.len()-1].0, c.clamp_hits, rl.min_phi);
        }
        // M_i(phi_lim) scan, stators on, lagged phi valve
        for (shape, lp, hp) in shapes.clone() {
            let m = bare65(lp, hp);
            for r in [0.25, 0.5, 1.0] {
                let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds };
                let shut = read(&sibling(&m, Some(&stat()), None), &fl, &march(&sibling(&m, Some(&stat()), None), &fl, &ramp, None).0);
                for tau in [None, Some(0.01), Some(0.05), Some(0.2)] {
                    let mut xs = vec![shut.min_phi];
                    for k in 0..=14 { xs.push(0.70 + 0.01 * k as f64); }
                    xs.push(1.5);
                    let mut prev = f64::NEG_INFINITY; let mut mono = true; let mut out = String::new();
                    let mut clamps = 0;
                    for x in xs {
                        Census65::reset();
                        let mm = sibling(&m, Some(&stat()), Some(BleedLimiter::with_tau(x, BMAX, tau)));
                        let rr = read(&mm, &fl, &march(&mm, &fl, &ramp, None).0);
                        clamps += Census65::take().clamp_hits;
                        if rr.m_i < prev { mono = false; }
                        prev = rr.m_i;
                        out += &format!(" {x:.3}:{:.4}", rr.m_i);
                    }
                    println!("P4c {shape} r={r} tau={tau:?} shutMi={:.4} monotone={mono} clamps={clamps} |{out}", shut.m_i);
                }
            }
        }
    }
}

