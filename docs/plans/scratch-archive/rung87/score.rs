// Rung 87 scoring (scratch). Prints every cell of anchor § 3 at the step given (arg: ds).
use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{build_limited_bleed, BleedLimiter};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorArm, StatorSchedule};
use turbojet::three_levers::{ArmedLimiter, Limiter};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::valve_reference::{incidence_valve, m_lim, matched_phi, run, sibling, ValveRun};

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
const BMAX: f64 = 0.10;

fn cls(r: &ValveRun, m: f64) -> &'static str {
    if r.riding == 0 && r.saturated == 0 { "D" } else if r.saturated > 0 && r.m_i < m - 1e-9 { "S" } else { "R" }
}
fn bill(r: &ValveRun, shut: &ValveRun) -> String {
    format!("b_int={:.5e} b_pk={:.4} dFint={:+.4} dnuL={:+.2e} n={} pin={:.0e} m_i={:.7}@{:.3}",
        r.b_int, r.b_peak, r.thrust_int - shut.thrust_int, r.nu_lp_end - shut.nu_lp_end, r.npts, r.pin_dev, r.m_i, r.s_at_m_i)
}

fn main() {
    let ds: f64 = std::env::args().nth(1).map(|s| s.parse().unwrap()).unwrap_or(0.005);
    let which: String = std::env::args().nth(2).unwrap_or_default();
    let fl = flight();
    let stat = LeverArm::stator(StatorArm::scheduled_lp(StatorSchedule::new(0.20, 0.65)));
    let sms = [0.30, 0.34, 0.38, 0.42, 0.46, 0.50, 0.54, 0.58];
    for (shape, lp, hp) in [("shaped", lp_map(), hp_map()), ("tilted", tilt_map(), tilt_map())] {
        let bare = lt(lp, hp);
        let t_c = bare.design_map(Spool::Lp).tan_beta1_crit();
        for r in [0.25, 0.5, 1.0] {
            let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds };
            let shut = [run(&sibling(&bare, None, None), &fl, &ramp, None), run(&sibling(&bare, Some(&stat), None), &fl, &ramp, None)];
            println!("== {shape} r={r} ds={ds} shut S0 m_i={:.7} S1 m_i={:.7}", shut[0].m_i, shut[1].m_i);
            for &sm in &sms {
                let phi_lim = 0.55 * (1.0 + sm);
                let ml = m_lim(t_c, phi_lim);
                let iv = incidence_valve(phi_lim, BMAX);
                let inc = [run(&sibling(&bare, None, Some(iv)), &fl, &ramp, None), run(&sibling(&bare, Some(&stat), Some(iv)), &fl, &ramp, None)];
                if which.is_empty() || which == "h" {
                    println!("  sm={sm:.2} m_lim={ml:.5} INC S0 {} {} | S1 {} {}", cls(&inc[0], ml), bill(&inc[0], &shut[0]), cls(&inc[1], ml), bill(&inc[1], &shut[1]));
                    if cls(&inc[1], ml) == "R" {
                        match matched_phi(&bare, &fl, &ramp, Some(&stat), BMAX, inc[1].m_i, 1e-10) {
                            None => println!("      MATCH: no bracket"),
                            Some((phi, pr)) => println!("      MATCH phi*={phi:.7} (sm*={:.4}) {}  | H1 {} H2 {}  ratio_b={:.3}",
                                phi / 0.55 - 1.0, bill(&pr, &shut[1]),
                                if pr.b_int > inc[1].b_int { "HIT" } else { "MISS" },
                                if pr.thrust_int < inc[1].thrust_int { "HIT" } else { "MISS" },
                                pr.b_int / inc[1].b_int),
                        }
                    }
                }
                if which.is_empty() || which == "f" {
                    let b0 = sibling(&bare, None, None);
                    let phif = ArmedLimiter::on(&b0, &fl, &ramp, Spool::Lp, Limiter::PhiFloor { sm });
                    let incf = ArmedLimiter::on(&b0, &fl, &ramp, Spool::Lp, Limiter::IncidenceFloor { sm });
                    let s1i = sibling(&bare, Some(&stat), Some(iv));
                    let a = run(&s1i, &fl, &ramp, Some(&phif));
                    let b = run(&s1i, &fl, &ramp, Some(&incf));
                    let pv = sibling(&bare, Some(&stat), Some(BleedLimiter::new(phi_lim, BMAX)));
                    let c = run(&pv, &fl, &ramp, Some(&incf));
                    let pvr = run(&pv, &fl, &ramp, None);
                    println!("  sm={sm:.2} F incV+phiF: n={} withheld={:.4e} Fint={:.3} | incV+incF: n={} withheld={:.4e} Fint={:.3} | phiV+incF: n={} withheld={:.4e} cutmax={:.3e} (phiV sat pts={})",
                        a.npts, a.fuel_withheld, a.thrust_int, b.npts, b.fuel_withheld, b.thrust_int, c.npts, c.fuel_withheld, c.fuel_cut_max, pvr.saturated);
                }
            }
        }
    }
}
