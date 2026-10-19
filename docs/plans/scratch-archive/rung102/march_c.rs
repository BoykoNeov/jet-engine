// Part C probe: the Controls slam with the afterburner fitted / lit at a chosen time (a light-off ramp).
use std::time::Instant as Clock;
use turbojet::afterburner::{self as abm, Afterburner, LpAb, Throat};
use turbojet::sandbox_controls::{build, stop_reason, ControlsSettings, ControlsSolver, DS};
use turbojet::fuel_transient::FuelPoint;
use turbojet::two_spool::Spool;

fn w_at(s: f64, s_on: f64, tau: f64) -> f64 {
    if s <= s_on { 0.0 } else if tau <= 0.0 || s >= s_on + tau { 1.0 } else { (s - s_on) / tau }
}

fn margins(sv: &ControlsSolver, p: &FuelPoint) -> (f64, f64) {
    let (_, sl) = sv.stall(Spool::Lp, p.nu_lp, p.nu_hp);
    let (_, sh) = sv.stall(Spool::Hp, p.nu_lp, p.nu_hp);
    (p.phi_lp / sl - 1.0, p.phi_hp / sh - 1.0)
}

fn census() -> (u64, u64, u64) {
    (abm::LP_CALLS.with(|c| c.replace(0)), abm::LP_TRIALS.with(|c| c.replace(0)), abm::EXIT_CALLS.with(|c| c.replace(0)))
}

struct Case { label: String, fitted: bool, throat: Throat, tt7: f64, s_on: f64, tau: f64 }

fn run(s: &ControlsSettings, c: &Case, base_pts: Option<&[FuelPoint]>) -> Vec<FuelPoint> {
    abm::set_lp_ab(None);
    let sv0 = build(s).expect("builds");
    let x = LpAb::capture(&sv0.core.fuel.inner.inner.base, Afterburner { tt7: c.tt7, ..Afterburner::default() }, c.throat);
    abm::set_lp_ab(if c.fitted { Some(x) } else { None });
    let t0 = Clock::now();
    let sv = match build(s) { Ok(v) => v, Err(e) => { println!("{}: build refused: {:?}", c.label, e); return vec![]; } };
    let sched = s.schedule(sv.mf_lo, sv.mf_hi);
    let (s_on, tau) = (c.s_on, c.tau);
    let wrapped = |t: f64| { abm::set_lp_w(w_at(t, s_on, tau)); sched(t) };
    let nu0 = (sv.start.nu_lp, sv.start.nu_hp);
    let pts = sv.core.fuel.integrate_fuel(&sv.flight, wrapped, nu0, s.s_end(), DS, &sv.limiters());
    let ms = t0.elapsed().as_secs_f64() * 1e3;
    let (calls, trials, exits) = census();
    println!("== {}  ({} points of {}, {:.0} ms; LP calls {calls}, burn trials {trials}, exits {exits})",
             c.label, pts.len(), s.expected_points(), ms);
    if pts.len() < s.expected_points() {
        abm::set_lp_w(pts.last().map_or(0.0, |p| w_at(p.s, s_on, tau)));
        if let Some(st) = stop_reason(&sv, &pts) {
            println!("   STOPPED at s={:.2}: {:?}", st.s_last, st.failure.map(|f| f.message));
        }
    }
    if let Some(b) = base_pts {
        let n = b.len().min(pts.len());
        let mut worst = 0.0f64;
        for i in 0..n { worst = worst.max((b[i].nu_lp - pts[i].nu_lp).abs()).max((b[i].nu_hp - pts[i].nu_hp).abs())
            .max(((b[i].sp_thrust - pts[i].sp_thrust) / b[i].sp_thrust).abs()); }
        println!("   vs reference run: worst |d nu| / rel thrust over {n} points = {worst:.3e}");
    }
    let mut min_lp = (f64::INFINITY, 0.0); let mut min_hp = (f64::INFINITY, 0.0);
    for p in &pts { let (ml, mh) = margins(&sv, p); if ml < min_lp.0 { min_lp = (ml, p.s) } if mh < min_hp.0 { min_hp = (mh, p.s) } }
    println!("   min LP margin {:.4} at s={:.2}; min HP margin {:.4} at s={:.2}", min_lp.0, min_lp.1, min_hp.0, min_hp.1);
    println!("   {:>5} {:>4} {:>8} {:>8} {:>7} {:>8} {:>8} {:>9} {:>6} {:>8}", "s", "w", "nu_lp", "nu_hp", "Tt4", "sm_lp", "sm_hp", "thrust", "br", "Tt5");
    for p in &pts {
        let k = (p.s / DS).round() as i64;
        if k % 10 != 0 && (p.s - s_on).abs() > 0.15 { continue; }
        abm::set_lp_w(w_at(p.s, s_on, tau));
        let r = sv.read(p);
        let (ml, mh) = margins(&sv, p);
        match r {
            Ok(r) => println!("   {:5.2} {:4.2} {:8.5} {:8.5} {:7.1} {:8.4} {:8.4} {:9.2} {:>6} {:8.1}", p.s, w_at(p.s, s_on, tau), p.nu_lp, p.nu_hp, p.tt4, ml, mh, r.thrust, format!("{:?}", p.branch), r.tt5),
            Err(e) => println!("   {:5.2} read failed: {}", p.s, e.0),
        }
    }
    abm::set_lp_ab(None);
    census();
    pts
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut s = ControlsSettings::defaults();
    s.settle = 4.0;
    if args.len() > 2 { s.from = args[1].parse().unwrap(); s.to = args[2].parse().unwrap(); }
    println!("slam {} -> {} K, ramp {}, settle {} (HP tau), DS {}", s.from, s.to, s.ramp, s.settle, DS);
    let shipped = run(&s, &Case { label: "not fitted".into(), fitted: false, throat: Throat::Held(1.0), tt7: 2000.0, s_on: 99.0, tau: 0.1 }, None);
    abm::set_lp_ab(None);
    let sv = build(&s).unwrap();
    let m = sv.march();
    let same = m.len() == shipped.len() && m.iter().zip(&shipped).all(|(a, b)| a.nu_lp.to_bits() == b.nu_lp.to_bits() && a.nu_hp.to_bits() == b.nu_hp.to_bits() && a.sp_thrust.to_bits() == b.sp_thrust.to_bits());
    println!("   probe path vs the shipped march: bit-identical = {same}");
    let unlit = run(&s, &Case { label: "fitted, never lit".into(), fitted: true, throat: Throat::Held(1.0), tt7: 2000.0, s_on: 99.0, tau: 0.1 }, Some(&shipped));
    for &(s_on, when) in &[(0.25, "mid-ramp"), (3.0, "settled")] {
        for &tt7 in &[1800.0, 2000.0] {
            for th in [Throat::Held(1.0), Throat::Held(1.15), Throat::Match] {
                let label = format!("lit {when} s_on={s_on} Tt7={tt7} throat={th:?}");
                run(&s, &Case { label, fitted: true, throat: th, tt7, s_on, tau: 0.1 }, Some(&unlit));
            }
        }
    }
}
