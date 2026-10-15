// Rung 87 pre-anchor probes (scratch, not in the repo). Rig = rung 86's.
use std::panic::{catch_unwind, AssertUnwindSafe};
use turbojet::bleed_transient::{build_scheduled_bleed, BleedSchedule, LeverArm};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::FuelPoint;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{build_limited_bleed, BleedLimiter};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorArm, StatorSchedule};
use turbojet::three_levers::{march, ArmedLimiter, Limiter};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const REAL: TwoSpoolLosses = TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true };
fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }
fn cpg() -> Gas { Gas::new(GasSpec { gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
    gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0, hpr: 42.8e6, ..GasSpec::default() }) }
fn lp_map() -> ComponentMap { ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }.with_phi_surge(0.55) }
fn hp_map() -> ComponentMap { ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }.with_phi_surge(0.55) }
fn tilt_map() -> ComponentMap { ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..ComponentMap::flat() }.with_phi_surge(0.55) }
fn design() -> TwoSpoolEngine { build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL) }
fn lt(lp: ComponentMap, hp: ComponentMap) -> ScheduledStatorCore {
    match build_limited_bleed(design(), flight(), 1.0, Some(lp), Some(hp), 1.0, &LeverArm::default()) {
        ScheduledStatorTransient::Full(c) => c, _ => unreachable!() } }
fn bt(lp: ComponentMap, hp: ComponentMap) -> ScheduledStatorCore {
    match build_scheduled_bleed(design(), flight(), 1.0, Some(lp), Some(hp), 1.0, &LeverArm::default()) {
        ScheduledStatorTransient::Full(c) => c, _ => unreachable!() } }

const BMAX: f64 = 0.10;
fn stat() -> LeverArm { LeverArm::stator(StatorArm::scheduled_lp(StatorSchedule::new(0.20, 0.65))) }

/// Per-run summary, every bleed number re-solved through b_at_point.
fn run(m: &ScheduledStatorCore, ramp: &Ramp, lim: Option<&ArmedLimiter>) -> Option<(Vec<FuelPoint>, Vec<f64>, Vec<f64>)> {
    let fl = flight();
    let r = catch_unwind(AssertUnwindSafe(|| march(m, &fl, ramp, lim)));
    let (traj, _) = match r { Ok(t) => t, Err(_) => return None };
    let (tt2, _, _) = m.fuel.inner.inlet(&fl);
    let b: Vec<f64> = traj.iter().map(|p| m.b_at_point(&fl, p)).collect();
    let v: Vec<f64> = traj.iter().map(|p| m.v_of(Spool::Lp, p.nu_lp, p.nu_hp, Some(tt2))).collect();
    Some((traj, b, v))
}

fn summary(tag: &str, m: &ScheduledStatorCore, ramp: &Ramp, lim: Option<&ArmedLimiter>, phi_lim: Option<f64>) {
    let t_c = m.design_map(Spool::Lp).tan_beta1_crit();
    match run(m, ramp, lim) {
        None => println!("{tag}: PANIC"),
        Some((traj, b, v)) if traj.is_empty() => { let _ = (b, v); println!("{tag}: EMPTY run") }
        Some((traj, b, v)) => {
            let n = traj.len();
            let mut minphi = f64::INFINITY; let mut mi = f64::INFINITY; let (mut jphi, mut jmi) = (0, 0);
            let (mut bint, mut bpk, mut thr_int) = (0.0, 0.0f64, 0.0);
            let (mut shut, mut ride, mut sat) = (0, 0, 0);
            let mut ride_mi_dev: f64 = 0.0; // max |M_i - (T_c - 1/phi_lim + v)| over riding points
            for i in 0..n {
                let p = &traj[i];
                if p.phi_lp < minphi { minphi = p.phi_lp; jphi = i; }
                let m_i = t_c - 1.0 / p.phi_lp + v[i];
                if m_i < mi { mi = m_i; jmi = i; }
                bpk = bpk.max(b[i]);
                if b[i] <= 0.0 { shut += 1 } else if b[i] >= BMAX { sat += 1 } else {
                    ride += 1;
                    if let Some(pl) = phi_lim { ride_mi_dev = ride_mi_dev.max((m_i - (t_c - 1.0 / pl + v[i])).abs()); }
                }
                if i > 0 {
                    let h = p.s - traj[i - 1].s;
                    bint += 0.5 * h * (b[i] + b[i - 1]);
                    thr_int += 0.5 * h * (p.sp_thrust * p.mdot_air + traj[i - 1].sp_thrust * traj[i - 1].mdot_air);
                }
            }
            let last = &traj[n - 1];
            let rm: f64 = traj.iter().map(|p| p.mf_sched - p.mf).fold(0.0, f64::max);
            println!("{tag}: n={n} minphi={minphi:.6}@s={:.3}(v={:.4},b={:.4}) gridMi={mi:.6}@s={:.3}(v={:.4},phi={:.5},b={:.4}) bint={bint:.5e} bpk={bpk:.4} nuL_end={:.5} nuH_end={:.5} F_end={:.2} Fint={thr_int:.3} tt4pk={:.1} shut/ride/sat={shut}/{ride}/{sat} rideMiDev={ride_mi_dev:.1e} maxcut={rm:.3e}",
                traj[jphi].s, v[jphi], b[jphi], traj[jmi].s, v[jmi], traj[jmi].phi_lp, b[jmi],
                last.nu_lp, last.nu_hp, last.sp_thrust * last.mdot_air,
                traj.iter().map(|p| p.tt4).fold(0.0, f64::max));
        }
    }
}

