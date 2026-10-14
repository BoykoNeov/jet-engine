use turbojet::bleed_transient::{build_scheduled_bleed, BleedSchedule, LeverArm};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::AsymmetricLag;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorArm, StatorSchedule};
use turbojet::three_levers::{cube, Cube, Limiter};
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
fn bt(lp: ComponentMap, hp: ComponentMap) -> ScheduledStatorCore {
    match build_scheduled_bleed(design(), flight(), 1.0, Some(lp), Some(hp), 1.0, &LeverArm::default()) {
        ScheduledStatorTransient::Full(c) => c, _ => unreachable!() } }

fn line(tag: &str, c: &Cube) {
    let r = |k: usize| if c.reached[k] { "R" } else { "." };
    let n: Vec<usize> = c.cells.iter().map(|x| x.read.npts).collect();
    println!("{tag} T={:+.6e} (bf {:+.3e} sf {:+.3e}) Isb=[{:+.6e},{:+.6e}] Ibf=[{:+.6e},{:+.6e}] Isf=[{:+.6e},{:+.6e}] reached={}{}{}{} pred={:+.6e} npts={:?}",
        c.t, c.t_bf - c.t, c.t_sf - c.t, c.i_sb[0], c.i_sb[1], c.i_bf[0], c.i_bf[1], c.i_sf[0], c.i_sf[1],
        r(0), r(1), r(2), r(3), c.predicted, n);
    // per cell detail
    for (k, (s, b)) in [(0, 0), (0, 1), (1, 0), (1, 1)].into_iter().enumerate() {
        let (fr, lg) = (c.at(s, b, 0), c.at(s, b, 1));
        let _ = k;
        println!("    S{s}B{b} free m_i={:.7} s={:.4} tt4stencil={:.1} minphi={:.6} | leg m_i={:.7} s={:.4} phi@min={:.7} v={:.6} removed={:.4e} sched={:.4e} frac={:.5e} s_eng={:.4} npts={}",
            fr.read.m_i, fr.read.s, fr.tt4_to_stencil, fr.read.min_phi, lg.read.m_i, lg.read.s, lg.phi_at_min, lg.read.v,
            lg.read.fuel_removed, lg.fuel_scheduled, c.withheld_fraction(s, b), lg.read.s_eng, lg.read.npts);
    }
    if let Some((led, tl)) = c.incidence_ledger() {
        println!("    ledger={:?} T_ledger={:+.6e} dT={:+.3e}", led, tl, c.t - tl);
    }
    if let Some(sp) = c.set_point { println!("    set_point={sp:.7} T_c={:.7}", c.t_c); }
}

fn main() {
    let bled = LeverArm::scheduled(BleedSchedule::new(0.10, 0.65));
    let stat = LeverArm::stator(StatorArm::scheduled_lp(StatorSchedule::new(0.20, 0.65)));
    let which: String = std::env::args().nth(1).unwrap_or_default();
    let mut lims: Vec<Limiter> = vec![];
    if which.is_empty() || which == "a" {
        lims.push(Limiter::Accel { margin: 0.25 });
        lims.push(Limiter::AccelLag { margin: 0.25, lag: AsymmetricLag::new(0.02, 0.10) });
    }
    if which.is_empty() || which == "b" {
        lims.push(Limiter::Topping { tt4_max: 1480.0 });
        lims.push(Limiter::ToppingLag { tt4_max: 1480.0, tau_gov: 0.2 });
    }
    if which.is_empty() || which == "c" {
        for sm in [0.30, 0.34, 0.38, 0.42, 0.46, 0.50] { lims.push(Limiter::IncidenceFloor { sm }); }
        for sm in [0.30, 0.34, 0.38, 0.42, 0.46, 0.50] { lims.push(Limiter::PhiFloor { sm }); }
    }
    for (shape, lp, hp) in [("shaped", lp_map(), hp_map()), ("tilted", tilt_map(), tilt_map())] {
        let m = bt(lp, hp);
        for r in [0.25, 0.5, 1.0] {
            for lim in &lims {
                for ds in [0.005, 0.0025] {
                    let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds };
                    let c = cube(&m, &flight(), &ramp, Spool::Lp, &stat, &bled, *lim);
                    line(&format!("{shape} r={r} ds={ds} {:?}", lim), &c);
                }
            }
        }
    }
}