fn main() {
    let which: String = std::env::args().nth(1).unwrap_or_default();
    let fl = flight();
    // ---------------- P1: builder trap
    if which.is_empty() || which == "p1" {
        let lim = BleedLimiter::new(0.55 * 1.38, BMAX);
        let m = lt(lp_map(), hp_map());
        let a = m.at_lever(&LeverArm::floored(lim));
        let ab = m.at_lever(&LeverArm::merged(&stat(), &LeverArm::floored(lim)));
        println!("P1 limited: floored lim={:?} armed={} | merged lim={:?} armed={} sched={}",
            a.fuel.inner.lever.lim.is_some(), a.armed_bleed(), ab.fuel.inner.lever.lim.is_some(), ab.armed_bleed(), ab.arming().is_scheduled());
        let m62 = bt(lp_map(), hp_map());
        let r = catch_unwind(AssertUnwindSafe(|| {
            let a62 = m62.at_lever(&LeverArm::floored(lim));
            (a62.fuel.inner.lever.lim.is_some(), a62.armed_bleed())
        }));
        println!("P1 rung-62 builder with a floored arm: {:?}", r.map_err(|_| "PANIC"));
    }
    for (shape, lp, hp) in [("shaped", lp_map(), hp_map()), ("tilted", tilt_map(), tilt_map())] {
        let m = lt(lp, hp);
        for r in [0.25, 0.5, 1.0] {
            let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds: 0.005 };
            // ---------------- P2: ceilings (valve shut / schedule / fully open), stator off & on
            if which.is_empty() || which == "p2" {
                for (sname, s) in [("S0", LeverArm::default()), ("S1", stat())] {
                    for (bname, bl) in [("shut", LeverArm::default()), ("sched", LeverArm::scheduled(BleedSchedule::new(BMAX, 0.65))),
                                        ("full", LeverArm::constant(BMAX))] {
                        let sib = m.at_lever(&LeverArm::merged(&s, &bl));
                        summary(&format!("P2 {shape} r={r} {sname} {bname}"), &sib, &ramp, None, None);
                    }
                }
            }
            // ---------------- P3: the reacting valve at set points, stator off & on
            if which.is_empty() || which == "p3" {
                for sm in [0.30, 0.34, 0.38, 0.42, 0.46] {
                    let phi_lim = 0.55 * (1.0 + sm);
                    let fl_arm = LeverArm::floored(BleedLimiter::new(phi_lim, BMAX));
                    for (sname, s) in [("S0", LeverArm::default()), ("S1", stat())] {
                        let sib = m.at_lever(&LeverArm::merged(&s, &fl_arm));
                        assert!(sib.fuel.inner.lever.lim.is_some());
                        summary(&format!("P3 {shape} r={r} sm={sm} phi_lim={phi_lim:.4} {sname}"), &sib, &ramp, None, Some(phi_lim));
                    }
                }
            }
            // ---------------- P4: the valve beside a fuel phi floor (two loops on one variable)
            if which == "p4" {
                let bare = m.at_lever(&LeverArm::default());
                for (vsm, fsm) in [(0.38, 0.30), (0.38, 0.38), (0.38, 0.46), (0.46, 0.38)] {
                    let phi_lim = 0.55 * (1.0 + vsm);
                    let fl_arm = LeverArm::floored(BleedLimiter::new(phi_lim, BMAX));
                    let lim = ArmedLimiter::on(&bare, &fl, &ramp, Spool::Lp, Limiter::PhiFloor { sm: fsm });
                    for (sname, s) in [("S0", LeverArm::default()), ("S1", stat())] {
                        let sib = m.at_lever(&LeverArm::merged(&s, &fl_arm));
                        summary(&format!("P4 {shape} r={r} valve sm={vsm} fuel sm={fsm} {sname}"), &sib, &ramp, Some(&lim), Some(phi_lim));
                    }
                }
            }
        }
    }
}
